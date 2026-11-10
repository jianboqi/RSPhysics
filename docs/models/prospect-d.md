# PROSPECT-D

PROSPECT-D predicts leaf reflectance and transmittance from leaf structure, pigments, water, and dry matter over 400–2500 nm.

## How it works

Absorption by biochemical constituents is combined with internal scattering and layer interactions. The structural parameter N controls optical structure; it is not a directly measured count of cell layers.

## Usage

```python
import rsphysics as rsp

leaf = rsp.vegetation.prospect_d(chlorophyll=45, water=0.015)
wavelengths = leaf['result']['wavelengths_nm']
reflectance = leaf['result']['reflectance']
transmittance = leaf['result']['transmittance']
```

Model ID: `vegetation.prospect-d`. Use the same ID with `rsp.run`, `rsp.run_array`, or
`rsp.parameter_grid`; see [Basic usage](../usage.md).

## Parameters

| Parameter | Default | Unit | Supported range |
|---|---|---|---|
| `anthocyanins` | `0` | ug cm^-2 | 0.0–100.0 |
| `brown_pigments` | `0` | 1 | 0.0–5.0 |
| `carotenoids` | `8` | ug cm^-2 | 0.0–100.0 |
| `chlorophyll` | `40` | ug cm^-2 | 0.0–200.0 |
| `dry_matter` | `0.008` | g cm^-2 | 0.0–0.05 |
| `incidence_angle_deg` | `40` | deg | 1.0–90.0 |
| `n` | `1.5` | 1 | 1.0–5.0 |
| `water` | `0.01` | g cm^-2 | 0.0–0.1 |

## Outputs

| Field | Meaning |
|---|---|
| `wavelengths_nm` | Wavelengths, nm; 2,101 samples at 1 nm spacing |
| `reflectance / transmittance` | Dimensionless leaf reflectance and transmittance |

## Scope

The incidence input is the illumination-cone half-angle, not a canopy solar/view zenith. This is leaf optics; canopy structure and directional canopy reflection are handled by PROSAIL-D.

## Authors and references

J.-B. Féret, A. A. Gitelson, S. D. Noble, and S. Jacquemoud.

- [Féret et al. (2017). PROSPECT-D: Towards modeling leaf optical properties through a complete lifecycle. Remote Sensing of Environment.](https://doi.org/10.1016/j.rse.2017.03.004)
