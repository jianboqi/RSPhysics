"""Benchmark exact LUT retrieval on reproducible simulated rasters."""

import argparse
from hashlib import sha256
import json
from pathlib import Path
from statistics import median
from time import perf_counter

import numpy as np

import rsphysics as rsp
from rsphysics import lut


def forest_image(rng):
    sensor = rsp.sensors.load("landsat-8.oli").select(["B2", "B3", "B4", "B5", "B6", "B7"])
    channels = tuple(lut.Channel(band, "directional_reflectance") for band in sensor.band_ids)
    geometry = dict(solar_zenith_deg=30, view_zenith_deg=12, relative_azimuth_deg=40)
    options = dict(output="reflectance", sensor=sensor, channels=channels, fixed=geometry)
    ranges = {"crown_lai": (0.5, 4), "stem_density_per_ha": (350, 900)}
    grid = lut.sample_grid({name: np.linspace(*bounds, 24) for name, bounds in ranges.items()})
    table = lut.build("vegetation.inform", grid, **options)
    held_out = lut.sample_lhs(ranges, 384, seed=22)
    spectra = lut.build("vegetation.inform", held_out, **options).simulated
    image = spectra[rng.integers(0, len(spectra), (256, 256))].copy()
    return table, image


def large_image(rng):
    channels = tuple(lut.Channel(f"band_{i}", "reflectance") for i in range(6))
    simulated = rng.random((100_000, len(channels)))
    table = lut.LookupTable.from_arrays({"sample": np.arange(len(simulated))}, simulated, channels)
    image = simulated[rng.integers(0, len(simulated), (32, 48))].copy()
    return table, image


def benchmark(case, workers, repeats):
    rng = np.random.default_rng(190)
    table, image = case(rng)
    image += rng.normal(0, 0.002, image.shape)
    pixel_mask = rng.random(image.shape[:2]) > 0.1
    image[rng.random(image.shape[:2]) < 0.02, 0] = np.nan
    options = dict(channels=table.channels, pixel_mask=pixel_mask, sigma=0.002, k=5,
                   block_size=16_384, return_residuals=False, max_score=2)
    table.match(image[0, :1], channels=table.channels, mask=np.isfinite(image[0, :1]))
    timings = {}
    expected = None
    for count in workers:
        elapsed = []
        for _ in range(repeats):
            start = perf_counter()
            blocks = list(table.iter_match(image, workers=count, **options))
            elapsed.append(perf_counter() - start)
        digest = sha256()
        for block in blocks:
            for field in ("pixel_indices", "indices", "scores", "parameters", "near_boundary", "poor_fit"):
                digest.update(block[field].tobytes())
        fingerprint = digest.hexdigest()
        if expected is None:
            expected = fingerprint
        assert fingerprint == expected, "worker count changed the retrieval results"
        timings[str(count)] = median(elapsed)
    return dict(lut_rows=len(table.simulated), image_shape=image.shape, matched_pixels=int(pixel_mask.sum()),
                seconds=timings, result_sha256=expected)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workers", type=int, nargs="+", default=[1, 8])
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    results = {case.__name__: benchmark(case, args.workers, args.repeats)
               for case in (forest_image, large_image)}
    text = json.dumps(results, indent=2)
    print(text)
    if args.output is not None:
        args.output.write_text(text + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
