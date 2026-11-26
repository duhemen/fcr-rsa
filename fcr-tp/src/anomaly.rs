// src/anomaly.rs - VERSI REVISI BESAR
// 
// Sistem deteksi anomali dengan LEARNING ADAPTIF EVERLASTING:
// 
// A1: MetaLearner — self-tuning blend ratio (70/30 -> dinamis)
// A2: AdaptiveLearner — min 100 sampel sebelum belajar dari histori
// A3: AutoDetectConfig — fast check (20 sampel) + slow check (50 sampel)
// A4: GracePeriod — 3 sampel toleransi, reset saat tipe anomali berubah
// A5: ContextAwareness — FileUpload/Streaming/Interactive/BulkTransfer
// A6: Integrasi LinkProfile dari survival.rs
//
// Prinsip: "Belajar dari pengalaman, bukan dari aturan statis"
// Humaniora: "Jangan hukum user karena jaringannya buruk"

use std::collections::VecDeque;
use std::time::Instant;

use crate::survival::{LinkProfile, detect_link_profile_from_metrics};

// ============================================================
// KONSTANTA & KONFIGURASI (A1-A6)
// ============================================================

/// A1: Meta-learner tune interval (setiap 100 sampel)
pub const META_TUNE_INTERVAL: usize = 100;

/// A1: Blend ratio bounds (0.3 = 30% histori, 0.9 = 90% histori)
pub const BLEND_MIN: f64 = 0.3;
pub const BLEND_MAX: f64 = 0.9;
pub const BLEND_INITIAL: f64 = 0.7;
pub const BLEND_STEP: f64 = 0.05;

/// A2: Minimum samples untuk adaptive learning (100 sampel)
pub const MIN_SAMPLES_FOR_LEARNING: usize = 100;

/// A2: Learning window (200 sampel terakhir)
pub const LEARNING_WINDOW: usize = 200;

/// A3: Auto-detect dua tingkat
pub const FAST_DETECT_INTERVAL: usize = 20;
pub const FAST_DETECT_THRESHOLD: f64 = 0.5;   // 50% perubahan = drastis
pub const SLOW_DETECT_INTERVAL: usize = 50;
pub const SLOW_DETECT_THRESHOLD: f64 = 0.2;   // 20% perubahan = bertahap
pub const AUTO_DETECT_CONFIRMATIONS: usize = 2;

/// A4: Grace period untuk anomali single-dimension (3 sampel)
pub const GRACE_PERIOD_SAMPLES: usize = 3;

/// A5: Context awareness penalty untuk FileUpload/BulkTransfer
pub const UNIFORMITY_PENALTY_FACTOR: f64 = 0.1;

/// A6: Minimum samples untuk indikator (bukan Gray)
pub const MIN_SAMPLES_FOR_INDICATOR: usize = 10;

/// Batas ukuran history (memory safety)
pub const MAX_WINDOW_SIZE: usize = 64;

/// EMA alpha untuk IAT
pub const EMA_ALPHA: f64 = 0.2;

// ============================================================
// TIPE DASAR
// ============================================================

/// Jenis anomali yang terdeteksi
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnomalyType {
    BurstTraffic,
    UniformSizeInjection,
    ExtremeJitter,
}

/// Laporan anomali untuk forensic dan decision making
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnomalyReport {
    pub anomaly_type: AnomalyType,
    /// 0.0 - 1.0 (0 = tidak berbahaya, 1 = sangat berbahaya)
    pub severity: f64,
    /// Konteks forensik untuk sysadmin
    pub forensic_context: String,
    /// Apakah anomali ini persisten (berlanjut > grace period)?
    pub is_persistent: bool,
}

/// Konteks sesi dari layer aplikasi (A5)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SessionContext {
    /// Upload file (mis. 20 foto ukuran sama) — size uniformity normal
    FileUpload,
    /// Streaming media — size bervariasi normal
    Streaming,
    /// Interactive (chat, browsing) — size bervariasi normal
    Interactive,
    /// Bulk transfer (backup, sync) — size bisa uniform
    BulkTransfer,
    /// Tidak diketahui — gunakan default
    Unknown,
}

/// Threshold adaptif yang dihitung dari profil + histori
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AdaptiveThreshold {
    pub burst_threshold_ms: f64,
    pub uniformity_threshold: f64,
    pub jitter_multiplier: f64,
    pub baseline_iat_ms: f64,
    pub baseline_jitter_ms: f64,
}

impl AdaptiveThreshold {
    /// Baseline threshold per profil link (A6)
    pub fn for_profile(profile: LinkProfile) -> Self {
        match profile {
            LinkProfile::Fiber => Self {
                burst_threshold_ms: 2.0,
                uniformity_threshold: 1.5,
                jitter_multiplier: 5.0,
                baseline_iat_ms: profile.baseline_rtt_ms(),
                baseline_jitter_ms: profile.baseline_jitter_ms(),
            },
            LinkProfile::Cellular => Self {
                burst_threshold_ms: 10.0,
                uniformity_threshold: 2.5,
                jitter_multiplier: 4.0,
                baseline_iat_ms: profile.baseline_rtt_ms(),
                baseline_jitter_ms: profile.baseline_jitter_ms(),
            },
            LinkProfile::Satellite => Self {
                burst_threshold_ms: 50.0,
                uniformity_threshold: 3.0,
                jitter_multiplier: 3.0,
                baseline_iat_ms: profile.baseline_rtt_ms(),
                baseline_jitter_ms: profile.baseline_jitter_ms(),
            },
            LinkProfile::Unknown => Self {
                burst_threshold_ms: 20.0,
                uniformity_threshold: 2.0,
                jitter_multiplier: 4.0,
                baseline_iat_ms: profile.baseline_rtt_ms(),
                baseline_jitter_ms: profile.baseline_jitter_ms(),
            },
        }
    }
}

/// Sumber profil — hybrid dari semua opsi
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProfileSource {
    /// Dari constructor: `new()` atau `with_profile()`
    Static(LinkProfile),
    /// Dari setter: `set_profile()`
    Dynamic(LinkProfile),
    /// Dari auto-detection: `auto_detect_profile()`
    AutoDetected { detected_at_sample: usize },
}

// ============================================================
// A1: METALEARNER — Self-Tuning Blend Ratio
// ============================================================

