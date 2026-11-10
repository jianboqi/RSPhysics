//! SORT and SORT-L soil optical radiative transfer. Model author: Anxin Ding.
//! Component attribution and distribution terms are in LICENSE-ORIGIN.md.
#[cfg(test)]
mod extended_reference;
mod layer;
mod parameters;
mod shiltz;
mod solver;
pub use parameters::{FilmTransport, Input, LambertInput, Roughness, SubstrateAngles};
use rsphysics_core::SpectralAxis;
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use schemars::JsonSchema;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::sync::{Arc, OnceLock};

pub const BAND_COUNT: usize = 2001;
const BASIS: &str = include_str!("../data/soil_basis.csv");
fn domain(s: &str) -> ModelError {
    ModelError::new(ErrorKind::OutOfDomain, s)
}

struct Sample {
    wavelength_nm: f64,
    water_index: f64,
    water_absorption: f64,
    basis: [f64; 3],
}
struct SpectralData {
    axis: SpectralAxis,
    samples: Vec<Sample>,
}
static DATA: OnceLock<Result<Arc<SpectralData>, ModelError>> = OnceLock::new();
fn data(metadata: &ModelMetadata) -> Result<Arc<SpectralData>, ModelError> {
    let expected = format!("sha256:{:x}", Sha256::digest(BASIS.as_bytes()));
    if metadata.data_hashes.get("soil_basis") != Some(&expected) {
        return Err(ModelError::new(
            ErrorKind::MissingData,
            "SORT spectral basis SHA-256 mismatch",
        ));
    }
    DATA.get_or_init(|| {
        let mut samples = Vec::with_capacity(BAND_COUNT);
        let mut lines = BASIS.lines();
        if lines.next()
            != Some("wavelength_nm,n_w,alpha_w_cm-1,soil_basis_1,soil_basis_2,soil_basis_3")
        {
            return Err(ModelError::new(
                ErrorKind::MissingData,
                "SORT spectral header mismatch",
            ));
        }
        for (index, line) in lines.enumerate() {
            let row: Vec<f64> = line
                .split(',')
                .map(str::parse)
                .collect::<Result<_, _>>()
                .map_err(|_| {
                    ModelError::new(ErrorKind::MissingData, "invalid SORT spectral row")
                })?;
            if row.len() != 6
                || row.iter().any(|x| !x.is_finite())
                || row[0] != 400.0 + index as f64
                || row[1] <= 1.0
                || row[2] < 0.0
            {
                return Err(ModelError::new(
                    ErrorKind::MissingData,
                    "SORT spectral axis or domain mismatch",
                ));
            }
            samples.push(Sample {
                wavelength_nm: row[0],
                water_index: row[1],
                water_absorption: row[2],
                basis: [row[3], row[4], row[5]],
            });
        }
        if samples.len() != BAND_COUNT {
            return Err(ModelError::new(
                ErrorKind::MissingData,
                "SORT requires 2001 spectral samples",
            ));
        }
        let axis = SpectralAxis::new(samples.iter().map(|s| s.wavelength_nm).collect())
            .map_err(|e| ModelError::new(ErrorKind::MissingData, e.to_string()))?;
        Ok(Arc::new(SpectralData { axis, samples }))
    })
    .clone()
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub dry_basis_reflectance: Vec<f64>,
    pub single_scattering_albedo: Vec<f64>,
    pub substrate_brf: Vec<f64>,
    pub wavelengths_nm: SpectralAxis,
    pub dry_brf: Vec<f64>,
    pub wet_brf: Vec<f64>,
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LambertOutput {
    pub wavelengths_nm: SpectralAxis,
    pub dry_reflectance: Vec<f64>,
    pub wet_reflectance: Vec<f64>,
}
pub struct Model {
    metadata: ModelMetadata,
    data: Arc<SpectralData>,
}
pub struct LambertModel {
    metadata: ModelMetadata,
    data: Arc<SpectralData>,
}
impl Model {
    pub fn new() -> Result<Self, ModelError> {
        let metadata = parse_model_manifest(include_str!("../model.toml"))?;
        let data = data(&metadata)?;
        Ok(Self { metadata, data })
    }
    pub fn simulate(&self, p: &Input) -> Result<Output, ModelError> {
        p.validate()?;
        let weights: Vec<f64> = self
            .data
            .samples
            .iter()
            .map(|s| solver::basis(s, p).map(solver::ssa))
            .collect::<Result<_, _>>()?;
        solver::screen_energy(p, weights.iter().copied().fold(f64::NEG_INFINITY, f64::max))?;
        let mut dry_brf = Vec::with_capacity(BAND_COUNT);
        let mut wet_brf = Vec::with_capacity(BAND_COUNT);
        let mut dry_basis_reflectance = Vec::with_capacity(BAND_COUNT);
        let mut substrate_brf = Vec::with_capacity(BAND_COUNT);
        for (sample, &w) in self.data.samples.iter().zip(&weights) {
            let dry = solver::checked(
                solver::directional(
                    p,
                    w,
                    p.solar_zenith_deg.to_radians(),
                    p.view_zenith_deg.to_radians(),
                    p.relative_azimuth_deg.to_radians(),
                ),
                false,
            )?;
            dry_brf.push(dry);
            let (wet, substrate) = solver::wet_directional(sample, p, w, dry)?;
            wet_brf.push(wet);
            substrate_brf.push(substrate);
            dry_basis_reflectance.push(solver::basis(sample, p)?);
        }
        let o = Output {
            wavelengths_nm: self.data.axis.clone(),
            dry_brf,
            wet_brf,
            dry_basis_reflectance,
            single_scattering_albedo: weights,
            substrate_brf,
        };
        self.validate_output(&o)?;
        Ok(o)
    }
}
impl LambertModel {
    pub fn new() -> Result<Self, ModelError> {
        let metadata = parse_model_manifest(include_str!("../model-l.toml"))?;
        let data = data(&metadata)?;
        Ok(Self { metadata, data })
    }
    pub fn simulate(&self, p: &LambertInput) -> Result<LambertOutput, ModelError> {
        p.validate()?;
        let common = p.directional();
        let mut dry_reflectance = Vec::with_capacity(BAND_COUNT);
        let mut wet_reflectance = Vec::with_capacity(BAND_COUNT);
        for sample in &self.data.samples {
            let dry = solver::basis(sample, &common)?;
            dry_reflectance.push(dry);
            wet_reflectance.push(solver::wet_lambert(sample, &common, dry)?);
        }
        let o = LambertOutput {
            wavelengths_nm: self.data.axis.clone(),
            dry_reflectance,
            wet_reflectance,
        };
        self.validate_output(&o)?;
        Ok(o)
    }
}
fn validate(
    axis: &SpectralAxis,
    data: &SpectralData,
    dry: &[f64],
    wet: &[f64],
    bounded: bool,
) -> Result<(), ModelError> {
    if axis != &data.axis || dry.len() != BAND_COUNT || wet.len() != BAND_COUNT {
        return Err(ModelError::new(
            ErrorKind::Internal,
            "soil spectrum axis/shape mismatch",
        ));
    }
    for &v in dry.iter().chain(wet) {
        solver::checked(v, bounded)?;
    }
    Ok(())
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
        validate(&o.wavelengths_nm, &self.data, &o.dry_brf, &o.wet_brf, false)
    }
}
impl PhysicsModel for LambertModel {
    type Input = LambertInput;
    type Output = LambertOutput;
    fn metadata(&self) -> ModelMetadata {
        self.metadata.clone()
    }
    fn validate_input(&self, p: &LambertInput) -> Result<(), ModelError> {
        p.validate()
    }
    fn run(&self, p: &LambertInput) -> Result<LambertOutput, ModelError> {
        self.simulate(p)
    }
    fn validate_output(&self, o: &LambertOutput) -> Result<(), ModelError> {
        validate(
            &o.wavelengths_nm,
            &self.data,
            &o.dry_reflectance,
            &o.wet_reflectance,
            true,
        )
    }
}
