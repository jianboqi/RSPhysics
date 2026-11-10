"""Thermal infrared canopy radiative transfer with prescribed temperatures."""
from ._api import run
from ._arrays import run_array

FOURSAIL_PARAMETERS=('wavelength_nm', 'leaf_area_index', 'mean_leaf_angle_deg', 'lidf_a', 'lidf_b', 'hotspot', 'solar_zenith_deg', 'view_zenith_deg', 'relative_azimuth_deg', 'leaf_emissivity', 'soil_emissivity', 'sunlit_leaf_temperature_k', 'shaded_leaf_temperature_k', 'sunlit_soil_temperature_k', 'shaded_soil_temperature_k', 'sky_brightness_temperature_k', 'direct_irradiance_w_m2_um')

def foursail(*,wavelength_nm=9500.0,leaf_area_index=3.0,mean_leaf_angle_deg=60.0,lidf_a=-0.35,lidf_b=-0.15,hotspot=0.1,solar_zenith_deg=30.0,view_zenith_deg=0.0,relative_azimuth_deg=60.0,leaf_emissivity=0.98,soil_emissivity=0.95,sunlit_leaf_temperature_k=310.0,shaded_leaf_temperature_k=300.0,sunlit_soil_temperature_k=320.0,shaded_soil_temperature_k=305.0,sky_brightness_temperature_k=270.0,direct_irradiance_w_m2_um=0.0,leaf_angle_distribution='campbell'):
    """Four-component 4SAIL-TIR. Temperatures K; wavelength nm; radiance W/m²/sr/um.

    Opaque Lambertian leaves and soil. Sky is isotropic B_lambda(Tsky), not air
    temperature. Optional direct irradiance is measured on a horizontal surface.
    The fixed 8–14 um diagnostic holds emissivities and sky temperature constant.
    For arbitrary spectral emissivity, vary wavelength and emissivity in batches.
    """
    return run('thermal.foursail',locals())

def foursail_batch(parameters, *,shared_axes=False,leaf_angle_distribution='campbell'):
    """Native NumPy (batch,17) input in FOURSAIL_PARAMETERS order; released GIL."""
    return run_array('thermal.foursail',parameters,shared_axes=shared_axes,fixed={'leaf_angle_distribution':leaf_angle_distribution})
