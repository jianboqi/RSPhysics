# Lookup-table inversion

A lookup table (LUT) associates parameter samples with simulated observations. The matcher searches the table for the rows that best fit an observation. It supports RSPhysics forward models and tables produced by other simulators.

## Choose parameter samples

```python
import numpy as np
from rsphysics import lut

grid = lut.sample_grid({
    "crown_lai": np.linspace(0.5, 4.0, 15),
    "stem_density_per_ha": [350, 500, 650, 800],
})
lhs = lut.sample_lhs({
    "crown_lai": (0.5, 4.0),
    "stem_density_per_ha": (350, 800),
}, n_samples=60, seed=42)
```

The grid evaluates all combinations of the specified levels. `max_samples` limits its size before allocation. Latin hypercube sampling draws one point from each marginal stratum with a fixed total sample count. Use `log_parameters` for positive ranges that should be stratified in log space. Sampling metadata stores the parameter order, bounds and the seed used by the sampler.

## Build a sensor-space LUT

The example retrieves single-crown LAI from INFORM using a synthetic Landsat 8 observation. Geometry and stem density are fixed. A real observation must describe the same physical quantity under the same geometry, with bands in the same order.

```python
import numpy as np
import rsphysics as rsp
from rsphysics import lut

sensor = rsp.sensors.load("landsat-8.oli").select(["B4", "B5", "B6"])
channels = tuple(lut.Channel(band, "directional_reflectance") for band in sensor.band_ids)
geometry = {"solar_zenith_deg": 30, "view_zenith_deg": 12, "relative_azimuth_deg": 40}
context = {"sensor": sensor.id, **geometry}
fixed = {**geometry, "stem_density_per_ha": 650, "diffuse_fraction": 0.1}

samples = lut.sample_grid({"crown_lai": np.linspace(0.5, 3.5, 16)})
table = lut.build(
    "vegetation.inform", samples, output="reflectance", sensor=sensor,
    channels=channels, fixed=fixed, context=context,
)

spectrum = rsp.vegetation.inform(crown_lai=2.35, **fixed)["result"]
observed = sensor.convolve(spectrum["wavelengths_nm"], spectrum["reflectance"])["values"]
fit = table.match(observed, channels=channels, context=context, k=3)
print(fit["parameters"][0, 0, 0])  # Best table value for crown_lai
print(fit["residuals"][0, 0])      # Simulated minus observed, in channel order

table.save("inform-oli.npz")
table = lut.LookupTable.load("inform-oli.npz")
```

`output` accepts a scalar or vector field, an ordered sequence of fields, or a function that converts one model result into a feature vector. The sensor option integrates one spectral field using the selected response functions. Dotted names such as `crown_leaf.chlorophyll` address nested model inputs. A `constraint` can filter rows using sampled and fixed input values. The table records how many samples were attempted, accepted and rejected. Filtering can alter the marginal strata of an LHS design.

For radar backscatter from I²EM, use `output=("sigma0_vv_db", "sigma0_hh_db")` and channels such as `Channel("VV", "backscatter_coefficient", "dB")` and `Channel("HH", "backscatter_coefficient", "dB")`. Use `LookupTable.from_arrays(parameter_columns, simulated, channels)` to import external simulations. The simulation matrix is shaped `(samples, channels)`.

## Match and interpret candidates

- Pass one observation `(channels,)` or a matrix `(observations, channels)`. If a table defines `context`, supply the same observation context when matching. Channels compare by name, quantity, unit and order.
- The score is RMSE across active channels. With `sigma`, each residual is divided by its channel's standard deviation before taking the RMSE. Standard deviations can be shared or specified per observation. A boolean `mask` excludes unavailable channels. Each observation needs at least one active channel.
- `parameters` has shape `(observations, k, parameters)`, `scores` and `indices` have shape `(observations, k)`, and `residuals` has shape `(observations, k, channels)`. Lower scores fit better. Ties preserve table order. Masked residuals are NaN. With `return_residuals=False`, the result contains `residuals=None`.
- `workers` sets the maximum number of Rust search threads. The default is 1. Workers share the prepared LUT and return results in observation order.
- Use `max_score` to flag poor fits at a threshold appropriate to the score and data. `near_boundary` flags best-match parameters in the outer 5% of their sampling interval by default. Examine the top candidates together when different parameter combinations give similar scores.
- Archives contain the accepted parameter samples, simulated observations, channels, context, sampling metadata, and available model and sensor provenance. Search uses bounded storage for the top `k` candidates per observation.
- Large, low-dimensional tables can use an exact k-d tree for observations with all channels active. The matcher prepares the index on demand and reuses it across blocks. Observations with missing channels use a full scan. Both paths use the same scores and tie order.

## Process a raster in blocks

`iter_match` accepts an array shaped `(height, width, channels)` or `(pixels, channels)`. It reads one block at a time, including for memory-mapped files and noncontiguous views. Supply a boolean `pixel_mask` to restrict the search to the study area or exclude clouds. Nonfinite observations and values excluded by the optional band `mask` are ignored. Pixels with no active bands are skipped.

```python
image = np.load("surface_reflectance.npy", mmap_mode="r")  # B4, B5, B6 on the last axis
valid_pixels = np.load("valid_pixels.npy", mmap_mode="r")
lai = np.lib.format.open_memmap("lai.npy", mode="w+", dtype="float32", shape=image.shape[:2])
lai[:] = np.nan

for block in table.iter_match(
    image, channels=channels, context=context, pixel_mask=valid_pixels,
    workers=8, block_size=16_384, return_residuals=False,
):
    lai.flat[block["pixel_indices"]] = block["parameters"][:, 0, 0]
lai.flush()
```

Each block includes its half-open flat pixel range `[start, stop)` and the zero-based row-major `pixel_indices` of matched pixels. The candidate arrays contain just those matched pixels. An entirely masked block yields empty candidate arrays. The iterator defaults to one best candidate and omits residual computation. Use `k` and `return_residuals=True` to request additional output.

The [synthetic raster benchmark](../benchmarks/infrastructure/lut_image.py) reports timings for different worker counts and checks that retrieved parameters, scores and quality flags agree exactly. Run it from the repository root with the installed package.

```console
python benchmarks/infrastructure/lut_image.py --workers 1 8
```

The [Sentinel-2B INFORM benchmark](../benchmarks/infrastructure/inform_sentinel2/README.md) runs tiled crown LAI retrieval on a real L2A scene and writes GeoTIFFs with candidate statistics and quality flags.
