//! Decryption module for FCR-RSA v0.6.0
//! Fully migrated to U2048 and constant-time operations.

use crypto_bigint::U2048;
use crate::fib::{
    u2048_to_bytes, derive_blinding, decrypt_with_blinding_ct, FcrKeyPair,
};

/// Dekripsi ciphertext (dalam bentuk U2048) menggunakan FCR-RSA v0.6.0.
/// Mengembalikan bytes asli (maksimal 255 byte).
/// 
/// # Arguments
/// * `ciphertext` - Ciphertext yang dienkripsi (tipe U2048).
/// * `key_pair` - Key pair yang sama dengan yang digunakan saat enkripsi.
/// * `master_seed` - Seed utama yang sama dengan yang digunakan saat enkripsi.
/// * `nonce` - Nonce yang sama dengan yang digunakan saat enkripsi (12 byte).
pub fn decrypt(
    ciphertext: &U2048,
    key_pair: &FcrKeyPair,
    master_seed: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, &'static str> {
    // 1. Derive blinding factor yang sama (deterministik)
    let blinding = derive_blinding(master_seed, nonce)?;
    
    // 2. Dekripsi constant-time
    let message = decrypt_with_blinding_ct(ciphertext, &key_pair.e_vec_inv, &blinding);
    
    // 3. Konversi kembali ke bytes
    let bytes = u2048_to_bytes(&message);
    
    // 4. Hapus leading zeros untuk mendapatkan pesan asli
    // (Aman untuk session seed acak, probabilitas leading zero sangat kecil)
    let first_non_zero = bytes.iter().position(|&x| x != 0).unwrap_or(bytes.len());
    
    Ok(bytes[first_non_zero..].to_vec())
}

/// Dekripsi langsung mengembalikan U2048 (untuk penggunaan internal/advanced).
pub fn decrypt_u2048(
    ciphertext: &U2048,
    key_pair: &FcrKeyPair,
    master_seed: &[u8],
    nonce: &[u8],
) -> Result<U2048, &'static str> {
    let blinding = derive_blinding(master_seed, nonce)?;
    Ok(decrypt_with_blinding_ct(ciphertext, &key_pair.e_vec_inv, &blinding))
}