//! Jacquemoud, Baret & Hanocq (1992) SOILSPECT, four-term phase function.
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use rsphysics_numerics::angular::gauss_unit;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{f64::consts::PI, sync::OnceLock};
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum Quadrature {
    #[default]
    Standard,
    Refined,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    #[schemars(range(min=0.,max=1.),extend("x-unit"="1"))]
    pub single_scattering_albedo: f64,
    #[schemars(range(min=0.,max=2.),extend("x-unit"="1"))]
    pub roughness_h: f64,
    #[schemars(range(min=-2.,max=2.),extend("x-unit"="1"))]
    pub phase_b: f64,
    #[schemars(range(min=-2.,max=2.),extend("x-unit"="1"))]
    pub phase_c: f64,
    #[schemars(range(min=-2.,max=2.),extend("x-unit"="1"))]
    pub phase_b_prime: f64,
    #[schemars(range(min=-2.,max=2.),extend("x-unit"="1"))]
    pub phase_c_prime: f64,
    #[schemars(range(min=0.,max=85.),extend("x-unit"="deg"))]
    pub solar_zenith_deg: f64,
    #[schemars(range(min=0.,max=85.),extend("x-unit"="deg"))]
    pub view_zenith_deg: f64,
    #[schemars(range(min=0.,max=180.),extend("x-unit"="deg"))]
    pub relative_azimuth_deg: f64,
    pub quadrature: Quadrature,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            single_scattering_albedo: 0.6,
            roughness_h: 0.1,
            phase_b: 0.3,
            phase_c: 0.1,
            phase_b_prime: 0.,
            phase_c_prime: 0.,
            solar_zenith_deg: 30.,
            view_zenith_deg: 20.,
            relative_azimuth_deg: 0.,
            quadrature: Quadrature::Standard,
        }
    }
}
impl Input {
    pub fn validate(&self) -> Result<(), ModelError> {
        for (k, v, l, h) in [
            (
                "single_scattering_albedo",
                self.single_scattering_albedo,
                0.,
                1.,
            ),
            ("roughness_h", self.roughness_h, 0., 2.),
            ("phase_b", self.phase_b, -2., 2.),
            ("phase_c", self.phase_c, -2., 2.),
            ("phase_b_prime", self.phase_b_prime, -2., 2.),
            ("phase_c_prime", self.phase_c_prime, -2., 2.),
            ("solar_zenith_deg", self.solar_zenith_deg, 0., 85.),
            ("view_zenith_deg", self.view_zenith_deg, 0., 85.),
            ("relative_azimuth_deg", self.relative_azimuth_deg, 0., 180.),
        ] {
            if !v.is_finite() || v < l || v > h {
                return Err(ModelError::new(
                    ErrorKind::OutOfDomain,
                    format!("{k} must be finite and in [{l},{h}]"),
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
    pub single_scattering_brf: f64,
    pub multiple_scattering_brf: f64,
    pub phase_function: f64,
    pub hotspot_factor: f64,
}
fn point_mu(p: &Input, ci: f64, ce: f64, phi: f64) -> Point {
    let product = ((1. - ci * ci) * (1. - ce * ce)).max(0.).sqrt() * phi.cos();
    let g = (ci * ce + product).clamp(-1., 1.);
    let gp = (ci * ce - product).clamp(-1., 1.);
    let phase_function = 1.
        + p.phase_b * g
        + 0.5 * p.phase_c * (3. * g * g - 1.)
        + p.phase_b_prime * gp
        + 0.5 * p.phase_c_prime * (3. * gp * gp - 1.);
    let bg = if p.roughness_h == 0. {
        if g == 1. { 1. } else { 0. }
    } else if g == -1. {
        0.
    } else {
        p.roughness_h / (p.roughness_h + ((1. - g) / (1. + g)).sqrt())
    };
    let w = p.single_scattering_albedo;
    let q = 2. * (1. - w).sqrt();
    let hi = (1. + 2. * ci) / (1. + q * ci);
    let he = (1. + 2. * ce) / (1. + q * ce);
    let scale = w / (4. * (ci + ce));
    let hotspot_factor = 1. + bg;
    let single_scattering_brf = scale * hotspot_factor * phase_function;
    let multiple_scattering_brf = scale * (hi * he - 1.);
    let brf = single_scattering_brf + multiple_scattering_brf;
    Point {
        brf,
        brdf: brf / PI,
        single_scattering_brf,
        multiple_scattering_brf,
        phase_function,
        hotspot_factor,
    }
}
pub fn evaluate(p: &Input) -> Result<Point, ModelError> {
    p.validate()?;
    Ok(point_mu(
        p,
        p.solar_zenith_deg.to_radians().cos(),
        p.view_zenith_deg.to_radians().cos(),
        p.relative_azimuth_deg.to_radians(),
    ))
}
fn nodes(refined: bool) -> &'static Vec<(f64, f64)> {
    static STANDARD: OnceLock<Vec<(f64, f64)>> = OnceLock::new();
    static FINE: OnceLock<Vec<(f64, f64)>> = OnceLock::new();
    if refined {
        FINE.get_or_init(|| gauss_unit(64))
    } else {
        STANDARD.get_or_init(|| gauss_unit(32))
    }
}
fn azimuth(refined: bool) -> &'static Vec<(f64, f64)> {
    static STANDARD: OnceLock<Vec<(f64, f64)>> = OnceLock::new();
    static FINE: OnceLock<Vec<(f64, f64)>> = OnceLock::new();
    let make = |n| gauss_unit(n).iter().map(|&(a, w)| (PI * a, w)).collect();
    if refined {
        FINE.get_or_init(|| make(128))
    } else {
        STANDARD.get_or_init(|| make(64))
    }
}
/// Full BRF/pi flux integral; includes hotspot and all four phase coefficients.
fn directional(p: &Input, mu: f64, refined: bool) -> f64 {
    2. * nodes(refined)
        .iter()
        .map(|&(v, w)| {
            w * v
                * azimuth(refined)
                    .iter()
                    .map(|&(a, wa)| wa * point_mu(p, mu, v, a).brf)
                    .sum::<f64>()
        })
        .sum::<f64>()
}
fn hemispherical(p: &Input, refined: bool) -> f64 {
    2. * nodes(refined)
        .iter()
        .map(|&(u, w)| w * u * directional(p, u, refined))
        .sum::<f64>()
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    #[serde(flatten)]
    pub point: Point,
    pub directional_hemispherical_reflectance: f64,
    pub bihemispherical_reflectance: f64,
    pub integration_error_estimate: f64,
    pub principal_view_zenith_deg: Vec<f64>,
    pub principal_brf: Vec<f64>,
    pub hemisphere_view_zenith_deg: Vec<f64>,
    pub hemisphere_azimuth_deg: Vec<f64>,
    pub hemisphere_brf: Vec<f64>,
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
    pub fn simulate(&self, p: &Input) -> Result<Output, ModelError> {
        p.validate()?;
        let selected = evaluate(p)?;
        let ci = p.solar_zenith_deg.to_radians().cos();
        // Always compare both orders; return selected order and a convergence diagnostic.
        let coarse_d = directional(p, ci, false);
        let fine_d = directional(p, ci, true);
        let coarse_b = hemispherical(p, false);
        let fine_b = hemispherical(p, true);
        let fine = p.quadrature == Quadrature::Refined;
        let mut o = Output {
            point: selected,
            directional_hemispherical_reflectance: if fine { fine_d } else { coarse_d },
            bihemispherical_reflectance: if fine { fine_b } else { coarse_b },
            integration_error_estimate: (fine_d - coarse_d).abs().max((fine_b - coarse_b).abs()),
            principal_view_zenith_deg: vec![],
            principal_brf: vec![],
            hemisphere_view_zenith_deg: vec![],
            hemisphere_azimuth_deg: vec![],
            hemisphere_brf: vec![],
            negative_sample_count: usize::from(selected.brf < 0.),
        };
        for z in -85_i32..=85 {
            let q = Input {
                view_zenith_deg: f64::from(z.abs()),
                relative_azimuth_deg: if z < 0 { 180. } else { 0. },
                ..*p
            };
            let brf = evaluate(&q)?.brf;
            o.principal_view_zenith_deg.push(f64::from(z));
            o.principal_brf.push(brf);
            o.negative_sample_count += usize::from(brf < 0.);
        }
        for z in (0..=85).step_by(5) {
            for a in (0..360).step_by(10) {
                let brf = evaluate(&Input {
                    view_zenith_deg: f64::from(z),
                    relative_azimuth_deg: f64::from(a.min(360 - a)),
                    ..*p
                })?
                .brf;
                o.hemisphere_view_zenith_deg.push(f64::from(z));
                o.hemisphere_azimuth_deg.push(f64::from(a));
                o.hemisphere_brf.push(brf);
                o.negative_sample_count += usize::from(brf < 0.);
            }
        }
        self.validate_output(&o)?;
        Ok(o)
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
        if o.principal_brf.len() != 171
            || o.hemisphere_brf.len() != 648
            || o.principal_brf
                .iter()
                .chain(&o.hemisphere_brf)
                .chain([
                    &o.point.brf,
                    &o.point.brdf,
                    &o.directional_hemispherical_reflectance,
                    &o.bihemispherical_reflectance,
                    &o.integration_error_estimate,
                ])
                .any(|v| !v.is_finite())
        {
            return Err(ModelError::new(
                ErrorKind::Internal,
                "invalid SOILSPECT response",
            ));
        }
        Ok(())
    }
}
