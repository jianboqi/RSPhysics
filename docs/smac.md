# SMAC: independent band-wise atmospheric conversion

SMAC approximates atmospheric radiative transfer with analytical parameterizations.
This implementation cites Rahman & Dedieu (1994) and uses a pinned Hagolle/CNES
Python implementation preserved/updated in ESA Sen2Like as its numerical reference.
Algorithms and coefficient loading live in the public `rsphysics-smac` Rust crate;
Python and the separate product WASM reuse that kernel.[^1][^2][^3]

## 📘 Usage

```python
import rsphysics as rsp

sets = rsp.atmosphere.smac_coefficients(satellite="Sentinel-2C / MSI")
out = rsp.atmosphere.smac(
    coefficient_id="sentinel-2c.msi.B04.continental",
    direction="toa_to_surface", reflectance=0.2,
    solar_zenith_deg=30, view_zenith_deg=5, relative_azimuth_deg=60,
    pressure_hpa=1013.25, aot550=0.1,
    ozone_atm_cm=0.3, water_vapour_g_cm2=2,
)
surface_reflectance = out["result"]["reflectance"]

# Reverse direction: simulate TOA from Lambertian surface reflectance.
forward = rsp.atmosphere.smac(direction="surface_to_toa", reflectance=0.3)

# Exact original text, pinned URL, SHA-256; multi-band files include a zero-based column.
source = rsp.atmosphere.smac_coefficient_source(sets[0]["id"])
assert source["text"].startswith("# Generated")
```

The generic call is `rsp.run("atmosphere.smac", parameters)`. It supplies defaults,
strict unknown-field checking, selector-aware JSON Schema, actual parameters,
and resource hashes. CLI uses the same model. Use `rsp.run_batch` for mixed
sensor IDs or directions supplied as per-row mappings.

## 📘 Quantities and algorithm

Inputs are dimensionless reflectances, not DN or radiance. Forward surface input
is 0–1; correction TOA input is 0–2. Angles use degrees. Zeniths are measured from
the local vertical. Relative azimuth is original SMAC `abs(phis-phiv)`, folded to
0–180°. The scattering cosine is
`−[cosθs cosθv + sinθs sinθv cos(phis−phiv)]`; conventions are not borrowed from
other models.

| Input | Unit/meaning | Engineering domain |
|---|---|---|
| `solar_zenith_deg`, `view_zenith_deg` | Degrees | 0–70 |
| `relative_azimuth_deg` | Degrees | 0–180 |
| `pressure_hpa` | Local surface pressure, not sea-level pressure | 300–1100 hPa |
| `aot550` | Aerosol optical depth at 550 nm | 0–1 |
| `ozone_atm_cm` | Vertical ozone column | 0–1 atm·cm; 0.3 = 300 DU |
| `water_vapour_g_cm2` | Vertical water-vapour column | 0–10 g/cm²; 1 g/cm² = 10 kg/m² |

These bounds prevent obvious extrapolation; they are not accuracy guarantees
for every coefficient vintage, band, or input combination.

Forward conversion is
`ρTOA = Tg [ρatm + Td Tu ρsurface / (1 − S ρsurface)]`; correction uses its
analytical inverse. `Tg` combines downward/upward gaseous transmission; `Td` and
`Tu` are downward/upward scattering transmissions. `ρatm` is atmospheric
reflectance before the gas multiplier, and `S` is atmospheric spherical albedo.
Raw Rayleigh/aerosol outputs exclude some fitted residuals; their direct sum
must not replace `ρatm`.

Outputs outside 0–1 are retained and marked by `outside_unit_interval=1`.
Small denominators or non-finite intermediates produce `out_of_domain`.
The 21-point `sweep_aot550` / `sweep_reflectance` changes only AOT550 while
holding input reflectance, coefficients, geometry, pressure, and gas columns
fixed. It is a single-model sensitivity experiment, not aerosol inversion or
generic model coupling.

## 📘 Coefficients and compression

367 entries from 331 original files share one `coefficients.json.zlib` resource:
82,152 bytes. Original text totals 219,290 bytes; including index, provenance,
and checksums, compression saves approximately 62.5% and avoids hundreds of
runtime files. Source text is stored once; Sentinel-2's 13 columns share a file.
Rust decompresses/caches once and parses f64 values. There is no rounding,
reduced precision, interpolation, or refitting. The resource and every original
text checksum are verified.

