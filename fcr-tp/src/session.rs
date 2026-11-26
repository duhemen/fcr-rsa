// src/session.rs - VERSI FINAL
//
// Perubahan:
// 1. `expected_port(&self)` pakai `peek_next_port()` (tidak konsumsi)
// 2. `process_chunk_at` pakai `peek_next_port()` untuk validasi,
//    lalu `next_port()` untuk konsumsi setelah validasi sukses
// 3. `make_forensic_with_valid_port` pakai `&TransportOrchestrator`
// 4. Semua method dalam SATU blok `impl`

use crate::crypto::{
    ChunkMetadata, FcrTpSession as CryptoSession, CURRENT_CRYPTO_VERSION,
};
use crate::hopper::FibonacciHopper;
use crate::reassembler::{
    ForensicData, Reassembler, ReassemblyResult, ReassemblyError,
};
use crate::survival::{
    SurvivalModeController, NetworkMetrics, SurvivalPolicy, LinkProfile,
};
use crate::anomaly::{
    AnomalyDetector, AnomalyReport, SessionContext,
};
use crate::spacetime::{
    SpacetimeValidator, SpacetimeSample,
};
use bytes::Bytes;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================
// TRANSPORT ORCHESTRATOR
// ============================================================

pub struct TransportOrchestrator {
    crypto: CryptoSession,
    pub survival: SurvivalModeController,
    pub reassembler: Reassembler,
    pub anomaly: AnomalyDetector,
    pub spacetime: SpacetimeValidator,
    hopper: FibonacciHopper,
    session_id: [u8; 8],
    seed: u64,
}

impl TransportOrchestrator {
    // ============================================================
    // CONSTRUCTORS
    // ============================================================

    pub fn new(seed: u64, master_key: &[u8; 32]) -> Self {
        let mut session_id = [0u8; 8];
        session_id.copy_from_slice(&seed.to_be_bytes());

        let crypto = CryptoSession::new(session_id, master_key, CURRENT_CRYPTO_VERSION);

        Self {
            crypto,
            survival: SurvivalModeController::new(),
            reassembler: Reassembler::new(),
            anomaly: AnomalyDetector::new(),
            spacetime: SpacetimeValidator::new(session_id),
            hopper: FibonacciHopper::new(seed),
            session_id,
            seed,
        }
    }

    pub fn with_profile(seed: u64, master_key: &[u8; 32], profile: LinkProfile) -> Self {
        let mut orch = Self::new(seed, master_key);
        orch.anomaly = AnomalyDetector::with_profile(profile);
        orch
    }

    // ============================================================
    // GETTERS
    // ============================================================

    pub fn session_id(&self) -> [u8; 8] { self.session_id }
    pub fn seed(&self) -> u64 { self.seed }
    pub fn crypto(&self) -> &CryptoSession { &self.crypto }
    pub fn crypto_mut(&mut self) -> &mut CryptoSession { &mut self.crypto }

    // ============================================================
    // KONFIGURASI
    // ============================================================

    pub fn set_context(&mut self, context: SessionContext) {
        self.anomaly.set_context(context);
    }

    pub fn set_profile(&mut self, profile: LinkProfile) {
        self.anomaly.set_profile(profile);
    }

    /// ✅ Dapatkan port yang diharapkan untuk chunk berikutnya.
    /// 
    /// Ini menggunakan `peek_next_port()` yang **tidak** mengonsumsi
    /// state hopper. Jadi panggilan berulang akan mengembalikan port
    /// yang sama sampai `process_chunk_at` benar-benar mengonsumsi
    /// port tersebut.
    pub fn expected_port(&self) -> u16 {
        self.hopper.peek_next_port()
    }

    // ============================================================
    // NETWORK METRICS
    // ============================================================

    pub fn process_network_metrics(&mut self, metrics: NetworkMetrics) -> SurvivalPolicy {
        let policy = self.survival.update_metrics(metrics);
        self.reassembler.apply_policy(&policy);
        policy
    }

    // ============================================================
    // MAIN PIPELINE
    // ============================================================

