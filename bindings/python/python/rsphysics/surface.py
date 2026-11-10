"""Surface directional models; all scientific calculations run in Rust."""
import numpy as np
from ._api import run
from ._arrays import run_array

ROSS_LI_PARAMETERS=('f_iso','f_vol','f_geo','solar_zenith_deg','view_zenith_deg','relative_azimuth_deg')
def ross_li(*,f_iso:float=.2,f_vol:float=.1,f_geo:float=.02,
            solar_zenith_deg:float=30.,view_zenith_deg:float=20.,relative_azimuth_deg:float=0.)->dict:
    """RossThick-LiSparseReciprocal, fixed h/b=2,b/r=1, one user-selected band.

    Angles are degrees; relative azimuth zero is same azimuth/backscatter.
    Result includes scalar BRF, BRDF [sr^-1], signed principal-plane arrays
    (-85..85 step1), and a flattened hemisphere (zenith0..85 step5, azimuth
    0..350 step10, zenith-major). No wavelength grid or spectral interpolation.
    Negative algebraic BRF is retained with negative_sample_count; it indicates
    nonphysical weights at those geometries. BRF above one is not clipped.
    """
    return run('surface.ross-li',locals())

def ross_li_batch(parameters:np.ndarray, *, shared_axes=False)->dict:
    """Rust batch from real (batch,6) matrix, columns ROSS_LI_PARAMETERS.

    Scalar result fields have shape (batch,), principal fields (batch,171),
    hemisphere fields (batch,648). Native NumPy transport; no Python scientific loop.
    """
    return run_array('surface.ross-li',parameters,shared_axes=shared_axes)



def rpv(*, rho0=0.2, minnaert_k=0.7, phase_asymmetry=-0.2,
        hotspot_mode="original_three_parameter", hotspot_rho=0.2,
        solar_zenith_deg=30.0, view_zenith_deg=20.0, relative_azimuth_deg=0.0):
    """RPV three- or four-parameter BRDF. Relative azimuth0 is backward.

    The original three-parameter mode uses rho_c=rho0. The independent
    four-parameter mode exposes hotspot_rho. BRF is not hemispherical albedo.
    """
    return run("surface.rpv",locals())


def rpv_batch(parameters, *, hotspot_mode="original_three_parameter", shared_axes=False):
    """RPV NumPy batch in parameters_for('surface.rpv') order."""
    return run_array("surface.rpv",parameters,shared_axes=shared_axes,fixed={"hotspot_mode":hotspot_mode})
