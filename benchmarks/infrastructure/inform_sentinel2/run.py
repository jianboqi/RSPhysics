"""Build and benchmark an INFORM crown LAI lookup table on a Sentinel-2B L2A tile."""

import argparse
from concurrent.futures import ThreadPoolExecutor
from itertools import product
import json
from pathlib import Path
from time import perf_counter
import xml.etree.ElementTree as ET

import numpy as np
from osgeo import gdal
import rsphysics as rsp
from rsphysics import lut


PROJECT = Path(__file__).resolve().parents[3]
SENSOR_ID = "sentinel-2b.msi"
LUT_FILENAME = "inform_lut.npz"
BANDS = ("B03", "B04", "B05", "B07", "B8A", "B11")
SENSOR_BANDS = ("B3", "B4", "B5", "B7", "B8A", "B11")
FLAGS = {
    1: "SCL 0 or 1, or invalid source DN in any input band",
    2: "SCL class other than 4 (vegetation)",
    4: "WorldCover class other than 10 (tree cover)",
    8: "Best spectral RMSE exceeds 0.04 reflectance",
    16: "Best crown LAI lies in the outer 5% of its LUT interval",
    32: "Top-16 crown LAI population standard deviation exceeds 1.5",
    64: "WorldCover has no data",
    128: "A selected surface reflectance lies outside [0,1]",
}
RANGES = {
    "crown_lai": (0.5, 8.0),
    "understory_lai": (0.05, 2.0),
    "stem_density_per_ha": (250, 1500),
    "tree_height_m": (7, 28),
    "crown_diameter_m": (2.5, 7.5),
    "crown_mean_leaf_angle_deg": (35, 65),
    "soil_brightness": (0.7, 1.3),
}
TOP_K = 16
MAX_RMSE = 0.04
SPREAD_LIMIT = 1.5
NODATA = -9999.0
BLOCK = 512


def write_json(path, data):
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def metadata(scene):
    product_xml = ET.parse(scene / "product-metadata.xml")
    tile_xml = ET.parse(scene / "granule-metadata.xml")
    if product_xml.findtext(".//SPACECRAFT_NAME") != "Sentinel-2B":
        raise ValueError("This benchmark uses Sentinel-2B response functions")
    ids = {item.attrib["physicalBand"]: item.attrib["bandId"] for item in product_xml.iter("Spectral_Information")}
    offsets = {item.attrib["band_id"]: float(item.text) for item in product_xml.iter("BOA_ADD_OFFSET")}
    scale = float(product_xml.find(".//BOA_QUANTIFICATION_VALUE").text)
    used_ids = [ids[name] for name in SENSOR_BANDS]
    sun = tile_xml.find(".//Mean_Sun_Angle")
    views = {item.attrib["bandId"]: item for item in tile_xml.iter("Mean_Viewing_Incidence_Angle")}
    sun_zenith, sun_azimuth = (float(sun.findtext(name)) for name in ("ZENITH_ANGLE", "AZIMUTH_ANGLE"))
    view_zenith = float(np.mean([float(views[i].findtext("ZENITH_ANGLE")) for i in used_ids]))
    view_azimuth = float(np.angle(np.mean(np.exp(1j * np.deg2rad(
        [float(views[i].findtext("AZIMUTH_ANGLE")) for i in used_ids]
    ))), deg=True) % 360)
    geometry = {
        "solar_zenith_deg": sun_zenith,
        "view_zenith_deg": view_zenith,
        "relative_azimuth_deg": abs((view_azimuth - sun_azimuth + 180) % 360 - 180),
    }
    return geometry, [offsets[i] if offsets else 0.0 for i in used_ids], scale, {
        "sun_azimuth_deg": sun_azimuth, "view_azimuth_deg": view_azimuth,
        "processing_baseline": product_xml.find(".//PROCESSING_BASELINE").text,
    }