    /// Proses chunk dengan timestamp injection (deterministik untuk test).
    /// 
    /// Alur:
    /// 1. Peek port yang diharapkan (tidak konsumsi)
    /// 2. Validasi spatiotemporal
    /// 3. Konsumsi port (setelah validasi sukses)
    /// 4. Deteksi anomali
    /// 5. Push chunk ke reassembler
    pub fn process_chunk_at(
        &mut self,
        metadata: &ChunkMetadata,
        data: Bytes,
        forensic: ForensicData,
        hop_distance_km: f64,
        timestamp_ms: u64,
    ) -> Result<(Option<ReassemblyResult>, Option<AnomalyReport>), ReassemblyError> {
        // 1. Peek port (tidak konsumsi)
        let expected_port = self.hopper.peek_next_port();

        // 2. Bangun sample spatiotemporal
        let sample = SpacetimeSample {
            session_id: self.session_id,
            epoch: self.crypto.current_epoch(),
            port: forensic.source_port,
            timestamp_ms,
            hop_distance_km,
        };

        // 3. Validasi spatiotemporal
        if let Err(_st_err) = self.spacetime.validate(&sample, expected_port) {
            return Err(ReassemblyError::InvalidChunk);
        }

        // 4. Konsumsi port (setelah validasi sukses)
        self.hopper.next_port();

        // 5. Deteksi anomali
        let anomaly_report = self.anomaly.record_chunk_at(
            data.len(),
            forensic.timestamp,
        );

        // 6. Push chunk
        let result = self.reassembler.push_chunk(metadata, data, forensic)?;

        // 7. Feedback loop
        if result.is_none() {
            let message_id = crate::reassembler::MessageId::from(metadata.message_id);
            if let Some(feedback_metrics) = self.reassembler.extract_assembly_metrics(&message_id) {
                self.process_network_metrics(feedback_metrics);
            }
        }

        Ok((result, anomaly_report))
    }

    /// Proses chunk (default: pakai `SystemTime::now()`).
    pub fn process_chunk(
        &mut self,
        metadata: &ChunkMetadata,
        data: Bytes,
        forensic: ForensicData,
        hop_distance_km: f64,
    ) -> Result<(Option<ReassemblyResult>, Option<AnomalyReport>), ReassemblyError> {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.process_chunk_at(metadata, data, forensic, hop_distance_km, timestamp_ms)
    }

    // ============================================================
    // ENKRIPSI / DEKRIPSI
    // ============================================================

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<crate::packet::FcrTpFrame>, &'static str> {
        self.crypto.encrypt_to_frames(plaintext)
    }

    pub fn encrypt_with_bucket(
        &self,
        plaintext: &[u8],
        bucket_size: usize,
    ) -> Result<Vec<crate::packet::FcrTpFrame>, &'static str> {
        self.crypto.encrypt_to_frames_with_bucket(plaintext, bucket_size)
    }

    pub fn decrypt(
        &self,
        frame: &crate::packet::FcrTpFrame,
    ) -> Result<(ChunkMetadata, Vec<u8>), &'static str> {
        self.crypto.decrypt_from_frame(frame)
    }

    // ============================================================
    // EPOCH MANAGEMENT
    // ============================================================

    pub fn advance_epoch(&mut self) {
        self.crypto.advance_epoch();
        self.hopper = FibonacciHopper::new(self.seed.wrapping_add(self.crypto.current_epoch()));
        self.spacetime.reset();
    }

    pub fn current_epoch(&self) -> u64 {
        self.crypto.current_epoch()
    }

    pub fn current_policy(&self) -> SurvivalPolicy {
        self.survival.current_policy()
    }
}

impl Default for TransportOrchestrator {
    fn default() -> Self {
        Self::new(0xCAFEBABE, &[0xAA; 32])
    }
}

