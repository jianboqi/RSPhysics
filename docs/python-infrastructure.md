# Python scientific infrastructure

The public Rust numerical crate implements interpolation, integration, bandpass
weights and radiometric equations. Python owns NumPy buffers, table IO, explicit
downloads, cache management and workflow conveniences. These operators do not
introduce a model graph, inversion or automatic model coupling. Existing model
IDs, equations, dedicated batch column orders and default output shapes remain
compatible.

## 📘 Spectral processing

```python
import rsphysics as rsp
import numpy as np

leaf = rsp.vegetation.prospect_d()['result']
sampled = rsp.spectral.resample(
    leaf['wavelengths_nm'], leaf['reflectance'], [450., 550., 670., 850.])
integrated = rsp.spectral.integrate(
    leaf['wavelengths_nm'], leaf['reflectance'], lower_nm=700, upper_nm=900)
custom = rsp.sensors.Bandpass('custom-red', [620., 650., 680.], [0., 1., 0.])
band = custom.convolve(leaf['wavelengths_nm'], leaf['reflectance'])
```

All wavelength axes are positive, finite, strictly increasing **nm**. Inputs have
shape `(..., wavelength)`; arbitrary leading dimensions and strided inputs work.
Values must be finite. No automatic sorting, unit guessing, extrapolation or
NaN filling occurs. Linear interpolation spans consecutive supplied nodes;
sampling resolution and unresolved narrow absorption features remain the
caller's responsibility. An interval integral has input-value units multiplied
by nm; a normalized band mean retains the input-value units.

The kernel treats the spectrum, response and optional incident irradiance as
piecewise-linear functions. It integrates on the union of their knots using
two-point Gauss quadrature, exact for their cubic product up to f64 rounding.
This differs from evaluating only at band centers or interpolating only to the
usually coarser response grid. Fraction-based independent references exercise
32 nonuniform-grid cases, including negative signal values and different
illumination/response grids.

```python
# Response-only spectral summary: integral(S R) / integral(R)
summary = rsp.spectral.convolve(x, spectra, response_x, response)
# Illumination-weighted summary: integral(S R E) / integral(R E)
weighted = rsp.spectral.convolve(
    x, spectra, response_x, response,
    irradiance=solar_spectrum, irradiance_wavelengths_nm=solar_x)
```

`x`, `spectra` etc. above are user-supplied arrays. A response-only average of
reflectance/BRF is a spectral summary, not automatically an illuminated band
reflectance. Appropriate incident irradiance, quantity definitions and geometry
must be supplied for the latter; no atmosphere or solar spectrum is assumed.
Irradiance is a single 1-D spectrum, shared across leading sample dimensions.
Inputs already in radiance units generally use response-only radiance averaging.

The response is assumed zero outside its tabulated interval; no unmeasured tail
is invented. Only exact-zero tails may be ignored for coverage. The default
rejects missing nonzero response support even if its area is tiny. Explicit
`allow_partial=True` normalizes over the overlap and reports **unweighted
response-area coverage**. Coverage does not estimate a missing signal or its
error. Irradiance must cover the entire chosen integration interval. Negative,
nonfinite and zero-area custom responses are rejected. Relative response scale
does not change the normalized result.

## 📘 Official sensor profiles

```python
rsp.sensors.list_sensors()               # version, bands, source, availability
oli = rsp.sensors.load('landsat-8.oli')   # offline; no network on load
canopy = rsp.vegetation.prosail_d()['result']
bands = oli.convolve(canopy['wavelengths_nm'], canopy['brf'])
print(bands['bands'], bands['values'], bands['coverage'])

# Explicit ordered subset; no silent dropping of unsupported bands.
red_nir = oli.select(['B4', 'B5'])

# Explicit acquisition/import: shared official v5 workbook installs A/B/C.
s2 = rsp.sensors.fetch('sentinel-2a.msi')
# Or: rsp.sensors.import_file('sentinel-2a.msi', '/path/to/official-v5.xlsx')
s2 = rsp.sensors.load('sentinel-2a.msi')
```

