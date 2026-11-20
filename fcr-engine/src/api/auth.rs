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

#[derive(Clone, Debug)]
pub struct AuthInfo {
    pub user_name: String,
    pub key_prefix: String,
    pub risk_score: u32,
    pub risk_reason: Option<String>,
}

#[derive(Clone, Debug)]
pub struct HoneypotContext {
    pub trace_id: String,
    pub reason: String,
}

pub async fn require_api_key(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    // 1. Header Authorization
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

    // 2. Hash & Lookup DB
    let key_hash = hash_api_key(&plaintext_key);
    let record = match db::find_active_api_key_by_hash(&state.pool, &key_hash).await {
        Ok(Some(r)) => r,
        Ok(None) => return Err(unauthorized("invalid, inactive, or locked API key")),
        Err(e) => {
            eprintln!("[auth] DB error: {}", e);
            return Err(internal_error("database error during auth"));
        }
    };

    // 3. WAJIBKAN HEADER SPATIOTEMPORAL + VALIDASI HMAC-SHA256
    let token = match extract_and_verify_spatiotemporal(request.headers(), &plaintext_key) {
        Ok(t) => t,
        Err((is_security_violation, err_msg)) => {
            eprintln!("🛡️ [AUTO-MUNE] Telemetry rejected for {} ({})", record.key_prefix, err_msg);
            if is_security_violation {
                return Err(unauthorized(&format!("Security verification failed: {}", err_msg)));
            } else {
                return Err(bad_request(&format!("Malformed telemetry header: {}", err_msg)));
            }
        }
    };

    // 4. AUTO-MUNE CHECK
    let last_state = db::get_last_state(&state.pool, record.id)
        .await
        .ok()
        .flatten()
        .map(|(loc_hash, ts)| crate::api::auto_immune::LastLocationState { loc_hash, timestamp: ts });

    let anomaly = crate::api::auto_immune::evaluate_risk(record.id, &token, last_state.as_ref());
    let risk_reason = anomaly.reason.clone(); 

    let _ = db::update_last_state(&state.pool, record.id, &anomaly.current_loc_hash, token.timestamp).await;

    // Risk Score >= 70 → Cooldown + SUNTIKKAN HoneypotContext (bukan short-circuit!)
    if anomaly.risk_score >= 70 {
        let reason = anomaly.reason.clone().unwrap_or_else(|| "high risk score".into());
        let trace_id = format!("TRC-{}", chrono::Utc::now().timestamp_millis());
        
        // 1. Simpan ke Database DENGAN trace_id
        let _ = db::log_anomaly(
            &state.pool,
            record.id,
            "auto_mune_hardened",
            &reason,
            anomaly.risk_score,
            anomaly.distance_km,
            &trace_id,
        ).await;
        
        // 2. Terapkan Cooldown
        if let Err(e) = db::apply_cooldown(&state.pool, record.id, 5, &reason).await {
            eprintln!("❌ [DB ERROR] Gagal apply cooldown: {}", e);
        }

        // 3. 🚨 ACTIVE AUTOMATED INCIDENT REPORT (Laporan Otomatis Aktif)
        eprintln!("\n============================================================");
        eprintln!("🚨 [SECURITY INCIDENT REPORT] AUTO-GENERATED 🚨");
        eprintln!("============================================================");
        eprintln!("⏰ WHEN       : {}", chrono::Utc::now().to_rfc3339());
        eprintln!("👤 WHO        : User '{}', Key Prefix '{}'", record.user_name, record.key_prefix);
        eprintln!("🎯 WHAT       : {}", reason);
        eprintln!("⚠️  RISK SCORE : {}/100", anomaly.risk_score);
        eprintln!("🔗 TRACE ID   : {}", trace_id);
        eprintln!("🛡️  ACTION     : Cooldown applied (5 mins) + Honeypot served");
        eprintln!("============================================================\n");

        // 4. Suntikkan Context ke Request untuk Handler
        request.extensions_mut().insert(HoneypotContext {
            trace_id,
            reason,
        });
    }

    if anomaly.risk_score >= 40 && anomaly.risk_score < 70 {
        eprintln!("⚠️ [AUTO-MUNE] Suspicious activity from {}. Score: {}", record.key_prefix, anomaly.risk_score);
    }

    // 5. Update last_used_at (fire-and-forget)
    let pool = state.pool.clone();
    let id = record.id;
    tokio::spawn(async move {
        db::touch_api_key(&pool, id).await;
    });

    // 6. Inject AuthInfo (DIPERBARUI)
    request.extensions_mut().insert(AuthInfo {
        user_name: record.user_name,
        key_prefix: record.key_prefix,
        risk_score: anomaly.risk_score,       // ✅ OPER SKOR DARI ANOMALI
        risk_reason: risk_reason,  // ✅ OPER ALASAN DARI ANOMALI
    });
    Ok(next.run(request).await)
}

