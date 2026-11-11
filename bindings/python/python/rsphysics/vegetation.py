"""Leaf optics. Parameters are model-owned; units follow the author PROSPECT-D implementation."""
from collections.abc import Mapping
import json
import numpy as np
from . import _native
from ._api import _call, _dumps, run
from ._arrays import run_array

PROSPECT_D_PARAMETERS = (
    "n", "chlorophyll", "carotenoids", "anthocyanins", "brown_pigments",
    "water", "dry_matter", "incidence_angle_deg",
)
PROSAIL_D_PARAMETERS = PROSPECT_D_PARAMETERS + (
    "leaf_area_index", "mean_leaf_angle_deg", "hotspot", "solar_zenith_deg",
    "view_zenith_deg", "relative_azimuth_deg", "soil_brightness", "soil_dry_fraction", "lidf_a", "lidf_b",
)
INFORM_PARAMETERS = (
    "crown_lai", "understory_lai", "stem_density_per_ha", "tree_height_m",
    "crown_diameter_m", "crown_mean_leaf_angle_deg", "understory_mean_leaf_angle_deg",
    "crown_hotspot", "understory_hotspot", "solar_zenith_deg", "view_zenith_deg",
    "relative_azimuth_deg", "diffuse_fraction", "soil_reflectance", "soil_brightness",
)


def _inform_inputs(value):
    if isinstance(value, Mapping):
        return {key: _inform_inputs(item) for key, item in value.items()}
    if isinstance(value, np.ndarray):
        return value.tolist()
    return value


def inform(**parameters) -> dict:
    """Simulate forest reflectance, canopy transmission and crown/gap fractions.

    Crown and understory leaf parameters are separate mappings. Optional
    ``spectra`` defines a shared wavelength grid and accepts NumPy arrays for
    soil reflectance, layer leaf optics and PROSPECT 3.01 coefficients.
    """
    return run("vegetation.inform", _inform_inputs(parameters))


def inform_batch(parameters, *, shared_axes=False, crown_leaf=None, understory_leaf=None, spectra=None) -> dict:
    """Run a NumPy matrix in INFORM_PARAMETERS order with fixed leaf and spectral inputs."""
    fixed = {
        key: _inform_inputs(value)
        for key, value in (("crown_leaf", crown_leaf), ("understory_leaf", understory_leaf), ("spectra", spectra))
        if value is not None
    }
    return run_array("vegetation.inform", parameters, shared_axes=shared_axes, fixed=fixed)


def prosail_d(*, n: float = 1.5, chlorophyll: float = 40.0, carotenoids: float = 8.0,
             anthocyanins: float = 0.0, brown_pigments: float = 0.0,
             water: float = 0.01, dry_matter: float = 0.008, incidence_angle_deg: float = 40.0,
             leaf_area_index: float = 3.0, mean_leaf_angle_deg: float = 60.0,
             hotspot: float = 0.1, solar_zenith_deg: float = 30.0, view_zenith_deg: float = 0.0,
             relative_azimuth_deg: float = 60.0, soil_brightness: float = 1.0,
             soil_dry_fraction: float = 0.5, lidf_a:float=-.35,lidf_b:float=-.15,
             leaf_angle_distribution:str="campbell") -> dict:
    """PROSPECT-D + 4SAIL with the author's 13-bin Campbell distribution.

    Direct illumination; angles in degrees. Returns float64 ``brf`` (may exceed
    one) and ``hemispherical_reflectance`` (original rsdt), never transmittance.
    Soil dry fraction blends the original dry/wet endmembers; it is not moisture.
    LAI=0 uses the analytical bare-soil limit. Equations run exclusively in Rust.
    """
    return run("vegetation.prosail-d", locals())


def prosail_d_batch(parameters: np.ndarray, *, shared_axes=False,leaf_angle_distribution="campbell") -> dict:
    """Ordered Rust batch, real numeric matrix (batch, 18), PROSAIL_D_PARAMETERS.

    Scientific arrays use native NumPy buffers. Spectra are float64 with
    shape (batch, 2101); the wavelength axis is shared. No Python scientific loop.
    """
    return run_array('vegetation.prosail-d',parameters,shared_axes=shared_axes,fixed={'leaf_angle_distribution':leaf_angle_distribution})


def prospect_d(*, n: float = 1.5, chlorophyll: float = 40.0, carotenoids: float = 8.0,
               anthocyanins: float = 0.0, brown_pigments: float = 0.0,
               water: float = 0.01, dry_matter: float = 0.008,
               incidence_angle_deg: float = 40.0) -> dict:
    """2101 samples from 400 to 2500 nm. Pigments: ug/cm²; water/dry matter: g/cm².

    n is dimensionless; brown pigments use the original arbitrary units. The
    incidence parameter is the illumination-cone half-angle in degrees.
    Returns a result/manifest envelope; result arrays have dtype float64.
    """
    parameters = dict(n=n, chlorophyll=chlorophyll, carotenoids=carotenoids,
                      anthocyanins=anthocyanins, brown_pigments=brown_pigments,
                      water=water, dry_matter=dry_matter, incidence_angle_deg=incidence_angle_deg)
    output = _call(_native.prospect_d_native, _dumps(parameters))
    manifest = json.loads(output.pop("manifest_json"))
    return {"result": output, "manifest": manifest}


def prospect_d_batch(parameters: np.ndarray, *, shared_axes=False) -> dict:
    """Rust-computed batch from a real numeric matrix with shape (batch, 8).

    Columns follow PROSPECT_D_PARAMETERS. Output spectra have shape (batch, 2101)
    and a shared 1-D wavelength axis. Inputs are copied before releasing Python's
    GIL; no per-row Python calls or implicit broadcasting occur. Empty batches
    are supported. An invalid row aborts the batch and is identified in the error.
    """
    return run_array('vegetation.prospect-d',parameters,shared_axes=shared_axes)


