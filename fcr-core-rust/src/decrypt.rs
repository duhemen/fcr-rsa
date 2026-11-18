//! Decryption module for FCR-RSA v0.6.0

use crypto_bigint::U2048;
use crate::fib::{
    u2048_to_bytes, decrypt_u2048, FcrKeyPair,
};

/// Dekripsi ciphertext (U2048) → plaintext (bytes)
pub fn decrypt_to_bytes(
    ciphertext: &U2048,
    key_pair: &FcrKeyPair,
    master_seed: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let message = decrypt_u2048(ciphertext, key_pair, master_seed, nonce)?;
    let bytes = u2048_to_bytes(&message);
    let first_non_zero = bytes.iter().position(|&x| x != 0).unwrap_or(bytes.len());
    Ok(bytes[first_non_zero..].to_vec())
}

/// API ini menyembunyikan U2048 sepenuhnya dari caller.
pub fn decrypt_from_bytes(
    ciphertext_bytes: &[u8],
    key_pair: &FcrKeyPair,
    master_seed: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, &'static str> {
    if ciphertext_bytes.len() != 256 {
        return Err("Ciphertext must be exactly 256 bytes");
    }
    let ct = crate::fib::bytes_to_u2048(ciphertext_bytes)?;
    decrypt_to_bytes(&ct, key_pair, master_seed, nonce)
}