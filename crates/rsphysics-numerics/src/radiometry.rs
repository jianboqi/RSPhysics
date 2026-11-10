//! Basic calibration operations. No scene reader, atmospheric correction or LST.
use rsphysics_core::DomainError;
pub const NONFINITE: u8 = 1;
pub const NODATA: u8 = 2;
pub const SATURATED: u8 = 4;
pub const MASKED: u8 = 8;
pub const DOMAIN: u8 = 16;
#[derive(Debug)]
pub struct Calibrated {
    pub values: Vec<f64>,
    pub flags: Vec<u8>,
}
fn check(v: f64) -> Result<f64, DomainError> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(DomainError("calibration constants must be finite"))
    }
}
fn zenith(theta: f64) -> Result<f64, DomainError> {
    if !theta.is_finite() || !(0.0..90.0).contains(&theta) {
        return Err(DomainError("solar zenith must be in [0,90) degrees"));
    }
    Ok(theta.to_radians().cos())
}
/// Gain and offset have the units of the intended output; no range clipping.
pub fn linear_calibrate(
    dn: &[f64],
    gain: f64,
    offset: f64,
    valid: &[bool],
    nodata: Option<f64>,
    saturation: Option<f64>,
) -> Result<Calibrated, DomainError> {
    check(gain)?;
    check(offset)?;
    if let Some(v) = nodata {
        check(v)?;
    }
    if let Some(v) = saturation {
        check(v)?;
    }
    if valid.len() != dn.len() {
        return Err(DomainError("validity mask must match the input"));
    }
    let mut values = Vec::with_capacity(dn.len());
    let mut flags = Vec::with_capacity(dn.len());
    for (&v, &ok) in dn.iter().zip(valid) {
        let mut flag = 0;
        if !v.is_finite() {
            flag |= NONFINITE;
        }
        if !ok {
            flag |= MASKED;
        }
        if nodata == Some(v) {
            flag |= NODATA;
        }
        if saturation.is_some_and(|s| v >= s) {
            flag |= SATURATED;
        }
        let out = v * gain + offset;
        if !out.is_finite() {
            flag |= NONFINITE;
        }
        values.push(if flag == 0 { out } else { f64::NAN });
        flags.push(flag);
    }
    Ok(Calibrated { values, flags })
}
/// L and E must share a spectral-density basis (both per nm, or both per um).
pub fn radiance_to_toa_reflectance(
    l: f64,
    esun: f64,
    solar_zenith_deg: f64,
    distance_au: f64,
) -> Result<f64, DomainError> {
    check(l)?;
    check(esun)?;
    check(distance_au)?;
    let c = zenith(solar_zenith_deg)?;
    if esun <= 0. || distance_au <= 0. {
        return Err(DomainError(
            "irradiance and Earth-Sun distance must be positive",
        ));
    }
    check(std::f64::consts::PI * l / esun * distance_au.powi(2) / c)
}
/// Product reflectance multipliers already include solar irradiance and distance.
pub fn correct_reflectance_sun_angle(
    uncorrected: f64,
    solar_zenith_deg: f64,
) -> Result<f64, DomainError> {
    check(uncorrected)?;
    check(uncorrected / zenith(solar_zenith_deg)?)
}
/// K1 uses W m^-2 sr^-1 um^-1, K2 uses K (Landsat MTL convention).
/// This is at-sensor brightness temperature, not emissivity-corrected LST.
pub fn brightness_temperature(
    l: f64,
    k1: f64,
    k2: f64,
    radiance_per_nm: bool,
) -> Result<f64, DomainError> {
    check(l)?;
    check(k1)?;
    check(k2)?;
    if l <= 0. || k1 <= 0. || k2 <= 0. {
        return Err(DomainError(
            "brightness temperature requires positive radiance and K1/K2",
        ));
    }
    let l = if radiance_per_nm { l * 1000. } else { l };
    // log(1+K1/L) = softplus(log K1 - log L), avoiding overflow of K1/L.
    let z = k1.ln() - l.ln();
    let log = if z > 0. {
        z + (-z).exp().ln_1p()
    } else {
        z.exp().ln_1p()
    };
    check(k2 / log)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metadata_scaling_flags_and_zero_dn() {
        let r = linear_calibrate(
            &[0., 1., 255., f64::NAN, 2.],
            2.,
            -1.,
            &[true, true, true, true, false],
            Some(1.),
            Some(255.),
        )
        .unwrap();
        assert_eq!(r.values[0], -1.);
        assert_eq!(r.flags, vec![0, NODATA, SATURATED, NONFINITE, MASKED]);
        assert!(r.values[1].is_nan());
    }
    #[test]
    fn radiance_reflectance_and_temperature_conventions() {
        let r = radiance_to_toa_reflectance(1., std::f64::consts::PI, 60., 1.).unwrap();
        assert!((r - 2.).abs() < 1e-12);
        assert!((correct_reflectance_sun_angle(0.1, 60.).unwrap() - 0.2).abs() < 1e-12);
        let l = 10.;
        let k1 = 774.8853;
        let k2 = 1321.0789;
        let t = brightness_temperature(l, k1, k2, false).unwrap();
        assert!((t - k2 / (k1 / l + 1.).ln()).abs() < 1e-12);
        assert!((brightness_temperature(l / 1000., k1, k2, true).unwrap() - t).abs() < 1e-12);
        assert!(brightness_temperature(0., k1, k2, false).is_err());
        assert!(correct_reflectance_sun_angle(0.1, 90.).is_err());
    }
}
