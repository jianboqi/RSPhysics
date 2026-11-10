use rsphysics_model_api::{ErrorKind, ModelError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LeafAngleDistribution {
    #[default]
    Campbell,
    Verhoef,
}
/// PROSAIL-D is one scientific model with its own flat parameter contract.
/// Leaf equations are delegated to the existing PROSPECT-D solver.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    #[schemars(range(min=1.0,max=5.0),extend("x-unit"="1"))]
    pub n: f64,
    #[schemars(range(min=0.0,max=200.0),extend("x-unit"="ug cm^-2"))]
    pub chlorophyll: f64,
    #[schemars(range(min=0.0,max=100.0),extend("x-unit"="ug cm^-2"))]
    pub carotenoids: f64,
    #[schemars(range(min=0.0,max=100.0),extend("x-unit"="ug cm^-2"))]
    pub anthocyanins: f64,
    #[schemars(range(min=0.0,max=5.0),extend("x-unit"="1"))]
    pub brown_pigments: f64,
    #[schemars(range(min=0.0,max=0.1),extend("x-unit"="g cm^-2"))]
    pub water: f64,
    #[schemars(range(min=0.0,max=0.05),extend("x-unit"="g cm^-2"))]
    pub dry_matter: f64,
    #[schemars(range(min=1.0,max=90.0),extend("x-unit"="deg"))]
    pub incidence_angle_deg: f64,
    /// Leaf area index (one-sided leaf area per ground area).
    #[schemars(range(min=0.0,max=10.0),extend("x-unit"="1"))]
    pub leaf_area_index: f64,
    /// Average leaf inclination for the author's 13-bin Campbell distribution.
    #[schemars(range(min=5.0,max=85.0),extend("x-unit"="deg"))]
    pub mean_leaf_angle_deg: f64,
    /// Leaf inclination distribution: Campbell ellipsoid or Verhoef bimodal.
    #[schemars(with="String",extend("enum"=["campbell", "verhoef"]))]
    pub leaf_angle_distribution: LeafAngleDistribution,
    #[schemars(range(min=-1.0,max=1.0),extend("x-unit"="1"))]
    pub lidf_a: f64,
    #[schemars(range(min=-1.0,max=1.0),extend("x-unit"="1"))]
    pub lidf_b: f64,
    /// Hotspot size parameter of 4SAIL.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub hotspot: f64,
    #[schemars(range(min=0.0,max=85.0),extend("x-unit"="deg"))]
    pub solar_zenith_deg: f64,
    #[schemars(range(min=0.0,max=85.0),extend("x-unit"="deg"))]
    pub view_zenith_deg: f64,
    #[schemars(range(min=0.0,max=180.0),extend("x-unit"="deg"))]
    pub relative_azimuth_deg: f64,
    /// Scalar multiplier of the original dry/wet soil spectrum blend.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub soil_brightness: f64,
    /// Linear dry-spectrum weight; 0=wet endmember, 1=dry. Not water content.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub soil_dry_fraction: f64,
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
            leaf_area_index: 3.0,
            mean_leaf_angle_deg: 60.0,
            leaf_angle_distribution: LeafAngleDistribution::Campbell,
            lidf_a: -0.35,
            lidf_b: -0.15,
            hotspot: 0.1,
            solar_zenith_deg: 30.0,
            view_zenith_deg: 0.0,
            relative_azimuth_deg: 60.0,
            soil_brightness: 1.0,
            soil_dry_fraction: 0.5,
        }
    }
}
impl Input {
    pub fn leaf(&self) -> rsphysics_prospect_d::Input {
        rsphysics_prospect_d::Input {
            n: self.n,
            chlorophyll: self.chlorophyll,
            carotenoids: self.carotenoids,
            anthocyanins: self.anthocyanins,
            brown_pigments: self.brown_pigments,
            water: self.water,
            dry_matter: self.dry_matter,
            incidence_angle_deg: self.incidence_angle_deg,
        }
    }
    pub fn validate(&self) -> Result<(), ModelError> {
        self.leaf().validate()?;
        for (name, value, min, max) in [
            ("leaf_area_index", self.leaf_area_index, 0.0, 10.0),
            ("mean_leaf_angle_deg", self.mean_leaf_angle_deg, 5.0, 85.0),
            ("lidf_a", self.lidf_a, -1., 1.),
            ("lidf_b", self.lidf_b, -1., 1.),
            ("hotspot", self.hotspot, 0.0, 1.0),
            ("solar_zenith_deg", self.solar_zenith_deg, 0.0, 85.0),
            ("view_zenith_deg", self.view_zenith_deg, 0.0, 85.0),
            (
                "relative_azimuth_deg",
                self.relative_azimuth_deg,
                0.0,
                180.0,
            ),
            ("soil_brightness", self.soil_brightness, 0.0, 1.0),
            ("soil_dry_fraction", self.soil_dry_fraction, 0.0, 1.0),
        ] {
            if !value.is_finite() || !(min..=max).contains(&value) {
                return Err(ModelError::new(
                    ErrorKind::OutOfDomain,
                    format!("{name} must be finite and in [{min}, {max}]"),
                ));
            }
        }
        if self.leaf_angle_distribution == LeafAngleDistribution::Verhoef
            && self.lidf_a.abs() + self.lidf_b.abs() > 1. + 1e-12
        {
            return Err(ModelError::new(
                ErrorKind::OutOfDomain,
                "Verhoef LIDF requires |a|+|b| <= 1",
            ));
        }
        Ok(())
    }
}
