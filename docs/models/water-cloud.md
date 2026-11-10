# Water-cloud model

The water-cloud model represents radar backscatter as vegetation scattering plus a soil response attenuated by two canopy passages.

## How it works

The V1=V2=VWC parameterization uses `t² = exp(-2*B*VWC/cos(theta))` and `sigma0 = A*VWC*cos(theta)*(1-t²) + sigma0_soil*t²`.

## Usage

```python
import rsphysics as rsp

radar = rsp.microwave.water_cloud(
    vegetation_water_content=1., scattering_coefficient=0.1,
    attenuation_coefficient=0.2, soil_backscatter=0.1,
    observation_zenith_deg=40.)
sigma0 = radar['result']['sigma0']
```

Model ID: `microwave.water-cloud`. Use the same ID with `rsp.run`, `rsp.run_array`, or
`rsp.parameter_grid`; see [Basic usage](../usage.md).

## Parameters

| Parameter | Default | Unit | Supported range |
|---|---|---|---|
| `attenuation_coefficient` | `0.2` | m^2 kg^-1 | 0.0–2.0 |
| `observation_zenith_deg` | `40` | deg | 0.0–80.0 |
| `scattering_coefficient` | `0.1` | m^2 kg^-1 | 0.0–2.0 |
| `soil_backscatter` | `0.1` | 1 | 0.0–10.0 |
| `vegetation_water_content` | `1` | kg m^-2 | 0.0–10.0 |

## Outputs

| Field | Meaning |
|---|---|
| `sigma0` | Total normalized radar cross section, linear power |
| `vegetation_backscatter` | Vegetation contribution, linear power |
| `attenuated_soil_backscatter` | Attenuated soil contribution, linear power |
| `two_way_transmissivity` | Two-way canopy power transmission |

## Scope

A and B must be calibrated for the channel, vegetation descriptor, and observing conditions. Soil backscatter is supplied in linear power, not dB. The model does not directly retrieve soil moisture; angle sweeps hold the coefficients and soil response fixed.

## Authors and references

E. P. W. Attema and Fawwaz T. Ulaby.

- [Attema & Ulaby (1978). Vegetation modeled as a water cloud. Radio Science 13(2), 357–364.](https://doi.org/10.1029/RS013i002p00357)


## Independent vegetation descriptors

The default `descriptor_mode="water_content"` retains V1=V2=VWC. For the
general form, select `descriptor_mode="independent"` and provide
`scattering_descriptor` (V1) and `attenuation_descriptor` (V2). A and B must be
calibrated in units matching those descriptors; they cannot be transferred
unchanged between VWC, LAI or other descriptors.

```python
canopy = rsp.microwave.water_cloud(descriptor_mode="independent",
    scattering_descriptor=3., attenuation_descriptor=0.5)
```

The NumPy table has seven columns: the original five followed by V1 and V2.
The selector is fixed per batch. This interface still takes the bare-soil
backscatter directly and performs no soil-moisture inversion.

- [Park et al. (2019). Theoretical Evaluation of Water Cloud Model Vegetation Parameters.](https://doi.org/10.3390/rs11080894)
