"""Check the output rasters and independently recompute sampled LUT retrievals."""

import argparse
import json
from pathlib import Path

import numpy as np
from osgeo import gdal

from rsphysics import lut
from run import (
    FLAGS, LUT_FILENAME, MAX_RMSE, NODATA, RANGES, SPREAD_LIMIT, TOP_K,
    aligned_bands, image_windows, metadata, reflectance_window, write_json,
)


def verify(scene, output):
    gdal.UseExceptions()
    summary = json.loads((output / "scene_summary.json").read_text(encoding="utf-8"))
    assert summary["scene"] == scene.name
    reference = gdal.Open(str(scene / "original/SCL.tif"))
    maps = gdal.Open(str(output / "inform_lai_20m.tif"))
    quality = gdal.Open(str(output / "inform_quality_20m.tif"))
    cover = gdal.Open(str(output / "worldcover_2021_aligned_20m.tif"))
    height, width = summary["raster_dimensions"]
    for dataset in (maps, quality, cover):
        assert (dataset.RasterYSize, dataset.RasterXSize) == (height, width)
        assert dataset.GetGeoTransform() == reference.GetGeoTransform()
        assert dataset.GetSpatialRef().IsSame(reference.GetSpatialRef())
    assert maps.RasterCount == 4 and quality.RasterCount == 1
    assert quality.GetRasterBand(1).DataType == gdal.GDT_Byte
    for index in range(1, 5):
        band = maps.GetRasterBand(index)
        assert band.DataType == gdal.GDT_Float32 and band.GetNoDataValue() == NODATA

    _, offsets, scale, _ = metadata(scene)
    inputs = aligned_bands(scene, reference)
    table = lut.LookupTable.load(output / LUT_FILENAME)
    rng = np.random.default_rng(20240923)
    counts = np.zeros(256, dtype=np.int64)
    matched = checks = 0
    low, high = RANGES["crown_lai"]
    for x, y, w, h in image_windows(width, height):
        flags = quality.ReadAsArray(x, y, w, h)
        data = maps.ReadAsArray(x, y, w, h)
        valid = data[0] != NODATA
        assert np.all((data != NODATA) == valid)
        assert np.array_equal(valid, (flags & (1 | 2 | 4 | 64 | 128)) == 0)
        assert np.all((flags[~valid] & (8 | 16 | 32)) == 0)
        assert np.isfinite(data[:, valid]).all()
        assert np.all((data[:2, valid] >= low) & (data[:2, valid] <= high))
        assert np.all(data[2:, valid] >= 0)
        for bit, band, threshold in ((8, 3, MAX_RMSE), (32, 2, SPREAD_LIMIT)):
            values = data[band, valid]
            # The flags are computed before Float32 rounding for storage.
            unambiguous = np.abs(values - threshold) > np.spacing(np.float32(threshold))
            np.testing.assert_array_equal((flags[valid][unambiguous] & bit) != 0,
                                          values[unambiguous] > threshold)
        counts += np.bincount(flags.ravel(), minlength=256)
        positions = np.flatnonzero(valid)
        matched += len(positions)
        if not len(positions):
            continue
        cube, _, _ = reflectance_window(inputs, offsets, scale, x, y, w, h)
        selected = rng.choice(positions, size=min(len(positions), 4), replace=False)
        for position in selected:
            row, col = divmod(position, w)
            rmse = np.sqrt(np.mean((table.simulated - cube[row, col]) ** 2, axis=1))
            order = np.argsort(rmse, kind="stable")[:TOP_K]
            candidates = table.samples.values[order, 0]
            expected = [candidates[0], candidates.mean(), candidates.std(), rmse[order[0]]]
            np.testing.assert_allclose(data[:, row, col], expected, rtol=1e-6, atol=1e-8)
            checks += 1

    assert counts.sum() == width * height == summary["pixels_processed"]
    assert matched == summary["matched_pixels"]
    assert counts[0] == summary["matched_without_flags"]
    for bit in FLAGS:
        assert counts[(np.arange(256) & bit) != 0].sum() == summary["quality_bit_counts"][str(bit)]
    report = {"crs": summary["crs"], "pixel_size_m": 20, "verified_quality_pixels": int(counts.sum()),
              "verified_retrieved_pixels": matched, "independent_exhaustive_lut_checks": checks}
    write_json(output / "verification.json", report)
    print(json.dumps(report))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scene", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    verify(args.scene, args.output)
