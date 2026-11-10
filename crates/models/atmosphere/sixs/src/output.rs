use rsphysics_model_api::{ErrorKind, ModelError};
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Output {
    pub apparent_reflectance: f64,
    pub apparent_radiance_w_m2_sr_um: f64,
    pub quantities: BTreeMap<String, Vec<Option<f64>>>,
    pub spectral_rows: Vec<Vec<f64>>,
    pub report: String,
    pub files: BTreeMap<String, String>,
}
fn numbers(line: &str) -> Vec<f64> {
    line.split_whitespace()
        .filter_map(|x| {
            x.trim_matches(['*', ','])
                .replace(['D', 'd'], "E")
                .parse()
                .ok()
        })
        .collect()
}
fn key(label: &str) -> String {
    label
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}
impl Output {
    pub fn validate(&self) -> Result<(), ModelError> {
        if !self.apparent_reflectance.is_finite() || !self.apparent_radiance_w_m2_sr_um.is_finite()
        {
            return Err(ModelError::new(
                ErrorKind::NonConvergence,
                "6SV2.1 returned a non-finite apparent signal",
            ));
        }
        Ok(())
    }
}
pub fn parse_report(report: String, files: BTreeMap<String, String>) -> Result<Output, ModelError> {
    let mut quantities = BTreeMap::new();
    let mut reflectance = None;
    let mut radiance = None;
    let mut spectral_rows = vec![];
    let mut spectral = false;
    for line in report.lines() {
        let text = line.trim().trim_matches('*').trim();
        let v = numbers(text);
        if text.contains("apparent reflectance") && v.len() >= 2 {
            reflectance = Some(v[0]);
            radiance = Some(v[1]);
        }
        if let Some((label, values)) = text.split_once(':') {
            let n = numbers(values);
            if !n.is_empty() {
                quantities.insert(
                    key(label),
                    n.into_iter().map(|x| x.is_finite().then_some(x)).collect(),
                );
            }
        }
        if text.contains("trans  down   up") {
            spectral = true;
            continue;
        }
        if spectral && v.len() >= 10 {
            spectral_rows.push(v);
        } else if spectral && !v.is_empty() {
            spectral = false;
        }
    }
    let missing = || {
        ModelError::new(
            ErrorKind::NonConvergence,
            format!(
                "6SV2.1 did not produce a complete report: {}",
                report.chars().take(500).collect::<String>()
            ),
        )
    };
    if !report.contains("global gas. trans.") {
        return Err(missing());
    }
    let output = Output {
        apparent_reflectance: reflectance.ok_or_else(missing)?,
        apparent_radiance_w_m2_sr_um: radiance.ok_or_else(missing)?,
        quantities,
        spectral_rows,
        report,
        files,
    };
    output.validate()?;
    Ok(output)
}