| Common sensor | Included sets | Source/boundary |
|---|---|---|
| Sentinel-2A/B/C MSI | 13 bands per platform, continental | CNES-generated 2024 tables preserved in pinned ESA Sen2Like multi-column format |
| Landsat-8 OLI | B1–B9, including PAN/cirrus | ESA Sen2Like; not substituted for Landsat-9 |
| Landsat-4/5 TM, Landsat-7 ETM+ | Six reflective bands each: B1–B5/B7 | Author legacy continental sets; no thermal or invented PAN sets |
| MODIS | B1–B19/B26, continental/desert | Author legacy sets; upstream does not distinguish Terra/Aqua or a current platform SRF version |
| Other legacy sensors | ASTER, MERIS, MISR, SPOT, AVHRR, SeaWiFS, MSG, selected commercial sensors | `legacy.*` preserves original names; inspect the catalog for exact scope |

RSR curves are not substituted for SMAC coefficients: the latter are fitted
atmospheric parameters, not spectral-response functions. Continental/desert sets
are different fits and must be chosen for the intended conditions. Strongly
absorbing water-vapour/cirrus bands, large angles, high aerosol loads, or complex
surfaces can be unreliable. Inclusion supports original-reference/controlled
exploration, not a claim that every band is suitable for surface retrieval.[^2]

## 📘 NumPy batches

```python
table = rsp.parameter_grid("atmosphere.smac", {"aot550": [0.05, 0.1, 0.2]})
batch = rsp.atmosphere.smac_batch(
    table, coefficient_id="landsat-8.oli.B4.continental",
    direction="toa_to_surface", shared_axes=True,
)
# Equivalent generic path; string selectors are separate from numerical columns.
same = rsp.run_array("atmosphere.smac", table, shared_axes=True,
    fixed={"coefficient_id": "landsat-8.oli.B4.continental",
           "direction": "toa_to_surface"})
```

`parameters_for("atmosphere.smac")` / `atmosphere.SMAC_PARAMETERS` specify eight
columns. `fixed` accepts only `coefficient_id` and `direction`, never numerical
column overrides. Each manifest records both actual selectors. Empty batches,
strided views, byte-order conversion, float32 promotion, GIL release, and chunked
`iter_array(..., fixed=...)` are supported. Scientific arrays do not traverse
JSON; small selectors and provenance manifests do.

## 📘 Reference, licensing, and cleanup

839 original cases cover both directions for every coefficient set, random
inputs, and boundaries; two additional full AOT curves are retained. Comparisons
check individual transport quantities, not only final reflectance. Original code
executes unchanged without importing the production Rust evaluator. This is
implementation agreement, not observational accuracy validation.

The particular Sen2Like SMAC copy explicitly states GPLv3, so the current Rust
adaptation retains `GPL-3.0-only`, full license text, and provenance. Sen2Like's
top-level Apache-2.0 does not override that component notice. No separate data
license was found in the author's old repository; no public-domain status is
inferred. Project-original code and website now target Apache-2.0; SMAC remains
included while additional permission awaits the relevant rights holders. See
[ADR 0004](decisions/0004-apache-project-license.md) and [license scope](licensing.md).
No PyPI or website release has occurred. Data permissions and applicable complete
source delivery remain release considerations.

All original code/downloads and reference scratch files use `.reference-cache/smac`.
Normal execution/tests only need the packed resource, manifest, and fixture:

```sh
python tooling/smac-data/fetch.py
python tooling/smac-data/prepare.py
python benchmarks/reference/smac/generate.py
python tooling/smac-data/verify.py
python tooling/reference-runner/clean.py --model smac --remove
```

[^1]: Rahman & Dedieu (1994). SMAC. https://doi.org/10.1080/01431169408954055
[^2]: Author implementation, pinned commit. https://github.com/olivierhagolle/SMAC/tree/77bf73dd1f2a2b6a33a97b8997da469210f9ce3e
[^3]: ESA Sen2Like SMAC, pinned commit. https://github.com/senbox-org/sen2like/tree/6dd3f0da82a7461b03a2914df56d305070e50d6d/sen2like/sen2like/atmcor/smac

Credit CNES for calculating the relevant sensor coefficients.
