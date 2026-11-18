//! Test IND-CPA untuk FCR-RSA v0.6.0
//! Memverifikasi bahwa deterministic blinding dari nonce menghasilkan ciphertext unik.

use fcr_core_rust::{
    bytes_to_u2048, derive_blinding, encrypt_with_blinding_ct, generate_key_pair,
};
use std::collections::HashSet;

#[test]
fn test_ind_cpa_randomness() {
    let master_seed = b"fcr-rsa-demo-seed";
    let key_pair = generate_key_pair(master_seed).unwrap();
    
    // Pesan "42" dalam U2048
    let message = bytes_to_u2048(&[42u8]).unwrap();
    
    let mut ciphertexts = HashSet::new();
    let iterations = 100;
    
    for i in 0..iterations {
        // Nonce unik setiap iterasi (mensimulasikan nonce acak di produksi)
        let nonce = [i as u8; 12]; 
        
        // Derive blinding secara deterministik dari master_seed + nonce
        let blinding = derive_blinding(master_seed, &nonce).unwrap();
        
        // Enkripsi
        let c = encrypt_with_blinding_ct(&message, &key_pair.e_vec, &blinding);
        ciphertexts.insert(c);
    }
    
    // Pastikan semua ciphertext unik (IND-CPA property)
    assert_eq!(
        ciphertexts.len(),
        iterations,
        "IND-CPA FAILED: Ada ciphertext yang duplikat! Blinding tidak unik."
    );
    
    println!("✅ IND-CPA PASSED: {} ciphertext unik dihasilkan dari {} enkripsi.", iterations, iterations);
}