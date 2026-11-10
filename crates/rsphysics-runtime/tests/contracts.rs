//! Engineering fixtures only. These tests provide no scientific validation claim.
use rsphysics_catalog::{Catalog, official_catalog};
use rsphysics_model_api::*;
use rsphysics_runtime::{BuildInfo, Runtime};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Input {
    #[schemars(range(min = 0.0, max = 1.0))]
    x: f64,
    #[serde(default = "default_scale")]
    #[schemars(range(min = 0.0, max = 1.0))]
    scale: f64,
}

fn default_scale() -> f64 {
    1.0
}

#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Output {
    #[schemars(range(min = 0.0, max = 1.0))]
    value: f64,
}

struct Fixture {
    output_override: Option<f64>,
    batch: bool,
}

impl Default for Fixture {
    fn default() -> Self {
        Self {
            output_override: None,
            batch: true,
        }
    }
}

fn metadata() -> ModelMetadata {
    ModelMetadata {
        id: ModelId::new("test.engineering-fixture").unwrap(),
        name: "Engineering fixture (not a physics model)".into(),
        family: "test".into(),
        category: "test".into(),
        scientific_version: "none".into(),
        implementation_version: "0.1.0".into(),
        api_version: API_VERSION.into(),
        source_crate: "rsphysics-runtime".into(),
        authors: vec!["RSPhysics test fixture".into()],
        references: vec!["contracts.rs: arithmetic fixture, no scientific reference".into()],
        license: "UNSET (test fixture only)".into(),
        limitations: "No scientific meaning; never registered in the official catalog".into(),
        capabilities: Capabilities {
            native: true,
            python: false,
            wasm: false,
            batch: true,
            precision: Precision::F64,
        },
        status: ValidationStatus::Experimental,
        validation: vec![],
        data_hashes: BTreeMap::new(),
        input_semantics: BTreeMap::new(),
        output_semantics: BTreeMap::new(),
    }
}

impl PhysicsModel for Fixture {
    type Input = Input;
    type Output = Output;
    fn metadata(&self) -> ModelMetadata {
        let mut metadata = metadata();
        metadata.capabilities.batch = self.batch;
        metadata
    }
    fn validate_input(&self, input: &Input) -> Result<(), ModelError> {
        if !input.x.is_finite()
            || !input.scale.is_finite()
            || !(0.0..=0.8).contains(&input.x)
            || !(0.0..=1.0).contains(&input.scale)
        {
            return Err(ModelError::new(
                ErrorKind::OutOfDomain,
                "fixture scientific domain: x <= 0.8",
            ));
        }
        Ok(())
    }
    fn run(&self, input: &Input) -> Result<Output, ModelError> {
        self.validate_input(input)?;
        Ok(Output {
            value: self.output_override.unwrap_or(input.x * input.scale),
        })
    }
    fn validate_output(&self, output: &Output) -> Result<(), ModelError> {
        if !output.value.is_finite() {
            return Err(ModelError::new(
                ErrorKind::Internal,
                "non-finite fixture output",
            ));
        }
        Ok(())
    }
}

fn runtime(fixture: Fixture, backend: Backend) -> Runtime {
    let catalog = Catalog::new(vec![Box::new(TypedAdapter::new(fixture).unwrap())]).unwrap();
    Runtime::new(
        catalog,
        BuildInfo {
            git_revision: Some("test-revision".into()),
            build_id: Some("test-build".into()),
            ..BuildInfo::default()
        },
        backend,
    )
}

const ID: &str = "test.engineering-fixture";

#[test]
fn run_records_effective_defaults_and_version_provenance() {
    let result = runtime(Fixture::default(), Backend::Native)
        .run(ID, &json!({"x": 0.5}))
        .unwrap();
    assert_eq!(result.result, json!({"value": 0.5}));
    assert_eq!(result.manifest.parameters, json!({"x": 0.5, "scale": 1.0}));
    assert_eq!(result.manifest.model_id, ID);
    assert_eq!(
        result.manifest.build.git_revision.as_deref(),
        Some("test-revision")
    );
    assert_eq!(result.manifest.implementation_version, "0.1.0");
    assert!(result.manifest.validation.is_empty());
    let restored: rsphysics_runtime::ResultEnvelope =
        serde_json::from_value(serde_json::to_value(&result).unwrap()).unwrap();
    assert_eq!(restored.result, result.result);
}

#[test]
fn schemas_are_generated_from_types_and_reject_bad_shapes_and_ranges() {
    let runtime = runtime(Fixture::default(), Backend::Native);
    let descriptor = runtime.describe(ID).unwrap();
    assert_eq!(descriptor.input_schema["properties"]["x"]["maximum"], 1.0);
    assert_eq!(
        descriptor.output_schema["properties"]["value"]["type"],
        "number"
    );
    for input in [
        json!({}),
        json!({"x": "0.5"}),
        json!({"x": -1}),
        json!({"x": 2}),
        json!({"x": 0.5, "typo": 1}),
    ] {
        assert_eq!(
            runtime.run(ID, &input).unwrap_err().kind,
            ErrorKind::InvalidParameter
        );
    }
    assert_eq!(
        runtime.run(ID, &json!({"x": 0.9})).unwrap_err().kind,
        ErrorKind::OutOfDomain
    );
}

