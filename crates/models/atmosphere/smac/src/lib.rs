//! SMAC analytic atmospheric correction. Based on Rahman & Dedieu (1994),
//! with the published Hagolle/CNES implementation as the numerical reference.
//! SPDX-License-Identifier: GPL-3.0-only
pub mod coefficients;
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    #[default]
    ToaToSurface,
    SurfaceToToa,
}
fn direction_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({"type":"string","enum":["toa_to_surface","surface_to_toa"]})
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    /// Exact sensor-band-aerosol coefficient identity; no sensor substitution.
    #[schemars(schema_with="coefficients::coefficient_schema", extend("x-unit"="1"))]
    pub coefficient_id: String,
    /// TOA -> surface atmospheric correction, or surface -> TOA simulation.
    #[schemars(schema_with="direction_schema",extend("x-unit"="1"))]
    pub direction: Direction,
    /// Input reflectance. Forward surface reflectance is restricted to [0,1].
    #[schemars(range(min=0.0,max=2.0),extend("x-unit"="1"))]
    pub reflectance: f64,
    #[schemars(range(min=0.0,max=70.0),extend("x-unit"="deg"))]
    pub solar_zenith_deg: f64,
    #[schemars(range(min=0.0,max=70.0),extend("x-unit"="deg"))]
    pub view_zenith_deg: f64,
    /// Absolute difference between solar and viewing azimuths, upstream phis-phiv.
    #[schemars(range(min=0.0,max=180.0),extend("x-unit"="deg"))]
    pub relative_azimuth_deg: f64,
    /// Local surface pressure (not sea-level pressure), hPa.
    #[schemars(range(min=300.0,max=1100.0),extend("x-unit"="hPa"))]
    pub pressure_hpa: f64,
    /// Aerosol optical depth at 550 nm, dimensionless.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="1"))]
    pub aot550: f64,
    /// Ozone vertical column, atm-cm; 0.3 corresponds to 300 DU.
    #[schemars(range(min=0.0,max=1.0),extend("x-unit"="atm cm"))]
    pub ozone_atm_cm: f64,
    /// Total column water vapour, g/cm^2; 1 g/cm^2 = 10 kg/m^2.
    #[schemars(range(min=0.0,max=10.0),extend("x-unit"="g cm^-2"))]
    pub water_vapour_g_cm2: f64,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            coefficient_id: coefficients::DEFAULT_COEFFICIENT.into(),
            direction: Direction::ToaToSurface,
            reflectance: 0.2,
            solar_zenith_deg: 30.,
            view_zenith_deg: 5.,
            relative_azimuth_deg: 60.,
            pressure_hpa: 1013.25,
            aot550: 0.1,
            ozone_atm_cm: 0.3,
            water_vapour_g_cm2: 2.,
        }
    }
}
fn domain(message: impl Into<String>) -> ModelError {
    ModelError::new(ErrorKind::OutOfDomain, message)
}
impl Input {
    pub fn validate(&self) -> Result<(), ModelError> {
        coefficients::get(&self.coefficient_id)?;
        for (name, v, lo, hi) in [
            ("reflectance", self.reflectance, 0., 2.),
            ("solar_zenith_deg", self.solar_zenith_deg, 0., 70.),
            ("view_zenith_deg", self.view_zenith_deg, 0., 70.),
            ("relative_azimuth_deg", self.relative_azimuth_deg, 0., 180.),
            ("pressure_hpa", self.pressure_hpa, 300., 1100.),
            ("aot550", self.aot550, 0., 1.),
            ("ozone_atm_cm", self.ozone_atm_cm, 0., 1.),
            ("water_vapour_g_cm2", self.water_vapour_g_cm2, 0., 10.),
        ] {
            if !v.is_finite() || !(lo..=hi).contains(&v) {
                return Err(domain(format!("{name} must be finite in [{lo}, {hi}]")));
            }
        }
        if self.direction == Direction::SurfaceToToa && self.reflectance > 1. {
            return Err(domain(
                "forward Lambertian surface reflectance must be <= 1",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
pub struct Point {
    pub reflectance: f64,
    /// Joint downward/upward gaseous transmittance, NOT a one-way transmission.
    pub gas_transmittance: f64,
    pub downward_scattering_transmittance: f64,
    pub upward_scattering_transmittance: f64,
    pub spherical_albedo: f64,
    /// Intrinsic atmospheric reflectance before the gas multiplier.
    pub atmospheric_reflectance: f64,
    pub rayleigh_reflectance: f64,
    pub aerosol_reflectance: f64,
    /// 1 when result is outside [0,1]; values are retained without clipping.
    pub outside_unit_interval: u32,
}
pub fn evaluate(p: &Input) -> Result<Point, ModelError> {
    p.validate()?;
    point(p)
}
fn point(p: &Input) -> Result<Point, ModelError> {
    let c = &coefficients::get(&p.coefficient_id)?.values;
    let [
        ah2o,
        nh2o,
        ao3,
        no3,
        ao2,
        no2,
        po2,
        aco2,
        nco2,
        pco2,
        ach4,
        nch4,
        pch4,
        ano2,
        nno2,
        pno2,
        aco,
        nco,
        pco,
        a0s,
        a1s,
        a2s,
        a3s,
        a0t,
        a1t,
        a2t,
        a3t,
        taur,
        _unused,
        a0taup,
        a1taup,
        wo,
        gc,
        a0p,
        a1p,
        a2p,
        a3p,
        a4p,
        rest1,
        rest2,
        rest3,
        rest4,
        resr1,
        resr2,
        resr3,
        resa1,
        resa2,
        resa3,
        resa4,
    ]: [f64; 49] = c
        .as_slice()
        .try_into()
        .expect("verified 49 coefficient layout");
    let us = p.solar_zenith_deg.to_radians().cos();
    let uv = p.view_zenith_deg.to_radians().cos();
    let pressure = p.pressure_hpa / 1013.25;
    let m = 1. / us + 1. / uv;
    let tau = a0taup + a1taup * p.aot550;
    let gas = |a: f64, n: f64, u: f64| (a * (u * m).powf(n)).exp();
    let tg = gas(ah2o, nh2o, p.water_vapour_g_cm2)
        * gas(ao3, no3, p.ozone_atm_cm)
        * gas(ao2, no2, pressure.powf(po2))
        * gas(aco2, nco2, pressure.powf(pco2))
        * gas(ach4, nch4, pressure.powf(pch4))
        * gas(aco, nco, pressure.powf(pco))
        * gas(ano2, nno2, pressure.powf(pno2));
    let td = a0t + a1t * p.aot550 / us + (a2t * pressure + a3t) / (1. + us);
    let tu = a0t + a1t * p.aot550 / uv + (a2t * pressure + a3t) / (1. + uv);
    let s = a0s * pressure + a3s + a1s * p.aot550 + a2s * p.aot550.powi(2);
    // Preserve the original SMAC azimuth/scattering convention, independently of other models.
    let cosine = (-(us * uv
        + (1. - us * us).sqrt()
            * (1. - uv * uv).sqrt()
            * p.relative_azimuth_deg.to_radians().cos()))
    .clamp(-1., 1.);
    let angle = cosine.acos().to_degrees();
    let phase = 0.7190443 * (1. + cosine * cosine) + 0.0412742;
    let ray = taur * phase / (4. * us * uv) * pressure;
    let residual_ray =
        resr1 + resr2 * taur * phase / (us * uv) + resr3 * (taur * phase / (us * uv)).powi(2);
    let aerosol_phase =
        a0p + a1p * angle + a2p * angle * angle + a3p * angle.powi(3) + a4p * angle.powi(4);
    let ak2 = (1. - wo) * (3. - wo * 3. * gc);
    let ak = ak2.sqrt();
    let e = -3. * us * us * wo / (4. * (1. - ak2 * us * us));
    let f = -(1. - wo) * 3. * gc * us * us * wo / (4. * (1. - ak2 * us * us));
    let dp = e / (3. * us) + us * f;
    let d = e + f;
    let b = 2. * ak / (3. - wo * 3. * gc);
    let delta = (ak * tau).exp() * (1. + b) * (1. + b) - (-ak * tau).exp() * (1. - b) * (1. - b);
    let ww = wo / 4.;
    let ss = us / (1. - ak2 * us * us);
    let q1 = 2. + 3. * us + (1. - wo) * 3. * gc * us * (1. + 2. * us);
    let q2 = 2. - 3. * us - (1. - wo) * 3. * gc * us * (1. - 2. * us);
    let q3 = q2 * (-tau / us).exp();
    let c1 = ww * ss / delta * (q1 * (ak * tau).exp() * (1. + b) + q3 * (1. - b));
    let c2 = -(ww * ss / delta) * (q1 * (-ak * tau).exp() * (1. - b) + q3 * (1. + b));
    let cp1 = c1 * ak / (3. - wo * 3. * gc);
    let cp2 = -c2 * ak / (3. - wo * 3. * gc);
    let z = d - wo * 3. * gc * uv * dp + wo * aerosol_phase / 4.;
    let x = c1 - wo * 3. * gc * uv * cp1;
    let y = c2 - wo * 3. * gc * uv * cp2;
    let aa1 = uv / (1. + ak * uv);
    let aa2 = uv / (1. - ak * uv);
    let aa3 = us * uv / (us + uv);
    let aerosol = (x * aa1 * (1. - (-tau / aa1).exp())
        + y * aa2 * (1. - (-tau / aa2).exp())
        + z * aa3 * (1. - (-tau / aa3).exp()))
        / (us * uv);
    let residual_aerosol = resa1
        + resa2 * (tau * m * cosine)
        + resa3 * (tau * m * cosine).powi(2)
        + resa4 * (tau * m * cosine).powi(3);
    let total_tau = tau + taur * pressure;
    let residual_total = rest1
        + rest2 * (total_tau * m * cosine)
        + rest3 * (total_tau * m * cosine).powi(2)
        + rest4 * (total_tau * m * cosine).powi(3);
    let atmosphere = ray - residual_ray + aerosol - residual_aerosol + residual_total;
    let (numerator, denominator, offset) = match p.direction {
        Direction::SurfaceToToa => (
            p.reflectance * tg * td * tu,
            1. - p.reflectance * s,
            atmosphere * tg,
        ),
        Direction::ToaToSurface => {
            let r = p.reflectance - atmosphere * tg;
            (r, tg * td * tu + r * s, 0.)
        }
    };
    if !denominator.is_finite() || denominator.abs() < 1e-15 {
        return Err(domain(
            "SMAC reflectance conversion is singular/ill-conditioned for these coefficients and atmospheric inputs",
        ));
    }
    let reflectance = numerator / denominator + offset;
    let out = Point {
        reflectance,
        gas_transmittance: tg,
        downward_scattering_transmittance: td,
        upward_scattering_transmittance: tu,
        spherical_albedo: s,
        atmospheric_reflectance: atmosphere,
        rayleigh_reflectance: ray,
        aerosol_reflectance: aerosol,
        outside_unit_interval: u32::from(!(0.0..=1.0).contains(&reflectance)),
    };
    if [reflectance, tg, td, tu, s, atmosphere, ray, aerosol]
        .iter()
        .any(|v| !v.is_finite())
    {
        return Err(domain(
            "SMAC produced non-finite intermediate/output values",
        ));
    }
    Ok(out)
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Output {
    #[serde(flatten)]
    pub point: Point,
    /// AOT sensitivity only; all other inputs and the coefficient set are fixed.
    pub sweep_aot550: Vec<f64>,
    pub sweep_reflectance: Vec<f64>,
}
pub struct Model {
    metadata: ModelMetadata,
}
impl Model {
    pub fn new() -> Result<Self, ModelError> {
        coefficients::library()?;
        Ok(Self {
            metadata: parse_model_manifest(include_str!("../model.toml"))?,
        })
    }
    pub fn simulate(&self, p: &Input) -> Result<Output, ModelError> {
        p.validate()?;
        let sweep_aot550: Vec<_> = (0..=20).map(|i| f64::from(i) / 20.).collect();
        let sweep_reflectance = sweep_aot550
            .iter()
            .map(|&aot550| {
                point(&Input {
                    aot550,
                    ..p.clone()
                })
                .map(|v| v.reflectance)
            })
            .collect::<Result<_, _>>()?;
        Ok(Output {
            point: point(p)?,
            sweep_aot550,
            sweep_reflectance,
        })
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
        self.simulate(p)
    }
    fn validate_output(&self, out: &Output) -> Result<(), ModelError> {
        if out.sweep_aot550.len() != 21
            || out.sweep_reflectance.len() != 21
            || !out.point.reflectance.is_finite()
            || out.sweep_reflectance.iter().any(|v| !v.is_finite())
        {
            return Err(ModelError::new(ErrorKind::Internal, "invalid SMAC output"));
        }
        Ok(())
    }
}
