//! Single-scale monostatic I2EM, HH/VV. Original numerical conventions retained.
use num_complex::Complex64 as C;
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CorrelationFunction {
    #[default]
    Exponential,
    Gaussian,
    XPower,
    XExponential,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    /// Radar frequency. Wave number follows the reference c = 0.30 m/ns.
    #[schemars(range(min=0.1,max=100.0),extend("x-unit"="GHz"))]
    pub frequency_ghz: f64,
    /// Real part of the relative permittivity, epsilon = real - i loss.
    #[schemars(range(min=1.01,max=100.0),extend("x-unit"="1"))]
    pub permittivity_real: f64,
    /// Nonnegative dielectric loss; the imaginary part is its negative.
    #[schemars(range(min=0.0,max=100.0),extend("x-unit"="1"))]
    pub permittivity_loss: f64,
    /// RMS surface height; k*s must be strictly below one.
    #[schemars(range(min=0.000001,max=1.0),extend("x-unit"="m"))]
    pub rms_height_m: f64,
    /// Surface correlation length; s/L must not exceed 0.25.
    #[schemars(range(min=0.00001,max=10.0),extend("x-unit"="m"))]
    pub correlation_length_m: f64,
    /// Incidence angle from surface normal; exact normal/grazing limits excluded.
    #[schemars(range(min=1.0,max=80.0),extend("x-unit"="deg"))]
    pub observation_zenith_deg: f64,
    #[schemars(with = "String", extend("enum" = ["exponential", "gaussian", "x_power", "x_exponential"]))]
    pub correlation_function: CorrelationFunction,
    /// Shape exponent of x-power / x-exponential correlations (unused otherwise).
    #[schemars(range(min=1.000001,max=10.0),extend("x-unit"="1"))]
    pub correlation_exponent: f64,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            frequency_ghz: 5.3,
            permittivity_real: 10.,
            permittivity_loss: 1.,
            rms_height_m: 0.005,
            correlation_length_m: 0.06,
            observation_zenith_deg: 30.,
            correlation_function: CorrelationFunction::Exponential,
            correlation_exponent: 1.5,
        }
    }
}
fn domain(message: impl Into<String>) -> ModelError {
    ModelError::new(ErrorKind::OutOfDomain, message)
}
impl Input {
    pub fn validate(&self) -> Result<(), ModelError> {
        for (name, v, lo, hi) in [
            (
                "correlation_exponent",
                self.correlation_exponent,
                1.000001,
                10.,
            ),
            ("frequency_ghz", self.frequency_ghz, 0.1, 100.),
            ("permittivity_real", self.permittivity_real, 1.01, 100.),
            ("permittivity_loss", self.permittivity_loss, 0., 100.),
            ("rms_height_m", self.rms_height_m, 0.000001, 1.),
            (
                "correlation_length_m",
                self.correlation_length_m,
                0.00001,
                10.,
            ),
            (
                "observation_zenith_deg",
                self.observation_zenith_deg,
                1.,
                80.,
            ),
        ] {
            if !v.is_finite() || !(lo..=hi).contains(&v) {
                return Err(domain(format!("{name} must be finite in [{lo}, {hi}]")));
            }
        }
        if matches!(self.correlation_function, CorrelationFunction::XExponential)
            && self.correlation_exponent > 2.
        {
            return Err(domain(
                "x-exponential covariance requires 1 < exponent <= 2 for a nonnegative spectrum",
            ));
        }
        if self.ks() >= 1. {
            return Err(domain(
                "I2EM requires k*rms_height_m < 1 (k = 2*pi*frequency_ghz/0.30)",
            ));
        }
        if self.rms_height_m / self.correlation_length_m > 0.25 {
            return Err(domain(
                "I2EM requires rms_height_m/correlation_length_m <= 0.25",
            ));
        }
        Ok(())
    }
    fn ks(&self) -> f64 {
        2. * PI * self.frequency_ghz / 0.30 * self.rms_height_m
    }
}
#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub sigma0_vv: f64,
    pub sigma0_hh: f64,
    pub sigma0_vv_db: f64,
    pub sigma0_hh_db: f64,
    pub ks: f64,
    pub height_length_ratio: f64,
}

