//! Four-stream canopy transport shared by optical and thermal forward models.
//! Optical equations adapted from jbferet/prosail commit 46549958 (MIT).
//! Thermal source responses follow Verhoef et al. (2007), Eqs. (1)–(5).
//! See SAIL_NOTICE.md for attribution and component terms.
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LeafAngleDistribution {
    Campbell,
    Verhoef,
}
#[derive(Debug, Clone, Copy)]
pub struct Parameters {
    pub leaf_area_index: f64,
    pub mean_leaf_angle_deg: f64,
    pub leaf_angle_distribution: LeafAngleDistribution,
    pub lidf_a: f64,
    pub lidf_b: f64,
    pub hotspot: f64,
    pub solar_zenith_deg: f64,
    pub view_zenith_deg: f64,
    pub relative_azimuth_deg: f64,
}
#[derive(Debug, Clone, Copy)]
pub struct Coefficients {
    pub rdd: f64,
    pub tdd: f64,
    pub rsd: f64,
    pub tsd: f64,
    pub rdo: f64,
    pub tdo: f64,
    pub tss: f64,
    pub too: f64,
    pub tsstoo: f64,
    pub rso: f64,
    /// Sunlit leaf source responses: downward, upward and in the viewing direction.
    /// Multiply by leaf absorptivity and (B_sunlit - B_shaded).
    pub gamma_sdf: f64,
    pub gamma_sdb: f64,
    pub gamma_so: f64,
}
pub struct Geometry {
    ks: f64,
    ko: f64,
    bf: f64,
    sob: f64,
    sof: f64,
    dso: f64,
}
impl Geometry {
    pub fn new(p: &Parameters) -> Self {
        let distribution = match p.leaf_angle_distribution {
            LeafAngleDistribution::Campbell => campbell(p.mean_leaf_angle_deg, &CAMPBELL_BINS),
            LeafAngleDistribution::Verhoef => verhoef(p.lidf_a, p.lidf_b),
        };
        Self::from_distribution(p, distribution)
    }

    /// The 15 inclination classes used by INFORM.
    pub fn inform(p: &Parameters) -> Self {
        let bins = [
            10., 20., 30., 40., 50., 60., 68., 76., 78., 80., 82., 84., 86., 88., 90.,
        ];
        Self::from_distribution(p, campbell(p.mean_leaf_angle_deg, &bins))
    }

    fn from_distribution(p: &Parameters, distribution: Vec<(f64, f64)>) -> Self {
        let sun = p.solar_zenith_deg.to_radians();
        let view = p.view_zenith_deg.to_radians();
        let mut g = Self {
            ks: 0.,
            ko: 0.,
            bf: 0.,
            sob: 0.,
            sof: 0.,
            dso: (sun.tan().powi(2) + view.tan().powi(2)
                - 2. * sun.tan() * view.tan() * p.relative_azimuth_deg.to_radians().cos())
            .max(0.)
            .sqrt(),
        };
        for (angle, weight) in distribution {
            let (chi_s, chi_o, frho, ftau) = volscatt(
                sun,
                view,
                p.relative_azimuth_deg.to_radians(),
                angle.to_radians(),
            );
            g.ks += chi_s / sun.cos() * weight;
            g.ko += chi_o / view.cos() * weight;
            g.bf += angle.to_radians().cos().powi(2) * weight;
            g.sob += frho * PI / (sun.cos() * view.cos()) * weight;
            g.sof += ftau * PI / (sun.cos() * view.cos()) * weight;
        }
        g
    }
    pub fn coefficients(&self, rho: f64, tau: f64, p: &Parameters) -> Coefficients {
        self.solve(rho, tau, p, false)
    }

    /// Four-stream coefficients with the INFORM hotspot scaling.
    pub fn inform_coefficients(&self, rho: f64, tau: f64, p: &Parameters) -> Coefficients {
        self.solve(rho, tau, p, true)
    }

