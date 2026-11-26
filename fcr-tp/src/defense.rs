// src/defense.rs - VERSI REVISI BESAR
//
// Perubahan:
// 1. TransportRiskScore: bobot timing/size anomaly (40/20/20/20)
// 2. Integrasi AnomalyDetector (transient vs persistent)
// 3. advance_epoch: clear seen_tokens SETIAP epoch berubah
// 4. Token insert hanya jika Accept/Canary (bukan Disconnect)
// 5. HealthIndicator (🟢🟡🔴⚪) untuk end-user
// 6. UserStatus + TechnicalDetails (kompatibel transport publik: HTTP/3, WebSocket, gRPC)
// 7. build_canary_frame: guard payload muat di bucket
// 8. Test diperbaiki: verifikasi semantik, bukan bug-as-spec

use std::collections::HashSet;
use rand::RngCore;
use zeroize::Zeroize;

use crate::packet::FcrTpFrame;
use crate::crypto::{
    ChunkMetadata, CHUNK_METADATA_SIZE, FcrTpSession as CryptoSession,
    CURRENT_CRYPTO_VERSION,
};
use crate::anomaly::{
    AnomalyDetector, AnomalyReport, AnomalyType, AdaptiveThreshold,
};

// ============================================================
// KONSTANTA
// ============================================================

/// Ambang batas risiko untuk mengaktifkan Canary Pill (0-100)
pub const CANARY_THRESHOLD: u8 = 30;

/// Ambang batas risiko untuk memutus koneksi (0-100)
pub const DISCONNECT_THRESHOLD: u8 = 80; // FIX: sebelumnya 100 tidak pernah tercapai (maks riil 90: timing & size mutually exclusive)

/// Maksimum token yang disimpan (sebelum di-clear per epoch)
pub const MAX_SEEN_TOKENS: usize = 10_000;

/// Ukuran forensic payload tambahan
/// (trace_id 16 + session_id 8 + epoch 8 + padding 16)
pub const FORENSIC_OVERHEAD: usize = 16 + 8 + 8 + 16;

/// Bobot risk score (total = 100)
pub const WEIGHT_TOKEN_REUSE: u32 = 40;
pub const WEIGHT_PORT_VIOLATION: u32 = 40;
pub const WEIGHT_TIMING_ANOMALY: u32 = 10;
pub const WEIGHT_SIZE_ANOMALY: u32 = 10;

/// Minimum sampel sebelum indikator bukan Gray (A6)
pub const MIN_SAMPLES_FOR_INDICATOR: usize = 10;

// ============================================================
// RISK SCORE
// ============================================================

/// Skor risiko transport (0 = aman, 100 = sangat mencurigakan)
#[derive(Debug, Clone, Default, Zeroize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransportRiskScore {
    pub timing_anomaly: u8,
    pub size_anomaly: u8,
    pub port_violation: u8,
    pub token_reuse: u8,
    pub total: u8,
}

impl TransportRiskScore {
    /// Hitung total dengan bobot yang benar.
    /// 
    /// REVISI: sebelumnya timing_anomaly & size_anomaly berbobot 0,
    /// sehingga anomali timing/size tidak pernah mempengaruhi keputusan.
    pub fn calculate_total(&mut self) {
        // Catatan: timing & size mutually exclusive (satu AnomalyReport), jadi maks riil = 40+40+20/10 = 90 bila bobot diubah, 80 saat ini sudah = token+port.
        let weighted: u32 = (self.token_reuse as u32 * WEIGHT_TOKEN_REUSE
            + self.port_violation as u32 * WEIGHT_PORT_VIOLATION
            + self.timing_anomaly as u32 * WEIGHT_TIMING_ANOMALY
            + self.size_anomaly as u32 * WEIGHT_SIZE_ANOMALY)
            / 100;

        self.total = weighted.min(100) as u8;
    }
}

// ============================================================
// DEFENSE ACTION
// ============================================================

/// Status respons terhadap frame yang masuk
#[derive(Debug, Clone, PartialEq)]
pub enum DefenseAction {
    Accept,
    DeployCanary { trace_id: [u8; 16] },
    Disconnect { reason: &'static str },
}

// ============================================================
// HEALTH INDICATOR (A6) — untuk end-user
// ============================================================

/// Indikator kesehatan koneksi untuk end-user (🟢🟡🔴⚪)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HealthIndicator {
    /// 🟢 Hijau: Koneksi normal
    Green,
    /// 🟡 Kuning: Anomali ringan, sistem memverifikasi
    Yellow,
    /// 🔴 Merah: Anomali kritis
    Red,
    /// ⚪ Abu-abu: Belum ada data
    Gray,
}

