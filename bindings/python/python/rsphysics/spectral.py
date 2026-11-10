"""Piecewise-linear spectral operators, executed in Rust. All wavelengths are nm.

Values have shape (..., wavelength). No sorting, extrapolation, gap filling or
implicit wavelength-unit conversion. Band integration uses the union of spectral,
response and optional irradiance knots and exactly integrates their linear
interpolants' products, up to floating-point rounding.
"""
import numpy as np
from . import _native
from ._api import _call, ModelError
from ._numeric import real_array, axis

def resample(wavelengths_nm, values, target_wavelengths_nm):
    """Linear resampling on positive, strictly increasing axes; full coverage required."""
    return _call(_native.spectral_resample, axis(wavelengths_nm), real_array(values), axis(target_wavelengths_nm, 'target_wavelengths_nm'))

def integrate(wavelengths_nm, values, *, lower_nm=None, upper_nm=None):
    """Integral over a covered interval; result units are input-value units times nm."""
    x = axis(wavelengths_nm)
    return _call(_native.spectral_integrate, x, real_array(values), float(x[0] if lower_nm is None else lower_nm), float(x[-1] if upper_nm is None else upper_nm))

def _illumination(x, irradiance, irradiance_wavelengths_nm):
    if irradiance is None:
        if irradiance_wavelengths_nm is not None:
            raise ModelError('invalid_parameter', 'irradiance wavelengths require irradiance values')
        return np.empty(0), np.empty(0)
    return (x if irradiance_wavelengths_nm is None else axis(irradiance_wavelengths_nm, 'irradiance_wavelengths_nm'), real_array(irradiance, ndim=1, name='irradiance'))

def convolve(wavelengths_nm, values, response_wavelengths_nm, response, *, irradiance=None, irradiance_wavelengths_nm=None, allow_partial=False):
    """Normalized response-weighted mean, optionally weighted by incident irradiance.

    Default: integral(values*R) / integral(R). With irradiance E:
    integral(values*R*E) / integral(R*E). A response-only mean of reflectance
    is a spectral summary, not automatically a physically illuminated band
    reflectance. E must be compatible with the input quantity and geometry.

    Returns values (...), unweighted response-area coverage, full-response
    centroid and peak-normalized equivalent width [nm]. Partial support is
    rejected by default; allow_partial=True renormalizes on the overlap and
    reports coverage. Tabulated response is taken as zero outside its bounds;
    no unmeasured response tail is invented. Negative response is rejected.
    """
    if not isinstance(allow_partial, (bool, np.bool_)):
        raise ModelError('invalid_parameter', 'allow_partial must be boolean')
    x = axis(wavelengths_nm)
    ix, iy = _illumination(x, irradiance, irradiance_wavelengths_nm)
    out = _call(_native.spectral_convolve, x, real_array(values), axis(response_wavelengths_nm, 'response_wavelengths_nm'), real_array(response, ndim=1, name='response'), ix, iy, bool(allow_partial))
    out['weighting'] = 'response' if irradiance is None else 'response_times_irradiance'
    return out