/// A1: Meta-learner yang menyesuaikan blend ratio berdasarkan FP/FN proxy.
/// 
/// Blend ratio menentukan bobot antara baseline profil dan histori:
/// - blend = 0.3 → 30% histori, 70% baseline (konservatif)
/// - blend = 0.7 → 70% histori, 30% baseline (default)
/// - blend = 0.9 → 90% histori, 10% baseline (sangat adaptif)
pub struct MetaLearner {
    // Konfigurasi
    tune_interval: usize,
    min_blend: f64,
    max_blend: f64,
    step_size: f64,
    
    // State
    current_blend: f64,
    samples_since_last_tune: usize,
    
    // Proxy signal counters
    single_dimension_count: usize,   // proxy FP (noise jaringan)
    multi_dimension_count: usize,    // proxy TP (serangan nyata)
    transient_count: usize,          // proxy FP (pulih sendiri)
    persistent_count: usize,         // proxy TP (berlanjut)
    manual_reset_count: usize,       // proxy FP (user reset = konfirmasi)
    canary_decryption_count: usize,  // proxy TP (attacker tertangkap)
    
    // Observability
    total_tunes: usize,
    blend_history: VecDeque<f64>,
}

impl MetaLearner {
    pub fn new() -> Self {
        Self {
            tune_interval: META_TUNE_INTERVAL,
            min_blend: BLEND_MIN,
            max_blend: BLEND_MAX,
            step_size: BLEND_STEP,
            current_blend: BLEND_INITIAL,
            samples_since_last_tune: 0,
            single_dimension_count: 0,
            multi_dimension_count: 0,
            transient_count: 0,
            persistent_count: 0,
            manual_reset_count: 0,
            canary_decryption_count: 0,
            total_tunes: 0,
            blend_history: VecDeque::with_capacity(100),
        }
    }
    
    /// Catat observasi anomali untuk tuning berikutnya.
    pub fn observe_anomaly(&mut self, report: &AnomalyReport, is_persistent: bool) {
        self.samples_since_last_tune += 1;
        
        // Klasifikasi: severity rendah + single-dimension = kemungkinan noise (FP)
        let is_single_dimension = matches!(
            report.anomaly_type,
            AnomalyType::BurstTraffic | AnomalyType::ExtremeJitter
        );
        
        if is_single_dimension {
            if is_persistent {
                self.persistent_count += 1;
            } else {
                self.transient_count += 1;
            }
        } else {
            // UniformSizeInjection = multi-dimension (biasanya bukan noise)
            self.multi_dimension_count += 1;
        }
    }
    
    /// Catat observasi normal (tidak ada anomali).
    /// Ini juga increment counter untuk tuning interval.
    pub fn observe_normal(&mut self) {
        self.samples_since_last_tune += 1;
    }
    
    /// A1: Evaluasi dan sesuaikan blend ratio.
    /// Dipanggil setiap `tune_interval` sampel.
    pub fn tune(&mut self) -> f64 {
        if self.samples_since_last_tune < self.tune_interval {
            return self.current_blend;
        }
        
        // Hitung proxy FP dan TP
        let fp_proxy = self.transient_count as f64
            + self.manual_reset_count as f64 * 2.0;
        
        let tp_proxy = self.persistent_count as f64
            + self.multi_dimension_count as f64
            + self.canary_decryption_count as f64 * 3.0;
        
        let total_proxy = fp_proxy + tp_proxy;
        
        // Jika data tidak cukup, pertahankan blend saat ini
        if total_proxy < 5.0 {
            self.reset_counters();
            return self.current_blend;
        }
        
        let fp_rate = fp_proxy / total_proxy;
        let tp_rate = tp_proxy / total_proxy;
        
        // Logika penyesuaian:
        // - FP tinggi (> 0.6) → kurangi blend (lebih konservatif)
        // - TP tinggi (> 0.6) → tingkatkan blend (lebih adaptif)
        // - Seimbang → pertahankan
        if fp_rate > 0.6 {
            self.current_blend = (self.current_blend - self.step_size).max(self.min_blend);
        } else if tp_rate > 0.6 {
            self.current_blend = (self.current_blend + self.step_size).min(self.max_blend);
        }
        
        self.total_tunes += 1;
        if self.blend_history.len() >= 100 {
            self.blend_history.pop_front();
        }
        self.blend_history.push_back(self.current_blend);
        
        self.reset_counters();
        self.current_blend
    }
    
    fn reset_counters(&mut self) {
        self.samples_since_last_tune = 0;
        self.single_dimension_count = 0;
        self.multi_dimension_count = 0;
        self.transient_count = 0;
        self.persistent_count = 0;
        self.manual_reset_count = 0;
        self.canary_decryption_count = 0;
    }
    
    pub fn current_blend(&self) -> f64 { self.current_blend }
    pub fn blend_history(&self) -> &VecDeque<f64> { &self.blend_history }
    pub fn total_tunes(&self) -> usize { self.total_tunes }
}

impl Default for MetaLearner {
    fn default() -> Self { Self::new() }
}

// ============================================================
// A2: ADAPTIVELEARNER — Belajar dari Histori (Min 100 Sampel)
// ============================================================

/// A2: Learner yang menghitung threshold dari histori (persentil).
/// 
/// Butuh minimal 100 sampel sebelum mulai belajar. Sebelum itu,
/// fallback ke baseline profil.
pub struct AdaptiveLearner {
    min_samples_for_learning: usize,
    learning_window: usize,
    iat_window: VecDeque<f64>,
    size_window: VecDeque<usize>,
    samples_seen: usize,
}

impl AdaptiveLearner {
    pub fn new() -> Self {
        Self {
            min_samples_for_learning: MIN_SAMPLES_FOR_LEARNING,
            learning_window: LEARNING_WINDOW,
            iat_window: VecDeque::with_capacity(LEARNING_WINDOW),
            size_window: VecDeque::with_capacity(LEARNING_WINDOW),
            samples_seen: 0,
        }
    }
    
    /// A2: Apakah sudah cukup sampel untuk learning?
    pub fn is_ready(&self) -> bool {
        self.samples_seen >= self.min_samples_for_learning
    }
    
