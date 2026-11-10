# rsphysics Python package

RSPhysics provides Rust-backed physical models and numerical tools through Python and NumPy. This guide covers installation, model calls, batch computation, and physical input/output conventions. For the library overview, see the [project README](../../README.md).

```python
import numpy as np
import rsphysics as rsp

leaf = rsp.run("vegetation.prospect-d", params={"chlorophyll": 45.0})
wavelength = leaf["result"]["wavelengths_nm"]   # float64, shape (2101,)
reflectance = leaf["result"]["reflectance"]
manifest = leaf["manifest"]
leaf = rsp.vegetation.prospect_d(n=1.5, chlorophyll=40, water=0.01, dry_matter=0.008)
parameters = np.array([[1.5, 40, 8, 0, 0, 0.01, 0.008, 40],
                       [2.0, 60, 12, 0, 0, 0.02, 0.010, 40]], dtype=np.float64)
batch = rsp.vegetation.prospect_d_batch(parameters)
assert batch["result"]["reflectance"].shape == (2, 2101)
```

Batch columns follow `PROSPECT_D_PARAMETERS`: `n, chlorophyll, carotenoids, anthocyanins, brown_pigments, water, dry_matter, incidence_angle_deg`. Pigments use ug cm^-2; water and dry matter use g cm^-2. The incidence parameter is the maximum illumination-cone angle in degrees, not the observer zenith. `n` and `brown_pigments` are dimensionless; the latter retains the original model's arbitrary units.

Supported leaf domains are n∈[1,5], chlorophyll∈[0,200], carotenoids/anthocyanins∈[0,100], brown_pigments∈[0,5], water∈[0,0.1], dry_matter∈[0,0.05], and incidence_angle_deg∈[1,90]. These are implementation limits, not a statement that every combination is biologically meaningful. PROSPECT-PRO and inversion are not provided.

```python
canopy = rsp.vegetation.prosail_d(leaf_area_index=3, mean_leaf_angle_deg=60,
                                solar_zenith_deg=30, view_zenith_deg=0,
                                relative_azimuth_deg=60, soil_dry_fraction=0.5)
brf = canopy["result"]["brf"]  # float64 (2101,); may exceed 1
hemi = canopy["result"]["hemispherical_reflectance"]
rows = rsp.run_batch("vegetation.prosail-d", [{"leaf_area_index": 1}, {"leaf_area_index": 5}])
```

PROSAIL-D fixes PROSPECT-D + 4SAIL, the author's Campbell 13-bin distribution, direct illumination, and original dry/wet soil spectra. Added domains: LAI∈[0,10], mean leaf angle∈[5,85]°, hotspot∈[0,1], solar/view zenith∈[0,85]°, relative azimuth∈[0,180]°, soil brightness/dry-soil weight∈[0,1]. Dry-soil weight is not soil moisture. Outputs are BRF (original `rsot`) and directional–hemispherical reflectance (`rsdt`), which must not be added as R/T. LAI=0 uses the analytical bare-soil limit; positive LAI with water=dry_matter=0 remains unsupported.

`prosail_d_batch(matrix)` accepts `(batch,16)` in `PROSAIL_D_PARAMETERS` order: the eight leaf parameters, followed by `leaf_area_index, mean_leaf_angle_deg, hotspot, solar_zenith_deg, view_zenith_deg, relative_azimuth_deg, soil_brightness, soil_dry_fraction`. Rust executes the batch. Scientific arrays use native NumPy buffers, returning `(batch,2101)` spectra and a shared wavelength axis.

`list_models()` / `describe(id)` return installed models, units, and schemas. `run_batch(id, [params,...])` is generic ordered execution. Dedicated NumPy batches loop in Rust and return two-dimensional spectra plus a manifest per row. Errors are `ModelError` with `kind` and `message`; batches fail on the first error and report its zero-based row.

Original references execute the author's pinned `prospect` v1.7.0 / `prosail` v3.1.1 R scientific functions independently of Rust. PROSPECT has 32 cases; PROSAIL has 24 original cases plus one separately labelled bare-soil limit. Downloads/reference environments are isolated in disposable `.reference-cache`; required runtime coefficients and offline regression spectra remain.

Build locally from the repository root with `python -m maturin build --manifest-path bindings/python/Cargo.toml --release --out dist`, then install the resulting platform Wheel with pip. Building requires Rust; installing a compatible Wheel does not. Formal distribution still requires resolution of applicable code/data permissions. Packaged sources are documented in `THIRD_PARTY_NOTICES.md`.

Build a source archive with `python -m maturin sdist --manifest-path bindings/python/Cargo.toml --out dist`. Test source installation using a separate `CARGO_TARGET_DIR`, such as `target/sdist-build`, to prevent different source trees from sharing cached relative paths. CI separately configures Wheel and source-archive installation tests.

## 📘 Ross–Li directional reflectance

```python
direction = rsp.surface.ross_li(f_iso=0.2, f_vol=0.1, f_geo=0.02,
                               solar_zenith_deg=30, view_zenith_deg=20,
                               relative_azimuth_deg=0)
brf = direction["result"]["brf"]    # Python float, dimensionless
brdf = direction["result"]["brdf"]  # Python float，sr^-1
plane = direction["result"]["principal_brf"]  # float64 (171,)
hemisphere = direction["result"]["hemisphere_brf"]  # float64 (648,)
```

