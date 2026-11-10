//! BSM, Verhoef (2012), Yang et al. (2020). GPL-3.0-only, see NOTICE.md.
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use rsphysics_numerics::optics::dielectric_cone_transmittance as tav;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::{Arc, OnceLock};
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum PoissonMode {
    #[default]
    Complete,
    LegacySixFilms,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    #[schemars(range(min=0.,max=1.),extend("x-unit"="1"))]
    pub soil_brightness: f64,
    #[schemars(range(min=-90.,max=90.),extend("x-unit"="deg"))]
    pub shape_latitude_deg: f64,
    #[schemars(range(min=0.,max=360.),extend("x-unit"="deg"))]
    pub shape_longitude_deg: f64,
    #[schemars(range(min=0.,max=55.),extend("x-unit"="percent"))]
    pub soil_moisture_percent: f64,
    #[schemars(range(min=5.,max=100.),extend("x-unit"="percent"))]
    pub moisture_capacity_percent: f64,
    #[schemars(range(min=0.,max=1.),extend("x-unit"="cm"))]
    pub water_film_thickness_cm: f64,
    pub poisson_mode: PoissonMode,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            soil_brightness: 0.5,
            shape_latitude_deg: -10.,
            shape_longitude_deg: 100.,
            soil_moisture_percent: 25.,
            moisture_capacity_percent: 25.,
            water_film_thickness_cm: 0.015,
            poisson_mode: PoissonMode::Complete,
        }
    }
}
impl Input {
    pub fn validate(&self) -> Result<(), ModelError> {
        for (k, v, l, h) in [
            ("soil_brightness", self.soil_brightness, 0., 1.),
            ("shape_latitude_deg", self.shape_latitude_deg, -90., 90.),
            ("shape_longitude_deg", self.shape_longitude_deg, 0., 360.),
            ("soil_moisture_percent", self.soil_moisture_percent, 0., 55.),
            (
                "moisture_capacity_percent",
                self.moisture_capacity_percent,
                5.,
                100.,
            ),
            (
                "water_film_thickness_cm",
                self.water_film_thickness_cm,
                0.,
                1.,
            ),
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
struct Sample {
    wl: f64,
    kw: f64,
    basis: [f64; 3],
    ratio: f64,
    p: f64,
    rw: f64,
}
const COEF: &str = include_str!("../data/BSM_coef.dat");
static DATA: OnceLock<Result<Arc<Vec<Sample>>, ModelError>> = OnceLock::new();
pub struct Model {
    metadata: ModelMetadata,
    data: Arc<Vec<Sample>>,
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub wavelengths_nm: Vec<f64>,
    pub dry_reflectance: Vec<f64>,
    pub wet_reflectance: Vec<f64>,
    pub mean_film_count: f64,
    pub dry_area_fraction: f64,
    pub poisson_probability_sum: f64,
    pub omitted_probability: f64,
}
impl Model {
    pub fn new() -> Result<Self, ModelError> {
        let metadata = parse_model_manifest(include_str!("../model.toml"))?;
        if metadata.data_hashes.get("bsm_coefficients")
            != Some(&format!("sha256:{:x}", Sha256::digest(COEF.as_bytes())))
        {
            return Err(ModelError::new(
                ErrorKind::MissingData,
                "BSM coefficient hash mismatch",
            ));
        }
        let data = DATA
            .get_or_init(|| {
                let mut data = vec![];
                let t2 = tav(90., 2.).expect("fixed real index");
                for line in COEF.lines().skip(1).filter(|l| !l.trim().is_empty()) {
                    let v: Vec<f64> = line
                        .split_whitespace()
                        .map(str::parse)
                        .collect::<Result<_, _>>()
                        .map_err(|_| ModelError::new(ErrorKind::MissingData, "invalid BSM data"))?;
                    if v.len() != 6
                        || v.iter().any(|x| !x.is_finite())
                        || v[0] != 400. + data.len() as f64
                        || v[1] <= 1.
                        || v[1] >= 2.
                        || v[2] < 0.
                    {
                        return Err(ModelError::new(
                            ErrorKind::MissingData,
                            "BSM table shape/domain mismatch",
                        ));
                    }
                    let n = v[1];
                    data.push(Sample {
                        wl: v[0],
                        kw: v[2],
                        basis: [v[3], v[4], v[5]],
                        ratio: tav(90., 2. / n).expect("validated index") / t2,
                        p: 1. - tav(90., n).expect("validated index") / (n * n),
                        rw: 1. - tav(40., n).expect("validated index"),
                    });
                }
                if data.len() != 2001 {
                    return Err(ModelError::new(
                        ErrorKind::MissingData,
                        "BSM needs 400..2400nm at 1nm",
                    ));
                }
                Ok(Arc::new(data))
            })
            .clone()?;
        Ok(Self { metadata, data })
    }
    pub fn simulate(&self, p: &Input) -> Result<Output, ModelError> {
        p.validate()?;
        let a = p.shape_latitude_deg.to_radians();
        let b = p.shape_longitude_deg.to_radians();
        let f = [
            p.soil_brightness * a.sin(),
            p.soil_brightness * a.cos() * b.sin(),
            p.soil_brightness * a.cos() * b.cos(),
        ];
        let mu = ((p.soil_moisture_percent - 5.) / p.moisture_capacity_percent).max(0.);
        let mut probabilities = vec![(-mu).exp()];
        let mut mass = probabilities[0];
        for k in 1..=100 {
            let value = probabilities[k - 1] * mu / k as f64;
            probabilities.push(value);
            mass += value;
            if p.poisson_mode == PoissonMode::LegacySixFilms && k == 6 {
                break;
            }
            if p.poisson_mode == PoissonMode::Complete && k as f64 > mu && 1. - mass < 1e-14 {
                break;
            }
        }
        let mut o = Output {
            wavelengths_nm: vec![],
            dry_reflectance: vec![],
            wet_reflectance: vec![],
            mean_film_count: mu,
            dry_area_fraction: probabilities[0],
            poisson_probability_sum: mass,
            omitted_probability: (1. - mass).max(0.),
        };
        for s in self.data.iter() {
            let dry: f64 = f.iter().zip(s.basis).map(|(a, b)| a * b).sum();
            if !(0. ..=1.).contains(&dry) {
                return Err(ModelError::new(
                    ErrorKind::OutOfDomain,
                    format!(
                        "dry BSM reflectance {dry} at {}nm is outside [0,1]; change brightness/shape",
                        s.wl
                    ),
                ));
            }
            let bac = 1. - (1. - dry) * (dry * s.ratio + 1. - dry);
            let wet = if mu == 0. {
                dry
            } else {
                let mut wet = dry * probabilities[0];
                for (k, &prob) in probabilities.iter().enumerate().skip(1) {
                    let tw = (-2. * s.kw * p.water_film_thickness_cm * k as f64).exp();
                    wet +=
                        prob * (s.rw + (1. - s.rw) * (1. - s.p) * tw * bac / (1. - s.p * tw * bac));
                }
                wet
            };
            o.wavelengths_nm.push(s.wl);
            o.dry_reflectance.push(dry);
            o.wet_reflectance.push(wet);
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
        if o.wavelengths_nm.len() != 2001
            || [&o.dry_reflectance, &o.wet_reflectance].iter().any(|a| {
                a.len() != 2001
                    || a.iter()
                        .any(|v| !v.is_finite() || *v < -1e-12 || *v > 1. + 1e-12)
            })
        {
            return Err(ModelError::new(ErrorKind::Internal, "invalid BSM spectrum"));
        }
        Ok(())
    }
}
