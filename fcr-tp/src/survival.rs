// src/survival.rs - VERSI REVISI
// Perubahan:
// 1. Ekspor fungsi bebas `detect_link_profile_from_metrics` (untuk anomaly.rs)
// 2. `SurvivalModeController::detect_link_profile` memanggil fungsi bebas (DRY)
// 3. Tambah `LinkProfile::baseline_rtt()` dan `baseline_jitter()` (untuk anomaly.rs)
// 4. Test baru untuk fungsi bebas

use std::collections::VecDeque;

/// Profil karakteristik link jaringan (Heuristik)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LinkProfile {
    Fiber,       // RTT rendah, Jitter sangat rendah
    Cellular,    // RTT sedang/tinggi, Jitter tinggi (radio scheduling)
    Satellite,   // RTT sangat tinggi, Jitter rendah (jalur fisik panjang tapi stabil)
    Unknown,     // Belum teridentifikasi, menggunakan baseline dinamis
}

impl LinkProfile {
    /// Baseline RTT per profil (ms) — untuk AdaptiveThreshold
    pub fn baseline_rtt_ms(&self) -> f64 {
        match self {
            LinkProfile::Fiber => 20.0,
            LinkProfile::Cellular => 80.0,
            LinkProfile::Satellite => 600.0,
            LinkProfile::Unknown => 200.0,
        }
    }

    /// Baseline jitter per profil (ms)
    pub fn baseline_jitter_ms(&self) -> f64 {
        match self {
            LinkProfile::Fiber => 5.0,
            LinkProfile::Cellular => 30.0,
            LinkProfile::Satellite => 20.0,
            LinkProfile::Unknown => 20.0,
        }
    }

    /// Nama profil untuk logging/UI
    pub fn name(&self) -> &'static str {
        match self {
            LinkProfile::Fiber => "Fiber",
            LinkProfile::Cellular => "Cellular",
            LinkProfile::Satellite => "Satellite",
            LinkProfile::Unknown => "Unknown",
        }
    }
}

/// ✅ FUNGSI BEBAS: Deteksi profil link dari metrik.
/// 
/// Ini adalah fungsi murni yang bisa dipanggil dari mana saja
/// (termasuk `anomaly.rs`) tanpa perlu `SurvivalModeController`.
pub fn detect_link_profile_from_metrics(
    rtt_ms: f64,
    jitter_ms: f64,
    _loss_rate: f64,  // untuk masa depan, saat ini belum dipakai
) -> LinkProfile {
    if rtt_ms < 50.0 && jitter_ms < 10.0 {
        LinkProfile::Fiber
    } else if rtt_ms > 200.0 && jitter_ms < 30.0 {
        LinkProfile::Satellite
    } else if jitter_ms > 15.0 {
        LinkProfile::Cellular
    } else {
        LinkProfile::Unknown
    }
}

/// Metrik jaringan real-time
#[derive(Debug, Clone, Copy)]
pub struct NetworkMetrics {
    pub rtt_ms: f64,
    pub jitter_ms: f64,
    pub loss_rate: f64,
    pub throughput_bps: f64,
}

/// Skor kesehatan jaringan (0.0 - 1.0, 1.0 = Sempurna)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NetworkHealthScore {
    pub total: f64,
    pub latency_score: f64,
    pub loss_score: f64,
    pub jitter_score: f64,
}

/// Ukuran bucket yang adaptif
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BucketSize {
    B256,
    B512,
    B1200,
}

impl BucketSize {
    pub fn as_bytes(&self) -> usize {
        match self {
            BucketSize::B256 => 256,
            BucketSize::B512 => 512,
            BucketSize::B1200 => 1200,
        }
    }
}

/// Tier kontrol untuk integrasi dengan Reassembler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlTier {
    Advisor,
    Enforce,
}

/// Kebijakan Survival Mode
#[derive(Debug, Clone, PartialEq)]
pub struct SurvivalPolicy {
    pub tier: ControlTier,
    pub suggested_bucket: BucketSize,
    pub timeout_multiplier: f32,
    pub enable_partial_assembly: bool,
}

