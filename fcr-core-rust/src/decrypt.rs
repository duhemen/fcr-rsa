//! Decryption (symmetric, prototype) — Tier B.
//!
//! Konsisten dengan `encrypt.rs` (Tier B) dan `tier_b.py`.

use num_bigint::BigInt;
use num_traits::Zero;

use crate::fib;

pub fn decrypt(
    ciphertext: &BigInt,
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

    let mut c = (ciphertext - blinding) % q;
    if c < BigInt::zero() {
        c += q;
    }

    let inv = mod_inverse(e_vec, q)?;
    let m = (c * inv) % q;
    Ok(if m < BigInt::zero() { m + q } else { m })
}

fn mod_inverse(a: &BigInt, m: &BigInt) -> Result<BigInt, &'static str> {
    use num_traits::One;
    let (g, x, _) = extended_gcd(a.clone(), m.clone());
    if g != BigInt::one() {
        return Err("no modular inverse");
    }
    let x = x % m;
    Ok(if x < BigInt::zero() { x + m } else { x })
}

fn extended_gcd(a: BigInt, b: BigInt) -> (BigInt, BigInt, BigInt) {
    use num_traits::{One, Zero};
    if b.is_zero() {
        return (a, BigInt::one(), BigInt::zero());
    }
    let (g, x, y) = extended_gcd(b.clone(), &a % &b);
    (g, y.clone(), x - (&a / &b) * y)
}
