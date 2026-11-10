# PROSAIL-D

PROSAIL-D combines leaf optics with canopy radiative transfer to predict directional canopy reflectance and directional-hemispherical reflectance.

## How it works

PROSPECT-D supplies leaf optical properties. The 4SAIL canopy formulation represents direct illumination, leaf-angle distribution, hotspot geometry, and a soil background. Both Campbell ellipsoidal and Verhoef bimodal leaf-angle distributions use the original 13 inclination bins. Direct and diffuse illumination responses are computed together.

## Usage

```python
import rsphysics as rsp

canopy = rsp.vegetation.prosail_d(
    leaf_area_index=3., chlorophyll=45., solar_zenith_deg=30.,
    view_zenith_deg=10., relative_azimuth_deg=60.)
wavelengths = canopy['result']['wavelengths_nm']
brf = canopy['result']['brf']
```

Model ID: `vegetation.prosail-d`. Use the same ID with `rsp.run`, `rsp.run_array`, or
`rsp.parameter_grid`; see [Basic usage](../usage.md).

## Parameters

| Parameter | Default | Unit | Supported range |
|---|---|---|---|
| `anthocyanins` | `0` | ug cm^-2 | 0.0–100.0 |
| `brown_pigments` | `0` | 1 | 0.0–5.0 |
| `carotenoids` | `8` | ug cm^-2 | 0.0–100.0 |
| `chlorophyll` | `40` | ug cm^-2 | 0.0–200.0 |
| `dry_matter` | `0.008` | g cm^-2 | 0.0–0.05 |
| `hotspot` | `0.1` | 1 | 0.0–1.0 |
| `incidence_angle_deg` | `40` | deg | 1.0–90.0 |
| `leaf_area_index` | `3` | 1 | 0.0–10.0 |
| `mean_leaf_angle_deg` | `60` | deg | 5.0–85.0 |
| `n` | `1.5` | 1 | 1.0–5.0 |
| `relative_azimuth_deg` | `60` | deg | 0.0–180.0 |
| `soil_brightness` | `1` | 1 | 0.0–1.0 |
| `soil_dry_fraction` | `0.5` | 1 | 0.0–1.0 |
| `solar_zenith_deg` | `30` | deg | 0.0–85.0 |
| `view_zenith_deg` | `0` | deg | 0.0–85.0 |
| `water` | `0.01` | g cm^-2 | 0.0–0.1 |

## Outputs

| Field | Meaning |
|---|---|
| `wavelengths_nm` | Wavelengths, nm; 400–2500 nm |
| `hemispherical_directional_reflectance` | Diffuse-illumination response in the viewing direction (rdot) |
| `bihemispherical_reflectance` | Diffuse-illumination hemispherical reflectance (rddt) |
| `vegetation_cover` | One minus view-direction beam transmittance |
| `direct_absorptance / diffuse_absorptance` | Canopy absorptance with soil feedback |
| `direct_albedo / diffuse_albedo` | Direct/diffuse albedo (rsdstar/rddstar) |
| `brf` | Dimensionless bidirectional reflectance factor |
| `hemispherical_reflectance` | Directional-hemispherical reflectance under direct illumination |

## Scope

A homogeneous canopy is assumed. BRF can exceed one and must not be added to transmittance as an energy fraction. Soil dry fraction mixes optical endmembers; it is not volumetric soil moisture. LAI=0 gives bare soil.

## Authors and references

PROSPECT-D: Féret, Gitelson, Noble, and Jacquemoud. SAIL/4SAIL: Wout Verhoef and collaborators. The integrated PROSPECT–SAIL framework is reviewed by Jacquemoud et al.

- [Jacquemoud et al. (2009). PROSPECT + SAIL models: A review of use for vegetation characterization. Remote Sensing of Environment.](https://doi.org/10.1016/j.rse.2008.01.026)
- [Verhoef et al. (2007). Unified Optical-Thermal Four-Stream Radiative Transfer Theory for Homogeneous Vegetation Canopies. IEEE TGRS.](https://doi.org/10.1109/TGRS.2007.895844)
- [Féret et al. (2017). PROSPECT-D.](https://doi.org/10.1016/j.rse.2017.03.004)


Select the Verhoef distribution with `leaf_angle_distribution="verhoef"`,
`lidf_a` and `lidf_b`, with `|a|+|b| <= 1`. Campbell uses `mean_leaf_angle_deg`.
The NumPy table has 18 columns; the final two are `lidf_a` and `lidf_b`.
The distribution selector is fixed per numeric batch. The exact conservative
transfer-matrix limit handles nonabsorbing leaves without a 0/0 singularity.

```python
diffuse = rsp.vegetation.prosail_d(
    leaf_angle_distribution="verhoef", lidf_a=-0.35, lidf_b=-0.15)
rddt = diffuse['result']['bihemispherical_reflectance']
absorbed = diffuse['result']['diffuse_absorptance']
```
