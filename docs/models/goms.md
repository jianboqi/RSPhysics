# GOMS

GOMS predicts directional reflectance of discrete ellipsoidal tree crowns,
including crown shape and mutual shadowing. It supports terrain slope/aspect
and 1–10 spectral bands.

## How it works

Solar and viewing geometry determine projected crown overlap and visible
sunlit surfaces. Crown density, shape, height and height variation determine
three mixing weights: sunlit ground KG, sunlit crown KC and a shared shaded
term KZ. Per-band reflectance is `KG*RG + KC*RC + KZ*RZ`.

The parameterization shares one shaded reflectance between crown and
ground. It does not identify four separate component fractions.

## Usage

```python
import rsphysics as rsp

canopy = rsp.vegetation.goms(
    normalized_crown_density=0.1, crown_axis_ratio=1.733,
    crown_height_ratio=2.577, relative_height_variation=0.769,
    sunlit_ground_reflectance=[0.2, 0.3],
    sunlit_crown_reflectance=[0.55, 0.65], shaded_reflectance=[0.05, 0.08],
    solar_zenith_deg=45., solar_azimuth_deg=30.,
    view_zenith_deg=20., view_azimuth_deg=60., slope_deg=10., slope_aspect_deg=90.)
brf = canopy['result']['brf']
```

All three coefficient vectors must have the same length, from one to ten. The
convenience function accepts scalar coefficients and broadcasts them to the
explicit band count; `rsp.run` expects vectors. No wavelength is inferred.
Normalized crown density is number density multiplied by squared horizontal
crown radius, not stems per square metre. Azimuths and terrain aspect share
the same directional reference.

## Parameters

| Parameter | Unit | Meaning |
|---|---|---|
| `normalized_crown_density` | 1 | Crown number density times squared horizontal crown radius |
| `crown_axis_ratio` | 1 | Vertical-to-horizontal ellipsoid semi-axis ratio |
| `crown_height_ratio` | 1 | Mean crown-center height divided by vertical semi-axis |
| `relative_height_variation` | 1 | Crown-center height distribution parameter |
| `sunlit_ground_reflectance` | 1 | Per-band sunlit-ground reflectance |
| `sunlit_crown_reflectance` | 1 | Per-band sunlit-crown reflectance |
| `shaded_reflectance` | 1 | Per-band shared shaded-crown / shaded-ground reflectance |
| `solar_zenith_deg`, `view_zenith_deg` | degrees | Solar and viewing zenith angles |
| `solar_azimuth_deg`, `view_azimuth_deg` | degrees | Solar and viewing azimuths in a shared reference frame |
| `slope_deg`, `slope_aspect_deg` | degrees | Terrain slope and aspect |
| `geometry_convention` | — | `area_consistent` (default) or `legacy_c`; see below |

Use `rsp.describe('vegetation.goms')['input_schema']` for defaults and accepted
numeric ranges.

## Outputs

| Field | Meaning |
|---|---|
| `brf / brdf` | Per-band reflectance factor; BRDF = BRF/pi, sr^-1 |
| `sunlit_ground_weight / sunlit_crown_weight / shaded_weight` | Three geometric weights, shared across bands |
| `invalid_weight_count` | Selected-point weights outside [0,1], retained as diagnostics |
| `principal_brf` | Shape (band,171), signed view zenith -85..85 degrees |
| `hemisphere_brf` | Shape (band,648), view zenith 0..85 step5 and azimuth 0..350 step10 |
| `principal_valid / hemisphere_valid` | One-dimensional validity masks; zero denotes an invalid local direction |
| `hotspot_brf / hotspot_shaded_weight` | Reflectance and shaded weight at exact sun-observer alignment |
| `hotspot_view_zenith_deg / hotspot_view_azimuth_deg` | Hotspot direction, equal to the solar direction |
| `principal_shaded_weight` | Visible shaded weight along the principal plane |

The angular arrays contain zero placeholders where their mask is zero. These
placeholders are not physical zero reflectance. Selected-point invalid local
geometry returns an error.

Positive principal-plane zenith angles use the solar azimuth; negative angles
use the opposite azimuth. At exact sun-observer alignment, the shaded weight
is zero in the area-consistent convention. This geometric hotspot need not be
the global maximum of the complete BRF curve, which also depends on component
reflectances and crown visibility at large angles.

## Numerical conventions

`geometry_convention="area_consistent"` is the default. General overlap area
uses the same units as principal-plane overlap. At nadir, this gives the
Poisson ground probability exp(-pi*normalized_density). Azimuth differences are
folded consistently, slope rotation updates projected tangent distances, atan2
defines the slope-frame azimuth, and all nonzero slopes are applied.

`geometry_convention="legacy_c"` retains supplied C arithmetic for comparison,
including its approximate pi, overlap scaling and slope threshold. Out-of-range
geometric weights are reported rather than reinterpreted as valid fractions.

## NumPy batches

`rsp.vegetation.goms_batch` accepts a single-band table with 13 columns in
`GOMS_PARAMETERS` order. For multiband arrays, use
`rsp.vegetation.goms_multiband_batch(geometry, reflectances)`: geometry has shape
`(batch,10)` in `GOMS_GEOMETRY_PARAMETERS` order; reflectances have shape
`(batch,band,3)`, with the last axis ordered sunlit ground, sunlit crown and
shared shade. With shared axes, angular coordinates are not repeated for each
batch row. `rsp.run_batch` also accepts multiband parameter mappings.

## Scope and reference

This is a crown-scale geometric-optical approximation with prescribed component
reflectances. It does not solve crown-interior multiple scattering, inversion
or model coupling.

Xiaowen Li and Alan H. Strahler.

- [Li & Strahler (1992). Geometric-optical bidirectional reflectance modeling of the discrete crown vegetation canopy: Effect of crown shape and mutual shadowing. IEEE TGRS 30(2), 276–292.](https://doi.org/10.1109/36.134078)
