//! Typed scientific API and object-safe, schema-validated boundary.

mod adapter;
mod manifest;
mod metadata;

pub use adapter::{AdapterResult, ModelAdapter, PhysicsModel, TypedAdapter};
pub use manifest::parse_model_manifest;
pub use metadata::*;
pub use schemars;
pub use serde;
pub use serde_json;

use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

pub const API_VERSION: &str = "0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    InvalidParameter,
    OutOfDomain,
    MissingData,
    BackendUnavailable,
    NonConvergence,
    UnknownModel,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelError {
    pub kind: ErrorKind,
    pub message: String,
}

impl ModelError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl Error for ModelError {}