// ============================================================
// TEST SUITE
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::ChunkMetadata;
    use crate::survival::{NetworkMetrics, ControlTier, LinkProfile};
    use crate::anomaly::AnomalyType;
    use std::time::{Duration, Instant};

    const TEST_SEED: u64 = 0xDEADBEEF;
    const TEST_MASTER_KEY: [u8; 32] = [0xAA; 32];

    // ============================================================
    // Helper
    // ============================================================

    fn make_metadata(message_id: u64, chunk_index: u16, total_chunks: u16) -> ChunkMetadata {
        ChunkMetadata {
            message_id,
            version: CURRENT_CRYPTO_VERSION,
            chunk_index,
            total_chunks,
            is_final: chunk_index == total_chunks - 1,
        }
    }

    /// Buat forensic dengan port dari orchestrator (peek, tidak konsumsi).
    fn make_forensic_with_valid_port(orch: &TransportOrchestrator) -> ForensicData {
        let valid_port = orch.expected_port();
        ForensicData {
            trace_id: "session-test".to_string(),
            session_id: orch.session_id(),
            source_port: valid_port,
            source_ip: Some([127, 0, 0, 1]),
            timestamp: Instant::now(),
            rtt_ms: Some(50),
            packet_loss: None,
        }
    }

    fn make_orchestrator() -> TransportOrchestrator {
        TransportOrchestrator::new(TEST_SEED, &TEST_MASTER_KEY)
    }

    // ============================================================
    // Test: Integrasi survival mode
    // ============================================================

    #[test]
    fn test_session_integration_adaptive_timeout() {
        let mut orch = make_orchestrator();
        let critical_metrics = NetworkMetrics {
            rtt_ms: 1000.0,
            jitter_ms: 200.0,
            loss_rate: 0.8,
            throughput_bps: 1000.0,
        };
        let policy = orch.process_network_metrics(critical_metrics);

        assert_eq!(policy.tier, ControlTier::Enforce);
        assert_eq!(
            orch.reassembler.current_timeout(),
            Duration::from_millis(25000)
        );
    }

    // ============================================================
    // Test: Anomaly detection integration
    // ============================================================

    #[test]
    fn test_session_anomaly_detection_integration() {
        let mut orch = make_orchestrator();
        let base_ts: u64 = 1_000_000;

        // Frame 1 (baseline)
        let forensic1 = make_forensic_with_valid_port(&orch);
        orch.process_chunk_at(
            &make_metadata(10, 0, 10),
            Bytes::from(vec![0xAA; 100]),
            forensic1,
            0.0,
            base_ts,
        ).unwrap();

        // Frame 2 (IAT 50ms, normal)
        let forensic2 = make_forensic_with_valid_port(&orch);
        orch.process_chunk_at(
            &make_metadata(10, 1, 10),
            Bytes::from(vec![0xBB; 100]),
            forensic2,
            0.0,
            base_ts + 50,
        ).unwrap();

        // Burst: IAT 1ms
        let mut report = None;
        for i in 2..10u64 {
            let forensic = make_forensic_with_valid_port(&orch);
            let (_, r) = orch.process_chunk_at(
                &make_metadata(10, i as u16, 10),
                Bytes::from(vec![0xCC; 100]),
                forensic,
                0.0,
                base_ts + 50 + (i - 1),
            ).unwrap();
            if r.is_some() {
                report = r;
                break;
            }
        }

        assert!(report.is_some(), "Burst harus terdeteksi setelah grace period");
        assert_eq!(report.unwrap().anomaly_type, AnomalyType::BurstTraffic);
    }

    // ============================================================
    // Test: Spatiotemporal rejection
    // ============================================================

    #[test]
    fn test_session_rejects_spatiotemporal_anomaly() {
        let mut orch = make_orchestrator();
        let session_id = orch.session_id();

        let bad_forensic = ForensicData {
            trace_id: "attacker".to_string(),
            session_id,
            source_port: 8080,
            source_ip: Some([192, 168, 1, 1]),
            timestamp: Instant::now(),
            rtt_ms: Some(100),
            packet_loss: None,
        };

        let result = orch.process_chunk_at(
            &make_metadata(10, 0, 3),
            Bytes::from("evil"),
            bad_forensic,
            100.0,
            1_000_000,
        );

        assert!(result.is_err(), "Harus reject port mismatch");
        assert_eq!(result.unwrap_err(), ReassemblyError::InvalidChunk);
    }

    // ============================================================
    // Test: Feedback loop
    // ============================================================

    #[test]
    fn test_session_feedback_loop() {
        let mut orch = make_orchestrator();
        let base_ts: u64 = 1_000_000;

        let forensic1 = make_forensic_with_valid_port(&orch);
        orch.process_chunk_at(
            &make_metadata(10, 0, 3),
            Bytes::from("A"),
            forensic1,
            0.0,
            base_ts,
        ).unwrap();

        let forensic2 = make_forensic_with_valid_port(&orch);
        orch.process_chunk_at(
            &make_metadata(10, 1, 3),
            Bytes::from("B"),
            forensic2,
            0.0,
            base_ts + 50,
        ).unwrap();

        assert!(orch.survival.current_policy().suggested_bucket.as_bytes() > 0);
    }

    // ============================================================
    // Test: Encrypt/decrypt
    // ============================================================

    #[test]
    fn test_orchestrator_encrypt_decrypt() {
        let orch = make_orchestrator();
        let plaintext = b"Halo dari orchestrator";

        let frames = orch.encrypt(plaintext).expect("Encrypt gagal");
        assert_eq!(frames.len(), 1);

        let (metadata, decrypted) = orch.decrypt(&frames[0]).expect("Decrypt gagal");
        assert_eq!(decrypted, plaintext.to_vec());
        assert!(metadata.is_final);
    }

    #[test]
    fn test_orchestrator_set_context() {
        let mut orch = make_orchestrator();
        orch.set_context(SessionContext::FileUpload);
    }

    #[test]
    fn test_orchestrator_advance_epoch() {
        let mut orch = make_orchestrator();
        assert_eq!(orch.current_epoch(), 0);
        orch.advance_epoch();
        assert_eq!(orch.current_epoch(), 1);
    }

    // ============================================================
    // Test: Session ID propagation
    // ============================================================

    #[test]
    fn test_session_id_propagated_to_spacetime() {
        let orch = make_orchestrator();
        assert_eq!(orch.spacetime.session_id(), orch.session_id());
    }

    #[test]
    fn test_session_id_propagated_to_forensic() {
        let orch = make_orchestrator();
        let session_id = orch.session_id();
        let forensic = make_forensic_with_valid_port(&orch);
        assert_eq!(forensic.session_id, session_id);
    }

    // ============================================================
    // Test: Port validation
    // ============================================================

    #[test]
    fn test_forensic_port_used_not_rtt() {
        // ✅ FIX: gunakan `make_forensic_with_valid_port(&orch)` yang
        // pakai `peek_next_port()` (tidak konsumsi).
        let mut orch = make_orchestrator();

        let forensic = make_forensic_with_valid_port(&orch);
        // Override rtt_ms untuk memastikan tidak dipakai sebagai port
        let forensic = ForensicData {
            rtt_ms: Some(9999),
            ..forensic
        };

        let result = orch.process_chunk_at(
            &make_metadata(10, 0, 1),
            Bytes::from("test"),
            forensic,
            0.0,
            1_000_000,
        );

        assert!(result.is_ok(), "Harus OK karena source_port benar");
    }

    #[test]
    fn test_wrong_port_rejected() {
        let mut orch = make_orchestrator();
        let session_id = orch.session_id();

        let forensic = ForensicData {
            trace_id: "test".to_string(),
            session_id,
            source_port: 12345,
            source_ip: None,
            timestamp: Instant::now(),
            rtt_ms: Some(50),
            packet_loss: None,
        };

        let result = orch.process_chunk_at(
            &make_metadata(10, 0, 1),
            Bytes::from("test"),
            forensic,
            0.0,
            1_000_000,
        );

        assert!(result.is_err(), "Harus reject port salah");
    }

    // ============================================================
    // Test: Session isolation
    // ============================================================

    #[test]
    fn test_different_session_rejected() {
        let mut orch = make_orchestrator();

        let bad_forensic = ForensicData {
            trace_id: "test".to_string(),
            session_id: [0xBB; 8],
            source_port: 50000,
            source_ip: None,
            timestamp: Instant::now(),
            rtt_ms: Some(50),
            packet_loss: None,
        };

        let result = orch.process_chunk_at(
            &make_metadata(10, 0, 1),
            Bytes::from("test"),
            bad_forensic,
            0.0,
            1_000_000,
        );

        assert!(result.is_err(), "Harus reject sesi berbeda");
    }

    // ============================================================
    // Test: Crypto integration
    // ============================================================

    #[test]
    fn test_crypto_epoch_used() {
        let mut orch = make_orchestrator();

        assert_eq!(orch.current_epoch(), 0);

        let plaintext = b"Data epoch 0";
        let frames = orch.encrypt(plaintext).expect("Encrypt gagal");

        orch.advance_epoch();
        assert_eq!(orch.current_epoch(), 1);

        let result = orch.decrypt(&frames[0]);
        assert!(result.is_err(), "Frame dari epoch lama harus ditolak");

        let frames2 = orch.encrypt(plaintext).expect("Encrypt gagal");
        let (_, decrypted) = orch.decrypt(&frames2[0]).expect("Decrypt gagal");
        assert_eq!(decrypted, plaintext.to_vec());
    }

    #[test]
    fn test_with_profile_constructor() {
        let orch = TransportOrchestrator::with_profile(
            TEST_SEED,
            &TEST_MASTER_KEY,
            LinkProfile::Satellite,
        );

        assert_eq!(orch.anomaly.profile(), LinkProfile::Satellite);
        assert_eq!(orch.anomaly.active_threshold().burst_threshold_ms, 50.0);
    }
}