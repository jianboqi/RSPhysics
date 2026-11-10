# L-MEB

L-MEB models L-band soil–vegetation microwave emission. This interface provides 1.41 GHz soil-forward calculations at vertical and horizontal polarization.

## How it works

Soil moisture, texture, and temperature determine dielectric properties and effective soil temperature. Fresnel reflectivity, empirical roughness, directional vegetation opacity, and downwelling sky radiation determine V/H brightness temperatures.

## Usage

```python
import rsphysics as rsp

emission = rsp.microwave.lmeb(
    soil_moisture=0.2, nadir_optical_depth=0.3,
    surface_temperature_k=300., canopy_temperature_k=295.,
    observation_zenith_deg=40.)
tb_v = emission['result']['brightness_temperature_v_k']
tb_h = emission['result']['brightness_temperature_h_k']
```

Model ID: `microwave.lmeb`. Use the same ID with `rsp.run`, `rsp.run_array`, or
`rsp.parameter_grid`; see [Basic usage](../usage.md).

## Parameters

| Parameter | Default | Unit | Supported range |
|---|---|---|---|
| `bulk_density_g_cm3` | `1.28` | g cm^-3 | 0.5–2.0 |
| `canopy_structure_h` | `2` | 1 | 0.0–5.0 |
| `canopy_structure_v_ratio` | `1` | 1 | 0.0–3.0 |
| `canopy_temperature_k` | `300` | K | 250.0–330.0 |
| `clay_fraction` | `0.328` | 1 | 0.0–1.0 |
| `deep_temperature_k` | `283` | K | 250.0–330.0 |
| `nadir_optical_depth` | `0` | 1 | 0.0–3.0 |
| `observation_zenith_deg` | `40` | deg | 0.0–80.0 |
| `roughness_exponent_h` | `0` | 1 | 0.0–4.0 |
| `roughness_exponent_v` | `0` | 1 | 0.0–4.0 |
| `roughness_height` | `0.1` | 1 | 0.0–2.0 |
| `roughness_mixing` | `0` | 1 | 0.0–0.5 |
| `sand_fraction` | `0.132` | 1 | 0.0–1.0 |
| `scattering_albedo_h` | `0.05` | 1 | 0.0–1.0 |
| `scattering_albedo_v_difference` | `0` | 1 | -1.0–1.0 |
| `sky_air_temperature_k` | `300` | K | 250.0–330.0 |
| `soil_moisture` | `0.07` | m^3 m^-3 | 0.005–0.6 |
| `surface_temperature_k` | `300` | K | 250.0–330.0 |
| `temperature_moisture_exponent` | `0.3` | 1 | 0.0–1.0 |
| `temperature_moisture_scale` | `0.3` | m^3 m^-3 | 0.05–0.6 |

## Outputs

| Field | Meaning |
|---|---|
| `brightness_temperature_v_k / brightness_temperature_h_k` | Surface V/H brightness temperature, K |
| `effective_soil_temperature_k` | Moisture-weighted effective soil temperature, K |
| `soil_reflectivity_v / soil_reflectivity_h` | Roughness-adjusted power reflectivities |
| `sweep_brightness_temperature_v_k / sweep_brightness_temperature_h_k` | 0–80° polarization response curves |

## Scope

The supported frequency is 1.41 GHz. Soil-forward mode includes frozen-soil and dry-sand parameterizations. Water surfaces, higher-frequency formulations, and inversion are outside this interface. Soil moisture is a volume fraction; texture fractions sum to at most one. Sky-model air temperature is supplied separately.

## Authors and references

Jean-Pierre Wigneron and collaborators.

- [Wigneron et al. (2007). L-band Microwave Emission of the Biosphere (L-MEB) Model: Description and calibration against experimental data sets over crop fields. Remote Sensing of Environment 107, 639–655.](https://doi.org/10.1016/j.rse.2006.10.014)
