//! Encryption with IND-CPA patch.
//!
//! Tier B (lightweight): C = (M * E_vec) + R_blind  mod Q
//! Tier A (heavyweight): C = (M * E_vec) + N_comp + R_blind  mod Q
//!
//! Untuk prototype ini, `encrypt` mengimplementasikan Tier B
//! (tanpa N_comp), konsisten dengan `tier_b.py`.

use num_bigint::BigInt;
use num_traits::Zero;

use crate::fib;

/// Encrypt a plaintext BigInt using FCR-RSA Tier B.
pub fn encrypt(
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
