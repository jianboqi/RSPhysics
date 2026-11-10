//! Shared synchronous execution and reproducibility records. Binding-specific buffers
//! and parallel execution will be introduced after the first reference model.
use rsphysics_catalog::Catalog;
use rsphysics_model_api::{
    Backend, ErrorKind, FieldSemantics, ModelDescriptor, ModelError, Precision, ValidationEvidence,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildInfo {
    pub platform_version: String,
    pub git_revision: Option<String>,
    pub build_id: Option<String>,
    pub target: String,
}

impl Default for BuildInfo {
    fn default() -> Self {
        Self {
            platform_version: env!("CARGO_PKG_VERSION").into(),
            git_revision: option_env!("RSP_GIT_REVISION").map(str::to_owned),
            build_id: option_env!("RSP_BUILD_ID").map(str::to_owned),
            target: format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunManifest {
    pub model_id: String,
    pub scientific_version: String,
    pub implementation_version: String,
    pub api_version: String,
    pub build: BuildInfo,
    pub backend: Backend,
    pub precision: Precision,
    pub parameters: Value,
    pub data_hashes: BTreeMap<String, String>,
    pub validation: Vec<ValidationEvidence>,
    pub input_semantics: BTreeMap<String, FieldSemantics>,
    pub output_semantics: BTreeMap<String, FieldSemantics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultEnvelope {
    pub result: Value,
    pub manifest: RunManifest,
}

/// Batch is ordered and fail-fast. An error identifies the zero-based row;
/// partial results are not returned. No implicit broadcasting or thread pool.
pub struct Runtime {
    catalog: Catalog,
    build: BuildInfo,
    backend: Backend,
}

impl Runtime {
    pub fn new(catalog: Catalog, build: BuildInfo, backend: Backend) -> Self {
        Self {
            catalog,
            build,
            backend,
        }
    }

    pub fn list_models(&self) -> Vec<&ModelDescriptor> {
        self.catalog.list_models()
    }

    pub fn describe(&self, id: &str) -> Result<&ModelDescriptor, ModelError> {
        Ok(self.catalog.get(id)?.descriptor())
    }

    pub fn run(&self, id: &str, parameters: &Value) -> Result<ResultEnvelope, ModelError> {
        let adapter = self.catalog.get(id)?;
        let metadata = &adapter.descriptor().metadata;
        if !metadata.capabilities.supports(self.backend) {
            return Err(ModelError::new(
                ErrorKind::BackendUnavailable,
                "model does not support this runtime backend",
            ));
        }
        let result = adapter.run_json(parameters)?;
        Ok(ResultEnvelope {
            result: result.result,
            manifest: self.manifest_for(id, &result.effective_parameters)?,
        })
    }

    /// Build provenance for a completed, validated typed computation (e.g. NumPy binding).
    /// This method does not execute or validate parameter values; the caller must do so.
    pub fn manifest_for(
        &self,
        id: &str,
        effective_parameters: &Value,
    ) -> Result<RunManifest, ModelError> {
        let metadata = &self.catalog.get(id)?.descriptor().metadata;
        if !metadata.capabilities.supports(self.backend) {
            return Err(ModelError::new(
                ErrorKind::BackendUnavailable,
                "model does not support this runtime backend",
            ));
        }
        Ok(RunManifest {
            model_id: metadata.id.as_str().into(),
            scientific_version: metadata.scientific_version.clone(),
            implementation_version: metadata.implementation_version.clone(),
            api_version: metadata.api_version.clone(),
            build: self.build.clone(),
            backend: self.backend,
            precision: metadata.capabilities.precision,
            parameters: effective_parameters.clone(),
            data_hashes: metadata.data_hashes.clone(),
            validation: metadata.validation.clone(),
            input_semantics: metadata.input_semantics.clone(),
            output_semantics: metadata.output_semantics.clone(),
        })
    }

    pub fn run_batch(&self, id: &str, rows: &[Value]) -> Result<Vec<ResultEnvelope>, ModelError> {
        let descriptor = self.describe(id)?;
        if !descriptor.metadata.capabilities.batch
            || !descriptor.metadata.capabilities.supports(self.backend)
        {
            return Err(ModelError::new(
                ErrorKind::BackendUnavailable,
                "model does not support batch on this backend",
            ));
        }
        rows.iter()
            .enumerate()
            .map(|(i, row)| {
                self.run(id, row)
                    .map_err(|e| ModelError::new(e.kind, format!("batch row {i}: {}", e.message)))
            })
            .collect()
    }
}
