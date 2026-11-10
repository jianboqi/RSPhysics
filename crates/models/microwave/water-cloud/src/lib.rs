//! Independent implementation of published water-cloud equations. No upstream code linked.
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DescriptorMode {
    #[default]
    WaterContent,
    Independent,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    /// water_content uses V1=V2=VWC; independent uses the two supplied descriptors.
    #[schemars(with="String",extend("enum"=["water_content", "independent"]))]
    pub descriptor_mode: DescriptorMode,
    /// V1 for vegetation scattering; coefficient A must match its units.
    #[schemars(range(min=0.0,max=100.0),extend("x-unit"="1"))]
    pub scattering_descriptor: f64,
    /// V2 for attenuation; coefficient B must match its units.
    #[schemars(range(min=0.0,max=100.0),extend("x-unit"="1"))]
    pub attenuation_descriptor: f64,

    /// Canopy water mass per ground area; V1=V2=VWC in this parameterization.
    #[schemars(range(min=0.0,max=10.0),extend("x-unit"="kg m^-2"))]
    pub vegetation_water_content: f64,
    /// A for the chosen channel and VWC descriptor; requires external calibration.
    #[schemars(range(min=0.0,max=2.0),extend("x-unit"="m^2 kg^-1"))]
    pub scattering_coefficient: f64,
    /// B for the chosen channel and VWC descriptor; B*VWC is nadir optical depth.
    #[schemars(range(min=0.0,max=2.0),extend("x-unit"="m^2 kg^-1"))]
    pub attenuation_coefficient: f64,
    /// Bare-soil normalized radar cross section sigma0 in LINEAR POWER, never dB; supplied directly.
    #[schemars(range(min=0.0,max=10.0),extend("x-unit"="1"))]
    pub soil_backscatter: f64,
    /// Radar incidence angle from the surface normal.
    #[schemars(range(min=0.0,max=80.0),extend("x-unit"="deg"))]
    pub observation_zenith_deg: f64,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            descriptor_mode: DescriptorMode::WaterContent,
            scattering_descriptor: 1.,
            attenuation_descriptor: 1.,
            vegetation_water_content: 1.0,
            scattering_coefficient: 0.1,
            attenuation_coefficient: 0.2,
            soil_backscatter: 0.1,
            observation_zenith_deg: 40.0,
        }
    }
}
impl Input {
    pub fn validate(&self) -> Result<(), ModelError> {
        for (key, v, lo, hi) in [
            (
                "scattering_descriptor",
                self.scattering_descriptor,
                0.,
                100.,
            ),
            (
                "attenuation_descriptor",
                self.attenuation_descriptor,
                0.,
                100.,
            ),
            (
                "vegetation_water_content",
                self.vegetation_water_content,
                0.0,
                10.0,
            ),
            (
                "scattering_coefficient",
                self.scattering_coefficient,
                0.0,
                2.0,
            ),
            (
                "attenuation_coefficient",
                self.attenuation_coefficient,
                0.0,
                2.0,
            ),
            ("soil_backscatter", self.soil_backscatter, 0.0, 10.0),
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
    /// Total normalized radar cross section in linear power; may exceed one.
    pub sigma0: f64,
    /// Canopy backscatter in linear power.
    pub vegetation_backscatter: f64,
    /// Soil backscatter after two canopy passages, linear power.
    pub attenuated_soil_backscatter: f64,
    /// Two-way power transmissivity exp(-2*B*VWC/cos(theta)).
    pub two_way_transmissivity: f64,
}
pub fn evaluate(p: &Input) -> Result<Point, ModelError> {
    p.validate()?;
    Ok(point(p))
}
fn point(p: &Input) -> Point {
    let c = p.observation_zenith_deg.to_radians().cos();
    let (v1, v2) = if p.descriptor_mode == DescriptorMode::WaterContent {
        (p.vegetation_water_content, p.vegetation_water_content)
    } else {
        (p.scattering_descriptor, p.attenuation_descriptor)
    };
    let x = -2. * p.attenuation_coefficient * v2 / c;
    let transmission = x.exp();
    let vegetation = p.scattering_coefficient * v1 * c * (-x.exp_m1());
    let soil = p.soil_backscatter * transmission;
    Point {
        sigma0: vegetation + soil,
        vegetation_backscatter: vegetation,
        attenuated_soil_backscatter: soil,
        two_way_transmissivity: transmission,
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
                .sigma0
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
            out.point.sigma0,
            out.point.vegetation_backscatter,
            out.point.attenuated_soil_backscatter,
            out.point.two_way_transmissivity,
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
