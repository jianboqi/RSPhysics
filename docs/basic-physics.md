# Fresnel reflection and Planck radiation

## Fresnel reflection

Calculate s/p power reflectances and complex amplitudes at a planar interface:

```python
import numpy as np
import rsphysics as rsp

glass = rsp.optics.fresnel([0., 30., 60.], 1.5, n_incident=1.)
print(glass['reflectance_s'], glass['reflectance_p'])

absorber = rsp.optics.fresnel(40., 2.5 + 0.3j)
epsilon = rsp.optics.fresnel_permittivity(40., 6.0 + 0.5j)
tir = rsp.optics.fresnel(60., 1., n_incident=1.5)
```

Angles use degrees from the interface normal, `0 <= angle < 90`. Refractive
indices follow `n + i*k` with time convention `exp(-i*omega*t)`. The incident
medium is lossless; the transmitted medium may absorb. Media are homogeneous,
isotropic, and nonmagnetic. Relative-permittivity input assumes `mu_r = 1`.

| Output | Meaning |
|---|---|
| `reflectance_s`, `reflectance_p` | Dimensionless power reflectances |
| `amplitude_r_s`, `amplitude_r_p` | Complex field reflection amplitudes |
| `reflectance_unpolarized` | Equal average of s/p power reflectances |
| `interface_transmittance_s`, `interface_transmittance_p` | Normal power flux entering the second medium |
| `total_internal_reflection` | Total-internal-reflection indicator |

Interface transmittance is the flux at the boundary, not transmission through a
finite slab. Microwave permittivity written using the opposite time convention
must be converted explicitly. Inputs broadcast according to NumPy rules.

## Planck radiation

Compute vacuum spectral radiance and its inverse:

```python
wavelength_nm = np.array([8000., 10000., 12000.])
unit = 'W m^-2 sr^-1 um^-1'
radiance = rsp.radiometry.planck_radiance(
    wavelength_nm, 300., radiance_unit=unit)
temperature_k = rsp.radiometry.inverse_planck(
    wavelength_nm, radiance, radiance_unit=unit)

frequency_hz = np.array([2.e13, 3.e13])
frequency_radiance = rsp.radiometry.planck_frequency_radiance(frequency_hz, 300.)
```

Wavelength input always uses **nm**, temperature uses **K**, and frequency uses
**Hz**. `radiance_unit` selects wavelength density per m, nm, or µm; frequency
radiance uses `W m^-2 sr^-1 Hz^-1`. Spectral radiance is per steradian; ideal
Lambertian blackbody exitance is π times radiance.

`inverse_planck` and `inverse_planck_frequency` return monochromatic brightness
temperature. They do not correct emissivity or integrate a sensor band. Use
product-specific K1/K2 conversion for calibrated thermal-band radiance.

Temperatures/radiances must be nonnegative and finite. Exact zero maps to its
zero-temperature/radiance limit. Extremely small radiance can underflow to zero
and cannot then recover the original positive temperature.

Physical conventions: [NIST SI constants](https://www.nist.gov/pml/special-publication-330/sp-330-section-2)
and [CIE Planck's law](https://cie.co.at/eilvterm/17-24-005).
