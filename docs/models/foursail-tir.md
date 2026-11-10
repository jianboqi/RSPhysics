# 4SAIL-TIR

4SAIL-TIR predicts top-of-canopy thermal infrared radiance from four prescribed
surface temperatures: sunlit leaves, shaded leaves, sunlit soil and shaded soil.
It accounts for canopy scattering, soil feedback, directional shadow overlap
and reflected downwelling sky radiation.

## Usage

```python
import rsphysics as rsp

simulation = rsp.thermal.foursail(
    wavelength_nm=9500.,
    leaf_area_index=3.,
    leaf_emissivity=0.98,
    soil_emissivity=0.95,
    sunlit_leaf_temperature_k=310.,
    shaded_leaf_temperature_k=300.,
    sunlit_soil_temperature_k=320.,
    shaded_soil_temperature_k=305.,
    sky_brightness_temperature_k=270.,
    solar_zenith_deg=30.,
    view_zenith_deg=0.,
    relative_azimuth_deg=60.,
)
result = simulation["result"]
brightness_k = result["brightness_temperature_k"]
radiance = result["radiance_w_m2_sr_um"]
```

Temperatures are in kelvin and angles in degrees. The selected wavelength is
in nanometres, with a supported numerical range of 3000–50000 nm. This range
does not imply that every leaf or soil remains opaque or Lambertian throughout
it; emissivities and physical assumptions must match the material and wavelength.

## How it works

The four-stream equations propagate diffuse upward/downward fluxes, a direct
beam and the viewing radiance through a homogeneous turbid canopy. Leaf
orientation follows either the Campbell distribution (`mean_leaf_angle_deg`)
or the Verhoef distribution (`leaf_angle_distribution="verhoef"`, `lidf_a`,
`lidf_b`; `abs(a)+abs(b) <= 1`). `hotspot` controls joint sun-view shadowing.

The thermal source terms distinguish sunlit and shaded leaves. Adding the
soil boundary gives the monochromatic radiance

`L = sum(w_i * B(T_i)) + r_sky * B(T_sky) + BRF * E_direct / pi`.

The four `w_i` are effective emission weights including scattering feedback,
not geometric area fractions. Their sum is the directional emissivity, and
`sum(w_i) + r_sky = 1`. Brightness temperature is the inverse Planck function
of the complete radiance; it is not an arithmetic average of temperatures.

## Inputs and outputs

Sky brightness temperature specifies isotropic incident radiance
`B_lambda(T_sky)`. It is not air temperature; 0 K supplies no sky radiation.
Optional `direct_irradiance_w_m2_um` specifies direct spectral irradiance on
a **horizontal surface**, excluding diffuse sky irradiance. It defaults to zero.
Solar geometry still determines the sunlit populations when temperatures are
prescribed independently of this optional radiative input.

| Output | Meaning and units |
|---|---|
| `radiance_w_m2_sr_um` | Total top-of-canopy radiance, W m^-2 sr^-1 um^-1 |
| `brightness_temperature_k` | Monochromatic brightness temperature, K |
| `effective_emissivity` | Directional ensemble emissivity |
| `sunlit_leaf_weight`, `shaded_leaf_weight`, `sunlit_soil_weight`, `shaded_soil_weight` | Four effective thermal emission weights |
| `*_radiance_w_m2_sr_um` | Four surface contributions, reflected sky and reflected direct radiation |
| `hemispherical_emissivity` | Ensemble hemispherical emissivity |
| `upwelling_flux_w_m2_um` | Top-of-canopy upward hemispherical spectral flux, W m^-2 um^-1 |
| `ground_downwelling_irradiance_w_m2_um`, `ground_upwelling_flux_w_m2_um` | Irradiance and upward flux at the soil boundary |
| `principal_*` | 171 principal-plane samples from -85 to +85 degrees |
| `hemisphere_*` | 648 directions: zenith 0–85 degrees, azimuth 0–350 degrees |
| `spectral_*` | 61 diagnostic wavelengths from 8 to 14 um |

Positive principal-plane zenith angles share the solar azimuth; negative
angles use the opposite azimuth. Hemisphere azimuths are relative to the sun.
Exact alignment is also evaluated in `hotspot_brightness_temperature_k`.

The diagnostic spectrum holds emissivities, sky brightness temperature and
direct spectral irradiance fixed. To supply wavelength-dependent emissivities,
use separate numeric batch rows:

```python
import numpy as np
import rsphysics as rsp

parameters = rsp.parameter_grid("thermal.foursail", {
    "wavelength_nm": [9000., 10000., 12000.],
})
columns = rsp.thermal.FOURSAIL_PARAMETERS
parameters[:, columns.index("leaf_emissivity")] = [0.97, 0.98, 0.99]
batch = rsp.thermal.foursail_batch(parameters, shared_axes=True)
selected_radiance = batch["result"]["radiance_w_m2_sr_um"]
assert selected_radiance.shape == (3,)
```

The native NumPy matrix has 17 columns in `FOURSAIL_PARAMETERS` order. String
leaf-angle distribution selection is fixed per batch. Coordinate arrays may
be shared; other output arrays retain the batch dimension.

## Scope

- Leaf transmittance is zero; both leaf sides share the same emissivity.
  Leaf and soil scattering are Lambertian.
- Temperatures are prescribed. The model does not solve an energy balance or
  retrieve kinetic surface temperature from measured radiance.
- Outputs are at the canopy top. Atmospheric path absorption and path radiance
  between the canopy and a sensor are not included.
- The homogeneous canopy approximation does not explicitly resolve discrete
  crowns, rows or terrain.

## Authors and reference

Wout Verhoef, Li Jia, Qing Xiao and Zhongbo Su.

- [Unified optical–thermal four-stream radiative transfer theory for homogeneous vegetation canopies (2007), IEEE TGRS 45(6), 1808–1822.](https://research.utwente.nl/en/publications/unified-optical-thermal-four-stream-radiative-transfer-theory-for-2/)
- [Public technical report NLR-TP-2007-242.](https://reports.nlr.nl/server/api/core/bitstreams/ecd3960d-4c3c-4a76-9bd6-186044eb0862/content)
