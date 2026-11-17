//! REST API module: router, AppState, server bootstrap.

pub mod auth;
pub mod handlers;

use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// State yang dibagikan ke seluruh handler & middleware.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    /// Port server (untuk logging & info).
    pub port: u16,
}

/// Build router lengkap dengan middleware.
pub fn build_router(state: AppState) -> Router {
    // Route publik (tanpa auth) — hanya health check.
    let public = Router::new().route("/health", get(health));

    // Route terproteksi (butuh API key).
    let protected = Router::new()
        .route("/keygen", post(handlers::keygen))
        .route("/encrypt", post(handlers::encrypt))
        .route("/decrypt", post(handlers::decrypt))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_api_key,
        ));

    // Gabungkan + middleware global.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(public)
        .merge(protected)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

/// Handler health check.
async fn health(State(state): State<AppState>) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "service": "fcr-engine",
        "version": env!("CARGO_PKG_VERSION"),
        "port": state.port,
    }))
}

/// Jalankan server. Blocking sampai dihentikan (Ctrl+C).
pub async fn serve(pool: PgPool, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState { pool, port };
    let app = build_router(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("🚀 FCR-RSA API server listening on http://{}", addr);
    println!("   Health check: http://{}/health", addr);
    println!("   Tekan Ctrl+C untuk berhenti.");

    axum::serve(listener, app).await?;
    Ok(())
}