//! PROSPECT-D leaf optics, ported from the authors' MIT-declared prospect v1.7.0.
//! Original authors: Jean-Baptiste Feret and Florian de Boissieu.
//! See LICENSE-ORIGIN.md for the pinned source, data and scientific references.
mod parameters;
mod solver;
pub use parameters::Input;

use rsphysics_core::SpectralAxis;
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use schemars::JsonSchema;
use serde::Serialize;
use sha2::{Digest, Sha256};

pub const BAND_COUNT: usize = 2101;
const COEFFICIENTS: &str = include_str!("../data/coefficients.tsv");

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub wavelengths_nm: SpectralAxis,
    pub reflectance: Vec<f64>,
    pub transmittance: Vec<f64>,
}

struct Coefficient {
    refractive_index: f64,
    absorption: [f64; 6],
    tav_40: f64,
    tav_90: f64,
}

pub struct Model {
    metadata: ModelMetadata,
    axis: SpectralAxis,
    coefficients: Vec<Coefficient>,
}

impl Model {
    pub fn new() -> Result<Self, ModelError> {
        let metadata = parse_model_manifest(include_str!("../model.toml"))?;
        let hash = format!("sha256:{:x}", Sha256::digest(COEFFICIENTS.as_bytes()));
        if metadata.data_hashes.get("coefficients") != Some(&hash) {
            return Err(ModelError::new(
                ErrorKind::MissingData,
                "embedded coefficient SHA-256 differs from manifest",
            ));
        }
        let mut coefficients = Vec::with_capacity(BAND_COUNT);
        let mut wavelengths = Vec::with_capacity(BAND_COUNT);
        let mut lines = COEFFICIENTS.lines();
        if lines.next()
            != Some(
                "lambda\tnrefrac\tSAC_CHL\tSAC_CAR\tSAC_ANT\tSAC_BROWN\tSAC_EWT\tSAC_LMA\tSAC_PROT\tSAC_CBC",
            )
        {
            return Err(ModelError::new(
                ErrorKind::MissingData,
                "coefficient header changed",
            ));
        }
        for (i, line) in lines.enumerate() {
            let values: Vec<f64> = line
                .split_whitespace()
                .map(str::parse)
                .collect::<Result<_, _>>()
                .map_err(|_| {
                    ModelError::new(ErrorKind::MissingData, "cannot decode coefficient row")
                })?;
            if values.len() != 10
                || values.iter().any(|x| !x.is_finite())
                || values[0] != 400.0 + i as f64
                || values[1] <= 1.0
                || values[2..].iter().any(|x| *x < 0.0)
            {
                return Err(ModelError::new(
                    ErrorKind::MissingData,
                    "coefficient axis, shape or domain changed",
                ));
            }
            wavelengths.push(values[0]);
            coefficients.push(Coefficient {
                refractive_index: values[1],
                absorption: values[2..8]
                    .try_into()
                    .expect("six D absorption coefficients"),
                tav_40: solver::calctav(40.0, values[1]),
                tav_90: solver::calctav(90.0, values[1]),
            });
        }
        if coefficients.len() != BAND_COUNT {
            return Err(ModelError::new(
                ErrorKind::MissingData,
                "expected 2101 coefficient rows",
            ));
        }
        let axis = SpectralAxis::new(wavelengths)
            .map_err(|e| ModelError::new(ErrorKind::MissingData, e.to_string()))?;
        Ok(Self {
            metadata,
            axis,
            coefficients,
        })
    }

    /// Same typed solver used by Native, JSON Runtime and Python bindings.
    pub fn simulate(&self, input: &Input) -> Result<Output, ModelError> {
        input.validate()?;
        let mut reflectance = Vec::with_capacity(BAND_COUNT);
        let mut transmittance = Vec::with_capacity(BAND_COUNT);
        for coefficient in &self.coefficients {
            let (r, t) = solver::solve_band(input, coefficient);
            reflectance.push(r);
            transmittance.push(t);
        }
        let output = Output {
            wavelengths_nm: self.axis.clone(),
            reflectance,
            transmittance,
        };
        self.validate_output(&output)?;
        Ok(output)
    }

    pub fn wavelengths_nm(&self) -> &[f64] {
        self.axis.wavelengths_nm()
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
        if output.wavelengths_nm != self.axis
            || output.reflectance.len() != BAND_COUNT
            || output.transmittance.len() != BAND_COUNT
        {
            return Err(ModelError::new(
                ErrorKind::Internal,
                "PROSPECT-D output axis/shape mismatch",
            ));
        }
        for (&r, &t) in output.reflectance.iter().zip(&output.transmittance) {
            // Floating-point allowance only, not clipping or alteration of scientific outputs.
            if !r.is_finite() || !t.is_finite() || r < -2e-10 || t < -2e-10 || r + t > 1.0 + 2e-10 {
                return Err(ModelError::new(
                    ErrorKind::NonConvergence,
                    "PROSPECT-D produced non-finite or non-conservative optics",
                ));
            }
        }
        Ok(())
    }
}
