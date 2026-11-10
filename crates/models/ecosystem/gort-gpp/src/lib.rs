//! Adaptation of supplied research MATLAB source; attribution and scope in LICENSE-ORIGIN.md.
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
mod solver;
fn domain(s: impl Into<String>) -> ModelError {
    ModelError::new(ErrorKind::OutOfDomain, s)
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    /// Total one-sided leaf area per ground area.
    #[schemars(range(min=0.5,max=10.0),extend("x-unit"="m^2 m^-2"))]
    pub leaf_area_index: f64,
    /// Horizontal crown radius.
    #[schemars(range(min=0.1,max=10.0),extend("x-unit"="m"))]
    pub crown_radius_m: f64,
    /// Crown number density per ground area.
    #[schemars(range(min=0.001,max=2.0),extend("x-unit"="m^-2"))]
    pub crown_density_m2: f64,
    /// Lower crown-height boundary h1.
    #[schemars(range(min=0.0,max=60.0),extend("x-unit"="m"))]
    pub crown_base_height_m: f64,
    /// Upper crown-height boundary h2; must exceed h1.
    #[schemars(range(min=0.1,max=80.0),extend("x-unit"="m"))]
    pub crown_top_height_m: f64,
    /// Crown vertical/horizontal axis ratio.
    #[schemars(range(min=0.1,max=5.0),extend("x-unit"="1"))]
    pub crown_axis_ratio: f64,
    /// Solar zenith; positive lower bound avoids original one-based zero-index failure.
    #[schemars(range(min=1.0,max=80.0),extend("x-unit"="deg"))]
    pub solar_zenith_deg: f64,
    /// Direct incident PAR per ground area.
    #[schemars(range(min=0.0,max=2500.0),extend("x-unit"="umol m^-2 s^-1"))]
    pub direct_par_umol_m2_s: f64,
    /// Diffuse incident PAR per ground area.
    #[schemars(range(min=0.0,max=2500.0),extend("x-unit"="umol m^-2 s^-1"))]
    pub diffuse_par_umol_m2_s: f64,
    /// Air/leaf temperature used by the selected photosynthetic path.
    #[schemars(range(min=5.0,max=40.0),extend("x-unit"="degC"))]
    pub air_temperature_c: f64,
    /// Vapour pressure deficit; cannot exceed saturation vapour pressure at the supplied air temperature.
    #[schemars(range(min=0.0,max=8000.0),extend("x-unit"="Pa"))]
    pub vapour_pressure_deficit_pa: f64,
    /// Ambient atmospheric carbon dioxide concentration.
    #[schemars(range(min=200.0,max=1200.0),extend("x-unit"="umol mol^-1"))]
    pub co2_ppm: f64,
    /// Canopy-top weight w1 for young-leaf Vcmax, not whole-canopy leaf-age fraction.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub young_leaf_fraction: f64,
    /// Young-leaf Vcmax at 25 degrees C.
    #[schemars(range(min=1.0,max=200.0),extend("x-unit"="umol m^-2 s^-1"))]
    pub vcmax25_young: f64,
    /// Mature-leaf Vcmax at 25 degrees C.
    #[schemars(range(min=1.0,max=200.0),extend("x-unit"="umol m^-2 s^-1"))]
    pub vcmax25_mature: f64,
    /// Ambient pressure.
    #[schemars(range(min=50000.0,max=110000.0),extend("x-unit"="Pa"))]
    pub pressure_pa: f64,
    /// Ball-Berry slope m.
    #[schemars(range(min=1.0,max=20.0),extend("x-unit"="1"))]
    pub stomatal_slope: f64,
    /// Photon/electron ratio ppe used in electron transport.
    #[schemars(range(min=1.0,max=5.0),extend("x-unit"="1"))]
    pub electron_photon_ratio: f64,
    /// Campbell leaf-angle parameter x.
    #[schemars(range(min=0.1,max=5.0),extend("x-unit"="1"))]
    pub leaf_angle_parameter: f64,
    /// PAR single-scattering albedo Omega.
    #[schemars(range(min=0.0,max=0.9),extend("x-unit"="1"))]
    pub leaf_single_scattering_albedo: f64,
    /// Background PAR reflectance.
    #[schemars(range(min=0.0,max=0.9),extend("x-unit"="1"))]
    pub background_reflectance: f64,
    /// Minimum stomatal conductance g0.
    #[schemars(range(min=0.001,max=0.1),extend("x-unit"="mol m^-2 s^-1"))]
    pub minimum_stomatal_conductance: f64,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            leaf_area_index: 4.31,
            crown_radius_m: 1.7,
            crown_density_m2: 0.1905,
            crown_base_height_m: 7.8,
            crown_top_height_m: 14.1,
            crown_axis_ratio: 1.17,
            solar_zenith_deg: 40.0,
            direct_par_umol_m2_s: 1200.0,
            diffuse_par_umol_m2_s: 200.0,
            air_temperature_c: 25.0,
            vapour_pressure_deficit_pa: 1000.0,
            co2_ppm: 400.0,
            young_leaf_fraction: 0.4,
            vcmax25_young: 52.1,
            vcmax25_mature: 34.17,
            pressure_pa: 101325.0,
            stomatal_slope: 8.0,
            electron_photon_ratio: 2.6,
            leaf_angle_parameter: 0.5,
            leaf_single_scattering_albedo: 0.1,
            background_reflectance: 0.1,
            minimum_stomatal_conductance: 0.01,
        }
    }
}
impl Input {
    pub fn validate(&self) -> Result<(), ModelError> {
        for (name, v, lo, hi) in [
            ("leaf_area_index", self.leaf_area_index, 0.5, 10.0),
            ("crown_radius_m", self.crown_radius_m, 0.1, 10.0),
            ("crown_density_m2", self.crown_density_m2, 0.001, 2.0),
            ("crown_base_height_m", self.crown_base_height_m, 0.0, 60.0),
            ("crown_top_height_m", self.crown_top_height_m, 0.1, 80.0),
            ("crown_axis_ratio", self.crown_axis_ratio, 0.1, 5.0),
            ("solar_zenith_deg", self.solar_zenith_deg, 1.0, 80.0),
            (
                "direct_par_umol_m2_s",
                self.direct_par_umol_m2_s,
                0.0,
                2500.0,
            ),
            (
                "diffuse_par_umol_m2_s",
                self.diffuse_par_umol_m2_s,
                0.0,
                2500.0,
            ),
            ("air_temperature_c", self.air_temperature_c, 5.0, 40.0),
            (
                "vapour_pressure_deficit_pa",
                self.vapour_pressure_deficit_pa,
                0.0,
                8000.0,
            ),
            ("co2_ppm", self.co2_ppm, 200.0, 1200.0),
            ("young_leaf_fraction", self.young_leaf_fraction, 0.0, 1.0),
            ("vcmax25_young", self.vcmax25_young, 1.0, 200.0),
            ("vcmax25_mature", self.vcmax25_mature, 1.0, 200.0),
            ("pressure_pa", self.pressure_pa, 50000.0, 110000.0),
            ("stomatal_slope", self.stomatal_slope, 1.0, 20.0),
            (
                "electron_photon_ratio",
                self.electron_photon_ratio,
                1.0,
                5.0,
            ),
            ("leaf_angle_parameter", self.leaf_angle_parameter, 0.1, 5.0),
            (
                "leaf_single_scattering_albedo",
                self.leaf_single_scattering_albedo,
                0.0,
                0.9,
            ),
            (
                "background_reflectance",
                self.background_reflectance,
                0.0,
                0.9,
            ),
            (
                "minimum_stomatal_conductance",
                self.minimum_stomatal_conductance,
                0.001,
                0.1,
            ),
        ] {
            if !v.is_finite() || !(lo..=hi).contains(&v) {
                return Err(domain(format!("{name} must be finite in [{lo},{hi}]")));
            }
        }
        if self.crown_top_height_m <= self.crown_base_height_m {
            return Err(domain("crown_top_height_m must exceed crown_base_height_m"));
        }
        let svp = 610.7 * (17.38 * self.air_temperature_c / (239. + self.air_temperature_c)).exp();
        if self.vapour_pressure_deficit_pa > svp {
            return Err(domain(
                "vapour pressure deficit exceeds saturation pressure",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    /// Canopy net assimilation per ground area; may be negative.
    pub net_assimilation_umol_m2_s: f64,
    /// Sunlit net assimilation per sunlit leaf area.
    pub sunlit_assimilation_umol_m2_s: f64,
    /// Shaded net assimilation per shaded leaf area.
    pub shaded_assimilation_umol_m2_s: f64,
    /// Sunlit leaf area per ground area.
    pub sunlit_leaf_area_index: f64,
    /// Shaded leaf area per ground area.
    pub shaded_leaf_area_index: f64,
    /// Absorbed PAR per sunlit leaf area using original scaling.
    pub sunlit_absorbed_par: f64,
    /// Absorbed PAR per shaded leaf area.
    pub shaded_absorbed_par: f64,
    /// Integrated between-crown openness using original quadrature.
    pub between_crown_openness: f64,
    /// Integrated within-crown openness using original quadrature.
    pub within_crown_openness: f64,
    /// Absorbed PAR at the background per ground area.
    pub floor_absorbed_par: f64,
    /// Original gap sampling: 0..89 degrees.
    pub gap_zenith_deg: Vec<f64>,
    /// Between-crown gap probability at gap angles.
    pub between_crown_gaps: Vec<f64>,
    /// Within-crown gap probability at gap angles.
    pub within_crown_gaps: Vec<f64>,
}
pub struct Model {
    metadata: ModelMetadata,
}
impl Model {
    pub fn new() -> Result<Self, ModelError> {
        Ok(Self {
            metadata: parse_model_manifest(include_str!("../model.toml"))?,
        })
    }
    pub fn simulate(&self, p: &Input) -> Result<Output, ModelError> {
        p.validate()?;
        let out = solver::simulate(p)?;
        self.validate_output(&out)?;
        Ok(out)
    }
}
impl PhysicsModel for Model {
    type Input = Input;
    type Output = Output;
    fn metadata(&self) -> ModelMetadata {
        self.metadata.clone()
    }
    fn validate_input(&self, p: &Input) -> Result<(), ModelError> {
        p.validate()
    }
    fn run(&self, p: &Input) -> Result<Output, ModelError> {
        self.simulate(p)
    }
    fn validate_output(&self, o: &Output) -> Result<(), ModelError> {
        let scalars = [
            o.net_assimilation_umol_m2_s,
            o.sunlit_assimilation_umol_m2_s,
            o.shaded_assimilation_umol_m2_s,
            o.sunlit_leaf_area_index,
            o.shaded_leaf_area_index,
            o.sunlit_absorbed_par,
            o.shaded_absorbed_par,
            o.between_crown_openness,
            o.within_crown_openness,
            o.floor_absorbed_par,
        ];
        if scalars.iter().any(|x| !x.is_finite())
            || o.gap_zenith_deg.len() != 90
            || o.gap_zenith_deg.iter().any(|x| !x.is_finite())
            || o.between_crown_gaps.len() != 90
            || o.between_crown_gaps.iter().any(|x| !x.is_finite())
            || o.within_crown_gaps.len() != 90
            || o.within_crown_gaps.iter().any(|x| !x.is_finite())
        {
            return Err(ModelError::new(
                ErrorKind::NonConvergence,
                "nonfinite model output or invalid shape",
            ));
        }
        Ok(())
    }
}
