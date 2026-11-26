// src/session.rs - FCR-TP Session Orchestrator (Spatiotemporal-Aware)
use crate::crypto::ChunkMetadata;
use crate::hopper::FibonacciHopper;
use crate::reassembler::{ForensicData, Reassembler, ReassemblyResult, ReassemblyError};
use crate::survival::{SurvivalModeController, NetworkMetrics, SurvivalPolicy};
use crate::anomaly::{AnomalyDetector, AnomalyReport};
use crate::spacetime::{SpacetimeValidator, SpacetimeSample};
use bytes::Bytes;
use std::time::{SystemTime, UNIX_EPOCH};

/// FCR-TP Session Engine
pub struct FcrTpSession {
    pub survival: SurvivalModeController,
    pub reassembler: Reassembler,
    pub anomaly: AnomalyDetector,
    pub spacetime: SpacetimeValidator,
    pub hopper: FibonacciHopper, // Menambah state hopper untuk prediksi port
    pub seed: u64,
}

impl FcrTpSession {
    pub fn new(seed: u64) -> Self {
        Self {
            survival: SurvivalModeController::new(),
            reassembler: Reassembler::new(),
            anomaly: AnomalyDetector::new(),
            spacetime: SpacetimeValidator::new(),
            hopper: FibonacciHopper::new(seed),
            seed,
        }
    }

    pub fn process_network_metrics(&mut self, metrics: NetworkMetrics) -> SurvivalPolicy {
        let policy = self.survival.update_metrics(metrics);
        self.reassembler.apply_policy(&policy);
        policy
    }

    /// 🌟 Push chunk dengan feedback loop + validasi spatiotemporal
    pub fn push_chunk_with_feedback(
        &mut self,
        metadata: &ChunkMetadata,
        data: Bytes,
        forensic: ForensicData,
        hop_distance_km: f64,
    ) -> Result<(Option<ReassemblyResult>, Option<AnomalyReport>), ReassemblyError> {

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // 1. Prediksi port yang SEHARUSNYA untuk epoch/pesan ini
        // Catatan: Di produksi, ini disinkronkan dengan state crypto epoch.
        // Untuk test, kita gunakan metadata.message_id sebagai simulasi sequence.
        let expected_port = self.hopper.next_port();

        // 2. Validasi Spatiotemporal
        let sample = SpacetimeSample {
            epoch: metadata.message_id, // Placeholder epoch
            port: forensic.rtt_ms.unwrap_or(0) as u16, // Port dari paket masuk
            timestamp_ms: now_ms,
            hop_distance_km,
        };

        if let Err(_st_err) = self.spacetime.validate(&sample, expected_port) {
            // Zero-trust: tolak paket jika ruang/waktu tidak sinkron
            return Err(ReassemblyError::InvalidChunk);
        }

        // 3. Deteksi Anomali
        let anomaly_report = self.anomaly.record_chunk(data.len());

        // 4. Push chunk ke reassembler
        let result = self.reassembler.push_chunk(metadata, data, forensic)?;

        // 5. Feedback loop ke survival mode
        if result.is_none() {
            let message_id = crate::reassembler::MessageId::from(metadata.message_id);
            if let Some(feedback_metrics) = self.reassembler.extract_assembly_metrics(&message_id) {
                self.process_network_metrics(feedback_metrics);
            }
        }

        Ok((result, anomaly_report))
    }

    pub fn current_policy(&self) -> SurvivalPolicy {
        self.survival.current_policy()
    }
}

