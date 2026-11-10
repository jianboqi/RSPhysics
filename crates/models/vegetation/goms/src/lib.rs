//! Li–Strahler geometric-optical mutual shadowing, including terrain and 1..10 bands.
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GeometryConvention {
    #[default]
    AreaConsistent,
    LegacyC,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    /// Crown number density multiplied by squared horizontal crown radius.
    #[schemars(range(min=0.0,max=2.0),extend("x-unit"="1"))]
    pub normalized_crown_density: f64,
    #[schemars(range(min=0.1,max=5.0),extend("x-unit"="1"))]
    pub crown_axis_ratio: f64,
    #[schemars(range(min=1.0,max=10.0),extend("x-unit"="1"))]
    pub crown_height_ratio: f64,
    #[schemars(range(min=0.0,max=5.0),extend("x-unit"="1"))]
    pub relative_height_variation: f64,
    #[schemars(length(min=1,max=10),extend("x-unit"="1"))]
    pub sunlit_ground_reflectance: Vec<f64>,
    #[schemars(length(min=1,max=10),extend("x-unit"="1"))]
    pub sunlit_crown_reflectance: Vec<f64>,
    /// Shared shaded-crown and shaded-ground reflectance for each band.
    #[schemars(length(min=1,max=10),extend("x-unit"="1"))]
    pub shaded_reflectance: Vec<f64>,
    #[schemars(range(min=0.0,max=85.0),extend("x-unit"="deg"))]
    pub solar_zenith_deg: f64,
    #[schemars(range(min=0.0,max=360.0),extend("x-unit"="deg"))]
    pub solar_azimuth_deg: f64,
    #[schemars(range(min=0.0,max=85.0),extend("x-unit"="deg"))]
    pub view_zenith_deg: f64,
    #[schemars(range(min=0.0,max=360.0),extend("x-unit"="deg"))]
    pub view_azimuth_deg: f64,
    #[schemars(range(min=0.0,max=60.0),extend("x-unit"="deg"))]
    pub slope_deg: f64,
    #[schemars(range(min=0.0,max=360.0),extend("x-unit"="deg"))]
    pub slope_aspect_deg: f64,
    #[schemars(with="String",extend("enum"=["area_consistent", "legacy_c"]))]
    pub geometry_convention: GeometryConvention,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            normalized_crown_density: 0.1,
            crown_axis_ratio: 1.733,
            crown_height_ratio: 2.577,
            relative_height_variation: 0.769,
            sunlit_ground_reflectance: vec![0.2],
            sunlit_crown_reflectance: vec![0.55],
            shaded_reflectance: vec![0.05],
            solar_zenith_deg: 45.,
            solar_azimuth_deg: 0.,
            view_zenith_deg: 30.,
            view_azimuth_deg: 0.,
            slope_deg: 0.,
            slope_aspect_deg: 0.,
            geometry_convention: GeometryConvention::default(),
        }
    }
}
fn domain(s: impl Into<String>) -> ModelError {
    ModelError::new(ErrorKind::OutOfDomain, s)
}
impl Input {
    pub fn validate(&self) -> Result<(), ModelError> {
        for (key, v, lo, hi) in [
            (
                "normalized_crown_density",
                self.normalized_crown_density,
                0.,
                2.,
            ),
            ("crown_axis_ratio", self.crown_axis_ratio, 0.1, 5.),
            ("crown_height_ratio", self.crown_height_ratio, 1., 10.),
            (
                "relative_height_variation",
                self.relative_height_variation,
                0.,
                5.,
            ),
            ("solar_zenith_deg", self.solar_zenith_deg, 0., 85.),
            ("solar_azimuth_deg", self.solar_azimuth_deg, 0., 360.),
            ("view_zenith_deg", self.view_zenith_deg, 0., 85.),
            ("view_azimuth_deg", self.view_azimuth_deg, 0., 360.),
            ("slope_deg", self.slope_deg, 0., 60.),
            ("slope_aspect_deg", self.slope_aspect_deg, 0., 360.),
        ] {
            if !v.is_finite() || !(lo..=hi).contains(&v) {
                return Err(domain(format!("{key} must be finite in [{lo}, {hi}]")));
            }
        }
        let nb = self.sunlit_ground_reflectance.len();
        for a in [
            &self.sunlit_ground_reflectance,
            &self.sunlit_crown_reflectance,
            &self.shaded_reflectance,
        ] {
            if !(1..=10).contains(&nb)
                || a.len() != nb
                || a.iter().any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
            {
                return Err(domain(
                    "three reflectance vectors must have equal length 1..10 and finite values in [0,1]",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Point {
    pub brf: Vec<f64>,
    pub brdf: Vec<f64>,
    pub sunlit_ground_weight: f64,
    pub sunlit_crown_weight: f64,
    pub shaded_weight: f64,
    pub invalid_weight_count: usize,
}
fn mean_shadow(t: f64) -> f64 {
    if t < 1e-5 {
        0.
    } else {
        1. - (1. - (-t).exp()) / t
    }
}
fn rotate(mut theta: f64, mut phi: f64, p: &Input, pi: f64, legacy: bool) -> (f64, f64) {
    let tilt = p.slope_deg * pi / 180.;
    if if legacy { tilt < 0.01 } else { tilt == 0. } {
        return (theta, phi);
    }
    phi -= p.slope_aspect_deg * pi / 180.;
    let x = phi.cos() * theta.sin() * tilt.cos() - theta.cos() * tilt.sin();
    let y = theta.sin() * phi.sin();
    let z = theta.sin() * phi.cos() * tilt.sin() + tilt.cos() * theta.cos();
    theta = if legacy {
        z.acos()
    } else {
        z.clamp(-1., 1.).acos()
    };
    phi = if legacy {
        let a = (y / (x * x + y * y).sqrt()).asin();
        if x < 0. {
            if y > 0. { pi - a } else { -pi - a }
        } else {
            a
        }
    } else {
        y.atan2(x)
    };
    (theta, phi)
}
#[allow(clippy::approx_constant)]
fn point(p: &Input) -> Option<Point> {
    let legacy = p.geometry_convention == GeometryConvention::LegacyC;
    let pi = if legacy { 3.1415926 } else { PI };
    let mut ti = p.crown_axis_ratio * (p.solar_zenith_deg * pi / 180.).tan();
    let mut tv = p.crown_axis_ratio * (p.view_zenith_deg * pi / 180.).tan();
    let (thetai, sunphi) = rotate(ti.atan(), p.solar_azimuth_deg * pi / 180., p, pi, legacy);
    let mut viewphi = p.view_azimuth_deg * pi / 180.;
    while viewphi > pi {
        viewphi -= 2. * pi;
    }
    while viewphi < -pi {
        viewphi += 2. * pi;
    }
    let (thetav, viewphi) = rotate(tv.atan(), viewphi, p, pi, legacy);
    if !thetai.is_finite()
        || !thetav.is_finite()
        || !sunphi.is_finite()
        || !viewphi.is_finite()
        || thetai.cos() <= 1e-10
        || thetav.cos() <= 1e-10
    {
        return None;
    }
    let mut phi = (viewphi - sunphi).abs();
    if !legacy {
        ti = thetai.tan();
        tv = thetav.tan();
        phi %= 2. * pi;
        if phi > pi {
            phi = 2. * pi - phi;
        }
    }
    let (si, ci) = thetai.sin_cos();
    let (sv, cv) = thetav.sin_cos();
    let (sp, cp) = phi.sin_cos();
    let secti = 1. / ci;
    let sectv = 1. / cv;
    let sum = secti + sectv;
    let lambda = p.normalized_crown_density;
    let gammai = pi * secti;
    let gammav = pi * sectv;
    let d = p.crown_axis_ratio * p.relative_height_variation * (thetai * 0.5).tan();
    let density = lambda * gammai;
    let beta = if density < 1e-5 {
        1.
    } else {
        (density / (density + d)) * ((1. - (-(density + d)).exp()) / (1. - (-density).exp()))
    };
    let distance2 = ti * ti + tv * tv - 2. * ti * tv * cp;
    let (x, sinpv, cospv, sinpi, cospi) = if distance2 > 1e-5 {
        let x = distance2.sqrt();
        (
            x,
            ti * sp / x,
            (ti * cp - tv) / x,
            tv * sp / x,
            (tv * cp - ti) / x,
        )
    } else {
        (0., 0., 1., 0., 1.)
    };
    let axes = (sinpv * sinpv + cospv * cospv * sectv * sectv)
        .max(0.)
        .sqrt()
        + (sinpi * sinpi + cospi * cospi * secti * secti)
            .max(0.)
            .sqrt();
    let cost = (p.crown_height_ratio * x / axes).max(0.);
    let t = if cost >= 1. { 0. } else { cost.acos() };
    // Area in the same units as pi*sec(theta), including the principal-plane terms.
    let overlap = (t - t.sin() * t.cos()) * sum / if legacy { pi } else { 1. };
    let gamma = pi * sum - overlap;
    let kg = (-lambda * gamma).exp();
    let principal_gamma = |distance: f64| {
        let cost = (p.crown_height_ratio * distance.abs() / sum).max(0.);
        let t = if cost >= 1. { 0. } else { cost.acos() };
        pi * sum - (t - (2. * t).sin() / 2.) * sum
    };
    let gamma0 = principal_gamma(ti - tv);
    let gamma180 = principal_gamma(ti + tv);
    let iv0 = ci * cv + si * sv;
    let iv180 = ci * cv - si * sv;
    let gammac0 = 0.5 * (1. + iv0) * gammav;
    let gammac180 = 0.5 * (1. + iv180) * gammav;
    let bigf0 = gammac0 / gamma0;
    let bigf180 = gammac180 / gamma180;
    let m0 = mean_shadow(lambda * gamma0);
    let m180 = mean_shadow(lambda * gamma180);
    let mi = mean_shadow(lambda * gammai).max(0.);
    let mv = mean_shadow(lambda * gammav);
    let theta_mi = 2. * mi.sqrt().asin();
    let pvm0 = mv - (1. - (thetav - thetai).cos()) / 2.;
    let pvm180 = mv - (1. - (-thetav - thetai).cos()) / 2.;
    let pimi0 = (1. - (theta_mi * (1. - (thetai - thetav) / pi)).cos()) / 2.;
    let pimi180 = (1. - (theta_mi * (1. - (thetai + thetav) / pi)).cos()) / 2.;
    let pm0 = pvm0.max(pimi0);
    let pm180 = pvm180.max(pimi180);
    let f0 = (1. - gammav * if thetav > thetai { pvm0 } else { pm0 } / gammac0) / (1. - m0);
    let f180 = (1. - gammav * pm180 / gammac180) / (1. - m180);
    let iv = ci * cv + si * sv * cp;
    let bigf = 0.5 * (1. + iv) * gammav / gamma;
    let f = (1. - phi / pi) * f0 * bigf0 + (phi / pi) * f180 * bigf180;
    let kc = (beta * f + (1. - beta) * bigf) * (1. - kg);
    let kz = 1. - kg - kc;
    let weights = [kg, kc, kz];
    if weights.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let brf = p
        .sunlit_ground_reflectance
        .iter()
        .zip(&p.sunlit_crown_reflectance)
        .zip(&p.shaded_reflectance)
        .map(|((&rg, &rc), &rz)| {
            let v = kg * rg + kc * rc + kz * rz;
            if legacy {
                if v > 10000. { 1. } else { v.max(0.) }
            } else {
                v
            }
        })
        .collect::<Vec<_>>();
    if brf.iter().any(|v| !v.is_finite()) {
        return None;
    }
    Some(Point {
        brdf: brf.iter().map(|v| v / PI).collect(),
        brf,
        sunlit_ground_weight: kg,
        sunlit_crown_weight: kc,
        shaded_weight: kz,
        invalid_weight_count: weights
            .iter()
            .filter(|&&v| !(-1e-10..=1. + 1e-10).contains(&v))
            .count(),
    })
}
pub fn evaluate(p: &Input) -> Result<Point, ModelError> {
    p.validate()?;
    point(p).ok_or_else(|| {
        domain("directions are below the local horizon or the GOMS geometry is singular")
    })
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Output {
    pub hotspot_view_zenith_deg: f64,
    pub hotspot_view_azimuth_deg: f64,
    pub hotspot_brf: Vec<f64>,
    pub hotspot_shaded_weight: f64,
    pub principal_shaded_weight: Vec<f64>,
    #[serde(flatten)]
    pub point: Point,
    pub band_index: Vec<f64>,
    pub principal_view_zenith_deg: Vec<f64>,
    pub principal_valid: Vec<f64>,
    pub principal_brf: Vec<Vec<f64>>,
    pub hemisphere_view_zenith_deg: Vec<f64>,
    pub hemisphere_azimuth_deg: Vec<f64>,
    pub hemisphere_valid: Vec<f64>,
    pub hemisphere_brf: Vec<Vec<f64>>,
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
        let selected = evaluate(p)?;
        let nb = selected.brf.len();
        let mut hotspot_input = p.clone();
        hotspot_input.view_zenith_deg = p.solar_zenith_deg;
        hotspot_input.view_azimuth_deg = p.solar_azimuth_deg;
        let hotspot = evaluate(&hotspot_input)?;
        let mut out = Output {
            hotspot_view_zenith_deg: p.solar_zenith_deg,
            hotspot_view_azimuth_deg: p.solar_azimuth_deg,
            hotspot_brf: hotspot.brf,
            hotspot_shaded_weight: hotspot.shaded_weight,
            principal_shaded_weight: Vec::with_capacity(171),
            point: selected,
            band_index: (1..=nb).map(|b| b as f64).collect(),
            principal_view_zenith_deg: (-85..=85).map(f64::from).collect(),
            principal_valid: vec![],
            principal_brf: vec![vec![]; nb],
            hemisphere_view_zenith_deg: vec![],
            hemisphere_azimuth_deg: vec![],
            hemisphere_valid: vec![],
            hemisphere_brf: vec![vec![]; nb],
        };
        let mut q = p.clone();
        for &a in &out.principal_view_zenith_deg {
            q.view_zenith_deg = a.abs();
            q.view_azimuth_deg = (p.solar_azimuth_deg + if a < 0. { 180. } else { 0. }) % 360.;
            let sample = point(&q);
            out.principal_shaded_weight
                .push(sample.as_ref().map_or(0., |v| v.shaded_weight));
            append(sample, &mut out.principal_valid, &mut out.principal_brf);
        }
        for z in (0..=85).step_by(5) {
            for az in (0..360).step_by(10) {
                q.view_zenith_deg = z as f64;
                q.view_azimuth_deg = az as f64;
                out.hemisphere_view_zenith_deg.push(z as f64);
                out.hemisphere_azimuth_deg.push(az as f64);
                append(
                    point(&q),
                    &mut out.hemisphere_valid,
                    &mut out.hemisphere_brf,
                );
            }
        }
        self.validate_output(&out)?;
        Ok(out)
    }
}
fn append(p: Option<Point>, mask: &mut Vec<f64>, bands: &mut [Vec<f64>]) {
    mask.push(if p.is_some() { 1. } else { 0. });
    for (b, row) in bands.iter_mut().enumerate() {
        row.push(p.as_ref().map_or(0., |v| v.brf[b]));
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
        let nb = o.band_index.len();
        if !(1..=10).contains(&nb)
            || o.point.brf.len() != nb
            || o.point.brdf.len() != nb
            || o.hotspot_brf.len() != nb
            || o.principal_shaded_weight.len() != 171
            || o.principal_brf.len() != nb
            || o.hemisphere_brf.len() != nb
            || o.principal_view_zenith_deg.len() != 171
            || o.principal_valid.len() != 171
            || o.hemisphere_view_zenith_deg.len() != 648
            || o.hemisphere_azimuth_deg.len() != 648
            || o.hemisphere_valid.len() != 648
            || o.principal_brf.iter().any(|v| v.len() != 171)
            || o.hemisphere_brf.iter().any(|v| v.len() != 648)
            || o.point
                .brf
                .iter()
                .chain(&o.point.brdf)
                .chain(o.principal_brf.iter().flatten())
                .chain(o.hemisphere_brf.iter().flatten())
                .any(|v| !v.is_finite())
        {
            return Err(ModelError::new(
                ErrorKind::Internal,
                "invalid GOMS output shape or nonfinite values",
            ));
        }
        Ok(())
    }
}
