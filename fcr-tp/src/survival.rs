// src/survival.rs
use std::collections::VecDeque;

/// Profil karakteristik link jaringan (Heuristik)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkProfile {
    Fiber,       // RTT rendah, Jitter sangat rendah
    Cellular,    // RTT sedang/tinggi, Jitter tinggi (radio scheduling)
    Satellite,   // RTT sangat tinggi, Jitter rendah (jalur fisik panjang tapi stabil)
    Unknown,     // Belum teridentifikasi, menggunakan baseline dinamis
}

/// Metrik jaringan real-time
#[derive(Debug, Clone, Copy)]
pub struct NetworkMetrics {
    pub rtt_ms: f64,
    pub jitter_ms: f64,
    pub loss_rate: f64,   // 0.0 - 1.0
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
    Advisor, // Reassembler boleh menolak jika melanggar batas keamanan
    Enforce, // Reassembler wajib mematuhi (kondisi kritis)
}

/// Kebijakan Survival Mode yang dikirim ke modul lain (Reassembler, Crypto)
#[derive(Debug, Clone, PartialEq)]
pub struct SurvivalPolicy {
    pub tier: ControlTier,
    pub suggested_bucket: BucketSize,
    pub timeout_multiplier: f32, // Dikali dengan default timeout
    pub enable_partial_assembly: bool,
}

