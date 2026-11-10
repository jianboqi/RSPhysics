# SORT-L

SORT-L describes angle-independent Lambertian soil reflectance using a three-component dry-soil spectral basis and a diffuse water-film transport approximation.

## How it works

The dry basis combines a brightness multiplier with two spherical mixing angles. A water film uses a complex-permittivity mixture of water and particles, Fresnel interface transmission, absorption, and internal feedback. Covered and uncovered fractions are combined with the model's nonlinear mixing relation.

## Usage

```python
import rsphysics as rsp

soil = rsp.soil.sort_l(water_coverage=0.4, water_film_thickness_mm=0.3)
wavelengths = soil["result"]["wavelengths_nm"]
dry = soil["result"]["dry_reflectance"]
wet = soil["result"]["wet_reflectance"]
```

NumPy batches use `parameters_for("soil.sort-l")` column order:

```python
matrix = rsp.parameter_grid("soil.sort-l", {"water_coverage": [0.0, 0.3, 0.8]})
batch = rsp.soil.sort_l_batch(matrix)
```

## Parameters

| Parameter | Meaning | Unit | Default | Range / options |
|---|---|---|---|---|
| `particle_extinction_coefficient` | Non-negative imaginary refractive index of film particles. | 1 | `0.001` | 0.0–0.1 |
| `particle_refractive_index` | Real refractive index of particles in the film. | 1 | `1.53` | 1.0001–3.0 |
| `particle_volume_fraction` | Particle volume fraction within the water film. | 1 | `0.05` | 0.0–1.0 |
| `soil_brightness` | Soil spectral brightness multiplier. | 1 | `0.6` | 0.0–1.5 |
| `spectral_alpha_deg` | First spherical mixing angle of the soil spectral basis. | deg | `20.0` | -90.0–90.0 |
| `spectral_gamma_deg` | Second spherical mixing angle of the soil spectral basis. | deg | `50.0` | 0.0–90.0 |
| `water_coverage` | Water-film covered surface fraction. | 1 | `0.5` | 0.0–1.0 |
| `water_film_thickness_mm` | Water-film thickness in millimetres. | mm | `0.3` | 0.0–10.0 |

The spectral mixing angles are basis coefficients, not illumination or viewing angles. Film thickness is in **mm**; water absorption coefficients in the embedded basis are in cm⁻¹. Coverage is an area fraction, not volumetric soil moisture.

## Outputs

- `wavelengths_nm`: 2,001 samples from 400 to 2400 nm at 1 nm spacing.
- `dry_reflectance`: dry-soil Lambertian reflectance.
- `wet_reflectance`: the water-film-covered / uncovered mixture.

Lambertian reflectance lies in [0, 1]; divide by π when a BRDF value in sr⁻¹ is required.

If coverage or film thickness is zero, wet and dry outputs are exactly equal. The dry and wet spectra represent alternative soil states; they must not be added as reflection and transmission.

## Scope

- Lambertian, angle-independent soil response. No directional hotspot or macroscopic roughness operator.
- Parameter combinations must preserve nonnegative optical responses and the applicable bounded or energy constraints. Input intervals are computational support ranges, not a guarantee of observational accuracy.
- Fixed embedded spectral basis; no inversion, calibration, or model coupling.

## Authors and references

Anxin Ding (丁安心), Fujian Normal University. Contact: danxin@whu.edu.cn; danxin@fjnu.edu.cn.

Ding, A. (2026). *SORT / SORT-L: Soil Optical Radiative Transfer Model*. Model documentation, version 1.0.0.
