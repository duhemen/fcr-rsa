//! Wrapper enkripsi/dekripsi FCR-RSA untuk engine.
//!
//! Menyediakan:
//! - `encrypt_text` / `decrypt_text`: per-byte block cipher (Base-256)
//! - Generate random blinding yang aman (fix dari time-based)

use fcr_rsa::{decrypt, decrypt_tier_a, encrypt_with_blinding, encrypt_tier_a_with_blinding};
use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};
use rand::Rng;
use std::str::FromStr;

use crate::db::KeyRecord;

/// Generate random blinding yang aman (32 byte → modulo 1e9).
/// Menggantikan `SystemTime::now().subsec_nanos()` yang tidak aman.
pub fn generate_blinding() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let val = BigInt::from_bytes_be(num_bigint::Sign::Plus, &bytes) % BigInt::from(1_000_000_000u64);
    // Pastikan tidak nol
    let val = if val == BigInt::from(0u64) {
        BigInt::from(1u64)
    } else {
        val
    };
    val.to_string()
}

/// Enkripsi teks per-byte. Return format `"BLINDING:CT1,CT2,..."`.
///
/// - Jika `blinding` = `Some`, gunakan deterministik (untuk test vector).
/// - Jika `blinding` = `None`, generate random (IND-CPA produksi).
pub fn encrypt_text(
    text: &str,
    key: &KeyRecord,
    tier: &str,
    blinding: Option<&str>,
) -> Result<String, String> {
    let q = BigInt::from_str(&key.q).map_err(|e| format!("invalid Q: {}", e))?;
    let seed = key.seed.as_bytes();
    let n = key.n as usize;
    let is_tier_a = tier.eq_ignore_ascii_case("a");

    let (b_str, _is_random) = match blinding {
        Some(b) => (b.to_string(), false),
        None => (generate_blinding(), true),
    };
    let b_big = BigInt::from_str(&b_str).map_err(|e| format!("invalid blinding: {}", e))?;

    let mut cts = Vec::with_capacity(text.len());
    for byte in text.as_bytes() {
        let m = BigInt::from_u8(*byte).ok_or("byte conversion failed")?;
        let ct = if is_tier_a {
            encrypt_tier_a_with_blinding(&m, seed, n, &q, &b_big)
        } else {
            encrypt_with_blinding(&m, seed, n, &q, &b_big)
        }
        .map_err(|e| format!("encrypt error: {}", e))?;
        cts.push(ct.to_string());
    }

    Ok(format!("{}:{}", b_str, cts.join(",")))
}

/// Dekripsi ciphertext. Mendukung format `"BLINDING:CT1,CT2,..."` atau
/// `"CT1,CT2,..."` (jika `blinding` diberikan manual).
pub fn decrypt_text(
    ciphertext: &str,
    key: &KeyRecord,
    tier: &str,
    blinding: Option<&str>,
) -> Result<String, String> {
    let q = BigInt::from_str(&key.q).map_err(|e| format!("invalid Q: {}", e))?;
    let seed = key.seed.as_bytes();
    let n = key.n as usize;
    let is_tier_a = tier.eq_ignore_ascii_case("a");

    let (b_str, ct_part) = match blinding {
        Some(b) => (b.to_string(), ciphertext.to_string()),
        None => {
            let parts: Vec<&str> = ciphertext.split(':').collect();
            if parts.len() != 2 {
                return Err(
                    "format ciphertext tidak valid: butuh --blinding atau format BLINDING:CT..."
                        .into(),
                );
            }
            (parts[0].to_string(), parts[1].to_string())
        }
    };
    let b_big = BigInt::from_str(&b_str).map_err(|e| format!("invalid blinding: {}", e))?;

    let mut bytes = Vec::new();
    for ct_str in ct_part.split(',') {
        let c = BigInt::from_str(ct_str.trim()).map_err(|e| format!("invalid ciphertext: {}", e))?;
        let m = if is_tier_a {
            decrypt_tier_a(&c, seed, n, &q, &b_big)
        } else {
            decrypt(&c, seed, n, &q, &b_big)
        }
        .map_err(|e| format!("decrypt error: {}", e))?;
        let byte = m.to_u8().ok_or("decrypted value bukan byte valid")?;
        bytes.push(byte);
    }

    String::from_utf8(bytes).map_err(|e| format!("hasil dekripsi bukan UTF-8 valid: {}", e))
}

/// Helper untuk `apikey create`: generate API key + hash + prefix.
pub fn generate_api_key() -> (String, String, String) {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);

    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    let plaintext = URL_SAFE_NO_PAD.encode(&bytes); // 43 char

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(plaintext.as_bytes());
    let key_hash = hex::encode(hasher.finalize());

    let prefix = plaintext[..8].to_string();
    (plaintext, key_hash, prefix)
}

/// Helper: hash API key dari header Authorization (untuk middleware).
pub fn hash_api_key(plaintext: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(plaintext.as_bytes());
    hex::encode(hasher.finalize())
}