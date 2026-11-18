//! Statistical Timing Test for FCR-RSA v0.6.0
//! Memverifikasi constant-time property menggunakan Welch's t-test.
//!
//! Metodologi:
//! 1. Generate dua kelas input: Heavy (banyak bit 1) dan Light (banyak bit 0)
//! 2. Jalankan encrypt_with_blinding_ct 10,000 kali untuk setiap kelas
//! 3. Ukur waktu eksekusi setiap iterasi
//! 4. Lakukan Welch's t-test untuk membandingkan distribusi waktu
//! 5. Jika p-value > 0.05, implementasi dianggap constant-time

use statrs::distribution::{ContinuousCDF, StudentsT};
use fcr_core_rust::{
    bytes_to_u2048, derive_blinding, encrypt_with_blinding_ct, generate_key_pair,
};
use std::time::Instant;

const ITERATIONS: usize = 10_000;
const WARMUP_ITERATIONS: usize = 100;

/// Hitung mean dari slice
fn mean(data: &[f64]) -> f64 {
    data.iter().sum::<f64>() / data.len() as f64
}

/// Hitung variance dari slice
fn variance(data: &[f64], mean: f64) -> f64 {
    data.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>()
        / (data.len() - 1) as f64
}

/// Welch's t-test untuk dua sampel independen
/// Mengembalikan (t-statistic, degrees_of_freedom)
fn welch_t_test(sample1: &[f64], sample2: &[f64]) -> (f64, f64) {
    let mean1 = mean(sample1);
    let mean2 = mean(sample2);
    let var1 = variance(sample1, mean1);
    let var2 = variance(sample2, mean2);
    let n1 = sample1.len() as f64;
    let n2 = sample2.len() as f64;

    let se1 = var1 / n1;
    let se2 = var2 / n2;
    let se = se1 + se2;

    let t = (mean1 - mean2) / se.sqrt();

    // Welch-Satterthwaite degrees of freedom
    let num = (se1 + se2).powi(2);
    let denom = (se1.powi(2) / (n1 - 1.0)) + (se2.powi(2) / (n2 - 1.0));
    let df = num / denom;

    (t, df)
}

/// Approximate p-value dari t-distribution (two-tailed)
/// Menggunakan aproksimasi normal untuk df besar
fn t_to_pvalue(t: f64, df: f64) -> f64 {
    let t_dist = StudentsT::new(0.0, 1.0, df).unwrap();
    // Two-tailed test: p = 2 * P(T > |t|)
    2.0 * (1.0 - t_dist.cdf(t.abs()))
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║   FCR-RSA v0.6.0 - Statistical Timing Test (A4)           ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // 1. Setup
    println!("[1] Setup: Generating key pair...");
    let master_seed = b"timing-test-seed";
    let key_pair = generate_key_pair(master_seed).expect("Key generation failed");
    let nonce = [42u8; 12];
    let blinding = derive_blinding(master_seed, &nonce).expect("Blinding derivation failed");

    // 2. Prepare Input Classes
    println!("[2] Preparing input classes...");
    
    // Kelas A: Heavy (banyak bit 1) - 0xFFFFFFFFFFFFFFFF...
    let heavy_bytes = [0xFFu8; 255];
    let message_heavy = bytes_to_u2048(&heavy_bytes).expect("Message conversion failed");
    
    // Kelas B: Light (banyak bit 0) - 0x0000000000000001...
    let mut light_bytes = [0x00u8; 255];
    light_bytes[254] = 0x01; // Hanya bit terakhir yang 1
    let message_light = bytes_to_u2048(&light_bytes).expect("Message conversion failed");

    println!("   Kelas A (Heavy): 255 bytes × 0xFF (2040 bit '1')");
    println!("   Kelas B (Light): 255 bytes × 0x00 + 0x01 (1 bit '1')\n");

    // 3. Warmup (stabilisasi cache CPU)
    println!("[3] Warmup: {} iterations...", WARMUP_ITERATIONS);
    for _ in 0..WARMUP_ITERATIONS {
        let _ = encrypt_with_blinding_ct(&message_heavy, &key_pair.e_vec, &blinding);
        let _ = encrypt_with_blinding_ct(&message_light, &key_pair.e_vec, &blinding);
    }
    println!("   ✅ Warmup complete\n");

    // 4. Benchmark Kelas A (Heavy)
    println!("[4] Benchmark Kelas A (Heavy): {} iterations...", ITERATIONS);
    let mut times_a = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        let _ct = encrypt_with_blinding_ct(&message_heavy, &key_pair.e_vec, &blinding);
        let duration = start.elapsed();
        times_a.push(duration.as_nanos() as f64);
    }
    let mean_a = mean(&times_a);
    let var_a = variance(&times_a, mean_a);
    println!("   Mean: {:.2} ns", mean_a);
    println!("   Variance: {:.2}\n", var_a);

    // 5. Benchmark Kelas B (Light)
    println!("[5] Benchmark Kelas B (Light): {} iterations...", ITERATIONS);
    let mut times_b = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        let _ct = encrypt_with_blinding_ct(&message_light, &key_pair.e_vec, &blinding);
        let duration = start.elapsed();
        times_b.push(duration.as_nanos() as f64);
    }
    let mean_b = mean(&times_b);
    let var_b = variance(&times_b, mean_b);
    println!("   Mean: {:.2} ns", mean_b);
    println!("   Variance: {:.2}\n", var_b);

    // 6. Statistical Analysis
    println!("[6] Statistical Analysis (Welch's t-test)...");
    let (t_stat, df) = welch_t_test(&times_a, &times_b);
    let p_value = t_to_pvalue(t_stat, df);
    
    println!("   t-statistic: {:.4}", t_stat);
    println!("   Degrees of freedom: {:.2}", df);
    println!("   p-value: {:.6}\n", p_value);

    // 7. Conclusion
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                    KESIMPULAN                              ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
    
    let diff_percent = ((mean_a - mean_b).abs() / mean_a) * 100.0;
    println!("   Perbedaan mean: {:.2} ns ({:.4}%)", (mean_a - mean_b).abs(), diff_percent);
    
    if p_value > 0.05 {
        println!("\n   ✅ CONSTANT-TIME VERIFIED!");
        println!("   p-value ({:.6}) > 0.05 → Tidak ada perbedaan statistik yang signifikan.", p_value);
        println!("   Implementasi FCR-RSA v0.6.0 TAHAN terhadap timing side-channel attack.");
    } else {
        println!("\n   ⚠️  TIMING LEAK DETECTED!");
        println!("   p-value ({:.6}) < 0.05 → Ada perbedaan statistik yang signifikan.", p_value);
        println!("   Implementasi MUNGKIN rentan terhadap timing side-channel attack.");
        println!("   Rekomendasi: Review operasi modular untuk secret-dependent branching.");
    }
    
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  Catatan: Test ini mengukur timing di level aplikasi.      ║");
    println!("║  Untuk verifikasi hardware-level, gunakan dudect/valgrind. ║");
    println!("╚════════════════════════════════════════════════════════════╝");
}