def build_lut(scene, output, workers, sensor_cache):
    start = perf_counter()
    geometry, _, _, _ = metadata(scene)
    sensor = rsp.sensors.load(SENSOR_ID, cache_dir=sensor_cache).select(SENSOR_BANDS)
    channels = tuple(lut.Channel(name, "BOA directional reflectance") for name in BANDS)
    context = {"scene": scene.name, "sensor": sensor.id, **geometry}
    soil = np.loadtxt(PROJECT / "crates/models/vegetation/prosail-d/data/soil.tsv", skiprows=1)
    wavelength_nm, dry, wet = soil[:, 0], soil[:, 3], soil[:, 4]
    # The band integration needs only the nonzero support of these six bands.
    support = [band.wavelengths_nm[band.response > 0] for band in sensor.bands]
    span = (wavelength_nm >= min(x[0] for x in support) - 2) & (wavelength_nm <= max(x[-1] for x in support) + 2)
    wavelength_nm, dry, wet = wavelength_nm[span], dry[span], wet[span]
    fixed = {**geometry, "diffuse_fraction": 0.1}
    columns = rsp.parameters_for("vegetation.inform")
    defaults = rsp.describe("vegetation.inform")["input_schema"]["properties"]
    base = np.array([fixed.get(name, defaults[name]["default"]) for name in columns])
    indices = [columns.index(name) for name in RANGES]
    groups = list(product((30., 50., 70., 90.), (0.30, 0.70)))

    def simulate(item):
        group, (chlorophyll, dry_fraction) = item
        samples = lut.sample_lhs(RANGES, 1024, seed=240923 + group)
        matrix = np.broadcast_to(base, (len(samples.values), len(base))).copy()
        matrix[:, indices] = samples.values
        spectrum = {
            "wavelengths_nm": wavelength_nm,
            "soil_reflectance": dry_fraction * dry + (1. - dry_fraction) * wet,
        }
        crown = {"n": 1.8, "chlorophyll": chlorophyll, "water": 0.015, "dry_matter": 0.01}
        simulated = []
        for offset in range(0, len(matrix), 128):
            output = rsp.vegetation.inform_batch(matrix[offset:offset + 128], spectra=spectrum,
                                                crown_leaf=crown)["result"]
            bands = sensor.convolve(output["wavelengths_nm"], output["reflectance"])
            simulated.append(bands["values"])
        print(f"LUT group {group + 1}/{len(groups)} ready", flush=True)
        parameters = np.column_stack((samples.values, np.full((len(matrix), 1), chlorophyll),
                                      np.full((len(matrix), 1), dry_fraction)))
        return parameters, np.concatenate(simulated)

    with ThreadPoolExecutor(max_workers=workers) as pool:
        groups_result = list(pool.map(simulate, enumerate(groups)))
    values, spectra = (np.concatenate(rows) for rows in zip(*groups_result))
    names = (*RANGES, "crown_leaf.chlorophyll", "soil_dry_fraction")
    design = {"method": "latin_hypercube_by_optics_group", "seed_base": 240923,
              "bounds": [*RANGES.values(), [30, 90], [0.3, 0.7]], "group_rows": 1024}
    samples = lut.Samples(names, values, design)
    table = lut.LookupTable(samples, spectra, channels, context, {
        "model": rsp.describe("vegetation.inform")["metadata"],
        "fixed": fixed, "sensor_version": sensor.version,
        "soil_data": "RSPhysics PROSAIL-D dry and wet soil endmembers",
        "crown_leaf_fixed": {"n": 1.8, "water": 0.015, "dry_matter": 0.01},
    })
    table.save(output / LUT_FILENAME)
    elapsed = perf_counter() - start
    write_json(output / "lut_timing.json", {"wall_seconds": elapsed, "rows": len(values), "workers": workers})
    print(f"{len(values):,}-row LUT built in {elapsed:.2f}s", flush=True)


def forest_grid(reference, tiles, output):
    start = perf_counter()
    path = output / "worldcover_2021_aligned_20m.tif"
    temp = output / "worldcover_2021_aligned_20m.part.tif"
    mosaic = gdal.BuildVRT("", [str(tile) for tile in tiles], strict=True)
    x, sx, _, y, _, sy = reference.GetGeoTransform()
    width, height = reference.RasterXSize, reference.RasterYSize
    ds = gdal.Warp(
        str(temp), mosaic, outputBounds=(x, y + sy * height, x + sx * width, y),
        width=width, height=height, dstSRS=reference.GetProjection(), srcNodata=0,
        dstNodata=0, outputType=gdal.GDT_Byte, resampleAlg="mode",
        multithread=True, warpOptions=["NUM_THREADS=4"],
        creationOptions=["TILED=YES", "COMPRESS=DEFLATE"],
    )
    ds.Close()
    mosaic.Close()
    temp.replace(path)
    return gdal.Open(str(path)), perf_counter() - start


def aligned_bands(scene, reference):
    x, sx, rx, y, ry, sy = reference.GetGeoTransform()
    crs = reference.GetSpatialRef()
    if (sx, sy, rx, ry) != (20, -20, 0, 0) or not crs.IsProjected() or crs.GetLinearUnits() != 1:
        raise ValueError("The SCL raster must be on an unrotated 20 m projected grid")
    width, height = reference.RasterXSize, reference.RasterYSize
    datasets = []
    for band in BANDS:
        ds = gdal.Open(str(scene / "original" / f"{band}.tif"))
        ratio = ds.RasterXSize // width
        if (ratio not in (1, 2) or ds.RasterXSize != width * ratio or ds.RasterYSize != height * ratio
                or ds.GetGeoTransform() != (x, sx / ratio, 0, y, 0, sy / ratio)
                or not ds.GetSpatialRef().IsSame(reference.GetSpatialRef())):
            raise ValueError(f"Source grid for {band} is not aligned")
        datasets.append((ds, ratio))
    return datasets


