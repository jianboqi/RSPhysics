# 6SV2.1 — vector atmospheric radiative transfer

6SV2.1 computes solar radiation transmitted and scattered by a plane-parallel
atmosphere, including molecular scattering, aerosol scattering, gaseous
absorption, polarization, and surface reflection. Its successive-orders-of-
scattering solver covers 0.25–4 μm. It does not simulate thermal emission.

The Python interface calls the bundled Fortran library directly. Prebuilt wheels
require neither a Fortran compiler nor a separately installed 6S executable.
This model is available in the development source tree; it is not included in
the published PyPI 0.1.0 wheels.

## Basic calculation

```python
from rsphysics import sixs

run = sixs.run(
    geometry=sixs.Geometry.user(30, 0, 20, 90, month=7, day=1),
    atmosphere=sixs.Atmosphere.standard(2),
    aerosol=sixs.Aerosol.predefined(1, aot550=0.2),
    spectrum=sixs.Spectrum.monochromatic(0.55),
    surface=sixs.Surface.lambertian(0.2),
)
result = run["result"]
print(result["apparent_reflectance"])
print(result["apparent_radiance_w_m2_sr_um"])
```

The generic interface is also available:

```python
import rsphysics as rsp

run = rsp.run("atmosphere.sixsv21", {"aerosol": {"aot550": 0.15}})
```

Parameters are grouped into `geometry`, `atmosphere`, `aerosol`, `altitudes`,
`spectrum`, `surface`, `correction`, and `polarization`. Omitted values receive
defaults. Use `sixs.prepare(configuration)` to inspect the complete effective
configuration and its input cards. Unknown parameter names are rejected.
`sixs.SixS(configuration, **sections)` stores a reusable configuration and offers
`prepare()`, `input_cards()`, `to_dict()`, and `run()`.

## Geometry and atmospheric profiles

Angles are in degrees; azimuths are measured clockwise from north. Geometry mode
0 takes solar/view zenith and azimuth angles, month, and day. Modes 1–7 retain
the original Meteosat, GOES East/West, AVHRR AM/PM, SPOT HRV, and Landsat TM
geometry calculations. Configure those with `Geometry.satellite(mode, **values)`
using UTC hour and the corresponding image column/line, ascending-node position,
or ground longitude/latitude. Historical satellite geometry conventions should
be checked before using them with another instrument.

`Geometry.user(max_scattering_order=7)` sets the multiple-scattering order
limit for user geometry. Omitting it uses the solver default of 20. The
satellite geometry modes do not accept this override.

| Profile | Atmosphere |
|---|---|
| 0 | No gaseous absorption |
| 1 | Tropical |
| 2 | Midlatitude summer |
| 3 | Midlatitude winter |
| 4 | Subarctic summer |
| 5 | Subarctic winter |
| 6 | US Standard 1962 |
| 7 | User-supplied 34-level profile |
| 8 | User-supplied water and ozone columns |

```python
atmosphere = sixs.Atmosphere.water_ozone(2.0, 0.3)  # g/cm², atm-cm
levels = sixs.standard_levels(6)                    # NumPy (34, 5)
radiosonde = sixs.Atmosphere.radiosonde(levels)
```

Profile columns are altitude (km), pressure (hPa), temperature (K), water-vapour
density (g/m³), and ozone density (g/m³). The original final 99999-km sentinel is
retained. Supply all 34 rows, with strictly increasing altitudes.

## Aerosols and Mie calculations

| ID | Aerosol |
|---|---|
| -1 | Layered profile: thickness (km), AOT550, aerosol type |
| 0 | No aerosols |
| 1, 2, 3 | Continental, maritime, urban |
| 4 | Dust, water-soluble, oceanic, soot volume fractions |
| 5, 6, 7 | Desert, biomass burning, stratospheric |
| 8 | Up to four log-normal modes |
| 9 | Modified Gamma distribution |
| 10 | Junge distribution |
| 11 | Sun-photometer particle-size distribution |
| 12 | Previously saved Mie optical properties |

