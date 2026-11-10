use crate::{API_VERSION, ErrorKind, ModelError};
use rsphysics_core::RadiometricQuantity;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ModelId(String);

impl ModelId {
    pub fn new(value: impl Into<String>) -> Result<Self, ModelError> {
        let value = value.into();
        let segments: Vec<_> = value.split('.').collect();
        if segments.len() < 2
            || segments.iter().any(|s| {
                s.is_empty()
                    || !s.as_bytes()[0].is_ascii_lowercase()
                    || s.ends_with('-')
                    || !s
                        .bytes()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
            })
        {
            return Err(ModelError::new(
                ErrorKind::InvalidParameter,
                "model ID must have at least two dotted lowercase segments, starting with a letter and not ending with a hyphen",
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ModelId {
    type Error = ModelError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<ModelId> for String {
    fn from(value: ModelId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Backend {
    Native,
    Python,
    Wasm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Precision {
    #[serde(rename = "f64")]
    F64,
    #[serde(rename = "mixed_f32_f64")]
    MixedF32F64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    pub native: bool,
    pub python: bool,
    pub wasm: bool,
    pub batch: bool,
    pub precision: Precision,
}

impl Capabilities {
    pub fn supports(self, backend: Backend) -> bool {
        match backend {
            Backend::Native => self.native,
            Backend::Python => self.python,
            Backend::Wasm => self.wasm,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationStatus {
    Experimental,
    ReferenceValidated,
    BenchmarkEvaluated,
    Deprecated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationKind {
    Implementation,
    ScientificBenchmark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationEvidence {
    pub kind: ValidationKind,
    pub suite: String,
    pub implementation_version: String,
    pub backend: Backend,
}

/// Physical semantics keyed by JSON Pointer, e.g. `/reflectance`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldSemantics {
    pub unit: String,
    pub quantity: Option<RadiometricQuantity>,
    pub axes: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelMetadata {
    pub id: ModelId,
    pub name: String,
    pub family: String,
    pub category: String,
    pub scientific_version: String,
    pub implementation_version: String,
    pub api_version: String,
    pub source_crate: String,
    pub authors: Vec<String>,
    pub references: Vec<String>,
    pub license: String,
    pub limitations: String,
    pub capabilities: Capabilities,
    pub status: ValidationStatus,
    pub validation: Vec<ValidationEvidence>,
    /// SHA-256 of resources actually consumed by this implementation; empty means none.
    pub data_hashes: BTreeMap<String, String>,
    pub input_semantics: BTreeMap<String, FieldSemantics>,
    pub output_semantics: BTreeMap<String, FieldSemantics>,
}

impl ModelMetadata {
    pub fn validate(&self) -> Result<(), ModelError> {
        let invalid = |message| ModelError::new(ErrorKind::InvalidParameter, message);
        if self.api_version != API_VERSION {
            return Err(invalid("unsupported Model API version"));
        }
        if semver::Version::parse(&self.implementation_version).is_err() {
            return Err(invalid("implementation_version must be a full SemVer"));
        }
        if !(self.capabilities.native || self.capabilities.python || self.capabilities.wasm) {
            return Err(invalid("at least one execution backend must be supported"));
        }
        if [
            &self.name,
            &self.family,
            &self.category,
            &self.scientific_version,
            &self.implementation_version,
            &self.source_crate,
            &self.license,
            &self.limitations,
        ]
        .iter()
        .any(|s| s.trim().is_empty())
            || self.authors.is_empty()
            || self.references.is_empty()
            || self
                .authors
                .iter()
                .chain(&self.references)
                .any(|s| s.trim().is_empty())
        {
            return Err(invalid(
                "model identity, attribution, license and limitations must be explicit",
            ));
        }
        for hash in self.data_hashes.values() {
            let valid = hash
                .strip_prefix("sha256:")
                .is_some_and(|s| s.len() == 64 && s.bytes().all(|c| c.is_ascii_hexdigit()));
            if !valid {
                return Err(invalid("resource hashes must use sha256:<64 hex digits>"));
            }
        }
        for (pointer, semantics) in self.input_semantics.iter().chain(&self.output_semantics) {
            if !pointer.starts_with('/')
                || semantics.unit.trim().is_empty()
                || semantics.description.trim().is_empty()
                || semantics
                    .quantity
                    .is_some_and(|q| q.unit() != semantics.unit)
            {
                return Err(invalid(
                    "field semantics require JSON Pointer, description and a compatible explicit unit",
                ));
            }
        }
        for evidence in &self.validation {
            if evidence.suite.trim().is_empty()
                || evidence.implementation_version != self.implementation_version
                || !self.capabilities.supports(evidence.backend)
            {
                return Err(invalid(
                    "validation evidence must match this implementation and a supported backend",
                ));
            }
        }
        let has = |kind| self.validation.iter().any(|e| e.kind == kind);
        if matches!(
            self.status,
            ValidationStatus::ReferenceValidated | ValidationStatus::BenchmarkEvaluated
        ) && !has(ValidationKind::Implementation)
        {
            return Err(invalid("validated status requires implementation evidence"));
        }
        if self.status == ValidationStatus::BenchmarkEvaluated
            && !has(ValidationKind::ScientificBenchmark)
        {
            return Err(invalid(
                "benchmark status requires scientific benchmark evidence",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelDescriptor {
    pub metadata: ModelMetadata,
    pub input_schema: Value,
    pub output_schema: Value,
}
