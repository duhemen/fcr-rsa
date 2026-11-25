//! FCR-TP Defense Layer
//! 
//! Lapisan pertahanan transport yang mendeteksi anomali di layer jaringan dan merespons
//! dengan Canary Pill (data forensik beracun) ketika ancaman terdeteksi.
//! 
//! TIGA PILAR PERTAHANAN:
//! 1. Transport-Level Auto-Mune: Deteksi anomali timing, size, port, token
//! 2. MITM Protection: Session binding, downgrade prevention, directional nonce
//! 3. Transport-Level Canary Pill: Frame valid yang berisi forensic trace

use std::collections::HashSet;
use rand::RngCore;
use zeroize::Zeroize;

use crate::packet::{FcrTpFrame, HeaderMode};
use crate::crypto::{ChunkMetadata, CHUNK_METADATA_SIZE};

/// Ambang batas risiko untuk mengaktifkan Canary Pill (0-100)
/// Token reuse (70 poin) atau port violation (30 poin) saja sudah cukup trigger canary
pub const CANARY_THRESHOLD: u8 = 30;

/// Ambang batas risiko untuk memutus koneksi (0-100)
/// Token reuse + port violation = 100 poin → disconnect
pub const DISCONNECT_THRESHOLD: u8 = 100;

/// Maksimum token yang disimpan untuk deteksi reuse (mencegah memory exhaustion)
pub const MAX_SEEN_TOKENS: usize = 10_000;

/// Ukuran forensic payload tambahan (di luar metadata): TraceId (16) + SessionId (8) + Epoch (8) + Padding (16)
pub const FORENSIC_OVERHEAD: usize = 16 + 8 + 8 + 16;

/// Skor risiko transport (0 = aman, 100 = sangat mencurigakan)
#[derive(Debug, Clone, Default, Zeroize)]
pub struct TransportRiskScore {
    /// Anomali timing: paket datang dengan pola tidak wajar (burst, jitter ekstrem)
    pub timing_anomaly: u8,
    /// Anomali ukuran: distribusi ukuran paket tidak sesuai pola normal
    pub size_anomaly: u8,
    /// Pelanggaran port hopping: paket datang di port yang tidak sesuai jadwal
    pub port_violation: u8,
    /// Token reuse: token routing yang sama muncul dua kali dalam epoch berbeda
    pub token_reuse: u8,
    /// Skor total (rata-rata tertimbang)
    pub total: u8,
}

impl TransportRiskScore {
    /// Hitung skor total dari komponen individual
    pub fn calculate_total(&mut self) {
        // Bobot baru: token_reuse (70%), port_violation (30%)
        // timing_anomaly dan size_anomaly akan diimplementasi nanti
        let weighted = (self.token_reuse as u16 * 70
            + self.port_violation as u16 * 30
            + self.timing_anomaly as u16 * 0
            + self.size_anomaly as u16 * 0) / 100;
        
        self.total = weighted.min(100) as u8;
    }
}

/// Status respons terhadap frame yang masuk
#[derive(Debug, Clone)]
pub enum DefenseAction {
    /// Frame normal, proses seperti biasa
    Accept,
    /// Frame mencurigakan, balas dengan Canary Pill
    DeployCanary { trace_id: [u8; 16] },
    /// Frame sangat berbahaya, putus koneksi setelah cooldown
    Disconnect { reason: &'static str },
}

/// Mesin pertahanan transport
pub struct DefenseEngine {
    /// ID sesi unik — digunakan untuk forensic traceability dan multi-tenant safety
    session_id: [u8; 8],
    current_epoch: u64,
    seen_tokens: HashSet<[u8; 8]>,
    risk_history: Vec<TransportRiskScore>,
    expected_port: Option<u16>, // Untuk validasi port hopping
}

impl DefenseEngine {
    pub fn new(session_id: [u8; 8]) -> Self {
        Self {
            session_id,
            current_epoch: 0,
            seen_tokens: HashSet::with_capacity(1000),
            risk_history: Vec::new(),
            expected_port: None,
        }
    }

    /// Aksesor untuk session_id (dipanggil oleh layer di atas untuk logging/telemetry)
    pub fn session_id(&self) -> [u8; 8] {
        self.session_id
    }

    /// Set port yang diharapkan untuk epoch saat ini (dari hopper)
    pub fn set_expected_port(&mut self, port: u16) {
        self.expected_port = Some(port);
    }

    /// Increment epoch (dipanggil saat port hopping terjadi)
    pub fn advance_epoch(&mut self) {
        self.current_epoch += 1;
        // Bersihkan token lama untuk menghemat memori
        if self.seen_tokens.len() > MAX_SEEN_TOKENS {
            self.seen_tokens.clear();
        }
    }