| Profile IDs | Scope | Distribution |
| --- | --- | --- |
| `sentinel-2a.msi`, `sentinel-2b.msi`, `sentinel-2c.msi` | Official v5, 13 instrument-average bands each | Local download/import, not bundled |
| `landsat-4.tm`, `landsat-5.tm` | 6 reflective bands each | Bundled USGS tables |
| `landsat-7.etm-plus` | 7 reflective/panchromatic bands | Bundled USGS table |
| `landsat-8.oli`, `landsat-9.oli-2` | 9 reflective/panchromatic bands each | Bundled USGS tables |
| `landsat-8.tirs`, `landsat-9.tirs-2` | Thermal B10/B11, nm converted from provider um | Bundled USGS tables |
| `terra.modis`, `aqua.modis` | 16 reflective band-average responses from NASA OBPG | Bundled NASA tables |

There are **12 registered profiles / 112 response curves**; **9 profiles / 73
curves** work directly from the installed package. MODIS labels retain the
provider's `RSR_412` etc. wavelength names and are not inferred MODIS band
numbers. These profiles are neither all 36 MODIS bands nor detector-specific
MCST responses. OBPG tails below 400 nm are significant for strict coverage:
PROSAIL's 400–2500 nm output may need an explicitly acknowledged partial
calculation. TIRS curves cannot be applied to an optical PROSAIL spectrum.

The returned band values have shape `(..., band)`. Coverage, full-response
centroids and equivalent widths are 1-D per-band arrays. Result provenance
includes the platform, version, source URL and original SHA-256. The registry
also pins processed assets' SHA-256, source tabulation and preprocessing details.
USGS negative measurement-noise samples are zeroed and counted (L8 OLI: 43;
L7 ETM+: 5). The L4 B5 source has out-of-order tail rows; preparation sorts
these original pairs and records that fact, rejecting duplicate wavelengths.
No Gaussian response approximation or silent response resampling is used.

SentiWiki's terms restrict redistribution and derivatives of its materials, so
no ESA workbook or response samples are included in the wheel/sdist. Only
identification, download URLs and checksums are registered. `fetch()` is an
explicit network operation and verifies the pinned source before parsing.
Changed upstream bytes fail rather than silently changing an instrument version.
`load()` never downloads. XLSX is parsed as XML using the standard library,
without Excel, openpyxl, formula execution or archive extraction.

The default local cache is `%LOCALAPPDATA%/rsphysics/sensors` on Windows and
`$XDG_CACHE_HOME/rsphysics/sensors` (or `~/.cache/rsphysics/sensors`) elsewhere.
Override with `RSPHYSICS_DATA_DIR` or `cache_dir=`. Original and parsed local ESA
assets remain available offline. Pinned official imports cannot be mislabeled
as custom versions; use `Bandpass` for user-provided arrays.

