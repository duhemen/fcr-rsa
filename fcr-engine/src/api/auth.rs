//! Middleware autentikasi API key (Bearer token) + Auto-Mune Check (Hardened).

use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde_json::json;

use crate::api::auto_immune::SpatiotemporalToken;
use crate::api::AppState;
use crate::crypto::hash_api_key;
use crate::db;

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Info user yang di-inject ke request extensions setelah auth sukses.
#[derive(Clone, Debug)]
pub struct AuthInfo {
    pub user_name: String,
    pub key_prefix: String,
}

/// Middleware: validasi `Authorization: Bearer <key>` + wajib header spatiotemporal + deteksi anomali.
pub async fn require_api_key(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    // 1. Ambil header Authorization
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.trim().to_string());

    let Some(plaintext_key) = auth_header else {
        return Err(unauthorized("missing Authorization: Bearer <key>"));
    };

    if plaintext_key.is_empty() {
        return Err(unauthorized("empty API key"));
    }

    // 2. Hash & Lookup DB (otomatis menolak jika sedang masa cooldown)
    let key_hash = hash_api_key(&plaintext_key);

    let record = match db::find_active_api_key_by_hash(&state.pool, &key_hash).await {
        Ok(Some(r)) => r,
        Ok(None) => return Err(unauthorized("invalid, inactive, or locked API key")),
        Err(e) => {
            eprintln!("[auth] DB error: {}", e);
            return Err(internal_error("database error during auth"));
        }
    };

    // ============================================================
    // 3. WAJIBKAN HEADER SPATIOTEMPORAL + VALIDASI HMAC-SHA256
    // ============================================================
    let token = match extract_and_verify_spatiotemporal(request.headers(), &plaintext_key) {
        Ok(t) => t,
        Err((is_security_violation, err_msg)) => {
            eprintln!(
                "🛡️ [AUTO-MUNE] Telemetry rejected for {} ({})",
                record.key_prefix, err_msg
            );
            if is_security_violation {
                return Err(unauthorized(&format!("Security verification failed: {}", err_msg)));
            } else {
                return Err(bad_request(&format!("Malformed telemetry header: {}", err_msg)));
            }
        }
    };

    // ============================================================
    // 4. AUTO-MUNE CHECK (Privacy-Preserving Risk Scoring)
    // ============================================================
    let last_state = db::get_last_state(&state.pool, record.id)
        .await
        .ok()
        .flatten()
        .map(|(loc_hash, ts)| crate::api::auto_immune::LastLocationState {
            loc_hash,
            timestamp: ts,
        });

    let anomaly = crate::api::auto_immune::evaluate_risk(record.id, &token, last_state.as_ref());

    // Simpan HANYA hash lokasi (tanpa koordinat mentah) untuk request berikutnya
    let _ = db::update_last_state(
        &state.pool,
        record.id,
        &anomaly.current_loc_hash,
        token.timestamp,
    )
    .await;


    // Jika Risk Score >= 70, terapkan Cooldown & balas dengan Honeypot tersamar
    if anomaly.risk_score >= 70 {
        let reason = anomaly.reason.unwrap_or_else(|| "high risk score".into());

        let _ = db::log_anomaly(
            &state.pool,
            record.id,
            "auto_mune_hardened",
            &reason,
            anomaly.risk_score,
            anomaly.distance_km,
        )
        .await;

        if let Err(e) = db::apply_cooldown(&state.pool, record.id, 5, &reason).await {
            eprintln!("❌ [DB ERROR] Gagal apply cooldown: {}", e);
        }

        eprintln!(
            "🚨 [AUTO-MUNE] Cooldown applied to {} (user={}). Score: {}. Reason: {}",
            record.key_prefix, record.user_name, anomaly.risk_score, reason
        );

        // Kembalikan Honeypot yang menyerupai ciphertext asli
        return Ok(honeypot_response());
    }

    if anomaly.risk_score >= 40 {
        eprintln!(
            "⚠️ [AUTO-MUNE] Suspicious activity from {}. Score: {}",
            record.key_prefix, anomaly.risk_score
        );
    }

    // 5. Update last_used_at (fire-and-forget)
    let pool = state.pool.clone();
    let id = record.id;
    tokio::spawn(async move {
        db::touch_api_key(&pool, id).await;
    });

    // 6. Inject AuthInfo ke extensions
    request.extensions_mut().insert(AuthInfo {
        user_name: record.user_name,
        key_prefix: record.key_prefix,
    });

    Ok(next.run(request).await)
}