Use `aot550` or `visibility_km` for aerosol loading. Layered profiles contain
1–50 rows and share one aerosol optical model, including custom mixtures and
Mie distributions. Pass the optical model parameters to `Aerosol.layered`.
Mixture and log-normal mode
fractions must sum to one. Microphysical distributions use radii in μm and exactly
20 refractive-index samples at `sixs.REFRACTIVE_WAVELENGTHS_UM`. NumPy arrays are
accepted by the Python helpers.

```python
import numpy as np

mode = sixs.Aerosol.mode(
    mean_radius_um=0.1, sigma=1.5, fraction=1.0,
    refractive_real=np.full(20, 1.5),
    refractive_imag=np.full(20, 0.01),
)
run = sixs.run(aerosol=sixs.Aerosol.lognormal([mode], mie_file="aerosol"))
saved = run["result"]["files"]["aerosol.mie"]
reused = sixs.run(
    aerosol=sixs.Aerosol.from_mie("saved.mie"),
    files={"saved.mie": saved},
)
```

Other constructors are `Aerosol.mixture`, `layered`, `gamma`, `junge`, and
`sunphotometer`. A saved Mie file uses the original formatted precision, so
reloading it can introduce a small rounding difference. Auxiliary filenames
must be simple ASCII names without whitespace or directory components.

## Target and sensor altitude

`Altitudes.satellite()` selects a satellite sensor. Use `target_altitude_km` or
`target_pressure_hpa` to specify the target, and `Altitudes.ground()` for a
ground-level sensor. `Altitudes.aircraft(height_km, ...)` takes sensor height
above the target. Its `path_water_vapour_g_cm2`, `path_ozone_atm_cm`, and
`path_aot550` default to `-1`, requesting the original automatic estimates.

## Wavelengths and sensor response

All wavelength arguments use μm.

```python
bands = sixs.bands()                         # 198 original band IDs, 2–199
band = sixs.Spectrum.band(sixs.Band.LANDSAT_TM_B1)
uniform = sixs.Spectrum.uniform(0.5, 0.6)
response = sixs.Spectrum.response(0.5, 0.51, [0, 0.5, 1, 0.5, 0])
step = sixs.Spectrum.step(0.5, 0.51)
```

Custom responses follow the original 2.5-nm grid from 0.25 μm. The endpoints
are mapped to that grid; the response length must match the selected interval
and include positive support. `step` retains the per-wavelength rows in
`result["spectral_rows"]`. For named NumPy output arrays, use
`sixs.spectrum(wavelengths_um, configuration)`.

## Surface reflection

`Surface.lambertian(value)` supplies a constant reflectance.
`Surface.predefined(1..4)` selects vegetation, clear water, sand, or lake water.
`Surface.spectrum(lower_um, upper_um, values)` supplies a tabulated spectrum.
`Surface.heterogeneous(target, background, radius_km=...)` models a circular
target and its background; each spectrum is a scalar or a reflectance mapping.
`Surface.from_samples(wavelengths_um, reflectance)` defines a uniform
Lambertian surface from a measured spectrum. For a heterogeneous surface, pass
`Surface.sampled_curve(wavelengths_um, reflectance)` as either the target or
background. Each run resamples the curve to the active band. Measured
wavelengths must cover that band's 6S grid.

For directional surfaces, use `Surface.brdf(name_or_id, parameters)`. Parameters
are flattened in the following order; integer option selectors remain integers.
The named constructors also accept keyword arguments, for example
`Surface.rahman(rho0=0.15, asymmetry=-0.2, k=0.8)`.

| ID / name | Parameter order |
|---|---|
| 0 / `measured` | Use `Surface.measured`: two 13×10 BRDF tables, spherical albedo, selected-direction BRF |
| 1 / `hapke` | Single-scattering albedo, phase asymmetry, hotspot amplitude, hotspot width |
| 2 / `verstraete` | `opt3,opt4,opt5`; `str1,str2,str3,str4`; `optics1,optics2,optics3` |
| 3 / `roujean` | `k0,k1,k2` |
| 4 / `walthall` | `a,ap,b,c` |
| 5 / `minnaert` | `k`, surface albedo |
| 6 / `ocean` | Wind speed (m/s), wind azimuth (°), salinity (ppt), pigment (mg/m³) |
| 7 / `iaquinta_pinty` | Leaf-angle selector, hotspot selector; LAI, hotspot size; leaf reflectance, leaf transmittance, soil albedo |
| 8 / `rahman` | `rho0`, phase asymmetry, structural exponent `k` |
| 9 / `kuusk` | LAI, LAD eccentricity, LAD angle (°), relative leaf size; chlorophyll (μg/cm²), leaf water thickness (cm), internal layers, refractive-index ratio, soil Price-function weight |
| 10 / `modis` | Isotropic, RossThick, LiSparse kernel weights |
| 11 / `ross_li_maignan` | Isotropic, RossThick with hotspot, LiSparse kernel weights |

