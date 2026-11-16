use dudect_bencher::{ctbench_main, BenchRng, Class, CtRunner};
use fcr_rsa::decrypt;
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use rand::RngExt;

// Parameter yang akan digunakan di seluruh tes
const Q: u64 = 1_000_000_007;
const SEED: &[u8] = b"fcr-rsa-demo-seed";
const N: usize = 10;
const BLINDING: u64 = 12345;

/// Fungsi pembantu untuk membuat input yang valid (Kelas Left)
fn make_valid_input() -> BigInt {
    let q = BigInt::from_u64(Q).unwrap();
    let m = BigInt::from_u64(42).unwrap();
    // PERBAIKAN: encrypt sekarang hanya butuh 4 argumen (tanpa blinding)
    // karena fungsi produksi akan otomatis generate R_blind.
    fcr_rsa::encrypt(&m, SEED, N, &q).unwrap()
}

/// Fungsi pembantu untuk membuat input yang rusak (Kelas Right)
fn make_corrupted_input(rng: &mut BenchRng) -> BigInt {
    let valid_ct = make_valid_input();
    let delta = rng.random_range(1..1000);
    &valid_ct + BigInt::from_u64(delta).unwrap()
}

/// Benchmark utama untuk dudect
fn decrypt_timing_test(runner: &mut CtRunner, rng: &mut BenchRng) {
    let q = BigInt::from_u64(Q).unwrap();
    let b = BigInt::from_u64(BLINDING).unwrap();

    let samples = 100_000;
    let mut inputs = Vec::with_capacity(samples);
    let mut classes = Vec::with_capacity(samples);

    for _ in 0..samples {
        if rng.random_bool(0.5) {
            inputs.push(make_valid_input());
            classes.push(Class::Left);
        } else {
            inputs.push(make_corrupted_input(rng));
            classes.push(Class::Right);
        }
    }

    for (class, ct) in classes.into_iter().zip(inputs.into_iter()) {
        runner.run_one(class, || {
            // Fungsi decrypt masih butuh blinding karena dekripsi harus tahu R.
            let _ = decrypt(&ct, SEED, N, &q, &b);
        });
    }
}

ctbench_main!(decrypt_timing_test);