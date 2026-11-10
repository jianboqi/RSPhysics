//! Scientific conventions: wavelengths in nm, angles in radians, explicit radiometric quantities.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainError(pub &'static str);

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl Error for DomainError {}

/// A positive, finite, strictly increasing wavelength axis, in nanometres.
/// Non-uniform sampling and single wavelength axes are supported.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "Vec<f64>", into = "Vec<f64>")]
pub struct SpectralAxis(Vec<f64>);

impl SpectralAxis {
    pub fn new(wavelengths_nm: Vec<f64>) -> Result<Self, DomainError> {
        if wavelengths_nm.is_empty()
            || wavelengths_nm.iter().any(|x| !x.is_finite() || *x <= 0.0)
            || wavelengths_nm.windows(2).any(|w| w[0] >= w[1])
        {
            return Err(DomainError(
                "wavelengths_nm must be nonempty, finite, positive and strictly increasing",
            ));
        }
        Ok(Self(wavelengths_nm))
    }

    pub fn wavelengths_nm(&self) -> &[f64] {
        &self.0
    }
}

impl TryFrom<Vec<f64>> for SpectralAxis {
    type Error = DomainError;
    fn try_from(value: Vec<f64>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<SpectralAxis> for Vec<f64> {
    fn from(value: SpectralAxis) -> Self {
        value.0
    }
}

/// Finite angle serialized as radians. Use named constructors for degree inputs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "f64", into = "f64")]
pub struct Angle(f64);

impl Angle {
    pub fn from_radians(radians: f64) -> Result<Self, DomainError> {
        if !radians.is_finite() {
            return Err(DomainError("angle must be finite"));
        }
        Ok(Self(radians))
    }

    pub fn from_degrees(degrees: f64) -> Result<Self, DomainError> {
        Self::from_radians(degrees.to_radians())
    }

    pub fn radians(self) -> f64 {
        self.0
    }

    pub fn degrees(self) -> f64 {
        self.0.to_degrees()
    }
}

impl TryFrom<f64> for Angle {
    type Error = DomainError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::from_radians(value)
    }
}

impl From<Angle> for f64 {
    fn from(value: Angle) -> Self {
        value.0
    }
}

/// Upper-hemisphere geometry. Relative azimuth is the unsigned separation
/// between sun and view azimuths in [0, pi], with zero meaning the same azimuth.
/// Models requiring oriented azimuths must define a different geometry type.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "GeometryWire", into = "GeometryWire")]
pub struct Geometry {
    solar_zenith: Angle,
    view_zenith: Angle,
    relative_azimuth: Angle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct GeometryWire {
    solar_zenith_rad: f64,
    view_zenith_rad: f64,
    relative_azimuth_rad: f64,
}

impl Geometry {
    pub fn new(
        solar_zenith: Angle,
        view_zenith: Angle,
        relative_azimuth: Angle,
    ) -> Result<Self, DomainError> {
        use std::f64::consts::{FRAC_PI_2, PI};
        if !(0.0..=FRAC_PI_2).contains(&solar_zenith.radians())
            || !(0.0..=FRAC_PI_2).contains(&view_zenith.radians())
            || !(0.0..=PI).contains(&relative_azimuth.radians())
        {
            return Err(DomainError(
                "zeniths must be in [0, pi/2] and relative azimuth in [0, pi]",
            ));
        }
        Ok(Self {
            solar_zenith,
            view_zenith,
            relative_azimuth,
        })
    }

    pub fn solar_zenith(self) -> Angle {
        self.solar_zenith
    }
    pub fn view_zenith(self) -> Angle {
        self.view_zenith
    }
    pub fn relative_azimuth(self) -> Angle {
        self.relative_azimuth
    }
}

impl TryFrom<GeometryWire> for Geometry {
    type Error = DomainError;
    fn try_from(value: GeometryWire) -> Result<Self, Self::Error> {
        Self::new(
            Angle::from_radians(value.solar_zenith_rad)?,
            Angle::from_radians(value.view_zenith_rad)?,
            Angle::from_radians(value.relative_azimuth_rad)?,
        )
    }
}

impl From<Geometry> for GeometryWire {
    fn from(value: Geometry) -> Self {
        Self {
            solar_zenith_rad: value.solar_zenith.radians(),
            view_zenith_rad: value.view_zenith.radians(),
            relative_azimuth_rad: value.relative_azimuth.radians(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RadiometricQuantity {
    SpectralRadiance,
    SpectralIrradiance,
    Brdf,
    Brf,
    Reflectance,
    Transmittance,
    HemisphericalAlbedo,
    BrightnessTemperature,
    /// Normalized radar cross section sigma0 in linear power.
    RadarBackscatter,
}

impl RadiometricQuantity {
    pub fn unit(self) -> &'static str {
        match self {
            Self::SpectralRadiance => "W m^-2 sr^-1 nm^-1",
            Self::SpectralIrradiance => "W m^-2 nm^-1",
            Self::Brdf => "sr^-1",
            Self::BrightnessTemperature => "K",
            _ => "1",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spectral_invariants_also_hold_at_deserialization_boundary() {
        for values in [
            vec![],
            vec![0.0],
            vec![f64::NAN],
            vec![f64::INFINITY],
            vec![500.0, 500.0],
            vec![600.0, 500.0],
        ] {
            assert!(SpectralAxis::new(values).is_err());
        }
        assert!(serde_json::from_str::<SpectralAxis>("[600,500]").is_err());
        let axis = SpectralAxis::new(vec![400.0, 410.0, 800.0]).unwrap();
        assert_eq!(
            serde_json::from_str::<SpectralAxis>(&serde_json::to_string(&axis).unwrap()).unwrap(),
            axis
        );
    }

    #[test]
    fn angles_and_geometry_have_explicit_units_and_bounds() {
        let right = Angle::from_degrees(90.0).unwrap();
        assert!((right.radians() - std::f64::consts::FRAC_PI_2).abs() < 1e-15);
        assert!(Angle::from_radians(f64::INFINITY).is_err());
        assert!(Geometry::new(right, right, Angle::from_degrees(180.0).unwrap()).is_ok());
        assert!(Geometry::new(Angle::from_degrees(91.0).unwrap(), right, right).is_err());
        assert!(
            serde_json::from_str::<Geometry>(
                r#"{"solar_zenith_rad":-1,"view_zenith_rad":0,"relative_azimuth_rad":0}"#
            )
            .is_err()
        );
        assert_ne!(
            RadiometricQuantity::Brdf.unit(),
            RadiometricQuantity::Brf.unit()
        );
    }
}
