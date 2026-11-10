"""Soil optical radiative transfer models."""
from ._api import run
from ._arrays import run_array

def sort(*,
    solar_zenith_deg: float = 45.0,
    view_zenith_deg: float = 0.0,
    relative_azimuth_deg: float = 0.0,
    phase_asymmetry: float = 0.4,
    phase_backscatter: float = 0.0,
    hotspot_amplitude: float = 0.4,
    hotspot_width: float = 0.1,
    solid_filling_factor: float = 0.2,
    rms_slope: float = 0.4,
    roughness_correction: str = 'none',
    multifacet_reflectance: float = 0.2,
    soil_brightness: float = 0.6,
    spectral_alpha_deg: float = 15.0,
    spectral_gamma_deg: float = 50.0,
    water_coverage: float = 0.3,
    water_film_thickness_mm: float = 0.3,
    particle_volume_fraction: float = 0.05,
    particle_refractive_index: float = 1.53,
    particle_extinction_coefficient: float = 0.001,
    substrate_angle_mode:str='refracted',water_film_mode:str='legacy_scalar'
) -> dict:
    """Run the SORT soil optical forward model."""
    return run("soil.sort", locals())

def sort_l(*,
    soil_brightness: float = 0.6,
    spectral_alpha_deg: float = 20.0,
    spectral_gamma_deg: float = 50.0,
    water_coverage: float = 0.5,
    water_film_thickness_mm: float = 0.3,
    particle_volume_fraction: float = 0.05,
    particle_refractive_index: float = 1.53,
    particle_extinction_coefficient: float = 0.001
) -> dict:
    """Run the SORT-L soil optical forward model."""
    return run("soil.sort-l", locals())

def sort_batch(parameters, *, roughness_correction="none", substrate_angle_mode="refracted",water_film_mode="legacy_scalar",shared_axes=False):
    """NumPy batch in parameters_for("soil.sort") column order."""
    return run_array("soil.sort", parameters, shared_axes=shared_axes, fixed={"roughness_correction":roughness_correction,"substrate_angle_mode":substrate_angle_mode,"water_film_mode":water_film_mode})

def sort_l_batch(parameters, *, shared_axes=False):
    """NumPy batch in parameters_for("soil.sort-l") column order."""
    return run_array("soil.sort-l", parameters, shared_axes=shared_axes)


def gsv(*, c1=0.4, c2=-0.01, c3=0.04, csm=-0.13, variant="gsv3", wavelengths_nm=None):
    """GSV1/GSV3 soil spectrum; csm is a vector coefficient, not water content.

    None or [] returns the native 400..2500nm axis, step10. Explicit wavelengths
    are linearly interpolated in Rust, without extrapolation or clipping.
    """
    from ._numeric import real_array
    wavelengths_nm=[] if wavelengths_nm is None else real_array(wavelengths_nm,ndim=1,name="wavelengths_nm").tolist()
    return run("soil.gsv", {"c1":c1,"c2":c2,"c3":c3,"csm":csm,"variant":variant,"wavelengths_nm":wavelengths_nm})


def gsv_batch(parameters, *, variant="gsv3", wavelengths_nm=None, shared_axes=False):
    """Native NumPy batch, columns c1,c2,c3,csm; one shared wavelength request."""
    from ._numeric import real_array
    wl=[] if wavelengths_nm is None else real_array(wavelengths_nm,ndim=1,name="wavelengths_nm").tolist()
    return run_array("soil.gsv",parameters,shared_axes=shared_axes,fixed={"variant":variant,"wavelengths_nm":wl})


def bsm(*, soil_brightness=0.5, shape_latitude_deg=-10.0, shape_longitude_deg=100.0,
        soil_moisture_percent=25.0, moisture_capacity_percent=25.0,
        water_film_thickness_cm=0.015, poisson_mode="complete"):
    """BSM soil reflectance, 400..2400nm. Moisture uses percentage points.

    Complete mode sums the Poisson ensemble to omitted mass<1e-14. The explicit
    legacy_six_films mode reproduces the author implementation's k=0..6 sum.
    """
    return run("soil.bsm",locals())


def bsm_batch(parameters, *, poisson_mode="complete", shared_axes=False):
    """BSM NumPy batch in parameters_for('soil.bsm') column order."""
    return run_array("soil.bsm",parameters,shared_axes=shared_axes,fixed={"poisson_mode":poisson_mode})


def soilspect(*, single_scattering_albedo=0.6, roughness_h=0.1, phase_b=0.3,
             phase_c=0.1, phase_b_prime=0.0, phase_c_prime=0.0,
             solar_zenith_deg=30.0, view_zenith_deg=20.0, relative_azimuth_deg=0.0,
             quadrature="standard"):
    """SOILSPECT four-term phase function, one spectral band.

    Includes selected BRF/BRDF, single/multiple components, full-equation
    directional/bihemispherical integrals and angular grids. Use batches for
    an externally supplied single-scattering-albedo spectrum. Angles in degrees.
    """
    return run("soil.soilspect",locals())


def soilspect_batch(parameters, *, quadrature="standard", shared_axes=False):
    """SOILSPECT NumPy batch in parameters_for('soil.soilspect') order."""
    return run_array("soil.soilspect",parameters,shared_axes=shared_axes,fixed={"quadrature":quadrature})