FLUSPECT_CX_PARAMETERS=('n','chlorophyll','carotenoids','anthocyanins','brown_pigments','water','dry_matter','xanthophyll_state','fluorescence_efficiency','protein','carbon_based_constituents')

def fluspect_cx(*,n:float=1.5,chlorophyll:float=40.,carotenoids:float=8.,anthocyanins:float=0.,brown_pigments:float=0.,water:float=.01,dry_matter:float=.009,xanthophyll_state:float=0.,fluorescence_efficiency:float=.01,protein:float=0.,carbon_based_constituents:float=0.,optical_coefficients:str='prospect_d',spectral_resolution:str='full_1nm')->dict:
    """Complete CX forward optics with full (211,351) or light (53,71) matrices.

    Optical data: prospect_d or prospect_pro; protein/CBC require prospect_pro.
    xanthophyll_state=-999 selects the original fixed-carotenoid spectrum.
    Matrices act on photon spectra, include the excitation-bin width, and combine
    PSI/PSII. Use fluspect_fluorescence for energy-irradiance to energy-flux conversion.
    """
    return run('vegetation.fluspect-cx',locals())

def fluspect_cx_batch(parameters, *, shared_axes=False,optical_coefficients='prospect_d',spectral_resolution='full_1nm')->dict:
    """Native (batch,11) matrix; fluorescence shape (batch,emission,excitation)."""
    return run_array('vegetation.fluspect-cx',parameters,shared_axes=shared_axes,fixed={'optical_coefficients':optical_coefficients,'spectral_resolution':spectral_resolution})

def fluspect_fluorescence(incident_irradiance_w_m2_nm, **parameters)->dict:
    """Apply CX photon-domain transfer to energy irradiance on its excitation grid.

    Returns leaf-side spectral energy flux in W m^-2 nm^-1. The wavelength ratio
    lambda_exc/lambda_em performs photon/energy conversion; bin weight is already
    present in the matrix. No canopy or atmosphere is assumed.
    """
    from ._numeric import real_array
    incident=real_array(incident_irradiance_w_m2_nm,ndim=1,name='incident_irradiance_w_m2_nm')
    return _call(_native.fluspect_irradiance_native,_dumps(parameters),incident)


GOMS_PARAMETERS=('normalized_crown_density', 'crown_axis_ratio', 'crown_height_ratio', 'relative_height_variation', 'sunlit_ground_reflectance', 'sunlit_crown_reflectance', 'shaded_reflectance', 'solar_zenith_deg', 'solar_azimuth_deg', 'view_zenith_deg', 'view_azimuth_deg', 'slope_deg', 'slope_aspect_deg')

def goms(*, normalized_crown_density=.1,crown_axis_ratio=1.733,crown_height_ratio=2.577,relative_height_variation=.769,
         sunlit_ground_reflectance=.2,sunlit_crown_reflectance=.55,shaded_reflectance=.05,
         solar_zenith_deg=45.,solar_azimuth_deg=0.,view_zenith_deg=30.,view_azimuth_deg=0.,slope_deg=0.,slope_aspect_deg=0.,geometry_convention='area_consistent'):
    """GOMS forward reflectance for 1..10 bands and optional sloping terrain.

    Coefficients accept a scalar or equal-length one-dimensional vectors. Scalar
    coefficients broadcast to the explicit band count. No wavelength is inferred.
    Angular outputs have shape (band,angle); validity masks mark horizon failures.
    legacy_c reproduces the supplied C convention; area_consistent corrects its
    overlap units and angular/terrain operations. Scientific arithmetic is Rust.
    """
    from ._numeric import real_array
    params=locals().copy();params.pop('real_array',None)
    keys=('sunlit_ground_reflectance','sunlit_crown_reflectance','shaded_reflectance')
    arrays={key:real_array(np.atleast_1d(params[key]),ndim=1,name=key) for key in keys}
    count=max(map(len,arrays.values()))
    for key,a in arrays.items():params[key]=(np.repeat(a,count) if len(a)==1 else a).tolist()
    return run('vegetation.goms',params)

def goms_batch(parameters, *, shared_axes=False,geometry_convention='area_consistent'):
    """Single-band numeric batch (batch,13); use run_batch for multiband rows."""
    return run_array('vegetation.goms',parameters,shared_axes=shared_axes,fixed={'geometry_convention':geometry_convention})


GOMS_GEOMETRY_PARAMETERS=tuple(k for k in GOMS_PARAMETERS if k not in ('sunlit_ground_reflectance','sunlit_crown_reflectance','shaded_reflectance'))

def goms_multiband_batch(geometry,reflectances, *,geometry_convention='area_consistent',shared_axes=False):
    """Native NumPy multiband GOMS: geometry (batch,10), optics (batch,band,3).

    Geometry columns follow GOMS_GEOMETRY_PARAMETERS; the coefficient axis is
    [sunlit ground, sunlit crown, shared shade]. All rows share 1..10 bands.
    Inputs are copied in Rust before releasing the GIL. Scientific outputs use
    native NumPy buffers; only provenance manifests traverse JSON.
    """
    from ._numeric import real_array
    from ._api import ModelError
    if not isinstance(shared_axes,(bool,np.bool_)):raise ModelError('invalid_parameter','shared_axes must be boolean')
    geo=real_array(geometry,ndim=2,name='geometry');optics=real_array(reflectances,ndim=3,name='reflectances')
    out=_call(_native.goms_multiband_native,geo,optics,geometry_convention,bool(shared_axes))
    out['manifests']=json.loads(out.pop('manifests_json'))
    return out
