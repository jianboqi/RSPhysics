//! INFORM forest reflectance. FLIM crown geometry and two SAIL leaf layers.
mod input;
mod stand;

pub use input::{Input, LeafSpectrum, Prospect301Coefficients, SpectralInputs};

use rsphysics_core::SpectralAxis;
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use rsphysics_prospect_d::{Input as ProspectInput, leaf_optics_from_absorption};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Output {
    pub wavelengths_nm: SpectralAxis,
    pub reflectance: Vec<f64>,
    pub soil_reflectance: Vec<f64>,
    pub understory_reflectance: Vec<f64>,
    pub dense_crown_reflectance: Vec<f64>,
    pub crown_leaf_reflectance: Vec<f64>,
    pub crown_leaf_transmittance: Vec<f64>,
    pub understory_leaf_reflectance: Vec<f64>,
    pub understory_leaf_transmittance: Vec<f64>,
    pub crown_transmittance_sun: Vec<f64>,
    pub crown_transmittance_view: Vec<f64>,
    pub crown_factor: Vec<f64>,
    pub ground_factor: Vec<f64>,
    pub crown_cover: f64,
    pub shadow_cover: f64,
    pub shadowed_crown_fraction: f64,
    pub sunlit_crown_fraction: f64,
    pub shadowed_gap_fraction: f64,
    pub sunlit_gap_fraction: f64,
}

pub struct Model {
    metadata: ModelMetadata,
    leaf: rsphysics_prospect_d::Model,
}

impl Model {
    pub fn new() -> Result<Self, ModelError> {
        let metadata = parse_model_manifest(include_str!("../model.toml"))?;
        let leaf = rsphysics_prospect_d::Model::new()?;
        if metadata.data_hashes.get("prospect_d_coefficients")
            != leaf.metadata().data_hashes.get("coefficients")
        {
            return Err(ModelError::new(
                ErrorKind::MissingData,
                "INFORM leaf coefficient identity mismatch",
            ));
        }
        Ok(Self { metadata, leaf })
    }

    pub fn simulate(&self, input: &Input) -> Result<Output, ModelError> {
        input.validate()?;
        let axis = match &input.spectra {
            Some(spectra) => spectra.wavelengths_nm.clone(),
            None => SpectralAxis::new(self.leaf.wavelengths_nm().to_vec())
                .map_err(|e| ModelError::new(ErrorKind::Internal, e.to_string()))?,
        };
        let wavelengths = axis.wavelengths_nm();
        let crown = self.leaf_spectrum(input, wavelengths, false)?;
        let understory = self.leaf_spectrum(input, wavelengths, true)?;
        let stand = stand::Stand::new(input);
        let capacity = wavelengths.len();
        let soil = input
            .spectra
            .as_ref()
            .and_then(|s| s.soil_reflectance.as_deref());
        let mut output = Output {
            wavelengths_nm: axis,
            reflectance: Vec::with_capacity(capacity),
            soil_reflectance: Vec::with_capacity(capacity),
            understory_reflectance: Vec::with_capacity(capacity),
            dense_crown_reflectance: Vec::with_capacity(capacity),
            crown_leaf_reflectance: crown.reflectance,
            crown_leaf_transmittance: crown.transmittance,
            understory_leaf_reflectance: understory.reflectance,
            understory_leaf_transmittance: understory.transmittance,
            crown_transmittance_sun: Vec::with_capacity(capacity),
            crown_transmittance_view: Vec::with_capacity(capacity),
            crown_factor: Vec::with_capacity(capacity),
            ground_factor: Vec::with_capacity(capacity),
            crown_cover: stand.cover,
            shadow_cover: stand.shadow,
            shadowed_crown_fraction: stand.shadowed_crown,
            sunlit_crown_fraction: stand.sunlit_crown,
            shadowed_gap_fraction: stand.shadowed_gap,
            sunlit_gap_fraction: stand.sunlit_gap,
        };
        for i in 0..capacity {
            let bare_soil = input.soil_brightness * soil.map_or(input.soil_reflectance, |s| s[i]);
            let band = stand.solve(
                bare_soil,
                (
                    output.crown_leaf_reflectance[i],
                    output.crown_leaf_transmittance[i],
                ),
                (
                    output.understory_leaf_reflectance[i],
                    output.understory_leaf_transmittance[i],
                ),
            );
            output.reflectance.push(band.forest);
            output.soil_reflectance.push(bare_soil);
            output.understory_reflectance.push(band.understory);
            output.dense_crown_reflectance.push(band.dense_crown);
            output.crown_transmittance_sun.push(band.transmittance_sun);
            output
                .crown_transmittance_view
                .push(band.transmittance_view);
            output.crown_factor.push(band.crown_factor);
            output.ground_factor.push(band.ground_factor);
        }
        self.validate_output(&output)?;
        Ok(output)
    }