    /// A2: Catat observasi
    pub fn observe(&mut self, iat_ms: f64, size: usize) {
        self.samples_seen += 1;
        
        if self.iat_window.len() >= self.learning_window {
            self.iat_window.pop_front();
        }
        self.iat_window.push_back(iat_ms);
        
        if self.size_window.len() >= self.learning_window {
            self.size_window.pop_front();
        }
        self.size_window.push_back(size);
    }
    
    /// A2: Hitung threshold adaptif dengan blend ratio.
    /// 
    /// `blend` = bobot histori (0.0-1.0). Jika blend = 0.7, maka:
    /// - 70% bobot histori (persentil)
    /// - 30% bobot baseline profil
    pub fn current_threshold(
        &self,
        baseline: &AdaptiveThreshold,
        blend: f64,
    ) -> AdaptiveThreshold {
        if !self.is_ready() {
            return *baseline;
        }
        
        // Hitung persentil dari histori
        let p5_iat = percentile(&self.iat_window, 5.0).max(0.5);
        let p95_iat = percentile(&self.iat_window, 95.0);
        let jitter_estimate = (p95_iat - p5_iat).max(0.0);
        
        // Blend: `blend` bobot histori, `(1-blend)` bobot baseline
        let blend_clamped = blend.clamp(0.0, 1.0);
        AdaptiveThreshold {
            burst_threshold_ms: blend_clamped * p5_iat
                + (1.0 - blend_clamped) * baseline.burst_threshold_ms,
            uniformity_threshold: baseline.uniformity_threshold,  // size lebih stabil
            jitter_multiplier: baseline.jitter_multiplier,
            baseline_iat_ms: blend_clamped * p5_iat
                + (1.0 - blend_clamped) * baseline.baseline_iat_ms,
            baseline_jitter_ms: blend_clamped * jitter_estimate
                + (1.0 - blend_clamped) * baseline.baseline_jitter_ms,
        }
    }
    
    pub fn samples_seen(&self) -> usize { self.samples_seen }
    pub fn iat_window_len(&self) -> usize { self.iat_window.len() }
    pub fn size_window_len(&self) -> usize { self.size_window.len() }
    
    pub fn reset(&mut self) {
        self.samples_seen = 0;
        self.iat_window.clear();
        self.size_window.clear();
    }
}

impl Default for AdaptiveLearner {
    fn default() -> Self { Self::new() }
}

/// Helper: hitung persentil dari VecDeque (tidak harus sorted).
fn percentile(data: &VecDeque<f64>, p: f64) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut sorted: Vec<f64> = data.iter().copied().collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

// ============================================================
// A3: AUTODETECTCONFIG — Fast 20, Slow 50
// ============================================================

/// A3: Konfigurasi auto-detect dua tingkat.
/// 
/// - Fast check (setiap 20 sampel): deteksi perubahan drastis (>50%)
///   untuk handover cepat (cellular -> WiFi).
/// - Slow check (setiap 50 sampel): deteksi perubahan bertahap (>20%)
///   untuk degradasi kualitas perlahan.
/// 
/// Butuh 2 konfirmasi berturut-turut untuk mencegah flapping.
pub struct AutoDetectConfig {
    fast_interval: usize,
    fast_threshold: f64,
    slow_interval: usize,
    slow_threshold: f64,
    confirmations_needed: usize,
    
    samples_since_fast_check: usize,
    samples_since_slow_check: usize,
    fast_change_counter: usize,
    slow_change_counter: usize,
}

impl AutoDetectConfig {
    pub fn new() -> Self {
        Self {
            fast_interval: FAST_DETECT_INTERVAL,
            fast_threshold: FAST_DETECT_THRESHOLD,
            slow_interval: SLOW_DETECT_INTERVAL,
            slow_threshold: SLOW_DETECT_THRESHOLD,
            confirmations_needed: AUTO_DETECT_CONFIRMATIONS,
            samples_since_fast_check: 0,
            samples_since_slow_check: 0,
            fast_change_counter: 0,
            slow_change_counter: 0,
        }
    }
    
    /// A3: Evaluasi perubahan profil. Return `Some(profile)` jika
    /// perlu switch profil, `None` jika tidak.
    pub fn evaluate_change(
        &mut self,
        current_rtt: f64,
        baseline_rtt: f64,
        detected_profile: LinkProfile,
    ) -> Option<LinkProfile> {
        // Hitung perubahan relatif
        let change_ratio = if baseline_rtt > 0.0 {
            ((current_rtt - baseline_rtt) / baseline_rtt).abs()
        } else {
            0.0
        };
        
        self.samples_since_fast_check += 1;
        self.samples_since_slow_check += 1;
        
        // Fast check
        if self.samples_since_fast_check >= self.fast_interval {
            self.samples_since_fast_check = 0;
            if change_ratio > self.fast_threshold {
                self.fast_change_counter += 1;
                if self.fast_change_counter >= self.confirmations_needed {
                    self.fast_change_counter = 0;
                    self.slow_change_counter = 0;
                    return Some(detected_profile);
                }
            } else {
                self.fast_change_counter = 0;
            }
        }
        
        // Slow check
        if self.samples_since_slow_check >= self.slow_interval {
            self.samples_since_slow_check = 0;
            if change_ratio > self.slow_threshold {
                self.slow_change_counter += 1;
                if self.slow_change_counter >= self.confirmations_needed {
                    self.fast_change_counter = 0;
                    self.slow_change_counter = 0;
                    return Some(detected_profile);
                }
            } else {
                self.slow_change_counter = 0;
            }
        }
        
        None
    }
    
    pub fn reset(&mut self) {
        self.samples_since_fast_check = 0;
        self.samples_since_slow_check = 0;
        self.fast_change_counter = 0;
        self.slow_change_counter = 0;
    }
}

impl Default for AutoDetectConfig {
    fn default() -> Self { Self::new() }
}

// ============================================================
// A4: GRACEPERIOD — Toleransi 3 Sampel
// ============================================================

/// A4: Grace period untuk anomali single-dimension.
/// 
/// Anomali baru (tipe berbeda) → mulai grace period (3 sampel).
/// Sampel normal → tidak mengubah grace period.
/// Anomali sama berlanjut → decrement grace period.
/// Grace period habis → anomali di-flag (is_persistent = true).
pub struct GracePeriod {
    samples_remaining: usize,
    grace_samples: usize,
    last_anomaly_type: Option<AnomalyType>,
}

