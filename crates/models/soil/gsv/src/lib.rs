//! Jiang & Fang (2019) GSV1/GSV3. See LICENSE-ORIGIN.md for resource terms.
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::{Arc, OnceLock};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum Variant {
    Gsv1,
    #[default]
    Gsv3,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    pub variant: Variant,
    #[schemars(range(min=-10.,max=10.),extend("x-unit"="1"))]
    pub c1: f64,
    #[schemars(range(min=-10.,max=10.),extend("x-unit"="1"))]
    pub c2: f64,
    #[schemars(range(min=-10.,max=10.),extend("x-unit"="1"))]
    pub c3: f64,
    #[schemars(range(min=-0.6,max=0.),extend("x-unit"="1"))]
    pub csm: f64,
    /// Empty uses 400..2500 nm at 10 nm. Otherwise request order is retained.
    #[schemars(length(min=0,max=10000),extend("x-unit"="nm"))]
    pub wavelengths_nm: Vec<f64>,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            variant: Variant::Gsv3,
            c1: 0.4,
            c2: -0.01,
            c3: 0.04,
            csm: -0.13,
            wavelengths_nm: vec![],
        }
    }
}
fn domain(s: impl Into<String>) -> ModelError {
    ModelError::new(ErrorKind::OutOfDomain, s)
}
impl Input {
    pub fn validate(&self) -> Result<(), ModelError> {
        for (name, v) in [("c1", self.c1), ("c2", self.c2), ("c3", self.c3)] {
            if !v.is_finite() || !(-10. ..=10.).contains(&v) {
                return Err(domain(format!("{name} must be finite and within [-10,10]")));
            }
        }
        if !self.csm.is_finite() || !(-0.6..=0.).contains(&self.csm) {
            return Err(domain(
                "csm must be in [-0.6,0]; it is a spectral coefficient, not volumetric moisture",
            ));
        }
        if self.wavelengths_nm.len() > 10000
            || self
                .wavelengths_nm
                .iter()
                .any(|w| !w.is_finite() || !(400. ..=2500.).contains(w))
        {
            return Err(domain(
                "request at most 10000 wavelengths, each finite and in [400,2500] nm; no extrapolation",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub wavelengths_nm: Vec<f64>,
    pub dry_reflectance: Vec<f64>,
    pub moisture_contribution: Vec<f64>,
    pub reflectance: Vec<f64>,
    pub outside_unit_interval_count: usize,
}
const DRY: &str = include_str!("../data/DryVec.txt");
const WET: &str = include_str!("../data/SMVec.txt");
type Basis = Vec<Vec<f64>>;
static DATA: OnceLock<Result<Arc<Basis>, ModelError>> = OnceLock::new();
pub struct Model {
    metadata: ModelMetadata,
    basis: Arc<Basis>,
}
impl Model {
    pub fn new() -> Result<Self, ModelError> {
        let metadata = parse_model_manifest(include_str!("../model.toml"))?;
        for (key, content) in [("dry_vectors", DRY), ("moisture_vector", WET)] {
            if metadata.data_hashes.get(key)
                != Some(&format!("sha256:{:x}", Sha256::digest(content.as_bytes())))
            {
                return Err(ModelError::new(
                    ErrorKind::MissingData,
                    "GSV resource hash mismatch",
                ));
            }
        }
        let basis = DATA
            .get_or_init(|| {
                let mut rows = vec![];
                for text in [DRY, WET] {
                    for line in text.lines().filter(|l| !l.trim().is_empty()) {
                        let row: Vec<f64> = line
                            .split_whitespace()
                            .map(str::parse)
                            .collect::<Result<_, _>>()
                            .map_err(|_| {
                                ModelError::new(ErrorKind::MissingData, "invalid GSV basis")
                            })?;
                        if row.len() != 211 || row.iter().any(|v| !v.is_finite()) {
                            return Err(ModelError::new(
                                ErrorKind::MissingData,
                                "GSV basis must have 211 finite samples",
                            ));
                        }
                        rows.push(row);
                    }
                }
                if rows.len() != 4 {
                    return Err(ModelError::new(
                        ErrorKind::MissingData,
                        "GSV requires four basis vectors",
                    ));
                }
                Ok(Arc::new(rows))
            })
            .clone()?;
        Ok(Self { metadata, basis })
    }
    pub fn simulate(&self, p: &Input) -> Result<Output, ModelError> {
        p.validate()?;
        let wl = if p.wavelengths_nm.is_empty() {
            (0..211).map(|i| 400. + 10. * f64::from(i)).collect()
        } else {
            p.wavelengths_nm.clone()
        };
        let mut out = Output {
            wavelengths_nm: wl,
            dry_reflectance: vec![],
            moisture_contribution: vec![],
            reflectance: vec![],
            outside_unit_interval_count: 0,
        };
        for &w in &out.wavelengths_nm {
            let x = (w - 400.) / 10.;
            let lo = x.floor() as usize;
            let hi = (lo + 1).min(210);
            let t = x - lo as f64;
            let b = |j: usize| self.basis[j][lo] + t * (self.basis[j][hi] - self.basis[j][lo]);
            let dry = p.c1 * b(0)
                + if p.variant == Variant::Gsv3 {
                    p.c2 * b(1) + p.c3 * b(2)
                } else {
                    0.
                };
            let moisture = p.csm * b(3);
            let wet = dry + moisture;
            out.dry_reflectance.push(dry);
            out.moisture_contribution.push(moisture);
            out.reflectance.push(wet);
            out.outside_unit_interval_count += usize::from(!(0. ..=1.).contains(&wet));
        }
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
        let n = o.wavelengths_nm.len();
        if n == 0
            || [&o.dry_reflectance, &o.moisture_contribution, &o.reflectance]
                .iter()
                .any(|a| a.len() != n || a.iter().any(|v| !v.is_finite()))
        {
            return Err(ModelError::new(ErrorKind::Internal, "invalid GSV spectrum"));
        }
        Ok(())
    }
}
