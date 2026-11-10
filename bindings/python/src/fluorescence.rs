//! NumPy leaf-fluorescence operator; all numerical work stays in the Rust model.
use numpy::{IntoPyArray, PyReadonlyArray1};
use pyo3::{prelude::*, types::PyDict};

#[pyfunction]
pub fn fluspect_irradiance_native<'py>(
    py: Python<'py>,
    parameters: &str,
    incident: PyReadonlyArray1<'py, f64>,
) -> PyResult<Bound<'py, PyDict>> {
    let p: rsphysics_fluspect_cx::Input = serde_json::from_str(parameters).map_err(|e| {
        super::error(rsphysics_model_api::ModelError::new(
            rsphysics_model_api::ErrorKind::InvalidParameter,
            e.to_string(),
        ))
    })?;
    let incident: Vec<f64> = incident.as_array().iter().copied().collect();
    let result = py
        .detach(|| {
            let model = rsphysics_fluspect_cx::Model::new()?;
            let output = model.simulate(&p)?;
            rsphysics_fluspect_cx::fluorescence_from_irradiance(&output, &incident)
        })
        .map_err(super::error)?;
    let out = PyDict::new(py);
    out.set_item(
        "emission_wavelengths_nm",
        result.emission_wavelengths_nm.into_pyarray(py),
    )?;
    out.set_item("backward_w_m2_nm", result.backward_w_m2_nm.into_pyarray(py))?;
    out.set_item("forward_w_m2_nm", result.forward_w_m2_nm.into_pyarray(py))?;
    Ok(out)
}