def reflectance_window(datasets, offsets, scale, x, y, w, h):
    cube = np.empty((h, w, len(datasets)), dtype=np.float32)
    valid = np.ones((h, w), dtype=bool)
    outside = np.zeros((h, w), dtype=bool)
    for i, ((ds, ratio), offset) in enumerate(zip(datasets, offsets)):
        dn = ds.ReadAsArray(x * ratio, y * ratio, w * ratio, h * ratio)
        if ratio == 2:
            values = dn.reshape(h, ratio, w, ratio)
            valid &= (values != 0).all(axis=(1, 3))
            mean_dn = values.mean(axis=(1, 3), dtype=np.float32)
        else:
            valid &= dn != 0
            mean_dn = dn.astype(np.float32)
        rho = (mean_dn + offset) / scale
        cube[:, :, i] = rho
        outside |= (rho < 0.) | (rho > 1.)
    return cube, valid, outside


def new_outputs(reference, scene, output):
    driver = gdal.GetDriverByName("GTiff")
    width, height = reference.RasterXSize, reference.RasterYSize
    common = ["TILED=YES", "COMPRESS=DEFLATE", "NUM_THREADS=2", "BIGTIFF=IF_SAFER"]
    lai = driver.Create(str(output / "inform_lai_20m.part.tif"), width, height, 4, gdal.GDT_Float32,
                        common + ["PREDICTOR=3", "INTERLEAVE=BAND"])
    quality = driver.Create(str(output / "inform_quality_20m.part.tif"), width, height, 1, gdal.GDT_Byte,
                            common + ["PREDICTOR=2"])
    for ds in (lai, quality):
        ds.SetGeoTransform(reference.GetGeoTransform())
        ds.SetProjection(reference.GetProjection())
        ds.SetMetadataItem("SCENE", scene.name)
        ds.SetMetadataItem("MODEL", "RSPhysics vegetation.inform")
    labels = ["Best-fit single-crown LAI", "Top-16 mean single-crown LAI",
              "Top-16 population standard deviation of single-crown LAI", "Best spectral reflectance RMSE"]
    for idx, label in enumerate(labels, 1):
        band = lai.GetRasterBand(idx)
        band.SetDescription(label)
        band.SetUnitType("m2/m2" if idx <= 3 else "reflectance")
        band.SetNoDataValue(NODATA)
    quality.GetRasterBand(1).SetDescription("INFORM quality flags (bit mask)")
    for flag, description in FLAGS.items():
        quality.SetMetadataItem(f"BIT_{flag}", description)
    return lai, quality


def image_windows(width, height):
    return [(x, y, min(BLOCK, width - x), min(BLOCK, height - y))
            for y in range(0, height, BLOCK) for x in range(0, width, BLOCK)]


