//! Shiltz–Bachmann slope integral and Heitz shadow probability.
use crate::solver::flat;
use crate::{Input, Roughness};
use rsphysics_model_api::{ErrorKind, ModelError};
use std::f64::consts::PI;
fn lambda(nu: f64) -> f64 {
    if nu.is_infinite() {
        0.
    } else {
        (-nu * nu).exp() / (2. * PI.sqrt() * nu) - 0.5 * libm::erfc(nu)
    }
}
fn log_add_exp(x: f64) -> f64 {
    if x > 0. {
        x + (-x).exp().ln_1p()
    } else {
        x.exp().ln_1p()
    }
}
fn shadow(i: f64, e: f64, psi: f64, m: f64) -> f64 {
    let a = 1. / i.max(e).tan() / (2_f64.sqrt() * m);
    let b = 1. / i.min(e).tan() / (2_f64.sqrt() * m);
    let correlation = if psi == 0. {
        0.
    } else if psi >= PI / 2. || a.is_infinite() && b.is_infinite() {
        1.
    } else {
        let d = (a - b).abs();
        if d <= 1e-12 {
            1.
        } else {
            let loga = 0.17_f64.ln() - 10.49 * d.ln();
            let denominator = log_add_exp(loga + 8.85 * (PI / 2.).ln());
            if denominator.abs() <= 1e-12 {
                0.01837972835 * psi.powf(8.85)
            } else {
                log_add_exp(loga + 8.85 * psi.ln()) / denominator
            }
        }
    }
    .clamp(0., 1.);
    1. / (1. + lambda(a) + correlation * lambda(b))
}
pub fn at_order(p: &Input, w: f64, i: f64, e: f64, mut psi: f64, n: usize) -> f64 {
    let m = p.rms_slope;
    if i.max(e) < 1e-12 {
        psi = 0.;
    }
    let phase = (i.cos() * e.cos() + i.sin() * e.sin() * psi.cos())
        .clamp(-1., 1.)
        .acos();
    if m <= 1e-12 {
        return flat(p, w, i, e, phase);
    }
    let step = 10. * m / (n - 1) as f64;
    let mut integral = 0.;
    for x in 0..n {
        let mx = -5. * m + x as f64 * step;
        let wx = if x == 0 || x == n - 1 { 0.5 } else { 1. };
        for y in 0..n {
            let my = -5. * m + y as f64 * step;
            let wy = if y == 0 || y == n - 1 { 0.5 } else { 1. };
            let mi = (psi / 2.).cos() * mx + (psi / 2.).sin() * my;
            let me = (psi / 2.).cos() * mx - (psi / 2.).sin() * my;
            if 1. / i.tan() - mi < 0. || 1. / e.tan() - me < 0. {
                continue;
            }
            let tilt = 1. / (mx * mx + my * my + 1.).sqrt();
            let ci = ((i.cos() - mi * i.sin()) * tilt).clamp(-1., 1.);
            let ce = ((e.cos() - me * e.sin()) * tilt).clamp(-1., 1.);
            let density = (-(mx * mx + my * my) / (2. * m * m)).exp() / (2. * PI * m * m);
            integral += wx
                * wy
                * (1. - me * e.tan()).max(0.)
                * density
                * ci.max(0.)
                * flat(p, w, ci.acos(), ce.acos(), phase);
        }
    }
    let mut value = shadow(i, e, psi, m) * integral * step * step / i.cos();
    if p.roughness_correction == Roughness::ShiltzFull {
        value += 0.19
            * m
            * p.multifacet_reflectance
            * (1. + 6.5 * (-4. / PI * (PI - phase).powi(2)).exp());
    }
    value
}
pub fn converged(p: &Input, w: f64, i: f64, e: f64, psi: f64) -> Result<f64, ModelError> {
    let mut previous: Option<f64> = None;
    for n in [100, 200, 400, 800] {
        let v = at_order(p, w, i, e, psi, n);
        if let Some(old) = previous
            && (v - old).abs() <= 1e-6 + 2e-4 * v.abs()
        {
            return Ok(v);
        }
        previous = Some(v);
    }
    Err(ModelError::new(
        ErrorKind::NonConvergence,
        "Shiltz slope integral did not converge through N=800",
    ))
}
