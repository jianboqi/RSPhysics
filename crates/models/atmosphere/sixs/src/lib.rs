//! Structured configuration and output handling for the original 6SV2.1 kernel.
mod input;
mod output;
mod products;
pub use input::*;
pub use output::*;
pub const RESOURCES_JSON: &str = include_str!("../data/resources.json");
use rsphysics_model_api::{ModelError, ModelMetadata, PhysicsModel, parse_model_manifest};
use serde::Deserialize;
use std::sync::OnceLock;

fn band_grid_indices(band: u16) -> std::ops::RangeInclusive<usize> {
    #[derive(Deserialize)]
    struct Band {
        id: u16,
        lower_um: f32,
        upper_um: f32,
    }
    #[derive(Deserialize)]
    struct Resources {
        bands: Vec<Band>,
    }
    static RESOURCES: OnceLock<Resources> = OnceLock::new();
    let band_id = band;
    let band = &RESOURCES
        .get_or_init(|| serde_json::from_str(RESOURCES_JSON).expect("valid bundled 6S resources"))
        .bands[usize::from(band_id - 2)];
    debug_assert_eq!(band.id, band_id);
    // Match the indexing of the single-precision Fortran band bounds.
    let index = |um| ((um - 0.25_f32) / 0.0025 + 1.5) as usize - 1;
    index(band.lower_um)..=index(band.upper_um)
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
