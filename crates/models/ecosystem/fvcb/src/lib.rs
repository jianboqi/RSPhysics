//! Leaf FvCB biochemistry and an explicit homogeneous single-big-leaf scaling.
mod solver;
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub use solver::{Point, evaluate_big_leaf, evaluate_leaf};
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(inline)]
pub enum GasExchangeMode {
    #[default]
    #[serde(rename = "stomatal")]
    Stomatal,
    #[serde(rename = "prescribed_ci")]
    PrescribedCi,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(inline)]
pub enum StomatalModel {
    #[default]
    #[serde(rename = "medlyn")]
    Medlyn,
    #[serde(rename = "ball_berry")]
    BallBerry,
    #[serde(rename = "leuning")]
    Leuning,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(inline)]
pub enum TemperatureResponse {
    #[default]
    #[serde(rename = "arrhenius")]
    Arrhenius,
    #[serde(rename = "peaked_arrhenius")]
    PeakedArrhenius,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(inline)]
pub enum TpuLimitation {
    #[default]
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "simple")]
    Simple,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(inline)]
pub enum LeuningCompensation {
    #[default]
    #[serde(rename = "net_compensation")]
    NetCompensation,
    #[serde(rename = "gamma_star")]
    GammaStar,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct Parameters {
    /// Prescribed leaf temperature; biochemical temperature functions are calibrated over 10–40 degC.
    #[schemars(range(min=10.0, max=40.0), extend("x-unit"="degC"))]
    pub leaf_temperature_c: f64,
    /// CO2 mole fraction at the leaf surface, not atmospheric partial pressure.
    #[schemars(range(min=1.0, max=3000.0), extend("x-unit"="umol mol^-1"))]
    pub co2_surface_umol_mol: f64,
    /// Prescribed Ci, used only in prescribed_ci mode.
    #[schemars(range(min=0.0, max=5000.0), extend("x-unit"="umol mol^-1"))]
    pub intercellular_co2_umol_mol: f64,
    /// Total pressure; kinetic partial-pressure constants are converted to mole fractions.
    #[schemars(range(min=50000.0, max=110000.0), extend("x-unit"="Pa"))]
    pub pressure_pa: f64,
    /// Intercellular oxygen mole fraction.
    #[schemars(range(min=0.0, max=1000.0), extend("x-unit"="mmol mol^-1"))]
    pub oxygen_mmol_mol: f64,
    /// Rubisco capacity at 25 degC per leaf area, or per top-canopy leaf area in the big-leaf model.
    #[schemars(range(min=0.0, max=500.0), extend("x-unit"="umol m^-2 s^-1"))]
    pub vcmax25_umol_m2_s: f64,
    /// Maximum electron transport at 25 degC per leaf area, or top-canopy leaf area.
    #[schemars(range(min=0.0, max=1000.0), extend("x-unit"="umol m^-2 s^-1"))]
    pub jmax25_umol_m2_s: f64,
    /// Day respiration at 25 degC on the same leaf-area basis as Vcmax25.
    #[schemars(range(min=0.0, max=20.0), extend("x-unit"="umol m^-2 s^-1"))]
    pub respiration25_umol_m2_s: f64,
    /// Electron transport quantum yield per absorbed PAR photon; not a CO2 quantum yield.
    #[schemars(range(min=0.0, max=0.5), extend("x-unit"="mol electron mol^-1 photon"))]
    pub electron_quantum_yield: f64,
    /// Non-rectangular hyperbola curvature; zero and one use their continuous limits.
    #[schemars(range(min=0.0, max=1.0), extend("x-unit"="1"))]
    pub light_response_curvature: f64,
    /// Triose phosphate utilization at the prescribed temperature; used only in simple TPU mode.
    #[schemars(range(min=0.0, max=100.0), extend("x-unit"="umol m^-2 s^-1"))]
    pub tpu_umol_m2_s: f64,
    /// Leaf-to-leaf-surface water-vapour deficit; the leaf surface is the imposed humidity boundary.
    #[schemars(range(min=0.0, max=8.0), extend("x-unit"="kPa"))]
    pub vapour_pressure_deficit_kpa: f64,
    /// Residual stomatal conductance to water vapour per leaf area.
    #[schemars(range(min=0.0, max=1.0), extend("x-unit"="mol m^-2 s^-1"))]
    pub minimum_stomatal_conductance_water_mol_m2_s: f64,
    /// Ball–Berry slope on a water-vapour conductance basis.
    #[schemars(range(min=0.0, max=40.0), extend("x-unit"="1"))]
    pub ball_berry_slope: f64,
    /// Leuning slope on a CO2 conductance basis; water conductance includes the factor 1.6.
    #[schemars(range(min=0.0, max=30.0), extend("x-unit"="1"))]
    pub leuning_slope_co2: f64,
    /// Leuning humidity-deficit scale.
    #[schemars(range(min=0.01, max=10.0), extend("x-unit"="kPa"))]
    pub leuning_d0_kpa: f64,
    /// Medlyn slope, distinct in definition and units from the other stomatal slopes.
    #[schemars(range(min=0.0, max=20.0), extend("x-unit"="kPa^0.5"))]
    pub medlyn_slope_kpa_sqrt: f64,
    /// Explicit VPD regularization for the Medlyn square root; actual transpiration uses the actual VPD.
    #[schemars(range(min=1e-06, max=1.0), extend("x-unit"="kPa"))]
    pub medlyn_vpd_floor_kpa: f64,
    /// Vcmax activation energy; demonstration default, not a species calibration.
    #[schemars(range(min=0.0, max=200000.0), extend("x-unit"="J mol^-1"))]
    pub vcmax_activation_j_mol: f64,
    /// Jmax activation energy; demonstration default, not a species calibration.
    #[schemars(range(min=0.0, max=200000.0), extend("x-unit"="J mol^-1"))]
    pub jmax_activation_j_mol: f64,
    /// Day-respiration activation energy.
    #[schemars(range(min=0.0, max=200000.0), extend("x-unit"="J mol^-1"))]
    pub respiration_activation_j_mol: f64,
    /// Vcmax deactivation energy, used by peaked Arrhenius only.
    #[schemars(range(min=50000.0, max=400000.0), extend("x-unit"="J mol^-1"))]
    pub vcmax_deactivation_j_mol: f64,
    /// Jmax deactivation energy, used by peaked Arrhenius only.
    #[schemars(range(min=50000.0, max=400000.0), extend("x-unit"="J mol^-1"))]
    pub jmax_deactivation_j_mol: f64,
    /// Vcmax entropy factor, used by peaked Arrhenius only.
    #[schemars(range(min=100.0, max=1000.0), extend("x-unit"="J mol^-1 K^-1"))]
    pub vcmax_entropy_j_mol_k: f64,
    /// Jmax entropy factor, used by peaked Arrhenius only.
    #[schemars(range(min=100.0, max=1000.0), extend("x-unit"="J mol^-1 K^-1"))]
    pub jmax_entropy_j_mol_k: f64,
    pub gas_exchange_mode: GasExchangeMode,
    pub stomatal_model: StomatalModel,
    pub temperature_response: TemperatureResponse,
    pub tpu_limitation: TpuLimitation,
    pub leuning_compensation: LeuningCompensation,
}
impl Default for Parameters {
    fn default() -> Self {
        Self {
            leaf_temperature_c: 25.0,
            co2_surface_umol_mol: 400.0,
            intercellular_co2_umol_mol: 280.0,
            pressure_pa: 101325.0,
            oxygen_mmol_mol: 210.0,
            vcmax25_umol_m2_s: 80.0,
            jmax25_umol_m2_s: 160.0,
            respiration25_umol_m2_s: 1.2,
            electron_quantum_yield: 0.24,
            light_response_curvature: 0.7,
            tpu_umol_m2_s: 12.0,
            vapour_pressure_deficit_kpa: 1.0,
            minimum_stomatal_conductance_water_mol_m2_s: 0.01,
            ball_berry_slope: 9.0,
            leuning_slope_co2: 6.0,
            leuning_d0_kpa: 1.5,
            medlyn_slope_kpa_sqrt: 4.0,
            medlyn_vpd_floor_kpa: 0.05,
            vcmax_activation_j_mol: 65330.0,
            jmax_activation_j_mol: 43540.0,
            respiration_activation_j_mol: 46390.0,
            vcmax_deactivation_j_mol: 200000.0,
            jmax_deactivation_j_mol: 200000.0,
            vcmax_entropy_j_mol_k: 650.0,
            jmax_entropy_j_mol_k: 650.0,
            gas_exchange_mode: GasExchangeMode::Stomatal,
            stomatal_model: StomatalModel::Medlyn,
            temperature_response: TemperatureResponse::Arrhenius,
            tpu_limitation: TpuLimitation::Off,
            leuning_compensation: LeuningCompensation::NetCompensation,
        }
    }
}
impl Parameters {
    pub fn validate(&self) -> Result<(), ModelError> {
        for (key, value, lo, hi) in [
            ("leaf_temperature_c", self.leaf_temperature_c, 10.0, 40.0),
            (
                "co2_surface_umol_mol",
                self.co2_surface_umol_mol,
                1.0,
                3000.0,
            ),
            (
                "intercellular_co2_umol_mol",
                self.intercellular_co2_umol_mol,
                0.0,
                5000.0,
            ),
            ("pressure_pa", self.pressure_pa, 50000.0, 110000.0),
            ("oxygen_mmol_mol", self.oxygen_mmol_mol, 0.0, 1000.0),
            ("vcmax25_umol_m2_s", self.vcmax25_umol_m2_s, 0.0, 500.0),
            ("jmax25_umol_m2_s", self.jmax25_umol_m2_s, 0.0, 1000.0),
            (
                "respiration25_umol_m2_s",
                self.respiration25_umol_m2_s,
                0.0,
                20.0,
            ),
            (
                "electron_quantum_yield",
                self.electron_quantum_yield,
                0.0,
                0.5,
            ),
            (
                "light_response_curvature",
                self.light_response_curvature,
                0.0,
                1.0,
            ),
            ("tpu_umol_m2_s", self.tpu_umol_m2_s, 0.0, 100.0),
            (
                "vapour_pressure_deficit_kpa",
                self.vapour_pressure_deficit_kpa,
                0.0,
                8.0,
            ),
            (
                "minimum_stomatal_conductance_water_mol_m2_s",
                self.minimum_stomatal_conductance_water_mol_m2_s,
                0.0,
                1.0,
            ),
            ("ball_berry_slope", self.ball_berry_slope, 0.0, 40.0),
            ("leuning_slope_co2", self.leuning_slope_co2, 0.0, 30.0),
            ("leuning_d0_kpa", self.leuning_d0_kpa, 0.01, 10.0),
            (
                "medlyn_slope_kpa_sqrt",
                self.medlyn_slope_kpa_sqrt,
                0.0,
                20.0,
            ),
            (
                "medlyn_vpd_floor_kpa",
                self.medlyn_vpd_floor_kpa,
                1e-06,
                1.0,
            ),
            (
                "vcmax_activation_j_mol",
                self.vcmax_activation_j_mol,
                0.0,
                200000.0,
            ),
            (
                "jmax_activation_j_mol",
                self.jmax_activation_j_mol,
                0.0,
                200000.0,
            ),
            (
                "respiration_activation_j_mol",
                self.respiration_activation_j_mol,
                0.0,
                200000.0,
            ),
            (
                "vcmax_deactivation_j_mol",
                self.vcmax_deactivation_j_mol,
                50000.0,
                400000.0,
            ),
            (
                "jmax_deactivation_j_mol",
                self.jmax_deactivation_j_mol,
                50000.0,
                400000.0,
            ),
            (
                "vcmax_entropy_j_mol_k",
                self.vcmax_entropy_j_mol_k,
                100.0,
                1000.0,
            ),
            (
                "jmax_entropy_j_mol_k",
                self.jmax_entropy_j_mol_k,
                100.0,
                1000.0,
            ),
        ] {
            if !value.is_finite() || !(lo..=hi).contains(&value) {
                return Err(domain(format!("{key} must be finite in [{lo}, {hi}]")));
            }
        }
        if self.gas_exchange_mode == GasExchangeMode::Stomatal
            && self.vapour_pressure_deficit_kpa
                > solver::saturation_pressure_kpa(self.leaf_temperature_c)
        {
            return Err(domain(
                "leaf-surface VPD exceeds saturation vapour pressure; relative humidity would be negative",
            ));
        }
        if self.temperature_response == TemperatureResponse::PeakedArrhenius
            && (self.vcmax_deactivation_j_mol <= self.vcmax_activation_j_mol
                || self.jmax_deactivation_j_mol <= self.jmax_activation_j_mol)
        {
            return Err(domain(
                "peaked Arrhenius requires deactivation energy greater than activation energy",
            ));
        }
        Ok(())
    }
}
pub(crate) fn domain(message: impl Into<String>) -> ModelError {
    ModelError::new(ErrorKind::OutOfDomain, message)
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
#[schemars(deny_unknown_fields)]
pub struct Input {
    #[serde(flatten)]
    pub parameters: Parameters,
    /// Absorbed PAR per leaf area; use absorbed photons, not incident radiation or W/m2.
    #[schemars(range(min=0.0,max=4000.0),extend("x-unit"="umol m^-2 s^-1"))]
    pub absorbed_par_umol_m2_s: f64,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            parameters: Parameters::default(),
            absorbed_par_umol_m2_s: 1200.,
        }
    }
}
impl Input {
    pub fn validate(&self) -> Result<(), ModelError> {
        self.parameters.validate()?;
        check(
            "absorbed_par_umol_m2_s",
            self.absorbed_par_umol_m2_s,
            0.,
            4000.,
        )
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
#[schemars(deny_unknown_fields)]
pub struct BigLeafInput {
    #[serde(flatten)]
    pub parameters: Parameters,
    /// Absorbed canopy PAR per horizontal ground area, supplied directly; no radiation-transfer model is assumed.
    #[schemars(range(min=0.0,max=4000.0),extend("x-unit"="umol m^-2 s^-1"))]
    pub absorbed_par_ground_umol_m2_s: f64,
    #[schemars(range(min=0.0,max=12.0),extend("x-unit"="m^2 m^-2"))]
    pub leaf_area_index: f64,
    /// Exponential decline of biochemical capacity with cumulative LAI; zero is uniform capacity.
    #[schemars(range(min=0.0,max=3.0),extend("x-unit"="1"))]
    pub nitrogen_extinction_coefficient: f64,
}
impl Default for BigLeafInput {
    fn default() -> Self {
        Self {
            parameters: Parameters::default(),
            absorbed_par_ground_umol_m2_s: 1200.,
            leaf_area_index: 3.,
            nitrogen_extinction_coefficient: 0.3,
        }
    }
}
impl BigLeafInput {
    pub fn validate(&self) -> Result<(), ModelError> {
        self.parameters.validate()?;
        check(
            "absorbed_par_ground_umol_m2_s",
            self.absorbed_par_ground_umol_m2_s,
            0.,
            4000.,
        )?;
        check("leaf_area_index", self.leaf_area_index, 0., 12.)?;
        check(
            "nitrogen_extinction_coefficient",
            self.nitrogen_extinction_coefficient,
            0.,
            3.,
        )?;
        if self.leaf_area_index == 0. && self.absorbed_par_ground_umol_m2_s != 0. {
            return Err(domain("a zero-leaf-area canopy cannot absorb PAR"));
        }
        Ok(())
    }
}
fn check(key: &str, value: f64, lo: f64, hi: f64) -> Result<(), ModelError> {
    if !value.is_finite() || !(lo..=hi).contains(&value) {
        Err(domain(format!("{key} must be finite in [{lo},{hi}]")))
    } else {
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Output {
    #[serde(flatten)]
    pub point: Point,
    pub capacity_area_factor: f64,
    pub leaf_area_factor: f64,
    pub ci_axis_umol_mol: Vec<f64>,
    pub ci_net_assimilation: Vec<f64>,
    pub ci_rubisco_limited_assimilation: Vec<f64>,
    pub ci_electron_limited_assimilation: Vec<f64>,
    pub light_axis_umol_m2_s: Vec<f64>,
    pub light_net_assimilation: Vec<f64>,
}
pub struct Model {
    metadata: ModelMetadata,
}
pub struct BigLeafModel {
    metadata: ModelMetadata,
}
macro_rules! model_impl {
    ($model:ident,$input:ty,$manifest:literal,$evaluate:ident,$factors:expr) => {
        impl $model {
            pub fn new() -> Result<Self, ModelError> {
                Ok(Self {
                    metadata: parse_model_manifest(include_str!($manifest))?,
                })
            }
            pub fn simulate(&self, p: &$input) -> Result<Output, ModelError> {
                p.validate()?;
                let (capacity, area, par) = ($factors)(p);
                solver::simulate(&p.parameters, par, capacity, area)
            }
        }
        impl PhysicsModel for $model {
            type Input = $input;
            type Output = Output;
            fn metadata(&self) -> ModelMetadata {
                self.metadata.clone()
            }
            fn validate_input(&self, p: &$input) -> Result<(), ModelError> {
                p.validate()
            }
            fn run(&self, p: &$input) -> Result<Output, ModelError> {
                self.simulate(p)
            }
            fn validate_output(&self, o: &Output) -> Result<(), ModelError> {
                solver::validate_output(o)
            }
        }
    };
}
model_impl!(Model, Input, "../model.toml", evaluate_leaf, |p: &Input| (
    1.,
    1.,
    p.absorbed_par_umol_m2_s
));
model_impl!(
    BigLeafModel,
    BigLeafInput,
    "../model-big-leaf.toml",
    evaluate_big_leaf,
    |p: &BigLeafInput| (
        solver::capacity_factor(p.leaf_area_index, p.nitrogen_extinction_coefficient),
        p.leaf_area_index,
        p.absorbed_par_ground_umol_m2_s
    )
);
