//! Adaptation of supplied research MATLAB source; attribution and scope in LICENSE-ORIGIN.md.
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
mod solver;
fn domain(s: impl Into<String>) -> ModelError {
    ModelError::new(ErrorKind::OutOfDomain, s)
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    /// Volumetric soil moisture; positive to avoid original zero-moisture singularity.
    #[schemars(range(min=0.005,max=0.6),extend("x-unit"="m^3 m^-3"))]
    pub soil_moisture: f64,
    /// Vegetation optical depth at nadir.
    #[schemars(range(min=0.0,max=3.0),extend("x-unit"="1"))]
    pub nadir_optical_depth: f64,
    /// Original para(3): air temperature for prescribed sky model, before forward-mode effective-temperature update.
    #[schemars(range(min=250.0,max=330.0),extend("x-unit"="K"))]
    pub sky_air_temperature_k: f64,
    /// Effective canopy temperature.
    #[schemars(range(min=250.0,max=330.0),extend("x-unit"="K"))]
    pub canopy_temperature_k: f64,
    /// Shallow soil temperature.
    #[schemars(range(min=250.0,max=330.0),extend("x-unit"="K"))]
    pub surface_temperature_k: f64,
    /// Deep soil temperature.
    #[schemars(range(min=250.0,max=330.0),extend("x-unit"="K"))]
    pub deep_temperature_k: f64,
    /// Soil bulk density.
    #[schemars(range(min=0.5,max=2.0),extend("x-unit"="g cm^-3"))]
    pub bulk_density_g_cm3: f64,
    /// Sand mass fraction.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub sand_fraction: f64,
    /// Clay mass fraction; sand plus clay cannot exceed one.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub clay_fraction: f64,
    /// Moisture scale xa in effective soil-temperature parameterization.
    #[schemars(range(min=0.05,max=0.6),extend("x-unit"="m^3 m^-3"))]
    pub temperature_moisture_scale: f64,
    /// Exponent xb in effective soil-temperature parameterization.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub temperature_moisture_exponent: f64,
    /// Horizontal-polarization angular opacity factor tth.
    #[schemars(range(min=0.0,max=5.0),extend("x-unit"="1"))]
    pub canopy_structure_h: f64,
    /// Vertical/horizontal structure ratio rtt.
    #[schemars(range(min=0.0,max=3.0),extend("x-unit"="1"))]
    pub canopy_structure_v_ratio: f64,
    /// Horizontal-polarization single-scattering albedo.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub scattering_albedo_h: f64,
    /// Vertical minus horizontal albedo; their sum must lie in [0,1].
    #[schemars(range(min=-1.0,max=1.0),extend("x-unit"="1"))]
    pub scattering_albedo_v_difference: f64,
    /// Empirical dimensionless roughness height hsol.
    #[schemars(range(min=0.0,max=2.0),extend("x-unit"="1"))]
    pub roughness_height: f64,
    /// Empirical polarization mixing qsol.
    #[schemars(range(min=0.0,max=0.5),extend("x-unit"="1"))]
    pub roughness_mixing: f64,
    /// Vertical roughness angular exponent.
    #[schemars(range(min=0.0,max=4.0),extend("x-unit"="1"))]
    pub roughness_exponent_v: f64,
    /// Horizontal roughness angular exponent.
    #[schemars(range(min=0.0,max=4.0),extend("x-unit"="1"))]
    pub roughness_exponent_h: f64,
    /// Incidence angle from the surface normal.
    #[schemars(range(min=0.0,max=80.0),extend("x-unit"="deg"))]
    pub observation_zenith_deg: f64,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            soil_moisture: 0.07,
            nadir_optical_depth: 0.0,
            sky_air_temperature_k: 300.0,
            canopy_temperature_k: 300.0,
            surface_temperature_k: 300.0,
            deep_temperature_k: 283.0,
            bulk_density_g_cm3: 1.28,
            sand_fraction: 0.132,
            clay_fraction: 0.328,
            temperature_moisture_scale: 0.3,
            temperature_moisture_exponent: 0.3,
            canopy_structure_h: 2.0,
            canopy_structure_v_ratio: 1.0,
            scattering_albedo_h: 0.05,
            scattering_albedo_v_difference: 0.0,
            roughness_height: 0.1,
            roughness_mixing: 0.0,
            roughness_exponent_v: 0.0,
            roughness_exponent_h: 0.0,
            observation_zenith_deg: 40.0,
        }
    }
}
impl Input {
    pub fn validate(&self) -> Result<(), ModelError> {
        for (name, v, lo, hi) in [
            ("soil_moisture", self.soil_moisture, 0.005, 0.6),
            ("nadir_optical_depth", self.nadir_optical_depth, 0.0, 3.0),
            (
                "sky_air_temperature_k",
                self.sky_air_temperature_k,
                250.0,
                330.0,
            ),
            (
                "canopy_temperature_k",
                self.canopy_temperature_k,
                250.0,
                330.0,
            ),
            (
                "surface_temperature_k",
                self.surface_temperature_k,
                250.0,
                330.0,
            ),
            ("deep_temperature_k", self.deep_temperature_k, 250.0, 330.0),
            ("bulk_density_g_cm3", self.bulk_density_g_cm3, 0.5, 2.0),
            ("sand_fraction", self.sand_fraction, 0.0, 1.0),
            ("clay_fraction", self.clay_fraction, 0.0, 1.0),
            (
                "temperature_moisture_scale",
                self.temperature_moisture_scale,
                0.05,
                0.6,
            ),
            (
                "temperature_moisture_exponent",
                self.temperature_moisture_exponent,
                0.0,
                1.0,
            ),
            ("canopy_structure_h", self.canopy_structure_h, 0.0, 5.0),
            (
                "canopy_structure_v_ratio",
                self.canopy_structure_v_ratio,
                0.0,
                3.0,
            ),
            ("scattering_albedo_h", self.scattering_albedo_h, 0.0, 1.0),
            (
                "scattering_albedo_v_difference",
                self.scattering_albedo_v_difference,
                -1.0,
                1.0,
            ),
            ("roughness_height", self.roughness_height, 0.0, 2.0),
            ("roughness_mixing", self.roughness_mixing, 0.0, 0.5),
            ("roughness_exponent_v", self.roughness_exponent_v, 0.0, 4.0),
            ("roughness_exponent_h", self.roughness_exponent_h, 0.0, 4.0),
            (
                "observation_zenith_deg",
                self.observation_zenith_deg,
                0.0,
                80.0,
            ),
        ] {
            if !v.is_finite() || !(lo..=hi).contains(&v) {
                return Err(domain(format!("{name} must be finite in [{lo},{hi}]")));
            }
        }
        if self.sand_fraction + self.clay_fraction > 1.
            || !(0. ..=1.)
                .contains(&(self.scattering_albedo_h + self.scattering_albedo_v_difference))
        {
            return Err(domain(
                "sand+clay and polarization albedos must be physically valid",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    /// Vertical-polarization surface brightness temperature.
    pub brightness_temperature_v_k: f64,
    /// Horizontal-polarization surface brightness temperature.
    pub brightness_temperature_h_k: f64,
    /// Moisture-weighted effective soil temperature.
    pub effective_soil_temperature_k: f64,
    /// Real soil relative permittivity.
    pub soil_permittivity_real: f64,
    /// Imaginary part in the original negative-loss convention; dry-sand branch sign retained.
    pub soil_permittivity_imaginary: f64,
    /// Roughness-adjusted vertical power reflectivity.
    pub soil_reflectivity_v: f64,
    /// Roughness-adjusted horizontal power reflectivity.
    pub soil_reflectivity_h: f64,
    /// Prescribed downwelling sky-plus-atmosphere brightness.
    pub sky_brightness_temperature_k: f64,
    /// Vertical canopy transmission.
    pub transmissivity_v: f64,
    /// Horizontal canopy transmission.
    pub transmissivity_h: f64,
    /// Fixed incidence angles 0..80 degrees.
    pub sweep_zenith_deg: Vec<f64>,
    /// Vertical response with other parameters fixed.
    pub sweep_brightness_temperature_v_k: Vec<f64>,
    /// Horizontal response with other parameters fixed.
    pub sweep_brightness_temperature_h_k: Vec<f64>,
}
pub struct Model {
    metadata: ModelMetadata,
}
impl Model {
    pub fn new() -> Result<Self, ModelError> {
        Ok(Self {
            metadata: parse_model_manifest(include_str!("../model.toml"))?,
        })
    }
    pub fn simulate(&self, p: &Input) -> Result<Output, ModelError> {
        p.validate()?;
        let out = solver::simulate(p)?;
        self.validate_output(&out)?;
        Ok(out)
    }
}
impl PhysicsModel for Model {
    type Input = Input;
    type Output = Output;
    fn metadata(&self) -> ModelMetadata {
        self.metadata.clone()
    }
    fn validate_input(&self, p: &Input) -> Result<(), ModelError> {
        p.validate()
    }
    fn run(&self, p: &Input) -> Result<Output, ModelError> {
        self.simulate(p)
    }
    fn validate_output(&self, o: &Output) -> Result<(), ModelError> {
        let scalars = [
            o.brightness_temperature_v_k,
            o.brightness_temperature_h_k,
            o.effective_soil_temperature_k,
            o.soil_permittivity_real,
            o.soil_permittivity_imaginary,
            o.soil_reflectivity_v,
            o.soil_reflectivity_h,
            o.sky_brightness_temperature_k,
            o.transmissivity_v,
            o.transmissivity_h,
        ];
        if scalars.iter().any(|x| !x.is_finite())
            || o.sweep_zenith_deg.len() != 81
            || o.sweep_zenith_deg.iter().any(|x| !x.is_finite())
            || o.sweep_brightness_temperature_v_k.len() != 81
            || o.sweep_brightness_temperature_v_k
                .iter()
                .any(|x| !x.is_finite())
            || o.sweep_brightness_temperature_h_k.len() != 81
            || o.sweep_brightness_temperature_h_k
                .iter()
                .any(|x| !x.is_finite())
        {
            return Err(ModelError::new(
                ErrorKind::NonConvergence,
                "nonfinite model output or invalid shape",
            ));
        }
        Ok(())
    }
}
