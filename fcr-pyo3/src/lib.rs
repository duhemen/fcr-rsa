use pyo3::prelude::*;
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use fcr_rsa::{encrypt_tier_a, decrypt_tier_a};

/// Fungsi Python: enkripsi_tier_a(message: int, seed: bytes, n: int, q: int, blinding: int) -> int
#[pyfunction]
fn py_encrypt_tier_a(message: u64, seed: &[u8], n: usize, q: u64, blinding: u64) -> PyResult<String> {
    let msg_big = BigInt::from_u64(message).unwrap();
    let q_big = BigInt::from_u64(q).unwrap();
    let blind_big = BigInt::from_u64(blinding).unwrap();

    match encrypt_tier_a(&msg_big, seed, n, &q_big, &blind_big) {
        Ok(c) => Ok(c.to_string()), // Kembalikan sebagai string karena Python int tidak terbatas
        Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e)),
    }
}

/// Fungsi Python: dekripsi_tier_a(ciphertext: int, seed: bytes, n: int, q: int, blinding: int) -> int
#[pyfunction]
fn py_decrypt_tier_a(ciphertext: &str, seed: &[u8], n: usize, q: u64, blinding: u64) -> PyResult<String> {
    let ct_big = ciphertext.parse::<BigInt>().unwrap();
    let q_big = BigInt::from_u64(q).unwrap();
    let blind_big = BigInt::from_u64(blinding).unwrap();

    match decrypt_tier_a(&ct_big, seed, n, &q_big, &blind_big) {
        Ok(m) => Ok(m.to_string()),
        Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e)),
    }
}

#[pymodule]
fn fcr_pyo3(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_encrypt_tier_a, m)?)?;
    m.add_function(wrap_pyfunction!(py_decrypt_tier_a, m)?)?;
    Ok(())
}