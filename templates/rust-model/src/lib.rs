//! Copy this crate into crates/models/<domain>/<name>, then implement a verified solver.
use rsphysics_core::SpectralAxis;
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub wavelengths_nm: SpectralAxis,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub reflectance: Vec<f64>,
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
}

impl PhysicsModel for Model {
    type Input = Input;
    type Output = Output;

    fn metadata(&self) -> ModelMetadata {
        self.metadata.clone()
    }

    fn validate_input(&self, _input: &Input) -> Result<(), ModelError> {
        // SpectralAxis already enforces its invariants. Add model-specific domain checks.
        Ok(())
    }

    fn run(&self, input: &Input) -> Result<Output, ModelError> {
        self.validate_input(input)?;
        Err(ModelError::new(
            ErrorKind::MissingData,
            "template has no scientific solver or coefficient data",
        ))
    }

    fn validate_output(&self, output: &Output) -> Result<(), ModelError> {
        if output.reflectance.is_empty()
            || output
                .reflectance
                .iter()
                .any(|x| !x.is_finite() || !(0.0..=1.0).contains(x))
        {
            return Err(ModelError::new(
                ErrorKind::Internal,
                "reflectance must be nonempty, finite and in [0, 1]",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsphysics_model_api::{ModelAdapter, TypedAdapter, serde_json::json};

    #[test]
    fn template_describes_itself_but_never_returns_invented_scientific_results() {
        let adapter = TypedAdapter::new(Model::new().unwrap()).unwrap();
        assert_eq!(
            adapter.descriptor().metadata.id.as_str(),
            "template.example"
        );
        let error = match adapter.run_json(&json!({"wavelengths_nm": [400, 500]})) {
            Ok(_) => panic!("template must not invent scientific outputs"),
            Err(error) => error,
        };
        assert_eq!(error.kind, ErrorKind::MissingData);
    }
}