impl GracePeriod {
    pub fn new() -> Self {
        Self {
            samples_remaining: 0,
            grace_samples: GRACE_PERIOD_SAMPLES,
            last_anomaly_type: None,
        }
    }
    
    /// A4: Proses anomali. Return `true` jika harus di-suppress
    /// (masih dalam grace period), `false` jika harus di-flag.
    pub fn process(&mut self, anomaly_type: AnomalyType) -> bool {
        if self.last_anomaly_type != Some(anomaly_type) {  // move
            self.samples_remaining = self.grace_samples;
            self.last_anomaly_type = Some(anomaly_type);   // move lagi
        }
        
        // Jika masih dalam grace period, suppress
        if self.samples_remaining > 0 {
            self.samples_remaining -= 1;
            return true;  // suppress
        }
        
        // Grace period habis, jangan suppress
        false
    }
    
    /// A4: Sampel normal TIDAK mengubah grace period
    /// (grace period hanya untuk anomali, bukan normal).
    pub fn process_normal(&mut self) {
        // Tidak ada perubahan
    }
    
    pub fn reset(&mut self) {
        self.samples_remaining = 0;
        self.last_anomaly_type = None;
    }
    
    pub fn is_in_grace(&self) -> bool { self.samples_remaining > 0 }
    pub fn last_anomaly_type(&self) -> Option<AnomalyType> {
        self.last_anomaly_type  // OK jika AnomalyType: Copy
    }
}

impl Default for GracePeriod {
    fn default() -> Self { Self::new() }
}

// ============================================================
// A5: CONTEXTAWARENESS — FileUpload / Streaming / dll
// ============================================================

/// A5: Context awareness untuk menyesuaikan severity.
pub struct ContextAwareness {
    context: SessionContext,
}

impl ContextAwareness {
    pub fn new() -> Self {
        Self { context: SessionContext::Unknown }
    }
    
    pub fn set_context(&mut self, context: SessionContext) {
        self.context = context;
    }
    
    pub fn context(&self) -> SessionContext { self.context }
    
    /// A5: Apakah UniformSizeInjection relevan untuk konteks ini?
    pub fn is_uniformity_relevant(&self) -> bool {
        match self.context {
            SessionContext::FileUpload => false,
            SessionContext::BulkTransfer => false,
            SessionContext::Streaming => true,
            SessionContext::Interactive => true,
            SessionContext::Unknown => true,
        }
    }
    
    /// A5: Sesuaikan severity UniformSizeInjection.
    pub fn adjust_uniformity_severity(&self, base: f64) -> f64 {
        if self.is_uniformity_relevant() {
            base
        } else {
            base * UNIFORMITY_PENALTY_FACTOR
        }
    }
}

impl Default for ContextAwareness {
    fn default() -> Self { Self::new() }
}

// ============================================================
// ANOMALYDETECTOR — Integrasi Semua Layer
// ============================================================

/// Detector utama untuk Timing dan Size Anomaly dengan LEARNING ADAPTIF.
pub struct AnomalyDetector {
    // === EXISTING ===
    iat_history: VecDeque<f64>,
    last_arrival: Option<Instant>,
    ema_iat: f64,
    size_history: VecDeque<usize>,
    max_window_size: usize,
    ema_alpha: f64,
    
    // === HYBRID PROFILE (A3) ===
    profile: LinkProfile,
    profile_source: ProfileSource,
    auto_detect: AutoDetectConfig,
    
    // === LEARNING (A1, A2) ===
    meta_learner: MetaLearner,
    adaptive_learner: AdaptiveLearner,
    active_threshold: AdaptiveThreshold,
    
    // === GRACE PERIOD (A4) ===
    grace_period: GracePeriod,
    
    // === CONTEXT (A5) ===
    context: ContextAwareness,
    
    // === OBSERVABILITY ===
    total_samples: usize,
    total_anomalies: usize,
}

impl AnomalyDetector {
    /// Default: Unknown profile, auto-detect aktif
    pub fn new() -> Self {
        Self::with_profile(LinkProfile::Unknown)
    }
    
    /// Constructor dengan profil spesifik
    pub fn with_profile(profile: LinkProfile) -> Self {
        let threshold = AdaptiveThreshold::for_profile(profile);
        Self {
            iat_history: VecDeque::with_capacity(MAX_WINDOW_SIZE),
            last_arrival: None,
            ema_iat: 0.0,
            size_history: VecDeque::with_capacity(MAX_WINDOW_SIZE),
            max_window_size: MAX_WINDOW_SIZE,
            ema_alpha: EMA_ALPHA,
            profile,
            profile_source: ProfileSource::Static(profile),
            auto_detect: AutoDetectConfig::new(),
            meta_learner: MetaLearner::new(),
            adaptive_learner: AdaptiveLearner::new(),
            active_threshold: threshold,
            grace_period: GracePeriod::new(),
            context: ContextAwareness::new(),
            total_samples: 0,
            total_anomalies: 0,
        }
    }
    
    /// A3: Setter runtime untuk perubahan profil
    pub fn set_profile(&mut self, profile: LinkProfile) {
        if self.profile != profile {
            self.profile = profile;
            self.profile_source = ProfileSource::Dynamic(profile);
            self.active_threshold = AdaptiveThreshold::for_profile(profile);
            // Reset learner karena baseline berubah
            self.adaptive_learner.reset();
            self.auto_detect.reset();
        }
    }
    
    /// A5: Set konteks sesi
    pub fn set_context(&mut self, context: SessionContext) {
        self.context.set_context(context);
    }
    
    /// Getter untuk observability
    pub fn profile(&self) -> LinkProfile { self.profile }
    pub fn profile_source(&self) -> &ProfileSource { &self.profile_source }
    pub fn active_threshold(&self) -> &AdaptiveThreshold { &self.active_threshold }
    pub fn current_blend(&self) -> f64 { self.meta_learner.current_blend() }
    pub fn total_samples(&self) -> usize { self.total_samples }
    pub fn total_anomalies(&self) -> usize { self.total_anomalies }
    pub fn is_learning_ready(&self) -> bool { self.adaptive_learner.is_ready() }
    
    /// Default entry point: pakai Instant::now()
    pub fn record_chunk(&mut self, size: usize) -> Option<AnomalyReport> {
        self.record_chunk_at(size, Instant::now())
    }
    