#[test]
fn invalid_and_nonfinite_outputs_never_become_successful_results() {
    for value in [f64::NAN, f64::INFINITY, 2.0] {
        let runtime = runtime(
            Fixture {
                output_override: Some(value),
                ..Fixture::default()
            },
            Backend::Native,
        );
        assert_eq!(
            runtime.run(ID, &json!({"x": 0.5})).unwrap_err().kind,
            ErrorKind::Internal
        );
    }
}

#[test]
fn batches_preserve_order_and_identify_the_first_failed_row() {
    let runtime = runtime(Fixture::default(), Backend::Native);
    let result = runtime
        .run_batch(ID, &[json!({"x": 0.2}), json!({"x": 0.6})])
        .unwrap();
    assert_eq!(result[0].result["value"], 0.2);
    assert_eq!(result[1].result["value"], 0.6);
    assert!(runtime.run_batch(ID, &[]).unwrap().is_empty());
    let error = runtime
        .run_batch(ID, &[json!({"x": 0.2}), json!({"x": 0.9})])
        .unwrap_err();
    assert_eq!(error.kind, ErrorKind::OutOfDomain);
    assert!(error.message.contains("row 1"));
}

#[test]
fn capabilities_are_checked_even_for_empty_batches() {
    let python = runtime(Fixture::default(), Backend::Python);
    assert_eq!(
        python.run(ID, &json!({"x": 0.5})).unwrap_err().kind,
        ErrorKind::BackendUnavailable
    );
    assert_eq!(
        python.run_batch(ID, &[]).unwrap_err().kind,
        ErrorKind::BackendUnavailable
    );
    let no_batch = runtime(
        Fixture {
            batch: false,
            ..Fixture::default()
        },
        Backend::Native,
    );
    assert_eq!(
        no_batch.run_batch(ID, &[]).unwrap_err().kind,
        ErrorKind::BackendUnavailable
    );
}

#[test]
fn official_catalog_excludes_fixtures_and_unknown_ids_are_structured_errors() {
    let runtime = Runtime::new(
        official_catalog().unwrap(),
        BuildInfo::default(),
        Backend::Native,
    );
    assert!(
        runtime
            .list_models()
            .iter()
            .all(|d| !d.metadata.id.as_str().starts_with("test.")
                && !d.metadata.id.as_str().starts_with("template."))
    );
    assert_eq!(
        runtime.describe("test.unavailable").unwrap_err().kind,
        ErrorKind::UnknownModel
    );
}

#[test]
fn duplicate_ids_cannot_silently_replace_a_model() {
    let models: Vec<Box<dyn ModelAdapter>> = vec![
        Box::new(TypedAdapter::new(Fixture::default()).unwrap()),
        Box::new(TypedAdapter::new(Fixture::default()).unwrap()),
    ];
    assert!(Catalog::new(models).is_err());
}

#[test]
fn model_ids_and_versions_are_checked_at_boundaries() {
    for id in [
        "",
        "prospect",
        "Vegetation.prospect",
        "a..b",
        "a.b-",
        "a.1b",
        "a.b/c",
        "a.中文",
    ] {
        assert!(ModelId::new(id).is_err());
    }
    assert!(ModelId::new("vegetation.prospect-d").is_ok());
    assert!(serde_json::from_str::<ModelId>(r#""bad""#).is_err());
    let mut metadata = metadata();
    metadata.implementation_version = "1".into();
    assert!(metadata.validate().is_err());
    metadata.implementation_version = "0.1.0".into();
    metadata.api_version = "999".into();
    assert!(metadata.validate().is_err());
}

#[test]
fn validation_status_requires_matching_versioned_backend_evidence() {
    let mut metadata = metadata();
    metadata.status = ValidationStatus::ReferenceValidated;
    assert!(metadata.validate().is_err());
    metadata.validation.push(ValidationEvidence {
        kind: ValidationKind::Implementation,
        suite: "engineering-fixture-v1".into(),
        implementation_version: "0.0.1".into(),
        backend: Backend::Native,
    });
    assert!(metadata.validate().is_err());
    metadata.validation[0].implementation_version = "0.1.0".into();
    assert!(metadata.validate().is_ok());
    metadata.validation[0].backend = Backend::Wasm;
    assert!(metadata.validate().is_err());
    metadata.validation[0].backend = Backend::Native;
    metadata.status = ValidationStatus::BenchmarkEvaluated;
    assert!(metadata.validate().is_err());
}

#[test]
fn malformed_hashes_and_incompatible_physical_units_are_rejected() {
    let mut metadata = metadata();
    metadata
        .data_hashes
        .insert("coefficients".into(), "sha256:123".into());
    assert!(metadata.validate().is_err());
    metadata
        .data_hashes
        .insert("coefficients".into(), format!("sha256:{}", "a".repeat(64)));
    assert!(metadata.validate().is_ok());
    metadata.output_semantics.insert(
        "/value".into(),
        FieldSemantics {
            unit: "1".into(),
            quantity: Some(rsphysics_model_api::serde_json::from_value(json!("brdf")).unwrap()),
            axes: vec![],
            description: "incorrect BRDF unit".into(),
        },
    );
    assert!(metadata.validate().is_err());
}

#[test]
fn template_manifest_is_valid_but_unknown_fields_and_fake_validation_are_rejected() {
    let text = include_str!("../../../templates/rust-model/model.toml");
    assert!(parse_model_manifest(text).is_ok());
    assert!(
        parse_model_manifest(&text.replace("native = true", "native = true\nnatvie = true"))
            .is_err()
    );
    assert!(parse_model_manifest(&text.replace("experimental", "reference-validated")).is_err());
}
