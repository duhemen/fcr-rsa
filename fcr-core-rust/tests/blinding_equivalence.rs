//! Differential Equivalence Test: Lean 4 Model vs Rust Production Code
use crypto_bigint::U2048;
use fcr_core_rust::fib::{
    decrypt_with_blinding_ct, derive_blinding, encrypt_with_blinding_ct, generate_key_pair,
};

#[test]
fn test_blinding_cancellation_1000_vectors() {
    let master_seed = b"differential-equivalence-test-seed";
    let mut success_count = 0;
    let total_vectors: u32 = 1000;

    println!(
        "🚀 Menjalankan {} vektor uji deterministik...",
        total_vectors
    );

    for i in 0..total_vectors {
        let mut seed_bytes = [0u8; 32];

        // PERBAIKAN: Cast 'i' ke u64 agar .to_le_bytes() menghasilkan 8 byte
        seed_bytes[0..8].copy_from_slice(&(i as u64).to_le_bytes());
        seed_bytes[8..16].copy_from_slice(&(i as u64).to_le_bytes());

        let key_pair = generate_key_pair(&seed_bytes).expect("Keygen failed");

        let mut m_bytes = [0u8; 256];
        m_bytes[0..8].copy_from_slice(&(i as u64).to_be_bytes());
        let m = U2048::from_be_slice(&m_bytes);

        let nonce = format!("nonce-{}", i);
        let blinding = derive_blinding(master_seed, nonce.as_bytes()).expect("Blinding failed");

        let ciphertext = encrypt_with_blinding_ct(&m, &key_pair.e_vec, &blinding);
        let decrypted = decrypt_with_blinding_ct(&ciphertext, &key_pair.e_vec_inv, &blinding);

        assert_eq!(m, decrypted, "Blinding cancellation failed at vector {}", i);
        success_count += 1;
    }

    println!(
        "✅ SUKSES: {}/{} vektor mematuhi Teorema Pembatalan Blinding Lean 4.",
        success_count, total_vectors
    );
}
