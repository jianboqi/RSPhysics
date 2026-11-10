//! The versioned on-disk model.toml contract. Coefficients must be embedded or
//! supplied by a model-specific loader which verifies the declared hashes.
use crate::*;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    model: Identity,
    source: Source,
    capabilities: Capabilities,
    #[serde(default)]
    validation: Validation,
    #[serde(default)]
    data: Data,
    #[serde(default)]
    input_semantics: BTreeMap<String, FieldSemantics>,
    #[serde(default)]
    output_semantics: BTreeMap<String, FieldSemantics>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Identity {
    id: ModelId,
    name: String,
    family: String,
    category: String,
    scientific_version: String,
    implementation_version: String,
    api_version: String,
    status: ValidationStatus,
    limitations: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    #[serde(rename = "crate")]
    crate_name: String,
    authors: Vec<String>,
    references: Vec<String>,
    license: String,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Validation {
    #[serde(default)]
    evidence: Vec<ValidationEvidence>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Data {
    #[serde(default)]
    hashes: BTreeMap<String, String>,
}

pub fn parse_model_manifest(text: &str) -> Result<ModelMetadata, ModelError> {
    let manifest: Manifest = toml::from_str(text)
        .map_err(|e| ModelError::new(ErrorKind::InvalidParameter, e.to_string()))?;
    let metadata = ModelMetadata {
        id: manifest.model.id,
        name: manifest.model.name,
        family: manifest.model.family,
        category: manifest.model.category,
        scientific_version: manifest.model.scientific_version,
        implementation_version: manifest.model.implementation_version,
        api_version: manifest.model.api_version,
        source_crate: manifest.source.crate_name,
        authors: manifest.source.authors,
        references: manifest.source.references,
        license: manifest.source.license,
        limitations: manifest.model.limitations,
        capabilities: manifest.capabilities,
        status: manifest.model.status,
        validation: manifest.validation.evidence,
        data_hashes: manifest.data.hashes,
        input_semantics: manifest.input_semantics,
        output_semantics: manifest.output_semantics,
    };
    metadata.validate()?;
    Ok(metadata)
}
