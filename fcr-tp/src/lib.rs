// src/lib.rs - VERSI FINAL (Perbaikan)
// 
// Perubahan dari versi sebelumnya:
// 1. Tambah `LinkProfile` ke re-export `survival` (fix E0603)
// 2. Test `test_link_profile_accessible` sekarang bisa akses LinkProfile

//! # FCR-TP (FCR Transport Protocol)
//! 
//! Protokol lapisan transport yang aman, adaptif, dan ramah pengguna
//! untuk ekosistem FCR-RSA. Dirancang untuk tahan terhadap analisis lalu lintas
//! (Traffic Analysis) di era superposisi dan ASI.

pub mod packet;
pub mod hopper;
pub mod crypto;
pub mod defense;
pub mod survival;
pub mod reassembler;
pub mod anomaly;
pub mod spacetime;
pub mod session;

// ============================================================
// RE-EXPORTS: Tipe Utama
// ============================================================

// --- Packet layer ---
pub use packet::FcrTpFrame;
pub use packet::HeaderMode;
pub use packet::{
    BUCKET_SIZES,
    MIN_DATAGRAM_SIZE,
    MAX_DATAGRAM_SIZE,
    AEAD_TAG_SIZE,
    TOKEN_SIZE,
};

// --- Crypto layer ---
pub use crypto::FcrTpSession as CryptoSession;
pub use crypto::{
    ChunkMetadata,
    CURRENT_CRYPTO_VERSION,
    MAX_CHUNKS_PER_MESSAGE,
    CHUNK_METADATA_SIZE,
    PLAINTEXT_LEN_SIZE,
    NONCE_SIZE,
    DEFAULT_BUCKET_SIZE,
    FRAME_OVERHEAD,
};

// --- Reassembler layer ---
pub use reassembler::Reassembler;
pub use reassembler::{
    ForensicData,
    ReassemblyResult,
    ReassemblyError,
    MessageId,
};

// --- Anomaly layer ---
pub use anomaly::{
    AnomalyDetector,
    AnomalyReport,
    AnomalyType,
    SessionContext,
    AdaptiveThreshold,
    ProfileSource,
    // LinkProfile TIDAK di sini (dari survival)
    META_TUNE_INTERVAL,
    MIN_SAMPLES_FOR_LEARNING,
    GRACE_PERIOD_SAMPLES,
};

// --- Survival layer ---
// 
// ✅ FIX: `LinkProfile` ditambahkan ke re-export survival
// (sebelumnya hanya di anomaly, tapi anomaly tidak mengekspornya)
pub use survival::{
    SurvivalModeController,
    SurvivalPolicy,
    NetworkMetrics,
    NetworkHealthScore,
    BucketSize,
    ControlTier,
    LinkProfile,  // ✅ TAMBAHKAN INI
    detect_link_profile_from_metrics,
};

// --- Defense layer ---
pub use defense::{
    DefenseEngine,
    DefenseAction,
    TransportRiskScore,
    HealthIndicator,
    UserStatus,
    TechnicalDetails,
    AnomalyCategory,
    CANARY_THRESHOLD,
    DISCONNECT_THRESHOLD,
    MAX_SEEN_TOKENS,
    FORENSIC_OVERHEAD,
};

// --- Spacetime layer ---
pub use spacetime::{
    SpacetimeValidator,
    SpacetimeSample,
    SpacetimeError,
};

// --- Hopper layer ---
pub use hopper::FibonacciHopper;

// --- Session layer (orchestrator) ---
pub use session::TransportOrchestrator;

// ============================================================
// DOKUMENTASI TAMBAHAN
// ============================================================

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PROTOCOL_NAME: &str = "FCR-TP";
pub const PROTOCOL_DESCRIPTION: &str =
    "FCR Transport Protocol: Adaptive, Zero-Plaintext, and ASI-Resilient wrapper for FCR-RSA";

