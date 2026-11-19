//! FCR Engine - Hybrid KEM + AEAD (v0.6.0 FINAL & TEST-READY)
//! Integrasi dengan fcr-core-rust v0.6.0 (U2048 + Constant-Time)

use fcr_core_rust::fib::{
    generate_key_pair,
    derive_blinding,
    encrypt_with_blinding_ct,
    decrypt_with_blinding_ct,
    bytes_to_u2048,
    u2048_to_bytes,
    FcrKeyPair,
};
use crate::db::KeyRecord;
use hkdf::Hkdf;
use sha2::{Sha256, Digest};
use chacha20poly1305::{ChaCha20Poly1305, KeyInit, Nonce, aead::Aead};
use base64::{Engine as _, engine::general_purpose};
use rand::RngCore;

// ============================================================
// 1. Wrapper Kompatibilitas
// ============================================================

pub fn encrypt_text(
    text: &str,
    key: &KeyRecord,
    _tier: &str,
    blinding_override: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let seed_bytes = key.seed.as_bytes();
    let key_pair = generate_key_pair(seed_bytes)?;
    encrypt_hybrid_kem(text, &key_pair, seed_bytes, blinding_override)
}

pub fn decrypt_text(
    ciphertext: &str,
    key: &KeyRecord,
    _tier: &str,
    _blinding: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let seed_bytes = key.seed.as_bytes();
    let key_pair = generate_key_pair(seed_bytes)?;
    decrypt_hybrid_kem(ciphertext, &key_pair, seed_bytes)
}

