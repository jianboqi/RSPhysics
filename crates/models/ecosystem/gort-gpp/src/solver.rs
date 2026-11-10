//! Selected Main4PSN path; constants/indexing preserve the supplied reference.
use super::{Input, ModelError, NumericalConvention, Output};
// This original constant is part of the reference implementation's numerical identity.
#[allow(clippy::approx_constant)]
const PI: f64 = 3.1415926;
const CROWNS: usize = 10;

fn extinction(x: f64, theta: f64) -> f64 {
    (x * x + theta.tan().powi(2)).sqrt() / (x + 1.774 * (x + 1.182).powf(-0.733))
}
fn gaps(p: &Input) -> (Vec<f64>, Vec<f64>, f64, f64) {
    let r = p.crown_radius_m;
    let density = p.crown_density_m2;
    let b = p.crown_axis_ratio;
    let dh = p.crown_top_height_m - p.crown_base_height_m;
    let mut factorial = [1.; CROWNS + 1];
    for n in 1..=CROWNS {
        factorial[n] = factorial[n - 1] * n as f64;
    }
    let mut bgap = vec![0.; 90];
    let mut wgap = vec![0.; 90];
    let favd = p.leaf_area_index / (1.333333 * density * PI * b * r.powi(3));
    for i in 0..90 {
        let theta = i as f64 * PI / 180.;
        let theta_p = (b * theta.tan()).atan();
        bgap[i] = (-density * PI * r * r / theta_p.cos()).exp();
        let sp = dh / theta.cos();
        let t = density / dh * PI * r * r * sp;
        let mut sr = 0.;
        for (n, &fac) in factorial.iter().enumerate().skip(1) {
            let pn = t.powi(n as i32) * (-t).exp() / (fac * (1. - (-t).exp()));
            sr += pn * n as f64 * (4. / 3.) * r;
        }
        if p.numerical_convention == NumericalConvention::Corrected {
            sr = t / (-(-t).exp_m1()) * (4. / 3.) * r;
        }
        wgap[i] = (-favd * b * 0.5 * sr).exp();
    }
    let integrate = |v: &[f64]| -> f64 {
        let mut s = 0.;
        // Preserve original endpoint convention, including shared sin(2*theta).
        for i in 0..89 {
            let theta = (i + 1) as f64 * PI / 180.;
            let left = if p.numerical_convention == NumericalConvention::Corrected {
                i as f64 * PI / 180.
            } else {
                theta
            };
            s += (v[i] * (2. * left).sin() + v[i + 1] * (2. * theta).sin()) / 2. * PI / 180.;
        }
        s
    };
    let bo = integrate(&bgap);
    let wo = integrate(&wgap);
    (bgap, wgap, bo, wo)
}
fn quadratic(a: f64, b: f64, c: f64, corrected: bool) -> Result<f64, ModelError> {
    if a.abs() < 1e-14 {
        if corrected && b.abs() > 1e-14 {
            return Ok(-c / b);
        }
        return Err(super::domain(
            "degenerate original photosynthesis quadratic",
        ));
    }
    let d = b * b - 4. * a * c;
    Ok(if d < 0. {
        -1.
    } else {
        if corrected && b > 0. {
            -2. * c / (b + d.sqrt())
        } else {
            (-b + d.sqrt()) / (2. * a)
        }
    })
}
fn assimilation(p: &Input, apar: f64, vmax: f64) -> Result<f64, ModelError> {
    let t = p.air_temperature_c;
    let ko = 248. * 1.2_f64.powf((t - 25.) / 10.) * 100.;
    let kc = 404. * 2.1_f64.powf((t - 25.) / 10.) * 0.1;
    let o2 = 0.21 * p.pressure_pa;
    let gamma = (0.5 * 0.21 * kc * o2 / ko) * 1e6 / p.pressure_pa;
    let kc = kc * 1e6 / p.pressure_pa;
    let svp = 610.7 * (17.38 * t / (239. + t)).exp();
    let rh = 1. - p.vapour_pressure_deficit_pa / svp;
    let ca = p.co2_ppm;
    let g0 = p.minimum_stomatal_conductance;
    let g1 = p.stomatal_slope * rh / ca;
    let k = kc * (1. + o2 / ko);
    let rd = 0.015 * vmax;
    let corrected = p.numerical_convention == NumericalConvention::Corrected;
    let av = quadratic(
        ca * g1 + g1 * k - 1.,
        ca * g0 + g0 * k - vmax * ca * g1 + vmax + vmax * g1 * gamma + rd * ca * g1 - rd
            + rd * g1 * k,
        -vmax * ca * g0 + vmax * gamma * g0 + rd * ca * g0 + rd * k * g0,
        corrected,
    )?;
    let jmax = 2.1 * vmax;
    let b = -jmax - apar / p.electron_photon_ratio;
    let c = jmax * apar / p.electron_photon_ratio;
    let j = (-b - (b * b - 4. * 0.7 * c).sqrt()) / (2. * 0.7);
    let aj = quadratic(
        4.5 * ca * g1 - 4.5 + 10.5 * gamma * g1,
        4.5 * ca * g0 + 10.5 * gamma * g0 - j * g1 * ca + j + gamma * g1 * j + 4.5 * rd * ca * g1
            - 4.5 * rd
            + 10.5 * gamma * g1 * rd,
        -j * ca * g0 + 4.5 * rd * ca * g0 + 10.5 * gamma * g0 * rd,
        corrected,
    )?;
    Ok(if av == -1. && aj == -1. {
        0.
    } else if av == -1. {
        aj
    } else if aj == -1. {
        av
    } else {
        av.min(aj)
    })
}
pub(super) fn simulate(p: &Input) -> Result<Output, ModelError> {
    let (bg, wg, bo, wo) = gaps(p);
    let theta = p.solar_zenith_deg * PI / 180.;
    let kb = extinction(p.leaf_angle_parameter, theta);
    // Main4PSN int16 rounds degrees, but lacks radiation's explicit +1.
    let corrected = p.numerical_convention == NumericalConvention::Corrected;
    if !corrected && p.solar_zenith_deg < 0.5 {
        return Err(super::domain(
            "legacy MATLAB convention requires zenith >=0.5 degrees",
        ));
    }
    let main_index = if corrected {
        p.solar_zenith_deg.round() as usize
    } else {
        p.solar_zenith_deg.round() as usize - 1
    };
    let notgap = 1. - bg[main_index] - (1. - bg[main_index]) * wg[main_index];
    let sunlit = notgap / kb;
    let shaded = p.leaf_area_index - sunlit;
    if sunlit <= 0. || shaded <= 0. {
        return Err(super::domain(
            "source parameterization requires positive sunlit and shaded leaf areas",
        ));
    }
    let radiation_index = ((theta.cos().acos() * 180. / PI + 0.5 + 1.).trunc() as usize) - 1;
    let open = bo + (1. - bo) * wo;
    let direct_gap = bg[radiation_index] + (1. - bg[radiation_index]) * wg[radiation_index];
    let omega = p.leaf_single_scattering_albedo;
    let br = p.background_reflectance;
    let diffuse = p.diffuse_par_umol_m2_s;
    let direct = p.direct_par_umol_m2_s;
    let reflected = diffuse * (1. - open) * omega / 2. + direct * (1. - direct_gap) * omega / 2.;
    let mut absorbed = diffuse * (1. - open) * (1. - omega);
    let absorbed_direct = direct * (1. - direct_gap) * (1. - omega);
    let transmitted = diffuse * open + direct * direct_gap;
    let mut floor = (transmitted + reflected) * (1. - br);
    let mut bg_reflected = (transmitted + reflected) * br;
    for _ in 2..=5 {
        let reflected = bg_reflected * (1. - open) * omega / 2.;
        absorbed += bg_reflected * (1. - open) * (1. - omega);
        bg_reflected = reflected * br;
        floor += reflected * (1. - br);
    }
    let apar_shaded = absorbed / p.leaf_area_index;
    let apar_sunlit = if corrected {
        absorbed_direct / sunlit + apar_shaded
    } else {
        absorbed_direct * kb + apar_shaded
    };
    let vc =
        p.vcmax25_young * p.young_leaf_fraction + p.vcmax25_mature * (1. - p.young_leaf_fraction);
    let vc_canopy = p.leaf_area_index * vc * (1. - (-0.1_f64).exp()) / 0.1;
    let vc_sun = p.leaf_area_index * vc * (1. - (-0.1 - kb * p.leaf_area_index).exp())
        / (0.1 + kb * p.leaf_area_index);
    let temp = (0.051 * (p.air_temperature_c - 25.)).exp()
        / (1. + (0.205 * (p.air_temperature_c - 41.)).exp());
    let asun = assimilation(p, apar_sunlit, vc_sun / sunlit * temp)?;
    let ashade = assimilation(p, apar_shaded, (vc_canopy - vc_sun) / shaded * temp)?;
    Ok(Output {
        net_assimilation_umol_m2_s: asun * sunlit + ashade * shaded,
        sunlit_assimilation_umol_m2_s: asun,
        shaded_assimilation_umol_m2_s: ashade,
        sunlit_leaf_area_index: sunlit,
        shaded_leaf_area_index: shaded,
        sunlit_absorbed_par: apar_sunlit,
        shaded_absorbed_par: apar_shaded,
        between_crown_openness: bo,
        within_crown_openness: wo,
        floor_absorbed_par: floor,
        gap_zenith_deg: (0..90).map(f64::from).collect(),
        between_crown_gaps: bg,
        within_crown_gaps: wg,
    })
}
