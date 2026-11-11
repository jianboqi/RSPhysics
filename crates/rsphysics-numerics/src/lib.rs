//! Small, explicit numerical primitives. No extrapolation or silent resampling.

use rsphysics_core::{DomainError, SpectralAxis};

pub mod angular;
pub mod lut;
pub mod optics;
pub mod radiometry;
pub mod spectral;
pub mod thermal;

/// Piecewise linear interpolation. Rejects extrapolation, shape mismatches and non-finite data.
pub fn interpolate_linear(
    axis: &SpectralAxis,
    values: &[f64],
    wavelength_nm: f64,
) -> Result<f64, DomainError> {
    let x = axis.wavelengths_nm();
    if values.len() != x.len() || values.iter().any(|y| !y.is_finite()) {
        return Err(DomainError(
            "interpolation values must be finite and match the spectral axis",
        ));
    }
    if !wavelength_nm.is_finite() || wavelength_nm < x[0] || wavelength_nm > x[x.len() - 1] {
        return Err(DomainError(
            "interpolation outside the spectral axis is not supported",
        ));
    }
    match x.binary_search_by(|v| v.total_cmp(&wavelength_nm)) {
        Ok(i) => Ok(values[i]),
        Err(i) => {
            let weight = (wavelength_nm - x[i - 1]) / (x[i] - x[i - 1]);
            let result = (1.0 - weight) * values[i - 1] + weight * values[i];
            if !result.is_finite() {
                return Err(DomainError("interpolation produced a non-finite result"));
            }
            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_nonuniform_axis_without_extrapolation() {
        let axis = SpectralAxis::new(vec![400.0, 500.0, 800.0]).unwrap();
        assert_eq!(
            interpolate_linear(&axis, &[0.0, 1.0, 4.0], 650.0).unwrap(),
            2.5
        );
        assert_eq!(
            interpolate_linear(&axis, &[0.0, 1.0, 4.0], 800.0).unwrap(),
            4.0
        );
        assert!(interpolate_linear(&axis, &[0.0, 1.0, 4.0], 399.0).is_err());
        assert!(interpolate_linear(&axis, &[0.0], 400.0).is_err());
        assert!(interpolate_linear(&axis, &[0.0, f64::NAN, 4.0], 400.0).is_err());
        let single = SpectralAxis::new(vec![500.0]).unwrap();
        assert_eq!(interpolate_linear(&single, &[2.0], 500.0).unwrap(), 2.0);
    }
}

pub mod sail;
