//! Four-component thermal 4SAIL, Verhoef, Jia, Xiao & Su (2007), Eqs. (1)–(5).
//! Temperatures are prescribed. Leaf transmission is zero; soil is Lambertian.
//! Solar illumination determines hot/shaded populations, not their temperatures.
use rsphysics_model_api::{
    ErrorKind, ModelError, ModelMetadata, PhysicsModel, parse_model_manifest,
};
use rsphysics_numerics::{
    sail::{Geometry, LeafAngleDistribution as Lidf, Parameters},
    thermal::{WavelengthDensity, inverse_planck, planck_radiance},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LeafAngleDistribution {
    #[default]
    Campbell,
    Verhoef,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    #[schemars(range(min=3000.,max=50000.),extend("x-unit"="nm"))]
    pub wavelength_nm: f64,
    #[schemars(range(min=0.,max=10.),extend("x-unit"="1"))]
    pub leaf_area_index: f64,
    #[schemars(range(min=5.,max=85.),extend("x-unit"="deg"))]
    pub mean_leaf_angle_deg: f64,
    #[schemars(with="String",extend("enum"=["campbell","verhoef"]))]
    pub leaf_angle_distribution: LeafAngleDistribution,
    #[schemars(range(min=-1.,max=1.),extend("x-unit"="1"))]
    pub lidf_a: f64,
    #[schemars(range(min=-1.,max=1.),extend("x-unit"="1"))]
    pub lidf_b: f64,
    #[schemars(range(min=0.,max=1.),extend("x-unit"="1"))]
    pub hotspot: f64,
    #[schemars(range(min=0.,max=85.),extend("x-unit"="deg"))]
    pub solar_zenith_deg: f64,
    #[schemars(range(min=0.,max=85.),extend("x-unit"="deg"))]
    pub view_zenith_deg: f64,
    #[schemars(range(min=0.,max=180.),extend("x-unit"="deg"))]
    pub relative_azimuth_deg: f64,
    #[schemars(range(min=0.,max=1.),extend("x-unit"="1"))]
    pub leaf_emissivity: f64,
    #[schemars(range(min=0.,max=1.),extend("x-unit"="1"))]
    pub soil_emissivity: f64,
    #[schemars(range(min=150.,max=500.),extend("x-unit"="K"))]
    pub sunlit_leaf_temperature_k: f64,
    #[schemars(range(min=150.,max=500.),extend("x-unit"="K"))]
    pub shaded_leaf_temperature_k: f64,
    #[schemars(range(min=150.,max=500.),extend("x-unit"="K"))]
    pub sunlit_soil_temperature_k: f64,
    #[schemars(range(min=150.,max=500.),extend("x-unit"="K"))]
    pub shaded_soil_temperature_k: f64,
    /// Isotropic downwelling radiance B_lambda(Tsky), not air temperature.
    #[schemars(range(min=0.,max=500.),extend("x-unit"="K"))]
    pub sky_brightness_temperature_k: f64,
    /// Direct spectral irradiance on a horizontal surface, excluding the sky field.
    #[schemars(range(min=0.,max=1000.),extend("x-unit"="W m^-2 um^-1"))]
    pub direct_irradiance_w_m2_um: f64,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            wavelength_nm: 9500.,
            leaf_area_index: 3.,
            mean_leaf_angle_deg: 60.,
            leaf_angle_distribution: LeafAngleDistribution::Campbell,
            lidf_a: -0.35,
            lidf_b: -0.15,
            hotspot: 0.1,
            solar_zenith_deg: 30.,
            view_zenith_deg: 0.,
            relative_azimuth_deg: 60.,
            leaf_emissivity: 0.98,
            soil_emissivity: 0.95,
            sunlit_leaf_temperature_k: 310.,
            shaded_leaf_temperature_k: 300.,
            sunlit_soil_temperature_k: 320.,
            shaded_soil_temperature_k: 305.,
            sky_brightness_temperature_k: 270.,
            direct_irradiance_w_m2_um: 0.,
        }
    }
}
fn domain(s: impl Into<String>) -> ModelError {
    ModelError::new(ErrorKind::OutOfDomain, s)
}
impl Input {
    pub fn validate(&self) -> Result<(), ModelError> {
        for (name, v, lo, hi) in [
            ("wavelength_nm", self.wavelength_nm, 3000., 50000.),
            ("leaf_area_index", self.leaf_area_index, 0., 10.),
            ("mean_leaf_angle_deg", self.mean_leaf_angle_deg, 5., 85.),
            ("lidf_a", self.lidf_a, -1., 1.),
            ("lidf_b", self.lidf_b, -1., 1.),
            ("hotspot", self.hotspot, 0., 1.),
            ("solar_zenith_deg", self.solar_zenith_deg, 0., 85.),
            ("view_zenith_deg", self.view_zenith_deg, 0., 85.),
            ("relative_azimuth_deg", self.relative_azimuth_deg, 0., 180.),
            ("leaf_emissivity", self.leaf_emissivity, 0., 1.),
            ("soil_emissivity", self.soil_emissivity, 0., 1.),
            (
                "sunlit_leaf_temperature_k",
                self.sunlit_leaf_temperature_k,
                150.,
                500.,
            ),
            (
                "shaded_leaf_temperature_k",
                self.shaded_leaf_temperature_k,
                150.,
                500.,
            ),
            (
                "sunlit_soil_temperature_k",
                self.sunlit_soil_temperature_k,
                150.,
                500.,
            ),
            (
                "shaded_soil_temperature_k",
                self.shaded_soil_temperature_k,
                150.,
                500.,
            ),
            (
                "sky_brightness_temperature_k",
                self.sky_brightness_temperature_k,
                0.,
                500.,
            ),
            (
                "direct_irradiance_w_m2_um",
                self.direct_irradiance_w_m2_um,
                0.,
                1000.,
            ),
        ] {
            if !v.is_finite() || !(lo..=hi).contains(&v) {
                return Err(domain(format!("{name} must be finite in [{lo},{hi}]")));
            }
        }
        if self.leaf_angle_distribution == LeafAngleDistribution::Verhoef
            && self.lidf_a.abs() + self.lidf_b.abs() > 1. + 1e-12
        {
            return Err(domain("Verhoef distribution requires |a|+|b| <= 1"));
        }
        Ok(())
    }
    fn geometry(&self) -> Parameters {
        Parameters {
            leaf_area_index: self.leaf_area_index,
            mean_leaf_angle_deg: self.mean_leaf_angle_deg,
            leaf_angle_distribution: match self.leaf_angle_distribution {
                LeafAngleDistribution::Campbell => Lidf::Campbell,
                LeafAngleDistribution::Verhoef => Lidf::Verhoef,
            },
            lidf_a: self.lidf_a,
            lidf_b: self.lidf_b,
            hotspot: self.hotspot,
            solar_zenith_deg: self.solar_zenith_deg,
            view_zenith_deg: self.view_zenith_deg,
            relative_azimuth_deg: self.relative_azimuth_deg,
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Point {
    pub radiance_w_m2_sr_um: f64,
    pub brightness_temperature_k: f64,
    pub effective_emissivity: f64,
    pub leaf_emission_weight: f64,
    pub soil_emission_weight: f64,
    pub sunlit_leaf_weight: f64,
    pub shaded_leaf_weight: f64,
    pub sunlit_soil_weight: f64,
    pub shaded_soil_weight: f64,
    pub sky_reflectance: f64,
    pub direct_reflectance_factor: f64,
    pub sunlit_leaf_radiance_w_m2_sr_um: f64,
    pub shaded_leaf_radiance_w_m2_sr_um: f64,
    pub sunlit_soil_radiance_w_m2_sr_um: f64,
    pub shaded_soil_radiance_w_m2_sr_um: f64,
    pub reflected_sky_radiance_w_m2_sr_um: f64,
    pub reflected_direct_radiance_w_m2_sr_um: f64,
    pub hemispherical_emissivity: f64,
    pub upwelling_flux_w_m2_um: f64,
    pub ground_downwelling_irradiance_w_m2_um: f64,
    pub ground_upwelling_flux_w_m2_um: f64,
}
#[derive(Debug, Clone, Copy)]
struct Transfer {
    weights: [f64; 4],
    sky: f64,
    direct: f64,
    hemi: [f64; 4],
    hemi_sky: f64,
    hemi_direct: f64,
    rdd: f64,
    tdd: f64,
    tsd: f64,
    tss: f64,
    gamma_d: f64,
    gamma_sdf: f64,
    dn: f64,
}
fn transfer(p: &Input) -> Transfer {
    let g = p.geometry();
    let c = Geometry::new(&g).coefficients(1. - p.leaf_emissivity, 0., &g);
    let soil = 1. - p.soil_emissivity;
    let dn = 1. - soil * c.rdd;
    let observer = c.tdo + c.too;
    let gd = if p.leaf_emissivity == 0. {
        0.
    } else {
        1. - c.rdd - c.tdd
    };
    let go = if p.leaf_emissivity == 0. {
        0.
    } else {
        1. - c.rdo - c.tdo - c.too
    };
    let leaf = go + observer * soil * gd / dn;
    let ground = observer * p.soil_emissivity / dn;
    let hot_leaf = p.leaf_emissivity * (c.gamma_so + observer * soil * c.gamma_sdf / dn);
    let hot_soil = p.soil_emissivity * (c.tsstoo + c.tss * (c.tdo + c.too * soil * c.rdd) / dn);
    let hemi_leaf = gd * (1. + c.tdd * soil / dn);
    let hemi_ground = c.tdd * p.soil_emissivity / dn;
    let hemi_hot_leaf = p.leaf_emissivity * (c.gamma_sdb + c.tdd * soil * c.gamma_sdf / dn);
    let hemi_hot_soil = c.tdd * p.soil_emissivity * c.tss / dn;
    Transfer {
        weights: [hot_leaf, leaf - hot_leaf, hot_soil, ground - hot_soil],
        sky: c.rdo + c.tdd * soil * observer / dn,
        direct: c.rso
            + c.tsstoo * soil
            + ((c.tss + c.tsd) * c.tdo + (c.tsd + c.tss * soil * c.rdd) * c.too) * soil / dn,
        hemi: [
            hemi_hot_leaf,
            hemi_leaf - hemi_hot_leaf,
            hemi_hot_soil,
            hemi_ground - hemi_hot_soil,
        ],
        hemi_sky: c.rdd + c.tdd * soil * c.tdd / dn,
        hemi_direct: c.rsd + (c.tsd + c.tss) * soil * c.tdd / dn,
        rdd: c.rdd,
        tdd: c.tdd,
        tsd: c.tsd,
        tss: c.tss,
        gamma_d: gd,
        gamma_sdf: c.gamma_sdf,
        dn,
    }
}
fn planck(w: f64, t: f64) -> Result<f64, ModelError> {
    planck_radiance(w, t, WavelengthDensity::PerMicrometre).map_err(|e| domain(e.to_string()))
}
fn brightness(w: f64, l: f64) -> Result<f64, ModelError> {
    inverse_planck(w, l, WavelengthDensity::PerMicrometre).map_err(|e| domain(e.to_string()))
}
fn source_radiances(p: &Input, w: f64) -> Result<[f64; 5], ModelError> {
    Ok([
        planck(w, p.sunlit_leaf_temperature_k)?,
        planck(w, p.shaded_leaf_temperature_k)?,
        planck(w, p.sunlit_soil_temperature_k)?,
        planck(w, p.shaded_soil_temperature_k)?,
        planck(w, p.sky_brightness_temperature_k)?,
    ])
}
fn radiance(p: &Input, t: &Transfer, b: &[f64; 5]) -> f64 {
    (0..4).map(|i| t.weights[i] * b[i]).sum::<f64>()
        + t.sky * b[4]
        + t.direct * p.direct_irradiance_w_m2_um / PI
}
pub fn evaluate(p: &Input) -> Result<Point, ModelError> {
    p.validate()?;
    point(p, &transfer(p))
}
fn point(p: &Input, t: &Transfer) -> Result<Point, ModelError> {
    let b = source_radiances(p, p.wavelength_nm)?;
    let contributions: Vec<f64> = (0..4).map(|i| t.weights[i] * b[i]).collect();
    let total = radiance(p, t, &b);
    let soil = 1. - p.soil_emissivity;
    let e_soil = p.soil_emissivity * (b[3] + t.tss * (b[2] - b[3]));
    let forcing = t.gamma_d * b[1] + p.leaf_emissivity * t.gamma_sdf * (b[0] - b[1]);
    let down = (t.tdd * b[4]
        + t.rdd * e_soil
        + forcing
        + (t.tsd + t.rdd * soil * t.tss) * p.direct_irradiance_w_m2_um / PI)
        / t.dn;
    let ground_down = PI * down + t.tss * p.direct_irradiance_w_m2_um;
    Ok(Point {
        radiance_w_m2_sr_um: total,
        brightness_temperature_k: brightness(p.wavelength_nm, total)?,
        effective_emissivity: 1. - t.sky,
        leaf_emission_weight: t.weights[0] + t.weights[1],
        soil_emission_weight: t.weights[2] + t.weights[3],
        sunlit_leaf_weight: t.weights[0],
        shaded_leaf_weight: t.weights[1],
        sunlit_soil_weight: t.weights[2],
        shaded_soil_weight: t.weights[3],
        sky_reflectance: t.sky,
        direct_reflectance_factor: t.direct,
        sunlit_leaf_radiance_w_m2_sr_um: contributions[0],
        shaded_leaf_radiance_w_m2_sr_um: contributions[1],
        sunlit_soil_radiance_w_m2_sr_um: contributions[2],
        shaded_soil_radiance_w_m2_sr_um: contributions[3],
        reflected_sky_radiance_w_m2_sr_um: t.sky * b[4],
        reflected_direct_radiance_w_m2_sr_um: t.direct * p.direct_irradiance_w_m2_um / PI,
        hemispherical_emissivity: 1. - t.hemi_sky,
        upwelling_flux_w_m2_um: PI
            * ((0..4).map(|i| t.hemi[i] * b[i]).sum::<f64>() + t.hemi_sky * b[4])
            + t.hemi_direct * p.direct_irradiance_w_m2_um,
        ground_downwelling_irradiance_w_m2_um: ground_down,
        ground_upwelling_flux_w_m2_um: soil * ground_down + PI * e_soil,
    })
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Output {
    #[serde(flatten)]
    pub point: Point,
    pub wavelengths_nm: Vec<f64>,
    pub spectral_radiance_w_m2_sr_um: Vec<f64>,
    pub spectral_brightness_temperature_k: Vec<f64>,
    pub principal_view_zenith_deg: Vec<f64>,
    pub principal_brightness_temperature_k: Vec<f64>,
    pub principal_effective_emissivity: Vec<f64>,
    pub hemisphere_view_zenith_deg: Vec<f64>,
    pub hemisphere_azimuth_deg: Vec<f64>,
    pub hemisphere_brightness_temperature_k: Vec<f64>,
    pub hemisphere_effective_emissivity: Vec<f64>,
    pub hotspot_brightness_temperature_k: f64,
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
        p.validate()?;
        let t = transfer(p);
        let selected = point(p, &t)?;
        let mut out = Output {
            point: selected,
            wavelengths_nm: (8000..=14000).step_by(100).map(f64::from).collect(),
            spectral_radiance_w_m2_sr_um: vec![],
            spectral_brightness_temperature_k: vec![],
            principal_view_zenith_deg: (-85..=85).map(f64::from).collect(),
            principal_brightness_temperature_k: vec![],
            principal_effective_emissivity: vec![],
            hemisphere_view_zenith_deg: vec![],
            hemisphere_azimuth_deg: vec![],
            hemisphere_brightness_temperature_k: vec![],
            hemisphere_effective_emissivity: vec![],
            hotspot_brightness_temperature_k: 0.,
        };
        for &w in &out.wavelengths_nm {
            let b = source_radiances(p, w)?;
            let l = radiance(p, &t, &b);
            out.spectral_radiance_w_m2_sr_um.push(l);
            out.spectral_brightness_temperature_k
                .push(brightness(w, l)?);
        }
        let b = source_radiances(p, p.wavelength_nm)?;
        let mut q = *p;
        q.view_zenith_deg = p.solar_zenith_deg;
        q.relative_azimuth_deg = 0.;
        out.hotspot_brightness_temperature_k =
            brightness(p.wavelength_nm, radiance(p, &transfer(&q), &b))?;
        for &a in &out.principal_view_zenith_deg {
            q.view_zenith_deg = a.abs();
            q.relative_azimuth_deg = if a < 0. { 180. } else { 0. };
            let u = transfer(&q);
            out.principal_brightness_temperature_k
                .push(brightness(p.wavelength_nm, radiance(p, &u, &b))?);
            out.principal_effective_emissivity.push(1. - u.sky);
        }
        for z in (0..=85).step_by(5) {
            for az in (0..360).step_by(10) {
                q.view_zenith_deg = z as f64;
                q.relative_azimuth_deg = if az > 180 { 360 - az } else { az } as f64;
                let u = transfer(&q);
                out.hemisphere_view_zenith_deg.push(z as f64);
                out.hemisphere_azimuth_deg.push(az as f64);
                out.hemisphere_brightness_temperature_k
                    .push(brightness(p.wavelength_nm, radiance(p, &u, &b))?);
                out.hemisphere_effective_emissivity.push(1. - u.sky);
            }
        }
        self.validate_output(&out)?;
        Ok(out)
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
    fn validate_output(&self, o: &Output) -> Result<(), ModelError> {
        let p = &o.point;
        if [
            p.radiance_w_m2_sr_um,
            p.brightness_temperature_k,
            p.effective_emissivity,
            p.leaf_emission_weight,
            p.soil_emission_weight,
            p.sunlit_leaf_weight,
            p.shaded_leaf_weight,
            p.sunlit_soil_weight,
            p.shaded_soil_weight,
            p.sky_reflectance,
            p.direct_reflectance_factor,
            p.sunlit_leaf_radiance_w_m2_sr_um,
            p.shaded_leaf_radiance_w_m2_sr_um,
            p.sunlit_soil_radiance_w_m2_sr_um,
            p.shaded_soil_radiance_w_m2_sr_um,
            p.reflected_sky_radiance_w_m2_sr_um,
            p.reflected_direct_radiance_w_m2_sr_um,
            p.hemispherical_emissivity,
            p.upwelling_flux_w_m2_um,
            p.ground_downwelling_irradiance_w_m2_um,
            p.ground_upwelling_flux_w_m2_um,
            o.hotspot_brightness_temperature_k,
        ]
        .into_iter()
        .chain(o.wavelengths_nm.iter().copied())
        .chain(o.spectral_radiance_w_m2_sr_um.iter().copied())
        .chain(o.spectral_brightness_temperature_k.iter().copied())
        .chain(o.principal_view_zenith_deg.iter().copied())
        .chain(o.principal_brightness_temperature_k.iter().copied())
        .chain(o.principal_effective_emissivity.iter().copied())
        .chain(o.hemisphere_view_zenith_deg.iter().copied())
        .chain(o.hemisphere_azimuth_deg.iter().copied())
        .chain(o.hemisphere_brightness_temperature_k.iter().copied())
        .chain(o.hemisphere_effective_emissivity.iter().copied())
        .any(|v| !v.is_finite())
            || o.wavelengths_nm.len() != 61
            || o.spectral_radiance_w_m2_sr_um.len() != 61
            || o.spectral_brightness_temperature_k.len() != 61
            || o.principal_view_zenith_deg.len() != 171
            || o.principal_brightness_temperature_k.len() != 171
            || o.principal_effective_emissivity.len() != 171
            || o.hemisphere_view_zenith_deg.len() != 648
            || o.hemisphere_azimuth_deg.len() != 648
            || o.hemisphere_brightness_temperature_k.len() != 648
            || o.hemisphere_effective_emissivity.len() != 648
        {
            return Err(ModelError::new(
                ErrorKind::Internal,
                "nonfinite or invalid 4SAIL-TIR output",
            ));
        }
        Ok(())
    }
}
