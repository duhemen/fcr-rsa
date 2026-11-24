//! Database helpers: koneksi pool, record structs, query.

use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{FromRow, PgPool, Row};

#[derive(Debug, Clone)]
pub struct KeyRecord {
    pub seed: String,
    pub n: i32,
    pub q: String,
}

impl<'r> FromRow<'r, PgRow> for KeyRecord {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            seed: row.try_get("seed")?,
            n: row.try_get("n")?,
            q: row.try_get("q")?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ApiKeyRecord {
    pub id: i64,
    pub user_name: String,
    pub key_prefix: String,
}

impl<'r> FromRow<'r, PgRow> for ApiKeyRecord {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            user_name: row.try_get("user_name")?,
            key_prefix: row.try_get("key_prefix")?,
        })
    }
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
         WHERE key_hash = $1 
           AND is_active = TRUE 
           AND is_locked = FALSE
           AND (lockout_until IS NULL OR lockout_until < NOW())", // <-- Cek Cooldown
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

/// Catat anomali ke tabel anomaly_events (Hardened Version).
pub async fn log_anomaly(
    pool: &PgPool,
    api_key_id: i64,
    event_type: &str,
    details: &str,
    risk_score: u32,
    distance_km: f64,
    trace_id: &str, // <-- TAMBAHKAN PARAMETER INI
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO anomaly_events (api_key_id, event_type, details, risk_score, distance_km, trace_id)
         VALUES ($1, $2, $3, $4, $5, $6)", // <-- TAMBAHKAN $6 DI SINI
    )
    .bind(api_key_id)
    .bind(event_type)
    .bind(details)
    .bind(risk_score as i32)
    .bind(distance_km)
    .bind(trace_id) // <-- SIMPAN TRACE_ID KE DATABASE
    .execute(pool)
    .await?;
    Ok(())
}

// --- FUNGSI BARU UNTUK HARDENING ---

/// Ambil hash lokasi terakhir & timestamp untuk API Key.
pub async fn get_last_state(
    pool: &PgPool,
    api_key_id: i64,
) -> Result<Option<(String, chrono::DateTime<chrono::Utc>)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT last_loc_hash, last_timestamp FROM api_key_states WHERE api_key_id = $1",
    )
    .bind(api_key_id)
    .fetch_optional(pool)
    .await
}

/// Update hash lokasi terakhir & timestamp (UPSERT).
pub async fn update_last_state(
    pool: &PgPool,
    api_key_id: i64,
    loc_hash: &str,
    ts: chrono::DateTime<chrono::Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO api_key_states (api_key_id, last_loc_hash, last_timestamp, updated_at)
         VALUES ($1, $2, $3, NOW())
         ON CONFLICT (api_key_id) DO UPDATE SET 
            last_loc_hash = EXCLUDED.last_loc_hash,
            last_timestamp = EXCLUDED.last_timestamp,
            updated_at = NOW()",
    )
    .bind(api_key_id)
    .bind(loc_hash)
    .bind(ts)
    .execute(pool)
    .await?;
    Ok(())
}

/// Terapkan cooldown (bukan lock permanen).
pub async fn apply_cooldown(
    pool: &PgPool,
    api_key_id: i64,
    duration_minutes: i64,
    reason: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE api_keys 
         SET lockout_until = NOW() + make_interval(mins => $1), 
             lock_reason = $2 
         WHERE id = $3"
    )
    .bind(duration_minutes as i32) // Cast ke i32 karena make_interval butuh INTEGER
    .bind(reason)
    .bind(api_key_id)
    .execute(pool)
    .await?;
    Ok(())
}