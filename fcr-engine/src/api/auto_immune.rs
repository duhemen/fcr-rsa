//! Auto-Mune: Deteksi anomali spatiotemporal berbasis Privacy-Preserving Location Hash.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize, Clone)]
pub struct SpatiotemporalToken {
    pub gps_lat: f64,
    pub gps_lon: f64,
    pub timestamp: DateTime<Utc>,
    pub latency_ms: f64,
}

#[derive(Debug, Clone)]
pub struct LastLocationState {
    pub loc_hash: String,
    pub timestamp: DateTime<Utc>,
}

pub struct AnomalyResult {
    pub risk_score: u32,
    pub distance_km: f64,
    pub current_loc_hash: String,
    pub reason: Option<String>,
}

/// Kuantisasi koordinat ke grid ~11km (1 desimal) lalu hash dengan SHA-256 + api_key_id.
/// Mematuhi UU PDP: tidak ada koordinat mentah yang keluar dari memori.
pub fn hash_location_grid(api_key_id: i64, lat: f64, lon: f64) -> String {
    let grid_str = format!("{}:{:.1},{:.1}", api_key_id, lat, lon);
    let mut hasher = Sha256::new();
    hasher.update(grid_str.as_bytes());
    hex::encode(hasher.finalize())
}

/// Evaluasi token dan hitung Risk Score (0-100) tanpa menyimpan koordinat mentah.
pub fn evaluate_risk(
    api_key_id: i64,
    current: &SpatiotemporalToken,
    last: Option<&LastLocationState>,
) -> AnomalyResult {
    let mut score = 0;
    let mut reasons = Vec::new();
    let mut estimated_distance_km = 0.0;

    let current_loc_hash = hash_location_grid(api_key_id, current.gps_lat, current.gps_lon);

    // 1. Cek Latency (Hardened)
    if current.latency_ms > 1500.0 {
        score += 80;
        reasons.push(format!("extreme latency anomaly: {:.1} ms", current.latency_ms));
    } else if current.latency_ms > 800.0 {
        score += 20;
    }

    // 2. Cek Perubahan Zona Lokasi (Hash Comparison) & Timestamp
    if let Some(last) = last {
        let dt = (current.timestamp - last.timestamp).num_seconds() as f64;

        if dt < -300.0 {
            score += 30;
            reasons.push(format!("timestamp anomaly: {:.0}s drift", dt));
        } else if dt >= 0.0 {
            // Jika hash grid berubah dalam waktu singkat (< 30 menit / 1800 detik)
            // berarti terjadi perpindahan antar-zona (> ~11 km) secara tidak wajar
            if !last.loc_hash.is_empty() && current_loc_hash != last.loc_hash {
                estimated_distance_km = 15.0; // Estimasi batas bawah lintas grid
                if dt < 60.0 {
                    score += 55;
                    reasons.push(format!("impossible zone jump in {:.0}s", dt));
                } else if dt < 1800.0 {
                    score += 30;
                    reasons.push(format!("rapid zone transition in {:.0}s", dt));
                }
            }
        }
    } else {
        score += 5;
    }

    AnomalyResult {
        risk_score: score.min(100),
        distance_km: estimated_distance_km,
        current_loc_hash,
        reason: if reasons.is_empty() {
            None
        } else {
            Some(reasons.join("; "))
        },
    }
}