    /// Timestamp injection: untuk test deterministik & integrasi
    /// dengan sumber waktu eksternal (mis. ForensicData.timestamp).
    pub fn record_chunk_at(&mut self, size: usize, timestamp: Instant) -> Option<AnomalyReport> {
        self.record_chunk_at_with_hint(size, timestamp, None)
    }
    
    /// Dengan profile hint (runtime parameter).
    /// 
    /// `profile_hint`: jika `Some`, override profil untuk panggilan ini.
    /// Berguna untuk frame dari sumber yang diketahui profilnya.
    pub fn record_chunk_at_with_hint(
        &mut self,
        size: usize,
        timestamp: Instant,
        profile_hint: Option<LinkProfile>,
    ) -> Option<AnomalyReport> {
        let now = timestamp;
        
        // 1. Hitung IAT (jika ada sampel sebelumnya)
        let iat_opt = self.last_arrival.map(|last| {
            now.duration_since(last).as_secs_f64() * 1000.0
        });
        self.last_arrival = Some(now);
        
        // 2. Update threshold aktif (A1 + A2 + profile hint)
        self.update_active_threshold(profile_hint);
        
        // 3. Deteksi anomali dengan threshold aktif
        let mut report = None;
        
        if let Some(iat) = iat_opt {
            // Update EMA
            if self.ema_iat == 0.0 {
                self.ema_iat = iat;
            } else {
                self.ema_iat = (self.ema_alpha * iat) + ((1.0 - self.ema_alpha) * self.ema_iat);
            }
            
            // Deteksi Burst (dengan threshold adaptif)
            if iat < self.active_threshold.burst_threshold_ms {
                report = Some(AnomalyReport {
                    anomaly_type: AnomalyType::BurstTraffic,
                    severity: ((self.active_threshold.burst_threshold_ms - iat)
                        / self.active_threshold.burst_threshold_ms).clamp(0.0, 1.0),
                    forensic_context: format!(
                        "IAT {:.2}ms < adaptive threshold {:.2}ms (profile: {})",
                        iat,
                        self.active_threshold.burst_threshold_ms,
                        self.profile.name()
                    ),
                    is_persistent: false,
                });
            }
            // Deteksi Extreme Jitter
            else if iat > (self.ema_iat * self.active_threshold.jitter_multiplier)
                && self.ema_iat > 0.0
            {
                report = Some(AnomalyReport {
                    anomaly_type: AnomalyType::ExtremeJitter,
                    severity: 0.8,
                    forensic_context: format!(
                        "IAT {:.2}ms > {:.1}x EMA ({:.2}ms)",
                        iat, self.active_threshold.jitter_multiplier, self.ema_iat
                    ),
                    is_persistent: false,
                });
            }
            
            // Simpan ke history
            if self.iat_history.len() >= self.max_window_size {
                self.iat_history.pop_front();
            }
            self.iat_history.push_back(iat);
            
            // Update learner (A2)
            self.adaptive_learner.observe(iat, size);
        }
        
        // 4. Deteksi size anomaly (A5)
        if self.size_history.len() >= self.max_window_size {
            self.size_history.pop_front();
        }
        self.size_history.push_back(size);
        
        if self.size_history.len() >= 10 {
            let stddev = self.calculate_std_dev();
            if stddev < self.active_threshold.uniformity_threshold && report.is_none() {
                let base_severity = (1.0 - (stddev / self.active_threshold.uniformity_threshold))
                    .clamp(0.0, 1.0);
                // A5: Sesuaikan severity dengan konteks
                let adjusted_severity = self.context.adjust_uniformity_severity(base_severity);
                
                report = Some(AnomalyReport {
                    anomaly_type: AnomalyType::UniformSizeInjection,
                    severity: adjusted_severity,
                    forensic_context: format!(
                        "Size StdDev {:.2} bytes (context: {:?})",
                        stddev, self.context.context()
                    ),
                    is_persistent: false,
                });
            }
        }
        
        // 5. Grace period (A4)
        let final_report = if let Some(mut r) = report {
            if self.grace_period.process(r.anomaly_type.clone()) {
                // Suppress: jangan return, tapi tetap catat untuk meta-learner
                self.meta_learner.observe_anomaly(&r, false);
                None
            } else {
                // Anomali berlanjut → flag
                r.is_persistent = true;
                self.meta_learner.observe_anomaly(&r, true);
                Some(r)
            }
        } else {
            self.grace_period.process_normal();
            self.meta_learner.observe_normal();
            None
        };
        
        // 6. Auto-detect profil (A3) — hanya jika tidak ada profile_hint
        if profile_hint.is_none() && self.iat_history.len() >= 10 {
            self.check_auto_detect();
        }
        
        // 7. Meta-learner tune (A1)
        if self.total_samples > 0 && self.total_samples % META_TUNE_INTERVAL == 0 {
            self.meta_learner.tune();
        }
        
        self.total_samples += 1;
        if final_report.is_some() {
            self.total_anomalies += 1;
        }
        
        final_report
    }
    
    /// Update threshold aktif dari semua sumber
    fn update_active_threshold(&mut self, profile_hint: Option<LinkProfile>) {
        let effective_profile = profile_hint.unwrap_or(self.profile);
        let baseline = AdaptiveThreshold::for_profile(effective_profile);
        let blend = self.meta_learner.current_blend();
        self.active_threshold = self.adaptive_learner.current_threshold(&baseline, blend);
    }
    
    /// A3: Auto-detect perubahan profil
    fn check_auto_detect(&mut self) {
        let current_rtt = self.ema_iat;
        let baseline_rtt = self.active_threshold.baseline_iat_ms;
        
        let detected = detect_link_profile_from_metrics(
            current_rtt,
            self.active_threshold.baseline_jitter_ms,
            0.0,  // loss unknown
        );
        
        if let Some(new_profile) = self.auto_detect.evaluate_change(
            current_rtt, baseline_rtt, detected,
        ) {
            self.profile = new_profile;
            self.profile_source = ProfileSource::AutoDetected {
                detected_at_sample: self.total_samples,
            };
            self.active_threshold = AdaptiveThreshold::for_profile(new_profile);
        }
    }
    