// ============================================================
// Helper: Extract Spatiotemporal
// ============================================================
fn extract_and_verify_spatiotemporal(
    headers: &HeaderMap,
    secret_key: &str,
) -> Result<SpatiotemporalToken, (bool, &'static str)> {
    let lat_str = headers.get("X-FCR-GPS-Lat")
        .ok_or((false, "missing X-FCR-GPS-Lat"))?
        .to_str().map_err(|_| (false, "invalid X-FCR-GPS-Lat encoding"))?;
    let lon_str = headers.get("X-FCR-GPS-Lon")
        .ok_or((false, "missing X-FCR-GPS-Lon"))?
        .to_str().map_err(|_| (false, "invalid X-FCR-GPS-Lon encoding"))?;
    let ts_str = headers.get("X-FCR-Timestamp")
        .ok_or((false, "missing X-FCR-Timestamp"))?
        .to_str().map_err(|_| (false, "invalid X-FCR-Timestamp encoding"))?;
    let latency_str = headers.get("X-FCR-Latency")
        .ok_or((false, "missing X-FCR-Latency"))?
        .to_str().map_err(|_| (false, "invalid X-FCR-Latency encoding"))?;
    let sig_hex = headers.get("X-FCR-Signature")
        .ok_or((false, "missing X-FCR-Signature"))?
        .to_str().map_err(|_| (false, "invalid X-FCR-Signature encoding"))?;

    let canonical_payload = format!("{}:{}:{}:{}", lat_str, lon_str, ts_str, latency_str);
    let sig_bytes = hex::decode(sig_hex.trim())
        .map_err(|_| (false, "X-FCR-Signature must be valid hex"))?;
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .map_err(|_| (true, "invalid key length for HMAC"))?;
    mac.update(canonical_payload.as_bytes());
    mac.verify_slice(&sig_bytes)
        .map_err(|_| (true, "invalid X-FCR-Signature (possible header spoofing)"))?;

    let lat: f64 = lat_str.parse().map_err(|_| (false, "X-FCR-GPS-Lat must be a float"))?;
    let lon: f64 = lon_str.parse().map_err(|_| (false, "X-FCR-GPS-Lon must be a float"))?;
    let ts = DateTime::parse_from_rfc3339(ts_str)
        .map_err(|_| (false, "X-FCR-Timestamp must be RFC3339 format"))?
        .with_timezone(&Utc);
    let latency: f64 = latency_str.parse().map_err(|_| (false, "X-FCR-Latency must be a float"))?;

    Ok(SpatiotemporalToken { gps_lat: lat, gps_lon: lon, timestamp: ts, latency_ms: latency })
}

// ============================================================
// Error helpers (module level — brace sudah seimbang)
// ============================================================
fn bad_request(msg: &str) -> Response {
    (StatusCode::BAD_REQUEST, Json(json!({ "error": "bad_request", "message": msg }))).into_response()
}
fn unauthorized(msg: &str) -> Response {
    (StatusCode::UNAUTHORIZED, Json(json!({ "error": "unauthorized", "message": msg }))).into_response()
}
fn internal_error(msg: &str) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "internal_error", "message": msg }))).into_response()
}