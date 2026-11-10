//! Typed NumPy batch collector. Only selectors and manifests use JSON.
use numpy::{
    IntoPyArray, PyReadonlyArray2, PyReadonlyArray3, PyUntypedArrayMethods,
    ndarray::{ArrayD, IxDyn},
};
use pyo3::{prelude::*, types::PyDict};
use rsphysics_model_api::{ErrorKind, ModelError, PhysicsModel};
use rsphysics_runtime::RunManifest;
use serde_json::{Map, Value};
use std::{collections::BTreeMap, sync::OnceLock};

const GSV: &[&str] = &["c1", "c2", "c3", "csm"];
const BSM: &[&str] = &[
    "soil_brightness",
    "shape_latitude_deg",
    "shape_longitude_deg",
    "soil_moisture_percent",
    "moisture_capacity_percent",
    "water_film_thickness_cm",
];
const RPV: &[&str] = &[
    "rho0",
    "minnaert_k",
    "phase_asymmetry",
    "hotspot_rho",
    "solar_zenith_deg",
    "view_zenith_deg",
    "relative_azimuth_deg",
];
const SOILSPECT: &[&str] = &[
    "single_scattering_albedo",
    "roughness_h",
    "phase_b",
    "phase_c",
    "phase_b_prime",
    "phase_c_prime",
    "solar_zenith_deg",
    "view_zenith_deg",
    "relative_azimuth_deg",
];
const FVCB: &[&str] = &[
    "leaf_temperature_c",
    "co2_surface_umol_mol",
    "intercellular_co2_umol_mol",
    "pressure_pa",
    "oxygen_mmol_mol",
    "vcmax25_umol_m2_s",
    "jmax25_umol_m2_s",
    "respiration25_umol_m2_s",
    "electron_quantum_yield",
    "light_response_curvature",
    "tpu_umol_m2_s",
    "vapour_pressure_deficit_kpa",
    "minimum_stomatal_conductance_water_mol_m2_s",
    "ball_berry_slope",
    "leuning_slope_co2",
    "leuning_d0_kpa",
    "medlyn_slope_kpa_sqrt",
    "medlyn_vpd_floor_kpa",
    "vcmax_activation_j_mol",
    "jmax_activation_j_mol",
    "respiration_activation_j_mol",
    "vcmax_deactivation_j_mol",
    "jmax_deactivation_j_mol",
    "vcmax_entropy_j_mol_k",
    "jmax_entropy_j_mol_k",
    "absorbed_par_umol_m2_s",
];
const FVCB_BIG_LEAF: &[&str] = &[
    "leaf_temperature_c",
    "co2_surface_umol_mol",
    "intercellular_co2_umol_mol",
    "pressure_pa",
    "oxygen_mmol_mol",
    "vcmax25_umol_m2_s",
    "jmax25_umol_m2_s",
    "respiration25_umol_m2_s",
    "electron_quantum_yield",
    "light_response_curvature",
    "tpu_umol_m2_s",
    "vapour_pressure_deficit_kpa",
    "minimum_stomatal_conductance_water_mol_m2_s",
    "ball_berry_slope",
    "leuning_slope_co2",
    "leuning_d0_kpa",
    "medlyn_slope_kpa_sqrt",
    "medlyn_vpd_floor_kpa",
    "vcmax_activation_j_mol",
    "jmax_activation_j_mol",
    "respiration_activation_j_mol",
    "vcmax_deactivation_j_mol",
    "jmax_deactivation_j_mol",
    "vcmax_entropy_j_mol_k",
    "jmax_entropy_j_mol_k",
    "absorbed_par_ground_umol_m2_s",
    "leaf_area_index",
    "nitrogen_extinction_coefficient",
];
const FOURSAIL: &[&str] = &[
    "wavelength_nm",
    "leaf_area_index",
    "mean_leaf_angle_deg",
    "lidf_a",
    "lidf_b",
    "hotspot",
    "solar_zenith_deg",
    "view_zenith_deg",
    "relative_azimuth_deg",
    "leaf_emissivity",
    "soil_emissivity",
    "sunlit_leaf_temperature_k",
    "shaded_leaf_temperature_k",
    "sunlit_soil_temperature_k",
    "shaded_soil_temperature_k",
    "sky_brightness_temperature_k",
    "direct_irradiance_w_m2_um",
];
const GOMS: &[&str] = &[
    "normalized_crown_density",
    "crown_axis_ratio",
    "crown_height_ratio",
    "relative_height_variation",
    "sunlit_ground_reflectance",
    "sunlit_crown_reflectance",
    "shaded_reflectance",
    "solar_zenith_deg",
    "solar_azimuth_deg",
    "view_zenith_deg",
    "view_azimuth_deg",
    "slope_deg",
    "slope_aspect_deg",
];
const FLUSPECT: &[&str] = &[
    "n",
    "chlorophyll",
    "carotenoids",
    "anthocyanins",
    "brown_pigments",
    "water",
    "dry_matter",
    "xanthophyll_state",
    "fluorescence_efficiency",
    "protein",
    "carbon_based_constituents",
];
const I2EM: &[&str] = &[
    "frequency_ghz",
    "permittivity_real",
    "permittivity_loss",
    "rms_height_m",
    "correlation_length_m",
    "observation_zenith_deg",
    "correlation_exponent",
];
const SORT: &[&str] = &[
    "solar_zenith_deg",
    "view_zenith_deg",
    "relative_azimuth_deg",
    "phase_asymmetry",
    "phase_backscatter",
    "hotspot_amplitude",
    "hotspot_width",
    "solid_filling_factor",
    "rms_slope",
    "multifacet_reflectance",
    "soil_brightness",
    "spectral_alpha_deg",
    "spectral_gamma_deg",
    "water_coverage",
    "water_film_thickness_mm",
    "particle_volume_fraction",
    "particle_refractive_index",
    "particle_extinction_coefficient",
];
const SORT_L: &[&str] = &[
    "soil_brightness",
    "spectral_alpha_deg",
    "spectral_gamma_deg",
    "water_coverage",
    "water_film_thickness_mm",
    "particle_volume_fraction",
    "particle_refractive_index",
    "particle_extinction_coefficient",
];
const PROSPECT: &[&str] = &[
    "n",
    "chlorophyll",
    "carotenoids",
    "anthocyanins",
    "brown_pigments",
    "water",
    "dry_matter",
    "incidence_angle_deg",
];
const PROSAIL: &[&str] = &[
    "n",
    "chlorophyll",
    "carotenoids",
    "anthocyanins",
    "brown_pigments",
    "water",
    "dry_matter",
    "incidence_angle_deg",
    "leaf_area_index",
    "mean_leaf_angle_deg",
    "hotspot",
    "solar_zenith_deg",
    "view_zenith_deg",
    "relative_azimuth_deg",
    "soil_brightness",
    "soil_dry_fraction",
    "lidf_a",
    "lidf_b",
];
const ROSS: &[&str] = &[
    "f_iso",
    "f_vol",
    "f_geo",
    "solar_zenith_deg",
    "view_zenith_deg",
    "relative_azimuth_deg",
];
const TAU: &[&str] = &[
    "soil_temperature_k",
    "vegetation_temperature_k",
    "soil_reflectivity",
    "vegetation_optical_depth",
    "single_scattering_albedo",
    "observation_zenith_deg",
];
const WATER: &[&str] = &[
    "vegetation_water_content",
    "scattering_coefficient",
    "attenuation_coefficient",
    "soil_backscatter",
    "observation_zenith_deg",
    "scattering_descriptor",
    "attenuation_descriptor",
];
const SMAC: &[&str] = &[
    "reflectance",
    "solar_zenith_deg",
    "view_zenith_deg",
    "relative_azimuth_deg",
    "pressure_hpa",
    "aot550",
    "ozone_atm_cm",
    "water_vapour_g_cm2",
];
const LMEB: &[&str] = &[
    "soil_moisture",
    "nadir_optical_depth",
    "sky_air_temperature_k",
    "canopy_temperature_k",
    "surface_temperature_k",
    "deep_temperature_k",
    "bulk_density_g_cm3",
    "sand_fraction",
    "clay_fraction",
    "temperature_moisture_scale",
    "temperature_moisture_exponent",
    "canopy_structure_h",
    "canopy_structure_v_ratio",
    "scattering_albedo_h",
    "scattering_albedo_v_difference",
    "roughness_height",
    "roughness_mixing",
    "roughness_exponent_v",
    "roughness_exponent_h",
    "observation_zenith_deg",
    "frequency_ghz",
    "water_temperature_k",
    "water_salinity",
];
const GORT: &[&str] = &[
    "leaf_area_index",
    "crown_radius_m",
    "crown_density_m2",
    "crown_base_height_m",
    "crown_top_height_m",
    "crown_axis_ratio",
    "solar_zenith_deg",
    "direct_par_umol_m2_s",
    "diffuse_par_umol_m2_s",
    "air_temperature_c",
    "vapour_pressure_deficit_pa",
    "co2_ppm",
    "young_leaf_fraction",
    "vcmax25_young",
    "vcmax25_mature",
    "pressure_pa",
    "stomatal_slope",
    "electron_photon_ratio",
    "leaf_angle_parameter",
    "leaf_single_scattering_albedo",
    "background_reflectance",
    "minimum_stomatal_conductance",
];
fn columns(id: &str) -> Result<&'static [&'static str], ModelError> {
    match id {
        "soil.gsv" => Ok(GSV),
        "soil.bsm" => Ok(BSM),
        "surface.rpv" => Ok(RPV),
        "soil.soilspect" => Ok(SOILSPECT),
        "ecosystem.fvcb" => Ok(FVCB),
        "ecosystem.fvcb-big-leaf" => Ok(FVCB_BIG_LEAF),
        "thermal.foursail" => Ok(FOURSAIL),
        "vegetation.goms" => Ok(GOMS),
        "vegetation.fluspect-cx" => Ok(FLUSPECT),
        "microwave.i2em" => Ok(I2EM),
        "soil.sort" => Ok(SORT),
        "soil.sort-l" => Ok(SORT_L),
        "ecosystem.gort-gpp" => Ok(GORT),
        "microwave.lmeb" => Ok(LMEB),
        "vegetation.prospect-d" => Ok(PROSPECT),
        "vegetation.prosail-d" => Ok(PROSAIL),
        "surface.ross-li" => Ok(ROSS),
        "microwave.tau-omega" => Ok(TAU),
        "microwave.water-cloud" => Ok(WATER),
        "atmosphere.smac" => Ok(SMAC),
        _ => Err(ModelError::new(
            ErrorKind::UnknownModel,
            format!("no NumPy batch adapter for {id}"),
        )),
    }
}
#[pyfunction]
pub fn array_parameters(id: &str) -> PyResult<Vec<String>> {
    Ok(columns(id)
        .map_err(super::error)?
        .iter()
        .map(|s| (*s).to_owned())
        .collect())
}
#[derive(Default)]
struct Row {
    scalars: Vec<(&'static str, f64)>,
    integers: Vec<(&'static str, i64)>,
    arrays: Vec<(&'static str, Vec<f64>)>,
    matrices: Vec<(&'static str, Vec<Vec<f64>>)>,
}
struct Numeric {
    tail: Vec<usize>,
    values: Vec<f64>,
}
struct Buffers {
    numbers: BTreeMap<&'static str, Numeric>,
    integers: BTreeMap<&'static str, Vec<i64>>,
    axes: BTreeMap<&'static str, Vec<f64>>,
}
fn shared(key: &str, enabled: bool) -> bool {
    key == "wavelengths_nm"
        || enabled
            && matches!(
                key,
                "principal_view_zenith_deg"
                    | "hemisphere_view_zenith_deg"
                    | "hemisphere_azimuth_deg"
                    | "sweep_zenith_deg"
                    | "sweep_aot550"
                    | "gap_zenith_deg"
                    | "excitation_wavelengths_nm"
                    | "emission_wavelengths_nm"
                    | "ci_axis_umol_mol"
                    | "light_axis_umol_m2_s"
            )
}
impl Buffers {
    fn new(template: Row, enabled: bool) -> Self {
        let mut b = Self {
            numbers: BTreeMap::new(),
            integers: BTreeMap::new(),
            axes: BTreeMap::new(),
        };
        for (k, _) in template.scalars {
            b.numbers.insert(
                k,
                Numeric {
                    tail: vec![],
                    values: vec![],
                },
            );
        }
        for (k, _) in template.integers {
            b.integers.insert(k, vec![]);
        }
        for (k, v) in template.arrays {
            if shared(k, enabled) {
                b.axes.insert(k, v);
            } else {
                b.numbers.insert(
                    k,
                    Numeric {
                        tail: vec![v.len()],
                        values: vec![],
                    },
                );
            }
        }
        for (k, v) in template.matrices {
            b.numbers.insert(
                k,
                Numeric {
                    tail: vec![v.len(), v.first().map_or(0, Vec::len)],
                    values: vec![],
                },
            );
        }
        b
    }
    fn push(&mut self, row: Row) -> Result<(), ModelError> {
        for (k, v) in row.scalars {
            self.numbers
                .get_mut(k)
                .expect("typed scalar field")
                .values
                .push(v);
        }
        for (k, v) in row.integers {
            self.integers
                .get_mut(k)
                .expect("typed integer field")
                .push(v);
        }
        for (k, v) in row.arrays {
            if let Some(axis) = self.axes.get(k) {
                if axis != &v {
                    return Err(ModelError::new(
                        ErrorKind::Internal,
                        "model coordinate varies across rows; shared layout is invalid",
                    ));
                }
            } else {
                let b = self.numbers.get_mut(k).expect("typed array field");
                if b.tail != [v.len()] {
                    return Err(ModelError::new(
                        ErrorKind::Internal,
                        "variable model output shape in NumPy batch",
                    ));
                }
                b.values.extend(v);
            }
        }
        for (k, v) in row.matrices {
            let b = self.numbers.get_mut(k).expect("typed matrix field");
            if b.tail.len() != 2 || v.len() != b.tail[0] || v.iter().any(|r| r.len() != b.tail[1]) {
                return Err(ModelError::new(
                    ErrorKind::Internal,
                    "variable matrix shape in NumPy batch",
                ));
            }
            b.values.extend(v.into_iter().flatten());
        }
        Ok(())
    }
}
macro_rules! cache {
    ($func:ident,$static:ident,$model:ty) => {
        static $static: OnceLock<Result<$model, ModelError>> = OnceLock::new();
        fn $func() -> PyResult<&'static $model> {
            $static
                .get_or_init(<$model>::new)
                .as_ref()
                .map_err(|e| super::error(e.clone()))
        }
    };
}
cache!(gsv, GSV_MODEL, rsphysics_gsv::Model);
cache!(bsm, BSM_MODEL, rsphysics_bsm::Model);
cache!(rpv, RPV_MODEL, rsphysics_rpv::Model);
cache!(soilspect, SOILSPECT_MODEL, rsphysics_soilspect::Model);
cache!(fvcb, FVCB_MODEL, rsphysics_fvcb::Model);
cache!(
    fvcb_big_leaf,
    FVCB_BIG_LEAF_MODEL,
    rsphysics_fvcb::BigLeafModel
);
cache!(foursail, FOURSAIL_MODEL, rsphysics_foursail_tir::Model);
cache!(goms, GOMS_MODEL, rsphysics_goms::Model);
cache!(fluspect, FLUSPECT_MODEL, rsphysics_fluspect_cx::Model);
cache!(i2em, I2EM_MODEL, rsphysics_i2em::Model);
cache!(sort, SORT_MODEL, rsphysics_sort::Model);
cache!(sort_l, SORT_L_MODEL, rsphysics_sort::LambertModel);
cache!(prosail, PROSAIL_MODEL, rsphysics_prosail_d::Model);
cache!(ross, ROSS_MODEL, rsphysics_ross_li::Model);
cache!(tau, TAU_MODEL, rsphysics_tau_omega::Model);
cache!(water, WATER_MODEL, rsphysics_water_cloud::Model);
cache!(smac, SMAC_MODEL, rsphysics_smac::Model);

cache!(lmeb, LMEB_MODEL, rsphysics_lmeb::Model);

cache!(gort, GORT_MODEL, rsphysics_gort_gpp::Model);

fn typed<M: PhysicsModel>(
    model: &M,
    id: &str,
    values: &[f64],
    enabled: bool,
    convert: fn(M::Output) -> Row,
    fixed: &Map<String, Value>,
) -> Result<(Buffers, Vec<RunManifest>), ModelError>
where
    M::Input: Default,
{
    let names = columns(id)?;
    let runtime =
        super::runtime().map_err(|e| ModelError::new(ErrorKind::Internal, e.to_string()))?;
    let mut defaults = serde_json::to_value(M::Input::default())
        .map_err(|e| ModelError::new(ErrorKind::Internal, e.to_string()))?;
    defaults
        .as_object_mut()
        .expect("model object")
        .extend(fixed.clone());
    let default: M::Input = serde_json::from_value(defaults)
        .map_err(|e| ModelError::new(ErrorKind::InvalidParameter, e.to_string()))?;
    model.validate_input(&default)?;
    let template = model.run(&default)?;
    model.validate_output(&template)?;
    let mut buffers = Buffers::new(convert(template), enabled);
    let mut manifests = vec![];
    for (i, row) in values.chunks_exact(names.len()).enumerate() {
        let run = || {
            let mut params = fixed.clone();
            for (k, v) in names.iter().zip(row) {
                let n = serde_json::Number::from_f64(*v).ok_or_else(|| {
                    ModelError::new(
                        ErrorKind::InvalidParameter,
                        "batch parameters must be finite",
                    )
                })?;
                let value = Value::Number(n);
                let value = if id == "vegetation.goms"
                    && matches!(
                        *k,
                        "sunlit_ground_reflectance"
                            | "sunlit_crown_reflectance"
                            | "shaded_reflectance"
                    ) {
                    Value::Array(vec![value])
                } else {
                    value
                };
                params.insert((*k).to_owned(), value);
            }
            let input: M::Input = serde_json::from_value(Value::Object(params))
                .map_err(|e| ModelError::new(ErrorKind::InvalidParameter, e.to_string()))?;
            model.validate_input(&input)?;
            let output = model.run(&input)?;
            model.validate_output(&output)?;
            let manifest = runtime.manifest_for(
                id,
                &serde_json::to_value(input)
                    .map_err(|e| ModelError::new(ErrorKind::Internal, e.to_string()))?,
            )?;
            Ok::<_, ModelError>((convert(output), manifest))
        };
        let (output, manifest) =
            run().map_err(|e| ModelError::new(e.kind, format!("batch row {i}: {}", e.message)))?;
        buffers.push(output)?;
        manifests.push(manifest);
    }
    Ok((buffers, manifests))
}
fn gsv_row(o: rsphysics_gsv::Output) -> Row {
    Row {
        arrays: vec![
            ("wavelengths_nm", o.wavelengths_nm),
            ("dry_reflectance", o.dry_reflectance),
            ("moisture_contribution", o.moisture_contribution),
            ("reflectance", o.reflectance),
        ],
        integers: vec![(
            "outside_unit_interval_count",
            o.outside_unit_interval_count as i64,
        )],
        ..Row::default()
    }
}
fn bsm_row(o: rsphysics_bsm::Output) -> Row {
    Row {
        arrays: vec![
            ("wavelengths_nm", o.wavelengths_nm),
            ("dry_reflectance", o.dry_reflectance),
            ("wet_reflectance", o.wet_reflectance),
        ],
        scalars: vec![
            ("mean_film_count", o.mean_film_count),
            ("dry_area_fraction", o.dry_area_fraction),
            ("poisson_probability_sum", o.poisson_probability_sum),
            ("omitted_probability", o.omitted_probability),
        ],
        ..Row::default()
    }
}
fn rpv_row(o: rsphysics_rpv::Output) -> Row {
    Row {
        scalars: vec![
            ("brf", o.point.brf),
            ("brdf", o.point.brdf),
            ("minnaert_factor", o.point.minnaert_factor),
            ("phase_function", o.point.phase_function),
            ("hotspot_factor", o.point.hotspot_factor),
        ],
        integers: vec![("negative_sample_count", o.negative_sample_count as i64)],
        arrays: vec![
            ("principal_view_zenith_deg", o.principal_view_zenith_deg),
            ("principal_brf", o.principal_brf),
            ("hemisphere_view_zenith_deg", o.hemisphere_view_zenith_deg),
            ("hemisphere_azimuth_deg", o.hemisphere_azimuth_deg),
            ("hemisphere_brf", o.hemisphere_brf),
        ],
        ..Row::default()
    }
}
fn soilspect_row(o: rsphysics_soilspect::Output) -> Row {
    Row {
        scalars: vec![
            ("brf", o.point.brf),
            ("brdf", o.point.brdf),
            ("single_scattering_brf", o.point.single_scattering_brf),
            ("multiple_scattering_brf", o.point.multiple_scattering_brf),
            ("phase_function", o.point.phase_function),
            ("hotspot_factor", o.point.hotspot_factor),
            (
                "directional_hemispherical_reflectance",
                o.directional_hemispherical_reflectance,
            ),
            ("bihemispherical_reflectance", o.bihemispherical_reflectance),
            ("integration_error_estimate", o.integration_error_estimate),
        ],
        integers: vec![("negative_sample_count", o.negative_sample_count as i64)],
        arrays: vec![
            ("principal_view_zenith_deg", o.principal_view_zenith_deg),
            ("principal_brf", o.principal_brf),
            ("hemisphere_view_zenith_deg", o.hemisphere_view_zenith_deg),
            ("hemisphere_azimuth_deg", o.hemisphere_azimuth_deg),
            ("hemisphere_brf", o.hemisphere_brf),
        ],
        ..Row::default()
    }
}
fn fvcb_row(o: rsphysics_fvcb::Output) -> Row {
    let p = o.point;
    Row {
        scalars: vec![
            ("net_assimilation_umol_m2_s", p.net_assimilation_umol_m2_s),
            (
                "assimilation_before_day_respiration_umol_m2_s",
                p.assimilation_before_day_respiration_umol_m2_s,
            ),
            (
                "rubisco_limited_assimilation_umol_m2_s",
                p.rubisco_limited_assimilation_umol_m2_s,
            ),
            (
                "electron_limited_assimilation_umol_m2_s",
                p.electron_limited_assimilation_umol_m2_s,
            ),
            (
                "tpu_limited_assimilation_umol_m2_s",
                p.tpu_limited_assimilation_umol_m2_s,
            ),
            ("day_respiration_umol_m2_s", p.day_respiration_umol_m2_s),
            ("vcmax_umol_m2_s", p.vcmax_umol_m2_s),
            ("jmax_umol_m2_s", p.jmax_umol_m2_s),
            (
                "electron_transport_umol_m2_s",
                p.electron_transport_umol_m2_s,
            ),
            ("kc_umol_mol", p.kc_umol_mol),
            ("ko_mmol_mol", p.ko_mmol_mol),
            ("gamma_star_umol_mol", p.gamma_star_umol_mol),
            (
                "michaelis_menten_co2_umol_mol",
                p.michaelis_menten_co2_umol_mol,
            ),
            (
                "intercellular_co2_umol_mol",
                p.intercellular_co2_umol_mol.unwrap_or(f64::NAN),
            ),
            (
                "stomatal_conductance_water_mol_m2_s",
                p.stomatal_conductance_water_mol_m2_s.unwrap_or(f64::NAN),
            ),
            (
                "stomatal_conductance_co2_mol_m2_s",
                p.stomatal_conductance_co2_mol_m2_s.unwrap_or(f64::NAN),
            ),
            (
                "transpiration_mmol_m2_s",
                p.transpiration_mmol_m2_s.unwrap_or(f64::NAN),
            ),
            (
                "co2_flux_residual_umol_m2_s",
                p.co2_flux_residual_umol_m2_s.unwrap_or(f64::NAN),
            ),
            ("capacity_area_factor", o.capacity_area_factor),
            ("leaf_area_factor", o.leaf_area_factor),
        ],
        integers: vec![
            ("gas_exchange_computed", i64::from(p.gas_exchange_computed)),
            (
                "intercellular_co2_defined",
                i64::from(p.intercellular_co2_defined),
            ),
            ("limitation_code", i64::from(p.limitation_code)),
            ("solver_iterations", i64::from(p.solver_iterations)),
            (
                "tpu_limitation_enabled",
                i64::from(p.tpu_limitation_enabled),
            ),
        ],
        arrays: vec![
            ("ci_axis_umol_mol", o.ci_axis_umol_mol),
            ("ci_net_assimilation", o.ci_net_assimilation),
            (
                "ci_rubisco_limited_assimilation",
                o.ci_rubisco_limited_assimilation,
            ),
            (
                "ci_electron_limited_assimilation",
                o.ci_electron_limited_assimilation,
            ),
            ("light_axis_umol_m2_s", o.light_axis_umol_m2_s),
            ("light_net_assimilation", o.light_net_assimilation),
        ],
        ..Row::default()
    }
}
fn foursail_row(o: rsphysics_foursail_tir::Output) -> Row {
    Row {
        scalars: vec![
            ("radiance_w_m2_sr_um", o.point.radiance_w_m2_sr_um),
            ("brightness_temperature_k", o.point.brightness_temperature_k),
            ("effective_emissivity", o.point.effective_emissivity),
            ("leaf_emission_weight", o.point.leaf_emission_weight),
            ("soil_emission_weight", o.point.soil_emission_weight),
            ("sunlit_leaf_weight", o.point.sunlit_leaf_weight),
            ("shaded_leaf_weight", o.point.shaded_leaf_weight),
            ("sunlit_soil_weight", o.point.sunlit_soil_weight),
            ("shaded_soil_weight", o.point.shaded_soil_weight),
            ("sky_reflectance", o.point.sky_reflectance),
            (
                "direct_reflectance_factor",
                o.point.direct_reflectance_factor,
            ),
            (
                "sunlit_leaf_radiance_w_m2_sr_um",
                o.point.sunlit_leaf_radiance_w_m2_sr_um,
            ),
            (
                "shaded_leaf_radiance_w_m2_sr_um",
                o.point.shaded_leaf_radiance_w_m2_sr_um,
            ),
            (
                "sunlit_soil_radiance_w_m2_sr_um",
                o.point.sunlit_soil_radiance_w_m2_sr_um,
            ),
            (
                "shaded_soil_radiance_w_m2_sr_um",
                o.point.shaded_soil_radiance_w_m2_sr_um,
            ),
            (
                "reflected_sky_radiance_w_m2_sr_um",
                o.point.reflected_sky_radiance_w_m2_sr_um,
            ),
            (
                "reflected_direct_radiance_w_m2_sr_um",
                o.point.reflected_direct_radiance_w_m2_sr_um,
            ),
            ("hemispherical_emissivity", o.point.hemispherical_emissivity),
            ("upwelling_flux_w_m2_um", o.point.upwelling_flux_w_m2_um),
            (
                "ground_downwelling_irradiance_w_m2_um",
                o.point.ground_downwelling_irradiance_w_m2_um,
            ),
            (
                "ground_upwelling_flux_w_m2_um",
                o.point.ground_upwelling_flux_w_m2_um,
            ),
            (
                "hotspot_brightness_temperature_k",
                o.hotspot_brightness_temperature_k,
            ),
        ],
        arrays: vec![
            ("wavelengths_nm", o.wavelengths_nm),
            (
                "spectral_radiance_w_m2_sr_um",
                o.spectral_radiance_w_m2_sr_um,
            ),
            (
                "spectral_brightness_temperature_k",
                o.spectral_brightness_temperature_k,
            ),
            ("principal_view_zenith_deg", o.principal_view_zenith_deg),
            (
                "principal_brightness_temperature_k",
                o.principal_brightness_temperature_k,
            ),
            (
                "principal_effective_emissivity",
                o.principal_effective_emissivity,
            ),
            ("hemisphere_view_zenith_deg", o.hemisphere_view_zenith_deg),
            ("hemisphere_azimuth_deg", o.hemisphere_azimuth_deg),
            (
                "hemisphere_brightness_temperature_k",
                o.hemisphere_brightness_temperature_k,
            ),
            (
                "hemisphere_effective_emissivity",
                o.hemisphere_effective_emissivity,
            ),
        ],
        ..Row::default()
    }
}
fn sort_row(o: rsphysics_sort::Output) -> Row {
    Row {
        arrays: vec![
            ("wavelengths_nm", o.wavelengths_nm.wavelengths_nm().to_vec()),
            ("dry_brf", o.dry_brf),
            ("wet_brf", o.wet_brf),
            ("dry_basis_reflectance", o.dry_basis_reflectance),
            ("single_scattering_albedo", o.single_scattering_albedo),
            ("substrate_brf", o.substrate_brf),
        ],
        ..Row::default()
    }
}
fn sort_l_row(o: rsphysics_sort::LambertOutput) -> Row {
    Row {
        arrays: vec![
            ("wavelengths_nm", o.wavelengths_nm.wavelengths_nm().to_vec()),
            ("dry_reflectance", o.dry_reflectance),
            ("wet_reflectance", o.wet_reflectance),
        ],
        ..Row::default()
    }
}
fn prospect_row(o: rsphysics_prospect_d::Output) -> Row {
    Row {
        arrays: vec![
            ("wavelengths_nm", o.wavelengths_nm.wavelengths_nm().to_vec()),
            ("reflectance", o.reflectance),
            ("transmittance", o.transmittance),
        ],
        ..Row::default()
    }
}
fn prosail_row(o: rsphysics_prosail_d::Output) -> Row {
    Row {
        arrays: vec![
            ("wavelengths_nm", o.wavelengths_nm.wavelengths_nm().to_vec()),
            ("brf", o.brf),
            ("hemispherical_reflectance", o.hemispherical_reflectance),
            (
                "hemispherical_directional_reflectance",
                o.hemispherical_directional_reflectance,
            ),
            ("bihemispherical_reflectance", o.bihemispherical_reflectance),
            ("vegetation_cover", o.vegetation_cover),
            ("direct_absorptance", o.direct_absorptance),
            ("diffuse_absorptance", o.diffuse_absorptance),
            ("direct_albedo", o.direct_albedo),
            ("diffuse_albedo", o.diffuse_albedo),
        ],
        ..Row::default()
    }
}
fn ross_row(o: rsphysics_ross_li::Output) -> Row {
    Row {
        scalars: vec![
            ("brf", o.point.brf),
            ("brdf", o.point.brdf),
            ("k_iso", o.point.k_iso),
            ("k_vol", o.point.k_vol),
            ("k_geo", o.point.k_geo),
        ],
        integers: vec![("negative_sample_count", o.negative_sample_count as i64)],
        arrays: vec![
            ("principal_view_zenith_deg", o.principal_view_zenith_deg),
            ("principal_brf", o.principal_brf),
            ("hemisphere_view_zenith_deg", o.hemisphere_view_zenith_deg),
            ("hemisphere_azimuth_deg", o.hemisphere_azimuth_deg),
            ("hemisphere_brf", o.hemisphere_brf),
        ],
        ..Row::default()
    }
}
fn tau_row(o: rsphysics_tau_omega::Output) -> Row {
    Row {
        scalars: vec![
            ("brightness_temperature_k", o.point.brightness_temperature_k),
            ("soil_emission_k", o.point.soil_emission_k),
            ("vegetation_emission_k", o.point.vegetation_emission_k),
            (
                "reflected_vegetation_emission_k",
                o.point.reflected_vegetation_emission_k,
            ),
            ("transmissivity", o.point.transmissivity),
        ],
        arrays: vec![
            ("sweep_zenith_deg", o.sweep_zenith_deg),
            ("sweep_total", o.sweep_total),
        ],
        ..Row::default()
    }
}
fn goms_row(o: rsphysics_goms::Output) -> Row {
    Row {
        scalars: vec![
            ("hotspot_view_zenith_deg", o.hotspot_view_zenith_deg),
            ("hotspot_view_azimuth_deg", o.hotspot_view_azimuth_deg),
            ("hotspot_shaded_weight", o.hotspot_shaded_weight),
            ("sunlit_ground_weight", o.point.sunlit_ground_weight),
            ("sunlit_crown_weight", o.point.sunlit_crown_weight),
            ("shaded_weight", o.point.shaded_weight),
        ],
        integers: vec![("invalid_weight_count", o.point.invalid_weight_count as i64)],
        arrays: vec![
            ("hotspot_brf", o.hotspot_brf),
            ("principal_shaded_weight", o.principal_shaded_weight),
            ("brf", o.point.brf),
            ("brdf", o.point.brdf),
            ("band_index", o.band_index),
            ("principal_view_zenith_deg", o.principal_view_zenith_deg),
            ("principal_valid", o.principal_valid),
            ("hemisphere_view_zenith_deg", o.hemisphere_view_zenith_deg),
            ("hemisphere_azimuth_deg", o.hemisphere_azimuth_deg),
            ("hemisphere_valid", o.hemisphere_valid),
        ],
        matrices: vec![
            ("principal_brf", o.principal_brf),
            ("hemisphere_brf", o.hemisphere_brf),
        ],
    }
}
fn fluspect_row(o: rsphysics_fluspect_cx::Output) -> Row {
    Row {
        arrays: vec![
            ("wavelengths_nm", o.wavelengths_nm),
            ("reflectance", o.reflectance),
            ("transmittance", o.transmittance),
            (
                "chlorophyll_absorption_fraction",
                o.chlorophyll_absorption_fraction,
            ),
            (
                "carotenoid_absorption_fraction",
                o.carotenoid_absorption_fraction,
            ),
            ("excitation_wavelengths_nm", o.excitation_wavelengths_nm),
            ("emission_wavelengths_nm", o.emission_wavelengths_nm),
        ],
        matrices: vec![
            ("fluorescence_backward", o.fluorescence_backward),
            ("fluorescence_forward", o.fluorescence_forward),
        ],
        ..Row::default()
    }
}
fn i2em_row(o: rsphysics_i2em::Output) -> Row {
    Row {
        scalars: vec![
            ("sigma0_vv", o.sigma0_vv),
            ("sigma0_hh", o.sigma0_hh),
            ("sigma0_vv_db", o.sigma0_vv_db),
            ("sigma0_hh_db", o.sigma0_hh_db),
            ("ks", o.ks),
            ("height_length_ratio", o.height_length_ratio),
        ],
        ..Row::default()
    }
}
fn water_row(o: rsphysics_water_cloud::Output) -> Row {
    Row {
        scalars: vec![
            ("sigma0", o.point.sigma0),
            ("vegetation_backscatter", o.point.vegetation_backscatter),
            (
                "attenuated_soil_backscatter",
                o.point.attenuated_soil_backscatter,
            ),
            ("two_way_transmissivity", o.point.two_way_transmissivity),
        ],
        arrays: vec![
            ("sweep_zenith_deg", o.sweep_zenith_deg),
            ("sweep_total", o.sweep_total),
        ],
        ..Row::default()
    }
}

fn smac_row(o: rsphysics_smac::Output) -> Row {
    let p = o.point;
    Row {
        scalars: vec![
            ("reflectance", p.reflectance),
            ("gas_transmittance", p.gas_transmittance),
            (
                "downward_scattering_transmittance",
                p.downward_scattering_transmittance,
            ),
            (
                "upward_scattering_transmittance",
                p.upward_scattering_transmittance,
            ),
            ("spherical_albedo", p.spherical_albedo),
            ("atmospheric_reflectance", p.atmospheric_reflectance),
            ("rayleigh_reflectance", p.rayleigh_reflectance),
            ("aerosol_reflectance", p.aerosol_reflectance),
        ],
        integers: vec![("outside_unit_interval", i64::from(p.outside_unit_interval))],
        arrays: vec![
            ("sweep_aot550", o.sweep_aot550),
            ("sweep_reflectance", o.sweep_reflectance),
        ],
        ..Row::default()
    }
}

fn lmeb_row(o: rsphysics_lmeb::Output) -> Row {
    Row {
        scalars: vec![
            ("brightness_temperature_v_k", o.brightness_temperature_v_k),
            ("brightness_temperature_h_k", o.brightness_temperature_h_k),
            (
                "effective_soil_temperature_k",
                o.effective_soil_temperature_k,
            ),
            ("soil_permittivity_real", o.soil_permittivity_real),
            ("soil_permittivity_imaginary", o.soil_permittivity_imaginary),
            ("soil_reflectivity_v", o.soil_reflectivity_v),
            ("soil_reflectivity_h", o.soil_reflectivity_h),
            (
                "sky_brightness_temperature_k",
                o.sky_brightness_temperature_k,
            ),
            ("transmissivity_v", o.transmissivity_v),
            ("transmissivity_h", o.transmissivity_h),
        ],
        arrays: vec![
            ("sweep_zenith_deg", o.sweep_zenith_deg),
            (
                "sweep_brightness_temperature_v_k",
                o.sweep_brightness_temperature_v_k,
            ),
            (
                "sweep_brightness_temperature_h_k",
                o.sweep_brightness_temperature_h_k,
            ),
        ],
        ..Row::default()
    }
}

fn gort_row(o: rsphysics_gort_gpp::Output) -> Row {
    Row {
        scalars: vec![
            ("net_assimilation_umol_m2_s", o.net_assimilation_umol_m2_s),
            (
                "sunlit_assimilation_umol_m2_s",
                o.sunlit_assimilation_umol_m2_s,
            ),
            (
                "shaded_assimilation_umol_m2_s",
                o.shaded_assimilation_umol_m2_s,
            ),
            ("sunlit_leaf_area_index", o.sunlit_leaf_area_index),
            ("shaded_leaf_area_index", o.shaded_leaf_area_index),
            ("sunlit_absorbed_par", o.sunlit_absorbed_par),
            ("shaded_absorbed_par", o.shaded_absorbed_par),
            ("between_crown_openness", o.between_crown_openness),
            ("within_crown_openness", o.within_crown_openness),
            ("floor_absorbed_par", o.floor_absorbed_par),
        ],
        arrays: vec![
            ("gap_zenith_deg", o.gap_zenith_deg),
            ("between_crown_gaps", o.between_crown_gaps),
            ("within_crown_gaps", o.within_crown_gaps),
        ],
        ..Row::default()
    }
}

#[pyfunction]
pub fn model_batch_native<'py>(
    py: Python<'py>,
    id: &str,
    parameters: PyReadonlyArray2<'py, f64>,
    shared_axes: bool,
    fixed_parameters: &str,
) -> PyResult<Bound<'py, PyDict>> {
    let names = columns(id).map_err(super::error)?;
    let fixed: Map<String, Value> = serde_json::from_str(fixed_parameters)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    if fixed.keys().any(|k| !match id {
        "soil.gsv" => matches!(k.as_str(), "variant" | "wavelengths_nm"),
        "soil.bsm" => k == "poisson_mode",
        "surface.rpv" => k == "hotspot_mode",
        "soil.soilspect" => k == "quadrature",
        "vegetation.prosail-d" | "thermal.foursail" => k == "leaf_angle_distribution",
        "vegetation.goms" => k == "geometry_convention",
        "atmosphere.smac" => matches!(k.as_str(), "coefficient_id" | "direction"),
        "ecosystem.fvcb" | "ecosystem.fvcb-big-leaf" => matches!(
            k.as_str(),
            "gas_exchange_mode"
                | "stomatal_model"
                | "temperature_response"
                | "tpu_limitation"
                | "leuning_compensation"
        ),
        "ecosystem.gort-gpp" => k == "numerical_convention",
        "microwave.water-cloud" => k == "descriptor_mode",
        "microwave.lmeb" => k == "surface_type",
        "soil.sort" => matches!(
            k.as_str(),
            "roughness_correction" | "substrate_angle_mode" | "water_film_mode"
        ),
        "microwave.i2em" => k == "correlation_function",
        "vegetation.fluspect-cx" => {
            matches!(k.as_str(), "spectral_resolution" | "optical_coefficients")
        }
        _ => false,
    }) {
        return Err(super::error(ModelError::new(
            ErrorKind::InvalidParameter,
            "fixed accepts only model-specific selectors and supported shared spectral requests",
        )));
    }
    if parameters.shape()[1] != names.len() {
        return Err(super::error(ModelError::new(
            ErrorKind::InvalidParameter,
            format!(
                "parameters must have shape (batch, {}); use parameters_for()",
                names.len()
            ),
        )));
    }
    let count = parameters.shape()[0];
    let values: Vec<_> = parameters.as_array().iter().copied().collect();
    // Initialize cached models/runtime with the GIL, then release it for all rows.
    super::runtime()?;
    let (buffers, manifests) = match id {
        "soil.gsv" => {
            let m = gsv()?;
            py.detach(|| typed(m, id, &values, shared_axes, gsv_row, &fixed))
        }
        "soil.bsm" => {
            let m = bsm()?;
            py.detach(|| typed(m, id, &values, shared_axes, bsm_row, &fixed))
        }
        "surface.rpv" => {
            let m = rpv()?;
            py.detach(|| typed(m, id, &values, shared_axes, rpv_row, &fixed))
        }
        "soil.soilspect" => {
            let m = soilspect()?;
            py.detach(|| typed(m, id, &values, shared_axes, soilspect_row, &fixed))
        }
        "ecosystem.fvcb" => {
            let m = fvcb()?;
            py.detach(|| typed(m, id, &values, shared_axes, fvcb_row, &fixed))
        }
        "ecosystem.fvcb-big-leaf" => {
            let m = fvcb_big_leaf()?;
            py.detach(|| typed(m, id, &values, shared_axes, fvcb_row, &fixed))
        }
        "thermal.foursail" => {
            let m = foursail()?;
            py.detach(|| typed(m, id, &values, shared_axes, foursail_row, &fixed))
        }
        "vegetation.goms" => {
            let m = goms()?;
            py.detach(|| typed(m, id, &values, shared_axes, goms_row, &fixed))
        }
        "vegetation.fluspect-cx" => {
            let m = fluspect()?;
            py.detach(|| typed(m, id, &values, shared_axes, fluspect_row, &fixed))
        }
        "microwave.i2em" => {
            let m = i2em()?;
            py.detach(|| typed(m, id, &values, shared_axes, i2em_row, &fixed))
        }
        "soil.sort" => {
            let m = sort()?;
            py.detach(|| typed(m, id, &values, shared_axes, sort_row, &fixed))
        }
        "soil.sort-l" => {
            let m = sort_l()?;
            py.detach(|| typed(m, id, &values, shared_axes, sort_l_row, &fixed))
        }
        "ecosystem.gort-gpp" => {
            let m = gort()?;
            py.detach(|| typed(m, id, &values, shared_axes, gort_row, &fixed))
        }
        "microwave.lmeb" => {
            let m = lmeb()?;
            py.detach(|| typed(m, id, &values, shared_axes, lmeb_row, &fixed))
        }
        "vegetation.prospect-d" => {
            let m = super::model()?;
            py.detach(|| typed(m, id, &values, shared_axes, prospect_row, &fixed))
        }
        "vegetation.prosail-d" => {
            let m = prosail()?;
            py.detach(|| typed(m, id, &values, shared_axes, prosail_row, &fixed))
        }
        "surface.ross-li" => {
            let m = ross()?;
            py.detach(|| typed(m, id, &values, shared_axes, ross_row, &fixed))
        }
        "microwave.tau-omega" => {
            let m = tau()?;
            py.detach(|| typed(m, id, &values, shared_axes, tau_row, &fixed))
        }
        "microwave.water-cloud" => {
            let m = water()?;
            py.detach(|| typed(m, id, &values, shared_axes, water_row, &fixed))
        }
        "atmosphere.smac" => {
            let m = smac()?;
            py.detach(|| typed(m, id, &values, shared_axes, smac_row, &fixed))
        }
        _ => unreachable!("validated model ID"),
    }
    .map_err(super::error)?;
    pack(py, count, buffers, manifests, shared_axes)
}
fn pack<'py>(
    py: Python<'py>,
    count: usize,
    buffers: Buffers,
    manifests: Vec<RunManifest>,
    shared_axes: bool,
) -> PyResult<Bound<'py, PyDict>> {
    let result = PyDict::new(py);
    for (k, b) in buffers.numbers {
        let mut shape = vec![count];
        shape.extend(b.tail);
        result.set_item(
            k,
            ArrayD::from_shape_vec(IxDyn(&shape), b.values)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
                .into_pyarray(py),
        )?;
    }
    for (k, v) in buffers.integers {
        result.set_item(k, v.into_pyarray(py))?;
    }
    let coordinates = PyDict::new(py);
    for (k, v) in buffers.axes {
        if shared_axes {
            coordinates.set_item(k, v.into_pyarray(py))?;
        } else {
            result.set_item(k, v.into_pyarray(py))?;
        }
    }
    let envelope = PyDict::new(py);
    envelope.set_item("result", result)?;
    if shared_axes {
        envelope.set_item("coordinates", coordinates)?;
    }
    envelope.set_item(
        "manifests_json",
        serde_json::to_string(&manifests)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?,
    )?;
    Ok(envelope)
}