// ============================================================
// TEST
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_accessible() {
        assert!(!VERSION.is_empty());
    }
    
    #[test]
    fn test_protocol_name() {
        assert_eq!(PROTOCOL_NAME, "FCR-TP");
    }
    
    #[test]
    fn test_protocol_description() {
        assert!(!PROTOCOL_DESCRIPTION.is_empty());
    }
    
    #[test]
    fn test_crypto_session_alias() {
        let session = CryptoSession::new(
            [1, 2, 3, 4, 5, 6, 7, 8],
            &[0xAA; 32],
            CURRENT_CRYPTO_VERSION,
        );
        assert_eq!(session.crypto_version, CURRENT_CRYPTO_VERSION);
    }
    
    #[test]
    fn test_transport_orchestrator_accessible() {
        let orch = TransportOrchestrator::new(0xDEADBEEF, &[0xAA; 32]);
        assert_eq!(orch.current_epoch(), 0);
    }
    
    #[test]
    fn test_full_roundtrip_via_orchestrator() {
        let orch = TransportOrchestrator::new(0xDEADBEEF, &[0xAA; 32]);
        let plaintext = b"Halo dari lib.rs";
        
        let frames = orch.encrypt(plaintext).expect("Encrypt gagal");
        assert_eq!(frames.len(), 1);
        
        let wire = frames[0].encode().expect("Encode gagal");
        assert!(BUCKET_SIZES.contains(&wire.len()));
        
        let received = FcrTpFrame::decode(wire).expect("Decode gagal");
        let (metadata, decrypted) = orch.decrypt(&received).expect("Decrypt gagal");
        
        assert_eq!(decrypted, plaintext.to_vec());
        assert_eq!(metadata.version, CURRENT_CRYPTO_VERSION);
    }
    
    #[test]
    fn test_constants_accessible() {
        assert_eq!(CHUNK_METADATA_SIZE, 14);
        assert_eq!(PLAINTEXT_LEN_SIZE, 2);
        assert_eq!(NONCE_SIZE, 12);
        assert_eq!(AEAD_TAG_SIZE, 16);
        assert_eq!(TOKEN_SIZE, 8);
        assert_eq!(DEFAULT_BUCKET_SIZE, 1200);
        assert_eq!(FRAME_OVERHEAD, 52);
    }
    
    #[test]
    fn test_anomaly_constants_accessible() {
        assert_eq!(META_TUNE_INTERVAL, 100);
        assert_eq!(MIN_SAMPLES_FOR_LEARNING, 100);
        assert_eq!(GRACE_PERIOD_SAMPLES, 3);
    }
    
    #[test]
    fn test_defense_constants_accessible() {
        assert_eq!(CANARY_THRESHOLD, 30);
        assert_eq!(DISCONNECT_THRESHOLD, 100);
        assert_eq!(FORENSIC_OVERHEAD, 48);
    }
    
    #[test]
    fn test_health_indicator_accessible() {
        assert_eq!(HealthIndicator::Green.icon(), "🟢");
        assert_eq!(HealthIndicator::Yellow.icon(), "🟡");
        assert_eq!(HealthIndicator::Red.icon(), "🔴");
        assert_eq!(HealthIndicator::Gray.icon(), "⚪");
    }
    
    #[test]
    fn test_link_profile_accessible() {
        // ✅ FIX: LinkProfile sekarang diekspor dari survival
        assert_eq!(LinkProfile::Fiber.name(), "Fiber");
        assert_eq!(LinkProfile::Cellular.name(), "Cellular");
        assert_eq!(LinkProfile::Satellite.name(), "Satellite");
        assert_eq!(LinkProfile::Unknown.name(), "Unknown");
    }
    
    #[test]
    fn test_detect_link_profile_from_metrics_accessible() {
        let profile = detect_link_profile_from_metrics(10.0, 2.0, 0.0);
        assert_eq!(profile, LinkProfile::Fiber);
    }
    
    #[test]
    fn test_session_context_accessible() {
        let ctx = SessionContext::FileUpload;
        assert_eq!(ctx, SessionContext::FileUpload);
    }
    
    #[test]
    fn test_forensic_data_accessible() {
        use std::time::Instant;
        let forensic = ForensicData {
            trace_id: "test".to_string(),
            session_id: [1, 2, 3, 4, 5, 6, 7, 8],
            source_port: 50000,
            source_ip: Some([127, 0, 0, 1]),
            timestamp: Instant::now(),
            rtt_ms: Some(50),
            packet_loss: None,
        };
        assert_eq!(forensic.source_port, 50000);
    }
    
    #[test]
    fn test_spacetime_validator_accessible() {
        let validator = SpacetimeValidator::new([0xAA; 8]);
        assert_eq!(validator.session_id(), [0xAA; 8]);
    }
    
    #[test]
    fn test_hopper_accessible() {
        let mut hopper = FibonacciHopper::new(12345);
        let port = hopper.next_port();
        assert!(port >= 1024);
    }
}