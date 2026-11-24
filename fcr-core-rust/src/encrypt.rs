//! Encryption module for FCR-RSA v0.6.0

use crate::fib::{
    bytes_to_u2048, derive_blinding, encrypt_with_blinding_ct, u2048_to_bytes, FcrKeyPair,
};
use crypto_bigint::U2048;

/// Enkripsi pesan (bytes) → ciphertext (U2048)
pub fn encrypt_bytes(
    message_bytes: &[u8],
    key_pair: &FcrKeyPair,
    master_seed: &[u8],
    nonce: &[u8],
) -> Result<U2048, &'static str> {
    let message = bytes_to_u2048(message_bytes)?;
    let blinding = derive_blinding(master_seed, nonce)?;
    Ok(encrypt_with_blinding_ct(
        &message,
        &key_pair.e_vec,
        &blinding,
    ))
}

/// API ini menyembunyikan U2048 sepenuhnya dari caller.
pub fn encrypt_to_bytes(
    message_bytes: &[u8],
    key_pair: &FcrKeyPair,
    master_seed: &[u8],
    nonce: &[u8],
) -> Result<[u8; 256], &'static str> {
    let ct = encrypt_bytes(message_bytes, key_pair, master_seed, nonce)?;
    Ok(u2048_to_bytes(&ct))
}

/// Enkripsi langsung menggunakan tipe U2048
pub fn encrypt_u2048(
    message: &U2048,
    key_pair: &FcrKeyPair,
    master_seed: &[u8],
    nonce: &[u8],
) -> Result<U2048, &'static str> {
    let blinding = derive_blinding(master_seed, nonce)?;
    Ok(encrypt_with_blinding_ct(
        message,
        &key_pair.e_vec,
        &blinding,
    ))
}
