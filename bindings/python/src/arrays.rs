//! Typed NumPy batch collector. Only selectors and manifests use JSON.
use numpy::{
    IntoPyArray, PyReadonlyArray2, PyUntypedArrayMethods,
    ndarray::{ArrayD, IxDyn},
};
use pyo3::{prelude::*, types::PyDict};
use rsphysics_model_api::{ErrorKind, ModelError, PhysicsModel};
use rsphysics_runtime::RunManifest;
use serde_json::{Map, Value};
use std::{collections::BTreeMap, sync::OnceLock};

const I2EM: &[&str] = &[
    "frequency_ghz",
    "permittivity_real",
    "permittivity_loss",
    "rms_height_m",
    "correlation_length_m",
    "observation_zenith_deg",
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
                params.insert((*k).to_owned(), Value::Number(n));
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
fn sort_row(o: rsphysics_sort::Output) -> Row {
    Row {
        arrays: vec![
            ("wavelengths_nm", o.wavelengths_nm.wavelengths_nm().to_vec()),
            ("dry_brf", o.dry_brf),
            ("wet_brf", o.wet_brf),
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
        "atmosphere.smac" => matches!(k.as_str(), "coefficient_id" | "direction"),
        "soil.sort" => k == "roughness_correction",
        "microwave.i2em" => k == "correlation_function",
        _ => false,
    }) {
        return Err(super::error(ModelError::new(
            ErrorKind::InvalidParameter,
            "fixed accepts only model string selectors: SMAC coefficient_id/direction, SORT roughness_correction, or I2EM correlation_function",
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