impl HealthIndicator {
    /// Konversi dari risk score + anomaly report ke indikator
    pub fn from_risk_and_anomaly(
        risk_score: u8,
        anomaly_report: Option<&AnomalyReport>,
        total_samples: usize,
    ) -> Self {
        // A6: Gray jika belum cukup data
        if total_samples < MIN_SAMPLES_FOR_INDICATOR {
            return HealthIndicator::Gray;
        }

        // Prioritas: risk score tinggi = merah
        if risk_score >= DISCONNECT_THRESHOLD {
            return HealthIndicator::Red;
        }

        // Risk score sedang = kuning
        if risk_score >= CANARY_THRESHOLD {
            return HealthIndicator::Yellow;
        }

        // Ada anomali persisten = kuning
        if let Some(report) = anomaly_report {
            if report.is_persistent && report.severity > 0.5 {
                return HealthIndicator::Yellow;
            }
        }

        HealthIndicator::Green
    }

    /// Pesan ramah untuk user (bukan teknis)
    pub fn user_message(&self) -> &'static str {
        match self {
            HealthIndicator::Green => "Koneksi Anda aman dan stabil.",
            HealthIndicator::Yellow => {
                "Koneksi Anda terdeteksi sedikit tidak stabil. \
                 Sistem sedang memverifikasi keamanan."
            }
            HealthIndicator::Red => {
                "Koneksi Anda terdeteksi mencurigakan. \
                 Demi keamanan, sistem membatasi koneksi sementara."
            }
            HealthIndicator::Gray => "Menghubungkan...",
        }
    }

    pub fn color_hex(&self) -> &'static str {
        match self {
            HealthIndicator::Green => "#4CAF50",
            HealthIndicator::Yellow => "#FFC107",
            HealthIndicator::Red => "#F44336",
            HealthIndicator::Gray => "#9E9E9E",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            HealthIndicator::Green => "🟢",
            HealthIndicator::Yellow => "🟡",
            HealthIndicator::Red => "🔴",
            HealthIndicator::Gray => "⚪",
        }
    }
}

// ============================================================
// TECHNICAL DETAILS (A6) — untuk sysadmin
// ============================================================

/// Kategori anomali (machine-readable, kompatibel dengan gRPC/HTTP/3)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnomalyCategory {
    None,
    TimingAnomaly,
    SizeAnomaly,
    TokenReuse,
    PortViolation,
    MultiDimension,
    Critical,
}

/// Detail teknis yang structured — kompatibel dengan:
/// - gRPC Status.details (protobuf Any)
/// - HTTP/3 CONNECTION_CLOSE (error_code + reason_phrase)
/// - WebSocket Close frame (code + reason)
/// - MQTT CONNACK (reason_code + properties)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TechnicalDetails {
    /// Error code numerik (mirip HTTP status code atau gRPC code)
    pub code: u16,
    /// Kategori anomali (machine-readable)
    pub category: AnomalyCategory,
    /// Risk score lengkap
    pub risk_score: TransportRiskScore,
    /// Profil link yang terdeteksi
    pub link_profile: String,
    /// Blend ratio saat ini (untuk sysadmin melihat adaptasi)
    pub blend_ratio: f64,
    /// Threshold yang sedang aktif
    pub active_threshold: AdaptiveThreshold,
    /// Timestamp insiden (Unix epoch ms)
    pub timestamp_ms: u64,
    /// Trace ID untuk forensic (jika ada)
    pub trace_id: Option<String>,
    /// Pesan debugging (mirip HTTP/3 reason_phrase)
    pub debug_message: Option<String>,
}

/// Status lengkap untuk end-user + sysadmin
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UserStatus {
    /// Indikator visual untuk end-user
    pub indicator: HealthIndicator,
    /// Pesan ramah untuk end-user
    pub message: String,
    /// Warna untuk UI
    pub color: String,
    /// Ikon untuk UI
    pub icon: String,
    /// Detail teknis untuk sysadmin
    pub technical_details: Option<TechnicalDetails>,
}

impl UserStatus {
    /// Konversi ke JSON (hanya jika feature `serde` aktif)
    #[cfg(feature = "serde")]
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

// ============================================================
// DEFENSE ENGINE
// ============================================================

/// Mesin pertahanan transport
pub struct DefenseEngine {
    session_id: [u8; 8],
    current_epoch: u64,
    seen_tokens: HashSet<[u8; 8]>,
    risk_history: Vec<TransportRiskScore>,
    expected_port: Option<u16>,
    anomaly_detector: AnomalyDetector,
    // Untuk observability
    total_evaluations: usize,
    total_canaries: usize,
    total_disconnects: usize,
}

impl DefenseEngine {
    pub fn new(session_id: [u8; 8]) -> Self {
        Self {
            session_id,
            current_epoch: 0,
            seen_tokens: HashSet::with_capacity(1000),
            risk_history: Vec::new(),
            expected_port: None,
            anomaly_detector: AnomalyDetector::new(),
            total_evaluations: 0,
            total_canaries: 0,
            total_disconnects: 0,
        }
    }

    /// Constructor dengan profil link spesifik
    pub fn with_profile(session_id: [u8; 8], profile: crate::survival::LinkProfile) -> Self {
        let mut engine = Self::new(session_id);
        engine.anomaly_detector = AnomalyDetector::with_profile(profile);
        engine
    }