#[pyfunction]
pub fn goms_multiband_native<'py>(
    py: Python<'py>,
    geometry: PyReadonlyArray2<'py, f64>,
    reflectances: PyReadonlyArray3<'py, f64>,
    convention: &str,
    shared_axes: bool,
) -> PyResult<Bound<'py, PyDict>> {
    let shape = reflectances.shape();
    let count = geometry.shape()[0];
    if geometry.shape()[1] != 10
        || shape[0] != count
        || !(1..=10).contains(&shape[1])
        || shape[2] != 3
    {
        return Err(super::error(ModelError::new(
            ErrorKind::InvalidParameter,
            "geometry must be (batch,10) and reflectances (batch,band,3), with 1..10 bands",
        )));
    }
    let nb = shape[1];
    let geo: Vec<f64> = geometry.as_array().iter().copied().collect();
    let optics: Vec<f64> = reflectances.as_array().iter().copied().collect();
    let convention: rsphysics_goms::GeometryConvention =
        serde_json::from_value(Value::String(convention.into())).map_err(|e| {
            super::error(ModelError::new(ErrorKind::InvalidParameter, e.to_string()))
        })?;
    let model = goms()?;
    let runtime = super::runtime()?;
    let (buffers, manifests) = py
        .detach(|| {
            let template = rsphysics_goms::Input {
                sunlit_ground_reflectance: vec![0.2; nb],
                sunlit_crown_reflectance: vec![0.55; nb],
                shaded_reflectance: vec![0.05; nb],
                geometry_convention: convention,
                ..Default::default()
            };
            let mut buffers = Buffers::new(goms_row(model.simulate(&template)?), shared_axes);
            let mut manifests = Vec::with_capacity(count);
            for i in 0..count {
                let run = || {
                    let g = &geo[i * 10..(i + 1) * 10];
                    let a = &optics[i * nb * 3..(i + 1) * nb * 3];
                    let p = rsphysics_goms::Input {
                        normalized_crown_density: g[0],
                        crown_axis_ratio: g[1],
                        crown_height_ratio: g[2],
                        relative_height_variation: g[3],
                        solar_zenith_deg: g[4],
                        solar_azimuth_deg: g[5],
                        view_zenith_deg: g[6],
                        view_azimuth_deg: g[7],
                        slope_deg: g[8],
                        slope_aspect_deg: g[9],
                        sunlit_ground_reflectance: a.chunks_exact(3).map(|v| v[0]).collect(),
                        sunlit_crown_reflectance: a.chunks_exact(3).map(|v| v[1]).collect(),
                        shaded_reflectance: a.chunks_exact(3).map(|v| v[2]).collect(),
                        geometry_convention: convention,
                    };
                    let out = model.simulate(&p)?;
                    let manifest = runtime.manifest_for(
                        "vegetation.goms",
                        &serde_json::to_value(p)
                            .map_err(|e| ModelError::new(ErrorKind::Internal, e.to_string()))?,
                    )?;
                    Ok::<_, ModelError>((goms_row(out), manifest))
                };
                let (row, manifest) = run().map_err(|e| {
                    ModelError::new(e.kind, format!("batch row {i}: {}", e.message))
                })?;
                buffers.push(row)?;
                manifests.push(manifest);
            }
            Ok::<_, ModelError>((buffers, manifests))
        })
        .map_err(super::error)?;
    pack(py, count, buffers, manifests, shared_axes)
}
