"""Planck radiation and basic calibration with explicit constants and units.

Planck functions return broadcast float64 arrays and raise on invalid computed
elements. Calibration results contain values, valid and flags of the same shape.
Invalid output is NaN. Flags are bitwise: 1 nonfinite, 2 nodata, 4 saturated,
8 externally masked, 16 outside operation domain. No implicit DN=0 mask or
clipping to [0,1]. Coefficients are supplied from the specific product metadata;
this module does not guess sensors, read imagery or correct the atmosphere.
"""
import numpy as np
from . import _native
from ._api import _call, _dumps, ModelError
from ._numeric import real_array
from ._numeric import broadcast

NONFINITE, NODATA, SATURATED, MASKED, DOMAIN = 1, 2, 4, 8, 16
PLANCK_CONSTANT=_native.PLANCK_CONSTANT       # J s, exact SI defining value
SPEED_OF_LIGHT=_native.SPEED_OF_LIGHT         # m s^-1, exact
BOLTZMANN_CONSTANT=_native.BOLTZMANN_CONSTANT # J K^-1, exact

def planck_radiance(wavelength_nm,temperature_k,*,radiance_unit='W m^-2 sr^-1 nm^-1'):
    """Vacuum B_lambda blackbody spectral radiance; broadcast inputs, float64 output.

    Wavelength is always nm; temperature is K (>=0). Output density may be
    'W m^-2 sr^-1 nm^-1', 'W m^-2 sr^-1 um^-1' or 'W m^-2 sr^-1 m^-1'. This
    is radiance per steradian, not hemispheric exitance (which adds pi). It
    includes both polarizations. T=0 gives zero. Extreme Wien tails may underflow
    to zero; true output overflow raises ModelError. No emissivity is assumed
    beyond an ideal blackbody, and no atmosphere or sensor response is included.
    """
    x,t=broadcast(real_array(wavelength_nm),real_array(temperature_k))
    return _call(_native.planck_array,'wavelength',x,t,radiance_unit)

def inverse_planck(wavelength_nm,radiance,*,radiance_unit='W m^-2 sr^-1 nm^-1'):
    """Monochromatic brightness temperature [K], inverse of planck_radiance.

    Radiance is nonnegative in the explicitly selected wavelength-density unit;
    zero gives the limiting temperature 0 K. Underflowed forward radiance cannot
    recover its original temperature. This is distinct from a sensor-band K1/K2
    calibration and from surface/emissivity temperature.
    """
    x,l=broadcast(real_array(wavelength_nm),real_array(radiance))
    return _call(_native.planck_array,'inverse_wavelength',x,l,radiance_unit)

def planck_frequency_radiance(frequency_hz,temperature_k):
    """Vacuum B_nu [W m^-2 sr^-1 Hz^-1], frequency in Hz, temperature in K.

    B_nu and B_lambda are different spectral densities and require the
    wavelength/frequency Jacobian; their numeric values cannot be compared directly.
    """
    nu,t=broadcast(real_array(frequency_hz),real_array(temperature_k))
    return _call(_native.planck_array,'frequency',nu,t,'W m^-2 sr^-1 Hz^-1')

def inverse_planck_frequency(frequency_hz,radiance_per_hz):
    """Inverse B_nu, returning monochromatic brightness temperature [K]."""
    nu,l=broadcast(real_array(frequency_hz),real_array(radiance_per_hz))
    return _call(_native.planck_array,'inverse_frequency',nu,l,'W m^-2 sr^-1 Hz^-1')

def _operation(mode, values, *, valid_mask=None, nodata=None, saturation=None, unit='1', **constants):
    data = real_array(values)
    if valid_mask is None:
        mask = np.ones(data.shape, dtype=bool)
    else:
        mask = np.asarray(valid_mask)
        if mask.dtype.kind != 'b' or mask.shape != data.shape:
            raise ModelError('invalid_parameter', 'valid_mask must be boolean and exactly match input shape')
    config = dict(gain=1., offset=0., solar_zenith_deg=0., solar_irradiance=1., distance_au=1., k1=1., k2=1., radiance_per_nm=False, nodata=nodata, saturation=saturation)
    config.update(constants)
    out = _call(_native.radiometry_array, mode, data, mask, _dumps(config))
    out['unit'] = unit
    return out

def linear_calibrate(dn, *, gain, offset, unit, valid_mask=None, nodata=None, saturation=None):
    """gain*DN + offset; gain/offset and declared unit come from product metadata.

    saturation masks DN >= threshold. It is not inferred from dtype or sensor.
    Pass band-specific coefficients one band at a time. Unit is a required label.
    """
    if not isinstance(unit, str) or not unit.strip():
        raise ModelError('invalid_parameter', 'output unit must be explicit')
    return _operation('linear', dn, gain=gain, offset=offset, unit=unit, valid_mask=valid_mask, nodata=nodata, saturation=saturation)

def dn_to_toa_reflectance(dn, *, reflectance_mult, reflectance_add, solar_zenith_deg, valid_mask=None, nodata=None, saturation=None):
    """(M_rho*DN+A_rho)/cos(theta), using Level-1 product reflectance coefficients.

    This path already includes irradiance/distance through the product factors.
    Do not apply Earth-Sun distance or an ESUN correction a second time. A
    Level-2 surface reflectance scale/offset instead uses linear_calibrate.
    """
    return _operation('sun', dn, gain=reflectance_mult, offset=reflectance_add, solar_zenith_deg=solar_zenith_deg, valid_mask=valid_mask, nodata=nodata, saturation=saturation)

def radiance_to_toa_reflectance(radiance, *, solar_irradiance, solar_zenith_deg, earth_sun_distance_au=1., valid_mask=None):
    """pi*L*d^2/(ESUN*cos(theta)); ESUN is irradiance at 1 AU.

    L [W m^-2 sr^-1 nm^-1] and ESUN [W m^-2 nm^-1], or BOTH per um. Values
    and ESUN must describe the same band/basis. Output is TOA reflectance,
    not atmospherically corrected surface reflectance. Solar zenith is degrees.
    """
    return _operation('toa', radiance, solar_irradiance=solar_irradiance, solar_zenith_deg=solar_zenith_deg, distance_au=earth_sun_distance_au, valid_mask=valid_mask)

def radiance_to_brightness_temperature(radiance, *, k1, k2, radiance_unit='W m^-2 sr^-1 um^-1', valid_mask=None):
    """K2/log(1+K1/L), using positive band-specific thermal metadata constants.

    K1 always uses W m^-2 sr^-1 um^-1; K2 uses K. Input radiance_unit may be
    per um or per nm (converted in Rust). Nonpositive L is flagged DOMAIN.
    This is at-sensor brightness temperature, not surface temperature/LST.
    """
    if radiance_unit not in ('W m^-2 sr^-1 um^-1', 'W m^-2 sr^-1 nm^-1'):
        raise ModelError('invalid_parameter', 'radiance_unit must explicitly be per um or per nm')
    return _operation('bt', radiance, k1=k1, k2=k2, radiance_per_nm=radiance_unit.endswith('nm^-1'), valid_mask=valid_mask, unit='K')
