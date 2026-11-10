//! PROSAIL-D (PROSPECT-D + 4SAIL), authored forward-model reference, not a coupling framework.
mod parameters;
mod sail;
pub use parameters::Input;

use rsphysics_core::SpectralAxis;
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use rsphysics_prospect_d::BAND_COUNT;
use schemars::JsonSchema;
use serde::Serialize;
use sha2::{Digest, Sha256};

const SOIL: &str = include_str!("../data/soil.tsv");
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub wavelengths_nm: SpectralAxis,
    /// Direct-beam bidirectional reflectance factor, not bounded by one.
    pub brf: Vec<f64>,
    /// Directional-hemispherical reflectance under direct solar illumination.
    pub hemispherical_reflectance: Vec<f64>,
}
pub struct Model {
    metadata: ModelMetadata,
    leaf: rsphysics_prospect_d::Model,
    soil: Vec<(f64, f64)>,
}
impl Model {
    pub fn new() -> Result<Self, ModelError> {
        let metadata = parse_model_manifest(include_str!("../model.toml"))?;
        let leaf = rsphysics_prospect_d::Model::new()?;
        if metadata.data_hashes.get("leaf_coefficients")
            != leaf.metadata().data_hashes.get("coefficients")
            || metadata.data_hashes.get("soil_coefficients")
                != Some(&format!("sha256:{:x}", Sha256::digest(SOIL.as_bytes())))
        {
            return Err(ModelError::new(
                ErrorKind::MissingData,
                "PROSAIL coefficient identity mismatch",
            ));
        }
        let mut lines = SOIL.lines();
        if lines.next() != Some("lambda\tDirect_Light\tDiffuse_Light\tDry_Soil\tWet_Soil") {
            return Err(ModelError::new(
                ErrorKind::MissingData,
                "soil table header changed",
            ));
        }
        let mut soil = Vec::with_capacity(BAND_COUNT);
        for (i, line) in lines.enumerate() {
            let row: Vec<f64> = line
                .split_whitespace()
                .map(str::parse)
                .collect::<Result<_, _>>()
                .map_err(|_| {
                    ModelError::new(ErrorKind::MissingData, "cannot decode soil coefficients")
                })?;
            if row.len() != 5
                || row.iter().any(|v| !v.is_finite())
                || row[0] != 400.0 + i as f64
                || !(0.0..=1.0).contains(&row[3])
                || !(0.0..=1.0).contains(&row[4])
            {
                return Err(ModelError::new(
                    ErrorKind::MissingData,
                    "soil coefficient shape/axis/domain changed",
                ));
            }
            soil.push((row[3], row[4]));
        }
        if soil.len() != BAND_COUNT {
            return Err(ModelError::new(
                ErrorKind::MissingData,
                "expected 2101 soil rows",
            ));
        }
        Ok(Self {
            metadata,
            leaf,
            soil,
        })
    }
    pub fn simulate(&self, input: &Input) -> Result<Output, ModelError> {
        input.validate()?;
        let soil: Vec<f64> = self
            .soil
            .iter()
            .map(|(dry, wet)| {
                input.soil_brightness
                    * (input.soil_dry_fraction * dry + (1.0 - input.soil_dry_fraction) * wet)
            })
            .collect();
        let axis = SpectralAxis::new(self.leaf.wavelengths_nm().to_vec())
            .map_err(|e| ModelError::new(ErrorKind::Internal, e.to_string()))?;
        if input.leaf_area_index == 0.0 {
            return Ok(Output {
                wavelengths_nm: axis,
                brf: soil.clone(),
                hemispherical_reflectance: soil,
            });
        }
        let leaf = self.leaf.simulate(&input.leaf())?;
        let geometry = sail::Geometry::new(input);
        let mut brf = Vec::with_capacity(BAND_COUNT);
        let mut hemispherical_reflectance = Vec::with_capacity(BAND_COUNT);
        for ((rho, tau), soil) in leaf.reflectance.iter().zip(&leaf.transmittance).zip(soil) {
            let (r, h) = geometry.solve(*rho, *tau, soil, input);
            brf.push(r);
            hemispherical_reflectance.push(h);
        }
        let output = Output {
            wavelengths_nm: axis,
            brf,
            hemispherical_reflectance,
        };
        self.validate_output(&output)?;
        Ok(output)
    }
}
impl PhysicsModel for Model {
    type Input = Input;
    type Output = Output;
    fn metadata(&self) -> ModelMetadata {
        self.metadata.clone()
    }
    fn validate_input(&self, input: &Input) -> Result<(), ModelError> {
        input.validate()
    }
    fn run(&self, input: &Input) -> Result<Output, ModelError> {
        self.simulate(input)
    }
    fn validate_output(&self, output: &Output) -> Result<(), ModelError> {
        if output.wavelengths_nm.wavelengths_nm() != self.leaf.wavelengths_nm()
            || output.brf.len() != BAND_COUNT
            || output.hemispherical_reflectance.len() != BAND_COUNT
        {
            return Err(ModelError::new(
                ErrorKind::Internal,
                "PROSAIL output shape mismatch",
            ));
        }
        for (&brf, &hemi) in output.brf.iter().zip(&output.hemispherical_reflectance) {
            if !brf.is_finite()
                || !hemi.is_finite()
                || brf < -2e-10
                || !(-2e-10..=1.0 + 2e-10).contains(&hemi)
            {
                return Err(ModelError::new(
                    ErrorKind::NonConvergence,
                    "4SAIL produced invalid optics; BRF is intentionally not clipped to one",
                ));
            }
        }
        Ok(())
    }
}
