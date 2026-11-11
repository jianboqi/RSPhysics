use crate::products::{AerosolLut, MieTable, parse_lut, parse_mie};
use rsphysics_model_api::{ErrorKind, ModelError};
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::BTreeMap;

const SPECTRAL_COLUMNS: [&str; 11] = [
    "wavelength_um",
    "gas_transmittance",
    "downward_scattering_transmittance",
    "upward_scattering_transmittance",
    "spherical_albedo",
    "intrinsic_reflectance",
    "solar_irradiance_w_m2_um",
    "target_reflectance",
    "integration_weight",
    "earth_sun_irradiance_factor",
    "apparent_reflectance",
];

#[derive(Debug, Clone, Default, Serialize, JsonSchema)]
pub struct Diagnostics {
    pub direct_solar_irradiance_w_m2_um: Option<f64>,
    pub diffuse_solar_irradiance_w_m2_um: Option<f64>,
    pub environmental_irradiance_w_m2_um: Option<f64>,
    pub direct_irradiance_fraction: Option<f64>,
    pub diffuse_irradiance_fraction: Option<f64>,
    pub environmental_irradiance_fraction: Option<f64>,
    pub atmospheric_intrinsic_radiance_w_m2_sr_um: Option<f64>,
    pub background_radiance_w_m2_sr_um: Option<f64>,
    pub pixel_radiance_w_m2_sr_um: Option<f64>,
    pub atmospheric_intrinsic_reflectance: Option<f64>,
    pub background_reflectance: Option<f64>,
    pub pixel_reflectance: Option<f64>,
    pub apparent_polarized_reflectance: Option<f64>,
    pub apparent_polarized_radiance_w_m2_sr_um: Option<f64>,
    pub plane_of_polarization_deg: Option<f64>,
    pub polarization_ratio: Option<f64>,
    pub total_gaseous_transmittance: Option<f64>,
}

impl Diagnostics {
    fn from_report(report: &str) -> Self {
        let mut d = Self::default();
        let mut lines = report.lines();
        while let Some(line) = lines.next() {
            let columns = match line.trim().trim_start_matches('*').trim_start() {
                text if text.starts_with("% of irradiance at ground level") => [
                    &mut d.direct_irradiance_fraction,
                    &mut d.diffuse_irradiance_fraction,
                    &mut d.environmental_irradiance_fraction,
                ],
                text if text.starts_with("irr. at ground level") => [
                    &mut d.direct_solar_irradiance_w_m2_um,
                    &mut d.diffuse_solar_irradiance_w_m2_um,
                    &mut d.environmental_irradiance_w_m2_um,
                ],
                text if text.starts_with("rad at satel. level") => [
                    &mut d.atmospheric_intrinsic_radiance_w_m2_sr_um,
                    &mut d.background_radiance_w_m2_sr_um,
                    &mut d.pixel_radiance_w_m2_sr_um,
                ],
                text if text.starts_with("reflectance at satellite level") => [
                    &mut d.atmospheric_intrinsic_reflectance,
                    &mut d.background_reflectance,
                    &mut d.pixel_reflectance,
                ],
                text if text.starts_with("app. polarized refl.") => {
                    if let Some((left, right)) = text.split_once("app. pol. rad.") {
                        d.apparent_polarized_reflectance = last_number(left);
                        d.apparent_polarized_radiance_w_m2_sr_um = last_number(right);
                    }
                    continue;
                }
                text if text.starts_with("direction of the plane of polarization") => {
                    d.plane_of_polarization_deg = last_number(text);
                    continue;
                }
                text if text.starts_with("total polarization ratio") => {
                    d.polarization_ratio = last_number(text);
                    continue;
                }
                text if text.starts_with("total gaseous transmittance") => {
                    d.total_gaseous_transmittance = last_number(text);
                    continue;
                }
                _ => continue,
            };
            if let Some(values) = lines.nth(1) {
                // The three fixed-width columns may contain Fortran overflow stars.
                if let Some(values) = values.as_bytes().get(1..61) {
                    for (field, column) in columns.into_iter().zip(values.chunks_exact(20)) {
                        *field = std::str::from_utf8(column).ok().and_then(finite_number);
                    }
                }
            }
        }
        d
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Output {
    pub apparent_reflectance: f64,
    pub apparent_radiance_w_m2_sr_um: f64,
    pub quantities: BTreeMap<String, Vec<Option<f64>>>,
    pub diagnostics: Diagnostics,
    pub spectral_columns: [&'static str; 11],
    pub spectral_rows: Vec<[Option<f64>; 11]>,
    pub aerosol_lut: Option<AerosolLut>,
    pub mie_tables: BTreeMap<String, MieTable>,
    pub report: String,
    pub files: BTreeMap<String, String>,
}
pub(crate) fn finite_number(value: &str) -> Option<f64> {
    let value = value.trim().replace(['D', 'd'], "E").parse::<f64>().ok()?;
    value.is_finite().then_some(value)
}
fn last_number(text: &str) -> Option<f64> {
    finite_number(
        text.trim_end_matches(['*', ' '])
            .split_whitespace()
            .last()?,
    )
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
        if spectral {
            let row = line.strip_prefix('*').and_then(|fields| {
                let values =
                    std::array::from_fn(|i| fields.get(i * 7..i * 7 + 6).and_then(finite_number));
                values[0].filter(|wavelength| (0.25..=4.0).contains(wavelength))?;
                Some(values)
            });
            if let Some(row) = row {
                spectral_rows.push(row);
            } else if !v.is_empty() {
                spectral = false;
            }
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
        diagnostics: Diagnostics::from_report(&report),
        spectral_columns: SPECTRAL_COLUMNS,
        spectral_rows,
        aerosol_lut: files.get("rotoa_bs").and_then(|text| parse_lut(text)),
        mie_tables: files
            .iter()
            .filter_map(|(name, text)| {
                (name.to_ascii_lowercase().ends_with(".mie"))
                    .then(|| parse_mie(text))
                    .flatten()
                    .map(|table| (name.clone(), table))
            })
            .collect(),
        report,
        files,
    };
    output.validate()?;
    Ok(output)
}
