//! FCR Engine - Hybrid KEM + AEAD (v0.6.0)
//! Integrasi dengan Q 2048-bit

use crypto_bigint::U2048;
use fcr_core_rust::fib::{
    max_seed_bytes,
    generate_key_pair,
    derive_blinding,
    encrypt_with_blinding_ct,
    decrypt_with_blinding_ct,
    bytes_to_u2048,
    u2048_to_bytes,
    FcrKeyPair,
};
use hkdf::Hkdf;
use sha2::Sha256;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit, Nonce, aead::Aead};
use base64::{Engine as _, engine::general_purpose};
use rand::RngCore;

/// Enkripsi Hybrid KEM + AEAD (v0.6.0)
pub fn encrypt_hybrid_kem(
    text: &str, 
    key_pair: &FcrKeyPair,
    master_seed: &[u8]
) -> Result<String, Box<dyn std::error::Error>> {
    // 1. Generate session seed acak (255 byte untuk Q 2048-bit)
    let max_bytes = max_seed_bytes();
    let mut session_seed_bytes = vec![0u8; max_bytes];
    rand::rngs::OsRng.fill_bytes(&mut session_seed_bytes);
    
    // 2. Derive blinding factor secara deterministik
    let blinding = derive_blinding(master_seed, &nonce_bytes)?;
    // 3. Konversi session seed ke U2048
    let seed_bigint = bytes_to_u2048(&session_seed_bytes)?;
    
    // 4. Enkripsi seed dengan FCR-RSA (constant-time)
    let encrypted_seed = encrypt_with_blinding_ct(
        &seed_bigint, 
        &key_pair.e_vec, 
        &blinding
    );

    
    // 5. Turunkan kunci sesi 256-bit dari session seed menggunakan HKDF
    let hk = Hkdf::<Sha256>::new(None, &session_seed_bytes);
    let mut session_key_bytes = [0u8; 32];
    hk.expand(b"fcr-rsa-session-key-v1", &mut session_key_bytes)?;
    
    // 6. Enkripsi data dengan ChaCha20-Poly1305 (AEAD)
    let cipher = ChaCha20Poly1305::new_from_slice(&session_key_bytes)?;
    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, text.as_bytes())?;
    
    // 7. Format Output: ES:NONCE:CT (Base64)
    let es_b64 = general_purpose::STANDARD.encode(u2048_to_bytes(&encrypted_seed));
    let nonce_b64 = general_purpose::STANDARD.encode(nonce_bytes);
    let ct_b64 = general_purpose::STANDARD.encode(ciphertext);
    
    Ok(format!("{}:{}:{}", es_b64, nonce_b64, ct_b64))
}

/// Dekripsi Hybrid KEM + AEAD (v0.6.0)
pub fn decrypt_hybrid_kem(
    encrypted_data: &str, 
    key_pair: &FcrKeyPair,
    master_seed: &[u8]
) -> Result<String, Box<dyn std::error::Error>> {
    // 1. Parse format: ES:NONCE:CT
    let parts: Vec<&str> = encrypted_data.split(':').collect();
    if parts.len() != 3 {
        return Err("Invalid encrypted data format".into());
    }
    
    // 2. Decode Base64
    let encrypted_seed_bytes = general_purpose::STANDARD.decode(parts[0])?;
    let nonce_bytes = general_purpose::STANDARD.decode(parts[1])?;
    let ciphertext = general_purpose::STANDARD.decode(parts[2])?;
    
    // 3. Konversi encrypted seed ke U2048
    let encrypted_seed = bytes_to_u2048(&encrypted_seed_bytes)?;
    
    // 4. Dekripsi seed dengan FCR-RSA
    // Kita perlu derive blinding yang sama seperti saat enkripsi
    // Tapi kita belum tahu session_seed... ini masalah!
    
    // Solusi: Kita perlu menyimpan session_seed atau blinding di ciphertext
    // ATAU: Kita ubah desain agar blinding di-derive dari encrypted_seed itu sendiri
    
    // Untuk saat ini, kita gunakan pendekatan alternatif:
    // Blinding di-derive dari master_seed + encrypted_seed (deterministik)
    let blinding = derive_blinding(master_seed, &nonce_bytes)?;
    let session_seed_bigint = decrypt_with_blinding_ct(
        &encrypted_seed,
        &key_pair.e_vec_inv,
        &blinding
    );
    
    let session_seed_bytes = u2048_to_bytes(&session_seed_bigint);
    
    // 5. Turunkan kunci sesi 256-bit dari session seed
    let hk = Hkdf::<Sha256>::new(None, &session_seed_bytes);
    let mut session_key_bytes = [0u8; 32];
    hk.expand(b"fcr-rsa-session-key-v1", &mut session_key_bytes)?;
    
    // 6. Dekripsi data dengan ChaCha20-Poly1305
    let cipher = ChaCha20Poly1305::new_from_slice(&session_key_bytes)?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())?;
    
    Ok(String::from_utf8(plaintext)?)
}