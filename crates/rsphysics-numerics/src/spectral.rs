//! Piecewise-linear spectra and exact integration of their products.
//! Two-point Gauss quadrature on the union of all knots integrates the
//! signal * response * optional irradiance cubic exactly (up to f64 rounding).
use rsphysics_core::{DomainError, SpectralAxis};

fn axis(x: &[f64]) -> Result<(), DomainError> {
    SpectralAxis::new(x.to_vec()).map(|_| ())
}
fn finite(y: &[f64], n: usize) -> Result<(), DomainError> {
    if y.len() != n || y.iter().any(|v| !v.is_finite()) {
        return Err(DomainError("finite values must match the wavelength axis"));
    }
    Ok(())
}
fn bracket(x: &[f64], q: f64) -> (usize, usize, f64) {
    match x.binary_search_by(|v| v.total_cmp(&q)) {
        Ok(i) => (i, i, 0.),
        Err(i) => (i - 1, i, (q - x[i - 1]) / (x[i] - x[i - 1])),
    }
}
fn value(x: &[f64], y: &[f64], q: f64) -> f64 {
    let (i, j, t) = bracket(x, q);
    (1. - t) * y[i] + t * y[j]
}
fn knots(lo: f64, hi: f64, axes: &[&[f64]]) -> Vec<f64> {
    let mut k = vec![lo, hi];
    for x in axes {
        k.extend(x.iter().copied().filter(|v| *v > lo && *v < hi));
    }
    k.sort_by(f64::total_cmp);
    k.dedup();
    k
}
fn integrate_product(k: &[f64], f: impl Fn(f64) -> f64) -> f64 {
    let mut sum = 0.;
    // Kahan summation reduces loss for nonuniform axes and mixed-sign signals.
    let mut correction = 0.;
    for w in k.windows(2) {
        let mid = w[0] + (w[1] - w[0]) / 2.;
        let half = (w[1] - w[0]) / 2.;
        let d = half / 3_f64.sqrt();
        let term = half * (f(mid - d) + f(mid + d)) - correction;
        let next = sum + term;
        correction = (next - sum) - term;
        sum = next;
    }
    sum
}
pub fn resample(x: &[f64], y: &[f64], target: &[f64]) -> Result<Vec<f64>, DomainError> {
    axis(x)?;
    finite(y, x.len())?;
    axis(target)?;
    if target[0] < x[0] || target[target.len() - 1] > x[x.len() - 1] {
        return Err(DomainError(
            "resampling requires full coverage; extrapolation is disabled",
        ));
    }
    Ok(target.iter().map(|&q| value(x, y, q)).collect())
}
pub fn integrate(x: &[f64], y: &[f64], lo: f64, hi: f64) -> Result<f64, DomainError> {
    axis(x)?;
    finite(y, x.len())?;
    if x.len() < 2
        || !lo.is_finite()
        || !hi.is_finite()
        || lo >= hi
        || lo < x[0]
        || hi > x[x.len() - 1]
    {
        return Err(DomainError(
            "integration interval must have positive width inside the spectral axis",
        ));
    }
    let out = integrate_product(&knots(lo, hi, &[x]), |q| value(x, y, q));
    if !out.is_finite() {
        return Err(DomainError("spectral integral overflow"));
    }
    Ok(out)
}

