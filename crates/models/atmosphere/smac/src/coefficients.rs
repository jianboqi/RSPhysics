use rsphysics_model_api::{ErrorKind, ModelError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::OnceLock};

pub const PACK_SHA256: &str = "6902f7518fbe12790b5f39bde35b6abd62ed5a939a3f960c5c3c43962e43d51b";
pub const DEFAULT_COEFFICIENT: &str = "sentinel-2a.msi.B04.continental";
#[derive(Debug, Deserialize, Serialize)]
pub struct Source {
    pub text: String,
    pub sha256: String,
    pub url: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Coefficients {
    pub id: String,
    pub satellite: String,
    pub band: String,
    pub aerosol: String,
    pub source_key: String,
    pub column: Option<usize>,
    #[serde(skip)]
    pub values: Vec<f64>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Library {
    pub format_version: u32,
    pub sources: BTreeMap<String, Source>,
    pub records: Vec<Coefficients>,
}
static LIBRARY: OnceLock<Result<Library, ModelError>> = OnceLock::new();
fn invalid(message: impl Into<String>) -> ModelError {
    ModelError::new(ErrorKind::MissingData, message)
}
fn load() -> Result<Library, ModelError> {
    let bytes = include_bytes!("../data/coefficients.json.zlib");
    if format!("{:x}", Sha256::digest(bytes)) != PACK_SHA256 {
        return Err(invalid("SMAC coefficient resource checksum mismatch"));
    }
    let json = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(bytes, 2_000_000)
        .map_err(|e| invalid(format!("invalid SMAC compressed resource: {e:?}")))?;
    let mut library: Library = serde_json::from_slice(&json).map_err(|e| invalid(e.to_string()))?;
    if library.format_version != 1 || library.records.len() != 367 {
        return Err(invalid("unsupported SMAC coefficient resource"));
    }
    for source in library.sources.values() {
        if format!("{:x}", Sha256::digest(source.text.as_bytes())) != source.sha256 {
            return Err(invalid("SMAC original coefficient checksum mismatch"));
        }
    }
    let mut previous = "".to_owned();
    for record in &mut library.records {
        if record.id <= previous {
            return Err(invalid("SMAC coefficient IDs must be sorted and unique"));
        }
        previous = record.id.clone();
        let source = library
            .sources
            .get(&record.source_key)
            .ok_or_else(|| invalid("missing SMAC source"))?;
        let rows: Result<Vec<Vec<f64>>, _> = source
            .text
            .lines()
            .filter(|s| !s.trim().is_empty() && !s.starts_with('#'))
            .map(|line| line.split_whitespace().map(str::parse::<f64>).collect())
            .collect();
        let rows = rows.map_err(|e| invalid(e.to_string()))?;
        record.values = if let Some(column) = record.column {
            if rows.len() != 49 || column >= 13 || rows.iter().any(|row| row.len() != 13) {
                return Err(invalid("invalid SMAC multi-band layout"));
            }
            rows.iter().map(|row| row[column]).collect()
        } else {
            rows.into_iter().flatten().collect()
        };
        if record.values.len() != 49 || record.values.iter().any(|v| !v.is_finite()) {
            return Err(invalid("invalid SMAC coefficients"));
        }
    }
    Ok(library)
}
pub fn library() -> Result<&'static Library, ModelError> {
    LIBRARY.get_or_init(load).as_ref().map_err(Clone::clone)
}
pub fn get(id: &str) -> Result<&'static Coefficients, ModelError> {
    let records = &library()?.records;
    records
        .binary_search_by(|c| c.id.as_str().cmp(id))
        .map(|i| &records[i])
        .map_err(|_| {
            ModelError::new(
                ErrorKind::InvalidParameter,
                format!("unknown SMAC coefficient_id: {id}; no implicit sensor substitution"),
            )
        })
}
pub fn coefficient_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    let records = &library().expect("verified embedded SMAC resource").records;
    schemars::json_schema!({"type":"string", "enum":records.iter().map(|r| &r.id).collect::<Vec<_>>(), "x-options":records.iter().map(|r| serde_json::json!({"value":r.id,"label":format!("{} · {}",r.band,r.aerosol),"group":r.satellite})).collect::<Vec<_>>()})
}
