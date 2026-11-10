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


FVCB_PARAMETERS = (
    "leaf_temperature_c",
    "co2_surface_umol_mol",
    "intercellular_co2_umol_mol",
    "pressure_pa",
    "oxygen_mmol_mol",
    "vcmax25_umol_m2_s",
    "jmax25_umol_m2_s",
    "respiration25_umol_m2_s",
    "electron_quantum_yield",
    "light_response_curvature",
    "tpu_umol_m2_s",
    "vapour_pressure_deficit_kpa",
    "minimum_stomatal_conductance_water_mol_m2_s",
    "ball_berry_slope",
    "leuning_slope_co2",
    "leuning_d0_kpa",
    "medlyn_slope_kpa_sqrt",
    "medlyn_vpd_floor_kpa",
    "vcmax_activation_j_mol",
    "jmax_activation_j_mol",
    "respiration_activation_j_mol",
    "vcmax_deactivation_j_mol",
    "jmax_deactivation_j_mol",
    "vcmax_entropy_j_mol_k",
    "jmax_entropy_j_mol_k",
    "absorbed_par_umol_m2_s",
)

def fvcb(*,
    absorbed_par_umol_m2_s: float = 1200.0,
    leaf_temperature_c: float = 25.0,
    co2_surface_umol_mol: float = 400.0,
    intercellular_co2_umol_mol: float = 280.0,
    pressure_pa: float = 101325.0,
    oxygen_mmol_mol: float = 210.0,
    vcmax25_umol_m2_s: float = 80.0,
    jmax25_umol_m2_s: float = 160.0,
    respiration25_umol_m2_s: float = 1.2,
    electron_quantum_yield: float = 0.24,
    light_response_curvature: float = 0.7,
    tpu_umol_m2_s: float = 12.0,
    vapour_pressure_deficit_kpa: float = 1.0,
    minimum_stomatal_conductance_water_mol_m2_s: float = 0.01,
    ball_berry_slope: float = 9.0,
    leuning_slope_co2: float = 6.0,
    leuning_d0_kpa: float = 1.5,
    medlyn_slope_kpa_sqrt: float = 4.0,
    medlyn_vpd_floor_kpa: float = 0.05,
    vcmax_activation_j_mol: float = 65330.0,
    jmax_activation_j_mol: float = 43540.0,
    respiration_activation_j_mol: float = 46390.0,
    vcmax_deactivation_j_mol: float = 200000.0,
    jmax_deactivation_j_mol: float = 200000.0,
    vcmax_entropy_j_mol_k: float = 650.0,
    jmax_entropy_j_mol_k: float = 650.0,
    gas_exchange_mode: str = "stomatal",
    stomatal_model: str = "medlyn",
    temperature_response: str = "arrhenius",
    tpu_limitation: str = "off",
    leuning_compensation: str = "net_compensation",
)->dict:
    """Leaf FvCB; prescribed Ci or explicit stomatal solution, fluxes per leaf area."""
    return run("ecosystem.fvcb",locals())

def fvcb_batch(parameters, *, shared_axes=False, gas_exchange_mode="stomatal", stomatal_model="medlyn", temperature_response="arrhenius", tpu_limitation="off", leuning_compensation="net_compensation")->dict:
    """Native NumPy batch; undefined conductance/Ci values are NaN with explicit validity flags."""
    return run_array("ecosystem.fvcb", parameters, shared_axes=shared_axes, fixed={"gas_exchange_mode":gas_exchange_mode,"stomatal_model":stomatal_model,"temperature_response":temperature_response,"tpu_limitation":tpu_limitation,"leuning_compensation":leuning_compensation})


FVCB_BIG_LEAF_PARAMETERS = (
    "leaf_temperature_c",
    "co2_surface_umol_mol",
    "intercellular_co2_umol_mol",
    "pressure_pa",
    "oxygen_mmol_mol",
    "vcmax25_umol_m2_s",
    "jmax25_umol_m2_s",
    "respiration25_umol_m2_s",
    "electron_quantum_yield",
    "light_response_curvature",
    "tpu_umol_m2_s",
    "vapour_pressure_deficit_kpa",
    "minimum_stomatal_conductance_water_mol_m2_s",
    "ball_berry_slope",
    "leuning_slope_co2",
    "leuning_d0_kpa",
    "medlyn_slope_kpa_sqrt",
    "medlyn_vpd_floor_kpa",
    "vcmax_activation_j_mol",
    "jmax_activation_j_mol",
    "respiration_activation_j_mol",
    "vcmax_deactivation_j_mol",
    "jmax_deactivation_j_mol",
    "vcmax_entropy_j_mol_k",
    "jmax_entropy_j_mol_k",
    "absorbed_par_ground_umol_m2_s",
    "leaf_area_index",
    "nitrogen_extinction_coefficient",
)

def fvcb_big_leaf(*,
    absorbed_par_ground_umol_m2_s: float = 1200.0,
    leaf_area_index: float = 3.0,
    nitrogen_extinction_coefficient: float = 0.3,
    leaf_temperature_c: float = 25.0,
    co2_surface_umol_mol: float = 400.0,
    intercellular_co2_umol_mol: float = 280.0,
    pressure_pa: float = 101325.0,
    oxygen_mmol_mol: float = 210.0,
    vcmax25_umol_m2_s: float = 80.0,
    jmax25_umol_m2_s: float = 160.0,
    respiration25_umol_m2_s: float = 1.2,
    electron_quantum_yield: float = 0.24,
    light_response_curvature: float = 0.7,
    tpu_umol_m2_s: float = 12.0,
    vapour_pressure_deficit_kpa: float = 1.0,
    minimum_stomatal_conductance_water_mol_m2_s: float = 0.01,
    ball_berry_slope: float = 9.0,
    leuning_slope_co2: float = 6.0,
    leuning_d0_kpa: float = 1.5,
    medlyn_slope_kpa_sqrt: float = 4.0,
    medlyn_vpd_floor_kpa: float = 0.05,
    vcmax_activation_j_mol: float = 65330.0,
    jmax_activation_j_mol: float = 43540.0,
    respiration_activation_j_mol: float = 46390.0,
    vcmax_deactivation_j_mol: float = 200000.0,
    jmax_deactivation_j_mol: float = 200000.0,
    vcmax_entropy_j_mol_k: float = 650.0,
    jmax_entropy_j_mol_k: float = 650.0,
    gas_exchange_mode: str = "stomatal",
    stomatal_model: str = "medlyn",
    temperature_response: str = "arrhenius",
    tpu_limitation: str = "off",
    leuning_compensation: str = "net_compensation",
)->dict:
    """Single homogeneous big-leaf FvCB; fluxes per ground area."""
    return run("ecosystem.fvcb-big-leaf",locals())

def fvcb_big_leaf_batch(parameters, *, shared_axes=False, gas_exchange_mode="stomatal", stomatal_model="medlyn", temperature_response="arrhenius", tpu_limitation="off", leuning_compensation="net_compensation")->dict:
    """Native NumPy batch; undefined conductance/Ci values are NaN with explicit validity flags."""
    return run_array("ecosystem.fvcb-big-leaf", parameters, shared_axes=shared_axes, fixed={"gas_exchange_mode":gas_exchange_mode,"stomatal_model":stomatal_model,"temperature_response":temperature_response,"tpu_limitation":tpu_limitation,"leuning_compensation":leuning_compensation})