pub fn evaluate(p: &Input) -> Result<Output, ModelError> {
    p.validate()?;
    // Reference length conversion and wave number; do not substitute exact c.
    let sig = p.rms_height_m * 100.;
    let l = p.correlation_length_m * 100.;
    let k = 2. * PI * p.frequency_ghz / 30.;
    let theta = p.observation_zenith_deg.to_radians();
    let (s, cs) = (theta + 0.01).sin_cos();
    let (ss, css) = theta.sin_cos();
    let er = C::new(p.permittivity_real, -p.permittivity_loss);
    let rt = (er - s * s).sqrt();
    let rvi = (er * cs - rt) / (er * cs + rt);
    let rhi = (cs - rt) / (cs + rt);
    let rv0 = (er.sqrt() - 1.) / (er.sqrt() + 1.);
    let rh0 = -rv0;
    let ks = k * sig;
    let kz = k * cs;
    let ksz = k * css;
    let ql = k * (ss + s) * l;
    let mut ts = 1;
    let mut term = ks * ks * (cs + css).powi(2);
    loop {
        ts += 1;
        term *= ks * ks * (cs + css).powi(2) / ts as f64;
        if term <= 1e-8 {
            break;
        }
    }
    let wn: Vec<f64> = (1..=ts)
        .map(|n| {
            let n = n as f64;
            Ok(match p.correlation_function {
                CorrelationFunction::Exponential => {
                    l * l / n.powi(2) * (1. + (ql / n).powi(2)).powf(-1.5)
                }
                CorrelationFunction::Gaussian => l * l / (2. * n) * (-ql * ql / (4. * n)).exp(),
                CorrelationFunction::XPower => {
                    l * l * x_power_spectrum(ql, p.correlation_exponent * n)?
                }
                CorrelationFunction::XExponential => {
                    let scale = n.powf(1. / p.correlation_exponent);
                    l * l / (scale * scale)
                        * x_exponential_spectrum(ql / scale, p.correlation_exponent)?
                }
            })
        })
        .collect::<Result<_, ModelError>>()?;
    let ft = 8. * rv0 * rv0 * ss * (cs + rt) / (cs * rt);
    let mut a1 = 0.;
    let mut b1 = 0.;
    let mut a0 = 1.;
    for (i, w) in wn.iter().enumerate() {
        let n = (i + 1) as i32;
        a0 *= (ks * cs).powi(2) / n as f64;
        a1 += a0 * w;
        b1 += a0
            * (ft / 2. + 2_f64.powi(n + 1) * rv0 / cs * (-(ks * cs).powi(2)).exp()).norm_sqr()
            * w;
    }
    if b1 <= 0. || !b1.is_finite() {
        return Err(domain(
            "I2EM transition series underflow; correlation length is too large for this configuration",
        ));
    }
    let st = 0.25 * ft.norm_sqr() * a1 / b1;
    let st0 = 1. / (1. + 8. * rv0 / (cs * ft)).norm_sqr();
    let tf = 1. - st / st0;
    let rvt = rvi + (rv0 - rvi) * tf;
    let rht = rhi + (rh0 - rhi) * tf;
    let angular = 2. * (s * ss + 1. + cs * css) / (cs + css);
    let fvv = rvt * angular;
    let fhh = -rht * angular;
    let [up_i, up_s, dn_i, dn_s] =
        [(1., true), (1., false), (-1., true), (-1., false)].map(|(ud, incident)| {
            complementary(ud, incident, rvi, rhi, er, k, kz, ksz, s, cs, ss, css)
        });
    let qi = k * cs;
    let qs = k * css;
    let mut vv = 0.;
    let mut hh = 0.;
    let mut weight = 1.;
    for (i, w) in wn.iter().enumerate() {
        let n = (i + 1) as i32;
        let b = (kz + ksz).powi(n) * (-sig * sig * kz * ksz).exp();
        let ci = (ksz - qi).powi(n - 1) * (-sig * sig * (qi * qi - qi * (ksz - kz))).exp();
        let di = (ksz + qi).powi(n - 1) * (-sig * sig * (qi * qi + qi * (ksz - kz))).exp();
        let cs_ = (kz + qs).powi(n - 1) * (-sig * sig * (qs * qs - qs * (ksz - kz))).exp();
        let ds = (kz - qs).powi(n - 1) * (-sig * sig * (qs * qs + qs * (ksz - kz))).exp();
        let ivv = b * fvv + 0.25 * (up_i.0 * ci + dn_i.0 * di + up_s.0 * cs_ + dn_s.0 * ds);
        let ihh = b * fhh + 0.25 * (up_i.1 * ci + dn_i.1 * di + up_s.1 * cs_ + dn_s.1 * ds);
        weight *= sig * sig / n as f64;
        vv += ivv.norm_sqr() * weight * w;
        hh += ihh.norm_sqr() * weight * w;
    }
    let rss = match p.correlation_function {
        CorrelationFunction::Exponential => sig / l,
        CorrelationFunction::Gaussian => 2_f64.sqrt() * sig / l,
        CorrelationFunction::XPower => (2. * p.correlation_exponent).sqrt() * sig / l,
        CorrelationFunction::XExponential => sig / l,
    };
    let ct = 1. / theta.tan() / 2_f64.sqrt() / rss;
    let shad = 0.5 * ((-ct * ct).exp() / PI.sqrt() / ct - libm::erfc(ct));
    let scale = k * k / 2. * (-sig * sig * (kz * kz + ksz * ksz)).exp() / (1. + 2. * shad);
    vv *= scale;
    hh *= scale;
    if !vv.is_finite() || !hh.is_finite() || vv <= 0. || hh <= 0. {
        return Err(domain(
            "I2EM backscatter is outside the representable positive f64 range",
        ));
    }
    Ok(Output {
        sigma0_vv: vv,
        sigma0_hh: hh,
        sigma0_vv_db: 10. * vv.log10(),
        sigma0_hh_db: 10. * hh.log10(),
        ks: p.ks(),
        height_length_ratio: sig / l,
    })
}

