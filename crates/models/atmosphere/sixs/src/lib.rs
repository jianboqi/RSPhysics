//! Structured configuration and output handling for the original 6SV2.1 kernel.
mod input;
mod output;
pub use input::*;
pub use output::*;
pub const RESOURCES_JSON: &str = include_str!("../data/resources.json");
use rsphysics_model_api::{ModelError, ModelMetadata, PhysicsModel, parse_model_manifest};
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
    fn validate_input(&self, p: &Input) -> Result<(), ModelError> {
        p.prepare().map(|_| ())
    }
    fn run(&self, p: &Input) -> Result<Output, ModelError> {
        let prepared = p.prepare()?;
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (report, files) = rsphysics_sixs_native::run(
                &prepared.program,
                &prepared.input_cards,
                &prepared.files,
            )
            .map_err(|e| {
                rsphysics_model_api::ModelError::new(
                    rsphysics_model_api::ErrorKind::InvalidParameter,
                    e,
                )
            })?;
            parse_report(report, files)
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = prepared;
            Err(rsphysics_model_api::ModelError::new(
                rsphysics_model_api::ErrorKind::BackendUnavailable,
                "Use the original Fortran WASM entry point and the shared prepare/complete API",
            ))
        }
    }
    fn validate_output(&self, p: &Output) -> Result<(), ModelError> {
        p.validate()
    }
}