The fixed variant is RossThick–LiSparseReciprocal, h/b=2 and b/r=1, without hotspot enhancement. Three 0–1 weights apply to one band; no automatic spectral weights or wavelength axis are supplied. Solar/view zeniths span 0–85° and relative azimuth 0–180°; 0° is the solar azimuth/backscattering direction. Outputs include a point, kernels, and two grids: principal plane −85..85° every 1° (negative: relative azimuth 180°; positive: 0°), and a zenith-major hemisphere grid, zenith 0..85° every 5° / azimuth 0..350° every 10°. The remaining azimuth half uses symmetry.

BRF may exceed 1. Invalid weights can produce negative BRF, which is retained and counted in `negative_sample_count`, not automatically accepted as physical. Kernels themselves may legitimately be negative. Inversion, albedo integration, and generic coupling are not implemented.

`rsp.surface.ross_li_batch(matrix)` accepts `(batch,6)` in `ROSS_LI_PARAMETERS` order: `f_iso, f_vol, f_geo, solar_zenith_deg, view_zenith_deg, relative_azimuth_deg`. Scalars become `(batch,)`; angular arrays are `(batch,171)` / `(batch,648)`. `rsp.run` / `rsp.run_batch` invoke the same Rust solver.

The 32 original AMBRALS C cases and independent 80-digit formulas use separate precision-appropriate tolerances. Sources/errors are in `benchmarks/reference/ross-li`. Original C is only used in the disposable reference environment and is not distributed in the installation package.

## 📘 Two independent microwave models

```python
passive = rsp.microwave.tau_omega(
    soil_temperature_k=300, vegetation_temperature_k=295,
    soil_reflectivity=0.3, vegetation_optical_depth=0.3,
    single_scattering_albedo=0.05, observation_zenith_deg=40)
tb = passive['result']['brightness_temperature_k']  # Scalar, K
active = rsp.microwave.water_cloud(
    vegetation_water_content=1, scattering_coefficient=0.1,
    attenuation_coefficient=0.2, soil_backscatter=0.1,
    observation_zenith_deg=40)
sigma0 = active['result']['sigma0']  # Linear power, never dB
```

Tau–omega uses SMAP ATBD Revision B §2.1 Eq.1, with separate soil/vegetation temperatures (150–350 K). r and ω are 0–1; τ is nadir optical depth (0–5), not slant-path depth. It returns total TB, three K-valued contributions (soil emission, direct vegetation emission, soil-reflected vegetation emission), and one-way transmissivity. Sky, atmosphere, and higher-order scattering are omitted; no soil dielectric mixing or moisture conversion is performed.

WCM fixes V1=V2=VWC (0–10 kg/m²). A and B (each 0–2 m²/kg) must match frequency, polarization, vegetation, and descriptor; defaults are illustrative. Bare-soil sigma0 is supplied directly (0–10 linear power), never dB. Outputs are total sigma0, vegetation, attenuated soil, and two-way transmissivity. Exact zero stays zero; sigma0 may exceed 1. For display, positive values use 10 log10(sigma0); exact zero corresponds to −∞.

Both models support θ=0–80° and return float64 `(81,)` `sweep_zenith_deg` / `sweep_total`. Only θ varies; all other inputs, including soil response, stay fixed. This is a sensitivity experiment, not a complete physical angular surface model. There is no wavelength axis, inversion, or generic coupling.

`tau_omega_batch(matrix)` / `water_cloud_batch(matrix)` accept `(batch,6)` / `(batch,5)` in `TAU_OMEGA_PARAMETERS` / `WATER_CLOUD_PARAMETERS` order, matching the keywords above. Scalars become `(batch,)`, curves `(batch,81)`; empty batches preserve shape. Generic `run` / `run_batch` use the same solvers.

Each model has 32 independent 80-digit formula references. WCM also checks canopy/attenuation terms from a pinned SenSE commit. Tau–omega does not claim execution of the operational SMAP processor. Numerical agreement is not sensor calibration or observational validation; see `benchmarks/reference/microwave/README.md`.

## 📘 Shared spectral, sensor, and calibration tools

```python
import rsphysics as rsp
canopy = rsp.vegetation.prosail_d()['result']
oli = rsp.sensors.load('landsat-8.oli')  # Bundled data; offline loading
bands = oli.convolve(canopy['wavelengths_nm'], canopy['brf'])
print(bands['bands'], bands['values'], bands['coverage'])

s2 = rsp.sensors.fetch('sentinel-2a.msi')  # Explicit official v5 acquisition into local cache
s2_bands = s2.convolve(canopy['wavelengths_nm'], canopy['brf'])
```

`rsp.spectral` provides linear resampling, interval integration, custom-response integration, and optional incident-spectrum weighting. All numerical algorithms are in Rust; wavelengths use nm. Missing nonzero response coverage is an error by default. `allow_partial=True` explicitly permits partial coverage and returns response-area coverage. A response-weighted reflectance mean differs from incident-spectrum-weighted band reflectance; no solar spectrum or atmospheric correction is added implicitly.