#[allow(clippy::too_many_arguments)]
fn complementary(
    ud: f64,
    incident: bool,
    rv: C,
    rh: C,
    er: C,
    k: f64,
    kz: f64,
    ksz: f64,
    s: f64,
    cs: f64,
    ss: f64,
    css: f64,
) -> (C, C) {
    // Monostatic azimuth is pi: cf=1, cfs=-1, sfs=0.
    let (c11, c21, c31, c41, c51, c12, c22, c32, c42, c52);
    if incident {
        let q = ud * kz;
        let g = q;
        let gt = ud * k * (er - s * s).sqrt();
        c11 = C::from(-k * (ksz - q));
        c21 = C::from(-cs * (k * k * s * (-ss - s) + g * (k * css - q)));
        c31 = C::from(k * s * (-s * (k * css - q) + g * (-ss - s)));
        c41 = C::from(k * cs * (-css * (k * css - q) + k * ss * (-ss - s)));
        c51 = C::from(g * (-css * (q - k * css) - k * ss * (-ss - s)));
        c12 = c11;
        c22 = -cs * (k * k * s * (-ss - s) + gt * (k * css - q));
        c32 = k * s * (-s * (k * css - q) + gt * (-ss - s));
        c42 = c41;
        c52 = gt * (-css * (q - k * css) - k * ss * (-ss - s));
    } else {
        let q = ud * ksz;
        let g = q;
        let gt = ud * k * (er - ss * ss).sqrt();
        c11 = C::from(-k * (kz + q));
        c21 = C::from(-g * (cs * (k * cs + q) - k * s * (-ss - s)));
        c31 = C::from(k * ss * (k * cs * (-ss - s) + s * (kz + q)));
        c41 = C::from(-k * css * (cs * (kz + q) - k * s * (-ss - s)));
        c51 = C::from(-css * (k * k * ss * (-ss - s) - g * (kz + q)));
        c12 = c11;
        c22 = -gt * (cs * (kz + q) - k * s * (-ss - s));
        c32 = c31;
        c42 = c41;
        c52 = -css * (k * k * ss * (-ss - s) - gt * (kz + q));
    }
    let q = kz;
    let qt = k * (er - s * s).sqrt();
    let vv = (1. + rv) * (-(1. - rv) * c11 / q + (1. + rv) * c12 / qt)
        + (1. - rv) * ((1. - rv) * c21 / q - (1. + rv) * c22 / qt)
        + (1. + rv) * ((1. - rv) * c31 / q - (1. + rv) * c32 / er / qt)
        + (1. - rv) * ((1. + rv) * c41 / q - er * (1. - rv) * c42 / qt)
        + (1. + rv) * ((1. + rv) * c51 / q - (1. - rv) * c52 / qt);
    let hh = (1. + rh) * ((1. - rh) * c11 / q - er * (1. + rh) * c12 / qt)
        - (1. - rh) * ((1. - rh) * c21 / q - (1. + rh) * c22 / qt)
        - (1. + rh) * ((1. - rh) * c31 / q - (1. + rh) * c32 / qt)
        - (1. - rh) * ((1. + rh) * c41 / q - (1. - rh) * c42 / qt)
        - (1. + rh) * ((1. + rh) * c51 / q - (1. - rh) * c52 / qt);
    (vv, hh)
}