/// State untuk transisi bucket
struct TransitionState {
    current_bucket: BucketSize,
    hysteresis_counter: i32,
    last_change_time: std::time::Instant,
}

/// Controller Utama Survival Mode
pub struct SurvivalModeController {
    profile: LinkProfile,
    metrics_history: VecDeque<NetworkMetrics>,
    transition: TransitionState,
    max_history: usize,
    last_score: NetworkHealthScore,
    last_emergency: bool,
}

impl SurvivalModeController {
    pub const HYSTERESIS_THRESHOLD: i32 = 3;
    pub const CRITICAL_SCORE_THRESHOLD: f64 = 0.2;
    pub const POOR_SCORE_THRESHOLD: f64 = 0.5;
    pub const GOOD_SCORE_THRESHOLD: f64 = 0.8;

    pub fn new() -> Self {
        Self {
            profile: LinkProfile::Unknown,
            metrics_history: VecDeque::with_capacity(20),
            transition: TransitionState {
                current_bucket: BucketSize::B1200,
                hysteresis_counter: 0,
                last_change_time: std::time::Instant::now(),
            },
            max_history: 20,
            last_score: NetworkHealthScore { total: 1.0, latency_score: 1.0, loss_score: 1.0, jitter_score: 1.0 },
            last_emergency: false,
        }
    }

    /// Deteksi profil link berdasarkan metrik awal.
    /// 
    /// ✅ REVISI: sekarang memanggil fungsi bebas (DRY).
    pub fn detect_link_profile(&mut self, metrics: &NetworkMetrics) -> LinkProfile {
        self.profile = detect_link_profile_from_metrics(
            metrics.rtt_ms,
            metrics.jitter_ms,
            metrics.loss_rate,
        );
        self.profile
    }

    /// Dapatkan profil saat ini (tanpa mengubah)
    pub fn current_profile(&self) -> LinkProfile {
        self.profile
    }

    pub fn update_metrics(&mut self, metrics: NetworkMetrics) -> SurvivalPolicy {
        // FIX: NaN / loss di luar [0,1] sebelumnya meracuni health score (NaN menyebar ke semua perbandingan)
        if !metrics.rtt_ms.is_finite() || !metrics.jitter_ms.is_finite() || !metrics.loss_rate.is_finite() || !metrics.throughput_bps.is_finite() {
            return self.current_policy();
        }
        let metrics = NetworkMetrics { loss_rate: metrics.loss_rate.clamp(0.0, 1.0), ..metrics };
        if self.metrics_history.len() >= self.max_history {
            self.metrics_history.pop_front();
        }
        self.metrics_history.push_back(metrics);

        let score = self.calculate_health_score(&metrics);
        let (new_bucket, is_emergency) = self.determine_bucket_size(&score);
        self.last_score = score;
        self.last_emergency = is_emergency;

        if new_bucket != self.transition.current_bucket {
            if is_emergency {
                self.transition.current_bucket = new_bucket;
                self.transition.hysteresis_counter = 0;
            } else {
                if new_bucket.as_bytes() < self.transition.current_bucket.as_bytes() {
                    self.transition.hysteresis_counter -= 1;
                } else {
                    self.transition.hysteresis_counter += 1;
                }

                if self.transition.hysteresis_counter.abs() >= Self::HYSTERESIS_THRESHOLD {
                    self.transition.current_bucket = new_bucket;
                    self.transition.hysteresis_counter = 0;
                }
            }
            self.transition.last_change_time = std::time::Instant::now();
        }

        self.generate_policy(score, is_emergency)
    }

    fn calculate_health_score(&self, metrics: &NetworkMetrics) -> NetworkHealthScore {
        let (rtt_baseline, jitter_baseline) = match self.profile {
            LinkProfile::Fiber => (50.0, 10.0),
            LinkProfile::Cellular => (150.0, 30.0),
            LinkProfile::Satellite => (600.0, 20.0),
            LinkProfile::Unknown => (200.0, 20.0),
        };

        let latency_score = (1.0 - (metrics.rtt_ms / (rtt_baseline * 3.0)).min(1.0)).max(0.0);
        let jitter_score = (1.0 - (metrics.jitter_ms / (jitter_baseline * 3.0)).min(1.0)).max(0.0);
        let loss_score = (1.0 - metrics.loss_rate).max(0.0);

        let total = (loss_score * 0.5) + (latency_score * 0.3) + (jitter_score * 0.2);

        NetworkHealthScore {
            total,
            latency_score,
            loss_score,
            jitter_score,
        }
    }

