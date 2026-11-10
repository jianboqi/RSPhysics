# GORT-GPP

GORT-GPP uses crown geometry and radiation interception to describe sunlit/shaded leaf photosynthesis and canopy carbon assimilation.

## How it works

Between- and within-crown gap probabilities determine light interception. Direct/diffuse PAR is partitioned between leaf classes; leaf-age-dependent Vcmax and Farquhar–Ball–Berry photosynthesis determine canopy net assimilation.

## Usage

```python
import rsphysics as rsp

canopy = rsp.ecosystem.gort_gpp(
    leaf_area_index=4.31, solar_zenith_deg=40.,
    direct_par_umol_m2_s=1200., diffuse_par_umol_m2_s=200.,
    young_leaf_fraction=0.4, air_temperature_c=25., co2_ppm=400.)
net_assimilation = canopy['result']['net_assimilation_umol_m2_s']
```

Model ID: `ecosystem.gort-gpp`. Use the same ID with `rsp.run`, `rsp.run_array`, or
`rsp.parameter_grid`; see [Basic usage](../usage.md).

## Parameters

| Parameter | Default | Unit | Supported range |
|---|---|---|---|
| `air_temperature_c` | `25` | degC | 5.0–40.0 |
| `background_reflectance` | `0.1` | 1 | 0.0–0.9 |
| `co2_ppm` | `400` | umol mol^-1 | 200.0–1200.0 |
| `crown_axis_ratio` | `1.17` | 1 | 0.1–5.0 |
| `crown_base_height_m` | `7.8` | m | 0.0–60.0 |
| `crown_density_m2` | `0.1905` | m^-2 | 0.001–2.0 |
| `crown_radius_m` | `1.7` | m | 0.1–10.0 |
| `crown_top_height_m` | `14.1` | m | 0.1–80.0 |
| `diffuse_par_umol_m2_s` | `200` | umol m^-2 s^-1 | 0.0–2500.0 |
| `direct_par_umol_m2_s` | `1200` | umol m^-2 s^-1 | 0.0–2500.0 |
| `electron_photon_ratio` | `2.6` | 1 | 1.0–5.0 |
| `leaf_angle_parameter` | `0.5` | 1 | 0.1–5.0 |
| `leaf_area_index` | `4.31` | m^2 m^-2 | 0.5–10.0 |
| `leaf_single_scattering_albedo` | `0.1` | 1 | 0.0–0.9 |
| `minimum_stomatal_conductance` | `0.01` | mol m^-2 s^-1 | 0.001–0.1 |
| `pressure_pa` | `101325` | Pa | 50000.0–110000.0 |
| `solar_zenith_deg` | `40` | deg | 1.0–80.0 |
| `stomatal_slope` | `8` | 1 | 1.0–20.0 |
| `vapour_pressure_deficit_pa` | `1000` | Pa | 0.0–8000.0 |
| `vcmax25_mature` | `34.17` | umol m^-2 s^-1 | 1.0–200.0 |
| `vcmax25_young` | `52.1` | umol m^-2 s^-1 | 1.0–200.0 |
| `young_leaf_fraction` | `0.4` | 1 | 0.0–1.0 |

## Outputs

| Field | Meaning |
|---|---|
| `net_assimilation_umol_m2_s` | Canopy net assimilation per ground area, µmol CO₂ m⁻² s⁻¹ |
| `sunlit_assimilation_umol_m2_s / shaded_assimilation_umol_m2_s` | Net assimilation per leaf-class area |
| `sunlit_leaf_area_index / shaded_leaf_area_index` | Sunlit/shaded LAI; their sum is total LAI |
| `sunlit_absorbed_par / shaded_absorbed_par` | Absorbed PAR per leaf-class area |
| `between_crown_gaps / within_crown_gaps` | 0–89° crown-gap probability curves |

## Scope

The output is net assimilation, including respiration; it is not directly GPP, GEP, or NEE. Inputs represent one time step. Supply direct/diffuse PAR and solar zenith explicitly. Young-leaf fraction is the canopy-top weight for Vcmax. Annual station processing, evapotranspiration, and seasonal Vcmax tables are outside this interface.

## Authors and references

Qiaoli Wu, Conghe Song, Jinling Song, Jindi Wang, Shaoyuan Chen, Lei Yang, Wenhua Xiang, Zhonghui Zhao, and Jie Jiang.

- [Wu et al. (2021). Effects of leaf age and canopy structure on gross ecosystem production in a subtropical evergreen Chinese fir forest. Agricultural and Forest Meteorology 310, 108618.](https://doi.org/10.1016/j.agrformet.2021.108618)
