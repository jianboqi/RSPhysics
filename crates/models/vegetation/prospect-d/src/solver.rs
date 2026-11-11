// Scientific equations adapted from the authors' prospect v1.7.0, R/Lib_PROSPECT.R.
// Copyright 2020 Jean-Baptiste FERET. MIT declaration in that release's DESCRIPTION.
// Original implementation also credits the FLUSPECT optimizations of Verhoef,
// van der Tol and Timmermans. Rust adaptation: RSPhysics contributors, 2026-11-09.
use crate::{Coefficient, Input};

/// E1(x) = integral_x^infinity exp(-t)/t dt, for positive x.
/// Convergent power series on (0,1], modified-Lentz continued fraction above 1.
/// No external native numerical library is needed by the Python wheel.
fn exponential_integral_e1(x: f64) -> f64 {
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
        -0.577_215_664_901_532_9 - x.ln() - sum
    } else {
        let mut b = x + 1.0;
        let mut c = 1e300;
        let mut d = 1.0 / b;
        let mut h = d;
        for k in 1..=200 {
            let a = -((k * k) as f64);
            b += 2.0;
            d = 1.0 / (a * d + b);
            c = b + a / c;
            let delta = c * d;
            h *= delta;
            if (delta - 1.0).abs() <= 4.0 * f64::EPSILON {
                break;
            }
        }
        h * (-x).exp()
    }
}

/// Angularly averaged dielectric transmission (Stern/Allen), alpha in degrees.
pub(crate) fn calctav(alpha: f64, nr: f64) -> f64 {
    let n2 = nr * nr;
    let np = n2 + 1.0;
    let nm = n2 - 1.0;
    let a = (nr + 1.0) * (nr + 1.0) / 2.0;
    let k = -(n2 - 1.0) * (n2 - 1.0) / 4.0;
    let sa = (alpha * (std::f64::consts::PI / 180.0)).sin();
    let b2 = sa * sa - np / 2.0;
    let b1 = if alpha == 90.0 {
        0.0
    } else {
        (b2 * b2 + k).sqrt()
    };
    let b = b1 - b2;
    let ts = (k * k / (6.0 * b.powi(3)) + k / b - b / 2.0)
        - (k * k / (6.0 * a.powi(3)) + k / a - a / 2.0);
    let tp1 = -2.0 * n2 * (b - a) / np.powi(2);
    let tp2 = -2.0 * n2 * np * (b / a).ln() / nm.powi(2);
    let tp3 = n2 * (1.0 / b - 1.0 / a) / 2.0;
    let tp4 = 16.0
        * n2.powi(2)
        * (n2.powi(2) + 1.0)
        * ((2.0 * np * b - nm.powi(2)) / (2.0 * np * a - nm.powi(2))).ln()
        / (np.powi(3) * nm.powi(2));
    let tp5 =
        16.0 * n2.powi(3) * (1.0 / (2.0 * np * b - nm.powi(2)) - 1.0 / (2.0 * np * a - nm.powi(2)))
            / np.powi(3);
    let tp = tp1 + tp2 + tp3 + tp4 + tp5;
    (ts + tp) / (2.0 * (sa * sa))
}

pub(crate) fn solve_band(input: &Input, coefficient: &Coefficient) -> (f64, f64) {
    let contents = [
        input.chlorophyll,
        input.carotenoids,
        input.anthocyanins,
        input.brown_pigments,
        input.water,
        input.dry_matter,
    ];
    let absorption: f64 = contents
        .iter()
        .zip(coefficient.absorption)
        .map(|(c, a)| c * a)
        .sum::<f64>()
        / input.n;
    let talf = if input.incidence_angle_deg == 40.0 {
        coefficient.tav_40
    } else {
        calctav(input.incidence_angle_deg, coefficient.refractive_index)
    };
    solve_stack(
        input.n,
        absorption,
        coefficient.refractive_index,
        talf,
        coefficient.tav_90,
    )
}

pub(crate) fn solve_stack(
    n: f64,
    absorption: f64,
    refractive_index: f64,
    talf: f64,
    t12: f64,
) -> (f64, f64) {
    let tau = if absorption <= 0.0 {
        1.0
    } else if absorption >= 700.0 {
        0.0
    } else {
        ((1.0 - absorption) * (-absorption).exp()
            + absorption * absorption * exponential_integral_e1(absorption))
        .max(0.0)
    };
    let t21 = t12 / refractive_index.powi(2);
    let r21 = 1.0 - t21;
    let denom = 1.0 - (r21 * r21 * (tau * tau));
    let ta = talf * tau * t21 / denom;
    let ra = 1.0 - talf + r21 * tau * ta;
    let t = t12 * tau * t21 / denom;
    let r = 1.0 - t12 + r21 * tau * t;
    if n == 1.0 || t == 0.0 {
        return (ra, ta);
    }
    let (rsub, tsub) = if r + t >= 1.0 {
        let tsub = t / (t + (1.0 - t) * (n - 1.0));
        (1.0 - tsub, tsub)
    } else {
        let d = ((1.0 + r + t) * (1.0 + r - t) * (1.0 - r + t) * (1.0 - r - t)).sqrt();
        let a = (1.0 + r * r - t * t + d) / (2.0 * r);
        let b_inverse = (2.0 * t) / (1.0 - r * r + t * t + d);
        let bnm1 = b_inverse.powf(n - 1.0);
        let bn2 = bnm1 * bnm1;
        let a2 = a * a;
        let denom = a2 - bn2;
        (a * (1.0 - bn2) / denom, bnm1 * (a2 - 1.0) / denom)
    };
    let denom = 1.0 - rsub * r;
    (ra + ta * rsub * t / denom, ta * tsub / denom)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn e1_matches_independent_high_precision_constants() {
        for (x, expected) in [
            (0.1, 1.822_923_958_419_390_6),
            (1.0, 0.219_383_934_395_520_29),
            (2.0, 0.048_900_510_708_061_12),
            (10.0, 4.156_968_929_685_325e-6),
        ] {
            assert!((exponential_integral_e1(x) - expected).abs() < expected * 1e-13);
        }
    }
}
