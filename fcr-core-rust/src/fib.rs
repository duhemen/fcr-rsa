//! Non-linear Fibonacci sequence with deterministic LWE noise.

use num_bigint::BigInt;
use num_traits::{One, Zero};
use sha2::{Digest, Sha256};

/// Generate non-linear Fibonacci sequence.
///
/// `F_n = (F_{n-1} * F_{n-2}) + e_n mod Q`
pub fn generate(n: usize, seed: &[u8], q: &BigInt) -> Vec<BigInt> {
    let mut seq: Vec<BigInt> = Vec::with_capacity(n);
    let mut noise = NoiseStream::new(seed);

    if n == 0 {
        return seq;
    }
    seq.push(BigInt::one());
    if n == 1 {
        return seq;
    }
    seq.push(BigInt::one());

    for i in 2..n {
        let prod = &seq[i - 1] * &seq[i - 2];
        let e = noise.next_bigint(q);
        let val = (prod + e) % q;
        let val = if val < BigInt::zero() { val + q } else { val };
        seq.push(val);
    }
    seq
}

/// Deterministic noise stream from SHA-256(seed || counter).
pub struct NoiseStream {
    seed: Vec<u8>,
    counter: u64,
    buffer: Vec<u8>,
    pos: usize,
}

impl NoiseStream {
    pub fn new(seed: &[u8]) -> Self {
        Self {
            seed: seed.to_vec(),
            counter: 0,
            buffer: Vec::new(),
            pos: 0,
        }
    }

    fn refill(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(&self.seed);
        hasher.update(self.counter.to_le_bytes());
        self.buffer = hasher.finalize().to_vec();
        self.counter += 1;
        self.pos = 0;
    }

    fn next_bytes(&mut self, n: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(n);
        while out.len() < n {
            if self.pos >= self.buffer.len() {
                self.refill();
            }
            let take = (n - out.len()).min(self.buffer.len() - self.pos);
            out.extend_from_slice(&self.buffer[self.pos..self.pos + take]);
            self.pos += take;
        }
        out
    }

    pub fn next_bigint(&mut self, q: &BigInt) -> BigInt {
        let bytes = self.next_bytes(32);
        let val = BigInt::from_bytes_le(num_bigint::Sign::Plus, &bytes);
        let val = &val % q;
        if val < BigInt::zero() { val + q } else { val }
    }
}