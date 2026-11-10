//! Thin Python wrappers; scientific equations remain in the model crate.
use numpy::{IntoPyArray, PyArray1};
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
    types::PyDict,
};
use rsphysics_catalog::official_catalog;
use rsphysics_model_api::{Backend, ErrorKind, ModelError};
use rsphysics_prospect_d::{Input, Model};
use rsphysics_runtime::{BuildInfo, Runtime};
use serde_json::Value;
use std::sync::OnceLock;

const ID: &str = "vegetation.prospect-d";
mod arrays;
mod fluorescence;
mod operators;
mod physics;
static RUNTIME: OnceLock<Result<Runtime, ModelError>> = OnceLock::new();
static PROSPECT: OnceLock<Result<Model, ModelError>> = OnceLock::new();

fn error(error: ModelError) -> PyErr {
    let message = serde_json::to_string(&error).expect("serializable error");
    match error.kind {
        ErrorKind::InvalidParameter | ErrorKind::OutOfDomain | ErrorKind::UnknownModel => {
            PyValueError::new_err(message)
        }
        _ => PyRuntimeError::new_err(message),
    }
}

fn runtime() -> PyResult<&'static Runtime> {
    RUNTIME
        .get_or_init(|| {
            Ok(Runtime::new(
                official_catalog()?,
                BuildInfo::default(),
                Backend::Python,
            ))
        })
        .as_ref()
        .map_err(|e| error(e.clone()))
}

fn model() -> PyResult<&'static Model> {
    PROSPECT
        .get_or_init(Model::new)
        .as_ref()
        .map_err(|e| error(e.clone()))
}

#[pyfunction]
fn list_models_json() -> PyResult<String> {
    serde_json::to_string(&runtime()?.list_models())
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn describe_json(id: &str) -> PyResult<String> {
    serde_json::to_string(runtime()?.describe(id).map_err(error)?)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn smac_coefficients_json() -> PyResult<String> {
    serde_json::to_string(rsphysics_smac::coefficients::library().map_err(error)?)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn run_json(py: Python<'_>, id: &str, parameters: &str) -> PyResult<String> {
    let parameters: Value =
        serde_json::from_str(parameters).map_err(|e| PyValueError::new_err(e.to_string()))?;
    let runtime = runtime()?;
    let result = py.detach(|| runtime.run(id, &parameters)).map_err(error)?;
    serde_json::to_string(&result).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn run_batch_json(py: Python<'_>, id: &str, rows: &str) -> PyResult<String> {
    let rows: Vec<Value> =
        serde_json::from_str(rows).map_err(|e| PyValueError::new_err(e.to_string()))?;
    let runtime = runtime()?;
    let result = py.detach(|| runtime.run_batch(id, &rows)).map_err(error)?;
    serde_json::to_string(&result).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn prospect_d_native<'py>(py: Python<'py>, parameters: &str) -> PyResult<Bound<'py, PyDict>> {
    let input: Input =
        serde_json::from_str(parameters).map_err(|e| PyValueError::new_err(e.to_string()))?;
    let model = model()?;
    let output = py.detach(|| model.simulate(&input)).map_err(error)?;
    let manifest = runtime()?
        .manifest_for(
            ID,
            &serde_json::to_value(input).expect("finite validated input"),
        )
        .map_err(error)?;
    let result = PyDict::new(py);
    result.set_item(
        "wavelengths_nm",
        PyArray1::from_vec(py, output.wavelengths_nm.wavelengths_nm().to_vec()),
    )?;
    result.set_item("reflectance", output.reflectance.into_pyarray(py))?;
    result.set_item("transmittance", output.transmittance.into_pyarray(py))?;
    result.set_item(
        "manifest_json",
        serde_json::to_string(&manifest).map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
    )?;
    Ok(result)
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add(
        "PLANCK_CONSTANT",
        rsphysics_numerics::thermal::PLANCK_CONSTANT,
    )?;
    module.add(
        "SPEED_OF_LIGHT",
        rsphysics_numerics::thermal::SPEED_OF_LIGHT,
    )?;
    module.add(
        "BOLTZMANN_CONSTANT",
        rsphysics_numerics::thermal::BOLTZMANN_CONSTANT,
    )?;
    module.add_function(wrap_pyfunction!(list_models_json, module)?)?;
    module.add_function(wrap_pyfunction!(describe_json, module)?)?;
    module.add_function(wrap_pyfunction!(smac_coefficients_json, module)?)?;
    module.add_function(wrap_pyfunction!(run_json, module)?)?;
    module.add_function(wrap_pyfunction!(run_batch_json, module)?)?;
    module.add_function(wrap_pyfunction!(prospect_d_native, module)?)?;
    module.add_function(wrap_pyfunction!(arrays::model_batch_native, module)?)?;
    module.add_function(wrap_pyfunction!(arrays::goms_multiband_native, module)?)?;
    module.add_function(wrap_pyfunction!(arrays::array_parameters, module)?)?;
    module.add_function(wrap_pyfunction!(
        fluorescence::fluspect_irradiance_native,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(operators::spectral_resample, module)?)?;
    module.add_function(wrap_pyfunction!(operators::spectral_integrate, module)?)?;
    module.add_function(wrap_pyfunction!(operators::spectral_convolve, module)?)?;
    module.add_function(wrap_pyfunction!(operators::spectral_convolve_many, module)?)?;
    module.add_function(wrap_pyfunction!(operators::radiometry_array, module)?)?;
    module.add_function(wrap_pyfunction!(physics::planck_array, module)?)?;
    module.add_function(wrap_pyfunction!(physics::fresnel_array, module)?)?;
    Ok(())
}