The registry has 12 profiles / 112 curves: Sentinel-2 A/B/C MSI v5; Landsat 4/5 TM, 7 ETM+, 8/9 OLI, 8/9 TIRS; and Terra/Aqua MODIS (the 16 NASA OBPG reflective RSRs, not all 36 bands). Nine profiles / 73 curves are bundled. ESA responses are acquired/imported locally to avoid redistributing restricted material. Each profile pins raw/processed SHA-256, version, source, and preparation notes. `load` never accesses the network.

`run_array(id, matrix)` provides the unified native NumPy path for the five initial models; SMAC selectors are described below. `parameters_for` supplies column order, `parameter_grid` constructs explicit grids, and `iter_array` yields chunks. Dedicated `*_batch` defaults remain compatible. `shared_axes=True` moves fixed wavelengths/angles to `coordinates`. Only provenance/selectors use JSON; scientific arrays do not.

`rsp.radiometry` provides `linear_calibrate`, `dn_to_toa_reflectance`, `radiance_to_toa_reflectance`, and `radiance_to_brightness_temperature`. Supply coefficients from product metadata; results contain `values/valid/flags/unit`. Invalid DN and saturation thresholds are not guessed. Negative or above-one reflectances are not clipped. Level-1 versus Level-2 scaling, solar zenith, radiance per nm/per um, and brightness versus land-surface temperature are distinguished explicitly.

Full contracts, cache locations, units, examples, and validation are in `docs/python-infrastructure.md`; run `bindings/python/examples/satellite_band_workflow.py`. This stage includes no generic coupling, inversion, or complete image-product reader.

## 📘 Fresnel and Planck physics

`rsp.optics.fresnel(theta_deg, n_transmitted, n_incident=1)` returns s/p/unpolarized power reflectance, interface flux entering the second medium, and complex amplitudes. Passive complex indices and TIR are supported. `fresnel_permittivity` accepts relative permittivity for nonmagnetic media. The incident medium must be real and lossless. Complex values use exp(-iωt) and n+iκ (κ≥0); θ is measured from the normal, 0≤θ<90°. Interface entering flux is not whole-layer transmission through an absorbing slab.

`rsp.radiometry.planck_radiance(wavelength_nm, temperature_k, radiance_unit=...)` / `inverse_planck` provide wavelength form; `planck_frequency_radiance` / `inverse_planck_frequency` provide Hz form. Outputs are broadcast NumPy arrays. Radiance includes sr⁻¹ and is not exitance. Wavelength input is always nm; density may be per m/nm/um. Inversion yields monochromatic brightness temperature, distinct from product K1/K2 conversion and land-surface temperature.

Algorithms remain in the shared Rust kernel without model coupling. See `docs/basic-physics.md` for 96 independent 80-digit references, energy checks, and blackbody limits; run `bindings/python/examples/basic_physics.py`.

## 📘 SMAC atmospheric conversion

```python
corrected = rsp.atmosphere.smac(
    coefficient_id="sentinel-2c.msi.B04.continental",
    direction="toa_to_surface", reflectance=0.2,
    pressure_hpa=1013.25, aot550=0.1,
    ozone_atm_cm=0.3, water_vapour_g_cm2=2,
)
sets = rsp.atmosphere.smac_coefficients()  # 367 exact IDs; fully offline
source = rsp.atmosphere.smac_coefficient_source(sets[0]["id"])
table = rsp.parameter_grid("atmosphere.smac", {"aot550": [0.05, 0.1, 0.2]})
batch = rsp.atmosphere.smac_batch(table, shared_axes=True)
```

Independent single-band conversion has 839 original-reference cases. Coefficient/direction selectors remain strings; NumPy matrices have eight numerical columns. Generic `run_array/iter_array` accept `fixed={"coefficient_id": ..., "direction": ...}`. Atmospheric data are not implicitly downloaded and surface models are not coupled. See the [SMAC guide](../../docs/smac.md) for units, data versions, strong-absorption limits, source notices, and cleanup.

## 📘 License and rights contact

Project-original code is Apache-2.0; historical MIT model notices remain. The package includes SMAC. Its Sen2Like reference copy's GPLv3 notice is retained; additional Apache-2.0
permission is awaiting confirmation. The current Wheel is not Apache-only. See [license scope](../../docs/licensing.md),
the [rights-contact statement](../../RIGHTS-STATEMENT.md), and the [authorization inquiry draft](../../docs/smac-authorization-request.md).

## L-MEB and canopy photosynthesis

The Python package also provides `rsp.microwave.lmeb` and
`rsp.ecosystem.gort_gpp`, with dedicated `*_batch` functions and the shared
`parameters_for`, `run_array`, `iter_array`, and `parameter_grid` interfaces.
See the [L-MEB guide](../../docs/models/lmeb.md) and
[GORT-GPP guide](../../docs/models/gort-gpp.md) for supported formulations,
physical definitions, references, and examples. Publication permission for the supplied sources and adaptations has been
confirmed by the project maintainer; component-specific terms remain recorded.
