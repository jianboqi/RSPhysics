use super::{
    BigLeafInput, GasExchangeMode, Input, LeuningCompensation, ModelError, Output, Parameters,
    StomatalModel, TemperatureResponse, TpuLimitation, domain,
};
use schemars::JsonSchema;
use serde::Serialize;

const GAS_CONSTANT: f64 = 8.31446261815324;
const REFERENCE_TEMPERATURE_K: f64 = 298.15;
const REFERENCE_PRESSURE_PA: f64 = 101325.;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Point {
    pub net_assimilation_umol_m2_s: f64,
    /// Net photorespiratory carbon assimilation before subtraction of mitochondrial day respiration; not GPP.
    pub assimilation_before_day_respiration_umol_m2_s: f64,
    pub rubisco_limited_assimilation_umol_m2_s: f64,
    pub electron_limited_assimilation_umol_m2_s: f64,
    /// Potential simple TPU branch, also reported when disabled; consult tpu_limitation_enabled.
    pub tpu_limited_assimilation_umol_m2_s: f64,
    pub day_respiration_umol_m2_s: f64,
    pub vcmax_umol_m2_s: f64,
    pub jmax_umol_m2_s: f64,
    pub electron_transport_umol_m2_s: f64,
    pub kc_umol_mol: f64,
    pub ko_mmol_mol: f64,
    pub gamma_star_umol_mol: f64,
    pub michaelis_menten_co2_umol_mol: f64,
    pub intercellular_co2_umol_mol: Option<f64>,
    pub stomatal_conductance_water_mol_m2_s: Option<f64>,
    pub stomatal_conductance_co2_mol_m2_s: Option<f64>,
    /// Linear molar vapour-gradient flux from leaf interior to the prescribed leaf surface.
    pub transpiration_mmol_m2_s: Option<f64>,
    pub co2_flux_residual_umol_m2_s: Option<f64>,
    pub solver_iterations: u32,
    /// 0 Rubisco, 1 electron transport, 2 TPU, 3 co-limitation, 4 no canopy.
    pub limitation_code: u8,
    pub gas_exchange_computed: bool,
    pub intercellular_co2_defined: bool,
    pub tpu_limitation_enabled: bool,
}

#[derive(Clone, Copy)]
struct Rates {
    vc: f64,
    jm: f64,
    rd: f64,
    tp: f64,
    kc: f64,
    ko: f64,
    gamma: f64,
    km: f64,
}

