//! Vacuum blackbody spectral radiance (not exitance), with exact SI constants.
//! Log-domain evaluation avoids overflow in Wien and cancellation in RJ limits.
use rsphysics_core::DomainError;
pub const PLANCK_CONSTANT: f64 = 6.626_070_15e-34;
pub const SPEED_OF_LIGHT: f64 = 299_792_458.;
pub const BOLTZMANN_CONSTANT: f64 = 1.380_649e-23;

#[derive(Debug, Clone, Copy)]
pub enum WavelengthDensity {
    PerMetre,
    PerNanometre,
    PerMicrometre,
}
impl WavelengthDensity {
    fn log_scale(self) -> f64 {
        match self {
            Self::PerMetre => 0.,
            Self::PerNanometre => 1e-9_f64.ln(),
            Self::PerMicrometre => 1e-6_f64.ln(),
        }
    }
}
fn coordinate(v: f64) -> Result<(), DomainError> {
    if !v.is_finite() || v <= 0. {
        return Err(DomainError(
            "vacuum wavelength/frequency must be positive and finite",
        ));
    }
    Ok(())
}
fn nonnegative(v: f64) -> Result<(), DomainError> {
    if !v.is_finite() || v < 0. {
        return Err(DomainError(
            "temperature/radiance must be nonnegative and finite",
        ));
    }
    Ok(())
}
fn exp_result(log: f64) -> Result<f64, DomainError> {
    let result = log.exp();
    if !result.is_finite() {
        return Err(DomainError("thermal result exceeds the finite f64 range"));
    }
    Ok(result)
}
fn log_expm1_from_log(log_x: f64) -> f64 {
    if log_x < -36. {
        return log_x;
    }
    let x = log_x.exp();
    if x > 50. {
        x + (-(-x).exp()).ln_1p()
    } else {
        x.exp_m1().ln()
    }
}
fn log_softplus(z: f64) -> f64 {
    if z < -36. {
        z
    } else if z > 36. {
        (z + (-z).exp().ln_1p()).ln()
    } else {
        z.exp().ln_1p().ln()
    }
}
fn forward(log_prefactor: f64, log_characteristic: f64, t: f64) -> Result<f64, DomainError> {
    nonnegative(t)?;
    if t == 0. {
        return Ok(0.);
    }
    exp_result(log_prefactor - log_expm1_from_log(log_characteristic - t.ln()))
}
fn inverse(log_prefactor: f64, log_characteristic: f64, l: f64) -> Result<f64, DomainError> {
    nonnegative(l)?;
    if l == 0. {
        return Ok(0.);
    }
    exp_result(log_characteristic - log_softplus(log_prefactor - l.ln()))
}
/// B_lambda, wavelength supplied in nm; output density selected explicitly.
pub fn planck_radiance(
    wavelength_nm: f64,
    temperature_k: f64,
    unit: WavelengthDensity,
) -> Result<f64, DomainError> {
    coordinate(wavelength_nm)?;
    let lambda = wavelength_nm.ln() + 1e-9_f64.ln();
    forward(
        (2. * PLANCK_CONSTANT * SPEED_OF_LIGHT.powi(2)).ln() - 5. * lambda + unit.log_scale(),
        (PLANCK_CONSTANT * SPEED_OF_LIGHT / BOLTZMANN_CONSTANT).ln() - lambda,
        temperature_k,
    )
}
/// Monochromatic brightness temperature [K], not a sensor-band K1/K2 conversion.
pub fn inverse_planck(
    wavelength_nm: f64,
    radiance: f64,
    unit: WavelengthDensity,
) -> Result<f64, DomainError> {
    coordinate(wavelength_nm)?;
    let lambda = wavelength_nm.ln() + 1e-9_f64.ln();
    inverse(
        (2. * PLANCK_CONSTANT * SPEED_OF_LIGHT.powi(2)).ln() - 5. * lambda + unit.log_scale(),
        (PLANCK_CONSTANT * SPEED_OF_LIGHT / BOLTZMANN_CONSTANT).ln() - lambda,
        radiance,
    )
}
/// B_nu [W m^-2 sr^-1 Hz^-1], frequency in Hz.
pub fn planck_frequency_radiance(
    frequency_hz: f64,
    temperature_k: f64,
) -> Result<f64, DomainError> {
    coordinate(frequency_hz)?;
    let nu = frequency_hz.ln();
    forward(
        (2. * PLANCK_CONSTANT / SPEED_OF_LIGHT.powi(2)).ln() + 3. * nu,
        (PLANCK_CONSTANT / BOLTZMANN_CONSTANT).ln() + nu,
        temperature_k,
    )
}
pub fn inverse_planck_frequency(
    frequency_hz: f64,
    radiance_per_hz: f64,
) -> Result<f64, DomainError> {
    coordinate(frequency_hz)?;
    let nu = frequency_hz.ln();
    inverse(
        (2. * PLANCK_CONSTANT / SPEED_OF_LIGHT.powi(2)).ln() + 3. * nu,
        (PLANCK_CONSTANT / BOLTZMANN_CONSTANT).ln() + nu,
        radiance_per_hz,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn units_and_inverse_limits() {
        let l = planck_radiance(10000., 300., WavelengthDensity::PerMicrometre).unwrap();
        assert!((l - 9.9240333300707).abs() < 1e-12);
        let nm = planck_radiance(10000., 300., WavelengthDensity::PerNanometre).unwrap();
        assert!((l / nm - 1000.).abs() < 1e-10);
        assert!(
            (inverse_planck(10000., l, WavelengthDensity::PerMicrometre).unwrap() - 300.).abs()
                < 1e-10
        );
        assert_eq!(
            planck_radiance(1., 0., WavelengthDensity::PerMetre).unwrap(),
            0.
        );
        assert_eq!(
            planck_radiance(1., 1., WavelengthDensity::PerMetre).unwrap(),
            0.
        );
        assert_eq!(
            inverse_planck(1., 0., WavelengthDensity::PerMetre).unwrap(),
            0.
        );
        assert!(planck_radiance(0., 300., WavelengthDensity::PerMetre).is_err());
        assert!(planck_radiance(1000., -1., WavelengthDensity::PerMetre).is_err());
    }
    #[test]
    fn frequency_density_jacobian_and_rayleigh_jeans() {
        let lambda = 1e-5;
        let frequency = SPEED_OF_LIGHT / lambda;
        let bnu = planck_frequency_radiance(frequency, 300.).unwrap();
        let bl = planck_radiance(lambda * 1e9, 300., WavelengthDensity::PerMetre).unwrap();
        assert!((bnu / (bl * lambda * lambda / SPEED_OF_LIGHT) - 1.).abs() < 1e-12);
        assert!((inverse_planck_frequency(frequency, bnu).unwrap() - 300.).abs() < 1e-10);
        let rj = 2. * BOLTZMANN_CONSTANT * 1e6 * 1e6_f64.powi(2) / SPEED_OF_LIGHT.powi(2);
        assert!((planck_frequency_radiance(1e6, 1e6).unwrap() / rj - 1.).abs() < 3e-11);
    }
}