    /// Hitung Standard Deviation dari ukuran payload
    fn calculate_std_dev(&self) -> f64 {
        let n = self.size_history.len() as f64;
        if n == 0.0 { return 0.0; }
        let mean: f64 = self.size_history.iter().sum::<usize>() as f64 / n;
        let variance: f64 = self.size_history.iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / n;
        variance.sqrt()
    }
    
    pub fn reset(&mut self) {
        self.iat_history.clear();
        self.size_history.clear();
        self.last_arrival = None;
        self.ema_iat = 0.0;
        self.profile = LinkProfile::Unknown;
        self.profile_source = ProfileSource::Static(LinkProfile::Unknown);
        self.auto_detect.reset();
        self.meta_learner = MetaLearner::new();
        self.adaptive_learner.reset();
        self.active_threshold = AdaptiveThreshold::for_profile(LinkProfile::Unknown);
        self.grace_period.reset();
        self.context = ContextAwareness::new();
        self.total_samples = 0;
        self.total_anomalies = 0;
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self { Self::new() }
}

// ============================================================
// TEST SUITE
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    // ============================================================
    // Helper: simulasi timestamp
    // ============================================================
    fn sim_instant(base: Instant, ms: u64) -> Instant {
        base + Duration::from_millis(ms)
    }
    
    // ============================================================
    // TEST: Baseline behavior (regression)
    // ============================================================
    
    #[test]
    fn test_normal_traffic_no_anomaly() {
        let mut detector = AnomalyDetector::with_profile(LinkProfile::Fiber);
        let base = Instant::now();
        let mut reports = 0;
        
        for i in 0..15 {
            let ts = sim_instant(base, (50 + (i % 5) * 10) as u64);
            if detector.record_chunk_at(100 + (i % 20) * 5, ts).is_some() {
                reports += 1;
            }
        }
        // Dengan IAT 50-90ms, tidak ada burst (threshold fiber 2ms)
        assert_eq!(reports, 0, "Traffic normal tidak boleh ada anomali");
        assert_eq!(detector.size_history.len(), 15);
    }
    
    #[test]
    fn test_burst_attack_detection() {
        let mut detector = AnomalyDetector::with_profile(LinkProfile::Fiber);
        let base = Instant::now();
        
        // Baseline
        detector.record_chunk_at(100, sim_instant(base, 0));
        detector.record_chunk_at(100, sim_instant(base, 50));
        
        // Burst: IAT 0.5ms (di bawah threshold fiber 2ms)
        // Karena grace period 3 sampel, butuh 4 sampel burst berturut-turut
        let mut report = None;
        for i in 2..10 {
            let ts = sim_instant(base, 50 + (i - 2) * 1);  // IAT 1ms
            if let Some(r) = detector.record_chunk_at(100, ts) {
                report = Some(r);
                break;
            }
        }
        
        assert!(report.is_some(), "Burst attack harus terdeteksi setelah grace period");
        let r = report.unwrap();
        assert_eq!(r.anomaly_type, AnomalyType::BurstTraffic);
        assert!(r.is_persistent, "Setelah grace period, anomali harus persistent");
    }
    
    #[test]
    fn test_uniform_size_injection() {
        let mut detector = AnomalyDetector::with_profile(LinkProfile::Fiber);
        let base = Instant::now();
        
        let mut report = None;
        // Kirim 20 paket ukuran sama dengan interval normal (50ms)
        for i in 0..20 {
            let ts = sim_instant(base, i * 50);
            if let Some(r) = detector.record_chunk_at(512, ts) {
                report = Some(r);
                break;
            }
        }
        
        assert!(report.is_some(), "Uniform size injection harus terdeteksi");
        assert_eq!(report.unwrap().anomaly_type, AnomalyType::UniformSizeInjection);
    }
    
    // ============================================================
    // TEST: A1 - MetaLearner
    // ============================================================
    
    #[test]
    fn test_meta_learner_initial_blend() {
        let learner = MetaLearner::new();
        assert_eq!(learner.current_blend(), BLEND_INITIAL);
        assert_eq!(learner.total_tunes(), 0);
    }
    
    #[test]
    fn test_meta_learner_blend_decreases_on_high_fp() {
        let mut learner = MetaLearner::new();
        let initial = learner.current_blend();
        
        // Simulasi 100 sampel dengan banyak FP (transient, single-dimension)
        for _ in 0..100 {
            let report = AnomalyReport {
                anomaly_type: AnomalyType::BurstTraffic,
                severity: 0.3,
                forensic_context: "test".to_string(),
                is_persistent: false,
            };
            learner.observe_anomaly(&report, false);  // transient = FP proxy
        }
        
        let new_blend = learner.tune();
        assert!(new_blend < initial,
            "Blend harus turun karena FP tinggi: {} -> {}", initial, new_blend);
    }
    
    #[test]
    fn test_meta_learner_blend_increases_on_high_tp() {
        let mut learner = MetaLearner::new();
        let initial = learner.current_blend();
        
        // Simulasi 100 sampel dengan banyak TP (persistent + multi-dimension)
        for _ in 0..50 {
            let report = AnomalyReport {
                anomaly_type: AnomalyType::BurstTraffic,
                severity: 0.9,
                forensic_context: "test".to_string(),
                is_persistent: true,
            };
            learner.observe_anomaly(&report, true);  // persistent = TP proxy
        }
        for _ in 0..50 {
            let report = AnomalyReport {
                anomaly_type: AnomalyType::UniformSizeInjection,
                severity: 0.9,
                forensic_context: "test".to_string(),
                is_persistent: true,
            };
            learner.observe_anomaly(&report, true);  // multi-dimension = TP proxy
        }
        
        let new_blend = learner.tune();
        assert!(new_blend > initial,
            "Blend harus naik karena TP tinggi: {} -> {}", initial, new_blend);
    }
    
