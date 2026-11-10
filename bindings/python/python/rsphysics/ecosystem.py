"""Ecosystem carbon and canopy physiology models."""
from ._api import run
from ._arrays import run_array


def gort_gpp(*,
    leaf_area_index:float=4.31,
    crown_radius_m:float=1.7,
    crown_density_m2:float=0.1905,
    crown_base_height_m:float=7.8,
    crown_top_height_m:float=14.1,
    crown_axis_ratio:float=1.17,
    solar_zenith_deg:float=40.0,
    direct_par_umol_m2_s:float=1200.0,
    diffuse_par_umol_m2_s:float=200.0,
    air_temperature_c:float=25.0,
    vapour_pressure_deficit_pa:float=1000.0,
    co2_ppm:float=400.0,
    young_leaf_fraction:float=0.4,
    vcmax25_young:float=52.1,
    vcmax25_mature:float=34.17,
    pressure_pa:float=101325.0,
    stomatal_slope:float=8.0,
    electron_photon_ratio:float=2.6,
    leaf_angle_parameter:float=0.5,
    leaf_single_scattering_albedo:float=0.1,
    background_reflectance:float=0.1,
    minimum_stomatal_conductance:float=0.01,numerical_convention:str="corrected"
)->dict:
    """Run the selected supplied-reference forward parameterization; see model documentation."""
    return run('ecosystem.gort-gpp',locals())

def gort_gpp_batch(parameters, *, shared_axes=False,numerical_convention="corrected")->dict:
    """Native NumPy batch in parameters_for('ecosystem.gort-gpp') column order."""
    return run_array('ecosystem.gort-gpp',parameters,shared_axes=shared_axes,fixed={'numerical_convention':numerical_convention})
