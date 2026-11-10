//! SORT soil optical transport. Scientific author: Anxin Ding.
//! Hapke roughness adaptation retains the Dylan Shiltz MIT notice.
use crate::{Input, Roughness, Sample, domain};
use num_complex::Complex64;
use rsphysics_model_api::{ErrorKind, ModelError};
use std::{f64::consts::PI, sync::OnceLock};

const TINY: f64 = 1e-12;

pub fn checked(value: f64, bounded: bool) -> Result<f64, ModelError> {
    if !value.is_finite() || value < -1e-10 || (bounded && value > 1.0 + 1e-10) {
        return Err(domain(
            "soil optical response is nonfinite, negative, or outside its bounded physical domain",
        ));
    }
    Ok(value)
}

pub fn basis(s: &Sample, p: &Input) -> Result<f64, ModelError> {
    let a = p.spectral_alpha_deg.to_radians();
    let g = p.spectral_gamma_deg.to_radians();
    checked(
        p.soil_brightness
            * (a.sin() * s.basis[0]
                + a.cos() * g.sin() * s.basis[1]
                + a.cos() * g.cos() * s.basis[2]),
        true,
    )
}

pub fn ssa(r: f64) -> f64 {
    1.0 - ((1.0 - r) / (1.0 + r)).powi(2)
}

fn h_function(mu: f64, w: f64, packing: f64) -> f64 {
    let m = mu / packing;
    let sq = (1.0 - w).max(0.0).sqrt();
    let r = (1.0 - sq) / (1.0 + sq);
    let xlog = if m > 0.0 { m * (1.0 / m).ln_1p() } else { 0.0 };
    1.0 / (1.0 - w * (m * r + 0.5 * (1.0 - 2.0 * r * m) * xlog))
}

fn flat(p: &Input, w: f64, i: f64, e: f64, phase: f64) -> f64 {
    let mu0 = i.cos().clamp(0.0, 1.0);
    let mu = e.cos().clamp(0.0, 1.0);
    let b = p.phase_asymmetry;
    let c = p.phase_backscatter;
    let phase_function = (1.0 - b * b)
        * (0.5 * (1.0 + c) / (1.0 - 2.0 * b * phase.cos() + b * b).max(TINY).powf(1.5)
            + 0.5 * (1.0 - c) / (1.0 + 2.0 * b * phase.cos() + b * b).max(TINY).powf(1.5));
    let hotspot =
        p.hotspot_amplitude / (1.0 + (phase.min(PI - TINY) / 2.0).tan() / p.hotspot_width);
    let x = 1.209 * p.solid_filling_factor.powf(2.0 / 3.0);
    let packing = if x > TINY { -(1.0 - x).ln() / x } else { 1.0 };
    packing
        * (w / 4.0)
        * (phase_function * (1.0 + hotspot)
            + h_function(mu0, w, packing) * h_function(mu, w, packing)
            - 1.0)
        / (mu0 + mu).max(TINY)
}

fn effective_geometry(p: &Input, i: f64, e: f64, psi: f64) -> (f64, f64, f64) {
    let mut m = p.rms_slope;
    if p.roughness_correction == Roughness::HapkeMulti {
        m = (PI / 2.0).sqrt()
            * ((1.0 - p.multifacet_reflectance) * (m * (2.0 / PI).sqrt()).atan()).tan();
    }
    if m.abs() < TINY {
        return (i, e, 1.0);
    }
    let slope = (m * (2.0 / PI).sqrt()).atan();
    let t = slope.tan().max(TINY);
    let chi = 1.0 / (1.0 + PI * t * t).sqrt();
    let f1 = |angle: f64| (-2.0 / (PI * t * angle.tan().max(TINY))).exp();
    let f2 = |angle: f64| (-1.0 / (PI * t * t * angle.tan().max(TINY).powi(2))).exp();
    let eta = |angle: f64| {
        chi * (angle.cos() + angle.sin() * t * f2(angle) / (2.0 - f1(angle)).max(TINY))
    };
    let sin_half_squared = (psi / 2.0).sin().powi(2);
    let (mu0_eff, mu_eff) = if i < e {
        let d = (2.0 - f1(e) - (psi / PI) * f1(i)).max(TINY);
        (
            chi * (i.cos() + i.sin() * t * (psi.cos() * f2(e) + sin_half_squared * f2(i)) / d),
            chi * (e.cos() + e.sin() * t * (f2(e) - sin_half_squared * f2(i)) / d),
        )
    } else {
        let d = (2.0 - f1(i) - (psi / PI) * f1(e)).max(TINY);
        (
            chi * (i.cos() + i.sin() * t * (f2(i) - sin_half_squared * f2(e)) / d),
            chi * (e.cos() + e.sin() * t * (psi.cos() * f2(i) + sin_half_squared * f2(e)) / d),
        )
    };
    let eta_i = eta(i).max(TINY);
    let eta_e = eta(e).max(TINY);
    let f = (-2.0 * (psi / 2.0).tan()).exp();
    let term = if i < e {
        i.cos() / eta_i
    } else {
        e.cos() / eta_e
    };
    let denominator = (1.0 - f + f * chi * term).max(TINY);
    let shadow = ((mu_eff / eta_e) * (i.cos() / eta_i) * chi / denominator).max(0.0);
    (
        mu0_eff.clamp(0.0, 1.0).acos(),
        mu_eff.clamp(0.0, 1.0).acos(),
        shadow,
    )
}

