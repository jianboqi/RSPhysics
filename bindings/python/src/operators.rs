//! NumPy buffers and shape handling only; scientific calculations are public Rust.
use numpy::{
    IntoPyArray, PyReadonlyArray1, PyReadonlyArrayDyn, PyUntypedArrayMethods,
    ndarray::{ArrayD, IxDyn},
};
use pyo3::{prelude::*, types::PyDict};
use rsphysics_model_api::{ErrorKind, ModelError};
use rsphysics_numerics::{radiometry, spectral};
use serde_json::Value;
fn err(e: impl std::fmt::Display) -> PyErr {
    super::error(ModelError::new(ErrorKind::OutOfDomain, e.to_string()))
}
fn array<'py>(
    py: Python<'py>,
    shape: &[usize],
    values: Vec<f64>,
) -> PyResult<Bound<'py, numpy::PyArrayDyn<f64>>> {
    Ok(ArrayD::from_shape_vec(IxDyn(shape), values)
        .map_err(err)?
        .into_pyarray(py))
}
fn input(values: PyReadonlyArrayDyn<'_, f64>, n: usize) -> PyResult<(Vec<usize>, Vec<f64>)> {
    let shape = values.shape().to_vec();
    if shape.is_empty() || n == 0 || *shape.last().expect("nonempty") != n {
        return Err(err(
            "last dimension must match the nonempty wavelength axis",
        ));
    }
    Ok((shape, values.as_array().iter().copied().collect()))
}
#[pyfunction]
pub fn spectral_resample<'py>(
    py: Python<'py>,
    x: PyReadonlyArray1<'py, f64>,
    values: PyReadonlyArrayDyn<'py, f64>,
    target: PyReadonlyArray1<'py, f64>,
) -> PyResult<Bound<'py, numpy::PyArrayDyn<f64>>> {
    let x = x.as_array().to_vec();
    let target = target.as_array().to_vec();
    let (mut shape, values) = input(values, x.len())?;
    let data = py
        .detach(|| {
            spectral::resample(&x, &vec![0.; x.len()], &target)?;
            values
                .chunks_exact(x.len())
                .map(|v| spectral::resample(&x, v, &target))
                .collect::<Result<Vec<_>, _>>()
                .map(|v| v.into_iter().flatten().collect())
        })
        .map_err(err)?;
    *shape.last_mut().expect("nonempty") = target.len();
    array(py, &shape, data)
}
#[pyfunction]
pub fn spectral_integrate<'py>(
    py: Python<'py>,
    x: PyReadonlyArray1<'py, f64>,
    values: PyReadonlyArrayDyn<'py, f64>,
    lo: f64,
    hi: f64,
) -> PyResult<Bound<'py, numpy::PyArrayDyn<f64>>> {
    let x = x.as_array().to_vec();
    let (mut shape, values) = input(values, x.len())?;
    shape.pop();
    let data = py
        .detach(|| {
            spectral::integrate(&x, &vec![0.; x.len()], lo, hi)?;
            values
                .chunks_exact(x.len())
                .map(|v| spectral::integrate(&x, v, lo, hi))
                .collect::<Result<Vec<_>, _>>()
        })
        .map_err(err)?;
    array(py, &shape, data)
}
#[pyfunction]
#[allow(clippy::too_many_arguments)] // NumPy binding boundary; not a scientific parameter object.
pub fn spectral_convolve<'py>(
    py: Python<'py>,
    x: PyReadonlyArray1<'py, f64>,
    values: PyReadonlyArrayDyn<'py, f64>,
    rx: PyReadonlyArray1<'py, f64>,
    response: PyReadonlyArray1<'py, f64>,
    ix: PyReadonlyArray1<'py, f64>,
    irradiance: PyReadonlyArray1<'py, f64>,
    allow_partial: bool,
) -> PyResult<Bound<'py, PyDict>> {
    let x = x.as_array().to_vec();
    let rx = rx.as_array().to_vec();
    let response = response.as_array().to_vec();
    let ix = ix.as_array().to_vec();
    let irradiance = irradiance.as_array().to_vec();
    let (mut shape, values) = input(values, x.len())?;
    shape.pop();
    let (plan, data) = py
        .detach(|| {
            let illumination = if ix.is_empty() && irradiance.is_empty() {
                None
            } else {
                Some((ix.as_slice(), irradiance.as_slice()))
            };
            let plan =
                spectral::BandpassPlan::new(&x, &rx, &response, illumination, allow_partial)?;
            let data = values
                .chunks_exact(x.len())
                .map(|v| plan.apply(v))
                .collect::<Result<Vec<_>, _>>()?;
            Ok::<_, rsphysics_core::DomainError>((plan, data))
        })
        .map_err(err)?;
    let result = PyDict::new(py);
    result.set_item("values", array(py, &shape, data)?)?;
    result.set_item("coverage", plan.coverage)?;
    result.set_item("effective_wavelength_nm", plan.effective_wavelength_nm)?;
    result.set_item("equivalent_width_nm", plan.equivalent_width_nm)?;
    Ok(result)
}
#[pyfunction]
pub fn spectral_convolve_many<'py>(
    py: Python<'py>,
    x: PyReadonlyArray1<'py, f64>,
    values: PyReadonlyArrayDyn<'py, f64>,
    curves: Vec<(Vec<f64>, Vec<f64>)>,
    ix: PyReadonlyArray1<'py, f64>,
    irradiance: PyReadonlyArray1<'py, f64>,
    allow_partial: bool,
) -> PyResult<Bound<'py, PyDict>> {
    let x = x.as_array().to_vec();
    let ix = ix.as_array().to_vec();
    let irradiance = irradiance.as_array().to_vec();
    let (mut shape, values) = input(values, x.len())?;
    shape.pop();
    if curves.is_empty() {
        return Err(err("at least one response band is required"));
    }
    let (plans, data) = py
        .detach(|| {
            let illumination = if ix.is_empty() && irradiance.is_empty() {
                None
            } else {
                Some((ix.as_slice(), irradiance.as_slice()))
            };
            let plans = curves
                .iter()
                .enumerate()
                .map(|(i, (rx, r))| {
                    spectral::BandpassPlan::new(&x, rx, r, illumination, allow_partial)
                        .map_err(|e| format!("band {i}: {e}"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut data = Vec::with_capacity(values.len() / x.len() * plans.len());
            for row in values.chunks_exact(x.len()) {
                for plan in &plans {
                    data.push(plan.apply(row).map_err(|e| e.to_string())?);
                }
            }
            Ok::<_, String>((plans, data))
        })
        .map_err(err)?;
    shape.push(plans.len());
    let result = PyDict::new(py);
    result.set_item("values", array(py, &shape, data)?)?;
    for (key, data) in [
        (
            "coverage",
            plans.iter().map(|p| p.coverage).collect::<Vec<_>>(),
        ),
        (
            "effective_wavelength_nm",
            plans.iter().map(|p| p.effective_wavelength_nm).collect(),
        ),
        (
            "equivalent_width_nm",
            plans.iter().map(|p| p.equivalent_width_nm).collect(),
        ),
    ] {
        result.set_item(key, data.into_pyarray(py))?;
    }
    Ok(result)
}
#[pyfunction]
pub fn radiometry_array<'py>(
    py: Python<'py>,
    mode: &str,
    values: PyReadonlyArrayDyn<'py, f64>,
    mask: PyReadonlyArrayDyn<'py, bool>,
    config: &str,
) -> PyResult<Bound<'py, PyDict>> {
    let shape = values.shape().to_vec();
    if mask.shape() != shape {
        return Err(err("valid_mask must exactly match input shape"));
    }
    let input: Vec<_> = values.as_array().iter().copied().collect();
    let valid: Vec<_> = mask.as_array().iter().copied().collect();
    let config: Value = serde_json::from_str(config).map_err(err)?;
    let number = |key: &str| {
        config[key]
            .as_f64()
            .ok_or_else(|| err(format!("missing finite calibration constant {key}")))
    };
    let (gain, offset) = (number("gain")?, number("offset")?);
    let (theta, esun, distance, k1, k2) = (
        number("solar_zenith_deg")?,
        number("solar_irradiance")?,
        number("distance_au")?,
        number("k1")?,
        number("k2")?,
    );
    let per_nm = config["radiance_per_nm"].as_bool().unwrap_or(false);
    let nodata = config["nodata"].as_f64();
    let saturation = config["saturation"].as_f64();
    // Validate constants even for empty/all-masked inputs.
    match mode {
        "linear" => {}
        "sun" => {
            radiometry::correct_reflectance_sun_angle(1., theta).map_err(err)?;
        }
        "toa" => {
            radiometry::radiance_to_toa_reflectance(1., esun, theta, distance).map_err(err)?;
        }
        "bt" => {
            radiometry::brightness_temperature(1., k1, k2, per_nm).map_err(err)?;
        }
        _ => return Err(err("unknown radiometric operation")),
    }
    let data = py
        .detach(|| {
            let mut out =
                radiometry::linear_calibrate(&input, gain, offset, &valid, nodata, saturation)?;
            for (v, flag) in out.values.iter_mut().zip(&mut out.flags) {
                if *flag != 0 {
                    continue;
                }
                let next = match mode {
                    "linear" => Ok(*v),
                    "sun" => radiometry::correct_reflectance_sun_angle(*v, theta),
                    "toa" => radiometry::radiance_to_toa_reflectance(*v, esun, theta, distance),
                    "bt" => radiometry::brightness_temperature(*v, k1, k2, per_nm),
                    _ => unreachable!(),
                };
                match next {
                    Ok(n) => *v = n,
                    Err(_) => {
                        *v = f64::NAN;
                        *flag |= radiometry::DOMAIN;
                    }
                }
            }
            Ok::<_, rsphysics_core::DomainError>(out)
        })
        .map_err(err)?;
    let result = PyDict::new(py);
    result.set_item("values", array(py, &shape, data.values)?)?;
    let valid: Vec<_> = data.flags.iter().map(|v| *v == 0).collect();
    result.set_item(
        "valid",
        ArrayD::from_shape_vec(IxDyn(&shape), valid)
            .map_err(err)?
            .into_pyarray(py),
    )?;
    result.set_item(
        "flags",
        ArrayD::from_shape_vec(IxDyn(&shape), data.flags)
            .map_err(err)?
            .into_pyarray(py),
    )?;
    Ok(result)
}
