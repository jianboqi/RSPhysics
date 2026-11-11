use numpy::{IntoPyArray, PyReadonlyArray2, PyUntypedArrayMethods, ndarray::Array2};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyDict};
use rsphysics_model_api::{ErrorKind, ModelError};
use rsphysics_numerics::lut::match_rows;

#[pyfunction]
pub fn lut_match<'py>(
    py: Python<'py>,
    table: PyReadonlyArray2<'py, f64>,
    observations: PyReadonlyArray2<'py, f64>,
    sigma: PyReadonlyArray2<'py, f64>,
    mask: PyReadonlyArray2<'py, bool>,
    k: usize,
) -> PyResult<Bound<'py, PyDict>> {
    let rows = observations.shape()[0];
    let channels = observations.shape()[1];
    if table.shape()[1] != channels
        || mask.shape() != observations.shape()
        || (sigma.shape() != [0, 0] && sigma.shape() != observations.shape())
    {
        return Err(PyValueError::new_err(
            "LUT feature matrix shapes do not match",
        ));
    }
    let copy = |values: PyReadonlyArray2<'_, f64>| -> Vec<f64> {
        values.as_array().iter().copied().collect()
    };
    let table = copy(table);
    let observations = copy(observations);
    let sigma = copy(sigma);
    let mask: Vec<bool> = mask.as_array().iter().copied().collect();
    let matches = py
        .detach(|| match_rows(&table, &observations, channels, &sigma, &mask, k))
        .map_err(|error| {
            super::error(ModelError::new(
                ErrorKind::InvalidParameter,
                error.to_string(),
            ))
        })?;
    let result = PyDict::new(py);
    let indices =
        Array2::from_shape_vec((rows, k), matches.indices).expect("validated output shape");
    let scores = Array2::from_shape_vec((rows, k), matches.scores).expect("validated output shape");
    result.set_item("indices", indices.into_pyarray(py))?;
    result.set_item("scores", scores.into_pyarray(py))?;
    Ok(result)
}
