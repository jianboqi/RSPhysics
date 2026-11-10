//! FLUSPECT-CX with PROSPECT-D coefficients and the SCOPE light spectral grid.
//! Scientific equations: Verhoef, van der Tol, Timmermans and Vilfan (GPLv3).
mod solver;
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

pub const OPTICS_SHA256: &str = "8867eb9c0617a955d60331b5f26b660e1440296a5117a11b9c8d37251e277318";
pub const EMISSION_COUNT: usize = 53;
pub const EXCITATION_COUNT: usize = 71;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    #[schemars(range(min=1.0,max=5.0),extend("x-unit"="1"))]
    pub n: f64,
    #[schemars(range(min=0.0,max=150.0),extend("x-unit"="ug cm^-2"))]
    pub chlorophyll: f64,
    #[schemars(range(min=0.0,max=60.0),extend("x-unit"="ug cm^-2"))]
    pub carotenoids: f64,
    #[schemars(range(min=0.0,max=50.0),extend("x-unit"="ug cm^-2"))]
    pub anthocyanins: f64,
    #[schemars(range(min=0.0,max=2.0),extend("x-unit"="1"))]
    pub brown_pigments: f64,
    #[schemars(range(min=0.0,max=0.1),extend("x-unit"="g cm^-2"))]
    pub water: f64,
    #[schemars(range(min=0.0,max=0.05),extend("x-unit"="g cm^-2"))]
    pub dry_matter: f64,
    /// Violaxanthin to zeaxanthin transition state, zero to one.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub xanthophyll_state: f64,
    /// Combined PSI+PSII fluorescence efficiency; zero disables fluorescence.
    #[schemars(range(min=0.0,max=0.1),extend("x-unit"="1"))]
    pub fluorescence_efficiency: f64,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            n: 1.5,
            chlorophyll: 40.,
            carotenoids: 8.,
            anthocyanins: 0.,
            brown_pigments: 0.,
            water: 0.01,
            dry_matter: 0.009,
            xanthophyll_state: 0.,
            fluorescence_efficiency: 0.01,
        }
    }
}
impl Input {
    pub fn validate(&self) -> Result<(), ModelError> {
        for (key, value, lo, hi) in [
            ("n", self.n, 1., 5.),
            ("chlorophyll", self.chlorophyll, 0., 150.),
            ("carotenoids", self.carotenoids, 0., 60.),
            ("anthocyanins", self.anthocyanins, 0., 50.),
            ("brown_pigments", self.brown_pigments, 0., 2.),
            ("water", self.water, 0., 0.1),
            ("dry_matter", self.dry_matter, 0., 0.05),
            ("xanthophyll_state", self.xanthophyll_state, 0., 1.),
            (
                "fluorescence_efficiency",
                self.fluorescence_efficiency,
                0.,
                0.1,
            ),
        ] {
            if !value.is_finite() || !(lo..=hi).contains(&value) {
                return Err(ModelError::new(
                    ErrorKind::OutOfDomain,
                    format!("{key} must be finite in [{lo}, {hi}]"),
                ));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub wavelengths_nm: Vec<f64>,
    pub reflectance: Vec<f64>,
    pub transmittance: Vec<f64>,
    pub chlorophyll_absorption_fraction: Vec<f64>,
    pub carotenoid_absorption_fraction: Vec<f64>,
    pub excitation_wavelengths_nm: Vec<f64>,
    pub emission_wavelengths_nm: Vec<f64>,
    /// Rows: emission wavelength; columns: excitation wavelength. Includes 5 nm excitation-bin weight.
    pub fluorescence_backward: Vec<Vec<f64>>,
    /// Rows: emission wavelength; columns: excitation wavelength. Includes 5 nm excitation-bin weight.
    pub fluorescence_forward: Vec<Vec<f64>>,
}
pub struct Model {
    metadata: ModelMetadata,
    coefficients: &'static Vec<[f64; 11]>,
}
static COEFFICIENTS: OnceLock<Result<Vec<[f64; 11]>, ModelError>> = OnceLock::new();
fn coefficients() -> Result<&'static Vec<[f64; 11]>, ModelError> {
    COEFFICIENTS
        .get_or_init(|| {
            let raw = include_bytes!("../data/optics.csv.zlib");
            let error = |message: &str| ModelError::new(ErrorKind::Internal, message);
            if format!("{:x}", Sha256::digest(raw)) != OPTICS_SHA256 {
                return Err(error("FLUSPECT optical data checksum mismatch"));
            }
            let decoded = miniz_oxide::inflate::decompress_to_vec_zlib(raw)
                .map_err(|_| error("invalid FLUSPECT compressed optics"))?;
            let csv = std::str::from_utf8(&decoded)
                .map_err(|_| error("invalid FLUSPECT optics encoding"))?;
            let mut rows = vec![];
            for (index, line) in csv.lines().skip(1).enumerate() {
                let values = line
                    .split(',')
                    .map(str::parse::<f64>)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| error("invalid FLUSPECT optical coefficient"))?;
                let row: [f64; 11] = values
                    .try_into()
                    .map_err(|_| error("invalid FLUSPECT optical table shape"))?;
                if row.iter().any(|v| !v.is_finite())
                    || row[..10].iter().any(|v| *v < 0.)
                    || row[0] != 400. + index as f64
                    || row[1] <= 1.
                {
                    return Err(error("invalid FLUSPECT optical data domain"));
                }
                rows.push(row);
            }
            if rows.len() != 2001 {
                return Err(error("FLUSPECT requires 2001 optical bands"));
            }
            Ok(rows)
        })
        .as_ref()
        .map_err(Clone::clone)
}
impl Model {
    pub fn new() -> Result<Self, ModelError> {
        Ok(Self {
            metadata: parse_model_manifest(include_str!("../model.toml"))?,
            coefficients: coefficients()?,
        })
    }
    pub fn simulate(&self, p: &Input) -> Result<Output, ModelError> {
        p.validate()?;
        let output = solver::simulate(p, self.coefficients);
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
    fn validate_input(&self, p: &Input) -> Result<(), ModelError> {
        p.validate()
    }
    fn run(&self, p: &Input) -> Result<Output, ModelError> {
        self.simulate(p)
    }
    fn validate_output(&self, o: &Output) -> Result<(), ModelError> {
        let bad = || {
            ModelError::new(
                ErrorKind::Internal,
                "invalid FLUSPECT output shape, energy balance or finite-value invariant",
            )
        };
        if o.wavelengths_nm.len() != 2001
            || o.excitation_wavelengths_nm.len() != EXCITATION_COUNT
            || o.emission_wavelengths_nm.len() != EMISSION_COUNT
        {
            return Err(bad());
        }
        for values in [
            &o.reflectance,
            &o.transmittance,
            &o.chlorophyll_absorption_fraction,
            &o.carotenoid_absorption_fraction,
        ] {
            if values.len() != 2001
                || values
                    .iter()
                    .any(|v| !v.is_finite() || !(-1e-12..=1. + 1e-10).contains(v))
            {
                return Err(bad());
            }
        }
        if o.reflectance
            .iter()
            .zip(&o.transmittance)
            .any(|(r, t)| r + t > 1. + 1e-9)
        {
            return Err(bad());
        }
        for matrix in [&o.fluorescence_backward, &o.fluorescence_forward] {
            if matrix.len() != EMISSION_COUNT
                || matrix.iter().any(|row| {
                    row.len() != EXCITATION_COUNT || row.iter().any(|v| !v.is_finite() || *v < 0.)
                })
            {
                return Err(bad());
            }
        }
        Ok(())
    }
}