Measured tables use 13 relative azimuth rows (0–360° in 30° steps) and 10
zenith columns in descending order: 85°, 80°, 70°, …, 0°. Verstraete options
select extinction (0–2), phase function (0–2), and multiple scattering (0–1).
Its structural inputs are leaf area density, sunfleck radius, and two orientation
parameters; optical inputs are single-scattering albedo and phase coefficients.
Iaquinta–Pinty leaf-angle IDs 1–5 denote planophile, erectophile, plagiophile,
extremophile, and uniform distributions. Kuusk leaf optics cover 0.404–2.5 μm.

## Atmospheric correction and polarization

```python
run = sixs.run(
    correction=sixs.Correction.from_reflectance(0.25),
    surface=sixs.Surface.modis(0.3091, 0.129, 0.0762),
)
q = run["result"]["quantities"]
```

`Correction.from_radiance(value)` accepts W/(m²·sr·μm).
`Correction.from_reflectance(value)` accepts dimensionless apparent reflectance.
The report retains Lambertian and applicable BRDF correction results, including
`coefficients_xa_xb_xc` and `coefficients_xap_xb_xc` when present.

Surface polarization accepts `Polarization.stokes(q,u)`,
`Polarization.vegetation(fraction)`, or `Polarization.ocean(wind_speed_m_s,
wind_azimuth_deg)`. These describe the surface boundary; they do not disable the
original vector solver, whose `ipol=1` behavior is retained.

## Results, batches, and original input cards

The result contains apparent reflectance, apparent radiance, `quantities`,
`diagnostics`, the complete `report`, and named auxiliary `files`. Gas and
scattering transmission vectors are ordered downward/upward/total. Optical
depth, spherical albedo, and Stokes reflection vectors are ordered
Rayleigh/aerosol/total. Undefined diagnostics are represented by `None`.
Results retain the original report's printed precision and the kernel's mixed
REAL/DOUBLE PRECISION arithmetic.

`diagnostics` includes ground irradiance, satellite radiance and reflectance
components, apparent polarization, ground pressure and altitude, visibility
and AOT at 550 nm, solar irradiance, and ocean reflectance components. Fields
that are absent in a particular 6S report return `None`. `spectral_columns` labels the columns
in `spectral_rows` for a step calculation. `aerosol_lut` gives reflectance
samples by view zenith and scattering angle when a LUT is produced.
`mie_tables` contains optical coefficients and phase matrices with scattering
angle rows and wavelength columns when the run produces a Mie file.

```python
rows = [{"aerosol": {"aot550": x}} for x in [0.1, 0.2, 0.4]]
runs = sixs.run_batch(rows)
scan = sixs.spectrum([0.45, 0.55, 0.65, 0.85])
cards = sixs.prepare(rows[0])["prepared"]["input_cards"]
same = sixs.run_cards(cards)
lut = sixs.run({"program": "lutaero"})
```

`program="lutaero"` selects the original aerosol-LUT entry point and retains its
output files. Original input cards and a `files` mapping expose the original
card-based interface through the same direct library call; cards override
structured sections and bypass their physical validation.

Calls within a process are serialized and reset model-owned state. For parallel
independent calculations, use `sixs.run_batch(rows, workers=N)` under the usual
`if __name__ == "__main__":` guard. Errors identify the batch row. Mie scratch
memory follows the selected series order; the original maximum is retained.

See [the executable example](../../bindings/python/examples/sixs_workflow.py).

## Observation data and scans