pub struct Model {
    metadata: ModelMetadata,
}
impl Model {
    pub fn new() -> Result<Self, ModelError> {
        Ok(Self {
            metadata: parse_model_manifest(include_str!("../model.toml"))?,
        })
    }
    pub fn simulate(&self, p: &Input) -> Result<Output, ModelError> {
        evaluate(p)
    }
}
impl PhysicsModel for Model {
    type Input = Input;
    type Output = Output;
    fn metadata(&self) -> ModelMetadata {
        self.metadata.clone()
    }
    fn validate_input(&self, p: &Input) -> Result<(), ModelError> {
        p.validate()
    }
    fn run(&self, p: &Input) -> Result<Output, ModelError> {
        evaluate(p)
    }
    fn validate_output(&self, o: &Output) -> Result<(), ModelError> {
        if [
            o.sigma0_vv,
            o.sigma0_hh,
            o.sigma0_vv_db,
            o.sigma0_hh_db,
            o.ks,
            o.height_length_ratio,
        ]
        .iter()
        .any(|v| !v.is_finite())
            || o.sigma0_vv <= 0.
            || o.sigma0_hh <= 0.
        {
            return Err(ModelError::new(ErrorKind::Internal, "invalid I2EM output"));
        }
        Ok(())
    }
}

// Positive Laplace representation of q^(p-1) K_(p-1)(q)/(2^(p-1) Gamma(p)).
// Log-coordinate quadrature avoids overflow of separate Bessel-K/Gamma factors.
fn x_power_spectrum(q: f64, p: f64) -> Result<f64, ModelError> {
    if q == 0. {
        return Ok(1. / (2. * (p - 1.)));
    }
    let t = ((p - 1.) + ((p - 1.).powi(2) + q * q).sqrt()) / 2.;
    let center = t.ln();
    let log = |u: f64| {
        (p - 1.) * u - u.exp() - q * q / (4. * u.exp()) - std::f64::consts::LN_2 - libm::lgamma(p)
    };
    let peak = log(center);
    if peak < -740. {
        return Ok(0.);
    }
    let mut lo = center;
    let mut hi = center;
    while log(lo) > peak - 45. {
        lo -= 0.5;
    }
    while log(hi) > peak - 45. {
        hi += 0.5;
    }
    integrate(&|u| (log(u) - peak).exp(), lo, hi, 1e-12).map(|v| v * peak.exp())
}
fn x_exponential_spectrum(q: f64, exponent: f64) -> Result<f64, ModelError> {
    if exponent == 2. {
        return Ok(0.5 * (-q * q / 4.).exp());
    }
    // Original source's upper integration bound is nine, retained for comparison.
    let segments = ((9. * q / std::f64::consts::PI).ceil() as usize).max(8);
    if segments > 100000 {
        return Err(domain(
            "x-exponential Hankel quadrature exceeds the supported oscillation count",
        ));
    }
    let f = |z: f64| (-z.powf(exponent)).exp() * libm::j0(z * q) * z;
    let mut sum = 0.;
    for i in 0..segments {
        sum += integrate(
            &f,
            9. * i as f64 / segments as f64,
            9. * (i + 1) as f64 / segments as f64,
            1e-12 / segments as f64,
        )?;
    }
    if sum < 0. {
        return Err(domain(
            "x-exponential reference quadrature produced a negative roughness spectrum",
        ));
    }
    Ok(sum)
}
fn integrate<F: Fn(f64) -> f64>(f: &F, a: f64, b: f64, tol: f64) -> Result<f64, ModelError> {
    #[allow(clippy::too_many_arguments)]
    fn step<F: Fn(f64) -> f64>(
        f: &F,
        a: f64,
        b: f64,
        fa: f64,
        fm: f64,
        fb: f64,
        whole: f64,
        tol: f64,
        depth: u32,
    ) -> Result<f64, ModelError> {
        let m = (a + b) / 2.;
        let l = f((a + m) / 2.);
        let r = f((m + b) / 2.);
        let left = (m - a) * (fa + 4. * l + fm) / 6.;
        let right = (b - m) * (fm + 4. * r + fb) / 6.;
        let delta = left + right - whole;
        if delta.abs() <= 15. * tol {
            return Ok(left + right + delta / 15.);
        }
        if depth == 0 {
            return Err(ModelError::new(
                ErrorKind::NonConvergence,
                "roughness spectrum quadrature did not converge",
            ));
        }
        Ok(step(f, a, m, fa, l, fm, left, tol / 2., depth - 1)?
            + step(f, m, b, fm, r, fb, right, tol / 2., depth - 1)?)
    }
    let fa = f(a);
    let fm = f((a + b) / 2.);
    let fb = f(b);
    step(
        f,
        a,
        b,
        fa,
        fm,
        fb,
        (b - a) * (fa + 4. * fm + fb) / 6.,
        tol,
        24,
    )
}