    /// Evaluasi frame yang masuk dan tentukan tindakan
    pub fn evaluate(&mut self, frame: &FcrTpFrame, actual_port: u16) -> DefenseAction {
        let mut risk = self.calculate_risk(frame, actual_port);
        risk.calculate_total();
        
        // Catat token untuk deteksi reuse di masa depan
        self.seen_tokens.insert(frame.ephemeral_routing_token);
        self.risk_history.push(risk.clone());
        
        // Batasi riwayat untuk mencegah memory exhaustion
        if self.risk_history.len() > 1000 {
            self.risk_history.drain(0..500);
        }
        
        // Tentukan tindakan berdasarkan skor
        if risk.total >= DISCONNECT_THRESHOLD {
            DefenseAction::Disconnect { reason: "Risk score critical" }
        } else if risk.total >= CANARY_THRESHOLD {
            let trace_id = self.generate_trace_id();
            DefenseAction::DeployCanary { trace_id }
        } else {
            DefenseAction::Accept
        }
    }

    /// Hitung skor risiko berdasarkan berbagai anomali
    fn calculate_risk(&self, frame: &FcrTpFrame, actual_port: u16) -> TransportRiskScore {
        let mut risk = TransportRiskScore::default();

        // 1. Token Reuse Detection (70% bobot)
        if self.seen_tokens.contains(&frame.ephemeral_routing_token) {
            risk.token_reuse = 100; // Token sudah pernah dilihat = replay attack
        }

        // 2. Port Violation Detection (30% bobot)
        if let Some(expected) = self.expected_port {
            if actual_port != expected {
                risk.port_violation = 100; // Port tidak sesuai jadwal = MITM atau scanning
            }
        }

        // 3. Timing Anomaly Detection (0% bobot - akan diimplementasi nanti)
        risk.timing_anomaly = 0;

        // 4. Size Anomaly Detection (0% bobot - akan diimplementasi nanti)
        risk.size_anomaly = 0;

        risk
    }

    /// Generate trace_id unik untuk Canary Pill.
    /// 
    /// PENTING: trace_id digabungkan dengan session_id supaya:
    /// 1. Trace dari sesi A tidak tertukar dengan sesi B (multi-tenant safety)
    /// 2. Investigator bisa melacak trace_id kembali ke sesi spesifik
    /// 3. Attacker tidak bisa memprediksi trace_id berikutnya (karena ada komponen acak)
    fn generate_trace_id(&self) -> [u8; 16] {
        let mut trace_id = [0u8; 16];
        
        // 8 byte pertama = session_id (untuk forensic traceability)
        trace_id[0..8].copy_from_slice(&self.session_id);
        
        // 8 byte kedua = random (untuk unpredictability)
        rand::thread_rng().fill_bytes(&mut trace_id[8..16]);
        
        trace_id
    }

    /// Bangun Canary Frame yang terlihat valid tapi berisi forensic payload.
    /// 
    /// Forensic payload mengandung:
    /// - ChunkMetadata (agar frame terlihat valid saat diparse)
    /// - Trace ID (untuk identifikasi attacker)
    /// - Session ID (untuk forensic traceability ke sesi spesifik)
    /// - Epoch marker (untuk analisis timeline serangan)
    pub fn build_canary_frame(&self, trace_id: [u8; 16]) -> FcrTpFrame {
        // 1. Generate token acak
        let mut token = [0u8; 8];
        rand::thread_rng().fill_bytes(&mut token);

        // 2. Buat metadata yang terlihat valid
        let metadata = ChunkMetadata {
            message_id: u64::from_be_bytes(trace_id[0..8].try_into().unwrap()),
            version: crate::crypto::CURRENT_CRYPTO_VERSION,
            chunk_index: 0,
            total_chunks: 1,
            is_final: true,
        };

        // 3. Buat forensic payload
        // Struktur: [Metadata:CHUNK_METADATA_SIZE][TraceId:16][SessionId:8][Epoch:8][Padding:16]
        // Total: CHUNK_METADATA_SIZE + FORENSIC_OVERHEAD
        let total_forensic_size = CHUNK_METADATA_SIZE + FORENSIC_OVERHEAD;
        let mut forensic_payload = Vec::with_capacity(total_forensic_size);
        
        // Metadata (14 bytes)
        forensic_payload.extend_from_slice(&metadata.to_bytes());
        
        // Trace ID (16 bytes) — posisi: [CHUNK_METADATA_SIZE .. CHUNK_METADATA_SIZE+16]
        forensic_payload.extend_from_slice(&trace_id);
        
        // Session ID (8 bytes) — posisi: [CHUNK_METADATA_SIZE+16 .. CHUNK_METADATA_SIZE+24]
        forensic_payload.extend_from_slice(&self.session_id);
        
        // Epoch (8 bytes) — posisi: [CHUNK_METADATA_SIZE+24 .. CHUNK_METADATA_SIZE+32]
        forensic_payload.extend_from_slice(&self.current_epoch.to_be_bytes());
        
        // Padding (16 bytes)
        forensic_payload.extend_from_slice(&[0xBB; 16]);

        // 4. Generate AEAD tag (SIMULASI: nanti pakai ChaCha20-Poly1305 dari fcr-core-rust)
        let mut tag = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut tag);

