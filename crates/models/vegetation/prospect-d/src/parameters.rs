use rsphysics_model_api::{ErrorKind, ModelError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// PROSPECT-D parameters. No PROSPECT-PRO protein/CBC parameters are accepted.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    /// Leaf structure parameter (dimensionless).
    #[schemars(range(min = 1.0, max = 5.0), extend("x-unit" = "1"))]
    pub n: f64,
    /// Chlorophyll a+b content in micrograms per square centimetre.
    #[schemars(range(min = 0.0, max = 200.0), extend("x-unit" = "ug cm^-2"))]
    pub chlorophyll: f64,
    /// Carotenoid content in micrograms per square centimetre.
    #[schemars(range(min = 0.0, max = 100.0), extend("x-unit" = "ug cm^-2"))]
    pub carotenoids: f64,
    /// Anthocyanin content in micrograms per square centimetre.
    #[schemars(range(min = 0.0, max = 100.0), extend("x-unit" = "ug cm^-2"))]
    pub anthocyanins: f64,
    /// Brown pigments, in the original model's arbitrary units.
    #[schemars(range(min = 0.0, max = 5.0), extend("x-unit" = "1"))]
    pub brown_pigments: f64,
    /// Equivalent water mass per area in grams per square centimetre.
    #[schemars(range(min = 0.0, max = 0.1), extend("x-unit" = "g cm^-2"))]
    pub water: f64,
    /// Leaf dry mass per area (LMA) in grams per square centimetre.
    #[schemars(range(min = 0.0, max = 0.05), extend("x-unit" = "g cm^-2"))]
    pub dry_matter: f64,
    /// Maximum incidence angle of the illumination cone, in degrees.
    #[schemars(range(min = 1.0, max = 90.0), extend("x-unit" = "deg"))]
    pub incidence_angle_deg: f64,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            n: 1.5,
            chlorophyll: 40.0,
            carotenoids: 8.0,
            anthocyanins: 0.0,
            brown_pigments: 0.0,
            water: 0.01,
            dry_matter: 0.008,
            incidence_angle_deg: 40.0,
        }
    }
}

impl Input {
    /// This is the supported implementation envelope, not a universal biological domain.
    pub fn validate(&self) -> Result<(), ModelError> {
        for (name, value, low, high) in [
            ("n", self.n, 1.0, 5.0),
            ("chlorophyll", self.chlorophyll, 0.0, 200.0),
            ("carotenoids", self.carotenoids, 0.0, 100.0),
            ("anthocyanins", self.anthocyanins, 0.0, 100.0),
            ("brown_pigments", self.brown_pigments, 0.0, 5.0),
            ("water", self.water, 0.0, 0.1),
            ("dry_matter", self.dry_matter, 0.0, 0.05),
            ("incidence_angle_deg", self.incidence_angle_deg, 1.0, 90.0),
        ] {
            if !value.is_finite() || !(low..=high).contains(&value) {
                return Err(ModelError::new(
                    ErrorKind::OutOfDomain,
                    format!("{name} must be finite and in [{low}, {high}]"),
                ));
            }
        }
        Ok(())
    }
}
