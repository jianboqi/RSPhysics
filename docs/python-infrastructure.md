# Spectral, sensor, and calibration tools

## Spectral processing

Resample spectra, integrate wavelength intervals, or average through a bandpass:

```python
import numpy as np
import rsphysics as rsp

leaf = rsp.vegetation.prospect_d()['result']
x = leaf['wavelengths_nm']
reflectance = leaf['reflectance']
sampled = rsp.spectral.resample(x, reflectance, [450., 550., 670., 850.])
integral = rsp.spectral.integrate(x, reflectance, lower_nm=700, upper_nm=900)

red = rsp.sensors.Bandpass('red', [620., 650., 680.], [0., 1., 0.])
red_reflectance = red.convolve(x, reflectance)
```

Wavelengths use **nm** and must be positive, finite, and strictly increasing.
Spectra have shape `(..., wavelength)`. Resampling uses linear interpolation;
values outside the supplied wavelength domain are not extrapolated.
An interval integral has the input quantity's units multiplied by nm.

Bandpass averaging computes `integral(S R) / integral(R)`, where S is the spectrum
and R is the response. Optional `irradiance` and `irradiance_wavelengths_nm`
arguments to `rsp.spectral.convolve` instead use `integral(S R E) / integral(R E)`.
Use weighting appropriate to the physical quantity being averaged.

## Satellite bands

```python
canopy = rsp.vegetation.prosail_d()['result']
oli = rsp.sensors.load('landsat-8.oli')
bands = oli.convolve(canopy['wavelengths_nm'], canopy['brf'])
print(bands['bands'])
print(bands['values'])

red_nir = oli.select(['B4', 'B5'])
profiles = rsp.sensors.list_sensors()
```

| Sensors | Profiles |
|---|---|
| Landsat | 4/5 TM, 7 ETM+, 8/9 OLI and TIRS |
| MODIS | Terra/Aqua, 16 reflective response bands |
| Sentinel-2 | A/B/C MSI v5; requires local acquisition/import |

Landsat and MODIS profiles are bundled. Sentinel-2 responses can be acquired
explicitly with `rsp.sensors.fetch('sentinel-2a.msi')`, or imported from an
official workbook using `rsp.sensors.import_file`. Loading a profile does not
silently download data.

The spectrum must cover the nonzero response support. Explicit
`allow_partial=True` permits partial overlap and returns response-area coverage;
it does not estimate missing signal. A thermal TIRS response cannot be applied
to a visible/shortwave-infrared canopy spectrum.

Response information: [USGS](https://landsat.usgs.gov/spectral-characteristics-viewer),
[NASA OBPG](https://oceandata.sci.gsfc.nasa.gov/rsr_tables/), and
[ESA SentiWiki](https://sentiwiki.copernicus.eu/web/s2-documents).

## Radiometric calibration

```python
dn = np.array([0., 10000., 20000.])

# Illustrative coefficients; use the metadata of your actual product and band.
radiance = rsp.radiometry.linear_calibrate(
    dn, gain=0.0003342, offset=0.1,
    unit='W m^-2 sr^-1 um^-1', nodata=0)
brightness_temperature = rsp.radiometry.radiance_to_brightness_temperature(
    radiance['values'], k1=774.8853, k2=1321.0789,
    valid_mask=radiance['valid'])
```

| Function | Calculation |
|---|---|
| `linear_calibrate` | Gain × DN + offset |
| `dn_to_toa_reflectance` | Product reflectance scaling with solar-angle correction |
| `radiance_to_toa_reflectance` | Radiance, solar irradiance, Sun–Earth distance, and solar angle |
| `radiance_to_brightness_temperature` | Thermal-band K1/K2 brightness temperature |

Calibration returns `values`, `valid`, `flags`, and `unit`. Invalid samples are
NaN; supply `nodata`, `saturation`, or `valid_mask` explicitly as needed. Units
and coefficients must match the product. At-sensor brightness temperature is
not emissivity-corrected land surface temperature. Product conventions are
described in the [USGS Level-1 guide](https://www.usgs.gov/landsat-missions/using-usgs-landsat-level-1-data-product).

For parameter grids and array simulation, see [NumPy batches](usage.md#numpy-batches).