pub(crate) fn saturation_pressure_kpa(t: f64) -> f64 {
    // FAO-56 equation 11; the temperature is the prescribed leaf temperature.
    0.6108 * (17.27 * t / (t + 237.3)).exp()
}
pub(crate) fn capacity_factor(lai: f64, kn: f64) -> f64 {
    if kn == 0. {
        lai
    } else {
        -(-kn * lai).exp_m1() / kn
    }
}
fn arrhenius(activation: f64, tk: f64) -> f64 {
    (activation / GAS_CONSTANT * (1. / REFERENCE_TEMPERATURE_K - 1. / tk)).exp()
}
fn temperature_factor(
    p: &Parameters,
    tk: f64,
    activation: f64,
    deactivation: f64,
    entropy: f64,
) -> f64 {
    let mut factor = arrhenius(activation, tk);
    if p.temperature_response == TemperatureResponse::PeakedArrhenius {
        let inhibition = |t: f64| ((entropy * t - deactivation) / (GAS_CONSTANT * t)).exp();
        factor *= (1. + inhibition(REFERENCE_TEMPERATURE_K)) / (1. + inhibition(tk));
    }
    factor
}
fn rates(p: &Parameters, capacity: f64) -> Rates {
    let tk = p.leaf_temperature_c + 273.15;
    let kc = 404.9 * arrhenius(79430., tk) * REFERENCE_PRESSURE_PA / p.pressure_pa;
    let ko = 278.4 * arrhenius(36380., tk) * REFERENCE_PRESSURE_PA / p.pressure_pa;
    let gamma = 42.75 * arrhenius(37830., tk) * p.oxygen_mmol_mol / 210.;
    Rates {
        vc: capacity
            * p.vcmax25_umol_m2_s
            * temperature_factor(
                p,
                tk,
                p.vcmax_activation_j_mol,
                p.vcmax_deactivation_j_mol,
                p.vcmax_entropy_j_mol_k,
            ),
        jm: capacity
            * p.jmax25_umol_m2_s
            * temperature_factor(
                p,
                tk,
                p.jmax_activation_j_mol,
                p.jmax_deactivation_j_mol,
                p.jmax_entropy_j_mol_k,
            ),
        rd: capacity * p.respiration25_umol_m2_s * arrhenius(p.respiration_activation_j_mol, tk),
        tp: capacity * p.tpu_umol_m2_s,
        kc,
        ko,
        gamma,
        km: kc * (1. + p.oxygen_mmol_mol / ko),
    }
}
fn electron_transport(p: &Parameters, r: Rates, par: f64) -> f64 {
    let absorbed = p.electron_quantum_yield * par;
    let sum = absorbed + r.jm;
    if sum == 0. {
        return 0.;
    }
    let discriminant = ((absorbed - r.jm).powi(2)
        + 4. * (1. - p.light_response_curvature) * absorbed * r.jm)
        .sqrt();
    2. * absorbed * r.jm / (sum + discriminant)
}
fn biochemical(p: &Parameters, r: Rates, j: f64, ci: f64) -> (f64, f64, f64, f64, u8) {
    let ac = r.vc * (ci - r.gamma) / (ci + r.km) - r.rd;
    let aj = if ci + 2. * r.gamma == 0. {
        j / 4. - r.rd
    } else {
        j / 4. * (ci - r.gamma) / (ci + 2. * r.gamma) - r.rd
    };
    let ap = 3. * r.tp - r.rd;
    // FvCB limits the positive carboxylation rates before applying 1-Gamma*/Ci.
    // Below Gamma*, this factor is negative: minimum carboxylation therefore
    // corresponds to the maximum net branch. This also gives the correct dark
    // and zero-capacity limits without clipping negative assimilation.
    let mut a = if ci < r.gamma { ac.max(aj) } else { ac.min(aj) };
    if p.tpu_limitation == TpuLimitation::Simple {
        a = a.min(ap);
    }
    let close = |v: f64| (v - a).abs() <= 1e-10 * a.abs().max(1.);
    let mask = u8::from(close(ac))
        + 2 * u8::from(close(aj))
        + 4 * u8::from(p.tpu_limitation == TpuLimitation::Simple && close(ap));
    let limitation = match mask {
        1 => 0,
        2 => 1,
        4 => 2,
        _ => 3,
    };
    (a, ac, aj, ap, limitation)
}
fn compensation(p: &Parameters, r: Rates, j: f64) -> Option<f64> {
    if p.leuning_compensation == LeuningCompensation::GammaStar {
        return Some(r.gamma);
    }
    // The net compensation point is the zero of min(Ac, Aj, Ap), including Rd.
    if r.vc <= r.rd
        || j <= 4. * r.rd
        || (p.tpu_limitation == TpuLimitation::Simple && 3. * r.tp <= r.rd)
    {
        return None;
    }
    let rubisco = (r.vc * r.gamma + r.rd * r.km) / (r.vc - r.rd);
    let electron = r.gamma * (j + 8. * r.rd) / (j - 4. * r.rd);
    Some(rubisco.max(electron))
}
fn water_conductance(p: &Parameters, r: Rates, j: f64, area: f64, a: f64) -> f64 {
    let residual = p.minimum_stomatal_conductance_water_mol_m2_s * area;
    if a <= 0. {
        return residual;
    }
    let cs = p.co2_surface_umol_mol;
    let d = p.vapour_pressure_deficit_kpa;
    let enhancement = match p.stomatal_model {
        StomatalModel::BallBerry => {
            p.ball_berry_slope * a * (1. - d / saturation_pressure_kpa(p.leaf_temperature_c)) / cs
        }
        StomatalModel::Medlyn => {
            1.6 * (1. + p.medlyn_slope_kpa_sqrt / d.max(p.medlyn_vpd_floor_kpa).sqrt()) * a / cs
        }
        StomatalModel::Leuning => match compensation(p, r, j) {
            Some(gamma) if cs > gamma => {
                1.6 * p.leuning_slope_co2 * a / ((cs - gamma) * (1. + d / p.leuning_d0_kpa))
            }
            _ => 0.,
        },
    };
    residual + enhancement
}
fn point(p: &Parameters, par: f64, capacity: f64, area: f64) -> Result<Point, ModelError> {
    let r = rates(p, capacity);
    let j = electron_transport(p, r, par);
    let enabled = p.tpu_limitation == TpuLimitation::Simple;
    let stomatal = p.gas_exchange_mode == GasExchangeMode::Stomatal;
    if area == 0. {
        return Ok(Point {
            net_assimilation_umol_m2_s: 0.,
            assimilation_before_day_respiration_umol_m2_s: 0.,
            rubisco_limited_assimilation_umol_m2_s: 0.,
            electron_limited_assimilation_umol_m2_s: 0.,
            tpu_limited_assimilation_umol_m2_s: 0.,
            day_respiration_umol_m2_s: 0.,
            vcmax_umol_m2_s: 0.,
            jmax_umol_m2_s: 0.,
            electron_transport_umol_m2_s: 0.,
            kc_umol_mol: r.kc,
            ko_mmol_mol: r.ko,
            gamma_star_umol_mol: r.gamma,
            michaelis_menten_co2_umol_mol: r.km,
            intercellular_co2_umol_mol: None,
            stomatal_conductance_water_mol_m2_s: stomatal.then_some(0.),
            stomatal_conductance_co2_mol_m2_s: stomatal.then_some(0.),
            transpiration_mmol_m2_s: stomatal.then_some(0.),
            co2_flux_residual_umol_m2_s: stomatal.then_some(0.),
            solver_iterations: 0,
            limitation_code: 4,
            gas_exchange_computed: false,
            intercellular_co2_defined: false,
            tpu_limitation_enabled: enabled,
        });
    }
    let mut ci = p.intercellular_co2_umol_mol;
    let mut iterations = 0;
    if stomatal && p.minimum_stomatal_conductance_water_mol_m2_s == 0. {
        // With g0=0, all three daylight formulations are gs=k*A. Select the
        // positive-assimilation branch instead of the degenerate closed-stomata root.
        let slope = water_conductance(p, r, j, area, 1.);
        if slope <= 0. {
            return Err(domain("g0=0 provides no positive gas-exchange branch"));
        }
        ci = p.co2_surface_umol_mol - 1.6 / slope;
        if !(0. ..=5000.).contains(&ci) || biochemical(p, r, j, ci).0 <= 0. {
            return Err(domain(
                "g0=0 provides no unique positive gas-exchange solution; prescribe Ci or use nonzero residual conductance",
            ));
        }
    } else if stomatal {
        let residual = |ci: f64| {
            let a = biochemical(p, r, j, ci).0;
            a - water_conductance(p, r, j, area, a) / 1.6 * (p.co2_surface_umol_mol - ci)
        };
        let (mut lo, mut hi) = (0., 5000.);
        if residual(lo) > 0. || residual(hi) < 0. {
            return Err(domain(
                "no gas-exchange root within Ci 0–5000 umol/mol; revise respiration, CO2 or residual conductance",
            ));
        }
        for step in 1..=100 {
            iterations = step;
            ci = (lo + hi) / 2.;
            let value = residual(ci);
            if value.abs() < 1e-10 || hi - lo < 1e-11 {
                break;
            }
            if value > 0. {
                hi = ci;
            } else {
                lo = ci;
            }
        }
    }
    let (a, ac, aj, ap, limitation) = biochemical(p, r, j, ci);
    let gs = stomatal.then(|| water_conductance(p, r, j, area, a));
    let flux_residual = gs.map(|g| a - g / 1.6 * (p.co2_surface_umol_mol - ci));
    if flux_residual.is_some_and(|v| v.abs() > 1e-8 * a.abs().max(1.)) {
        return Err(domain("CO2 gas-exchange root did not converge"));
    }
    Ok(Point {
        net_assimilation_umol_m2_s: a,
        assimilation_before_day_respiration_umol_m2_s: a + r.rd,
        rubisco_limited_assimilation_umol_m2_s: ac,
        electron_limited_assimilation_umol_m2_s: aj,
        tpu_limited_assimilation_umol_m2_s: ap,
        day_respiration_umol_m2_s: r.rd,
        vcmax_umol_m2_s: r.vc,
        jmax_umol_m2_s: r.jm,
        electron_transport_umol_m2_s: j,
        kc_umol_mol: r.kc,
        ko_mmol_mol: r.ko,
        gamma_star_umol_mol: r.gamma,
        michaelis_menten_co2_umol_mol: r.km,
        intercellular_co2_umol_mol: Some(ci),
        stomatal_conductance_water_mol_m2_s: gs,
        stomatal_conductance_co2_mol_m2_s: gs.map(|v| v / 1.6),
        transpiration_mmol_m2_s: gs
            .map(|v| v * p.vapour_pressure_deficit_kpa * 1e6 / p.pressure_pa),
        co2_flux_residual_umol_m2_s: flux_residual,
        solver_iterations: iterations,
        limitation_code: limitation,
        gas_exchange_computed: stomatal,
        intercellular_co2_defined: true,
        tpu_limitation_enabled: enabled,
    })
}
pub fn evaluate_leaf(p: &Input) -> Result<Point, ModelError> {
    p.validate()?;
    point(&p.parameters, p.absorbed_par_umol_m2_s, 1., 1.)
}
pub fn evaluate_big_leaf(p: &BigLeafInput) -> Result<Point, ModelError> {
    p.validate()?;
    point(
        &p.parameters,
        p.absorbed_par_ground_umol_m2_s,
        capacity_factor(p.leaf_area_index, p.nitrogen_extinction_coefficient),
        p.leaf_area_index,
    )
}
pub(crate) fn simulate(
    p: &Parameters,
    par: f64,
    capacity: f64,
    area: f64,
) -> Result<Output, ModelError> {
    let selected = point(p, par, capacity, area)?;
    let r = rates(p, capacity);
    let j = electron_transport(p, r, par);
    let ci_axis: Vec<f64> = (0..=60).map(|i| f64::from(i) * 25.).collect();
    let responses: Vec<_> = ci_axis
        .iter()
        .map(|&ci| {
            if area == 0. {
                (0., 0., 0., 0., 4)
            } else {
                biochemical(p, r, j, ci)
            }
        })
        .collect();
    let light_axis: Vec<f64> = (0..=50).map(|i| f64::from(i) * 80.).collect();
    // This diagnostic holds the selected Ci fixed; it is not a second stomatal-feedback experiment.
    let light = light_axis
        .iter()
        .map(|&value| {
            selected.intercellular_co2_umol_mol.map_or(0., |ci| {
                biochemical(p, r, electron_transport(p, r, value), ci).0
            })
        })
        .collect();
    Ok(Output {
        point: selected,
        capacity_area_factor: capacity,
        leaf_area_factor: area,
        ci_axis_umol_mol: ci_axis,
        ci_net_assimilation: responses.iter().map(|v| v.0).collect(),
        ci_rubisco_limited_assimilation: responses.iter().map(|v| v.1).collect(),
        ci_electron_limited_assimilation: responses.iter().map(|v| v.2).collect(),
        light_axis_umol_m2_s: light_axis,
        light_net_assimilation: light,
    })
}
pub(crate) fn validate_output(o: &Output) -> Result<(), ModelError> {
    let p = &o.point;
    let finite = [
        p.net_assimilation_umol_m2_s,
        p.assimilation_before_day_respiration_umol_m2_s,
        p.rubisco_limited_assimilation_umol_m2_s,
        p.electron_limited_assimilation_umol_m2_s,
        p.tpu_limited_assimilation_umol_m2_s,
        p.day_respiration_umol_m2_s,
        p.vcmax_umol_m2_s,
        p.jmax_umol_m2_s,
        p.electron_transport_umol_m2_s,
        p.kc_umol_mol,
        p.ko_mmol_mol,
        p.gamma_star_umol_mol,
        p.michaelis_menten_co2_umol_mol,
        o.capacity_area_factor,
        o.leaf_area_factor,
    ];
    let optional = [
        p.intercellular_co2_umol_mol,
        p.stomatal_conductance_water_mol_m2_s,
        p.stomatal_conductance_co2_mol_m2_s,
        p.transpiration_mmol_m2_s,
        p.co2_flux_residual_umol_m2_s,
    ];
    if finite.iter().any(|v| !v.is_finite())
        || optional.into_iter().flatten().any(|v| !v.is_finite())
        || p.limitation_code > 4
        || o.ci_axis_umol_mol.len() != 61
        || o.ci_net_assimilation.len() != 61
        || o.ci_rubisco_limited_assimilation.len() != 61
        || o.ci_electron_limited_assimilation.len() != 61
        || o.light_axis_umol_m2_s.len() != 51
        || o.light_net_assimilation.len() != 51
        || o.ci_axis_umol_mol
            .iter()
            .chain(&o.ci_net_assimilation)
            .chain(&o.ci_rubisco_limited_assimilation)
            .chain(&o.ci_electron_limited_assimilation)
            .chain(&o.light_axis_umol_m2_s)
            .chain(&o.light_net_assimilation)
            .any(|v| !v.is_finite())
    {
        return Err(ModelError::new(
            super::ErrorKind::Internal,
            "invalid FvCB output",
        ));
    }
    Ok(())
}
