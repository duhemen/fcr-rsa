//! Auto-Mune: Deteksi anomali spatiotemporal & manajemen lockdown.

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Header yang dikirim klien untuk verifikasi spatiotemporal.
#[derive(Debug, Deserialize, Clone)]
pub struct SpatiotemporalToken {
    pub gps_lat: f64,
    pub gps_lon: f64,
    pub timestamp: DateTime<Utc>,
    pub latency_ms: f64,
}

/// Hasil pengecekan anomali.
pub struct AnomalyResult {
    pub is_anomaly: bool,
    pub reason: Option<String>,
}

/// Hitung jarak Haversine (dalam km).
fn haversine(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0; // Radius bumi dalam km
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    r * c
}

/// Cek apakah ada anomali antara token sekarang dan token sebelumnya.
pub fn check_anomaly(
    current: &SpatiotemporalToken,
    last: Option<&SpatiotemporalToken>,
) -> AnomalyResult {
    // 1. Latency Anomaly (Selalu dicek, bahkan tanpa last_token)
    if current.latency_ms > 500.0 {
        return AnomalyResult {
            is_anomaly: true,
            reason: Some(format!("latency anomaly: {:.1} ms", current.latency_ms)),
        };
    }

    // 2. Pengecekan yang butuh last_token
    let Some(last) = last else {
        return AnomalyResult { is_anomaly: false, reason: None };
    };

    let dt = (current.timestamp - last.timestamp).num_seconds() as f64;
    if dt <= 0.0 {
        return AnomalyResult {
            is_anomaly: true,
            reason: Some("non-monotonic timestamp".into()),
        };
    }

    // 3. Speed Anomaly (Haversine)
    let dist_km = haversine(last.gps_lat, last.gps_lon, current.gps_lat, current.gps_lon);
    let speed_kmh = dist_km / (dt / 3600.0);
    if speed_kmh > 900.0 { // Melebihi kecepatan pesawat komersial
        return AnomalyResult {
            is_anomaly: true,
            reason: Some(format!("speed anomaly: {:.1} km/h", speed_kmh)),
        };
    }

    AnomalyResult { is_anomaly: false, reason: None }
}