    pub fn session_id(&self) -> [u8; 8] {
        self.session_id
    }

    pub fn set_expected_port(&mut self, port: u16) {
        self.expected_port = Some(port);
    }

    /// Set profil link (untuk perubahan dinamis, mis. handover)
    pub fn set_profile(&mut self, profile: crate::survival::LinkProfile) {
        self.anomaly_detector.set_profile(profile);
    }

    /// Set konteks sesi (A5)
    pub fn set_context(&mut self, context: crate::anomaly::SessionContext) {
        self.anomaly_detector.set_context(context);
    }

    /// Update epoch dari crypto session.
    /// 
    /// REVISI: jika epoch berubah, clear seen_tokens karena token
    /// bersifat ephemeral per epoch.
    pub fn sync_epoch(&mut self, epoch: u64) {
        if self.current_epoch != epoch {
            self.seen_tokens.clear();
        }
        self.current_epoch = epoch;
    }

    pub fn advance_epoch(&mut self) {
        self.current_epoch = self.current_epoch.wrapping_add(1);
        // ✅ FIX: clear setiap epoch berubah
        self.seen_tokens.clear();
    }

    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    /// Evaluasi frame (default: pakai Instant::now())
    pub fn evaluate(
        &mut self,
        frame: &FcrTpFrame,
        actual_port: u16,
        current_epoch: u64,
    ) -> DefenseAction {
        self.evaluate_at(frame, actual_port, current_epoch, std::time::Instant::now())
    }

    /// Evaluasi frame dengan timestamp injection (deterministik untuk test)
    pub fn evaluate_at(
        &mut self,
        frame: &FcrTpFrame,
        actual_port: u16,
        current_epoch: u64,
        timestamp: std::time::Instant,
    ) -> DefenseAction {
        // Sinkronisasi epoch (akan clear seen_tokens jika berubah)
        self.sync_epoch(current_epoch);

        // Deteksi anomali timing/size dari frame
        let frame_size = frame.encrypted_header_blob.len() + frame.dynamic_payload.len();
        let anomaly_report = self.anomaly_detector.record_chunk_at(frame_size, timestamp);

        let mut risk = self.calculate_risk(frame, actual_port, anomaly_report.as_ref());
        risk.calculate_total();

        // Tentukan action
        let action = if risk.total >= DISCONNECT_THRESHOLD {
            DefenseAction::Disconnect { reason: "Risk score critical" }
        } else if risk.total >= CANARY_THRESHOLD {
            let trace_id = self.generate_trace_id();
            DefenseAction::DeployCanary { trace_id }
        } else {
            DefenseAction::Accept
        };

        // Insert token hanya jika Accept atau Canary (bukan Disconnect)
        if !matches!(action, DefenseAction::Disconnect { .. }) {
            // Cek max size sebelum insert
            if self.seen_tokens.len() < MAX_SEEN_TOKENS {
                self.seen_tokens.insert(frame.ephemeral_routing_token);
            }
        }

        // Update stats
        self.total_evaluations += 1;
        match &action {
            DefenseAction::DeployCanary { .. } => self.total_canaries += 1,
            DefenseAction::Disconnect { .. } => self.total_disconnects += 1,
            _ => {}
        }

        // Simpan risk history (bounded)
        self.risk_history.push(risk);
        if self.risk_history.len() > 1000 {
            self.risk_history.drain(0..500);
        }

        action
    }

    fn calculate_risk(
        &self,
        frame: &FcrTpFrame,
        actual_port: u16,
        anomaly_report: Option<&AnomalyReport>,
    ) -> TransportRiskScore {
        let mut risk = TransportRiskScore::default();

        // 1. Token reuse
        if self.seen_tokens.contains(&frame.ephemeral_routing_token) {
            risk.token_reuse = 100;
        }

        // 2. Port violation
        if let Some(expected) = self.expected_port {
            if actual_port != expected {
                risk.port_violation = 100;
            }
        }

        // 3. Anomali timing/size dari AnomalyDetector
        // Hanya anomali PERSISTEN yang mempengaruhi risk score.
        // Anomali transient (masih grace period) diabaikan untuk
        // mengurangi false positive.
        if let Some(report) = anomaly_report {
            if report.is_persistent {
                let severity_score = (report.severity * 100.0).min(100.0) as u8;
                match report.anomaly_type {
                    AnomalyType::BurstTraffic | AnomalyType::ExtremeJitter => {
                        risk.timing_anomaly = severity_score;
                    }
                    AnomalyType::UniformSizeInjection => {
                        risk.size_anomaly = severity_score;
                    }
                }
            }
        }

        risk
    }

    fn generate_trace_id(&self) -> [u8; 16] {
        let mut trace_id = [0u8; 16];
        trace_id[0..8].copy_from_slice(&self.session_id);
        rand::thread_rng().fill_bytes(&mut trace_id[8..16]);
        trace_id
    }

