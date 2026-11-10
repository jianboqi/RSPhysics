//! FLUSPECT-CX with PROSPECT-D coefficients and the full and light fluorescence grids.
//! Scientific equations: Verhoef, van der Tol, Timmermans and Vilfan (GPLv3).
mod solver;
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpticalCoefficients {
    #[default]
    ProspectD,
    ProspectPro,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpectralResolution {
    #[default]
    #[serde(rename = "full_1nm")]
    Full1nm,
    #[serde(rename = "light_5x4")]
    Light5x4,
}
impl SpectralResolution {
    pub fn grid(self) -> (usize, usize, usize, usize) {
        match self {
            Self::Full1nm => (351, 211, 1, 1),
            Self::Light5x4 => (71, 53, 5, 4),
        }
    }
}

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
    #[schemars(range(min=-999.0,max=1.0),extend("x-unit"="1"))]
    pub xanthophyll_state: f64,
    /// Combined PSI+PSII fluorescence efficiency; zero disables fluorescence.
    #[schemars(range(min=0.0,max=0.1),extend("x-unit"="1"))]
    pub fluorescence_efficiency: f64,
    #[schemars(range(min=0.0,max=0.05),extend("x-unit"="g cm^-2"))]
    pub protein: f64,
    #[schemars(range(min=0.0,max=0.05),extend("x-unit"="g cm^-2"))]
    pub carbon_based_constituents: f64,
    #[schemars(with="String",extend("enum"=["prospect_d","prospect_pro"]))]
    pub optical_coefficients: OpticalCoefficients,
    #[schemars(with="String",extend("enum"=["full_1nm","light_5x4"]))]
    pub spectral_resolution: SpectralResolution,
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
            protein: 0.,
            carbon_based_constituents: 0.,
            optical_coefficients: OpticalCoefficients::ProspectD,
            spectral_resolution: SpectralResolution::Full1nm,
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
            ("xanthophyll_state", self.xanthophyll_state, -999., 1.),
            ("protein", self.protein, 0., 0.05),
            (
                "carbon_based_constituents",
                self.carbon_based_constituents,
                0.,
                0.05,
            ),
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
        if self.xanthophyll_state != -999. && self.xanthophyll_state < 0. {
            return Err(ModelError::new(
                ErrorKind::OutOfDomain,
                "xanthophyll_state must be -999 (fixed carotenoid spectrum) or in [0,1]",
            ));
        }
        if self.optical_coefficients == OpticalCoefficients::ProspectD
            && (self.protein != 0. || self.carbon_based_constituents != 0.)
        {
            return Err(ModelError::new(
                ErrorKind::OutOfDomain,
                "protein/CBC inputs require prospect_pro optical coefficients",
            ));
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
    /// Rows: emission wavelength; columns: excitation wavelength. Includes the selected excitation-bin width (1 or 5 nm).
    pub fluorescence_backward: Vec<Vec<f64>>,
    /// Rows: emission wavelength; columns: excitation wavelength. Includes the selected excitation-bin width (1 or 5 nm).
    pub fluorescence_forward: Vec<Vec<f64>>,
}

/// Leaf-side spectral energy flux after photon-domain fluorescence transfer.
#[derive(Debug, Clone)]
pub struct FluorescenceSpectrum {
    pub emission_wavelengths_nm: Vec<f64>,
    pub backward_w_m2_nm: Vec<f64>,
    pub forward_w_m2_nm: Vec<f64>,
}
pub fn fluorescence_from_irradiance(
    o: &Output,
    incident_w_m2_nm: &[f64],
) -> Result<FluorescenceSpectrum, ModelError> {
    Model::new()?.validate_output(o)?;
    if incident_w_m2_nm.len() != o.excitation_wavelengths_nm.len()
        || incident_w_m2_nm.iter().any(|v| !v.is_finite() || *v < 0.)
    {
        return Err(ModelError::new(
            ErrorKind::InvalidParameter,
            "incident irradiance must be finite, nonnegative and match the excitation grid",
        ));
    }
    let apply = |matrix: &[Vec<f64>]| -> Vec<f64> {
        matrix
            .iter()
            .zip(&o.emission_wavelengths_nm)
            .map(|(row, emission)| {
                row.iter()
                    .zip(incident_w_m2_nm)
                    .zip(&o.excitation_wavelengths_nm)
                    .map(|((m, e), excitation)| m * e * excitation / emission)
                    .sum()
            })
            .collect()
    };
    Ok(FluorescenceSpectrum {
        emission_wavelengths_nm: o.emission_wavelengths_nm.clone(),
        backward_w_m2_nm: apply(&o.fluorescence_backward),
        forward_w_m2_nm: apply(&o.fluorescence_forward),
    })
}
pub struct Model {
    metadata: ModelMetadata,
}
type OpticalTable = Vec<[f64; 13]>;
static COEFFICIENTS: OnceLock<Result<[OpticalTable; 2], ModelError>> = OnceLock::new();
fn coefficients(mode: OpticalCoefficients) -> Result<&'static OpticalTable, ModelError> {
    let data = COEFFICIENTS
        .get_or_init(|| {
            let parse = |raw: &[u8], hash: &str| -> Result<OpticalTable, ModelError> {
                let bad =
                    || ModelError::new(ErrorKind::Internal, "invalid FLUSPECT optical resource");
                if format!("{:x}", Sha256::digest(raw)) != hash {
                    return Err(bad());
                }
                let decoded =
                    miniz_oxide::inflate::decompress_to_vec_zlib(raw).map_err(|_| bad())?;
                let csv = std::str::from_utf8(&decoded).map_err(|_| bad())?;
                let mut rows = vec![];
                for (index, line) in csv.lines().skip(1).enumerate() {
                    let row: [f64; 13] = line
                        .split(',')
                        .map(str::parse::<f64>)
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|_| bad())?
                        .try_into()
                        .map_err(|_| bad())?;
                    if row.iter().any(|v| !v.is_finite())
                        || row.iter().enumerate().any(|(i, v)| i != 10 && *v < 0.)
                        || row[0] != 400. + index as f64
                        || row[1] <= 1.
                    {
                        return Err(bad());
                    }
                    rows.push(row);
                }
                if rows.len() != 2001 {
                    return Err(bad());
                }
                Ok(rows)
            };
            Ok([
                parse(
                    include_bytes!("../data/optics-d.csv.zlib"),
                    "c7d9dc9e0e987fdcddec26c703e0ac28138e632d2f180d7f4a7f8b91854e6e3c",
                )?,
                parse(
                    include_bytes!("../data/optics-pro.csv.zlib"),
                    "ee34773e5bae1521988b8d029c94358e374d01dc507b271f14d393d7bf914134",
                )?,
            ])
        })
        .as_ref()
        .map_err(Clone::clone)?;
    Ok(&data[if mode == OpticalCoefficients::ProspectD {
        0
    } else {
        1
    }])
}
impl Model {
    pub fn new() -> Result<Self, ModelError> {
        coefficients(OpticalCoefficients::ProspectD)?;
        Ok(Self {
            metadata: parse_model_manifest(include_str!("../model.toml"))?,
        })
    }
    pub fn simulate(&self, p: &Input) -> Result<Output, ModelError> {
        p.validate()?;
        let output = solver::simulate(p, coefficients(p.optical_coefficients)?);
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
            || !matches!(
                (
                    o.excitation_wavelengths_nm.len(),
                    o.emission_wavelengths_nm.len()
                ),
                (351, 211) | (71, 53)
            )
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
            if matrix.len() != o.emission_wavelengths_nm.len()
                || matrix.iter().any(|row| {
                    row.len() != o.excitation_wavelengths_nm.len()
                        || row.iter().any(|v| !v.is_finite() || *v < 0.)
                })
            {
                return Err(bad());
            }
        }
        Ok(())
    }
}
