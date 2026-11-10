use rsphysics_model_api::{ErrorKind, ModelError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
fn domain(s: &str) -> ModelError {
    ModelError::new(ErrorKind::OutOfDomain, s)
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Roughness {
    #[default]
    None,
    HapkeSingle,
    HapkeMulti,
}
fn roughness_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({"type":"string","enum":["none","hapke_single","hapke_multi"]})
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    /// Soil spectral brightness multiplier.
    #[schemars(range(min=0.0,max=1.5),extend("x-unit"="1"))]
    pub soil_brightness: f64,
    /// First spherical mixing angle of the soil spectral basis.
    #[schemars(range(min=-90.0,max=90.0),extend("x-unit"="deg"))]
    pub spectral_alpha_deg: f64,
    /// Second spherical mixing angle of the soil spectral basis.
    #[schemars(range(min=0.0,max=90.0),extend("x-unit"="deg"))]
    pub spectral_gamma_deg: f64,
    /// Water-film covered surface fraction.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub water_coverage: f64,
    /// Water-film thickness in millimetres.
    #[schemars(range(min=0.0,max=10.0),extend("x-unit"="mm"))]
    pub water_film_thickness_mm: f64,
    /// Particle volume fraction within the water film.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub particle_volume_fraction: f64,
    /// Real refractive index of particles in the film.
    #[schemars(range(min=1.0001,max=3.0),extend("x-unit"="1"))]
    pub particle_refractive_index: f64,
    /// Non-negative imaginary refractive index of film particles.
    #[schemars(range(min=0.0,max=0.1),extend("x-unit"="1"))]
    pub particle_extinction_coefficient: f64,
    /// Solar zenith relative to the local surface normal.
    #[schemars(range(min=0.0,max=85.0),extend("x-unit"="deg"))]
    pub solar_zenith_deg: f64,
    /// Viewing zenith relative to the local surface normal.
    #[schemars(range(min=0.0,max=85.0),extend("x-unit"="deg"))]
    pub view_zenith_deg: f64,
    /// Folded relative azimuth; equal zeniths and zero azimuth define the hotspot.
    #[schemars(range(min=0.0,max=180.0),extend("x-unit"="deg"))]
    pub relative_azimuth_deg: f64,
    /// Two-term Henyey-Greenstein asymmetry parameter b.
    #[schemars(range(min=0.0,max=0.9999),extend("x-unit"="1"))]
    pub phase_asymmetry: f64,
    /// Two-term phase mixing parameter c, subject to a joint non-negativity constraint.
    #[schemars(range(min=-0.9999,max=1.4999),extend("x-unit"="1"))]
    pub phase_backscatter: f64,
    /// Shadow-hiding hotspot amplitude B0.
    #[schemars(range(min=0.0,max=2.0),extend("x-unit"="1"))]
    pub hotspot_amplitude: f64,
    /// Positive hotspot width h.
    #[schemars(range(min=0.0001,max=2.0),extend("x-unit"="1"))]
    pub hotspot_width: f64,
    /// Solid filling factor F; porosity equals one minus F.
    #[schemars(range(min=0.0,max=0.75),extend("x-unit"="1"))]
    pub solid_filling_factor: f64,
    /// Dimensionless optical RMS slope; inactive with no roughness correction.
    #[schemars(range(min=0.0,max=2.0),extend("x-unit"="1"))]
    pub rms_slope: f64,
    /// Diffuse reflectance factor for Hapke multifacet roughness rescaling.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub multifacet_reflectance: f64,
    /// Macroscopic surface roughness operator.
    #[schemars(schema_with="roughness_schema",extend("x-unit"="1"))]
    pub roughness_correction: Roughness,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            soil_brightness: 0.6,
            spectral_alpha_deg: 15.0,
            spectral_gamma_deg: 50.0,
            water_coverage: 0.3,
            water_film_thickness_mm: 0.3,
            particle_volume_fraction: 0.05,
            particle_refractive_index: 1.53,
            particle_extinction_coefficient: 0.001,
            solar_zenith_deg: 45.0,
            view_zenith_deg: 0.0,
            relative_azimuth_deg: 0.0,
            phase_asymmetry: 0.4,
            phase_backscatter: 0.0,
            hotspot_amplitude: 0.4,
            hotspot_width: 0.1,
            solid_filling_factor: 0.2,
            rms_slope: 0.4,
            multifacet_reflectance: 0.0,
            roughness_correction: Roughness::None,
        }
    }
}
impl Input {
    pub fn validate(&self) -> Result<(), ModelError> {
        if !self.soil_brightness.is_finite() || !(0.0..=1.5).contains(&self.soil_brightness) {
            return Err(domain("soil_brightness must be finite and in [0, 1.5]"));
        }
        if !self.spectral_alpha_deg.is_finite()
            || !(-90.0..=90.0).contains(&self.spectral_alpha_deg)
        {
            return Err(domain("spectral_alpha_deg must be finite and in [-90, 90]"));
        }
        if !self.spectral_gamma_deg.is_finite() || !(0.0..=90.0).contains(&self.spectral_gamma_deg)
        {
            return Err(domain("spectral_gamma_deg must be finite and in [0, 90]"));
        }
        if !self.water_coverage.is_finite() || !(0.0..=1.0).contains(&self.water_coverage) {
            return Err(domain("water_coverage must be finite and in [0, 1]"));
        }
        if !self.water_film_thickness_mm.is_finite()
            || !(0.0..=10.0).contains(&self.water_film_thickness_mm)
        {
            return Err(domain(
                "water_film_thickness_mm must be finite and in [0, 10]",
            ));
        }
        if !self.particle_volume_fraction.is_finite()
            || !(0.0..=1.0).contains(&self.particle_volume_fraction)
        {
            return Err(domain(
                "particle_volume_fraction must be finite and in [0, 1]",
            ));
        }
        if !self.particle_refractive_index.is_finite()
            || !(1.0001..=3.0).contains(&self.particle_refractive_index)
        {
            return Err(domain(
                "particle_refractive_index must be finite and in [1.0001, 3]",
            ));
        }
        if !self.particle_extinction_coefficient.is_finite()
            || !(0.0..=0.1).contains(&self.particle_extinction_coefficient)
        {
            return Err(domain(
                "particle_extinction_coefficient must be finite and in [0, 0.1]",
            ));
        }
        if !self.solar_zenith_deg.is_finite() || !(0.0..=85.0).contains(&self.solar_zenith_deg) {
            return Err(domain("solar_zenith_deg must be finite and in [0, 85]"));
        }
        if !self.view_zenith_deg.is_finite() || !(0.0..=85.0).contains(&self.view_zenith_deg) {
            return Err(domain("view_zenith_deg must be finite and in [0, 85]"));
        }
        if !self.relative_azimuth_deg.is_finite()
            || !(0.0..=180.0).contains(&self.relative_azimuth_deg)
        {
            return Err(domain(
                "relative_azimuth_deg must be finite and in [0, 180]",
            ));
        }
        if !self.phase_asymmetry.is_finite() || !(0.0..=0.9999).contains(&self.phase_asymmetry) {
            return Err(domain("phase_asymmetry must be finite and in [0, 0.9999]"));
        }
        if !self.phase_backscatter.is_finite()
            || !(-0.9999..=1.4999).contains(&self.phase_backscatter)
        {
            return Err(domain(
                "phase_backscatter must be finite and in [-0.9999, 1.4999]",
            ));
        }
        if !self.hotspot_amplitude.is_finite() || !(0.0..=2.0).contains(&self.hotspot_amplitude) {
            return Err(domain("hotspot_amplitude must be finite and in [0, 2]"));
        }
        if !self.hotspot_width.is_finite() || !(0.0001..=2.0).contains(&self.hotspot_width) {
            return Err(domain("hotspot_width must be finite and in [0.0001, 2]"));
        }
        if !self.solid_filling_factor.is_finite()
            || !(0.0..=0.75).contains(&self.solid_filling_factor)
        {
            return Err(domain(
                "solid_filling_factor must be finite and in [0, 0.75]",
            ));
        }
        if !self.rms_slope.is_finite() || !(0.0..=2.0).contains(&self.rms_slope) {
            return Err(domain("rms_slope must be finite and in [0, 2]"));
        }
        if !self.multifacet_reflectance.is_finite()
            || !(0.0..=1.0).contains(&self.multifacet_reflectance)
        {
            return Err(domain(
                "multifacet_reflectance must be finite and in [0, 1]",
            ));
        }
        if self.phase_asymmetry <= 1e-10 {
            if self.phase_backscatter.abs() > 1e-10 {
                return Err(domain(
                    "phase_backscatter must be zero when phase_asymmetry is zero",
                ));
            }
        } else {
            let b = self.phase_asymmetry;
            let bound = (1.0 + 3.0 * b * b) / (b * (3.0 + b * b));
            if self.phase_backscatter.abs() > bound + 1e-10 {
                return Err(domain(
                    "phase parameters yield a negative scattering function",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct LambertInput {
    /// Soil spectral brightness multiplier.
    #[schemars(range(min=0.0,max=1.5),extend("x-unit"="1"))]
    pub soil_brightness: f64,
    /// First spherical mixing angle of the soil spectral basis.
    #[schemars(range(min=-90.0,max=90.0),extend("x-unit"="deg"))]
    pub spectral_alpha_deg: f64,
    /// Second spherical mixing angle of the soil spectral basis.
    #[schemars(range(min=0.0,max=90.0),extend("x-unit"="deg"))]
    pub spectral_gamma_deg: f64,
    /// Water-film covered surface fraction.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub water_coverage: f64,
    /// Water-film thickness in millimetres.
    #[schemars(range(min=0.0,max=10.0),extend("x-unit"="mm"))]
    pub water_film_thickness_mm: f64,
    /// Particle volume fraction within the water film.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub particle_volume_fraction: f64,
    /// Real refractive index of particles in the film.
    #[schemars(range(min=1.0001,max=3.0),extend("x-unit"="1"))]
    pub particle_refractive_index: f64,
    /// Non-negative imaginary refractive index of film particles.
    #[schemars(range(min=0.0,max=0.1),extend("x-unit"="1"))]
    pub particle_extinction_coefficient: f64,
}
impl Default for LambertInput {
    fn default() -> Self {
        Self {
            soil_brightness: 0.6,
            spectral_alpha_deg: 20.0,
            spectral_gamma_deg: 50.0,
            water_coverage: 0.5,
            water_film_thickness_mm: 0.3,
            particle_volume_fraction: 0.05,
            particle_refractive_index: 1.53,
            particle_extinction_coefficient: 0.001,
        }
    }
}
impl LambertInput {
    pub fn validate(&self) -> Result<(), ModelError> {
        if !self.soil_brightness.is_finite() || !(0.0..=1.5).contains(&self.soil_brightness) {
            return Err(domain("soil_brightness must be finite and in [0, 1.5]"));
        }
        if !self.spectral_alpha_deg.is_finite()
            || !(-90.0..=90.0).contains(&self.spectral_alpha_deg)
        {
            return Err(domain("spectral_alpha_deg must be finite and in [-90, 90]"));
        }
        if !self.spectral_gamma_deg.is_finite() || !(0.0..=90.0).contains(&self.spectral_gamma_deg)
        {
            return Err(domain("spectral_gamma_deg must be finite and in [0, 90]"));
        }
        if !self.water_coverage.is_finite() || !(0.0..=1.0).contains(&self.water_coverage) {
            return Err(domain("water_coverage must be finite and in [0, 1]"));
        }
        if !self.water_film_thickness_mm.is_finite()
            || !(0.0..=10.0).contains(&self.water_film_thickness_mm)
        {
            return Err(domain(
                "water_film_thickness_mm must be finite and in [0, 10]",
            ));
        }
        if !self.particle_volume_fraction.is_finite()
            || !(0.0..=1.0).contains(&self.particle_volume_fraction)
        {
            return Err(domain(
                "particle_volume_fraction must be finite and in [0, 1]",
            ));
        }
        if !self.particle_refractive_index.is_finite()
            || !(1.0001..=3.0).contains(&self.particle_refractive_index)
        {
            return Err(domain(
                "particle_refractive_index must be finite and in [1.0001, 3]",
            ));
        }
        if !self.particle_extinction_coefficient.is_finite()
            || !(0.0..=0.1).contains(&self.particle_extinction_coefficient)
        {
            return Err(domain(
                "particle_extinction_coefficient must be finite and in [0, 0.1]",
            ));
        }
        Ok(())
    }
}
impl LambertInput {
    pub(crate) fn directional(&self) -> Input {
        Input {
            soil_brightness: self.soil_brightness,
            spectral_alpha_deg: self.spectral_alpha_deg,
            spectral_gamma_deg: self.spectral_gamma_deg,
            water_coverage: self.water_coverage,
            water_film_thickness_mm: self.water_film_thickness_mm,
            particle_volume_fraction: self.particle_volume_fraction,
            particle_refractive_index: self.particle_refractive_index,
            particle_extinction_coefficient: self.particle_extinction_coefficient,
            ..Input::default()
        }
    }
}
