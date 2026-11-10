//! Independent implementation of the published MODIS/JRC Ross-Li equations.
//! Fixed RossThick + LiSparseReciprocal, h/b=2, b/r=1, without hotspot enhancement.
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    /// Isotropic weight for one user-selected spectral band, not a spectral model.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub f_iso: f64,
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub f_vol: f64,
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub f_geo: f64,
    #[schemars(range(min=0.0,max=85.0),extend("x-unit"="deg"))]
    pub solar_zenith_deg: f64,
    #[schemars(range(min=0.0,max=85.0),extend("x-unit"="deg"))]
    pub view_zenith_deg: f64,
    /// Unsigned sun-view azimuth separation; zero is same azimuth/backscatter.
    #[schemars(range(min=0.0,max=180.0),extend("x-unit"="deg"))]
    pub relative_azimuth_deg: f64,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            f_iso: 0.2,
            f_vol: 0.1,
            f_geo: 0.02,
            solar_zenith_deg: 30.,
            view_zenith_deg: 20.,
            relative_azimuth_deg: 0.,
        }
    }
}
impl Input {
    pub fn validate(&self) -> Result<(), ModelError> {
        for (key, value, max) in [
            ("f_iso", self.f_iso, 1.),
            ("f_vol", self.f_vol, 1.),
            ("f_geo", self.f_geo, 1.),
            ("solar_zenith_deg", self.solar_zenith_deg, 85.),
            ("view_zenith_deg", self.view_zenith_deg, 85.),
            ("relative_azimuth_deg", self.relative_azimuth_deg, 180.),
        ] {
            if !value.is_finite() || !(0.0..=max).contains(&value) {
                return Err(ModelError::new(
                    ErrorKind::OutOfDomain,
                    format!("{key} must be finite and in [0, {max}]"),
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
    pub k_iso: f64,
    pub k_vol: f64,
    pub k_geo: f64,
}
/// Algebraic kernel response. Negative BRF indicates incompatible weights;
/// it is returned and diagnosed, not silently clipped. BRF above one is allowed.
pub fn evaluate(input: &Input) -> Result<Point, ModelError> {
    input.validate()?;
    Ok(point(input))
}
fn point(p: &Input) -> Point {
    let s = p.solar_zenith_deg.to_radians();
    let v = p.view_zenith_deg.to_radians();
    let phi = p.relative_azimuth_deg.to_radians();
    let cos_phase = (s.cos() * v.cos() + s.sin() * v.sin() * phi.cos()).clamp(-1., 1.);
    let phase = cos_phase.acos();
    let k_vol = ((FRAC_PI_2 - phase) * cos_phase + phase.sin()) / (s.cos() + v.cos()) - FRAC_PI_4;
    // b/r=1 makes primed angles equal to unprimed angles.
    let ts = s.tan();
    let tv = v.tan();
    let sec_sum = 1. / s.cos() + 1. / v.cos();
    // Equivalent distance identity avoids cancellation near the exact hotspot.
    let distance2 = (ts - tv).powi(2) + 4. * ts * tv * (phi / 2.).sin().powi(2);
    let cos_t = (2. * (distance2 + (ts * tv * phi.sin()).powi(2)).sqrt() / sec_sum).clamp(-1., 1.);
    let t = cos_t.acos();
    let overlap = (t - t.sin() * cos_t) * sec_sum / PI;
    let k_geo = overlap - sec_sum + 0.5 * (1. + cos_phase) / (s.cos() * v.cos());
    let brf = p.f_iso + p.f_vol * k_vol + p.f_geo * k_geo;
    Point {
        brf,
        brdf: brf / PI,
        k_iso: 1.,
        k_vol,
        k_geo,
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
                out.point.k_iso,
                out.point.k_vol,
                out.point.k_geo,
            ]
            .iter()
            .any(|v| !v.is_finite())
        {
            return Err(ModelError::new(
                ErrorKind::Internal,
                "invalid Ross-Li angular response",
            ));
        }
        Ok(())
    }
}