#[derive(Debug, Clone)]
pub struct BandpassPlan {
    pub weights: Vec<f64>,
    /// Fraction of unweighted response area covered by the supplied spectrum.
    pub coverage: f64,
    /// Full-response centroid, independent of signal and optional illumination.
    pub effective_wavelength_nm: f64,
    /// Integral of peak-normalized full response, in nm.
    pub equivalent_width_nm: f64,
}
impl BandpassPlan {
    pub fn new(
        x: &[f64],
        rx: &[f64],
        response: &[f64],
        irradiance: Option<(&[f64], &[f64])>,
        allow_partial: bool,
    ) -> Result<Self, DomainError> {
        axis(x)?;
        axis(rx)?;
        finite(response, rx.len())?;
        if x.len() < 2 || rx.len() < 2 || response.iter().any(|v| *v < 0.) {
            return Err(DomainError(
                "bandpass requires at least two samples and nonnegative response",
            ));
        }
        let peak = response.iter().copied().fold(0., f64::max);
        if peak <= 0. {
            return Err(DomainError("spectral response must have positive area"));
        }
        let r: Vec<_> = response.iter().map(|v| v / peak).collect();
        let first = r
            .iter()
            .position(|v| *v > 0.)
            .expect("positive peak")
            .saturating_sub(1);
        let last = (r.iter().rposition(|v| *v > 0.).expect("positive peak") + 1).min(rx.len() - 1);
        let a = rx[first];
        let b = rx[last];
        let full = knots(a, b, &[rx]);
        let area = integrate_product(&full, |q| value(rx, &r, q));
        let lo = a.max(x[0]);
        let hi = b.min(x[x.len() - 1]);
        if lo >= hi || !area.is_finite() || area <= 0. {
            return Err(DomainError(
                "response and spectrum have no positive-area overlap",
            ));
        }
        let overlap = integrate_product(&knots(lo, hi, &[rx]), |q| value(rx, &r, q));
        let coverage = (overlap / area).clamp(0., 1.);
        // Endpoint equality is strict: no silent truncation, even for tiny tails.
        if !allow_partial && (lo > a || hi < b) {
            return Err(DomainError(
                "spectrum does not cover the full nonzero response support; allow_partial must be explicit",
            ));
        }
        let effective_wavelength_nm = integrate_product(&full, |q| q * value(rx, &r, q)) / area;
        if !effective_wavelength_nm.is_finite() {
            return Err(DomainError("response centroid overflow"));
        }
        let normalized_irradiance = if let Some((ix, iy)) = irradiance {
            axis(ix)?;
            finite(iy, ix.len())?;
            if ix.len() < 2 || iy.iter().any(|v| *v < 0.) || ix[0] > lo || ix[ix.len() - 1] < hi {
                return Err(DomainError(
                    "irradiance must be nonnegative and cover the integration interval",
                ));
            }
            let max = iy.iter().copied().fold(0., f64::max);
            if max <= 0. {
                return Err(DomainError("irradiance must have positive weight"));
            }
            Some((ix, iy.iter().map(|v| v / max).collect::<Vec<_>>()))
        } else {
            None
        };
        let mut axes = vec![x, rx];
        if let Some((ix, _)) = &normalized_irradiance {
            axes.push(ix);
        }
        let k = knots(lo, hi, &axes);
        let mut weights = vec![0.; x.len()];
        for w in k.windows(2) {
            let half = (w[1] - w[0]) / 2.;
            let mid = w[0] + half;
            let delta = half / 3_f64.sqrt();
            for q in [mid - delta, mid + delta] {
                let mut weight = half * value(rx, &r, q);
                if let Some((ix, iy)) = &normalized_irradiance {
                    weight *= value(ix, iy, q);
                }
                let (i, j, t) = bracket(x, q);
                weights[i] += weight * (1. - t);
                weights[j] += weight * t;
            }
        }
        let norm: f64 = weights.iter().sum();
        if !norm.is_finite() || norm <= 0. {
            return Err(DomainError(
                "response/irradiance overlap has zero or nonfinite weight",
            ));
        }
        for w in &mut weights {
            *w /= norm;
        }
        Ok(Self {
            weights,
            coverage,
            effective_wavelength_nm,
            equivalent_width_nm: area,
        })
    }
    pub fn apply(&self, values: &[f64]) -> Result<f64, DomainError> {
        finite(values, self.weights.len())?;
        let out: f64 = self.weights.iter().zip(values).map(|(w, v)| w * v).sum();
        if !out.is_finite() {
            return Err(DomainError("bandpass result overflow"));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn near(a: f64, b: f64) {
        assert!((a - b).abs() < 2e-12, "{a} != {b}");
    }
    #[test]
    fn exact_linear_and_cubic_products_on_distinct_grids() {
        let p = BandpassPlan::new(
            &[1., 2., 4.],
            &[1., 1.5, 4.],
            &[1., 1.5, 4.],
            Some((&[1., 3., 4.], &[1., 3., 4.])),
            false,
        )
        .unwrap();
        // signal=lambda, response=lambda, irradiance=lambda: integral x^3 / integral x^2.
        near(
            p.apply(&[1., 2., 4.]).unwrap(),
            (4_f64.powi(4) - 1.) / 4. / ((4_f64.powi(3) - 1.) / 3.),
        );
        near(p.apply(&[7., 7., 7.]).unwrap(), 7.);
        near(p.coverage, 1.);
        near(
            p.effective_wavelength_nm,
            ((64. - 1.) / 3.) / ((16. - 1.) / 2.),
        );
        near(
            integrate(&[1., 2., 4.], &[1., 2., 4.], 1.5, 3.5).unwrap(),
            5.,
        );
    }
    #[test]
    fn coverage_and_zero_tails_are_explicit() {
        let rx = [1., 2., 3., 4., 5.];
        let r = [0., 0., 1., 0., 0.];
        assert!(BandpassPlan::new(&[2., 4.], &rx, &r, None, false).is_ok());
        assert!(BandpassPlan::new(&[3., 4.], &rx, &r, None, false).is_err());
        let p = BandpassPlan::new(&[3., 4.], &rx, &r, None, true).unwrap();
        near(p.coverage, 0.5);
        assert!(BandpassPlan::new(&[6., 7.], &rx, &r, None, true).is_err());
        assert!(BandpassPlan::new(&[1., 5.], &rx, &[0.; 5], None, false).is_err());
        assert!(BandpassPlan::new(&[1., 5.], &rx, &[0., -1., 1., 0., 0.], None, false).is_err());
    }
    #[test]
    fn invalid_axes_values_and_extrapolation_fail() {
        assert!(resample(&[1., 2.], &[0., 1.], &[0.9]).is_err());
        assert!(resample(&[1., 1.], &[0., 1.], &[1.]).is_err());
        assert!(resample(&[1., 2.], &[0., f64::NAN], &[1.]).is_err());
        assert_eq!(resample(&[2.], &[3.], &[2.]).unwrap(), vec![3.]);
        assert!(integrate(&[1., 2.], &[0., 1.], 2., 1.).is_err());
    }
}
