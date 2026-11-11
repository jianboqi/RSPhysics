use rsphysics_core::SpectralAxis;
use rsphysics_model_api::{ErrorKind, ModelError};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LeafSpectrum {
    pub reflectance: Vec<f64>,
    pub transmittance: Vec<f64>,
}

/// PROSPECT 3.01 coefficient arrays, all aligned to the supplied wavelength grid.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Prospect301Coefficients {
    pub refractive_index: Vec<f64>,
    pub albino_absorption: Vec<f64>,
    pub chlorophyll_absorption: Vec<f64>,
    pub water_absorption: Vec<f64>,
    pub dry_matter_absorption: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SpectralInputs {
    /// Wavelengths in nm, strictly increasing, from 400 to 2500.
    pub wavelengths_nm: SpectralAxis,
    pub soil_reflectance: Option<Vec<f64>>,
    pub crown_leaf: Option<LeafSpectrum>,
    pub understory_leaf: Option<LeafSpectrum>,
    pub prospect_301_coefficients: Option<Prospect301Coefficients>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    #[schemars(range(min = 0.0, max = 30.0), extend("x-unit" = "m² m⁻²"))]
    pub crown_lai: f64,
    #[schemars(range(min = 0.0, max = 30.0), extend("x-unit" = "m² m⁻²"))]
    pub understory_lai: f64,
    #[schemars(range(min = 0.0, max = 100_000.0), extend("x-unit" = "ha⁻¹"))]
    pub stem_density_per_ha: f64,
    #[schemars(range(min = 0.0, max = 150.0), extend("x-unit" = "m"))]
    pub tree_height_m: f64,
    #[schemars(range(min = 0.0, max = 100.0), extend("x-unit" = "m"))]
    pub crown_diameter_m: f64,
    #[schemars(range(min = 0.0, max = 90.0), extend("x-unit" = "deg"))]
    pub crown_mean_leaf_angle_deg: f64,
    #[schemars(range(min = 0.0, max = 90.0), extend("x-unit" = "deg"))]
    pub understory_mean_leaf_angle_deg: f64,
    #[schemars(range(min = 0.0, max = 1.0))]
    pub crown_hotspot: f64,
    #[schemars(range(min = 0.0, max = 1.0))]
    pub understory_hotspot: f64,
    #[schemars(range(min = 0.0, max = 89.9), extend("x-unit" = "deg"))]
    pub solar_zenith_deg: f64,
    #[schemars(range(min = 0.0, max = 89.9), extend("x-unit" = "deg"))]
    pub view_zenith_deg: f64,
    #[schemars(range(min = 0.0, max = 180.0), extend("x-unit" = "deg"))]
    pub relative_azimuth_deg: f64,
    #[schemars(range(min = 0.0, max = 1.0))]
    pub diffuse_fraction: f64,
    #[schemars(range(min = 0.0, max = 1.0))]
    pub soil_reflectance: f64,
    /// Multiplier for either the scalar or the spectral soil reflectance.
    #[schemars(range(min = 0.0, max = 10.0))]
    pub soil_brightness: f64,
    #[serde(deserialize_with = "crown_leaf")]
    #[schemars(with = "rsphysics_prospect_d::Input")]
    pub crown_leaf: rsphysics_prospect_d::Input,
    #[serde(deserialize_with = "understory_leaf")]
    #[schemars(with = "rsphysics_prospect_d::Input")]
    pub understory_leaf: rsphysics_prospect_d::Input,
    pub spectra: Option<SpectralInputs>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            crown_lai: 7.,
            understory_lai: 0.1,
            stem_density_per_ha: 650.,
            tree_height_m: 20.,
            crown_diameter_m: 4.5,
            crown_mean_leaf_angle_deg: 55.,
            understory_mean_leaf_angle_deg: 45.,
            crown_hotspot: 0.04,
            understory_hotspot: 0.,
            solar_zenith_deg: 30.,
            view_zenith_deg: 0.,
            relative_azimuth_deg: 0.,
            diffuse_fraction: 0.1,
            soil_reflectance: 0.2,
            soil_brightness: 1.,
            crown_leaf: default_leaf(60.),
            understory_leaf: default_leaf(50.),
            spectra: None,
        }
    }
}

fn default_leaf(chlorophyll: f64) -> rsphysics_prospect_d::Input {
    rsphysics_prospect_d::Input {
        n: 2.,
        chlorophyll,
        water: 0.025,
        dry_matter: 0.025,
        ..Default::default()
    }
}

fn crown_leaf<'de, D: Deserializer<'de>>(d: D) -> Result<rsphysics_prospect_d::Input, D::Error> {
    leaf_with_defaults(d, 60.)
}

fn understory_leaf<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<rsphysics_prospect_d::Input, D::Error> {
    leaf_with_defaults(d, 50.)
}

