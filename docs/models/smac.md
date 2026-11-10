# SMAC

SMAC provides band-wise analytical atmospheric correction and forward conversion between top-of-atmosphere and surface reflectance.

## How it works

Gas absorption, atmospheric scattering, transmission, and spherical albedo are parameterized for the selected sensor band and aerosol type. Surface-to-TOA conversion and its analytical inverse use the same physical quantities.

## Usage

```python
import rsphysics as rsp

corrected = rsp.atmosphere.smac(
    coefficient_id='sentinel-2a.msi.B04.continental',
    direction='toa_to_surface', reflectance=0.2,
    solar_zenith_deg=30., view_zenith_deg=5., relative_azimuth_deg=60.,
    pressure_hpa=1013.25, aot550=0.1,
    ozone_atm_cm=0.3, water_vapour_g_cm2=2.)
surface_reflectance = corrected['result']['reflectance']
sets = rsp.atmosphere.smac_coefficients()
```

Model ID: `atmosphere.smac`. Use the same ID with `rsp.run`, `rsp.run_array`, or
`rsp.parameter_grid`; see [Basic usage](../usage.md).

## Parameters

| Parameter | Default | Unit | Supported range |
|---|---|---|---|
| `aot550` | `0.1` | 1 | 0.0–1.0 |
| `coefficient_id` | `sentinel-2a.msi.B04.continental` | 1 | Available sensor-band IDs |
| `direction` | `toa_to_surface` | 1 | toa_to_surface, surface_to_toa |
| `ozone_atm_cm` | `0.3` | atm cm | 0.0–1.0 |
| `pressure_hpa` | `1013.25` | hPa | 300.0–1100.0 |
| `reflectance` | `0.2` | 1 | 0.0–2.0 |
| `relative_azimuth_deg` | `60` | deg | 0.0–180.0 |
| `solar_zenith_deg` | `30` | deg | 0.0–70.0 |
| `view_zenith_deg` | `5` | deg | 0.0–70.0 |
| `water_vapour_g_cm2` | `2` | g cm^-2 | 0.0–10.0 |

For NumPy batches, pass string choices with `fixed`:

```python
table = rsp.parameter_grid('atmosphere.smac', {'aot550': [0.05, 0.1, 0.2]})
batch = rsp.run_array('atmosphere.smac', table, fixed={
    'coefficient_id': 'sentinel-2a.msi.B04.continental',
    'direction': 'toa_to_surface',
})
```

## Outputs

| Field | Meaning |
|---|---|
| `reflectance` | Surface or TOA reflectance, according to direction |
| `gas_transmittance` | Combined downward/upward gas transmission |
| `downward_scattering_transmittance / upward_scattering_transmittance` | Scattering transmissions |
| `atmospheric_reflectance / spherical_albedo` | Atmospheric scattering and multiple-reflection quantities |
| `outside_unit_interval` | Diagnostic for output reflectance outside 0–1 |

## Scope

Inputs are reflectance, not DN or radiance. Supply band-matched coefficients and atmospheric parameters. The surface is Lambertian; values are not clipped. Use `surface_to_toa` for forward simulation. Common coefficient sets include Sentinel-2 MSI, Landsat, and legacy MODIS channels; inspect available IDs rather than substituting another sensor.

## Authors and references

H. Rahman and G. Dedieu.

- [Rahman & Dedieu (1994). SMAC: a simplified method for the atmospheric correction of satellite measurements in the solar spectrum. International Journal of Remote Sensing 15(1), 123–143.](https://doi.org/10.1080/01431169408954055)