/// State untuk transisi bucket (Hybrid: Hysteresis + Emergency Override)
struct TransitionState {
    current_bucket: BucketSize,
    hysteresis_counter: i32, // Positif = cenderung naik, Negatif = cenderung turun
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
    pub const HYSTERESIS_THRESHOLD: i32 = 3; // Butuh 3 sampel berturut-turut untuk switch normal
    pub const CRITICAL_SCORE_THRESHOLD: f64 = 0.2; // Di bawah ini = Emergency Override
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
        last_score: NetworkHealthScore { total: 1.0, latency_score: 1.0, loss_score: 1.0, jitter_score: 1.0 }, // ← BARU
        last_emergency: false,                                                                                   // ← BARU
    }
}

    /// Deteksi profil link berdasarkan metrik awal
    pub fn detect_link_profile(&mut self, metrics: &NetworkMetrics) -> LinkProfile {
        self.profile = if metrics.rtt_ms < 50.0 && metrics.jitter_ms < 10.0 {
            LinkProfile::Fiber
        } else if metrics.rtt_ms > 200.0 && metrics.jitter_ms < 30.0 {
            LinkProfile::Satellite
        } else if metrics.jitter_ms > 15.0 {
            LinkProfile::Cellular
        } else {
            LinkProfile::Unknown
        };
        self.profile
    }

    /// Update metrik dan dapatkan kebijakan survival terbaru
    pub fn update_metrics(&mut self, metrics: NetworkMetrics) -> SurvivalPolicy {
        // Simpan history untuk moving average
        if self.metrics_history.len() >= self.max_history {
            self.metrics_history.pop_front();
        }
        self.metrics_history.push_back(metrics);

        let score = self.calculate_health_score(&metrics);
        let (new_bucket, is_emergency) = self.determine_bucket_size(&score);
                self.last_score = score;
                self.last_emergency = is_emergency;
        // Update transition state
        if new_bucket != self.transition.current_bucket {
            if is_emergency {
                // Emergency Override: Bypass hysteresis
                self.transition.current_bucket = new_bucket;
                self.transition.hysteresis_counter = 0;
            } else {
                // Hysteresis: Accumulate counter
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

    /// Hitung skor kesehatan multi-dimensi
    fn calculate_health_score(&self, metrics: &NetworkMetrics) -> NetworkHealthScore {
        // Normalisasi berdasarkan profil link
        let (rtt_baseline, jitter_baseline) = match self.profile {
            LinkProfile::Fiber => (50.0, 10.0),
            LinkProfile::Cellular => (150.0, 30.0),
            LinkProfile::Satellite => (600.0, 20.0),
            LinkProfile::Unknown => (200.0, 20.0),
        };

        let latency_score = (1.0 - (metrics.rtt_ms / (rtt_baseline * 3.0)).min(1.0)).max(0.0);
        let jitter_score = (1.0 - (metrics.jitter_ms / (jitter_baseline * 3.0)).min(1.0)).max(0.0);
        let loss_score = (1.0 - metrics.loss_rate).max(0.0);

        // Weighted scoring: Loss paling kritis, lalu Latency, lalu Jitter
        let total = (loss_score * 0.5) + (latency_score * 0.3) + (jitter_score * 0.2);

        NetworkHealthScore {
            total,
            latency_score,
            loss_score,
            jitter_score,
        }
    }

    /// Tentukan ukuran bucket (Hybrid logic)
    fn determine_bucket_size(&self, score: &NetworkHealthScore) -> (BucketSize, bool) {
        // Emergency Override
        if score.total < Self::CRITICAL_SCORE_THRESHOLD {
            return (BucketSize::B256, true);
        }

        // Normal Hysteresis mapping
        let target = if score.total < Self::POOR_SCORE_THRESHOLD {
            BucketSize::B256
        } else if score.total < Self::GOOD_SCORE_THRESHOLD {
            BucketSize::B512
        } else {
            BucketSize::B1200
        };

        (target, false)
    }

    /// Generate kebijakan untuk modul lain
    fn generate_policy(&self, score: NetworkHealthScore, is_emergency: bool) -> SurvivalPolicy {
        let tier = if is_emergency || score.total < Self::POOR_SCORE_THRESHOLD {
            ControlTier::Enforce
        } else {
            ControlTier::Advisor
        };

        // Timeout multiplier: Semakin buruk jaringan, semakin longgar timeoutnya
        let timeout_multiplier = if score.total < Self::CRITICAL_SCORE_THRESHOLD {
            5.0 // 5x default timeout untuk kondisi kritis
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

    /// Cermin jujur dari kondisi terkini (bisa dipanggil kapan saja oleh Reassembler)
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
        controller.detect_link_profile(&make_metrics(10.0, 2.0, 0.0)); // Fiber

        // Kirim metrik buruk 2 kali (belum cukup untuk trigger hysteresis)
        let p1 = controller.update_metrics(make_metrics(200.0, 50.0, 0.3));
        assert_eq!(p1.suggested_bucket, BucketSize::B1200); // Masih B1200 karena hysteresis

        let p2 = controller.update_metrics(make_metrics(200.0, 50.0, 0.3));
        assert_eq!(p2.suggested_bucket, BucketSize::B1200);

        // Kali ke-3, hysteresis tercapai, switch ke B256
        let p3 = controller.update_metrics(make_metrics(200.0, 50.0, 0.3));
        assert_eq!(p3.suggested_bucket, BucketSize::B256);
        assert_eq!(p3.tier, ControlTier::Enforce);
    }

    #[test]
    fn test_emergency_override_bypasses_hysteresis() {
        let mut controller = SurvivalModeController::new();
        controller.detect_link_profile(&make_metrics(10.0, 2.0, 0.0)); // Fiber

        // Kirim metrik KRITIS (loss 80%, RTT tinggi)
        let p1 = controller.update_metrics(make_metrics(1000.0, 100.0, 0.8));
        
        // Harus langsung switch ke B256 dan Enforce, tanpa menunggu hysteresis
        assert_eq!(p1.suggested_bucket, BucketSize::B256);
        assert_eq!(p1.tier, ControlTier::Enforce);
        assert!(p1.enable_partial_assembly);
        assert!(p1.timeout_multiplier > 1.0);
    }

    #[test]
    fn test_recovery_hysteresis() {
        let mut controller = SurvivalModeController::new();
        controller.detect_link_profile(&make_metrics(10.0, 2.0, 0.0));

        // Rusak jaringan
        for _ in 0..3 {
            controller.update_metrics(make_metrics(200.0, 50.0, 0.3));
        }
        assert_eq!(controller.current_policy().suggested_bucket, BucketSize::B256);

        // Perbaiki jaringan
        let p1 = controller.update_metrics(make_metrics(10.0, 2.0, 0.0));
        assert_eq!(p1.suggested_bucket, BucketSize::B256); // Belum naik

        let p2 = controller.update_metrics(make_metrics(10.0, 2.0, 0.0));
        assert_eq!(p2.suggested_bucket, BucketSize::B256);

        let p3 = controller.update_metrics(make_metrics(10.0, 2.0, 0.0));
        assert_eq!(p3.suggested_bucket, BucketSize::B1200); // Naik kembali
    }

    #[test]
    fn test_satellite_baseline_normalization() {
        let mut controller = SurvivalModeController::new();
        controller.detect_link_profile(&make_metrics(600.0, 10.0, 0.0)); // Satellite

        // RTT 600ms di Satellite adalah NORMAL, skor harus tinggi
        let policy = controller.update_metrics(make_metrics(600.0, 10.0, 0.0));
        assert_eq!(policy.suggested_bucket, BucketSize::B1200);
        assert_eq!(policy.tier, ControlTier::Advisor);
    }
}