pub fn directional(p: &Input, w: f64, i: f64, e: f64, psi: f64) -> f64 {
    let phase = (i.cos() * e.cos() + i.sin() * e.sin() * psi.cos())
        .clamp(-1.0, 1.0)
        .acos();
    if p.roughness_correction == Roughness::None {
        return flat(p, w, i, e, phase);
    }
    let (a, b, shadow) = effective_geometry(p, i, e, psi);
    flat(p, w, a, b, phase) * shadow * a.cos() / i.cos()
}

/// Generalized exponential integral: twice E3(x), including the exact x=0 limit.
pub fn diffuse_transmission(x: f64) -> f64 {
    if x == 0.0 {
        return 1.0;
    }
    if x <= 1.0 {
        let mut factorial_term = -x;
        let mut sum = factorial_term;
        for k in 2..=200 {
            factorial_term *= -x / k as f64;
            let term = factorial_term / k as f64;
            sum += term;
            if term.abs() <= f64::EPSILON * sum.abs() {
                break;
            }
        }
        let e1 = -0.577_215_664_901_532_9 - x.ln() - sum;
        (1.0 - x) * (-x).exp() + x * x * e1
    } else {
        let mut b = x + 3.0;
        let mut c = 1e300;
        let mut d = 1.0 / b;
        let mut h = d;
        for k in 1..=200 {
            let a = -(k as f64) * (2.0 + k as f64);
            b += 2.0;
            d = 1.0 / (a * d + b);
            c = b + a / c;
            let delta = c * d;
            h *= delta;
            if (delta - 1.0).abs() <= 4.0 * f64::EPSILON {
                break;
            }
        }
        2.0 * h * (-x).exp()
    }
}

fn film_optics(s: &Sample, p: &Input) -> (f64, f64) {
    let wl_cm = s.wavelength_nm * 1e-7;
    let kw = s.water_absorption * wl_cm / (4.0 * PI);
    let nw = Complex64::new(s.water_index, kw);
    let ni = Complex64::new(
        p.particle_refractive_index,
        p.particle_extinction_coefficient,
    );
    let n = ((1.0 - p.particle_volume_fraction) * nw * nw + p.particle_volume_fraction * ni * ni)
        .sqrt();
    (n.re, 4.0 * PI * n.im / wl_cm)
}

fn fresnel(n1: f64, n2: f64, i: f64, t: f64) -> f64 {
    let ci = i.cos().max(0.0);
    let ct = t.cos().max(0.0);
    let rs = (n1 * ci - n2 * ct) / (n1 * ci + n2 * ct).max(TINY);
    let rp = (n2 * ci - n1 * ct) / (n2 * ci + n1 * ct).max(TINY);
    (0.5 * (rs * rs + rp * rp)).clamp(0.0, 1.0)
}

fn feedback(value: f64) -> Result<f64, ModelError> {
    let d = 1.0 - value;
    if !d.is_finite() || d <= TINY {
        return Err(domain("water-film feedback is nonpositive or singular"));
    }
    Ok(d)
}

fn mix(dry: f64, wet: f64, coverage: f64) -> Result<f64, ModelError> {
    checked(wet, false)?;
    if coverage == 0.0 {
        return Ok(dry);
    }
    if coverage == 1.0 {
        return Ok(wet);
    }
    checked(
        (coverage * wet.max(0.0).powf(1.0 / 2.27)
            + (1.0 - coverage) * dry.max(0.0).powf(1.0 / 2.27))
        .powf(2.27),
        false,
    )
}

pub fn wet_directional(s: &Sample, p: &Input, w: f64, dry: f64) -> Result<f64, ModelError> {
    if p.water_coverage == 0.0 || p.water_film_thickness_mm == 0.0 {
        return Ok(dry);
    }
    let (index, alpha) = film_optics(s, p);
    let n = index.max(1.0 + 1e-6);
    let i = p.solar_zenith_deg.to_radians();
    let e = p.view_zenith_deg.to_radians();
    let it = (i.sin() / n).clamp(-1.0, 1.0).asin();
    let et = (e.sin() / n).clamp(-1.0, 1.0).asin();
    let r12 = fresnel(1.0, n, i, it);
    let r21 = fresnel(n, 1.0, et, e);
    let length = p.water_film_thickness_mm * 0.1;
    let ts = diffuse_transmission(alpha * length / it.cos().max(TINY));
    let tv = diffuse_transmission(alpha * length / et.cos().max(TINY));
    let substrate = checked(
        directional(p, w, it, et, p.relative_azimuth_deg.to_radians()),
        false,
    )?;
    let x = substrate * ts * tv;
    mix(
        dry,
        (1.0 - r12) * (1.0 - r21) * x / feedback(r21 * x)?,
        p.water_coverage,
    )
}