Official sources: [Copernicus S2 v5 announcement](https://sentinels.copernicus.eu/-/sentinel-2-spectral-response-functions-update),
[SentiWiki documents](https://sentiwiki.copernicus.eu/web/s2-documents),
[SentiWiki terms](https://sentiwiki.copernicus.eu/web/terms-and-conditions),
[USGS spectral viewer](https://landsat.usgs.gov/spectral-characteristics-viewer),
[NASA OBPG tables](https://oceandata.sci.gsfc.nasa.gov/rsr_tables/).

## 📘 Unified native NumPy batches

```python
model = 'vegetation.prosail-d'
columns = rsp.parameters_for(model)
table = rsp.parameter_grid(model, {
    'leaf_area_index': [1., 3., 5.],
    'chlorophyll': [20., 40., 60.],
})
out = rsp.run_array(model, table, shared_axes=True)
x = out['coordinates']['wavelengths_nm']  # (2101,)
spectra = out['result']['brf']             # (9,2101)
satellite = oli.convolve(x, spectra)       # values (9,9)

for chunk in rsp.iter_array(model, table, chunk_size=4, shared_axes=True):
    consume(chunk['row_offset'], chunk['result'], chunk['coordinates'])
```

`consume` above is a user callback, not an RSPhysics function. All five models
use the same typed batch collector and direct NumPy scientific-output buffers;
only provenance manifests use JSON. Inputs are copied before releasing the GIL
to avoid concurrent NumPy mutation during a computation. There is no Python
scientific row loop or internal thread pool. Defaults preserve the existing
dedicated `*_batch` result layouts; those helpers also accept `shared_axes=True`.
In the shared layout, fixed wavelength/angle coordinates move to `coordinates`,
scalar outputs have `(batch,)`, curves `(batch,sample)`, and integer diagnostics
retain int64. Empty batches have correct shapes and coordinate arrays.

Parameters are a real `(batch, parameter)` matrix in `parameters_for()` order;
there is no implicit parameter broadcasting. Float32, integer, non-native-endian
and strided arrays are converted safely to f64. Invalid inputs identify a
zero-based row. `iter_array` yields bounded output chunks with global row
offsets; previous chunks remain valid if a later chunk fails. Collecting all
chunks retains all output memory, and the input parameter table itself is kept.
`parameter_grid` builds an explicit small Cartesian grid with a configurable
size guard; it is not an inversion or parameter-fitting tool. The existing
mapping-based `run` / `run_batch` runtime APIs remain available.

Local timing evidence lives in `benchmarks/infrastructure/validation-report.json`.
It compares native buffers and JSON runtime with identical 64-row model inputs,
using medians of three runs; hardware-dependent timings are not a speed guarantee.

## 📘 Basic radiometric calibration

```python
cal = rsp.radiometry.linear_calibrate(
    dn, gain=metadata_gain, offset=metadata_offset,
    unit='W m^-2 sr^-1 um^-1', nodata=0, saturation=65535)
bt = rsp.radiometry.radiance_to_brightness_temperature(
    cal['values'], k1=metadata_k1, k2=metadata_k2, valid_mask=cal['valid'])
toa = rsp.radiometry.dn_to_toa_reflectance(
    dn, reflectance_mult=metadata_mult, reflectance_add=metadata_add,
    solar_zenith_deg=90-metadata_sun_elevation)
```

`dn` and `metadata_*` above must come from the specific product. The example
thresholds are illustrative, not universal sensor defaults. Coefficients and
solar geometry are scalar per call; apply band-specific constants separately.

| Operation | Equation / convention |
| --- | --- |
| `linear_calibrate` | `gain*DN+offset`; explicit required output-unit label |
| `dn_to_toa_reflectance` | `(M_rho*DN+A_rho)/cos(theta)` using Level-1 reflectance metadata factors |
| `radiance_to_toa_reflectance` | `pi*L*d^2/(ESUN*cos(theta))`; L and ESUN must share the same spectral-density basis, ESUN at 1 AU |
| `radiance_to_brightness_temperature` | `K2/log(1+K1/L)`; K1 in per-um radiance units, K2 in K; per-nm input is explicitly converted |

Level-1 reflectance multipliers already incorporate irradiance/distance; do not
apply those corrections twice. Level-2 surface reflectance scale/offset uses
`linear_calibrate`, without adding a Level-1 solar-angle correction. Zenith is
measured from the vertical, not solar elevation. Thermal constants are specific
to a thermal band; the result is at-sensor brightness temperature, not surface
temperature or emissivity-corrected LST. No scene reader or atmospheric
correction is included. These conventions follow the
[USGS Level-1 guide](https://www.usgs.gov/landsat-missions/using-usgs-landsat-level-1-data-product).

Every operation returns same-shaped `values` (float64), `valid` (bool), `flags`
(uint8), and an explicit unit label. Invalid values become NaN. Flags can be
combined: 1 nonfinite, 2 configured nodata, 4 DN at/above saturation threshold,
8 externally masked, 16 outside the operation domain (e.g. nonpositive thermal
radiance). DN=0 is not implicitly nodata, saturation is not guessed from dtype,
and calibrated values are not clipped to [0,1]. Invalid scalar metadata raises
an error even for empty/all-masked inputs. This layer does not parse QA bits;
the caller can supply an exact-shape boolean `valid_mask` from product QA.

## 📘 Reproduce and verify

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
python tooling/sensor-data/verify.py
python -m pytest bindings/python/tests -q --basetemp target/pytest
python benchmarks/infrastructure/report.py
python bindings/python/examples/satellite_band_workflow.py
```

Standard tests are offline and skip the one optional actual ESA import test.
Set `RSPHYSICS_TEST_S2_CACHE` to a cache containing
`raw/sentinel-2-srf-v5.xlsx` to enable that test. The scientific reference fixture
generator is `benchmarks/reference/spectral/generate.py` and uses exact rational
polynomial integration independently of the Rust algorithm.

See [fundamental physics tools](basic-physics.md) for single-interface Fresnel optics and wavelength/frequency Planck radiance and inverses. Numerical algorithms remain in the shared Rust kernel without introducing model coupling.


SMAC supports the common NumPy interface. `parameters_for("atmosphere.smac")` defines eight numerical columns; pass string selections through `fixed`, which cannot override numerical columns. `iter_array` also accepts `fixed`; see [SMAC](smac.md).
