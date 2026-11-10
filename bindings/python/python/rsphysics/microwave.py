"""Microwave forward models evaluated in Rust.

Tau-omega and WCM take channel-specific soil responses directly; L-MEB adds
the supplied 1.41 GHz soil/vegetation forward parameterization with dielectric,
roughness, dual-polarization, and prescribed sky terms. See model-specific
documentation for physical inputs, validity, and source permissions.
"""
import numpy as np
from ._api import run
from ._arrays import run_array

TAU_OMEGA_PARAMETERS=('soil_temperature_k', 'vegetation_temperature_k', 'soil_reflectivity', 'vegetation_optical_depth', 'single_scattering_albedo', 'observation_zenith_deg')
def tau_omega(*,soil_temperature_k:float=300,vegetation_temperature_k:float=295,soil_reflectivity:float=0.3,vegetation_optical_depth:float=0.3,single_scattering_albedo:float=0.05,observation_zenith_deg:float=40)->dict:
    """TB and soil/direct-canopy/reflected-canopy contributions in K; optical depth is NADIR tau. Soil reflectivity is linear power, supplied directly. No sky/atmosphere."""
    return run('microwave.tau-omega',locals())

def tau_omega_batch(parameters:np.ndarray, *, shared_axes=False)->dict:
    """Real (batch,6) matrix, columns TAU_OMEGA_PARAMETERS; scalar arrays (batch,), sweep arrays (batch,81)."""
    return run_array('microwave.tau-omega',parameters,shared_axes=shared_axes)

WATER_CLOUD_PARAMETERS=('vegetation_water_content', 'scattering_coefficient', 'attenuation_coefficient', 'soil_backscatter', 'observation_zenith_deg')
def water_cloud(*,vegetation_water_content:float=1,scattering_coefficient:float=0.1,attenuation_coefficient:float=0.2,soil_backscatter:float=0.1,observation_zenith_deg:float=40)->dict:
    """V1=V2=VWC [kg/m2]. A and B in m2/kg require channel-specific calibration. Input bare-soil sigma0 and ALL returned sigma0 contributions are LINEAR POWER, never dB. Exact zero remains zero; values above one are allowed."""
    return run('microwave.water-cloud',locals())

def water_cloud_batch(parameters:np.ndarray, *, shared_axes=False)->dict:
    """Real (batch,5) matrix, columns WATER_CLOUD_PARAMETERS; scalar arrays (batch,), sweep arrays (batch,81)."""
    return run_array('microwave.water-cloud',parameters,shared_axes=shared_axes)


def lmeb(*,
    soil_moisture:float=0.07,
    nadir_optical_depth:float=0.0,
    sky_air_temperature_k:float=300.0,
    canopy_temperature_k:float=300.0,
    surface_temperature_k:float=300.0,
    deep_temperature_k:float=283.0,
    bulk_density_g_cm3:float=1.28,
    sand_fraction:float=0.132,
    clay_fraction:float=0.328,
    temperature_moisture_scale:float=0.3,
    temperature_moisture_exponent:float=0.3,
    canopy_structure_h:float=2.0,
    canopy_structure_v_ratio:float=1.0,
    scattering_albedo_h:float=0.05,
    scattering_albedo_v_difference:float=0.0,
    roughness_height:float=0.1,
    roughness_mixing:float=0.0,
    roughness_exponent_v:float=0.0,
    roughness_exponent_h:float=0.0,
    observation_zenith_deg:float=40.0
)->dict:
    """Run the selected supplied-reference forward parameterization; see model documentation."""
    return run('microwave.lmeb',locals())

def lmeb_batch(parameters, *, shared_axes=False)->dict:
    """Native NumPy batch in parameters_for('microwave.lmeb') column order."""
    return run_array('microwave.lmeb',parameters,shared_axes=shared_axes)
