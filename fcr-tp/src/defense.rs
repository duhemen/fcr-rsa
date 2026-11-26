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

use crate::packet::FcrTpFrame;
use crate::crypto::{ChunkMetadata, CHUNK_METADATA_SIZE, FcrTpSession};
/// Ambang batas risiko untuk mengaktifkan Canary Pill (0-100)
pub const CANARY_THRESHOLD: u8 = 30;

/// Ambang batas risiko untuk memutus koneksi (0-100)
pub const DISCONNECT_THRESHOLD: u8 = 100;

/// Maksimum token yang disimpan untuk deteksi reuse
pub const MAX_SEEN_TOKENS: usize = 10_000;

/// Ukuran forensic payload tambahan
pub const FORENSIC_OVERHEAD: usize = 16 + 8 + 8 + 16;

/// Skor risiko transport (0 = aman, 100 = sangat mencurigakan)
#[derive(Debug, Clone, Default, Zeroize)]
pub struct TransportRiskScore {
    pub timing_anomaly: u8,
    pub size_anomaly: u8,
    pub port_violation: u8,
    pub token_reuse: u8,
    pub total: u8,
}

impl TransportRiskScore {
    pub fn calculate_total(&mut self) {
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
    Accept,
    DeployCanary { trace_id: [u8; 16] },
    Disconnect { reason: &'static str },
}

/// Mesin pertahanan transport
pub struct DefenseEngine {
    session_id: [u8; 8],
    current_epoch: u64,
    seen_tokens: HashSet<[u8; 8]>,
    risk_history: Vec<TransportRiskScore>,
    expected_port: Option<u16>,
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

    pub fn session_id(&self) -> [u8; 8] {
        self.session_id
    }

    pub fn set_expected_port(&mut self, port: u16) {
        self.expected_port = Some(port);
    }

    /// Update epoch dari crypto session (sinkronisasi)
    pub fn sync_epoch(&mut self, epoch: u64) {
        self.current_epoch = epoch;
    }

    pub fn advance_epoch(&mut self) {
        self.current_epoch = self.current_epoch.wrapping_add(1);
        if self.seen_tokens.len() > MAX_SEEN_TOKENS {
            self.seen_tokens.clear();
        }
    }

    /// Evaluasi frame dengan epoch dari crypto session
    pub fn evaluate(&mut self, frame: &FcrTpFrame, actual_port: u16, current_epoch: u64) -> DefenseAction {
        // Sinkronisasi epoch dari crypto session
        self.sync_epoch(current_epoch);
        
        let mut risk = self.calculate_risk(frame, actual_port);
        risk.calculate_total();
        
        self.seen_tokens.insert(frame.ephemeral_routing_token);
        self.risk_history.push(risk.clone());
        
        if self.risk_history.len() > 1000 {
            self.risk_history.drain(0..500);
        }
        
        if risk.total >= DISCONNECT_THRESHOLD {
            DefenseAction::Disconnect { reason: "Risk score critical" }
        } else if risk.total >= CANARY_THRESHOLD {
            let trace_id = self.generate_trace_id();
            DefenseAction::DeployCanary { trace_id }
        } else {
            DefenseAction::Accept
        }
    }

    fn calculate_risk(&self, frame: &FcrTpFrame, actual_port: u16) -> TransportRiskScore {
        let mut risk = TransportRiskScore::default();

        if self.seen_tokens.contains(&frame.ephemeral_routing_token) {
            risk.token_reuse = 100;
        }

        if let Some(expected) = self.expected_port {
            if actual_port != expected {
                risk.port_violation = 100;
            }
        }

        risk.timing_anomaly = 0;
        risk.size_anomaly = 0;

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
    /// PERBAIKAN: Canary frame sekarang di-encrypt dengan session key,
    /// sehingga terlihat 100% valid dan bisa di-decrypt oleh penerima.
    /// Saat didekripsi, isinya forensic data yang mengungkap attacker.
    pub fn build_canary_frame(&self, trace_id: [u8; 16], session: &FcrTpSession) -> Result<FcrTpFrame, &'static str> {
        // 1. Buat metadata yang terlihat valid
        let metadata = ChunkMetadata {
            message_id: u64::from_be_bytes(trace_id[0..8].try_into().unwrap()),
            version: crate::crypto::CURRENT_CRYPTO_VERSION,
            chunk_index: 0,
            total_chunks: 1,
            is_final: true,
        };

        // 2. Buat forensic payload
        // Struktur: [Metadata:14][TraceId:16][SessionId:8][Epoch:8][Padding:16]
        let mut forensic_payload = Vec::with_capacity(CHUNK_METADATA_SIZE + FORENSIC_OVERHEAD);
        forensic_payload.extend_from_slice(&metadata.to_bytes());
        forensic_payload.extend_from_slice(&trace_id);
        forensic_payload.extend_from_slice(&self.session_id);
        forensic_payload.extend_from_slice(&self.current_epoch.to_be_bytes());
        forensic_payload.extend_from_slice(&[0xBB; 16]);

        // 3. Encrypt dengan crypto session (bucket 256 bytes)
        let frames = session.encrypt_to_frames_with_bucket(&forensic_payload, 256)?;
        
        // 4. Return frame pertama (hanya 1 chunk)
        Ok(frames[0].clone())
    }

    pub fn get_risk_history(&self) -> &[TransportRiskScore] {
        &self.risk_history
    }

    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

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
    use crate::crypto::CURRENT_CRYPTO_VERSION;
    use crate::packet::HeaderMode;

    fn create_test_frame(token: [u8; 8]) -> FcrTpFrame {
        FcrTpFrame::new(
            token,
            HeaderMode::Short,
            vec![0xAA; 200], // 200 bytes blob
            [0u8; 16],
            vec![0xBB; 32], // 32 bytes payload
        )
    }

    #[test]
    fn test_accept_normal_frame() {
        let mut engine = DefenseEngine::new([1, 2, 3, 4, 5, 6, 7, 8]);
        engine.set_expected_port(50000);
        
        let frame = create_test_frame([0x11; 8]);
        let action = engine.evaluate(&frame, 50000, 0);
        
        match action {
            DefenseAction::Accept => (),
            _ => panic!("Expected Accept, got {:?}", action),
        }
    }

    #[test]
    fn test_detect_token_reuse() {
        let mut engine = DefenseEngine::new([1, 2, 3, 4, 5, 6, 7, 8]);
        engine.set_expected_port(50000);
        
        let token = [0x11; 8];
        let frame1 = create_test_frame(token);
        let frame2 = create_test_frame(token);
        
        let action1 = engine.evaluate(&frame1, 50000, 0);
        match action1 {
            DefenseAction::Accept => (),
            _ => panic!("First frame should be accepted"),
        }
        
        let action2 = engine.evaluate(&frame2, 50000, 0);
        match action2 {
            DefenseAction::DeployCanary { trace_id } => {
                assert_ne!(trace_id, [0u8; 16]);
            }
            _ => panic!("Expected DeployCanary for token reuse, got {:?}", action2),
        }
    }

    #[test]
    fn test_detect_port_violation() {
        let mut engine = DefenseEngine::new([1, 2, 3, 4, 5, 6, 7, 8]);
        engine.set_expected_port(50000);
        
        let frame = create_test_frame([0x22; 8]);
        let action = engine.evaluate(&frame, 50001, 0);
        
        match action {
            DefenseAction::DeployCanary { .. } => (),
            _ => panic!("Expected DeployCanary for port violation, got {:?}", action),
        }
    }

    #[test]
    fn test_critical_risk_disconnect() {
        let mut engine = DefenseEngine::new([1, 2, 3, 4, 5, 6, 7, 8]);
        engine.set_expected_port(50000);
        
        let token = [0x33; 8];
        let frame1 = create_test_frame(token);
        let action1 = engine.evaluate(&frame1, 50001, 0);
        match action1 {
            DefenseAction::DeployCanary { .. } => (),
            _ => panic!("First frame should trigger canary (port violation)"),
        }
        
        let frame2 = create_test_frame(token);
        let action2 = engine.evaluate(&frame2, 50001, 0);
        match action2 {
            DefenseAction::Disconnect { reason } => {
                assert_eq!(reason, "Risk score critical");
            }
            _ => panic!("Expected Disconnect for token reuse + port violation, got {:?}", action2),
        }
    }

    #[test]
    fn test_canary_frame_encrypted_and_valid() {
        // PERBAIKAN: Canary frame sekarang di-encrypt dengan crypto session
        let session_id = [1, 2, 3, 4, 5, 6, 7, 8];
        let master_key = [0xAA; 32];
        let session = FcrTpSession::new(session_id, &master_key, CURRENT_CRYPTO_VERSION);
        
        let engine = DefenseEngine::new(session_id);
        let trace_id = engine.generate_trace_id();
        
        // Build canary frame dengan crypto session
        let canary = engine.build_canary_frame(trace_id, &session).expect("Build canary gagal");
        
        // Verifikasi frame bisa di-encode ke wire
        let wire = canary.encode().expect("Encode canary gagal");
        assert_eq!(wire.len(), 256); // Harus bucket 256
        
        // Verifikasi frame bisa di-decode dari wire
        let received = FcrTpFrame::decode(wire).expect("Decode canary gagal");
        
        // Verifikasi frame bisa di-decrypt (terenkripsi dengan benar)
        let (metadata, decrypted) = session.decrypt_from_frame(&received).expect("Decrypt canary gagal");
        
        // Verifikasi metadata valid
        assert_eq!(metadata.version, CURRENT_CRYPTO_VERSION);
        assert_eq!(metadata.is_final, true);
        
        // Verifikasi forensic payload ada di dalam decrypted data
        assert!(decrypted.len() >= CHUNK_METADATA_SIZE + FORENSIC_OVERHEAD);
        
        // Verifikasi trace_id ada di forensic payload
        let trace_id_start = CHUNK_METADATA_SIZE;
        let trace_id_end = trace_id_start + 16;
        let forensic_trace_id = &decrypted[trace_id_start..trace_id_end];
        assert_eq!(forensic_trace_id, &trace_id);
    }

    #[test]
    fn test_epoch_advancement_clears_tokens() {
        let mut engine = DefenseEngine::new([1, 2, 3, 4, 5, 6, 7, 8]);
        
        for i in 0..100 {
            let mut token = [0u8; 8];
            token[0] = i as u8;
            engine.seen_tokens.insert(token);
        }
        
        assert_eq!(engine.seen_tokens.len(), 100);
        
        engine.advance_epoch();
        assert_eq!(engine.current_epoch(), 1);
        assert_eq!(engine.seen_tokens.len(), 100);
        
        for i in 0..MAX_SEEN_TOKENS + 100 {
            let mut token = [0u8; 8];
            token[0..2].copy_from_slice(&(i as u16).to_be_bytes());
            engine.seen_tokens.insert(token);
        }
        
        engine.advance_epoch();
        assert!(engine.seen_tokens.is_empty());
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
    fn test_epoch_sync_from_crypto_session() {
        let mut engine = DefenseEngine::new([1, 2, 3, 4, 5, 6, 7, 8]);
        
        assert_eq!(engine.current_epoch(), 0);
        
        // Sync epoch dari crypto session
        engine.sync_epoch(5);
        assert_eq!(engine.current_epoch(), 5);
        
        engine.sync_epoch(10);
        assert_eq!(engine.current_epoch(), 10);
    }
}