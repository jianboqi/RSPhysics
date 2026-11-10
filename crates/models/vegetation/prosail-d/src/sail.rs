//! Forward equations adapted from jbferet/prosail commit 46549958 (MIT).
//! 4SAIL, Campbell's original 13 irregular angle bins, and 20-step hotspot integral.
use crate::Input;
use std::f64::consts::PI;

pub struct Geometry {
    ks: f64,
    ko: f64,
    bf: f64,
    sob: f64,
    sof: f64,
    dso: f64,
}
impl Geometry {
    pub fn new(p: &Input) -> Self {
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
        for (angle, weight) in campbell(p.mean_leaf_angle_deg) {
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
    pub fn solve(&self, rho: f64, tau: f64, soil: f64, p: &Input) -> (f64, f64) {
        let Self {
            ks,
            ko,
            bf,
            sob,
            sof,
            ..
        } = *self;
        let lai = p.leaf_area_index;
        let sdb = 0.5 * (ks + bf);
        let sdf = 0.5 * (ks - bf);
        let dob = 0.5 * (ko + bf);
        let dof = 0.5 * (ko - bf);
        let ddb = 0.5 * (1. + bf);
        let ddf = 0.5 * (1. - bf);
        let sigb = ddb * rho + ddf * tau;
        let sigf = ddf * rho + ddb * tau;
        let att = 1. - sigf;
        let m = ((att + sigb) * (att - sigb)).max(0.).sqrt();
        let sb = sdb * rho + sdf * tau;
        let sf = sdf * rho + sdb * tau;
        let vb = dob * rho + dof * tau;
        let vf = dof * rho + dob * tau;
        let w = sob * rho + sof * tau;
        let e1 = (-m * lai).exp();
        let e2 = e1 * e1;
        let rinf = (att - m) / sigb;
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
        let alf = if p.hotspot > 0. {
            (self.dso / p.hotspot) * 2. / (ks + ko)
        } else {
            1e6
        }
        .min(200.);
        let (tsstoo, sumint) = if alf == 0. {
            (tss, (1. - tss) / (ks * lai))
        } else {
            let fhot = lai * (ko * ks).sqrt();
            let mut x1 = 0.;
            let mut y1 = 0.;
            let mut f1 = 1.;
            let mut sum = 0.;
            let fint = (1. - (-alf).exp()) * 0.05;
            for i in 1..=20 {
                let x2 = if i < 20 {
                    -(1. - i as f64 * fint).ln() / alf
                } else {
                    1.
                };
                let y2 = -(ko + ks) * lai * x2 + fhot * (1. - (-alf * x2).exp()) / alf;
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
        let dn = 1. - soil * rdd;
        let hemi = rsd + (tsd + tss) * soil * tdd / dn;
        let rsodt = rsod + ((tss + tsd) * tdo + (tsd + tss * soil * rdd) * too) * soil / dn;
        (rsos + tsstoo * soil + rsodt, hemi)
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
fn campbell(ala: f64) -> Vec<(f64, f64)> {
    let upper: [f64; 13] = [
        10., 20., 30., 40., 50., 60., 70., 80., 82., 84., 86., 88., 90.,
    ];
    let lower: [f64; 13] = [
        0., 10., 20., 30., 40., 50., 60., 70., 80., 82., 84., 86., 88.,
    ];
    let ecc = (-1.6184e-5 * ala.powi(3) + 2.1145e-3 * ala.powi(2) - 1.2390e-1 * ala + 3.2491).exp();
    let mut bins = Vec::new();
    for (u, l) in upper.into_iter().zip(lower) {
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