    fn determine_bucket_size(&self, score: &NetworkHealthScore) -> (BucketSize, bool) {
        if score.total < Self::CRITICAL_SCORE_THRESHOLD {
            return (BucketSize::B256, true);
        }

        let target = if score.total < Self::POOR_SCORE_THRESHOLD {
            BucketSize::B256
        } else if score.total < Self::GOOD_SCORE_THRESHOLD {
            BucketSize::B512
        } else {
            BucketSize::B1200
        };

        (target, false)
    }

    fn generate_policy(&self, score: NetworkHealthScore, is_emergency: bool) -> SurvivalPolicy {
        let tier = if is_emergency || score.total < Self::POOR_SCORE_THRESHOLD {
            ControlTier::Enforce
        } else {
            ControlTier::Advisor
        };

        let timeout_multiplier = if score.total < Self::CRITICAL_SCORE_THRESHOLD {
            5.0
        } else if score.total < Self::POOR_SCORE_THRESHOLD {
            2.5
        } else {
            1.0
        };

        SurvivalPolicy {
            tier,
            suggested_bucket: self.transition.current_bucket,
            timeout_multiplier,
            enable_partial_assembly: score.total < Self::POOR_SCORE_THRESHOLD,
        }
    }

    pub fn current_policy(&self) -> SurvivalPolicy {
        self.generate_policy(self.last_score, self.last_emergency)
    }
}