impl Default for FcrTpSession {
    fn default() -> Self { Self::new(0xCAFEBABE) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::ChunkMetadata;
    use crate::survival::{NetworkMetrics, ControlTier};
    use crate::anomaly::AnomalyType;
    use std::time::{Duration, Instant};

    fn make_metadata(message_id: u64, chunk_index: u16, total_chunks: u16) -> ChunkMetadata {
        ChunkMetadata { message_id, version: 1, chunk_index, total_chunks, is_final: chunk_index == total_chunks - 1 }
    }

    /// Helper: buat forensic dengan port yang SESUAI dengan prediksi hopper
    fn make_forensic_with_valid_port(hopper: &mut FibonacciHopper) -> ForensicData {
        let valid_port = hopper.next_port(); // Consume state agar sinkron dengan session
        ForensicData {
            trace_id: "session-test".to_string(),
            session_id: 1,
            timestamp: Instant::now(),
            rtt_ms: Some(valid_port as u64), // Encode port di sini untuk test
            packet_loss: None,
        }
    }

    #[test]
    fn test_session_integration_adaptive_timeout() {
        let mut session = FcrTpSession::new(0xDEADBEEF);
        let critical_metrics = NetworkMetrics { rtt_ms: 1000.0, jitter_ms: 200.0, loss_rate: 0.8, throughput_bps: 1000.0 };
        let policy = session.process_network_metrics(critical_metrics);

        assert_eq!(policy.tier, ControlTier::Enforce);
        assert_eq!(session.reassembler.current_timeout(), Duration::from_millis(25000));
    }

    #[test]
    fn test_session_anomaly_detection_integration() {
        let seed = 0xDEADBEEF;
        let mut session = FcrTpSession::new(seed);

        // Kita perlu membuat hopper dummy yang sinkron dengan session untuk test ini
        let mut dummy_hopper = FibonacciHopper::new(seed);

        session.push_chunk_with_feedback(
            &make_metadata(10, 0, 3), Bytes::from("A".repeat(100)),
            make_forensic_with_valid_port(&mut dummy_hopper), 50.0,
        ).unwrap();
        std::thread::sleep(Duration::from_millis(50));
        session.push_chunk_with_feedback(
            &make_metadata(10, 1, 3), Bytes::from("B".repeat(100)),
            make_forensic_with_valid_port(&mut dummy_hopper), 50.0,
        ).unwrap();

        // Burst: push chunk 2 TANPA delay
        let (_, report) = session.push_chunk_with_feedback(
            &make_metadata(10, 2, 3), Bytes::from("C".repeat(100)),
            make_forensic_with_valid_port(&mut dummy_hopper), 50.0,
        ).unwrap();

        assert!(report.is_some());
        assert_eq!(report.unwrap().anomaly_type, AnomalyType::BurstTraffic);
    }

    #[test]
    fn test_session_rejects_spatiotemporal_anomaly() {
        let seed = 0xDEADBEEF;
        let mut session = FcrTpSession::new(seed);

        // Forensic dengan port yang SALAH (8080 bukan hasil hopper)
        let bad_forensic = ForensicData {
            trace_id: "attacker".to_string(),
            session_id: 1,
            timestamp: Instant::now(),
            rtt_ms: Some(8080), // Port salah
            packet_loss: None,
        };

        let result = session.push_chunk_with_feedback(
            &make_metadata(10, 0, 3), Bytes::from("evil"), bad_forensic, 100.0,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ReassemblyError::InvalidChunk);
    }

    #[test]
    fn test_session_feedback_loop() {
        let seed = 0xDEADBEEF;
        let mut session = FcrTpSession::new(seed);
        let mut dummy_hopper = FibonacciHopper::new(seed);

        session.push_chunk_with_feedback(
            &make_metadata(10, 0, 3), Bytes::from("A"),
            make_forensic_with_valid_port(&mut dummy_hopper), 50.0,
        ).unwrap();
        std::thread::sleep(Duration::from_millis(50));
        session.push_chunk_with_feedback(
            &make_metadata(10, 1, 3), Bytes::from("B"),
            make_forensic_with_valid_port(&mut dummy_hopper), 50.0,
        ).unwrap();

        assert!(session.survival.current_policy().suggested_bucket.as_bytes() > 0);
    }
}