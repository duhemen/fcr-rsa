//! Benchmarks (placeholder).
use criterion::{criterion_group, criterion_main, Criterion};
use fcr_rsa::{decrypt, encrypt};
use num_bigint::BigInt;
use num_traits::FromPrimitive;

fn bench_round_trip(c: &mut Criterion) {
    let q = BigInt::from_u64(1_000_000_007).unwrap();
    let seed = b"bench-seed";
    let n = 100usize;
    let b = BigInt::from_u64(7).unwrap();
    let m = BigInt::from_u64(42).unwrap();

    c.bench_function("encrypt_n100", |bench| {
        bench.iter(|| encrypt(&m, seed, n, &q, &b).unwrap())
    });
    let ct = encrypt(&m, seed, n, &q, &b).unwrap();
    c.bench_function("decrypt_n100", |bench| {
        bench.iter(|| decrypt(&ct, seed, n, &q, &b).unwrap())
    });
}

criterion_group!(benches, bench_round_trip);
criterion_main!(benches);