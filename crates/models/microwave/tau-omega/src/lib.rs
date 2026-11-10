//! Independent implementation of published tau-omega equations. No upstream code linked.
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    /// Effective soil temperature, not brightness temperature.
    #[schemars(range(min=150.0,max=350.0),extend("x-unit"="K"))]
    pub soil_temperature_k: f64,
    /// Canopy physical temperature.
    #[schemars(range(min=150.0,max=350.0),extend("x-unit"="K"))]
    pub vegetation_temperature_k: f64,
    /// Effective soil power reflectivity at the selected frequency, polarization and angle; supplied directly.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub soil_reflectivity: f64,
    /// Nadir vegetation optical depth tau, before division by cos(theta).
    #[schemars(range(min=0.0,max=5.0),extend("x-unit"="1"))]
    pub vegetation_optical_depth: f64,
    /// Vegetation single scattering albedo omega; zero-order loss parameter, no multiple scattering.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub single_scattering_albedo: f64,
    /// Observation zenith angle; zero is nadir.
    #[schemars(range(min=0.0,max=80.0),extend("x-unit"="deg"))]
    pub observation_zenith_deg: f64,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            soil_temperature_k: 300.0,
            vegetation_temperature_k: 295.0,
            soil_reflectivity: 0.3,
            vegetation_optical_depth: 0.3,
            single_scattering_albedo: 0.05,
            observation_zenith_deg: 40.0,
        }
    }
}
impl Input {
    pub fn validate(&self) -> Result<(), ModelError> {
        for (key, v, lo, hi) in [
            ("soil_temperature_k", self.soil_temperature_k, 150.0, 350.0),
            (
                "vegetation_temperature_k",
                self.vegetation_temperature_k,
                150.0,
                350.0,
            ),
            ("soil_reflectivity", self.soil_reflectivity, 0.0, 1.0),
            (
                "vegetation_optical_depth",
                self.vegetation_optical_depth,
                0.0,
                5.0,
            ),
            (
                "single_scattering_albedo",
                self.single_scattering_albedo,
                0.0,
                1.0,
            ),
            (
                "observation_zenith_deg",
                self.observation_zenith_deg,
                0.0,
                80.0,
            ),
        ] {
            if !v.is_finite() || !(lo..=hi).contains(&v) {
                return Err(ModelError::new(
                    ErrorKind::OutOfDomain,
                    format!("{key} must be finite in [{lo}, {hi}]"),
                ));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Point {
    /// Total surface brightness temperature; no sky or atmosphere.
    pub brightness_temperature_k: f64,
    /// Soil emission attenuated by one canopy passage.
    pub soil_emission_k: f64,
    /// Direct upward canopy emission.
    pub vegetation_emission_k: f64,
    /// Downward canopy emission reflected by soil and attenuated upward.
    pub reflected_vegetation_emission_k: f64,
    /// One-way canopy power transmissivity gamma.
    pub transmissivity: f64,
}
pub fn evaluate(p: &Input) -> Result<Point, ModelError> {
    p.validate()?;
    Ok(point(p))
}
fn point(p: &Input) -> Point {
    let c = p.observation_zenith_deg.to_radians().cos();
    let x = -p.vegetation_optical_depth / c;
    let gamma = x.exp();
    // exp_m1 retains weak-canopy emission when tau is close to zero.
    let loss = -x.exp_m1();
    let soil = p.soil_temperature_k * (1. - p.soil_reflectivity) * gamma;
    let vegetation = p.vegetation_temperature_k * (1. - p.single_scattering_albedo) * loss;
    let reflected = vegetation * p.soil_reflectivity * gamma;
    Point {
        brightness_temperature_k: soil + vegetation + reflected,
        soil_emission_k: soil,
        vegetation_emission_k: vegetation,
        reflected_vegetation_emission_k: reflected,
        transmissivity: gamma,
    }
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    #[serde(flatten)]
    pub point: Point,
    /// Fixed sensitivity experiment: zenith 0..80 degrees step1.
    pub sweep_zenith_deg: Vec<f64>,
    /// All non-angle inputs held fixed; not a full channel angular model.
    pub sweep_total: Vec<f64>,
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
        let angles: Vec<_> = (0..=80).map(f64::from).collect();
        let totals = angles
            .iter()
            .map(|&angle| {
                point(&Input {
                    observation_zenith_deg: angle,
                    ..*p
                })
                .brightness_temperature_k
            })
            .collect();
        Ok(Output {
            point: point(p),
            sweep_zenith_deg: angles,
            sweep_total: totals,
        })
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
    fn validate_output(&self, out: &Output) -> Result<(), ModelError> {
        if [
            out.point.brightness_temperature_k,
            out.point.soil_emission_k,
            out.point.vegetation_emission_k,
            out.point.reflected_vegetation_emission_k,
            out.point.transmissivity,
        ]
        .iter()
        .any(|v| !v.is_finite() || *v < 0.)
            || out.sweep_zenith_deg.len() != 81
            || out.sweep_total.len() != 81
            || out
                .sweep_zenith_deg
                .iter()
                .chain(&out.sweep_total)
                .any(|v| !v.is_finite() || *v < 0.)
        {
            return Err(ModelError::new(
                ErrorKind::Internal,
                "invalid microwave output",
            ));
        }
        Ok(())
    }
}