    fn solve(&self, rho: f64, tau: f64, p: &Parameters, inform: bool) -> Coefficients {
        let Self {
            ks,
            ko,
            bf,
            sob,
            sof,
            ..
        } = *self;
        let lai = p.leaf_area_index;
        if lai == 0. {
            return Coefficients {
                rdd: 0.,
                tdd: 1.,
                rsd: 0.,
                tsd: 0.,
                rdo: 0.,
                tdo: 0.,
                tss: 1.,
                too: 1.,
                tsstoo: 1.,
                rso: 0.,
                gamma_sdf: 0.,
                gamma_sdb: 0.,
                gamma_so: 0.,
            };
        }
        let sdb = 0.5 * (ks + bf);
        let sdf = 0.5 * (ks - bf);
        let dob = 0.5 * (ko + bf);
        let dof = 0.5 * (ko - bf);
        let ddb = 0.5 * (1. + bf);
        let ddf = 0.5 * (1. - bf);
        let sigb = ddb * rho + ddf * tau;
        let sigf = ddf * rho + ddb * tau;
        let att = 1. - sigf;
        let m = if (1. - rho - tau).abs() < 1e-12 {
            0.
        } else {
            ((att + sigb) * (att - sigb)).max(0.).sqrt()
        };
        let sb = sdb * rho + sdf * tau;
        let sf = sdf * rho + sdb * tau;
        let vb = dob * rho + dof * tau;
        let vf = dof * rho + dob * tau;
        let w = sob * rho + sof * tau;
        let (rdd, tdd, tsd, rsd, tdo, rdo, rsod) =
            transport(att, sigb, m, sf, sb, vf, vb, ks, ko, lai);
        let (_, _, gamma_sdf, gamma_sdb, _, _, gamma_sod) =
            transport(att, sigb, m, 1., 1., vf, vb, ks, ko, lai);
        let tss = (-ks * lai).exp();
        let too = (-ko * lai).exp();
        let alf = if inform && p.hotspot > 0. {
            self.dso / p.hotspot
        } else if inform {
            1e6
        } else if p.hotspot > 0. {
            (self.dso / p.hotspot) * 2. / (ks + ko)
        } else {
            1e6
        }
        .min(if inform { 1e6 } else { 200. });
        let (tsstoo, sumint) = if alf == 0. {
            (tss, -(-ks * lai).exp_m1() / (ks * lai))
        } else {
            let fhot = lai * (ko * ks).sqrt();
            let mut x1 = 0.;
            let mut y1 = 0.;
            let mut f1 = 1.;
            let mut sum = 0.;
            let fint = -(-alf).exp_m1() * 0.05;
            for i in 1..=20 {
                let x2 = if i < 20 {
                    -(-i as f64 * fint).ln_1p() / alf
                } else {
                    1.
                };
                let y2 = -(ko + ks) * lai * x2 - fhot * (-alf * x2).exp_m1() / alf;
                let f2 = y2.exp();
                // Continuous limit also covers numerically coincident integration endpoints.
                sum += if y2 == y1 {
                    f1 * (x2 - x1)
                } else {
                    (f2 - f1) * (x2 - x1) / (y2 - y1)
                };
                x1 = x2;
                y1 = y2;
                f1 = f2;
            }
            (f1, sum)
        };
        let rsos = w * lai * sumint;
        Coefficients {
            rdd,
            tdd,
            tsd,
            rsd,
            tdo,
            rdo,
            tss,
            too,
            tsstoo,
            rso: rsos + rsod,
            gamma_sdf,
            gamma_sdb,
            gamma_so: ko * lai * sumint + gamma_sod,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn transport(
    att: f64,
    sigb: f64,
    m: f64,
    sf: f64,
    sb: f64,
    vf: f64,
    vb: f64,
    ks: f64,
    ko: f64,
    lai: f64,
) -> (f64, f64, f64, f64, f64, f64, f64) {
    if m == 0. {
        conservative(att, sigb, sf, sb, vf, vb, ks, ko, lai)
    } else {
        let e1 = (-m * lai).exp();
        let e2 = e1 * e1;
        let rinf = sigb / (att + m);
        let rinf2 = rinf * rinf;
        let re = rinf * e1;
        let denom = 1. - rinf2 * e2;
        let j1ks = jfunc1(ks, m, lai);
        let j2ks = jfunc2(ks, m, lai);
        let j1ko = jfunc1(ko, m, lai);
        let j2ko = jfunc2(ko, m, lai);
        let ps = (sf + sb * rinf) * j1ks;
        let qs = (sf * rinf + sb) * j2ks;
        let pv = (vf + vb * rinf) * j1ko;
        let qv = (vf * rinf + vb) * j2ko;
        let rdd = rinf * (1. - e2) / denom;
        let tdd = (1. - rinf2) * e1 / denom;
        let tsd = (ps - re * qs) / denom;
        let rsd = (qs - re * ps) / denom;
        let tdo = (pv - re * qv) / denom;
        let rdo = (qv - re * pv) / denom;
        let tss = (-ks * lai).exp();
        let too = (-ko * lai).exp();
        let z = jfunc2(ks, ko, lai);
        let g1 = (z - j1ks * too) / (ko + m);
        let g2 = (z - j1ko * tss) / (ks + m);
        let tv1 = (vf * rinf + vb) * g1;
        let tv2 = (vf + vb * rinf) * g2;
        let rsod = (tv1 * (sf + sb * rinf) + tv2 * (sf * rinf + sb) - (rdo * qs + tdo * ps) * rinf)
            / (1. - rinf2);
        (rdd, tdd, tsd, rsd, tdo, rdo, rsod)
    }
}

fn jfunc1(k: f64, m: f64, l: f64) -> f64 {
    let del = (k - m) * l;
    if del.abs() > 1e-3 {
        ((-m * l).exp() - (-k * l).exp()) / (k - m)
    } else {
        0.5 * l * ((-k * l).exp() + (-m * l).exp()) * (1. - del * del / 12.)
    }
}
fn jfunc2(k: f64, m: f64, l: f64) -> f64 {
    (1. - (-(k + m) * l).exp()) / (k + m)
}
const CAMPBELL_BINS: [f64; 13] = [
    10., 20., 30., 40., 50., 60., 70., 80., 82., 84., 86., 88., 90.,
];

fn campbell(ala: f64, upper: &[f64]) -> Vec<(f64, f64)> {
    let ecc = (-1.6184e-5 * ala.powi(3) + 2.1145e-3 * ala.powi(2) - 1.2390e-1 * ala + 3.2491).exp();
    let mut bins = Vec::new();
    for (u, l) in upper
        .iter()
        .copied()
        .zip(std::iter::once(0.).chain(upper.iter().copied()))
    {
        let x1 = ecc / (1. + ecc * ecc * u.to_radians().tan().powi(2)).sqrt();
        let x2 = ecc / (1. + ecc * ecc * l.to_radians().tan().powi(2)).sqrt();
        let weight = if ecc == 1. {
            (u.to_radians().cos() - l.to_radians().cos()).abs()
        } else {
            let alpha = ecc / (1. - ecc * ecc).abs().sqrt();
            let a2 = alpha * alpha;
            let primitive = |x: f64| {
                if ecc > 1. {
                    let r = (a2 + x * x).sqrt();
                    x * r + a2 * (x + r).ln()
                } else {
                    let r = (a2 - x * x).max(0.).sqrt();
                    x * r + a2 * (x / alpha).clamp(-1., 1.).asin()
                }
            };
            (primitive(x1) - primitive(x2)).abs()
        };
        bins.push(((u + l) / 2., weight));
    }
    let total: f64 = bins.iter().map(|x| x.1).sum();
    for b in &mut bins {
        b.1 /= total;
    }
    bins
}
fn volscatt(s: f64, o: f64, psi: f64, l: f64) -> (f64, f64, f64, f64) {
    let cs = l.cos() * s.cos();
    let co = l.cos() * o.cos();
    let ss = l.sin() * s.sin();
    let so = l.sin() * o.sin();
    let cosbts = if ss.abs() > 1e-6 { -cs / ss } else { 5. };
    let cosbto = if so.abs() > 1e-6 { -co / so } else { 5. };
    let (bts, ds) = if cosbts.abs() < 1. {
        (cosbts.acos(), ss)
    } else {
        (PI, cs)
    };
    let (bto, doo) = if cosbto.abs() < 1. {
        (cosbto.acos(), so)
    } else {
        (PI, co)
    };
    let chi_s = 2. / PI * ((bts - PI * 0.5) * cs + bts.sin() * ss);
    let chi_o = 2. / PI * ((bto - PI * 0.5) * co + bto.sin() * so);
    let b1 = (bts - bto).abs();
    let b2 = PI - (bts + bto - PI).abs();
    let (bt1, bt2, bt3) = if psi <= b1 {
        (psi, b1, b2)
    } else if psi <= b2 {
        (b1, psi, b2)
    } else {
        (b1, b2, psi)
    };
    let t1 = 2. * cs * co + ss * so * psi.cos();
    let t2 = if bt2 > 0. {
        bt2.sin() * (2. * ds * doo + ss * so * bt1.cos() * bt3.cos())
    } else {
        0.
    };
    (
        chi_s,
        chi_o,
        ((PI - bt2) * t1 + t2).max(0.) / (2. * PI * PI),
        (-bt2 * t1 + t2).max(0.) / (2. * PI * PI),
    )
}

// Exact m=0 transfer-matrix limit: exp(A*x)=I+A*x, avoiding 0/0 in rinf.
#[allow(clippy::too_many_arguments)]
fn conservative(
    att: f64,
    sigb: f64,
    sf: f64,
    sb: f64,
    vf: f64,
    vb: f64,
    ks: f64,
    ko: f64,
    l: f64,
) -> (f64, f64, f64, f64, f64, f64, f64) {
    let j0 = |k: f64| -(-k * l).exp_m1() / k;
    let j1 = |k: f64| (j0(k) - l * (-k * l).exp()) / k;
    let source = |k: f64, f: f64, b: f64| {
        let i0 = j0(k);
        let i1 = (l - i0) / k;
        let reflect = (b * i0 + (sigb * f + att * b) * i1) / (1. + att * l);
        let transmit = sigb * l * reflect + f * i0 + (-att * f - sigb * b) * i1;
        (transmit, reflect)
    };
    let (tsd, rsd) = source(ks, sf, sb);
    let (tdo, rdo) = source(ko, vf, vb);
    let s0 = (j0(ko) - j0(ko + ks)) / ks;
    let s1 = j1(ko) / ks - s0 / ks;
    let d = sigb * rsd * j1(ko) + sf * s0 + (-att * sf - sigb * sb) * s1;
    let u = rsd * (j0(ko) + att * j1(ko)) - sb * s0 + (-sigb * sf - att * sb) * s1;
    (
        sigb * l / (1. + att * l),
        1. / (1. + att * l),
        tsd,
        rsd,
        tdo,
        rdo,
        vb * d + vf * u,
    )
}
fn verhoef(a: f64, b: f64) -> Vec<(f64, f64)> {
    let upper: [f64; 13] = [
        10., 20., 30., 40., 50., 60., 70., 80., 82., 84., 86., 88., 90.,
    ];
    let mut previous = 0.;
    let mut lower = 0.;
    let mut out = vec![];
    for t in upper {
        let cumulative = if t == 90. {
            1.
        } else if a >= 1. {
            1. - t.to_radians().cos()
        } else {
            let p = 2. * t.to_radians();
            let mut x = p;
            let mut y = 0.;
            for _ in 0..10000 {
                y = a * x.sin() + 0.5 * b * (2. * x).sin();
                let dx = 0.5 * (y - x + p);
                x += dx;
                if dx.abs() < 1e-8 {
                    break;
                }
            }
            (2. * y + p) / PI
        };
        out.push(((lower + t) / 2., cumulative - previous));
        previous = cumulative;
        lower = t;
    }
    out
}
