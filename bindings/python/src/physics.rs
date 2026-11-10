//! Broadcasted NumPy buffers; all optical/thermal equations live in public Rust.
use num_complex::Complex64;
use numpy::{
    IntoPyArray, PyReadonlyArrayDyn, PyUntypedArrayMethods,
    ndarray::{ArrayD, IxDyn},
};
use pyo3::{prelude::*, types::PyDict};
use rsphysics_model_api::{ErrorKind, ModelError};
use rsphysics_numerics::{optics, thermal};
fn err(e: impl std::fmt::Display) -> PyErr {
    super::error(ModelError::new(ErrorKind::OutOfDomain, e.to_string()))
}
fn density(unit: &str) -> PyResult<thermal::WavelengthDensity> {
    match unit {
        "W m^-2 sr^-1 m^-1" => Ok(thermal::WavelengthDensity::PerMetre),
        "W m^-2 sr^-1 nm^-1" => Ok(thermal::WavelengthDensity::PerNanometre),
        "W m^-2 sr^-1 um^-1" => Ok(thermal::WavelengthDensity::PerMicrometre),
        _ => Err(err(
            "wavelength radiance unit must explicitly be per m, nm or um",
        )),
    }
}
#[pyfunction]
pub fn planck_array<'py>(
    py: Python<'py>,
    mode: &str,
    a: PyReadonlyArrayDyn<'py, f64>,
    b: PyReadonlyArrayDyn<'py, f64>,
    unit: &str,
) -> PyResult<Bound<'py, numpy::PyArrayDyn<f64>>> {
    if a.shape() != b.shape() {
        return Err(err(
            "Planck binding arrays must have the same broadcast shape",
        ));
    }
    let shape = a.shape().to_vec();
    let av: Vec<_> = a.as_array().iter().copied().collect();
    let bv: Vec<_> = b.as_array().iter().copied().collect();
    let unit = if mode == "frequency" || mode == "inverse_frequency" {
        thermal::WavelengthDensity::PerMetre
    } else {
        density(unit)?
    };
    if !matches!(
        mode,
        "wavelength" | "inverse_wavelength" | "frequency" | "inverse_frequency"
    ) {
        return Err(err("unknown Planck operation"));
    }
    let result = py
        .detach(|| {
            av.iter()
                .zip(&bv)
                .enumerate()
                .map(|(i, (&a, &b))| {
                    let value = match mode {
                        "wavelength" => thermal::planck_radiance(a, b, unit),
                        "inverse_wavelength" => thermal::inverse_planck(a, b, unit),
                        "frequency" => thermal::planck_frequency_radiance(a, b),
                        "inverse_frequency" => thermal::inverse_planck_frequency(a, b),
                        _ => unreachable!(),
                    };
                    value.map_err(|e| format!("flat element {i}: {e}"))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .map_err(err)?;
    Ok(ArrayD::from_shape_vec(IxDyn(&shape), result)
        .map_err(err)?
        .into_pyarray(py))
}
#[pyfunction]
pub fn fresnel_array<'py>(
    py: Python<'py>,
    angle: PyReadonlyArrayDyn<'py, f64>,
    incident: PyReadonlyArrayDyn<'py, f64>,
    transmitted: PyReadonlyArrayDyn<'py, Complex64>,
    permittivity: bool,
) -> PyResult<Bound<'py, PyDict>> {
    let shape = angle.shape().to_vec();
    if incident.shape() != shape || transmitted.shape() != shape {
        return Err(err(
            "Fresnel binding arrays must have the same broadcast shape",
        ));
    }
    let angles: Vec<_> = angle.as_array().iter().copied().collect();
    let ni: Vec<_> = incident.as_array().iter().copied().collect();
    let nt: Vec<_> = transmitted.as_array().iter().copied().collect();
    let rows = py
        .detach(|| {
            angles
                .iter()
                .zip(&ni)
                .zip(&nt)
                .enumerate()
                .map(|(i, ((&theta, &ni), &nt))| {
                    (if permittivity {
                        optics::fresnel_permittivity(theta, ni, nt)
                    } else {
                        optics::fresnel(theta, ni, nt)
                    })
                    .map_err(|e| format!("flat element {i}: {e}"))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .map_err(err)?;
    let result = PyDict::new(py);
    for (key, values) in [
        (
            "reflectance_s",
            rows.iter().map(|r| r.reflectance_s).collect::<Vec<_>>(),
        ),
        (
            "reflectance_p",
            rows.iter().map(|r| r.reflectance_p).collect(),
        ),
        (
            "reflectance_unpolarized",
            rows.iter().map(|r| r.reflectance_unpolarized).collect(),
        ),
        (
            "interface_transmittance_s",
            rows.iter().map(|r| r.interface_transmittance_s).collect(),
        ),
        (
            "interface_transmittance_p",
            rows.iter().map(|r| r.interface_transmittance_p).collect(),
        ),
        (
            "interface_transmittance_unpolarized",
            rows.iter()
                .map(|r| r.interface_transmittance_unpolarized)
                .collect(),
        ),
    ] {
        result.set_item(
            key,
            ArrayD::from_shape_vec(IxDyn(&shape), values)
                .map_err(err)?
                .into_pyarray(py),
        )?;
    }
    for (key, values) in [
        (
            "amplitude_r_s",
            rows.iter().map(|r| r.amplitude_r_s).collect::<Vec<_>>(),
        ),
        (
            "amplitude_r_p",
            rows.iter().map(|r| r.amplitude_r_p).collect(),
        ),
    ] {
        result.set_item(
            key,
            ArrayD::from_shape_vec(IxDyn(&shape), values)
                .map_err(err)?
                .into_pyarray(py),
        )?;
    }
    result.set_item(
        "total_internal_reflection",
        ArrayD::from_shape_vec(
            IxDyn(&shape),
            rows.iter().map(|r| r.total_internal_reflection).collect(),
        )
        .map_err(err)?
        .into_pyarray(py),
    )?;
    Ok(result)
}