fn leaf_with_defaults<'de, D: Deserializer<'de>>(
    d: D,
    chlorophyll: f64,
) -> Result<rsphysics_prospect_d::Input, D::Error> {
    let mut leaf = default_leaf(chlorophyll);
    for (key, value) in BTreeMap::<String, f64>::deserialize(d)? {
        match key.as_str() {
            "n" => leaf.n = value,
            "chlorophyll" => leaf.chlorophyll = value,
            "carotenoids" => leaf.carotenoids = value,
            "anthocyanins" => leaf.anthocyanins = value,
            "brown_pigments" => leaf.brown_pigments = value,
            "water" => leaf.water = value,
            "dry_matter" => leaf.dry_matter = value,
            "incidence_angle_deg" => leaf.incidence_angle_deg = value,
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "unknown leaf parameter {key}"
                )));
            }
        }
    }
    Ok(leaf)
}

impl Input {
    pub fn validate(&self) -> Result<(), ModelError> {
        for (name, value, max) in [
            ("crown_lai", self.crown_lai, 30.),
            ("understory_lai", self.understory_lai, 30.),
            ("stem_density_per_ha", self.stem_density_per_ha, 100_000.),
            ("tree_height_m", self.tree_height_m, 150.),
            ("crown_diameter_m", self.crown_diameter_m, 100.),
            (
                "crown_mean_leaf_angle_deg",
                self.crown_mean_leaf_angle_deg,
                90.,
            ),
            (
                "understory_mean_leaf_angle_deg",
                self.understory_mean_leaf_angle_deg,
                90.,
            ),
            ("crown_hotspot", self.crown_hotspot, 1.),
            ("understory_hotspot", self.understory_hotspot, 1.),
            ("solar_zenith_deg", self.solar_zenith_deg, 89.9),
            ("view_zenith_deg", self.view_zenith_deg, 89.9),
            ("relative_azimuth_deg", self.relative_azimuth_deg, 180.),
            ("diffuse_fraction", self.diffuse_fraction, 1.),
            ("soil_reflectance", self.soil_reflectance, 1.),
            ("soil_brightness", self.soil_brightness, 10.),
        ] {
            if !(0.0..=max).contains(&value) {
                return Err(domain(format!("{name} must be in [0, {max}]")));
            }
        }
        self.crown_leaf.validate()?;
        self.understory_leaf.validate()?;
        let Some(spectra) = &self.spectra else {
            return check_scaled_soil(&[self.soil_reflectance], self.soil_brightness);
        };
        let axis = spectra.wavelengths_nm.wavelengths_nm();
        if axis[0] < 400. || axis[axis.len() - 1] > 2500. {
            return Err(domain("wavelengths_nm must be within [400, 2500]"));
        }
        let len = axis.len();
        if let Some(soil) = &spectra.soil_reflectance {
            check_array("soil_reflectance", soil, len, |x| (0.0..=1.0).contains(&x))?;
            check_scaled_soil(soil, self.soil_brightness)?;
        } else {
            check_scaled_soil(&[self.soil_reflectance], self.soil_brightness)?;
        }
        for (name, leaf) in [
            ("crown_leaf", &spectra.crown_leaf),
            ("understory_leaf", &spectra.understory_leaf),
        ] {
            if let Some(leaf) = leaf {
                check_array(name, &leaf.reflectance, len, |x| (0.0..=1.0).contains(&x))?;
                check_array(name, &leaf.transmittance, len, |x| (0.0..=1.0).contains(&x))?;
                if leaf
                    .reflectance
                    .iter()
                    .zip(&leaf.transmittance)
                    .any(|(r, t)| r + t > 1.0 + 1e-12)
                {
                    return Err(domain(format!(
                        "{name} reflectance plus transmittance must not exceed one"
                    )));
                }
            }
        }
        if let Some(c) = &spectra.prospect_301_coefficients {
            check_array("refractive_index", &c.refractive_index, len, |x| {
                (1.01..=3.).contains(&x)
            })?;
            for (name, values) in [
                ("albino_absorption", &c.albino_absorption),
                ("chlorophyll_absorption", &c.chlorophyll_absorption),
                ("water_absorption", &c.water_absorption),
                ("dry_matter_absorption", &c.dry_matter_absorption),
            ] {
                check_array(name, values, len, |x| x.is_finite() && x >= 0.)?;
            }
        }
        Ok(())
    }
}

fn domain(message: impl Into<String>) -> ModelError {
    ModelError::new(ErrorKind::OutOfDomain, message)
}

fn check_array(
    name: &str,
    values: &[f64],
    len: usize,
    valid: impl Fn(f64) -> bool,
) -> Result<(), ModelError> {
    if values.len() != len {
        return Err(ModelError::new(
            ErrorKind::InvalidParameter,
            format!("{name} must have {len} values"),
        ));
    }
    if values.iter().copied().any(|x| !valid(x)) {
        return Err(domain(format!("{name} has invalid values")));
    }
    Ok(())
}

fn check_scaled_soil(soil: &[f64], brightness: f64) -> Result<(), ModelError> {
    if soil.iter().any(|r| r * brightness > 1.) {
        return Err(domain("soil reflectance after scaling must not exceed one"));
    }
    Ok(())
}
