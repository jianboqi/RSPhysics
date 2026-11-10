//! Gaussian quadrature on [0,1] for hemispherical optical integrals.
use std::f64::consts::PI;
/// Positive order; callers use fixed model-owned orders.
pub fn gauss_unit(n: usize) -> Vec<(f64, f64)> {
    assert!(n > 0);
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let mut z = (PI * (i as f64 + 0.75) / (n as f64 + 0.5)).cos();
        let derivative = loop {
            let mut a = 1.;
            let mut b = z;
            for j in 2..=n {
                let c = ((2 * j - 1) as f64 * z * b - (j - 1) as f64 * a) / j as f64;
                a = b;
                b = c;
            }
            let d = n as f64 * (z * b - a) / (z * z - 1.);
            let next = z - b / d;
            if (next - z).abs() < 2e-15 {
                break d;
            }
            z = next;
        };
        out.push(((z + 1.) / 2., 1. / ((1. - z * z) * derivative * derivative)));
    }
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn moments() {
        for n in [8, 32, 64] {
            let q = gauss_unit(n);
            for k in 0..10 {
                let s: f64 = q.iter().map(|(x, w)| w * x.powi(k)).sum();
                assert!((s - 1. / f64::from(k + 1)).abs() < 2e-14);
            }
        }
    }
}