impl Default for SurvivalModeController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_metrics(rtt: f64, jitter: f64, loss: f64) -> NetworkMetrics {
        NetworkMetrics {
            rtt_ms: rtt,
            jitter_ms: jitter,
            loss_rate: loss,
            throughput_bps: 1_000_000.0,
        }
    }

    #[test]
    fn test_link_profiling_fiber() {
        let mut controller = SurvivalModeController::new();
        let profile = controller.detect_link_profile(&make_metrics(10.0, 2.0, 0.0));
        assert_eq!(profile, LinkProfile::Fiber);
    }

    #[test]
    fn test_link_profiling_satellite() {
        let mut controller = SurvivalModeController::new();
        let profile = controller.detect_link_profile(&make_metrics(600.0, 10.0, 0.0));
        assert_eq!(profile, LinkProfile::Satellite);
    }

    #[test]
    fn test_link_profiling_cellular() {
        let mut controller = SurvivalModeController::new();
        let profile = controller.detect_link_profile(&make_metrics(100.0, 25.0, 0.05));
        assert_eq!(profile, LinkProfile::Cellular);
    }

    #[test]
    fn test_normal_degradation_hysteresis() {
        let mut controller = SurvivalModeController::new();
        controller.detect_link_profile(&make_metrics(10.0, 2.0, 0.0));

        let p1 = controller.update_metrics(make_metrics(200.0, 50.0, 0.3));
        assert_eq!(p1.suggested_bucket, BucketSize::B1200);

        let p2 = controller.update_metrics(make_metrics(200.0, 50.0, 0.3));
        assert_eq!(p2.suggested_bucket, BucketSize::B1200);

        let p3 = controller.update_metrics(make_metrics(200.0, 50.0, 0.3));
        assert_eq!(p3.suggested_bucket, BucketSize::B256);
        assert_eq!(p3.tier, ControlTier::Enforce);
    }

    #[test]
    fn test_emergency_override_bypasses_hysteresis() {
        let mut controller = SurvivalModeController::new();
        controller.detect_link_profile(&make_metrics(10.0, 2.0, 0.0));

        let p1 = controller.update_metrics(make_metrics(1000.0, 100.0, 0.8));
        assert_eq!(p1.suggested_bucket, BucketSize::B256);
        assert_eq!(p1.tier, ControlTier::Enforce);
        assert!(p1.enable_partial_assembly);
        assert!(p1.timeout_multiplier > 1.0);
    }

    #[test]
    fn test_recovery_hysteresis() {
        let mut controller = SurvivalModeController::new();
        controller.detect_link_profile(&make_metrics(10.0, 2.0, 0.0));

        for _ in 0..3 {
            controller.update_metrics(make_metrics(200.0, 50.0, 0.3));
        }
        assert_eq!(controller.current_policy().suggested_bucket, BucketSize::B256);

        let p1 = controller.update_metrics(make_metrics(10.0, 2.0, 0.0));
        assert_eq!(p1.suggested_bucket, BucketSize::B256);

        let p2 = controller.update_metrics(make_metrics(10.0, 2.0, 0.0));
        assert_eq!(p2.suggested_bucket, BucketSize::B256);

        let p3 = controller.update_metrics(make_metrics(10.0, 2.0, 0.0));
        assert_eq!(p3.suggested_bucket, BucketSize::B1200);
    }

    #[test]
    fn test_satellite_baseline_normalization() {
        let mut controller = SurvivalModeController::new();
        controller.detect_link_profile(&make_metrics(600.0, 10.0, 0.0));

        let policy = controller.update_metrics(make_metrics(600.0, 10.0, 0.0));
        assert_eq!(policy.suggested_bucket, BucketSize::B1200);
        assert_eq!(policy.tier, ControlTier::Advisor);
    }

    // ============================================================
    // ✅ TEST BARU: Fungsi bebas detect_link_profile_from_metrics
    // ============================================================

    #[test]
    fn test_free_function_fiber() {
        assert_eq!(detect_link_profile_from_metrics(10.0, 2.0, 0.0), LinkProfile::Fiber);
    }

    #[test]
    fn test_free_function_satellite() {
        assert_eq!(detect_link_profile_from_metrics(600.0, 10.0, 0.0), LinkProfile::Satellite);
    }

    #[test]
    fn test_free_function_cellular() {
        assert_eq!(detect_link_profile_from_metrics(100.0, 25.0, 0.05), LinkProfile::Cellular);
    }

    #[test]
    fn test_free_function_unknown() {
        assert_eq!(detect_link_profile_from_metrics(100.0, 10.0, 0.0), LinkProfile::Unknown);
    }

    #[test]
    fn test_controller_uses_free_function() {
        // Verifikasi bahwa controller dan fungsi bebas menghasilkan hasil sama
        let mut controller = SurvivalModeController::new();
        let metrics = make_metrics(10.0, 2.0, 0.0);
        let from_controller = controller.detect_link_profile(&metrics);
        let from_free = detect_link_profile_from_metrics(metrics.rtt_ms, metrics.jitter_ms, metrics.loss_rate);
        assert_eq!(from_controller, from_free);
    }

    #[test]
    fn test_baseline_methods() {
        assert_eq!(LinkProfile::Fiber.baseline_rtt_ms(), 20.0);
        assert_eq!(LinkProfile::Cellular.baseline_rtt_ms(), 80.0);
        assert_eq!(LinkProfile::Satellite.baseline_rtt_ms(), 600.0);
        assert_eq!(LinkProfile::Unknown.baseline_rtt_ms(), 200.0);

        assert_eq!(LinkProfile::Fiber.baseline_jitter_ms(), 5.0);
        assert_eq!(LinkProfile::Cellular.baseline_jitter_ms(), 30.0);
        assert_eq!(LinkProfile::Satellite.baseline_jitter_ms(), 20.0);
        assert_eq!(LinkProfile::Unknown.baseline_jitter_ms(), 20.0);
    }

    #[test]
    fn test_profile_name() {
        assert_eq!(LinkProfile::Fiber.name(), "Fiber");
        assert_eq!(LinkProfile::Cellular.name(), "Cellular");
        assert_eq!(LinkProfile::Satellite.name(), "Satellite");
        assert_eq!(LinkProfile::Unknown.name(), "Unknown");
    }
}
