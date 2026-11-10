"""Independent SMAC band-wise atmospheric conversion, entirely computed in Rust.

Atmospheric correction needs external AOT550, water vapour, ozone and LOCAL
surface pressure. No automatic weather download, sensor substitution or coupling.
"""
from functools import lru_cache
import json
from . import _native
from ._api import _call, run, ModelError
from ._arrays import run_array

SMAC_PARAMETERS=('reflectance','solar_zenith_deg','view_zenith_deg','relative_azimuth_deg','pressure_hpa','aot550','ozone_atm_cm','water_vapour_g_cm2')
DEFAULT_COEFFICIENT='sentinel-2a.msi.B04.continental'

@lru_cache(maxsize=1)
def _library():
    return json.loads(_call(_native.smac_coefficients_json))

def smac_coefficients(*, satellite=None, aerosol=None):
    """List exact supported coefficient IDs, bands and original-file provenance.

    MODIS legacy data do not distinguish Terra/Aqua. Landsat-9 coefficients are
    not silently substituted with Landsat-8. Numerical library is offline.
    """
    library=_library()
    return [{**record, **{key:value for key,value in library['sources'][record['source_key']].items() if key!='text'}}
            for record in library['records']
            if (satellite is None or record['satellite']==satellite) and (aerosol is None or record['aerosol']==aerosol)]

def smac_coefficient_source(coefficient_id):
    """Export exact original ASCII text, SHA-256 and pinned URL for one set.

    A Sentinel-2 source file contains all 13 bands; 'column' is zero-based.
    Returned dictionaries are copies; callers cannot mutate the loaded library.
    """
    library=_library()
    for record in library['records']:
        if record['id']==coefficient_id:
            return {**record,**library['sources'][record['source_key']]}
    raise ModelError('invalid_parameter',f'unknown SMAC coefficient_id: {coefficient_id}')

def smac(*, coefficient_id=DEFAULT_COEFFICIENT, direction='toa_to_surface', reflectance=0.2,
         solar_zenith_deg=30.,view_zenith_deg=5.,relative_azimuth_deg=60.,pressure_hpa=1013.25,
         aot550=0.1,ozone_atm_cm=0.3,water_vapour_g_cm2=2.):
    """TOA->surface or surface->TOA; reflectances are dimensionless, never DN.

    Angles in degrees, pressure in hPa, ozone in atm-cm (300 DU = 0.3), water
    vapour in g/cm2 (1 g/cm2 = 10 kg/m2). Output is never clipped. AOT curve
    holds the selected input reflectance and all other atmospheric inputs fixed.
    """
    return run('atmosphere.smac',locals())

def smac_batch(parameters, *, coefficient_id=DEFAULT_COEFFICIENT,direction='toa_to_surface',shared_axes=False):
    """Native NumPy (batch,8) in SMAC_PARAMETERS order; selectors fixed per batch.

    Use run_batch for heterogeneous coefficient IDs/directions in one call.
    Scientific arrays never pass through JSON; string selectors and manifests do.
    """
    return run_array('atmosphere.smac',parameters,shared_axes=shared_axes,
                     fixed={'coefficient_id':coefficient_id,'direction':direction})
