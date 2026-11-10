"""Leaf optics. Parameters are model-owned; units follow the author PROSPECT-D implementation."""
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
    "view_zenith_deg", "relative_azimuth_deg", "soil_brightness", "soil_dry_fraction",
)


def prosail_d(*, n: float = 1.5, chlorophyll: float = 40.0, carotenoids: float = 8.0,
             anthocyanins: float = 0.0, brown_pigments: float = 0.0,
             water: float = 0.01, dry_matter: float = 0.008, incidence_angle_deg: float = 40.0,
             leaf_area_index: float = 3.0, mean_leaf_angle_deg: float = 60.0,
             hotspot: float = 0.1, solar_zenith_deg: float = 30.0, view_zenith_deg: float = 0.0,
             relative_azimuth_deg: float = 60.0, soil_brightness: float = 1.0,
             soil_dry_fraction: float = 0.5) -> dict:
    """PROSPECT-D + 4SAIL with the author's 13-bin Campbell distribution.

    Direct illumination; angles in degrees. Returns float64 ``brf`` (may exceed
    one) and ``hemispherical_reflectance`` (original rsdt), never transmittance.
    Soil dry fraction blends the original dry/wet endmembers; it is not moisture.
    LAI=0 uses the analytical bare-soil limit. Equations run exclusively in Rust.
    """
    return run("vegetation.prosail-d", locals())


def prosail_d_batch(parameters: np.ndarray, *, shared_axes=False) -> dict:
    """Ordered Rust batch, real numeric matrix (batch, 16), PROSAIL_D_PARAMETERS.

    Scientific arrays use native NumPy buffers. Spectra are float64 with
    shape (batch, 2101); the wavelength axis is shared. No Python scientific loop.
    """
    return run_array('vegetation.prosail-d',parameters,shared_axes=shared_axes)


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


FLUSPECT_CX_PARAMETERS=('n','chlorophyll','carotenoids','anthocyanins','brown_pigments','water','dry_matter','xanthophyll_state','fluorescence_efficiency')

def fluspect_cx(*,n:float=1.5,chlorophyll:float=40.,carotenoids:float=8.,anthocyanins:float=0.,brown_pigments:float=0.,water:float=.01,dry_matter:float=.009,xanthophyll_state:float=0.,fluorescence_efficiency:float=.01)->dict:
    """Leaf R/T (400..2400 nm) and combined-photosystem fluorescence matrices.

    Matrices are (emission=53, excitation=71): 640..848/4 nm, 400..750/5 nm.
    They include the original 5 nm excitation integration weight; no additional
    weight is needed for a matrix-vector product with spectral irradiance.
    No illumination is assumed and matrices are not absolute SIF radiance.
    """
    return run('vegetation.fluspect-cx',locals())

def fluspect_cx_batch(parameters, *, shared_axes=False)->dict:
    """Native NumPy (batch,9) in FLUSPECT_CX_PARAMETERS order.

    R/T shape (batch,2001), matrices (batch,53,71). shared_axes=True returns
    all coordinate arrays once in coordinates, including the two matrix axes.
    """
    return run_array('vegetation.fluspect-cx',parameters,shared_axes=shared_axes)
