use pyo3::prelude::*;
use pyo3::types::PyBytes;

mod core;

#[pyfunction]
fn hash(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyBytes>> {
    let digest = core::turb1600_hash(data);
    Ok(PyBytes::new(py, &digest).into())
}

#[pymodule]
fn turb1600(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hash, m)?)?;
    Ok(())
}