    fn leaf_spectrum(
        &self,
        input: &Input,
        axis: &[f64],
        understory: bool,
    ) -> Result<LeafSpectrum, ModelError> {
        let parameters = if understory {
            &input.understory_leaf
        } else {
            &input.crown_leaf
        };
        if let Some(s) = &input.spectra {
            let provided = if understory {
                &s.understory_leaf
            } else {
                &s.crown_leaf
            };
            if let Some(leaf) = provided {
                return Ok(leaf.clone());
            }
            if let Some(coefficients) = &s.prospect_301_coefficients {
                return Ok(prospect_301(parameters, coefficients));
            }
        }
        let leaf = self.leaf.simulate(parameters)?;
        if axis == self.leaf.wavelengths_nm() {
            return Ok(LeafSpectrum {
                reflectance: leaf.reflectance,
                transmittance: leaf.transmittance,
            });
        }
        let interpolate = |values: &[f64], wavelength: f64| {
            let offset = wavelength - 400.;
            let index = offset.floor() as usize;
            let weight = offset.fract();
            values[index] + weight * (values[(index + 1).min(values.len() - 1)] - values[index])
        };
        Ok(LeafSpectrum {
            reflectance: axis
                .iter()
                .map(|&x| interpolate(&leaf.reflectance, x))
                .collect(),
            transmittance: axis
                .iter()
                .map(|&x| interpolate(&leaf.transmittance, x))
                .collect(),
        })
    }
}

fn prospect_301(leaf: &ProspectInput, data: &Prospect301Coefficients) -> LeafSpectrum {
    let (reflectance, transmittance) = data
        .refractive_index
        .iter()
        .enumerate()
        .map(|(i, &n)| {
            let absorption = (leaf.chlorophyll * data.chlorophyll_absorption[i]
                + leaf.water * data.water_absorption[i]
                + leaf.dry_matter * data.dry_matter_absorption[i])
                / leaf.n
                + data.albino_absorption[i];
            leaf_optics_from_absorption(leaf.n, n, absorption, 60.)
        })
        .unzip();
    LeafSpectrum {
        reflectance,
        transmittance,
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
        let len = output.wavelengths_nm.wavelengths_nm().len();
        for spectrum in [
            &output.reflectance,
            &output.soil_reflectance,
            &output.understory_reflectance,
            &output.dense_crown_reflectance,
            &output.crown_leaf_reflectance,
            &output.crown_leaf_transmittance,
            &output.understory_leaf_reflectance,
            &output.understory_leaf_transmittance,
            &output.crown_transmittance_sun,
            &output.crown_transmittance_view,
            &output.crown_factor,
            &output.ground_factor,
        ] {
            if spectrum.len() != len || spectrum.iter().any(|x| !x.is_finite()) {
                return Err(ModelError::new(
                    ErrorKind::NonConvergence,
                    "INFORM returned invalid spectral output",
                ));
            }
        }
        Ok(())
    }
}
