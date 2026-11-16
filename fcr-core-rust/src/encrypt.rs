//! Encryption with IND-CPA patch and secure random blinding.

use num_bigint::BigInt;
use num_traits::Zero;
use rand::Rng; // Trait yang benar di rand 0.10 (menggantikan RngCore)

use crate::fib;

/// Helper: Generate a cryptographically secure random blinding factor.
fn generate_random_blinding(q: &BigInt) -> BigInt {
    let mut bytes = [0u8; 32];
    // Di rand 0.10, gunakan SysRng (pengganti OsRng) atau rand::rng()
    // Cara paling aman dan modern:
    rand::rng().fill_bytes(&mut bytes);
    
    // Konversi byte ke BigInt
    let random_val = BigInt::from_bytes_be(num_bigint::Sign::Plus, &bytes);
    random_val % q
}

/// Encrypt a plaintext BigInt using FCR-RSA Tier B (lightweight).
/// C = (M * E_vec) + R_blind mod Q
/// 
/// **Fungsi Produksi**: R_blind dibangkitkan secara acak (IND-CPA aman).
pub fn encrypt(
    message: &BigInt,
    seed: &[u8],
    n: usize,
    q: &BigInt,
) -> Result<BigInt, &'static str> {
    if n < 4 {
        return Err("n must be >= 4");
    }
    if q <= &BigInt::zero() {
        return Err("q must be positive");
    }

    let blinding = generate_random_blinding(q);
    encrypt_with_blinding(message, seed, n, q, &blinding)
}

/// **Fungsi Deterministik**: Digunakan untuk test vector & verifikasi.
/// Jangan gunakan di produksi karena R_blind statis membocorkan IND-CPA.
pub fn encrypt_with_blinding(
    message: &BigInt,
    seed: &[u8],
    n: usize,
    q: &BigInt,
    blinding: &BigInt,
) -> Result<BigInt, &'static str> {
    if n < 4 {
        return Err("n must be >= 4");
    }
    if q <= &BigInt::zero() {
        return Err("q must be positive");
    }
    let seq = fib::generate(n, seed, q);
    let e_vec = &seq[n - 2];

    let prod = (message * e_vec) % q;
    let c = (prod + blinding) % q;
    let c = if c < BigInt::zero() { c + q } else { c };
    Ok(c)
}

/// Encrypt using Tier A (heavyweight).
/// C = (M * E_vec) + N_comp + R_blind mod Q
/// 
/// **Fungsi Produksi**: R_blind dibangkitkan secara acak (IND-CPA aman).
pub fn encrypt_tier_a(
    message: &BigInt,
    seed: &[u8],
    n: usize,
    q: &BigInt,
) -> Result<BigInt, &'static str> {
    if n < 4 {
        return Err("n must be >= 4");
    }
    if q <= &BigInt::zero() {
        return Err("q must be positive");
    }

    let blinding = generate_random_blinding(q);
    encrypt_tier_a_with_blinding(message, seed, n, q, &blinding)
}

/// **Fungsi Deterministik**: Tier A dengan blinding statis (untuk test).
pub fn encrypt_tier_a_with_blinding(
    message: &BigInt,
    seed: &[u8],
    n: usize,
    q: &BigInt,
    blinding: &BigInt,
) -> Result<BigInt, &'static str> {
    if n < 4 {
        return Err("n must be >= 4");
    }
    if q <= &BigInt::zero() {
        return Err("q must be positive");
    }
    let seq = fib::generate(n, seed, q);
    let e_vec = &seq[n - 2];
    let n_comp = &seq[n - 1];

    let prod = (message * e_vec) % q;
    let c = (prod + n_comp + blinding) % q;
    let c = if c < BigInt::zero() { c + q } else { c };
    Ok(c)
}