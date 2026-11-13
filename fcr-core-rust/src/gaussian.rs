//! Gaussian integer mapping.

use num_bigint::BigInt;
use num_complex::Complex;

pub type GaussianInt = Complex<BigInt>;

/// Map Fibonacci sequence into Gaussian integers:
/// `Z_n = Re(F_n) + Im(F_{n+1}) * i`
pub fn map(seq: &[BigInt]) -> Vec<GaussianInt> {
    let mut out = Vec::with_capacity(seq.len().saturating_sub(1));
    for i in 0..seq.len().saturating_sub(1) {
        out.push(Complex::new(seq[i].clone(), seq[i + 1].clone()));
    }
    out
}