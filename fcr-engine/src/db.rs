//! Database helpers: koneksi pool, record structs, query.

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct KeyRecord {
    pub seed: String,
    pub n: i32,
    pub q: String,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct ApiKeyRecord {
    pub id: i64,
    pub user_name: String,
    pub key_prefix: String,
}

/// Buat pool koneksi PostgreSQL.
pub async fn connect_pool(db_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
}

/// Ambil key berdasarkan nama. Error jika tidak ditemukan.
pub async fn get_key_by_name(pool: &PgPool, name: &str) -> Result<KeyRecord, sqlx::Error> {
    sqlx::query_as::<_, KeyRecord>("SELECT seed, n, q FROM keys WHERE name = $1")
        .bind(name)
        .fetch_one(pool)
        .await
}

/// Simpan key baru (upsert berdasarkan nama).
pub async fn upsert_key(
    pool: &PgPool,
    name: &str,
    seed: &str,
    n: i32,
    q: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO keys (name, seed, n, q) VALUES ($1, $2, $3, $4)
         ON CONFLICT (name) DO UPDATE SET seed = $2, n = $3, q = $4",
    )
    .bind(name)
    .bind(seed)
    .bind(n)
    .bind(q)
    .execute(pool)
    .await?;
    Ok(())
}

/// List semua key (untuk CLI `list-keys`).
pub async fn list_keys(pool: &PgPool) -> Result<Vec<KeyRecord>, sqlx::Error> {
    sqlx::query_as::<_, KeyRecord>("SELECT seed, n, q FROM keys ORDER BY id")
        .fetch_all(pool)
        .await
}

/// Cari API key aktif berdasarkan hash. Return None jika tidak ada.
pub async fn find_active_api_key_by_hash(
    pool: &PgPool,
    key_hash: &str,
) -> Result<Option<ApiKeyRecord>, sqlx::Error> {
    sqlx::query_as::<_, ApiKeyRecord>(
        "SELECT id, user_name, key_prefix FROM api_keys
         WHERE key_hash = $1 AND is_active = TRUE AND is_locked = FALSE", // <-- Tambah is_locked
    )
    .bind(key_hash)
    .fetch_optional(pool)
    .await
}

/// Update `last_used_at` (fire-and-forget dari middleware).
pub async fn touch_api_key(pool: &PgPool, id: i64) {
    let _ = sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await;
}

/// Simpan API key baru (hash + prefix).
pub async fn insert_api_key(
    pool: &PgPool,
    user_name: &str,
    label: Option<&str>,
    key_prefix: &str,
    key_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO api_keys (user_name, label, key_prefix, key_hash)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(user_name)
    .bind(label)
    .bind(key_prefix)
    .bind(key_hash)
    .execute(pool)
    .await?;
    Ok(())
}

/// List API key (tanpa hash) untuk CLI `apikey list`.
pub async fn list_api_keys(
    pool: &PgPool,
) -> Result<Vec<(i64, String, Option<String>, String, bool)>, sqlx::Error> {
    // Catatan: `last_used_at` (TIMESTAMPTZ) belum disertakan di Fase 10A
    // karena butuh feature `chrono` di sqlx. Akan ditambahkan di Fase 10B.
    sqlx::query_as(
        "SELECT id, user_name, label, key_prefix, is_active FROM api_keys ORDER BY id",
    )
    .fetch_all(pool)
    .await
}

/// Nonaktifkan API key berdasarkan prefix.
pub async fn revoke_api_key_by_prefix(pool: &PgPool, prefix: &str) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("UPDATE api_keys SET is_active = FALSE WHERE key_prefix = $1 AND is_active = TRUE")
        .bind(prefix)
        .execute(pool)
        .await?;
    Ok(res.rows_affected())
}

/// Catat anomali ke tabel anomaly_events.
pub async fn log_anomaly(
    pool: &PgPool,
    api_key_id: i64,
    event_type: &str,
    details: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO anomaly_events (api_key_id, event_type, details) VALUES ($1, $2, $3)",
    )
    .bind(api_key_id)
    .bind(event_type)
    .bind(details)
    .execute(pool)
    .await?;
    Ok(())
}

/// Kunci API key (Kill Switch).
pub async fn lock_api_key(
    pool: &PgPool,
    api_key_id: i64,
    reason: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE api_keys SET is_locked = TRUE, locked_at = NOW(), lock_reason = $1 WHERE id = $2",
    )
    .bind(reason)
    .bind(api_key_id)
    .execute(pool)
    .await?;
    Ok(())
}