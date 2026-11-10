//! RPV surface BRDF following Rahman et al. (1993) and JRC RAMI.
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum HotspotMode {
    #[default]
    OriginalThreeParameter,
    IndependentFourParameter,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    #[schemars(range(min=0.,max=1.),extend("x-unit"="1"))]
    pub rho0: f64,
    #[schemars(range(min=0.,max=2.),extend("x-unit"="1"))]
    pub minnaert_k: f64,
    #[schemars(range(min=-0.95,max=0.95),extend("x-unit"="1"))]
    pub phase_asymmetry: f64,
    pub hotspot_mode: HotspotMode,
    #[schemars(range(min=0.,max=1.),extend("x-unit"="1"))]
    pub hotspot_rho: f64,
    #[schemars(range(min=0.,max=85.),extend("x-unit"="deg"))]
    pub solar_zenith_deg: f64,
    #[schemars(range(min=0.,max=85.),extend("x-unit"="deg"))]
    pub view_zenith_deg: f64,
    #[schemars(range(min=0.,max=180.),extend("x-unit"="deg"))]
    pub relative_azimuth_deg: f64,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            rho0: 0.2,
            minnaert_k: 0.7,
            phase_asymmetry: -0.2,
            hotspot_mode: HotspotMode::OriginalThreeParameter,
            hotspot_rho: 0.2,
            solar_zenith_deg: 30.,
            view_zenith_deg: 20.,
            relative_azimuth_deg: 0.,
        }
    }
}
impl Input {
    pub fn validate(&self) -> Result<(), ModelError> {
        for (key, v, lo, hi) in [
            ("rho0", self.rho0, 0., 1.),
            ("minnaert_k", self.minnaert_k, 0., 2.),
            ("phase_asymmetry", self.phase_asymmetry, -0.95, 0.95),
            ("hotspot_rho", self.hotspot_rho, 0., 1.),
            ("solar_zenith_deg", self.solar_zenith_deg, 0., 85.),
            ("view_zenith_deg", self.view_zenith_deg, 0., 85.),
            ("relative_azimuth_deg", self.relative_azimuth_deg, 0., 180.),
        ] {
            if !v.is_finite() || v < lo || v > hi {
                return Err(ModelError::new(
                    ErrorKind::OutOfDomain,
                    format!("{key} must be finite and in [{lo},{hi}]"),
                ));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub brf: f64,
    pub brdf: f64,
    pub minnaert_factor: f64,
    pub phase_function: f64,
    pub hotspot_factor: f64,
}
pub fn evaluate(p: &Input) -> Result<Point, ModelError> {
    p.validate()?;
    Ok(point(p))
}
fn point(p: &Input) -> Point {
    let s = p.solar_zenith_deg.to_radians();
    let v = p.view_zenith_deg.to_radians();
    let phi = p.relative_azimuth_deg.to_radians();
    let cg = (s.cos() * v.cos() + s.sin() * v.sin() * phi.cos()).clamp(-1., 1.);
    let t = s.tan();
    let u = v.tan();
    let g = ((t - u).powi(2) + 4. * t * u * (phi / 2.).sin().powi(2)).sqrt();
    let minnaert_factor = (s.cos() * v.cos() * (s.cos() + v.cos())).powf(p.minnaert_k - 1.);
    let a = p.phase_asymmetry;
    let phase_function = (1. - a * a) / (1. + a * a + 2. * a * cg).powf(1.5);
    let rho = if p.hotspot_mode == HotspotMode::OriginalThreeParameter {
        p.rho0
    } else {
        p.hotspot_rho
    };
    let hotspot_factor = 1. + (1. - rho) / (1. + g);
    let brf = p.rho0 * minnaert_factor * phase_function * hotspot_factor;
    Point {
        brf,
        brdf: brf / PI,
        minnaert_factor,
        phase_function,
        hotspot_factor,
    }
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    #[serde(flatten)]
    pub point: Point,
    /// Signed view angle: negative=phi180 forward; positive=phi0 backward.
    pub principal_view_zenith_deg: Vec<f64>,
    pub principal_brf: Vec<f64>,
    /// Flattened zenith-major grid: 0..85 step5, relative azimuth 0..350 step10.
    pub hemisphere_view_zenith_deg: Vec<f64>,
    pub hemisphere_azimuth_deg: Vec<f64>,
    pub hemisphere_brf: Vec<f64>,
    /// Number of negative algebraic BRF samples, selected point + both grids.
    pub negative_sample_count: usize,
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
    pub fn simulate(&self, input: &Input) -> Result<Output, ModelError> {
        input.validate()?;
        let selected = point(input);
        let mut negative = usize::from(selected.brf < 0.);
        let mut out = Output {
            point: selected,
            principal_view_zenith_deg: Vec::with_capacity(171),
            principal_brf: Vec::with_capacity(171),
            hemisphere_view_zenith_deg: Vec::with_capacity(648),
            hemisphere_azimuth_deg: Vec::with_capacity(648),
            hemisphere_brf: Vec::with_capacity(648),
            negative_sample_count: 0,
        };
        for angle in -85_i32..=85 {
            let p = Input {
                view_zenith_deg: f64::from(angle.abs()),
                relative_azimuth_deg: if angle < 0 { 180. } else { 0. },
                ..*input
            };
            let value = point(&p).brf;
            out.principal_view_zenith_deg.push(f64::from(angle));
            out.principal_brf.push(value);
            negative += usize::from(value < 0.);
        }
        for zenith in (0..=85).step_by(5) {
            for azimuth in (0..360).step_by(10) {
                let p = Input {
                    view_zenith_deg: f64::from(zenith),
                    relative_azimuth_deg: f64::from(azimuth.min(360 - azimuth)),
                    ..*input
                };
                let value = point(&p).brf;
                out.hemisphere_view_zenith_deg.push(f64::from(zenith));
                out.hemisphere_azimuth_deg.push(f64::from(azimuth));
                out.hemisphere_brf.push(value);
                negative += usize::from(value < 0.);
            }
        }
        out.negative_sample_count = negative;
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
    fn validate_output(&self, out: &Output) -> Result<(), ModelError> {
        let arrays = [
            &out.principal_view_zenith_deg,
            &out.principal_brf,
            &out.hemisphere_view_zenith_deg,
            &out.hemisphere_azimuth_deg,
            &out.hemisphere_brf,
        ];
        if arrays[0].len() != 171
            || arrays[1].len() != 171
            || arrays[2..].iter().any(|a| a.len() != 648)
            || arrays.iter().any(|a| a.iter().any(|v| !v.is_finite()))
            || [
                out.point.brf,
                out.point.brdf,
                out.point.minnaert_factor,
                out.point.phase_function,
                out.point.hotspot_factor,
            ]
            .iter()
            .any(|v| !v.is_finite())
        {
            return Err(ModelError::new(
                ErrorKind::Internal,
                "invalid RPV angular response",
            ));
        }
        Ok(())
    }
}
