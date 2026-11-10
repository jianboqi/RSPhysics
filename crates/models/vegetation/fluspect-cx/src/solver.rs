// FLUSPECT-CX equations adapted from the authors' SCOPE implementation (GPLv3).
// Verhoef, van der Tol, Timmermans and Vilfan. See component notices.
use crate::{Input, Output};

// The E1 numerical helper follows the MIT-origin RSPhysics PROSPECT implementation.
fn e1(x: f64) -> f64 {
    if x <= 1. {
        let mut term = -x;
        let mut sum = term;
        for k in 2..=200 {
            term *= -x / k as f64;
            let delta = term / k as f64;
            sum += delta;
            if delta.abs() <= f64::EPSILON * sum.abs() {
                break;
            }
        }
        -0.577_215_664_901_532_9 - x.ln() - sum
    } else {
        let mut b = x + 1.;
        let mut c = 1e300;
        let mut d = 1. / b;
        let mut h = d;
        for k in 1..=200 {
            let a = -((k * k) as f64);
            b += 2.;
            d = 1. / (a * d + b);
            c = b + a / c;
            let delta = c * d;
            h *= delta;
            if (delta - 1.).abs() <= 4. * f64::EPSILON {
                break;
            }
        }
        h * (-x).exp()
    }
}
fn tav(alpha: f64, n: f64) -> f64 {
    let n2 = n * n;
    let np = n2 + 1.;
    let nm = n2 - 1.;
    let a = (n + 1.).powi(2) / 2.;
    let k = -nm * nm / 4.;
    let sa = alpha.to_radians().sin();
    let b2 = sa * sa - np / 2.;
    let b1 = if alpha == 90. {
        0.
    } else {
        (b2 * b2 + k).max(0.).sqrt()
    };
    let b = b1 - b2;
    let ts =
        (k * k / (6. * b.powi(3)) + k / b - b / 2.) - (k * k / (6. * a.powi(3)) + k / a - a / 2.);
    let tp1 = -2. * n2 * (b - a) / np.powi(2);
    let tp2 = -2. * n2 * np * (b / a).ln() / nm.powi(2);
    let tp3 = n2 * (1. / b - 1. / a) / 2.;
    let tp4 = 16.
        * n2.powi(2)
        * (n2.powi(2) + 1.)
        * ((2. * np * b - nm * nm) / (2. * np * a - nm * nm)).ln()
        / (np.powi(3) * nm.powi(2));
    let tp5 = 16. * n2.powi(3) * (1. / (2. * np * b - nm * nm) - 1. / (2. * np * a - nm * nm))
        / np.powi(3);
    let tp = tp1 + tp2 + tp3 + tp4 + tp5;
    (ts + tp) / (2. * sa * sa)
}
struct Band {
    refl: f64,
    tran: f64,
    chl: f64,
    car: f64,
    k: f64,
    s: f64,
    kchl: f64,
    rho: f64,
    tau: f64,
    r21: f64,
    t21: f64,
    talf: f64,
    phi: f64,
}
fn band(p: &Input, c: &[f64; 13]) -> Band {
    let [
        _,
        nr,
        kab,
        kca_fixed,
        ks,
        kw,
        kdm,
        kcav,
        kcaz,
        kant,
        phi,
        kp,
        kcbc,
    ] = *c;
    let kca = if p.xanthophyll_state == -999. {
        kca_fixed
    } else {
        (1. - p.xanthophyll_state) * kcav + p.xanthophyll_state * kcaz
    };
    let kall = (p.chlorophyll * kab
        + p.carotenoids * kca
        + p.dry_matter * kdm
        + p.water * kw
        + p.brown_pigments * ks
        + p.anthocyanins * kant
        + p.protein * kp
        + p.carbon_based_constituents * kcbc)
        / p.n;
    let tint = if kall > 0. {
        (1. - kall) * (-kall).exp() + kall * kall * e1(kall)
    } else {
        1.
    };
    let (chl, car) = if kall > 0. {
        (
            p.chlorophyll * kab / (kall * p.n),
            p.carotenoids * kca / (kall * p.n),
        )
    } else {
        (0., 0.)
    };
    let talf = tav(59., nr);
    let ralf = 1. - talf;
    let t12 = tav(90., nr);
    let r12 = 1. - t12;
    let t21 = t12 / (nr * nr);
    let r21 = 1. - t21;
    let den = 1. - r21 * r21 * tint * tint;
    let ta = talf * tint * t21 / den;
    let ra = ralf + r21 * tint * ta;
    let t = t12 * tint * t21 / den;
    let r = r12 + r21 * tint * t;
    let d = ((1. + r + t) * (1. + r - t) * (1. - r + t) * (1. - r - t))
        .max(0.)
        .sqrt();
    let a = (1. + r * r - t * t + d) / (2. * r);
    let b = (1. - r * r + t * t + d) / (2. * t);
    let bn = b.powf(p.n - 1.);
    let bn2 = bn * bn;
    let a2 = a * a;
    // Exact conservative limit avoids an ill-conditioned 0/0 in the Stokes form.
    let (rsub, tsub) = if kall <= 0. || r + t >= 1. {
        let tsub = t / (t + (1. - t) * (p.n - 1.));
        (1. - tsub, tsub)
    } else {
        (
            a * (bn2 - 1.) / (a2 * bn2 - 1.),
            bn * (a2 - 1.) / (a2 * bn2 - 1.),
        )
    };
    let den = 1. - rsub * r;
    let tran = ta * tsub / den;
    let refl = ra + ta * rsub * t / den;
    let rb = (refl - ralf) / (talf * t21 + (refl - ralf) * r21);
    let z = tran * (1. - rb * r21) / (talf * t21);
    let rho = (rb - r21 * z * z) / (1. - (r21 * z).powi(2));
    let tau = (1. - rb * r21) / (1. - (r21 * z).powi(2)) * z;
    let r = rho.max(0.);
    let t = tau;
    let (a, b) = if r + t < 1. {
        let d = ((1. + r + t) * (1. + r - t) * (1. - r + t) * (1. - r - t))
            .max(0.)
            .sqrt();
        (
            (1. + r * r - t * t + d) / (2. * r),
            (1. - r * r + t * t + d) / (2. * t),
        )
    } else {
        (1., 1.)
    };
    let (s, k) = if a > 1. && a.is_finite() {
        (2. * a / (a * a - 1.) * b.ln(), (a - 1.) / (a + 1.) * b.ln())
    } else {
        (r / t, b.ln())
    };
    Band {
        refl,
        tran,
        chl,
        car,
        k,
        s,
        kchl: chl * k,
        rho,
        tau,
        r21,
        t21,
        talf,
        phi: phi.max(0.),
    }
}
pub(super) fn simulate(p: &Input, coefficients: &[[f64; 13]]) -> Output {
    let bands: Vec<_> = coefficients.iter().map(|c| band(p, c)).collect();
    let (ne, nf, excitation_step, emission_step) = p.spectral_resolution.grid();
    let ei: Vec<usize> = (0..ne).map(|i| i * excitation_step).collect();
    let fi: Vec<usize> = (0..nf).map(|i| 240 + i * emission_step).collect();
    let mut mb = vec![vec![0.; ne]; nf];
    let mut mf = mb.clone();
    if p.fluorescence_efficiency > 0. {
        let eps = 2_f64.powi(-15);
        let thin = |indices: &[usize]| -> (Vec<f64>, Vec<f64>) {
            (
                indices
                    .iter()
                    .map(|&i| 1. - (bands[i].k + bands[i].s) * eps)
                    .collect(),
                indices.iter().map(|&i| bands[i].s * eps).collect(),
            )
        };
        let (mut te, mut re) = thin(&ei);
        let (mut tf, mut rf) = thin(&fi);
        for (f, &j) in fi.iter().enumerate() {
            for (e, &i) in ei.iter().enumerate() {
                let sigmoid = 1.
                    / (1. + (-((400 + j) as f64) / 10.).exp() * (((400 + i) as f64) / 10.).exp());
                mb[f][e] = excitation_step as f64
                    * p.fluorescence_efficiency
                    * (0.5 * bands[j].phi)
                    * eps
                    * bands[i].kchl
                    * sigmoid;
                mf[f][e] = mb[f][e];
            }
        }
        for _ in 0..15 {
            let xe: Vec<_> = te.iter().zip(&re).map(|(t, r)| t / (1. - r * r)).collect();
            let xf: Vec<_> = tf.iter().zip(&rf).map(|(t, r)| t / (1. - r * r)).collect();
            for f in 0..nf {
                for e in 0..ne {
                    let x = xf[f] * xe[e];
                    let a11 = xf[f] + xe[e];
                    let a12 = x * (rf[f] + re[e]);
                    let a21 = 1. + x * (1. + rf[f] * re[e]);
                    let a22 = xf[f] * rf[f] + xe[e] * re[e];
                    let forward = mf[f][e];
                    let backward = mb[f][e];
                    mf[f][e] = forward * a11 + backward * a12;
                    mb[f][e] = backward * a21 + forward * a22;
                }
            }
            for i in 0..ne {
                te[i] *= xe[i];
                re[i] *= 1. + te[i];
            }
            for i in 0..nf {
                tf[i] *= xf[i];
                rf[i] *= 1. + tf[i];
            }
        }
        for (f, &j) in fi.iter().enumerate() {
            for (e, &i) in ei.iter().enumerate() {
                let ex = &bands[i];
                let em = &bands[j];
                let rbe = ex.rho + ex.tau * ex.tau * ex.r21 / (1. - ex.rho * ex.r21);
                let rbf = em.rho + em.tau * em.tau * em.r21 / (1. - em.rho * em.r21);
                let xe = ex.talf / (1. - ex.r21 * rbe);
                let xf = em.t21 / (1. - em.r21 * rbf);
                let ye = ex.tau * ex.r21 / (1. - ex.rho * ex.r21);
                let yf = em.tau * em.r21 / (1. - em.rho * em.r21);
                let a = xe * (1. + ye * yf) * xf;
                let b = xe * (ye + yf) * xf;
                let g = mb[f][e];
                let ff = mf[f][e];
                mb[f][e] = a * g + b * ff;
                mf[f][e] = a * ff + b * g;
            }
        }
    }
    Output {
        wavelengths_nm: coefficients.iter().map(|c| c[0]).collect(),
        reflectance: bands.iter().map(|b| b.refl).collect(),
        transmittance: bands.iter().map(|b| b.tran).collect(),
        chlorophyll_absorption_fraction: bands.iter().map(|b| b.chl).collect(),
        carotenoid_absorption_fraction: bands.iter().map(|b| b.car).collect(),
        excitation_wavelengths_nm: ei.iter().map(|i| 400. + *i as f64).collect(),
        emission_wavelengths_nm: fi.iter().map(|i| 400. + *i as f64).collect(),
        fluorescence_backward: mb,
        fluorescence_forward: mf,
    }
}
