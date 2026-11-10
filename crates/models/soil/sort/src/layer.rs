//! Directional water-film adding, including the (-1)^m interface reversal.
use crate::solver::{directional, gauss_legendre};
use crate::{Input, domain};
use rsphysics_model_api::{ErrorKind, ModelError};
use std::f64::consts::PI;
fn solve(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Result<Vec<f64>, ModelError> {
    let n = b.len();
    for k in 0..n {
        let pivot = (k..n)
            .max_by(|&i, &j| a[i][k].abs().total_cmp(&a[j][k].abs()))
            .unwrap();
        if a[pivot][k].abs() < 1e-14 {
            return Err(domain("singular internal-film transport operator"));
        }
        a.swap(k, pivot);
        b.swap(k, pivot);
        for i in k + 1..n {
            let factor = a[i][k] / a[k][k];
            let (upper, lower) = a.split_at_mut(i);
            for (value, pivot_value) in lower[0].iter_mut().zip(&upper[k]).skip(k) {
                *value -= factor * pivot_value;
            }
            b[i] -= factor * b[k];
        }
    }
    let mut x = vec![0.; n];
    for i in (0..n).rev() {
        x[i] = (b[i] - (i + 1..n).map(|j| a[i][j] * x[j]).sum::<f64>()) / a[i][i];
    }
    Ok(x)
}
#[allow(clippy::too_many_arguments)]
fn at_order(
    p: &Input,
    w: f64,
    index: f64,
    tau: f64,
    i: f64,
    e: f64,
    az: f64,
    order: usize,
    nphi: usize,
) -> Result<f64, ModelError> {
    let critical = (1. - 1. / (index * index)).sqrt();
    let mut mu = vec![];
    let mut wm = vec![];
    let rule = gauss_legendre(order);
    for segment in 0..2 {
        for &(x, weight) in &rule {
            let t = (x + 1.) / 2.;
            let (v, g) = if segment == 0 {
                (t * critical, weight * critical / 2.)
            } else {
                (
                    critical + t * t * (1. - critical),
                    weight * (1. - critical) * t,
                )
            };
            mu.push(v);
            wm.push(g * v);
        }
    }
    let n = mu.len();
    let modes = nphi / 2 + 1;
    let mut cq = vec![vec![vec![0.; n]; n]; modes];
    let mut ci = vec![vec![0.; n]; modes];
    let mut ce = ci.clone();
    for k in 0..nphi {
        let phi = k as f64 * 2. * PI / nphi as f64;
        let folded = phi.cos().clamp(-1., 1.).acos();
        let cosines: Vec<_> = (0..modes)
            .map(|m| (m as f64 * phi).cos() / nphi as f64)
            .collect();
        for a in 0..n {
            let theta = mu[a].acos();
            let vi = directional(p, w, i, theta, folded);
            let ve = directional(p, w, theta, e, folded);
            if !vi.is_finite() || !ve.is_finite() || vi < 0. || ve < 0. {
                return Err(domain("invalid angular film boundary"));
            }
            for m in 0..modes {
                ci[m][a] += vi * cosines[m];
                ce[m][a] += ve * cosines[m];
            }
            for b in 0..n {
                let v = directional(p, w, theta, mu[b].acos(), folded);
                if !v.is_finite() || v < 0. {
                    return Err(domain("invalid angular film boundary"));
                }
                for m in 0..modes {
                    cq[m][a][b] += v * cosines[m];
                }
            }
        }
    }
    let mut d = vec![];
    for (&v, &weight) in mu.iter().zip(&wm) {
        let sin2 = index * index * (1. - v * v);
        let reflect = if sin2 >= 1. {
            1.
        } else {
            let ca = (1. - sin2).sqrt();
            0.5 * (((index * v - ca) / (index * v + ca)).powi(2)
                + ((v - index * ca) / (v + index * ca)).powi(2))
        };
        d.push(2. * weight * reflect * (-2. * tau / v).exp());
    }
    for row in &cq[0] {
        if 2. * row.iter().zip(&wm).map(|(x, g)| x * g).sum::<f64>() > 1.002 {
            return Err(domain("film substrate violates hemispherical energy"));
        }
        if row.iter().zip(&d).map(|(x, g)| x * g).sum::<f64>() >= 1. - 1e-10 {
            return Err(domain("internal film series is not contractive"));
        }
    }
    let mut correction = 0.;
    for m in 0..modes {
        let sign = if m % 2 == 0 { 1. } else { -1. };
        let matrix = (0..n)
            .map(|a| {
                (0..n)
                    .map(|b| f64::from(a == b) - cq[m][a][b] * d[b] * sign)
                    .collect()
            })
            .collect();
        let solution = solve(matrix, ce[m].clone())?;
        let factor = if m == 0 || m == nphi / 2 { 1. } else { 2. };
        correction += factor
            * (m as f64 * az).cos()
            * ci[m]
                .iter()
                .zip(&d)
                .zip(solution)
                .map(|((c, g), s)| c * g * sign * s)
                .sum::<f64>();
    }
    Ok(directional(p, w, i, e, az) + correction)
}
#[allow(clippy::too_many_arguments)]
pub fn adding(
    p: &Input,
    w: f64,
    index: f64,
    tau: f64,
    i: f64,
    e: f64,
    az: f64,
) -> Result<f64, ModelError> {
    let mut previous: Option<f64> = None;
    for (order, nphi) in [(4, 16), (8, 32), (16, 64), (32, 128)] {
        let v = at_order(p, w, index, tau, i, e, az, order, nphi)?;
        if let Some(old) = previous
            && (v - old).abs() <= 2e-5 + 2e-3 * v.abs()
        {
            return crate::solver::checked(v, false);
        }
        previous = Some(v);
    }
    Err(ModelError::new(
        ErrorKind::NonConvergence,
        "angular film adding did not converge",
    ))
}