// ============================================================
// Helper: Extract Spatiotemporal (Mengembalikan pesan error spesifik)
// ============================================================
fn extract_and_verify_spatiotemporal(
    headers: &HeaderMap,
    secret_key: &str,
) -> Result<SpatiotemporalToken, (bool, &'static str)> {
    // 1. Ambil string mentah (400 Bad Request jika hilang/tidak valid)
    let lat_str = headers
        .get("X-FCR-GPS-Lat")
        .ok_or((false, "missing X-FCR-GPS-Lat"))?
        .to_str()
        .map_err(|_| (false, "invalid X-FCR-GPS-Lat encoding"))?;

    let lon_str = headers
        .get("X-FCR-GPS-Lon")
        .ok_or((false, "missing X-FCR-GPS-Lon"))?
        .to_str()
        .map_err(|_| (false, "invalid X-FCR-GPS-Lon encoding"))?;

    let ts_str = headers
        .get("X-FCR-Timestamp")
        .ok_or((false, "missing X-FCR-Timestamp"))?
        .to_str()
        .map_err(|_| (false, "invalid X-FCR-Timestamp encoding"))?;

    let latency_str = headers
        .get("X-FCR-Latency")
        .ok_or((false, "missing X-FCR-Latency"))?
        .to_str()
        .map_err(|_| (false, "invalid X-FCR-Latency encoding"))?;

    let sig_hex = headers
        .get("X-FCR-Signature")
        .ok_or((false, "missing X-FCR-Signature"))?
        .to_str()
        .map_err(|_| (false, "invalid X-FCR-Signature encoding"))?;

    // 2. Susun canonical payload & hitung HMAC-SHA256
    let canonical_payload = format!("{}:{}:{}:{}", lat_str, lon_str, ts_str, latency_str);

    let sig_bytes = hex::decode(sig_hex.trim())
        .map_err(|_| (false, "X-FCR-Signature must be valid hex"))?;

    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .map_err(|_| (true, "invalid key length for HMAC"))?;
    mac.update(canonical_payload.as_bytes());

    // Verifikasi constant-time (401 Unauthorized jika tanda tangan dipalsukan)
    mac.verify_slice(&sig_bytes)
        .map_err(|_| (true, "invalid X-FCR-Signature (possible header spoofing)"))?;

    // 3. Parse nilai (400 Bad Request jika format angka/tanggal salah)
    let lat: f64 = lat_str.parse().map_err(|_| (false, "X-FCR-GPS-Lat must be a float"))?;
    let lon: f64 = lon_str.parse().map_err(|_| (false, "X-FCR-GPS-Lon must be a float"))?;
    let ts = DateTime::parse_from_rfc3339(ts_str)
        .map_err(|_| (false, "X-FCR-Timestamp must be RFC3339 format"))?
        .with_timezone(&Utc);
    let latency: f64 = latency_str.parse().map_err(|_| (false, "X-FCR-Latency must be a float"))?;

    Ok(SpatiotemporalToken {
        gps_lat: lat,
        gps_lon: lon,
        timestamp: ts,
        latency_ms: latency,
    })
}

// ============================================================
// Helper: Deceptive Honeypot & Error Responses
// ============================================================
fn honeypot_response() -> Response {
    // Membuat deretan angka acak pseudo-deterministik berbasis nanosecond waktu saat ini
    // Format meniru persis output asli FCR-RSA: "seed:c1,c2,c3,..." yang di-encode Base64
    let nanos = Utc::now().timestamp_subsec_nanos() as u64;
    let mut state = nanos ^ 0x5DEECE66D;
    let mut nums = Vec::with_capacity(10);

    for _ in 0..10 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        nums.push(((state >> 32) % 900_000_000 + 100_000_000).to_string());
    }

    let seed_fake = (nanos % 900_000_000) + 100_000_000;
    let raw_payload = format!("{}:{}", seed_fake, nums.join(","));
    let fake_b64 = base64_encode_simple(raw_payload.as_bytes());

    (
        StatusCode::OK,
        Json(json!({
            "ciphertext_b64": fake_b64,
            "blocks": 10
        })),
    )
        .into_response()
}

/// Encoder Base64 ringan tanpa perlu menambah crate baru di Cargo.toml.
fn base64_encode_simple(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        out.push(if chunk.len() > 1 { CHARS[((n >> 6) & 0x3F) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { CHARS[(n & 0x3F) as usize] as char } else { '=' });
    }
    out
}

fn bad_request(msg: &str) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "error": "bad_request",
            "message": msg,
        })),
    )
        .into_response()
}

fn unauthorized(msg: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": "unauthorized",
            "message": msg,
        })),
    )
        .into_response()
}

fn internal_error(msg: &str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": "internal_error",
            "message": msg,
        })),
    )
        .into_response()
}
