//! Middleware autentikasi API key (Bearer token).

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::api::AppState;
use crate::crypto::hash_api_key;
use crate::db;

/// Info user yang di-inject ke request extensions setelah auth sukses.
#[derive(Clone, Debug)]
pub struct AuthInfo {
    pub user_name: String,
    pub key_prefix: String,
}

/// Middleware: validasi `Authorization: Bearer <key>`.
///
/// Alur:
/// 1. Ambil header Authorization.
/// 2. Hash dengan SHA-256.
/// 3. Lookup di `api_keys` (aktif).
/// 4. Inject `AuthInfo` ke extensions.
/// 5. Update `last_used_at` (fire-and-forget).
pub async fn require_api_key(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    // 1. Ambil header
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

    // 2. Hash
    let key_hash = hash_api_key(&plaintext_key);

    // 3. Lookup
    let record = match db::find_active_api_key_by_hash(&state.pool, &key_hash).await {
        Ok(Some(r)) => r,
        Ok(None) => return Err(unauthorized("invalid or inactive API key")),
        Err(e) => {
            eprintln!("[auth] DB error: {}", e);
            return Err(internal_error("database error during auth"));
        }
    };

    // 4. Update last_used_at (fire-and-forget, tidak blokir request)
    let pool = state.pool.clone();
    let id = record.id;
    tokio::spawn(async move {
        db::touch_api_key(&pool, id).await;
    });

    // 5. Inject AuthInfo
    request.extensions_mut().insert(AuthInfo {
        user_name: record.user_name,
        key_prefix: record.key_prefix,
    });

    // 6. Lanjut ke handler
    Ok(next.run(request).await)
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