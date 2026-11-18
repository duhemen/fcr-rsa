//! Encryption module for FCR-RSA v0.6.0
//! Fully migrated to U2048 and constant-time operations.

use crypto_bigint::U2048;
use crate::fib::{
    bytes_to_u2048, derive_blinding, encrypt_with_blinding_ct, FcrKeyPair,
};

/// Enkripsi pesan (dalam bentuk bytes) menggunakan FCR-RSA v0.6.0.
/// 
/// # Arguments
/// * `message_bytes` - Pesan yang akan dienkripsi (maksimal 255 byte).
/// * `key_pair` - Key pair yang dihasilkan dari `generate_key_pair`.
/// * `master_seed` - Seed utama untuk menurunkan blinding factor.
/// * `nonce` - Nonce acak (12 byte) yang akan dikirim bersama ciphertext.
pub fn encrypt(
    message_bytes: &[u8],
    key_pair: &FcrKeyPair,
    master_seed: &[u8],
    nonce: &[u8],
) -> Result<U2048, &'static str> {
    // 1. Konversi pesan ke U2048
    let message = bytes_to_u2048(message_bytes)?;
    
    // 2. Derive blinding factor secara deterministik dari master_seed + nonce
    let blinding = derive_blinding(master_seed, nonce)?;
    
    // 3. Enkripsi constant-time
    let ciphertext = encrypt_with_blinding_ct(&message, &key_pair.e_vec, &blinding);
    
    Ok(ciphertext)
}

/// Enkripsi langsung menggunakan tipe U2048 (untuk penggunaan internal/advanced).
pub fn encrypt_u2048(
    message: &U2048,
    key_pair: &FcrKeyPair,
    master_seed: &[u8],
    nonce: &[u8],
) -> Result<U2048, &'static str> {
    let blinding = derive_blinding(master_seed, nonce)?;
    Ok(encrypt_with_blinding_ct(message, &key_pair.e_vec, &blinding))
}