def run(scene, output, workers, worldcover_tiles):
    gdal.UseExceptions()
    gdal.SetCacheMax(512 * 1024 * 1024)
    start = perf_counter()
    geometry, offsets, scale, geometry_info = metadata(scene)
    context = {"scene": scene.name, "sensor": SENSOR_ID, **geometry}
    table = lut.LookupTable.load(output / LUT_FILENAME)
    if table.context != context or tuple(channel.name for channel in table.channels) != BANDS:
        raise ValueError("Build the LUT for this scene before running retrieval")
    scl = gdal.Open(str(scene / "original/SCL.tif"))
    datasets = aligned_bands(scene, scl)
    cover, warp_seconds = forest_grid(scl, worldcover_tiles, output)
    width, height = scl.RasterXSize, scl.RasterYSize
    windows = image_windows(width, height)
    lai, quality = new_outputs(scl, scene, output)
    inversion_start = perf_counter()
    elapsed_io = elapsed_search = elapsed_write = 0.
    counts = np.zeros(256, dtype=np.int64)
    matched_pixels = 0
    all_ok = 0
    sums = np.zeros(4)
    lai_range = [np.inf, -np.inf]
    for index, (x, y, w, h) in enumerate(windows, 1):
        stage = perf_counter()
        s = scl.ReadAsArray(x, y, w, h)
        c = cover.ReadAsArray(x, y, w, h)
        cube, spectral_valid, outside = reflectance_window(datasets, offsets, scale, x, y, w, h)
        flag = np.zeros((h, w), dtype=np.uint8)
        flag[(s <= 1) | ~spectral_valid] |= 1
        flag[s != 4] |= 2
        flag[(c != 10) & (c != 0)] |= 4
        flag[c == 0] |= 64
        flag[spectral_valid & outside] |= 128
        eligible = flag == 0
        elapsed_io += perf_counter() - stage

        stage = perf_counter()
        maps = np.full((4, h, w), NODATA, dtype=np.float32)
        if eligible.any():
            result = next(table.iter_match(
                cube, channels=table.channels, context=context,
                pixel_mask=eligible, workers=workers, block_size=h * w, k=TOP_K,
                max_score=MAX_RMSE, return_residuals=False,
            ))
            positions = result["pixel_indices"]
            candidates = result["parameters"][:, :, 0]
            scores = result["scores"][:, 0]
            spread = candidates.std(axis=1, ddof=0)
            values = np.vstack((candidates[:, 0], candidates.mean(axis=1), spread, scores))
            maps.reshape(4, -1)[:, positions] = values
            flag.flat[positions] |= (result["poor_fit"].astype(np.uint8) * 8
                                     | result["near_boundary"][:, 0].astype(np.uint8) * 16
                                     | (spread > SPREAD_LIMIT).astype(np.uint8) * 32)
            sums += values.sum(axis=1)
            matched_pixels += len(positions)
            all_ok += int(np.count_nonzero(flag.flat[positions] == 0))
            lai_range[0] = min(lai_range[0], candidates[:, 0].min())
            lai_range[1] = max(lai_range[1], candidates[:, 0].max())
        counts += np.bincount(flag.ravel(), minlength=256)
        elapsed_search += perf_counter() - stage

        stage = perf_counter()
        for band, values in enumerate(maps, 1):
            lai.GetRasterBand(band).WriteArray(values, x, y)
        quality.GetRasterBand(1).WriteArray(flag, x, y)
        elapsed_write += perf_counter() - stage
        if index == 1 or index % 12 == 0 or index == len(windows):
            print(f"block {index}/{len(windows)}, matched={matched_pixels:,}, "
                  f"elapsed={perf_counter() - inversion_start:.1f}s", flush=True)
    stage = perf_counter()
    lai.Close()
    quality.Close()
    for name in ("inform_lai_20m", "inform_quality_20m"):
        (output / f"{name}.part.tif").replace(output / f"{name}.tif")
    elapsed_write += perf_counter() - stage
    result = {
        "scene": scene.name, "source": str(scene), "crs": f"EPSG:{scl.GetSpatialRef().GetAuthorityCode(None)}",
        "raster_dimensions": [height, width], "pixel_size_m": 20, "block_pixels": BLOCK,
        "bands": BANDS, "boa_offsets": offsets, "quantification_value": scale,
        "geometry": {**geometry, **geometry_info}, "worldcover": "2021 v200, class 10, mode resampled to 20 m",
        "workers": workers, "top_k": TOP_K, "lut_rows": len(table.simulated),
        "window_count": len(windows), "pixels_processed": int(counts.sum()),
        "matched_pixels": matched_pixels, "matched_without_flags": all_ok,
        "quality_bit_definitions": FLAGS,
        "quality_bit_counts": {bit: int(counts[(np.arange(256) & bit) != 0].sum()) for bit in FLAGS},
        "mean_over_retrieved_pixels": dict(zip(("best_lai", "top16_mean_lai", "top16_lai_std", "best_rmse"),
                                               (sums / matched_pixels).tolist())) if matched_pixels else None,
        "best_lai_range": [float(x) for x in lai_range] if matched_pixels else None,
        "timing_seconds": {"worldcover_alignment": warp_seconds, "image_read_and_mask": elapsed_io,
                           "search_and_statistics": elapsed_search, "write_and_flush": elapsed_write,
                           "scene_wall": perf_counter() - inversion_start,
                           "run_wall_including_setup": perf_counter() - start},
    }
    write_json(output / "scene_summary.json", result)
    print(json.dumps(result, ensure_ascii=False), flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("lut", "scene"))
    parser.add_argument("--scene", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--workers", type=int)
    parser.add_argument("--sensor-cache", type=Path, help="optional pinned SRF cache for LUT construction")
    parser.add_argument("--worldcover", type=Path, nargs="+", help="WorldCover 2021 tiles covering the scene")
    args = parser.parse_args()
    if args.workers is not None and args.workers < 1:
        parser.error("--workers must be positive")
    if args.mode == "scene" and not args.worldcover:
        parser.error("scene mode requires --worldcover")
    args.output.mkdir(parents=True, exist_ok=True)
    if args.mode == "lut":
        build_lut(args.scene, args.output, args.workers or 4, args.sensor_cache)
    else:
        run(args.scene, args.output, args.workers or 16, args.worldcover)


if __name__ == "__main__":
    main()