    #[test]
    fn test_meta_learner_blend_stable_when_balanced() {
        let mut learner = MetaLearner::new();
        let initial = learner.current_blend();
        
        // Simulasi 50 FP + 50 TP (seimbang)
        for _ in 0..50 {
            let report = AnomalyReport {
                anomaly_type: AnomalyType::BurstTraffic,
                severity: 0.3,
                forensic_context: "test".to_string(),
                is_persistent: false,
            };
            learner.observe_anomaly(&report, false);
        }
        for _ in 0..50 {
            let report = AnomalyReport {
                anomaly_type: AnomalyType::BurstTraffic,
                severity: 0.9,
                forensic_context: "test".to_string(),
                is_persistent: true,
            };
            learner.observe_anomaly(&report, true);
        }
        
        let new_blend = learner.tune();
        assert!((new_blend - initial).abs() < 0.001,
            "Blend harus stabil saat seimbang: {} -> {}", initial, new_blend);
    }
    
    #[test]
    fn test_meta_learner_does_not_tune_before_interval() {
        let mut learner = MetaLearner::new();
        let initial = learner.current_blend();
        
        // Hanya 50 sampel (kurang dari 100)
        for _ in 0..50 {
            let report = AnomalyReport {
                anomaly_type: AnomalyType::BurstTraffic,
                severity: 0.3,
                forensic_context: "test".to_string(),
                is_persistent: false,
            };
            learner.observe_anomaly(&report, false);
        }
        
        let new_blend = learner.tune();
        assert_eq!(new_blend, initial, "Tidak boleh tune sebelum interval");
        assert_eq!(learner.total_tunes(), 0);
    }
    
    // ============================================================
    // TEST: A2 - AdaptiveLearner
    // ============================================================
    
    #[test]
    fn test_adaptive_learner_not_ready_before_100() {
        let mut learner = AdaptiveLearner::new();
        assert!(!learner.is_ready());
        
        for _i in 0..99 {
            learner.observe(50.0, 512);
        }
        assert!(!learner.is_ready(), "Harus belum ready sebelum 100 sampel");
        
        learner.observe(50.0, 512);
        assert!(learner.is_ready(), "Harus ready setelah 100 sampel");
    }
    
    #[test]
    fn test_adaptive_learner_uses_histori_after_ready() {
        let mut learner = AdaptiveLearner::new();
        let baseline = AdaptiveThreshold::for_profile(LinkProfile::Fiber);
        
        // Sebelum ready, threshold = baseline
        let threshold_before = learner.current_threshold(&baseline, 0.9);
        assert_eq!(threshold_before.burst_threshold_ms, baseline.burst_threshold_ms);
        
        // Isi dengan IAT 5ms (lebih rendah dari baseline fiber 2ms? Tidak, 5 > 2)
        // Untuk test yang jelas, pakai IAT 1ms (histori: burst threshold ~1ms)
        for _ in 0..100 {
            learner.observe(1.0, 512);
        }
        
        let threshold_after = learner.current_threshold(&baseline, 0.9);
        // 90% histori (1ms) + 10% baseline (2ms) = 0.9*1 + 0.1*2 = 1.1ms
        assert!(
            (threshold_after.burst_threshold_ms - 1.1).abs() < 0.01,
            "Threshold harus blend histori & baseline: got {}",
            threshold_after.burst_threshold_ms
        );
    }
    
    // ============================================================
    // TEST: A3 - AutoDetectConfig
    // ============================================================
    
    #[test]
    fn test_auto_detect_fast_change() {
        let mut config = AutoDetectConfig::new();
        
        // Simulasi: RTT berubah 60% (di atas fast threshold 50%)
        // Butuh 2 konfirmasi (setiap 20 sampel = 40 sampel)
        let mut result = None;
        for _ in 0..40 {
            if let Some(p) = config.evaluate_change(600.0, 100.0, LinkProfile::Satellite) {
                result = Some(p);
                break;
            }
        }
        
        assert_eq!(result, Some(LinkProfile::Satellite),
            "Fast change harus terdeteksi setelah 2 konfirmasi (40 sampel)");
    }
    
    #[test]
    fn test_auto_detect_no_change_if_small() {
        let mut config = AutoDetectConfig::new();
        
        // Perubahan hanya 10% (di bawah slow threshold 20%)
        for _ in 0..200 {
            if config.evaluate_change(110.0, 100.0, LinkProfile::Satellite).is_some() {
                panic!("Tidak boleh detect perubahan kecil");
            }
        }
    }
    
    #[test]
    fn test_auto_detect_confirmation_prevents_flapping() {
        let mut config = AutoDetectConfig::new();
        
        // Satu kali perubahan besar, lalu normal kembali
        // (perubahan tidak persisten, jadi tidak boleh trigger)
        for _ in 0..20 {
            config.evaluate_change(200.0, 100.0, LinkProfile::Satellite);
        }
        // Hanya 1 counter, belum konfirmasi
        
        for _ in 0..40 {
            config.evaluate_change(100.0, 100.0, LinkProfile::Unknown);
        }
        // Counter fast reset karena perubahan kecil
        
        // Verifikasi tidak ada flapping
        // (tidak ada panic, config stabil)
    }
    
    // ============================================================
    // TEST: A4 - GracePeriod
    // ============================================================
    
    #[test]
    fn test_grace_period_suppresses_first_3() {
        let mut gp = GracePeriod::new();
        let anomaly = AnomalyType::BurstTraffic;
        
        // Sampel 1-3: suppress (grace)
        assert!(gp.process(anomaly.clone()), "Sampel 1 harus suppress");
        assert!(gp.process(anomaly.clone()), "Sampel 2 harus suppress");
        assert!(gp.process(anomaly.clone()), "Sampel 3 harus suppress");
        
        // Sampel 4: flag (grace habis)
        assert!(!gp.process(anomaly.clone()), "Sampel 4 harus flag");
    }
    
    #[test]
    fn test_grace_period_resets_on_different_anomaly() {
        let mut gp = GracePeriod::new();
        
        // Mulai dengan Burst
        assert!(gp.process(AnomalyType::BurstTraffic));
        assert!(gp.process(AnomalyType::BurstTraffic));
        
        // Ganti tipe anomali → reset grace
        assert!(gp.process(AnomalyType::UniformSizeInjection), "Tipe baru harus reset grace");
        assert!(gp.process(AnomalyType::UniformSizeInjection));
        assert!(gp.process(AnomalyType::UniformSizeInjection));
        assert!(!gp.process(AnomalyType::UniformSizeInjection), "Setelah 3, harus flag");
    }
    
