//! Supplied JPW tauw1i soil-forward path, fixed 1.41 GHz.
use super::{Input, ModelError, Output};
use num_complex::Complex64 as C;

fn permittivity(p: &Input, tg: f64) -> C {
    let f = 1.41e9;
    let ts = tg - 273.15;
    let m = p.soil_moisture;
    if ts <= -0.5 {
        return C::new(5., -0.5);
    }
    if m < 0.02 && p.sand_fraction >= 0.9 {
        // The supplied dry-sand branch has the opposite loss sign; preserve it.
        return C::new(2.53, 0.) + 0.26 / C::new(1., -f / 0.27e9) + C::new(0., 0.002);
    }
    let sal: f64 = 0.65;
    let xa =
        1. + 1.613e-5 * ts * sal - 3.656e-3 * sal + 3.21e-5 * sal * sal - 4.232e-7 * sal.powi(3);
    let epw0 = xa * (87.134 - 1.949e-1 * ts - 1.276e-2 * ts * ts + 2.491e-4 * ts.powi(3));
    let xb =
        1. + 2.282e-5 * ts * sal - 7.638e-4 * sal - 7.76e-6 * sal * sal + 1.105e-8 * sal.powi(3);
    let topi = xb * (1.1109e-10 - 3.824e-12 * ts + 6.938e-14 * ts * ts - 5.096e-16 * ts.powi(3));
    let seff =
        -1.645 + 1.939 * p.bulk_density_g_cm3 - 2.256 * p.sand_fraction + 1.594 * p.clay_fraction;
    let epfw = C::new(4.9, 0.) + (epw0 - 4.9) / C::new(1., topi * f)
        - C::new(
            0.,
            seff * (2.66 - p.bulk_density_g_cm3)
                / (2. * std::f64::consts::PI * f * 8.854e-12 * 2.66 * m),
        );
    let b1 = 1.275 - 0.519 * p.sand_fraction - 0.152 * p.clay_fraction;
    let b2 = 1.338 - 0.603 * p.sand_fraction - 0.166 * p.clay_fraction;
    let es: f64 = (1.01_f64 + 0.44 * 2.66).powi(2) - 0.062;
    let x =
        1. + p.bulk_density_g_cm3 * (es.powf(0.65) - 1.) / 2.66 + m.powf(b1) * epfw.re.powf(0.65)
            - m;
    let y = m.powf(b2) * epfw.im.abs().powf(0.65);
    C::new(x.powf(1. / 0.65), -y.powf(1. / 0.65))
}

struct Point {
    tv: f64,
    th: f64,
    rv: f64,
    rh: f64,
    sky: f64,
    gv: f64,
    gh: f64,
}
fn point(p: &Input, tg: f64, eps: C, angle: f64) -> Point {
    let mu = angle.to_radians().cos();
    let sin2 = 1. - mu * mu;
    // Explicit original complex Fresnel operations, retaining its loss convention.
    let cs2 = C::new(sin2.sqrt(), 0.) * (C::new(1., 0.) / eps).sqrt();
    let cc2 = (C::new(1., 0.) - cs2 * cs2).sqrt();
    let root = eps.sqrt();
    let h = (mu - root * cc2) / (mu + root * cc2);
    let v = (root * mu - cc2) / (root * mu + cc2);
    let rv0 = v.norm_sqr();
    let rh0 = h.norm_sqr();
    let q = p.roughness_mixing;
    let rv =
        ((1. - q) * rv0 + q * rh0) * (-p.roughness_height * mu.powf(p.roughness_exponent_v)).exp();
    let rh =
        ((1. - q) * rh0 + q * rv0) * (-p.roughness_height * mu.powf(p.roughness_exponent_h)).exp();
    let tauh = p.nadir_optical_depth * (mu * mu + p.canopy_structure_h * sin2);
    let tauv = p.nadir_optical_depth
        * (mu * mu + p.canopy_structure_h * p.canopy_structure_v_ratio * sin2);
    let gh = (-tauh / mu).exp();
    let gv = (-tauv / mu).exp();
    // Original sky model uses input para(3) before forward effective-T update.
    let tauat = (-4.011 - 0.00334 * p.sky_air_temperature_k).exp();
    let taeq = (5.051 + 0.001727 * p.sky_air_temperature_k + 0.00192 * 10.).exp();
    let atmosphere = (-tauat / mu).exp();
    let sky = taeq * (1. - atmosphere) + 2.7 * atmosphere;
    let tc = p.canopy_temperature_k;
    let omv = p.scattering_albedo_h + p.scattering_albedo_v_difference;
    let tv = (1. - omv) * (1. - gv) * (1. + rv * gv) * tc + ((1. - rv) * tg + sky * rv * gv) * gv;
    let th = (1. - p.scattering_albedo_h) * (1. - gh) * (1. + rh * gh) * tc
        + ((1. - rh) * tg + sky * rh * gh) * gh;
    Point {
        tv,
        th,
        rv,
        rh,
        sky,
        gv,
        gh,
    }
}
pub(super) fn simulate(p: &Input) -> Result<Output, ModelError> {
    let tg = p.deep_temperature_k
        + (p.soil_moisture / p.temperature_moisture_scale).powf(p.temperature_moisture_exponent)
            * (p.surface_temperature_k - p.deep_temperature_k);
    if !tg.is_finite() || !(250. ..=330.).contains(&tg) {
        return Err(super::domain(
            "effective soil temperature must be within [250,330] K",
        ));
    }
    let eps = permittivity(p, tg);
    let out = point(p, tg, eps, p.observation_zenith_deg);
    let angles: Vec<_> = (0..=80).map(f64::from).collect();
    let mut vs = Vec::with_capacity(81);
    let mut hs = Vec::with_capacity(81);
    for &angle in &angles {
        let pt = point(p, tg, eps, angle);
        vs.push(pt.tv);
        hs.push(pt.th);
    }
    Ok(Output {
        brightness_temperature_v_k: out.tv,
        brightness_temperature_h_k: out.th,
        effective_soil_temperature_k: tg,
        soil_permittivity_real: eps.re,
        soil_permittivity_imaginary: eps.im,
        soil_reflectivity_v: out.rv,
        soil_reflectivity_h: out.rh,
        sky_brightness_temperature_k: out.sky,
        transmissivity_v: out.gv,
        transmissivity_h: out.gh,
        sweep_zenith_deg: angles,
        sweep_brightness_temperature_v_k: vs,
        sweep_brightness_temperature_h_k: hs,
    })
}