pub fn hash_api_key(plaintext_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(plaintext_key.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn generate_api_key() -> (String, String, String) {
    let mut rng = rand::thread_rng();
    let mut key_bytes = [0u8; 32];
    rng.fill_bytes(&mut key_bytes);
    let random_hex = hex::encode(&key_bytes[..16]);
    let plaintext = format!("fcr_{}", random_hex);
    let prefix = plaintext[..8].to_string();
    let hash = hash_api_key(&plaintext);
    (plaintext, hash, prefix)
}

// ============================================================
// 2. Hybrid KEM + AEAD Core (FIXED: Full Determinism for Test Vectors)
// ============================================================

pub fn encrypt_hybrid_kem(
    text: &str,
    key_pair: &FcrKeyPair,
    master_seed: &[u8],
    blinding_override: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    
    // 1. Session seed: Deterministik jika ada override, Acak jika tidak
    let mut session_seed_bytes = [0u8; 32];
    if let Some(override_str) = blinding_override {
        let mut hasher = Sha256::new();
        hasher.update(b"fcr-session-seed:");
        hasher.update(override_str.as_bytes());
        session_seed_bytes.copy_from_slice(&hasher.finalize()[..32]);
    } else {
        rand::rngs::OsRng.fill_bytes(&mut session_seed_bytes);
    }

    // 2. Blinding nonce: Deterministik jika ada override, Acak jika tidak
    let mut blinding_nonce = [0u8; 16];
    if let Some(override_str) = blinding_override {
        let bytes = override_str.as_bytes();
        let len = bytes.len().min(16);
        blinding_nonce[..len].copy_from_slice(&bytes[..len]);
    } else {
        rand::rngs::OsRng.fill_bytes(&mut blinding_nonce);
    }
    
    // 3. Derive blinding factor secara deterministik
    let blinding = derive_blinding(master_seed, &blinding_nonce)?;

    // 4. Konversi session seed ke U2048
    let seed_bigint = bytes_to_u2048(&session_seed_bytes)?;

    // 5. Enkripsi seed dengan FCR-RSA (Constant-Time)
    let encrypted_seed = encrypt_with_blinding_ct(&seed_bigint, &key_pair.e_vec, &blinding);

    // 6. Turunkan kunci sesi 256-bit dari session seed menggunakan HKDF
    let hk = Hkdf::<Sha256>::new(None, &session_seed_bytes);
    let mut session_key_bytes = [0u8; 32];
    hk.expand(b"fcr-rsa-session-key-v1", &mut session_key_bytes).map_err(|e| e.to_string())?;

    // 7. AEAD Nonce: Deterministik jika ada override, Acak jika tidak
    let mut aead_nonce = [0u8; 12];
    if let Some(override_str) = blinding_override {
        let mut hasher = Sha256::new();
        hasher.update(b"fcr-aead-nonce:");
        hasher.update(override_str.as_bytes());
        aead_nonce.copy_from_slice(&hasher.finalize()[..12]);
    } else {
        rand::rngs::OsRng.fill_bytes(&mut aead_nonce);
    }

    let cipher = ChaCha20Poly1305::new_from_slice(&session_key_bytes).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&aead_nonce);
    let ciphertext = cipher.encrypt(nonce, text.as_bytes()).map_err(|e| e.to_string())?;

    // 8. Format Output 4 BAGIAN: BLINDING_NONCE:ENCRYPTED_SEED:AEAD_NONCE:CIPHERTEXT
    let bn_b64 = general_purpose::STANDARD.encode(blinding_nonce);
    let es_b64 = general_purpose::STANDARD.encode(u2048_to_bytes(&encrypted_seed));
    let an_b64 = general_purpose::STANDARD.encode(aead_nonce);
    let ct_b64 = general_purpose::STANDARD.encode(ciphertext);
    
    Ok(format!("{}:{}:{}:{}", bn_b64, es_b64, an_b64, ct_b64))
}

pub fn decrypt_hybrid_kem(
    encrypted_data: &str,
    key_pair: &FcrKeyPair,
    master_seed: &[u8],
) -> Result<String, Box<dyn std::error::Error>> {
    // 1. Parse format 4 BAGIAN
    let parts: Vec<&str> = encrypted_data.split(':').collect();
    if parts.len() != 4 {
        return Err(format!("Invalid encrypted data format. Expected 4 parts, got {}.", parts.len()).into());
    }

    // 2. Decode Base64
    let blinding_nonce = general_purpose::STANDARD.decode(parts[0])?;
    let encrypted_seed_bytes = general_purpose::STANDARD.decode(parts[1])?;
    let aead_nonce = general_purpose::STANDARD.decode(parts[2])?;
    let ciphertext = general_purpose::STANDARD.decode(parts[3])?;

    // 3. Derive blinding yang SAMA PERSIS seperti saat enkripsi!
    let blinding = derive_blinding(master_seed, &blinding_nonce)?;

    // 4. Konversi encrypted seed ke U2048
    let encrypted_seed = bytes_to_u2048(&encrypted_seed_bytes)?;

    // 5. Dekripsi seed dengan FCR-RSA (Constant-Time)
    let session_seed_bigint = decrypt_with_blinding_ct(&encrypted_seed, &key_pair.e_vec_inv, &blinding);
    
    // 6. FIX KRITIS: Konversi U2048 ke byte dan POTONG menjadi 32 byte asli!
    let full_bytes = u2048_to_bytes(&session_seed_bigint);
    let session_seed_bytes = if full_bytes.len() > 32 {
        &full_bytes[full_bytes.len() - 32..]
    } else {
        &full_bytes
    };

    // 7. Turunkan kunci sesi 256-bit dari session seed (sekarang panjangnya 32 byte, cocok!)
    let hk = Hkdf::<Sha256>::new(None, session_seed_bytes);
    let mut session_key_bytes = [0u8; 32];
    hk.expand(b"fcr-rsa-session-key-v1", &mut session_key_bytes).map_err(|e| e.to_string())?;

    // 8. Dekripsi data dengan ChaCha20-Poly1305
    let cipher = ChaCha20Poly1305::new_from_slice(&session_key_bytes).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&aead_nonce);
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref()).map_err(|e| e.to_string())?;
    
    Ok(String::from_utf8(plaintext)?)
}