//! Tables from the formatted 6SV2.1 auxiliary products.
use crate::output::finite_number;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AerosolLut {
    pub wavelength_bounds_um: [f64; 2],
    pub aot_band: Option<f64>,
    pub aot550: Option<f64>,
    pub aerosol_model: String,
    pub rows: Vec<AerosolLutRow>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AerosolLutRow {
    pub solar_zenith_deg: Option<f64>,
    pub view_zenith_deg: Option<f64>,
    pub scattering_angles_deg: Vec<Option<f64>>,
    pub intrinsic_reflectance: Vec<Option<f64>>,
}

pub(crate) fn parse_lut(text: &str) -> Option<AerosolLut> {
    let mut lines = text.lines();
    let bounds = lines
        .next()?
        .trim()
        .strip_prefix("AERO-LUT Lambda min,max")?;
    let mut wavelengths = bounds.split_whitespace();
    let wavelength_bounds_um = [
        finite_number(wavelengths.next()?)?,
        finite_number(wavelengths.next()?)?,
    ];
    let loading = lines
        .next()?
        .trim()
        .strip_prefix("Tau-Lambda,Tau550 asol")?;
    let mut loading = loading.split_whitespace();
    let aot_band = finite_number(loading.next()?);
    let aot550 = finite_number(loading.next()?);
    let aerosol_model = lines
        .next()?
        .trim()
        .strip_prefix("aerosol model")?
        .trim()
        .to_owned();
    let mut rows = Vec::new();
    while let Some(header) = lines.next() {
        // The final fields have no delimiter (I3,F10.5,F10.5).
        let count: usize = header.get(22..25)?.trim().parse().ok()?;
        if count == 0 {
            return None;
        }
        let solar_zenith_deg = finite_number(header.get(..10)?);
        let view_zenith_deg = finite_number(header.get(11..21)?);
        let max_angle = finite_number(header.get(25..35)?);
        let min_angle = finite_number(header.get(35..45)?);
        let scattering_angles_deg = (0..count)
            .map(|i| {
                if i + 1 == count {
                    min_angle
                } else {
                    max_angle.map(|angle| angle - 4.0 * i as f64)
                }
            })
            .collect();
        // The Fortran format wraps rows at 41 samples.
        let mut intrinsic_reflectance = Vec::with_capacity(count);
        for _ in 0..count.div_ceil(41) {
            intrinsic_reflectance.extend(lines.next()?.split_whitespace().map(finite_number));
        }
        if intrinsic_reflectance.len() != count {
            return None;
        }
        rows.push(AerosolLutRow {
            solar_zenith_deg,
            view_zenith_deg,
            scattering_angles_deg,
            intrinsic_reflectance,
        });
    }
    (!rows.is_empty()).then_some(AerosolLut {
        wavelength_bounds_um,
        aot_band,
        aot550,
        aerosol_model,
        rows,
    })
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct MieTable {
    pub wavelengths_um: Vec<f64>,
    pub normalized_extinction: Vec<Option<f64>>,
    pub normalized_scattering: Vec<Option<f64>>,
    pub single_scattering_albedo: Vec<Option<f64>>,
    pub asymmetry: Vec<Option<f64>>,
    pub extinction: Vec<Option<f64>>,
    pub scattering: Vec<Option<f64>>,
    pub scattering_angles_deg: Vec<f64>,
    pub phase_i: Vec<Vec<Option<f64>>>,
    pub phase_q: Option<Vec<Vec<Option<f64>>>>,
    pub phase_u: Option<Vec<Vec<Option<f64>>>>,
}

pub(crate) fn parse_mie(text: &str) -> Option<MieTable> {
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let count: usize = lines.next()?.trim().parse().ok()?;
    if count == 0 || !lines.next()?.contains("Nor_Ext_Co") {
        return None;
    }
    let mut table = MieTable {
        wavelengths_um: Vec::new(),
        normalized_extinction: Vec::new(),
        normalized_scattering: Vec::new(),
        single_scattering_albedo: Vec::new(),
        asymmetry: Vec::new(),
        extinction: Vec::new(),
        scattering: Vec::new(),
        scattering_angles_deg: Vec::new(),
        phase_i: Vec::new(),
        phase_q: None,
        phase_u: None,
    };
    for _ in 0..20 {
        let values = lines
            .next()?
            .split_whitespace()
            .map(finite_number)
            .collect::<Vec<_>>();
        let [wavelength, ext, sca, albedo, asym, raw_ext, raw_sca] = values.try_into().ok()?;
        table.wavelengths_um.push(wavelength?);
        table.normalized_extinction.push(ext);
        table.normalized_scattering.push(sca);
        table.single_scattering_albedo.push(albedo);
        table.asymmetry.push(asym);
        table.extinction.push(raw_ext);
        table.scattering.push(raw_sca);
    }
    if !lines.next()?.contains("Phase Function") || !lines.next()?.trim().starts_with("TETA") {
        return None;
    }
    for line in lines {
        let mut columns = line.split_whitespace();
        let angle = finite_number(columns.next()?)?;
        let values = columns.map(finite_number).collect::<Vec<_>>();
        if values.len() != 20 {
            return None;
        }
        if table.phase_i.len() < count {
            table.scattering_angles_deg.push(angle);
            table.phase_i.push(values);
        } else if table.phase_q.as_ref().map_or(0, Vec::len) < count {
            table.phase_q.get_or_insert_with(Vec::new).push(values);
        } else {
            table.phase_u.get_or_insert_with(Vec::new).push(values);
        }
    }
    let full = |rows: &Option<Vec<_>>| rows.as_ref().is_some_and(|rows| rows.len() == count);
    (table.phase_i.len() == count
        && ((table.phase_q.is_none() && table.phase_u.is_none())
            || (full(&table.phase_q) && full(&table.phase_u))))
    .then_some(table)
}