        // 5. Rakit frame
        FcrTpFrame::new(
            token,
            HeaderMode::Short,
            Vec::new(), // encrypted_header_blob
            tag,
            forensic_payload,
        )
    }

    /// Dapatkan riwayat risiko untuk analisis forensik
    pub fn get_risk_history(&self) -> &[TransportRiskScore] {
        &self.risk_history
    }

    /// Dapatkan epoch saat ini (untuk logging dan debugging)
    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    /// Reset state (dipanggil saat sesi baru dimulai)
    pub fn reset(&mut self) {
        self.current_epoch = 0;
        self.seen_tokens.clear();
        self.risk_history.clear();
        self.expected_port = None;
    }
}

// === TEST SUITE ===
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_frame(token: [u8; 8]) -> FcrTpFrame {
        FcrTpFrame::new(
            token,
            HeaderMode::Short,
            Vec::new(),
            [0u8; 16],
            vec![0xAA; 32],
        )
    }

    #[test]
    fn test_accept_normal_frame() {
        let mut engine = DefenseEngine::new([1, 2, 3, 4, 5, 6, 7, 8]);
        engine.set_expected_port(50000);
        
        let frame = create_test_frame([0x11; 8]);
        let action = engine.evaluate(&frame, 50000);
        
        match action {
            DefenseAction::Accept => (), // Expected
            _ => panic!("Expected Accept, got {:?}", action),
        }
    }

    #[test]
    fn test_detect_token_reuse() {
        let mut engine = DefenseEngine::new([1, 2, 3, 4, 5, 6, 7, 8]);
        engine.set_expected_port(50000);
        
        let token = [0x11; 8];
        let frame1 = create_test_frame(token);
        let frame2 = create_test_frame(token); // Token sama!
        
        // Frame pertama harus diterima
        let action1 = engine.evaluate(&frame1, 50000);
        match action1 {
            DefenseAction::Accept => (),
            _ => panic!("First frame should be accepted"),
        }
        
        // Frame kedua dengan token sama harus trigger canary
        // Bobot baru: token_reuse (70%) → total = 70 >= CANARY_THRESHOLD (30)
        let action2 = engine.evaluate(&frame2, 50000);
        match action2 {
            DefenseAction::DeployCanary { trace_id } => {
                assert_ne!(trace_id, [0u8; 16]); // Trace ID harus unik
            }
            _ => panic!("Expected DeployCanary for token reuse, got {:?}", action2),
        }
    }

    #[test]
    fn test_detect_port_violation() {
        let mut engine = DefenseEngine::new([1, 2, 3, 4, 5, 6, 7, 8]);
        engine.set_expected_port(50000);
        
        let frame = create_test_frame([0x22; 8]);
        let action = engine.evaluate(&frame, 50001); // Port salah!
        
        // Bobot baru: port_violation (30%) → total = 30 >= CANARY_THRESHOLD (30)
        match action {
            DefenseAction::DeployCanary { .. } => (), // Expected
            _ => panic!("Expected DeployCanary for port violation, got {:?}", action),
        }
    }

    #[test]
    fn test_critical_risk_disconnect() {
        let mut engine = DefenseEngine::new([1, 2, 3, 4, 5, 6, 7, 8]);
        engine.set_expected_port(50000);
        
        // Frame pertama: token baru, port salah → port_violation (30) → canary
        let token = [0x33; 8];
        let frame1 = create_test_frame(token);
        let action1 = engine.evaluate(&frame1, 50001);
        match action1 {
            DefenseAction::DeployCanary { .. } => (),
            _ => panic!("First frame should trigger canary (port violation)"),
        }
        
        // Frame kedua: token reuse + port violation → total = 100 → disconnect
        let frame2 = create_test_frame(token);
        let action2 = engine.evaluate(&frame2, 50001);
        match action2 {
            DefenseAction::Disconnect { reason } => {
                assert_eq!(reason, "Risk score critical");
            }
            _ => panic!("Expected Disconnect for token reuse + port violation, got {:?}", action2),
        }
    }

    #[test]
    fn test_canary_frame_structure() {
        let engine = DefenseEngine::new([1, 2, 3, 4, 5, 6, 7, 8]);
        let trace_id = [0xAA; 16];
        
        let canary = engine.build_canary_frame(trace_id);
        
        // Verifikasi struktur frame
        assert_eq!(canary.ephemeral_routing_token.len(), 8);
        assert_eq!(canary.aead_auth_tag.len(), 16);
        assert_eq!(canary.dynamic_payload.len(), CHUNK_METADATA_SIZE + FORENSIC_OVERHEAD);
        
        // Verifikasi metadata di dalam payload
        let mut meta_buf = [0u8; CHUNK_METADATA_SIZE];
        meta_buf.copy_from_slice(&canary.dynamic_payload[0..CHUNK_METADATA_SIZE]);
        let metadata = ChunkMetadata::from_bytes(&meta_buf).expect("Metadata invalid");
        
        assert_eq!(metadata.version, crate::crypto::CURRENT_CRYPTO_VERSION);
        assert_eq!(metadata.is_final, true);
    }

    #[test]
    fn test_epoch_advancement_clears_tokens() {
        let mut engine = DefenseEngine::new([1, 2, 3, 4, 5, 6, 7, 8]);
        
        // Tambahkan banyak token
        for i in 0..100 {
            let mut token = [0u8; 8];
            token[0] = i as u8;
            engine.seen_tokens.insert(token);
        }
        
        assert_eq!(engine.seen_tokens.len(), 100);
        
        // Advance epoch (tidak trigger clear karena < MAX_SEEN_TOKENS)
        engine.advance_epoch();
        assert_eq!(engine.current_epoch(), 1);
        assert_eq!(engine.seen_tokens.len(), 100); // Masih ada
        
        // Paksa clear dengan menambahkan banyak token
        for i in 0..MAX_SEEN_TOKENS + 100 {
            let mut token = [0u8; 8];
            token[0..2].copy_from_slice(&(i as u16).to_be_bytes());
            engine.seen_tokens.insert(token);
        }
        
        engine.advance_epoch();
        assert!(engine.seen_tokens.is_empty()); // Harus dibersihkan
    }

    #[test]
    fn test_risk_score_calculation() {
        let mut risk = TransportRiskScore {
            timing_anomaly: 50,
            size_anomaly: 30,
            port_violation: 100,
            token_reuse: 100,
            total: 0,
        };
        
        risk.calculate_total();
        
        // Bobot baru: token_reuse (70%), port_violation (30%)
        // = (100*70 + 100*30) / 100 = 100
        assert_eq!(risk.total, 100);
    }

    #[test]
    fn test_session_id_accessible() {
        let session_id = [0xAB; 8];
        let engine = DefenseEngine::new(session_id);
        
        assert_eq!(engine.session_id(), session_id);
    }

    #[test]
    fn test_trace_id_contains_session_id() {
        let session_id = [0xAB; 8];
        let engine = DefenseEngine::new(session_id);
        
        let trace_id = engine.generate_trace_id();
        
        assert_eq!(&trace_id[0..8], &session_id);
        assert_ne!(&trace_id[8..16], &[0u8; 8]);
    }

    #[test]
    fn test_trace_id_uniqueness_per_call() {
        let engine = DefenseEngine::new([0xCD; 8]);
        
        let trace1 = engine.generate_trace_id();
        let trace2 = engine.generate_trace_id();
        
        assert_ne!(trace1, trace2);
        assert_eq!(&trace1[0..8], &trace2[0..8]);
    }

    #[test]
    fn test_canary_frame_contains_session_id_in_forensic_payload() {
        let session_id = [0xEF; 8];
        let engine = DefenseEngine::new(session_id);
        
        let trace_id = engine.generate_trace_id();
        let canary = engine.build_canary_frame(trace_id);
        
        let session_id_start = CHUNK_METADATA_SIZE + 16;
        let session_id_end = session_id_start + 8;
        let forensic_session_id = &canary.dynamic_payload[session_id_start..session_id_end];
        assert_eq!(forensic_session_id, &session_id);
        
        let trace_id_start = CHUNK_METADATA_SIZE;
        let trace_id_end = trace_id_start + 16;
        let forensic_trace_id = &canary.dynamic_payload[trace_id_start..trace_id_end];
        assert_eq!(forensic_trace_id, &trace_id);
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
}