//! Middleware autentikasi API key (Bearer token) + Auto-Mune Check.

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

/// Info user yang di-inject ke request extensions setelah auth sukses.
#[derive(Clone, Debug)]
pub struct AuthInfo {
    pub user_name: String,
    pub key_prefix: String,
}

/// Middleware: validasi `Authorization: Bearer <key>` + deteksi anomali.
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

    // ============================================================
    // 3. AUTO-MUNE CHECK (Deteksi Anomali Spatiotemporal)
    // ============================================================
    if let Some(token) = extract_spatiotemporal(request.headers()) {
        // Cek apakah ada anomali (contoh: latency > 500ms)
        let anomaly = crate::api::auto_immune::check_anomaly(&token, None);

        if anomaly.is_anomaly {
            let reason = anomaly.reason.unwrap_or_else(|| "unknown anomaly".into());

            // A. Log ke tabel anomaly_events
            let _ = db::log_anomaly(&state.pool, record.id, "auto_mune", &reason).await;

            // B. Kill Switch: Lock API key di DB
            let _ = db::lock_api_key(&state.pool, record.id, &reason).await;

            eprintln!(
                "🚨 [AUTO-MUNE] Lockdown key {} (user={}). Reason: {}",
                record.key_prefix, record.user_name, reason
            );

            // C. Honeypot Response (Berpura-pura sukses)
            return Ok(honeypot_response());
        }
    }

    // ============================================================
    // 4. Update last_used_at (fire-and-forget)
    // ============================================================
    let pool = state.pool.clone();
    let id = record.id;
    tokio::spawn(async move {
        db::touch_api_key(&pool, id).await;
    });

    // 5. Inject AuthInfo ke extensions
    request.extensions_mut().insert(AuthInfo {
        user_name: record.user_name,
        key_prefix: record.key_prefix,
    });

    // 6. Lanjut ke handler (next.run DI PALING BAWAH)
    Ok(next.run(request).await)
}

// ============================================================
// Helper: Extract Spatiotemporal dari Headers
// ============================================================
fn extract_spatiotemporal(headers: &HeaderMap) -> Option<SpatiotemporalToken> {
    let lat = headers.get("X-FCR-GPS-Lat")?.to_str().ok()?.parse().ok()?;
    let lon = headers.get("X-FCR-GPS-Lon")?.to_str().ok()?.parse().ok()?;
    let ts_str = headers.get("X-FCR-Timestamp")?.to_str().ok()?;
    let ts = DateTime::parse_from_rfc3339(ts_str).ok()?.with_timezone(&Utc);
    let latency = headers.get("X-FCR-Latency")?.to_str().ok()?.parse().ok()?;

    Some(SpatiotemporalToken {
        gps_lat: lat,
        gps_lon: lon,
        timestamp: ts,
        latency_ms: latency,
    })
}

// ============================================================
// Helper: Honeypot & Error Responses
// ============================================================
fn honeypot_response() -> Response {
    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "message": "Operation completed",
            "data": "HONEYPOT_TRAP_DATA"
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