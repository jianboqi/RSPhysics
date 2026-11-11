use numpy::{Element, IntoPyArray, PyReadonlyArray2, PyUntypedArrayMethods, ndarray::Array2};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyDict};
use rsphysics_core::DomainError;
use rsphysics_model_api::{ErrorKind, ModelError};
use rsphysics_numerics::lut::MatchTable;

#[pyclass(frozen, module = "rsphysics._native")]
pub struct LutMatcher(MatchTable<Vec<f64>>);

fn owned<T: Element + Copy>(array: &PyReadonlyArray2<'_, T>) -> Vec<T> {
    if array.is_c_contiguous()
        && let Ok(values) = array.as_slice()
    {
        return values.to_vec();
    }
    array.as_array().iter().copied().collect()
}

fn invalid(error: DomainError) -> PyErr {
    super::error(ModelError::new(
        ErrorKind::InvalidParameter,
        error.to_string(),
    ))
}

#[pymethods]
impl LutMatcher {
    #[new]
    fn new(py: Python<'_>, table: PyReadonlyArray2<'_, f64>) -> PyResult<Self> {
        let channels = table.shape()[1];
        let values = owned(&table);
        py.detach(|| MatchTable::new(values, channels))
            .map(Self)
            .map_err(invalid)
    }

    fn search<'py>(
        &self,
        py: Python<'py>,
        observations: PyReadonlyArray2<'py, f64>,
        sigma: PyReadonlyArray2<'py, f64>,
        mask: PyReadonlyArray2<'py, bool>,
        k: usize,
        workers: usize,
    ) -> PyResult<Bound<'py, PyDict>> {
        let rows = observations.shape()[0];
        let channels = self.0.channels();
        let valid_shape = |shape: &[usize]| {
            shape == [0, 0] || shape == [1, channels] || shape == [rows, channels]
        };
        if observations.shape()[1] != channels
            || !valid_shape(sigma.shape())
            || !valid_shape(mask.shape())
        {
            return Err(PyValueError::new_err(
                "LUT feature matrix shapes do not match",
            ));
        }
        let observations = owned(&observations);
        let sigma = owned(&sigma);
        let mask = owned(&mask);
        let matches = py
            .detach(|| self.0.search(&observations, &sigma, &mask, k, workers))
            .map_err(invalid)?;
        let result = PyDict::new(py);
        let indices =
            Array2::from_shape_vec((rows, k), matches.indices).expect("validated output shape");
        let scores =
            Array2::from_shape_vec((rows, k), matches.scores).expect("validated output shape");
        result.set_item("indices", indices.into_pyarray(py))?;
        result.set_item("scores", scores.into_pyarray(py))?;
        Ok(result)
    }
}
