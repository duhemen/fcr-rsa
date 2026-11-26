//! FCR Engine - Hybrid KEM + AEAD (v0.6.1 HARDENED)
//! Integrasi dengan fcr-core-rust v0.6.0 (U2048 + Constant-Time + Tier Isolation)

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
// 0. Domain Separation Helper (Tier A vs Tier B Isolation)
// ============================================================

fn build_hkdf_info(tier: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let clean_tier = tier.trim().to_lowercase();
    match clean_tier.as_str() {
        "a" | "b" => Ok(format!("fcr-rsa-session-key-v1:tier-{}", clean_tier).into_bytes()),
        _ => Err("Invalid tier: must be 'a' or 'b'".into()),
    }
}

// ============================================================
// 0b. Derivasi kunci sesi untuk FCR-TP (dipakai bersama oleh
//     enkripsi FCR-RSA dan transport). Perilaku HKDF-nya IDENTIK
//     dengan yang dipakai encrypt/decrypt di bawah.
// ============================================================

/// Session seed deterministik dari sebuah secret bersama.
/// SAMA PERSIS dengan cabang `blinding_override` di encrypt_hybrid_kem.
pub fn session_seed_from_secret(secret: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"fcr-session-seed:");
    hasher.update(secret.as_bytes());
    let mut out = [0u8; 32];
    out.copy_from_slice(&hasher.finalize()[..32]);
    out
}

/// Kunci sesi 32 byte dari session seed + tier.
pub fn derive_session_key(
    session_seed_bytes: &[u8],
    tier: &str,
) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let hkdf_info = build_hkdf_info(tier)?;
    derive_session_key_from_info(session_seed_bytes, &hkdf_info)
}

/// Versi internal: dipakai encrypt/decrypt yang sudah memegang hkdf_info.
fn derive_session_key_from_info(
    session_seed_bytes: &[u8],
    hkdf_info: &[u8],
) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let hk = Hkdf::<Sha256>::new(None, session_seed_bytes);
    let mut out = [0u8; 32];
    hk.expand(hkdf_info, &mut out).map_err(|e| e.to_string())?;
    Ok(out)
}

/// Seed hopper (u64) dari 8 byte pertama session seed.
/// Kedua pihak mendapat urutan port yang sama karena seed-nya sama.
pub fn derive_hopper_seed(session_seed_bytes: &[u8]) -> u64 {
    u64::from_be_bytes(session_seed_bytes[..8].try_into().unwrap())
}

// ============================================================
// 1. Wrapper Kompatibilitas
// ============================================================

pub fn encrypt_text(
    text: &str,
    key: &KeyRecord,
    tier: &str,
    blinding_override: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let hkdf_info = build_hkdf_info(tier)?;
    let seed_bytes = key.seed.as_bytes();
    let key_pair = generate_key_pair(seed_bytes)?;
    encrypt_hybrid_kem(text, &key_pair, seed_bytes, blinding_override, &hkdf_info)
}

pub fn decrypt_text(
    ciphertext: &str,
    key: &KeyRecord,
    tier: &str,
    _blinding: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let hkdf_info = build_hkdf_info(tier)?;
    let seed_bytes = key.seed.as_bytes();
    let key_pair = generate_key_pair(seed_bytes)?;
    decrypt_hybrid_kem(ciphertext, &key_pair, seed_bytes, &hkdf_info)
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
// 2. Hybrid KEM + AEAD Core (Full Determinism + Tier Bound)
// ============================================================

pub fn encrypt_hybrid_kem(
    text: &str,
    key_pair: &FcrKeyPair,
    master_seed: &[u8],
    blinding_override: Option<&str>,
    hkdf_info: &[u8],
) -> Result<String, Box<dyn std::error::Error>> {
    // 1. Session seed: Deterministik jika ada override, Acak jika tidak
    let mut session_seed_bytes = [0u8; 32];
    if let Some(override_str) = blinding_override {
        session_seed_bytes = session_seed_from_secret(override_str);
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

    // 6. Turunkan kunci sesi 256-bit dari session seed + TIER CONTEXT menggunakan HKDF
    let session_key_bytes = derive_session_key_from_info(&session_seed_bytes, hkdf_info)?;

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
    hkdf_info: &[u8],
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
    
    // 6. Konversi U2048 ke byte dan POTONG menjadi 32 byte asli
    let full_bytes = u2048_to_bytes(&session_seed_bigint);
    let session_seed_bytes = if full_bytes.len() > 32 {
        &full_bytes[full_bytes.len() - 32..]
    } else {
        &full_bytes
    };

    // 7. Turunkan kunci sesi 256-bit dengan TIER CONTEXT yang wajib cocok
    let session_key_bytes = derive_session_key_from_info(session_seed_bytes, hkdf_info)?;

    // 8. Dekripsi data dengan ChaCha20-Poly1305
    let cipher = ChaCha20Poly1305::new_from_slice(&session_key_bytes).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&aead_nonce);
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref()).map_err(|e| e.to_string())?;
    
    Ok(String::from_utf8(plaintext)?)
}
