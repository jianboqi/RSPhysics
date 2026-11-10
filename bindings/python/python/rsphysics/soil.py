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
