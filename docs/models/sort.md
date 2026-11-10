# SORT

SORT is a directional soil optical radiative transfer model for dry and water-film-covered soil. A three-component spectral basis describes dry soil; particle scattering, shadow hiding, packing, and macroscopic roughness determine angular reflectance.

## How it works

The dry basis combines a brightness multiplier with two spherical mixing angles. A water film uses a complex-permittivity mixture of water and particles, Fresnel interface transmission, absorption, and internal feedback. Covered and uncovered fractions are combined with the model's nonlinear mixing relation.

SORT converts the dry basis to single-scattering albedo and applies a two-term Henyey–Greenstein phase function, a shadow-hiding hotspot, and a packing correction. The water-covered substrate uses refracted solar and viewing angles.

## Usage

```python
import rsphysics as rsp

soil = rsp.soil.sort(water_coverage=0.4, water_film_thickness_mm=0.3)
wavelengths = soil["result"]["wavelengths_nm"]
dry = soil["result"]["dry_brf"]
wet = soil["result"]["wet_brf"]
```

NumPy batches use `parameters_for("soil.sort")` column order:

```python
matrix = rsp.parameter_grid("soil.sort", {"water_coverage": [0.0, 0.3, 0.8]})
batch = rsp.soil.sort_batch(matrix)
```

Use `roughness_correction="hapke_single"` or `"hapke_multi"` in either the scalar or dedicated batch interface to select a roughness operator. Batch rows share this string selector; numeric parameters may vary by row.

## Parameters

| Parameter | Meaning | Unit | Default | Range / options |
|---|---|---|---|---|
| `hotspot_amplitude` | Shadow-hiding hotspot amplitude B0. | 1 | `0.4` | 0.0–2.0 |
| `hotspot_width` | Positive hotspot width h. | 1 | `0.1` | 0.0001–2.0 |
| `multifacet_reflectance` | Diffuse reflectance factor for Hapke multifacet roughness rescaling. | 1 | `0.0` | 0.0–1.0 |
| `particle_extinction_coefficient` | Non-negative imaginary refractive index of film particles. | 1 | `0.001` | 0.0–0.1 |
| `particle_refractive_index` | Real refractive index of particles in the film. | 1 | `1.53` | 1.0001–3.0 |
| `particle_volume_fraction` | Particle volume fraction within the water film. | 1 | `0.05` | 0.0–1.0 |
| `phase_asymmetry` | Two-term Henyey-Greenstein asymmetry parameter b. | 1 | `0.4` | 0.0–0.9999 |
| `phase_backscatter` | Two-term phase mixing parameter c, subject to a joint non-negativity constraint. | 1 | `0.0` | -0.9999–1.4999 |
| `relative_azimuth_deg` | Folded relative azimuth; equal zeniths and zero azimuth define the hotspot. | deg | `0.0` | 0.0–180.0 |
| `rms_slope` | Dimensionless optical RMS slope; inactive with no roughness correction. | 1 | `0.4` | 0.0–2.0 |
| `roughness_correction` | Macroscopic surface roughness operator. | 1 | `none` | `none`, `hapke_single`, `hapke_multi` |
| `soil_brightness` | Soil spectral brightness multiplier. | 1 | `0.6` | 0.0–1.5 |
| `solar_zenith_deg` | Solar zenith relative to the local surface normal. | deg | `45.0` | 0.0–85.0 |
| `solid_filling_factor` | Solid filling factor F; porosity equals one minus F. | 1 | `0.2` | 0.0–0.75 |
| `spectral_alpha_deg` | First spherical mixing angle of the soil spectral basis. | deg | `15.0` | -90.0–90.0 |
| `spectral_gamma_deg` | Second spherical mixing angle of the soil spectral basis. | deg | `50.0` | 0.0–90.0 |
| `view_zenith_deg` | Viewing zenith relative to the local surface normal. | deg | `0.0` | 0.0–85.0 |
| `water_coverage` | Water-film covered surface fraction. | 1 | `0.3` | 0.0–1.0 |
| `water_film_thickness_mm` | Water-film thickness in millimetres. | mm | `0.3` | 0.0–10.0 |

The spectral mixing angles are basis coefficients, not illumination or viewing angles. Film thickness is in **mm**; water absorption coefficients in the embedded basis are in cm⁻¹. Coverage is an area fraction, not volumetric soil moisture.

## Outputs

- `wavelengths_nm`: 2,001 samples from 400 to 2400 nm at 1 nm spacing.
- `dry_brf`: dry-soil directional BRF.
- `wet_brf`: the water-film-covered / uncovered mixture.

BRF equals π × BRDF and may exceed one in individual directions. A high directional BRF does not by itself imply that hemispherical reflected energy exceeds incident energy. Dry-boundary energy is screened at the supplied incident and viewing zeniths with refined quadrature.

If coverage or film thickness is zero, wet and dry outputs are exactly equal. The dry and wet spectra represent alternative soil states; they must not be added as reflection and transmission.

## Scope

- Local-normal solar and viewing zeniths: 0–85°. Folded relative azimuth: 0–180°. Equal zeniths and zero relative azimuth define the hotspot direction.
- Roughness operators: `none`, `hapke_single`, and `hapke_multi`. RMS slope is inactive with `none`; `hapke_multi` also uses the diffuse reflectance factor.
- Water transport uses the scalar feedback closure with refracted substrate geometry. Shiltz slope integrals and angular adding are not provided. The wet result is not a strictly reciprocal general-purpose BSDF.
- Parameter combinations must preserve nonnegative optical responses and the applicable bounded or energy constraints. Input intervals are computational support ranges, not a guarantee of observational accuracy.
- Fixed embedded spectral basis; no inversion, calibration, or model coupling.

## Authors and references

Anxin Ding (丁安心), Fujian Normal University. Contact: danxin@whu.edu.cn; danxin@fjnu.edu.cn.

Ding, A. (2026). *SORT / SORT-L: Soil Optical Radiative Transfer Model*. Model documentation, version 1.0.0.


## Roughness and water-film modes

`roughness_correction` accepts `none`, `hapke_single`, `hapke_multi`,
`shiltz_single` and `shiltz_full`. Shiltz uses a convergence-checked Gaussian
slope integral; the full mode adds the multifacet contribution. These modes
can be substantially slower than the Hapke approximations.

`substrate_angle_mode` is `refracted` by default. `external` reproduces the
external-angle substrate convention with legacy scalar water transport.
`water_film_mode` accepts:

- `legacy_scalar`: original scalar water-film closure and diffuse transmission.
- `single_pass`: directional Beer–Lambert attenuation and the n^-2 radiance
  Jacobian; omits internal return paths.
- `adding`: experimental, convergence-checked angular transport with repeated
  substrate/interface reflections, including azimuth reversal at the interface.

Directional modes require refracted substrate angles. Adding with nonzero
Shiltz roughness is explicitly unvalidated in the scientific reference and is
rejected. Energy-invalid, singular or nonconverged configurations return errors;
results are not normalized or clipped into range.

```python
wet = rsp.soil.sort(water_film_mode="single_pass")
```

The additional outputs `dry_basis_reflectance`, `single_scattering_albedo` and
`substrate_brf` expose the spectral basis and the covered boundary before film
propagation. All three selectors are available in `sort_batch`.