```python
geometry = sixs.Geometry.from_time_and_location(40, -105, "2026-07-01T19:00:00Z")
profile = sixs.Atmosphere.from_latitude_and_date(40, "2026-07-01")
config = {"geometry": geometry, "atmosphere": profile}

modis = sixs.sensor_bands("modis", config, band_ids=[42, 43])
plane = sixs.principal_plane([-30, -15, 0, 15, 30], config)
azimuth = sixs.azimuth_scan(20, config, step_deg=30)
grid = sixs.angular_grid(
    [10, 20, 30, 40],
    [0, 60, 120, 180, 240, 300],
    config,
    output="diagnostics.pixel_reflectance",
)
print(grid["values"].shape)  # (4, 6), ordered by zenith then azimuth

window = sixs.Spectrum.band(42)
target = sixs.Surface.sampled_reflectance([0.5, 0.6, 0.7, 0.8],
                                          [0.1, 0.2, 0.3, 0.4], window)
surface = sixs.Surface.heterogeneous(target, 0.1)
```

`sixs.bands()` lists built-in band families, symbolic names and integration bounds.
`sensor_bands` also accepts a registry sensor ID, such as `landsat-8.oli`.
`Spectrum.from_samples` places tabulated responses on the 2.5 nm grid.
`Surface.sampled_reflectance` prepares a curve for a single known band.
`Surface.from_aster` reads a local ASTER/ECOSTRESS reflectance file.

`vnir` and `full_spectrum` select preset wavelength ranges. Pass
`direction="solar"` to `angular_grid` to vary the solar geometry. The
`output` argument on scan functions accepts a dotted result path such as
`diagnostics.pixel_radiance_w_m2_sr_um` or
`quantities.global_gas_trans.2`. The selected values are collected in
`values`. Undefined numeric values become NumPy NaN.

Install the optional `examples` dependencies to use `plot_spectrum`,
`plot_principal_plane`, and `plot_angular_grid` with Matplotlib.

`Atmosphere.from_radiosonde` interpolates observed levels onto the standard
6S grid. Outside the measured heights it uses the chosen standard profile,
with its pressure scaled to meet the observations at the transition.
`Atmosphere.from_wyoming` reads downloaded Wyoming TEXT:CSV or TEXT:LIST
tables. `Atmosphere.from_wyoming_url` fetches a single sounding from its
results URL, and
`Atmosphere.from_bas` reads the legacy BAS text table.
`Aerosol.from_aeronet` reads a local all-points CSV and selects the nearest
valid measurement within one hour by default. It estimates AOT at 550 nm
from bracketing channels when the 550 nm channel is unavailable. The aerosol
optical model is selected separately. `Aerosol.from_aeronet_inversion`
uses a single-site V3 ALL or V2 Combined file to build a Mie aerosol from
particle size and refractive index. It uses the nearest complete retrieval
within the time window. Refractive indices are interpolated onto the 20
6S wavelengths and held constant outside the measured interval. The 6S Mie
calculation assumes spherical particles. Solar angles use NOAA's geometric
approximation and require an explicit UTC offset. [AERONET V3 products](https://aeronet.gsfc.nasa.gov/print_web_data_help_v3_inv.html) · [NOAA solar equations](https://gml.noaa.gov/grad/solcalc/solareqns.PDF) · [Wyoming sounding columns](https://weather.uwyo.edu/upperair/columns.html)

## Authors and references

6S/6SV was developed by Eric Vermote, Didier Tanré, Jean-Luc Deuzé, Maurice
Herman, Jean-Jacques Morcrette, Svetlana Kotchenova, and collaborators.

- Vermote, E. F., Tanré, D., Deuzé, J. L., Herman, M., and Morcrette, J. J.
  (1997). Second Simulation of the Satellite Signal in the Solar Spectrum,
  6S: An Overview. *IEEE Transactions on Geoscience and Remote Sensing*,
  35(3), 675–686. [doi:10.1109/36.581987](https://doi.org/10.1109/36.581987).
- Kotchenova, S. Y., Vermote, E. F., Matarrese, R., and Klemm, F. J. (2006).
  Validation of a vector version of the 6S radiative transfer code for atmospheric
  correction of satellite data. Part I: Path radiance. *Applied Optics*,
  45(26), 6762–6774. [doi:10.1364/AO.45.006762](https://doi.org/10.1364/AO.45.006762).
