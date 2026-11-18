//! Cross-Language Test: Memverifikasi konsistensi antara Python Oracle dan Rust Implementation.
//! 
//! Test ini membaca test_vectors.json yang di-generate oleh Python,
//! lalu memverifikasi bahwa Rust menghasilkan ciphertext yang IDENTIK.

use fcr_core_rust::{
    bytes_to_u2048, derive_blinding, encrypt_with_blinding_ct, generate_key_pair, u2048_to_bytes,
};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct TestVector {
    id: u32,
    master_seed_hex: String,
    nonce_hex: String,
    message_hex: String,
    expected_ciphertext_hex: String,
}

/// Helper: Konversi hex string ke bytes
fn hex_to_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

/// Helper: Konversi bytes ke hex string
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[test]
fn test_cross_language_vectors() {
    // 1. Baca file JSON yang di-generate oleh Python
    let json_data = fs::read_to_string("tests/test_vectors.json")
        .expect("❌ Gagal membaca test_vectors.json. Jalankan generate_test_vectors.py terlebih dahulu!");
    
    let vectors: Vec<TestVector> = serde_json::from_str(&json_data).expect("❌ JSON parsing failed");
    
    println!("🦀 Memverifikasi {} test vectors dari Python Oracle...", vectors.len());

    let mut passed = 0;
    let mut failed = 0;

    // 2. Loop dan verifikasi setiap vector
    for vec in &vectors {
        let master_seed = hex_to_bytes(&vec.master_seed_hex);
        let nonce = hex_to_bytes(&vec.nonce_hex);
        let message_bytes = hex_to_bytes(&vec.message_hex);
        let expected_ct_bytes = hex_to_bytes(&vec.expected_ciphertext_hex);

        // Konversi message bytes ke U2048
        let message = bytes_to_u2048(&message_bytes).expect("Message conversion failed");

        // Generate Key Pair & Blinding (harus identik dengan Python)
        let key_pair = generate_key_pair(&master_seed).expect("Key gen failed");
        let blinding = derive_blinding(&master_seed, &nonce).expect("Blinding failed");

        // Enkripsi dengan Rust
        let rust_ciphertext = encrypt_with_blinding_ct(&message, &key_pair.e_vec, &blinding);
        let rust_ct_bytes = u2048_to_bytes(&rust_ciphertext);

        // 3. ASSERT: Output Rust HARUS sama persis dengan Output Python
        if rust_ct_bytes.to_vec() == expected_ct_bytes {
            println!("   ✅ Vector ID {}: MATCH", vec.id);
            passed += 1;
        } else {
            println!("   ❌ Vector ID {}: MISMATCH!", vec.id);
            // PERBAIKAN: Tambahkan '&' di depan slicing string
            println!("      Python expected: {}", &vec.expected_ciphertext_hex[..32]);
            println!("      Rust produced  : {}", &bytes_to_hex(&rust_ct_bytes)[..32]);
            failed += 1;
        }
    }
    
    println!("\n📊 HASIL: {} passed, {} failed dari {} vectors", passed, failed, vectors.len());
    
    assert_eq!(failed, 0, "Ada test vector yang tidak cocok antara Python dan Rust!");
    println!("✅ SEMUA VECTOR COCOK! Python dan Rust 100% Sinkron.");
}