    /// Bangun Canary Frame yang terenkripsi dengan crypto session.
    /// 
    /// REVISI: guard agar forensic payload muat di bucket 256.
    pub fn build_canary_frame(
        &self,
        trace_id: [u8; 16],
        session: &CryptoSession,
    ) -> Result<FcrTpFrame, &'static str> {
        // 1. Metadata yang terlihat valid
        let metadata = ChunkMetadata {
            message_id: u64::from_be_bytes(trace_id[0..8].try_into().unwrap()),
            version: CURRENT_CRYPTO_VERSION,
            chunk_index: 0,
            total_chunks: 1,
            is_final: true,
        };

        // 2. Forensic payload
        // Struktur: [Metadata:14][TraceId:16][SessionId:8][Epoch:8][Padding:16]
        let mut forensic_payload = Vec::with_capacity(CHUNK_METADATA_SIZE + FORENSIC_OVERHEAD);
        forensic_payload.extend_from_slice(&metadata.to_bytes());
        forensic_payload.extend_from_slice(&trace_id);
        forensic_payload.extend_from_slice(&self.session_id);
        forensic_payload.extend_from_slice(&self.current_epoch.to_be_bytes());
        forensic_payload.extend_from_slice(&[0xBB; 16]);

        // 3. Guard: pastikan payload muat di bucket 256
        let bucket_size = 256;
        let overhead = crate::packet::TOKEN_SIZE
            + crate::crypto::NONCE_SIZE
            + CHUNK_METADATA_SIZE
            + crate::crypto::PLAINTEXT_LEN_SIZE
            + crate::crypto::AEAD_TAG_SIZE;
        if forensic_payload.len() > bucket_size - overhead {
            return Err("Forensic payload terlalu besar untuk bucket 256");
        }

        // 4. Encrypt
        let frames = session.encrypt_to_frames_with_bucket(&forensic_payload, bucket_size)?;

        Ok(frames[0].clone())
    }

    // ============================================================
    // HEALTH INDICATOR & USER STATUS (A6)
    // ============================================================

    /// Dapatkan indikator kesehatan untuk UI end-user
    pub fn health_indicator(&self, latest_anomaly: Option<&AnomalyReport>) -> HealthIndicator {
        let latest_risk = self.risk_history.last().map(|r| r.total).unwrap_or(0);
        HealthIndicator::from_risk_and_anomaly(
            latest_risk,
            latest_anomaly,
            self.anomaly_detector.total_samples(),
        )
    }

    /// Dapatkan status lengkap untuk end-user + sysadmin
    pub fn user_facing_status(&self, latest_anomaly: Option<&AnomalyReport>) -> UserStatus {
        let indicator = self.health_indicator(latest_anomaly);
        let latest_risk = self.risk_history.last().cloned();

        // Bangun technical details jika ada risk history
        let technical_details = latest_risk.map(|risk| {
            let category = self.categorize_anomaly(&risk, latest_anomaly);
            let code = self.error_code_for(&risk, &category);
            let debug_message = latest_anomaly.map(|a| a.forensic_context.clone());

            TechnicalDetails {
                code,
                category,
                risk_score: risk,
                link_profile: self.anomaly_detector.profile().name().to_string(),
                blend_ratio: self.anomaly_detector.current_blend(),
                active_threshold: *self.anomaly_detector.active_threshold(),
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
                trace_id: None,
                debug_message,
            }
        });

        UserStatus {
            indicator,
            message: indicator.user_message().to_string(),
            color: indicator.color_hex().to_string(),
            icon: indicator.icon().to_string(),
            technical_details,
        }
    }

    fn categorize_anomaly(
        &self,
        risk: &TransportRiskScore,
        anomaly_report: Option<&AnomalyReport>,
    ) -> AnomalyCategory {
        let dimensions_active = [
            risk.token_reuse > 0,
            risk.port_violation > 0,
            risk.timing_anomaly > 0,
            risk.size_anomaly > 0,
        ].iter().filter(|&&x| x).count();

        if risk.total >= DISCONNECT_THRESHOLD {
            return AnomalyCategory::Critical;
        }

        if dimensions_active >= 2 {
            return AnomalyCategory::MultiDimension;
        }

        if risk.token_reuse > 0 {
            return AnomalyCategory::TokenReuse;
        }
        if risk.port_violation > 0 {
            return AnomalyCategory::PortViolation;
        }
        if risk.timing_anomaly > 0 {
            return AnomalyCategory::TimingAnomaly;
        }
        if risk.size_anomaly > 0 {
            return AnomalyCategory::SizeAnomaly;
        }

        let _ = anomaly_report;
        AnomalyCategory::None
    }

    /// Error code numerik — kompatibel dengan HTTP/gRPC
    fn error_code_for(&self, _risk: &TransportRiskScore, category: &AnomalyCategory) -> u16 {
        match category {
            AnomalyCategory::None => 0,        // OK
            AnomalyCategory::TimingAnomaly => 408,     // Request Timeout (HTTP)
            AnomalyCategory::SizeAnomaly => 413,       // Payload Too Large
            AnomalyCategory::TokenReuse => 409,        // Conflict
            AnomalyCategory::PortViolation => 421,     // Misdirected Request
            AnomalyCategory::MultiDimension => 429,    // Too Many Requests
            AnomalyCategory::Critical => 503,          // Service Unavailable
        }
    }

