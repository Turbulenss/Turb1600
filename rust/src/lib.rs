// =========================================================
//   turb1600 — Python bindings (pyo3)
// =========================================================

use pyo3::prelude::*;

mod core;

/// Compute the turb1600 hash of the given message.
///
/// Args:
///     data (bytes): Input message
///
/// Returns:
///     bytes: 1024-bit (128-byte) hash output
#[pyfunction]
fn hash(data: &[u8]) -> PyResult<Vec<u8>> {
    Ok(core::turb1600_hash(data))
}

/// Python module definition
#[pymodule]
fn turb1600(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hash, m)?)?;
    Ok(())
}