pub fn wet_lambert(s: &Sample, p: &Input, dry: f64) -> Result<f64, ModelError> {
    if p.water_coverage == 0.0 || p.water_film_thickness_mm == 0.0 {
        return Ok(dry);
    }
    let (n, alpha) = film_optics(s, p);
    let n2 = n * n;
    let r12 = (3.0 * n2 + 2.0 * n + 1.0) / (3.0 * (n + 1.0).powi(2))
        - 2.0 * n.powi(3) * (n2 + 2.0 * n - 1.0) / ((n2 + 1.0).powi(2) * (n2 - 1.0))
        + n2 * (n2 + 1.0) * n.ln() / (n2 - 1.0).powi(2)
        - n2 * (n2 - 1.0).powi(2) * (n * (n + 1.0) / (n - 1.0)).ln() / (n2 + 1.0).powi(3);
    let r21 = 1.0 - (1.0 - r12) / n2;
    checked(r12, true)?;
    checked(r21, true)?;
    let t = diffuse_transmission(alpha * p.water_film_thickness_mm * 0.1);
    let x = dry * t * t;
    checked(
        mix(
            dry,
            (1.0 - r12) * (1.0 - r21) * x / feedback(r21 * x)?,
            p.water_coverage,
        )?,
        true,
    )
}

fn gauss_legendre(order: usize) -> Vec<(f64, f64)> {
    let mut values = vec![(0.0, 0.0); order];
    for j in 0..order.div_ceil(2) {
        let mut z = (PI * (j as f64 + 0.75) / (order as f64 + 0.5)).cos();
        let mut derivative = 0.0;
        for _ in 0..64 {
            let (mut p0, mut p1) = (1.0, z);
            for k in 2..=order {
                let pn = ((2 * k - 1) as f64 * z * p1 - (k - 1) as f64 * p0) / k as f64;
                p0 = p1;
                p1 = pn;
            }
            derivative = order as f64 * (z * p1 - p0) / (z * z - 1.0);
            let step = p1 / derivative;
            z -= step;
            if step.abs() < 2e-15 {
                break;
            }
        }
        let weight = 2.0 / ((1.0 - z * z) * derivative * derivative);
        values[j] = (-z, weight);
        values[order - 1 - j] = (z, weight);
    }
    values
}

type Quadrature = Vec<(Vec<(f64, f64)>, Vec<(f64, f64)>)>;
static QUADRATURE: OnceLock<Quadrature> = OnceLock::new();

pub fn screen_energy(p: &Input, max_ssa: f64) -> Result<(), ModelError> {
    let rules = QUADRATURE.get_or_init(|| {
        [24, 48, 96, 192]
            .iter()
            .map(|&n| (gauss_legendre(n), gauss_legendre(2 * n)))
            .collect()
    });
    for incidence in [p.solar_zenith_deg, p.view_zenith_deg] {
        let mut previous: Option<f64> = None;
        let mut accepted = false;
        for (mu_rule, phi_rule) in rules {
            let mut dhr = 0.0;
            for &(node, weight) in mu_rule {
                let mu = (node + 1.0) * 0.5;
                for &(azimuth, azimuth_weight) in phi_rule {
                    let phi = (azimuth + 1.0) * PI * 0.5;
                    dhr += weight
                        * 0.5
                        * azimuth_weight
                        * mu
                        * directional(p, max_ssa, incidence.to_radians(), mu.acos(), phi);
                }
            }
            if !dhr.is_finite() {
                return Err(domain(
                    "nonfinite dry directional-hemispherical reflectance",
                ));
            }
            if let Some(old) = previous {
                let difference = (dhr - old).abs();
                let tolerance = if dhr > 0.98 { 1e-4 } else { 2e-3 };
                if difference <= tolerance {
                    if dhr + difference > 1.0 + 1e-6 {
                        return Err(domain(
                            "dry soil parameters fail directional-hemispherical energy screening; no BRF clipping was applied",
                        ));
                    }
                    accepted = true;
                    break;
                }
            }
            previous = Some(dhr);
        }
        if !accepted {
            return Err(ModelError::new(
                ErrorKind::NonConvergence,
                "dry soil energy quadrature did not converge through order 192",
            ));
        }
    }
    Ok(())
}
