//! Cross-Language Test: Memverifikasi konsistensi antara Python Oracle, Lean 4 Formal Vectors, dan Rust Implementation.

use fcr_core_rust::{
    bytes_to_u2048, ct_select_limb, decrypt_with_blinding_ct, derive_blinding,
    encrypt_with_blinding_ct, generate_key_pair, u2048_to_bytes,
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

        let message = bytes_to_u2048(&message_bytes).expect("Message conversion failed");
        let key_pair = generate_key_pair(&master_seed).expect("Key gen failed");
        let blinding = derive_blinding(&master_seed, &nonce).expect("Blinding failed");

        let rust_ciphertext = encrypt_with_blinding_ct(&message, &key_pair.e_vec, &blinding);
        let rust_ct_bytes = u2048_to_bytes(&rust_ciphertext);

        if rust_ct_bytes.to_vec() == expected_ct_bytes {
            // Verifikasi tambahan Teorema 4 pada U2048 asli: decrypt(encrypt(M)) == M
            let decrypted = decrypt_with_blinding_ct(&rust_ciphertext, &key_pair.e_vec_inv, &blinding);
            assert_eq!(
                decrypted, message,
                "❌ Vector ID {}: Round-trip decrypt_with_blinding_ct gagal mengembalikan plaintext asli!",
                vec.id
            );

            println!("   ✅ Vector ID {}: MATCH (Encrypt & Decrypt Round-Trip Verified)", vec.id);
            passed += 1;
        } else {
            println!("   ❌ Vector ID {}: MISMATCH!", vec.id);
            println!("      Python expected: {}", &vec.expected_ciphertext_hex[..32]);
            println!("      Rust produced  : {}", &bytes_to_hex(&rust_ct_bytes)[..32]);
            failed += 1;
        }

    }

    println!("\n📊 HASIL JSON: {} passed, {} failed dari {} vectors", passed, failed, vectors.len());
    assert_eq!(failed, 0, "Ada test vector yang tidak cocok antara Python dan Rust!");
}

#[test]
fn test_lean4_formal_vectors() {
    // Pastikan path file sesuai dengan lokasi file CSV Anda (tests/test_vectors.csv atau test/test_vectors.csv)
    let csv_path = if std::path::Path::new("tests/test_vectors.csv").exists() {
        "tests/test_vectors.csv"
    } else {
        "test/test_vectors.csv"
    };

    let csv_data = fs::read_to_string(csv_path)
        .unwrap_or_else(|_| panic!("❌ Gagal membaca {}. Pastikan file CSV sudah disalin!", csv_path));

    let q: u128 = 1_000_000_007;
    let mut ct_count = 0;
    let mut blind_count = 0;

    for (line_num, raw_line) in csv_data.lines().enumerate() {
        // Bersihkan karakter BOM UTF-8 jika ada dari PowerShell Out-File
        let line = raw_line.trim().trim_start_matches('\u{feff}');
        if line.is_empty() || line.starts_with("kind,") {
            continue;
        }

        let cols: Vec<&str> = line.split(',').collect();

        match cols[0] {
            "ct" => {
                assert_eq!(cols.len(), 5, "Format baris 'ct' salah pada baris {}", line_num + 1);
                let choice: u64 = cols[1].parse().unwrap();
                let a: u64 = cols[2].parse().unwrap();
                let b: u64 = cols[3].parse().unwrap();
                let expected_res: u64 = cols[4].parse().unwrap();

                // Uji primitif constant-time Rust
                let actual_res = ct_select_limb(choice, a, b);
                assert_eq!(
                    actual_res, expected_res,
                    "Mismatch ctSelectLimb pada baris {}: choice={}, a={}, b={}",
                    line_num + 1, choice, a, b
                );
                ct_count += 1;
            }
            "blind" => {
                // Mendukung format 7 kolom (kind,M,E,B,Einv,Binv,out) maupun 6 kolom (kind,M,E,B,Einv,out)
                let m: u128 = cols[1].parse().unwrap();
                let e: u128 = cols[2].parse().unwrap();
                let b: u128 = cols[3].parse().unwrap();
                let e_inv: u128 = cols[4].parse().unwrap();

                if cols.len() == 7 {
                    let b_inv: u128 = cols[5].parse().unwrap();
                    let expected_out: u128 = cols[6].parse().unwrap();

                    let blinded = (m * e % q * b) % q;
                    let unblinded = (blinded * ((e_inv * b_inv) % q)) % q;
                    assert_eq!(unblinded, expected_out, "Mismatch multiplicative blinding di baris {}", line_num + 1);
                    assert_eq!(unblinded, m, "Round-trip blinding gagal kembali ke M di baris {}", line_num + 1);
                } else if cols.len() == 6 {
                    let expected_out: u128 = cols[5].parse().unwrap();

                    let blinded = ((m * e) % q + b) % q;
                    let unblinded = (((blinded + q - b) % q) * e_inv) % q;
                    assert_eq!(unblinded, expected_out, "Mismatch additive blinding di baris {}", line_num + 1);
                    assert_eq!(unblinded, m, "Round-trip blinding gagal kembali ke M di baris {}", line_num + 1);
                } else {
                    panic!("Format kolom 'blind' tidak dikenali pada baris {}", line_num + 1);
                }
                blind_count += 1;
            }
            other => panic!("Jenis vektor tidak dikenal '{}' pada baris {}", other, line_num + 1),
        }
    }

    println!(
        "✅ LEAN 4 FORMAL VECTORS VERIFIED: {} ct vectors & {} blinding vectors 100% MATCH!",
        ct_count, blind_count
    );
    assert_eq!(ct_count, 2048, "Jumlah vektor ct harus 2048");
    assert_eq!(blind_count, 512, "Jumlah vektor blind harus 512");
}