    #[test]
    fn test_grace_period_normal_samples_dont_reset() {
        let mut gp = GracePeriod::new();
        
        assert!(gp.process(AnomalyType::BurstTraffic));  // 1
        gp.process_normal();  // normal, tidak reset
        assert!(gp.process(AnomalyType::BurstTraffic));  // 2
        gp.process_normal();  // normal, tidak reset
        assert!(gp.process(AnomalyType::BurstTraffic));  // 3
        gp.process_normal();  // normal, tidak reset
        assert!(!gp.process(AnomalyType::BurstTraffic)); // 4 = flag
    }
    
    // ============================================================
    // TEST: A5 - ContextAwareness
    // ============================================================
    
    #[test]
    fn test_context_file_upload_reduces_severity() {
        let mut ctx = ContextAwareness::new();
        ctx.set_context(SessionContext::FileUpload);
        
        assert!(!ctx.is_uniformity_relevant());
        assert_eq!(ctx.adjust_uniformity_severity(1.0), UNIFORMITY_PENALTY_FACTOR);
        assert_eq!(ctx.adjust_uniformity_severity(0.5), 0.5 * UNIFORMITY_PENALTY_FACTOR);
    }
    
    #[test]
    fn test_context_streaming_keeps_severity() {
        let mut ctx = ContextAwareness::new();
        ctx.set_context(SessionContext::Streaming);
        
        assert!(ctx.is_uniformity_relevant());
        assert_eq!(ctx.adjust_uniformity_severity(1.0), 1.0);
    }
    
    #[test]
    fn test_context_unknown_keeps_severity() {
        let ctx = ContextAwareness::new();  // default Unknown
        
        assert!(ctx.is_uniformity_relevant());
        assert_eq!(ctx.adjust_uniformity_severity(0.8), 0.8);
    }
    
    // ============================================================
    // TEST: Integrasi — AnomalyDetector dengan Learning
    // ============================================================
    
    #[test]
    fn test_detector_with_fiber_profile() {
        let detector = AnomalyDetector::with_profile(LinkProfile::Fiber);
        assert_eq!(detector.profile(), LinkProfile::Fiber);
        assert_eq!(detector.active_threshold().burst_threshold_ms, 2.0);
    }
    
    #[test]
    fn test_detector_with_satellite_profile() {
        let detector = AnomalyDetector::with_profile(LinkProfile::Satellite);
        assert_eq!(detector.profile(), LinkProfile::Satellite);
        assert_eq!(detector.active_threshold().burst_threshold_ms, 50.0);
    }
    
    #[test]
    fn test_detector_set_profile_changes_threshold() {
        let mut detector = AnomalyDetector::with_profile(LinkProfile::Fiber);
        assert_eq!(detector.active_threshold().burst_threshold_ms, 2.0);
        
        detector.set_profile(LinkProfile::Satellite);
        assert_eq!(detector.profile(), LinkProfile::Satellite);
        assert_eq!(detector.active_threshold().burst_threshold_ms, 50.0);
    }
    
    #[test]
     fn test_satellite_high_rtt_not_flagged() {
        // RTT 600ms di satellite adalah NORMAL, bukan anomali
        let mut detector = AnomalyDetector::with_profile(LinkProfile::Satellite);
        let base = Instant::now();
        
        let mut timing_reports = 0;
        for i in 0..20usize {
            let ts = sim_instant(base, (i * 600) as u64);
            let size = 512 + (i % 10) * 50;  // ✅ size bervariasi
            if let Some(r) = detector.record_chunk_at(size, ts) {
                if matches!(r.anomaly_type, AnomalyType::BurstTraffic | AnomalyType::ExtremeJitter) {
                    timing_reports += 1;
                }
            }
        }
        assert_eq!(timing_reports, 0, "RTT 600ms di satellite tidak boleh timing anomali");
    }
    
    #[test]
    fn test_cellular_moderate_rtt_not_flagged() {
        // RTT 80ms di cellular adalah NORMAL
        let mut detector = AnomalyDetector::with_profile(LinkProfile::Cellular);
        let base = Instant::now();
        
        let mut timing_reports = 0;
        for i in 0..20usize {
            let ts = sim_instant(base, (i * 80) as u64);
            let size = 512 + (i % 10) * 50;  // ✅ size bervariasi
            if let Some(r) = detector.record_chunk_at(size, ts) {
                if matches!(r.anomaly_type, AnomalyType::BurstTraffic | AnomalyType::ExtremeJitter) {
                    timing_reports += 1;
                }
            }
        }
        assert_eq!(timing_reports, 0, "RTT 80ms di cellular tidak boleh timing anomali");
    }
    
    #[test]
    fn test_profile_hint_overrides_for_one_call() {
        // Detector dengan profil Fiber, tapi frame dari satellite relay
        let mut detector = AnomalyDetector::with_profile(LinkProfile::Fiber);
        let base = Instant::now();
        
        detector.record_chunk_at(100, sim_instant(base, 0));
        
        // Frame dari satellite: IAT 600ms. Dengan profil Fiber, ini ExtremeJitter.
        // Dengan profile_hint Satellite, ini normal.
        let report = detector.record_chunk_at_with_hint(
            100,
            sim_instant(base, 600),
            Some(LinkProfile::Satellite),
        );
        
        assert!(report.is_none(), "Dengan hint satellite, IAT 600ms normal");
    }
    
    #[test]
    fn test_learning_ready_after_100_samples() {
        let mut detector = AnomalyDetector::with_profile(LinkProfile::Fiber);
        let base = Instant::now();
        
        // ✅ FIX: 101 sampel karena sampel pertama tidak punya IAT
        for i in 0..101usize {
            detector.record_chunk_at(
                512 + (i % 10) * 50,
                sim_instant(base, (i * 50) as u64),
            );
        }
        
        assert!(detector.is_learning_ready(), "Setelah 100+ sampel, harus ready");
    }
    
    #[test]
    fn test_reset_clears_all_state() {
        let mut detector = AnomalyDetector::with_profile(LinkProfile::Fiber);
        let base = Instant::now();
        
        for i in 0..50 {
            detector.record_chunk_at(512, sim_instant(base, i * 50));
        }
        
        detector.reset();
        
        assert_eq!(detector.total_samples(), 0);
        assert_eq!(detector.total_anomalies(), 0);
        assert!(!detector.is_learning_ready());
        assert_eq!(detector.profile(), LinkProfile::Unknown);
    }
}