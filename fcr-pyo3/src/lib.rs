//! FCR-RSA v0.6.0 - Python Bindings (PyO3)
//! API murni bytes-to-bytes, tanpa ekspos U2048 ke Python.

// ✅ Suppress warning non_local_definitions dari PyO3 v0.20 macro
// Ini adalah solusi resmi yang direkomendasikan PyO3 team.
// Warning ini 100% kosmetik dan tidak mempengaruhi keamanan/performa.
#![allow(non_local_definitions)]

use pyo3::prelude::*;
use fcr_core_rust::{
    generate_key_pair, encrypt_to_bytes, decrypt_from_bytes, FcrKeyPair,
};

/// Python wrapper untuk FcrKeyPair
#[pyclass]
pub struct PyFcrKeyPair {
    inner: FcrKeyPair,
}

#[pymethods]
impl PyFcrKeyPair {
    #[new]
    fn new(master_seed: &[u8]) -> PyResult<Self> {
        let inner = generate_key_pair(master_seed)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))?;
        Ok(PyFcrKeyPair { inner })
    }
}

/// Enkripsi pesan dari Python
/// Args: key_pair, message_bytes, master_seed, nonce
/// Returns: 256-byte ciphertext
#[pyfunction]
fn encrypt_message(
    key_pair: &PyFcrKeyPair,
    message_bytes: &[u8],
    master_seed: &[u8],
    nonce: &[u8],
) -> PyResult<Vec<u8>> {
    let ct = encrypt_to_bytes(message_bytes, &key_pair.inner, master_seed, nonce)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))?;
    Ok(ct.to_vec())
}

/// Dekripsi pesan ke Python
/// Args: key_pair, ciphertext_bytes (256), master_seed, nonce
/// Returns: plaintext bytes
#[pyfunction]
fn decrypt_message(
    key_pair: &PyFcrKeyPair,
    ciphertext_bytes: &[u8],
    master_seed: &[u8],
    nonce: &[u8],
) -> PyResult<Vec<u8>> {
    decrypt_from_bytes(ciphertext_bytes, &key_pair.inner, master_seed, nonce)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
}

/// Python Module Definition
#[pymodule]
fn fcr_pyo3(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyFcrKeyPair>()?;
    m.add_function(wrap_pyfunction!(encrypt_message, m)?)?;
    m.add_function(wrap_pyfunction!(decrypt_message, m)?)?;
    Ok(())
}