//! Handler REST API: /keygen, /encrypt, /decrypt.

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::api::AppState;
use crate::api::auth::AuthInfo;
use crate::crypto::{decrypt_text, encrypt_text};
use crate::db;

// ============================================================
// /keygen
// ============================================================

#[derive(Deserialize)]
pub struct KeygenRequest {
    pub name: String,
    #[serde(default = "default_tier")]
    pub tier: String,
    #[serde(default = "default_seed")]
    pub seed: String,
    #[serde(default = "default_n")]
    pub n: i32,
    #[serde(default = "default_q")]
    pub q: String,
}

fn default_tier() -> String { "b".to_string() }
fn default_seed() -> String { "fcr-rsa-demo-seed".to_string() }
fn default_n() -> i32 { 10 }
fn default_q() -> String { "1000000007".to_string() }

#[derive(Serialize)]
pub struct KeygenResponse {
    pub name: String,
    pub tier: String,
    /// base64 dari string parameter publik (q + n + seed hash).
    /// Untuk Fase 10A, kita kirim `q` & `n` sebagai base64 JSON string.
    pub public_key_b64: String,
}

pub async fn keygen(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthInfo>,
    Json(req): Json<KeygenRequest>,
) -> Response {
    // Validasi
    if req.n < 4 {
        return bad_request("n must be >= 4");
    }
    if req.name.trim().is_empty() {
        return bad_request("name tidak boleh kosong");
    }

    // Simpan ke DB
    if let Err(e) = db::upsert_key(&state.pool, &req.name, &req.seed, req.n, &req.q).await {
        eprintln!("[keygen] DB error: {}", e);
        return internal_error("gagal menyimpan key");
    }

    // Bangun "public key" — untuk Fase 10A, kita serialize {q, n, seed} sebagai JSON, lalu base64.
    // (Di Fase 11, kita bisa ekspos parameter publik yang lebih formal.)
    let pk_json = json!({
        "q": req.q,
        "n": req.n,
        "seed_hint": format!("{:x}", simple_hash(&req.seed)),
    });
    let pk_b64 = B64.encode(pk_json.to_string().as_bytes());

    println!(
        "[keygen] user={} key_prefix={} created key '{}' (tier {})",
        auth.user_name, auth.key_prefix, req.name, req.tier
    );

    (
        StatusCode::CREATED,
        Json(KeygenResponse {
            name: req.name,
            tier: req.tier,
            public_key_b64: pk_b64,
        }),
    )
        .into_response()
}

fn simple_hash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

// ============================================================
// /encrypt
// ============================================================

#[derive(Deserialize)]
pub struct EncryptRequest {
    pub key_name: String,
    /// Plaintext dalam base64 (UTF-8 bytes).
    pub text_b64: String,
    #[serde(default = "default_tier")]
    pub tier: String,
    /// Opsional: jika diisi, gunakan deterministik (untuk test vector).
    pub blinding: Option<String>,
}

#[derive(Serialize)]
pub struct EncryptResponse {
    /// Ciphertext format `BLINDING:CT1,CT2,...` dalam base64.
    pub ciphertext_b64: String,
    /// Jumlah blok (byte) yang dienkripsi.
    pub blocks: usize,
}

pub async fn encrypt(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthInfo>,
    Json(req): Json<EncryptRequest>,
) -> Response {
    // Decode base64 plaintext
    let text_bytes = match B64.decode(&req.text_b64) {
        Ok(b) => b,
        Err(_) => return bad_request("text_b64 bukan base64 valid"),
    };
    let text = match String::from_utf8(text_bytes) {
        Ok(s) => s,
        Err(_) => return bad_request("text_b64 bukan UTF-8 valid"),
    };

    // Ambil key dari DB
    let key = match db::get_key_by_name(&state.pool, &req.key_name).await {
        Ok(k) => k,
        Err(sqlx::Error::RowNotFound) => {
            return not_found(format!("key '{}' tidak ditemukan", req.key_name))
        }
        Err(e) => {
            eprintln!("[encrypt] DB error: {}", e);
            return internal_error("database error");
        }
    };

    // Enkripsi
    let blocks = text.len();
    let ct = match encrypt_text(&text, &key, &req.tier, req.blinding.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[encrypt] crypto error: {}", e);
            return bad_request(&format!("encryption failed: {}", e));
        }
    };

    println!(
        "[encrypt] user={} key_prefix={} key='{}' blocks={} tier={}",
        auth.user_name, auth.key_prefix, req.key_name, blocks, req.tier
    );

    Json(EncryptResponse {
        ciphertext_b64: B64.encode(ct.as_bytes()),
        blocks,
    })
    .into_response()
}

// ============================================================
// /decrypt
// ============================================================

#[derive(Deserialize)]
pub struct DecryptRequest {
    pub key_name: String,
    /// Ciphertext format `BLINDING:CT1,CT2,...` dalam base64.
    pub ciphertext_b64: String,
    #[serde(default = "default_tier")]
    pub tier: String,
    /// Opsional: hanya perlu jika ciphertext tidak menyertakan blinding.
    pub blinding: Option<String>,
}

#[derive(Serialize)]
pub struct DecryptResponse {
    /// Plaintext dalam base64.
    pub text_b64: String,
}

pub async fn decrypt(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthInfo>,
    Json(req): Json<DecryptRequest>,
) -> Response {
    // Decode base64 ciphertext
    let ct_bytes = match B64.decode(&req.ciphertext_b64) {
        Ok(b) => b,
        Err(_) => return bad_request("ciphertext_b64 bukan base64 valid"),
    };
    let ct_str = match String::from_utf8(ct_bytes) {
        Ok(s) => s,
        Err(_) => return bad_request("ciphertext_b64 bukan UTF-8 valid"),
    };

    // Ambil key
    let key = match db::get_key_by_name(&state.pool, &req.key_name).await {
        Ok(k) => k,
        Err(sqlx::Error::RowNotFound) => {
            return not_found(format!("key '{}' tidak ditemukan", req.key_name))
        }
        Err(e) => {
            eprintln!("[decrypt] DB error: {}", e);
            return internal_error("database error");
        }
    };

    // Dekripsi
    let pt = match decrypt_text(&ct_str, &key, &req.tier, req.blinding.as_deref()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[decrypt] crypto error: {}", e);
            return bad_request(&format!("decryption failed: {}", e));
        }
    };

    println!(
        "[decrypt] user={} key_prefix={} key='{}' tier={}",
        auth.user_name, auth.key_prefix, req.key_name, req.tier
    );

    Json(DecryptResponse {
        text_b64: B64.encode(pt.as_bytes()),
    })
    .into_response()
}

// ============================================================
// Error helpers
// ============================================================

fn bad_request(msg: &str) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": "bad_request", "message": msg })),
    )
        .into_response()
}

fn not_found(msg: String) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": "not_found", "message": msg })),
    )
        .into_response()
}

fn internal_error(msg: &str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": "internal_error", "message": msg })),
    )
        .into_response()
}