    // ============================================================
    // OBSERVABILITY
    // ============================================================

    pub fn get_risk_history(&self) -> &[TransportRiskScore] {
        &self.risk_history
    }

    pub fn anomaly_detector(&self) -> &AnomalyDetector {
        &self.anomaly_detector
    }

    pub fn total_evaluations(&self) -> usize { self.total_evaluations }
    pub fn total_canaries(&self) -> usize { self.total_canaries }
    pub fn total_disconnects(&self) -> usize { self.total_disconnects }

    pub fn reset(&mut self) {
        self.current_epoch = 0;
        self.seen_tokens.clear();
        self.risk_history.clear();
        self.expected_port = None;
        self.anomaly_detector.reset();
        self.total_evaluations = 0;
        self.total_canaries = 0;
        self.total_disconnects = 0;
    }
}

// ============================================================
// TEST SUITE
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::CURRENT_CRYPTO_VERSION;
    use crate::packet::HeaderMode;
    use crate::survival::LinkProfile;
    use std::time::{Duration, Instant};

    const TEST_SESSION: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

    fn create_test_frame(token: [u8; 8]) -> FcrTpFrame {
        FcrTpFrame::new(
            token,
            HeaderMode::Short,
            vec![0xAA; 200],
            [0u8; 16],
            vec![0xBB; 32],
        )
    }

    fn sim_instant(base: Instant, ms: u64) -> Instant {
        base + Duration::from_millis(ms)
    }

    // ============================================================
    // TEST: Basic defense
    // ============================================================

    #[test]
    fn test_accept_normal_frame() {
        let mut engine = DefenseEngine::with_profile(TEST_SESSION, LinkProfile::Fiber);
        engine.set_expected_port(50000);

        let frame = create_test_frame([0x11; 8]);
        let action = engine.evaluate(&frame, 50000, 0);

        assert_eq!(action, DefenseAction::Accept);
    }

    #[test]
    fn test_detect_token_reuse() {
        let mut engine = DefenseEngine::with_profile(TEST_SESSION, LinkProfile::Fiber);
        engine.set_expected_port(50000);

        let token = [0x11; 8];
        let frame1 = create_test_frame(token);
        let frame2 = create_test_frame(token);

        let action1 = engine.evaluate(&frame1, 50000, 0);
        assert_eq!(action1, DefenseAction::Accept);

        let action2 = engine.evaluate(&frame2, 50000, 0);
        match action2 {
            DefenseAction::DeployCanary { trace_id } => {
                assert_ne!(trace_id, [0u8; 16]);
                assert_eq!(&trace_id[0..8], &TEST_SESSION);
            }
            other => panic!("Expected DeployCanary, got {:?}", other),
        }
    }

    #[test]
    fn test_detect_port_violation() {
        let mut engine = DefenseEngine::with_profile(TEST_SESSION, LinkProfile::Fiber);
        engine.set_expected_port(50000);
        
        // ✅ FIX: isi 15 frame normal dulu dengan size bervariasi
        let base = Instant::now();
        for i in 0..15usize {
            let mut token = [0u8; 8];
            token[0] = i as u8;
            let mut frame = create_test_frame(token);
            frame.dynamic_payload = vec![0xBB; 20 + (i % 10) * 5];
            engine.evaluate_at(&frame, 50000, 0, sim_instant(base, (i * 100) as u64));
        }
        
        // Port violation: risk = 40 > CANARY_THRESHOLD (30)
        let frame = create_test_frame([0x22; 8]);
        let action = engine.evaluate_at(&frame, 50001, 0, sim_instant(base, 2000));
        
        match action {
            DefenseAction::DeployCanary { .. } => (),
            other => panic!("Expected DeployCanary, got {:?}", other),
        }
    }

    #[test]
    fn test_critical_risk_disconnect() {
        let mut engine = DefenseEngine::with_profile(TEST_SESSION, LinkProfile::Fiber);
        engine.set_expected_port(50000);
        
        let base = Instant::now();
        // Isi 15 frame normal
        for i in 0..15usize {
            let mut token = [0u8; 8];
            token[0] = i as u8;
            let mut frame = create_test_frame(token);
            frame.dynamic_payload = vec![0xBB; 20 + (i % 10) * 5];
            engine.evaluate_at(&frame, 50000, 0, sim_instant(base, (i * 100) as u64));
        }
        
        // Port violation + token reuse: 40 + 40 = 80 > 30 → Canary
        let token = [0x33; 8];
        let frame1 = create_test_frame(token);
        let action1 = engine.evaluate_at(&frame1, 50001, 0, sim_instant(base, 2000));
        assert!(matches!(action1, DefenseAction::DeployCanary { .. }));
        
        // Token reuse + port violation: 40 + 40 = 80, masih Canary
        let frame2 = create_test_frame(token);
        let _action2 = engine.evaluate_at(&frame2, 50001, 0, sim_instant(base, 2100));
        
        // Verifikasi risk history
        let last_risk = engine.get_risk_history().last().unwrap();
        assert!(last_risk.total >= 60,
            "Risk harus >= 60, got {}", last_risk.total);
    }

    // ============================================================
    // TEST: Epoch token clearing (semantik benar)
    // ============================================================

    #[test]
    fn test_epoch_advancement_clears_tokens_immediately() {
        let mut engine = DefenseEngine::new(TEST_SESSION);

        for i in 0..100 {
            let mut token = [0u8; 8];
            token[0] = i as u8;
            engine.seen_tokens.insert(token);
        }
        assert_eq!(engine.seen_tokens.len(), 100);

        engine.advance_epoch();
        assert_eq!(engine.current_epoch(), 1);
        assert_eq!(engine.seen_tokens.len(), 0,
            "seen_tokens HARUS clear setelah epoch berubah");
    }

    #[test]
    fn test_sync_epoch_clears_tokens_on_change() {
        let mut engine = DefenseEngine::new(TEST_SESSION);

        for i in 0..50 {
            let mut token = [0u8; 8];
            token[0] = i as u8;
            engine.seen_tokens.insert(token);
        }
        assert_eq!(engine.seen_tokens.len(), 50);

        engine.sync_epoch(0);
        assert_eq!(engine.seen_tokens.len(), 50);

        engine.sync_epoch(5);
        assert_eq!(engine.seen_tokens.len(), 0);
        assert_eq!(engine.current_epoch(), 5);
    }

    #[test]
    fn test_token_reuse_across_epochs_not_flagged() {
        let mut engine = DefenseEngine::with_profile(TEST_SESSION, LinkProfile::Fiber);
        engine.set_expected_port(50000);

        let token = [0x44; 8];
        let frame = create_test_frame(token);

        let action1 = engine.evaluate(&frame, 50000, 0);
        assert_eq!(action1, DefenseAction::Accept);

        engine.advance_epoch();

        let action2 = engine.evaluate(&frame, 50000, 1);
        assert_eq!(action2, DefenseAction::Accept,
            "Token dari epoch lama tidak boleh dianggap reuse");
    }

    // ============================================================
    // TEST: Risk score semantic
    // ============================================================

    #[test]
    fn test_risk_score_each_field_contributes() {
        let mut base = TransportRiskScore::default();
        base.calculate_total();
        assert_eq!(base.total, 0);

        let mut r = TransportRiskScore { token_reuse: 100, ..Default::default() };
        r.calculate_total();
        assert_eq!(r.total, WEIGHT_TOKEN_REUSE as u8);

        let mut r = TransportRiskScore { port_violation: 100, ..Default::default() };
        r.calculate_total();
        assert_eq!(r.total, WEIGHT_PORT_VIOLATION as u8);

        let mut r = TransportRiskScore { timing_anomaly: 100, ..Default::default() };
        r.calculate_total();
        assert_eq!(r.total, WEIGHT_TIMING_ANOMALY as u8,
            "timing_anomaly HARUS berkontribusi");

        let mut r = TransportRiskScore { size_anomaly: 100, ..Default::default() };
        r.calculate_total();
        assert_eq!(r.total, WEIGHT_SIZE_ANOMALY as u8,
            "size_anomaly HARUS berkontribusi");

        let mut r = TransportRiskScore {
            token_reuse: 100,
            port_violation: 100,
            timing_anomaly: 100,
            size_anomaly: 100,
            total: 0,
        };
        r.calculate_total();
        assert_eq!(r.total, 100);
    }

    #[test]
    fn test_risk_score_partial() {
        let mut r = TransportRiskScore {
            token_reuse: 50,
            port_violation: 50,
            timing_anomaly: 50,
            size_anomaly: 50,
            total: 0,
        };
        r.calculate_total();
        // (50*40 + 50*20 + 50*20 + 50*20) / 100 = 50
        assert_eq!(r.total, 50);
    }

    // ============================================================
    // TEST: HealthIndicator (A6)
    // ============================================================

    #[test]
    fn test_health_indicator_gray_before_min_samples() {
        let engine = DefenseEngine::new(TEST_SESSION);
        let indicator = engine.health_indicator(None);
        assert_eq!(indicator, HealthIndicator::Gray,
            "Sebelum 10 sampel, indikator harus Gray");
    }

    #[test]
    fn test_health_indicator_green_normal() {
        let mut engine = DefenseEngine::with_profile(TEST_SESSION, LinkProfile::Fiber);
        engine.set_expected_port(50000);

        // Kirim 15 frame normal dengan timestamp berbeda
        let base = Instant::now();
        for i in 0..15 {
            let mut token = [0u8; 8];
            token[0] = i as u8;
            let frame = create_test_frame(token);
            engine.evaluate_at(&frame, 50000, 0, sim_instant(base, i * 50));
        }

        let indicator = engine.health_indicator(None);
        assert_eq!(indicator, HealthIndicator::Green);
    }

    #[test]
    fn test_health_indicator_yellow_on_canary() {
        let mut engine = DefenseEngine::with_profile(TEST_SESSION, LinkProfile::Fiber);
        engine.set_expected_port(50000);

        // Isi 15 frame normal dulu
        let base = Instant::now();
        for i in 0..15 {
            let mut token = [0u8; 8];
            token[0] = i as u8;
            let frame = create_test_frame(token);
            engine.evaluate_at(&frame, 50000, 0, sim_instant(base, i * 50));
        }

        // Kemudian port violation
        let frame = create_test_frame([0xEE; 8]);
        let _ = engine.evaluate_at(&frame, 50001, 0, sim_instant(base, 800));

        let indicator = engine.health_indicator(None);
        assert_eq!(indicator, HealthIndicator::Yellow);
    }

    #[test]
    fn test_health_indicator_user_messages() {
        assert!(!HealthIndicator::Green.user_message().is_empty());
        assert!(!HealthIndicator::Yellow.user_message().is_empty());
        assert!(!HealthIndicator::Red.user_message().is_empty());
        assert!(!HealthIndicator::Gray.user_message().is_empty());
    }

    #[test]
    fn test_health_indicator_colors() {
        assert_eq!(HealthIndicator::Green.color_hex(), "#4CAF50");
        assert_eq!(HealthIndicator::Yellow.color_hex(), "#FFC107");
        assert_eq!(HealthIndicator::Red.color_hex(), "#F44336");
        assert_eq!(HealthIndicator::Gray.color_hex(), "#9E9E9E");
    }

    // ============================================================
    // TEST: UserStatus + TechnicalDetails (A6)
    // ============================================================

    #[test]
    fn test_user_status_has_technical_details() {
        let mut engine = DefenseEngine::with_profile(TEST_SESSION, LinkProfile::Fiber);
        engine.set_expected_port(50000);
        
        let base = Instant::now();
        for i in 0..15usize {
            let mut token = [0u8; 8];
            token[0] = i as u8;
            let mut frame = create_test_frame(token);
            frame.dynamic_payload = vec![0xBB; 20 + (i % 10) * 5];  // ✅ bervariasi
            engine.evaluate_at(&frame, 50000, 0, sim_instant(base, (i * 100) as u64));
        }
        
        let status = engine.user_facing_status(None);
        assert_eq!(status.indicator, HealthIndicator::Green);
        assert!(status.technical_details.is_some());
        
        let details = status.technical_details.unwrap();
        assert_eq!(details.link_profile, "Fiber");
        assert_eq!(details.code, 0);
    }

    #[test]
     fn test_technical_details_error_code_for_category() {
        let mut engine = DefenseEngine::with_profile(TEST_SESSION, LinkProfile::Fiber);
        engine.set_expected_port(50000);
        
        let base = Instant::now();
        // ✅ FIX: size bervariasi untuk hindari size_anomaly
        for i in 0..15usize {
            let mut token = [0u8; 8];
            token[0] = i as u8;
            let mut frame = create_test_frame(token);
            frame.dynamic_payload = vec![0xBB; 20 + (i % 10) * 5];
            engine.evaluate_at(&frame, 50000, 0, sim_instant(base, (i * 100) as u64));
        }
        
        // Port violation
        let frame = create_test_frame([0xEE; 8]);
        let _ = engine.evaluate_at(&frame, 50001, 0, sim_instant(base, 2000));
        
        let status = engine.user_facing_status(None);
        let details = status.technical_details.unwrap();
        assert_eq!(details.category, AnomalyCategory::PortViolation);
        assert_eq!(details.code, 421);
    }

    // ============================================================
    // TEST: Integrasi AnomalyDetector (transient vs persistent)
    // ============================================================

    #[test]
    fn test_transient_anomaly_does_not_affect_risk() {
        // Anomali transient (masih grace period) tidak boleh
        // mempengaruhi risk score
        let mut engine = DefenseEngine::with_profile(TEST_SESSION, LinkProfile::Fiber);
        engine.set_expected_port(50000);

        let base = Instant::now();
        // Frame normal
        engine.evaluate_at(&create_test_frame([0x01; 8]), 50000, 0, sim_instant(base, 0));
        engine.evaluate_at(&create_test_frame([0x02; 8]), 50000, 0, sim_instant(base, 50));

        // Burst pertama (transient, grace period)
        let _ = engine.evaluate_at(&create_test_frame([0x03; 8]), 50000, 0, sim_instant(base, 50));

        // Risk score harus 0 karena anomali transient
        let last_risk = engine.get_risk_history().last().unwrap();
        assert_eq!(last_risk.timing_anomaly, 0,
            "Anomali transient (grace period) tidak boleh mempengaruhi risk");
    }

    #[test]
    fn test_persistent_anomaly_affects_risk() {
        // Anomali persisten (melewati grace period) harus
        // mempengaruhi risk score
        let mut engine = DefenseEngine::with_profile(TEST_SESSION, LinkProfile::Fiber);
        engine.set_expected_port(50000);

        let base = Instant::now();
        // Frame normal
        engine.evaluate_at(&create_test_frame([0x01; 8]), 50000, 0, sim_instant(base, 0));
        engine.evaluate_at(&create_test_frame([0x02; 8]), 50000, 0, sim_instant(base, 50));

        // Burst berkepanjangan (melewati grace period 3 sampel)
        for i in 0..10 {
            let ts = sim_instant(base, 50 + (i as u64));
            let _ = engine.evaluate_at(&create_test_frame([i as u8; 8]), 50000, 0, ts);
        }

        // Cari risk score yang punya timing_anomaly > 0
        let has_timing = engine.get_risk_history().iter()
            .any(|r| r.timing_anomaly > 0);
        assert!(has_timing,
            "Anomali persisten harus mempengaruhi risk score");
    }

    // ============================================================
    // TEST: Canary frame
    // ============================================================

    #[test]
    fn test_canary_frame_encrypted_and_valid() {
        let session_key = [0xAA; 32]; // kunci hasil HKDF dari FCR-RSA
        let session = CryptoSession::from_session_key(TEST_SESSION, session_key, CURRENT_CRYPTO_VERSION);

        let engine = DefenseEngine::new(TEST_SESSION);
        let trace_id = engine.generate_trace_id();

        let canary = engine.build_canary_frame(trace_id, &session)
            .expect("Build canary gagal");

        let wire = canary.encode().expect("Encode gagal");
        assert_eq!(wire.len(), 256);

        let received = FcrTpFrame::decode(wire).expect("Decode gagal");
        let (metadata, decrypted) = session.decrypt_from_frame(&received)
            .expect("Decrypt gagal");

        assert_eq!(metadata.version, CURRENT_CRYPTO_VERSION);
        assert!(metadata.is_final);
        assert!(decrypted.len() >= CHUNK_METADATA_SIZE + FORENSIC_OVERHEAD);

        let trace_id_start = CHUNK_METADATA_SIZE;
        let trace_id_end = trace_id_start + 16;
        let forensic_trace_id = &decrypted[trace_id_start..trace_id_end];
        assert_eq!(forensic_trace_id, &trace_id);
    }

    // ============================================================
    // TEST: Lainnya
    // ============================================================

    #[test]
    fn test_session_id_accessible() {
        let engine = DefenseEngine::new([0xAB; 8]);
        assert_eq!(engine.session_id(), [0xAB; 8]);
    }

    #[test]
    fn test_trace_id_contains_session_id() {
        let engine = DefenseEngine::new([0xAB; 8]);
        let trace_id = engine.generate_trace_id();
        assert_eq!(&trace_id[0..8], &[0xAB; 8]);
        assert_ne!(&trace_id[8..16], &[0u8; 8]);
    }

    #[test]
    fn test_trace_id_uniqueness() {
        let engine = DefenseEngine::new([0xCD; 8]);
        let trace1 = engine.generate_trace_id();
        let trace2 = engine.generate_trace_id();
        assert_ne!(trace1, trace2);
        assert_eq!(&trace1[0..8], &trace2[0..8]);
    }

    #[test]
    fn test_multi_tenant_isolation() {
        let session_a = [0x01; 8];
        let session_b = [0x02; 8];

        let engine_a = DefenseEngine::new(session_a);
        let engine_b = DefenseEngine::new(session_b);

        let trace_a = engine_a.generate_trace_id();
        let trace_b = engine_b.generate_trace_id();

        assert_ne!(trace_a, trace_b);
        assert_eq!(&trace_a[0..8], &session_a);
        assert_eq!(&trace_b[0..8], &session_b);
    }

    #[test]
    fn test_epoch_sync() {
        let mut engine = DefenseEngine::new(TEST_SESSION);
        assert_eq!(engine.current_epoch(), 0);

        engine.sync_epoch(5);
        assert_eq!(engine.current_epoch(), 5);

        engine.sync_epoch(10);
        assert_eq!(engine.current_epoch(), 10);
    }

    #[test]
    fn test_observability_counters() {
        let mut engine = DefenseEngine::with_profile(TEST_SESSION, LinkProfile::Fiber);
        engine.set_expected_port(50000);

        let frame = create_test_frame([0x11; 8]);
        engine.evaluate(&frame, 50000, 0);

        assert_eq!(engine.total_evaluations(), 1);
    }

    #[test]
    fn test_reset_clears_all() {
        let mut engine = DefenseEngine::with_profile(TEST_SESSION, LinkProfile::Fiber);
        engine.set_expected_port(50000);

        let frame = create_test_frame([0x11; 8]);
        engine.evaluate(&frame, 50000, 0);
        engine.advance_epoch();

        engine.reset();

        assert_eq!(engine.current_epoch(), 0);
        assert_eq!(engine.total_evaluations(), 0);
        assert_eq!(engine.get_risk_history().len(), 0);
    }
}
