use crate::{ErrorKind, ModelDescriptor, ModelError, ModelMetadata};
use schemars::{JsonSchema, schema_for};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

/// Implementations must also validate scientific domain constraints when called directly.
/// JSON Schema validates structure/ranges at the dynamic boundary; cross-field physics
/// belongs in `validate_input`. `validate_output` checks physical/shape/finite invariants.
pub trait PhysicsModel: Send + Sync + 'static {
    type Input: DeserializeOwned + Serialize + JsonSchema;
    type Output: Serialize + JsonSchema;

    fn metadata(&self) -> ModelMetadata;
    fn validate_input(&self, input: &Self::Input) -> Result<(), ModelError>;
    fn run(&self, input: &Self::Input) -> Result<Self::Output, ModelError>;
    fn validate_output(&self, output: &Self::Output) -> Result<(), ModelError>;
}

/// Object-safe interface for a statically linked heterogeneous catalog.
pub trait ModelAdapter: Send + Sync {
    fn descriptor(&self) -> &ModelDescriptor;
    fn run_json(&self, parameters: &Value) -> Result<AdapterResult, ModelError>;
}

pub struct AdapterResult {
    pub result: Value,
    /// Typed input serialized after defaults/unit conversions are applied.
    pub effective_parameters: Value,
}

pub struct TypedAdapter<M> {
    model: M,
    descriptor: ModelDescriptor,
    input_validator: jsonschema::Validator,
    output_validator: jsonschema::Validator,
}

impl<M: PhysicsModel> TypedAdapter<M> {
    /// Schema validators are compiled once, not for every sample in a batch.
    pub fn new(model: M) -> Result<Self, ModelError> {
        let metadata = model.metadata();
        metadata.validate()?;
        let internal = |e: String| ModelError::new(ErrorKind::Internal, e);
        let input_schema =
            serde_json::to_value(schema_for!(M::Input)).map_err(|e| internal(e.to_string()))?;
        let output_schema =
            serde_json::to_value(schema_for!(M::Output)).map_err(|e| internal(e.to_string()))?;
        let input_validator =
            jsonschema::validator_for(&input_schema).map_err(|e| internal(e.to_string()))?;
        let output_validator =
            jsonschema::validator_for(&output_schema).map_err(|e| internal(e.to_string()))?;
        Ok(Self {
            model,
            descriptor: ModelDescriptor {
                metadata,
                input_schema,
                output_schema,
            },
            input_validator,
            output_validator,
        })
    }
}

impl<M: PhysicsModel> ModelAdapter for TypedAdapter<M> {
    fn descriptor(&self) -> &ModelDescriptor {
        &self.descriptor
    }

    fn run_json(&self, parameters: &Value) -> Result<AdapterResult, ModelError> {
        self.input_validator
            .validate(parameters)
            .map_err(|e| ModelError::new(ErrorKind::InvalidParameter, e.to_string()))?;
        let input = serde_json::from_value(parameters.clone())
            .map_err(|e| ModelError::new(ErrorKind::InvalidParameter, e.to_string()))?;
        self.model.validate_input(&input)?;
        let effective_parameters = serde_json::to_value(&input)
            .map_err(|e| ModelError::new(ErrorKind::Internal, e.to_string()))?;
        let output = self.model.run(&input)?;
        self.model.validate_output(&output).map_err(|e| {
            ModelError::new(ErrorKind::Internal, format!("invalid model output: {e}"))
        })?;
        let value = serde_json::to_value(output)
            .map_err(|e| ModelError::new(ErrorKind::Internal, e.to_string()))?;
        self.output_validator.validate(&value).map_err(|e| {
            ModelError::new(ErrorKind::Internal, format!("output schema violation: {e}"))
        })?;
        Ok(AdapterResult {
            result: value,
            effective_parameters,
        })
    }
}
