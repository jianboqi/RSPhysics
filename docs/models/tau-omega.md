# Tau–omega

The zero-order tau–omega model predicts passive microwave brightness temperature from soil emission, vegetation attenuation, and vegetation emission.

## How it works

For a channel at angle θ, canopy transmission is `gamma = exp(-tau/cos(theta))`. Soil temperature, vegetation temperature, soil power reflectivity, optical depth, and single-scattering albedo determine the total brightness temperature.

## Usage

```python
import rsphysics as rsp

emission = rsp.microwave.tau_omega(
    soil_temperature_k=300., vegetation_temperature_k=295.,
    soil_reflectivity=0.3, vegetation_optical_depth=0.3,
    single_scattering_albedo=0.05, observation_zenith_deg=40.)
tb = emission['result']['brightness_temperature_k']
```

Model ID: `microwave.tau-omega`. Use the same ID with `rsp.run`, `rsp.run_array`, or
`rsp.parameter_grid`; see [Basic usage](../usage.md).

## Parameters

| Parameter | Default | Unit | Supported range |
|---|---|---|---|
| `observation_zenith_deg` | `40` | deg | 0.0–80.0 |
| `single_scattering_albedo` | `0.05` | 1 | 0.0–1.0 |
| `soil_reflectivity` | `0.3` | 1 | 0.0–1.0 |
| `soil_temperature_k` | `300` | K | 150.0–350.0 |
| `vegetation_optical_depth` | `0.3` | 1 | 0.0–5.0 |
| `vegetation_temperature_k` | `295` | K | 150.0–350.0 |

## Outputs

| Field | Meaning |
|---|---|
| `brightness_temperature_k` | Total surface brightness temperature, K |
| `soil_emission_k` | Attenuated soil emission, K |
| `vegetation_emission_k` | Direct upward vegetation emission, K |
| `reflected_vegetation_emission_k` | Vegetation emission reflected by soil, K |
| `transmissivity` | One-way canopy power transmission |

## Scope

Parameters describe one frequency/polarization channel. Soil reflectivity is supplied directly; it is not converted from soil moisture. The angle sweep holds all other inputs fixed. Sky/atmospheric emission and multiple scattering are excluded.

## Authors and references

Reference formulation: Peggy O’Neill, Steven Chan, Eni Njoku, Tom Jackson, and Rajat Bindlish, SMAP passive-soil-moisture algorithm team.

- [O’Neill et al. (2015). SMAP Algorithm Theoretical Basis Document: Level 2 & 3 Soil Moisture (Passive) Data Products, Revision B.](https://smap.jpl.nasa.gov/system/internal_resources/details/original/316_L2_SM_P_ATBD_v7_Sep2015.pdf)
