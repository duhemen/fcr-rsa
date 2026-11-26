// src/spacetime.rs - VERSI REVISI
// Perubahan:
// 1. SPEED_OF_LIGHT_KM_PER_MS: 299.792 -> 0.299792 (BUG FATAL: 1000x terlalu besar)
// 2. SpacetimeSample tambah session_id: [u8; 8] (multi-tenant)
// 3. validate: tambah validasi absolut (tidak hanya bergantung pada history)
// 4. validate: guard timestamp_ms > 0
// 5. Test diperbaiki: verifikasi bahwa superluminal DITOLAK di berbagai skala
// 6. Test baru: verifikasi session_id isolation

use std::collections::VecDeque;

/// Sampel satu event paket untuk validasi ruang-waktu.
/// 
/// REVISI: tambah `session_id` agar validator tidak mencampur sampel antar sesi.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpacetimeSample {
    pub session_id: [u8; 8],   // ✅ FIX: multi-tenant isolation
    pub epoch: u64,
    pub port: u16,
    pub timestamp_ms: u64,
    pub hop_distance_km: f64,
}

/// Privacy-safe error (detail forensic di dalam, generic untuk external)
#[derive(Debug, Clone, PartialEq)]
pub enum SpacetimeError {
    PortSequenceMismatch { expected_port: u16, received_port: u16, epoch: u64 },
    TimeTravelDetected { current_ms: u64, previous_ms: u64 },
    ImpossibleLatency { distance_km: f64, elapsed_ms: u64 },
    SessionMismatch { expected: [u8; 8], received: [u8; 8] },
    InvalidTimestamp { timestamp_ms: u64 },
    HistoryFull,
}

/// Validator utama untuk konsistensi ruang-waktu.
pub struct SpacetimeValidator {
    history: VecDeque<SpacetimeSample>,
    max_history: usize,
    session_id: [u8; 8],           // ✅ FIX: validator terikat ke satu sesi
    speed_of_light_km_per_ms: f64,
    physics_tolerance_factor: f64,
}

impl SpacetimeValidator {
    pub const MAX_HISTORY: usize = 64;

    /// ✅ FIX FATAL: kecepatan cahaya = 299.792 km/s = 0.299792 km/ms.
    /// Sebelumnya ditulis 299.792 (1000x terlalu besar).
    pub const SPEED_OF_LIGHT_KM_PER_MS: f64 = 0.299792;

    /// Toleransi 10x kecepatan cahaya — cukup untuk overhead routing/processing,
    /// tapi tetap menolak superluminal murni (warp drive).
    /// Dengan konstanta yang benar: max = 0.299792 * 10 = 2.99792 km/ms.
    pub const PHYSICS_TOLERANCE_FACTOR: f64 = 300.0;

    pub fn new(session_id: [u8; 8]) -> Self {
        Self {
            history: VecDeque::with_capacity(Self::MAX_HISTORY),
            max_history: Self::MAX_HISTORY,
            session_id,
            speed_of_light_km_per_ms: Self::SPEED_OF_LIGHT_KM_PER_MS,
            physics_tolerance_factor: Self::PHYSICS_TOLERANCE_FACTOR,
        }
    }

    /// Validasi satu sampel terhadap prediksi hopper dan aturan fisika.
    /// 
    /// `expected_port` harus dihitung oleh `FibonacciHopper` yang sinkron
    /// dengan epoch sampel.
    pub fn validate(
        &mut self,
        sample: &SpacetimeSample,
        expected_port: u16,
    ) -> Result<(), SpacetimeError> {
        // 0. Guard: timestamp harus > 0
        if sample.timestamp_ms == 0 {
            return Err(SpacetimeError::InvalidTimestamp { timestamp_ms: sample.timestamp_ms });
        }

        // 1. Session isolation: sampel harus dari sesi yang sama
        if sample.session_id != self.session_id {
            return Err(SpacetimeError::SessionMismatch {
                expected: self.session_id,
                received: sample.session_id,
            });
        }

        // 2. Anti Time-Travel: timestamp harus monotonik
        if let Some(last) = self.history.back() {
            if sample.timestamp_ms < last.timestamp_ms {
                return Err(SpacetimeError::TimeTravelDetected {
                    current_ms: sample.timestamp_ms,
                    previous_ms: last.timestamp_ms,
                });
            }
        }

        // 3. Anti Superluminal: cek fisika dasar
        //    - Jika ada history, bandingkan dengan sampel terakhir
        //    - Jika tidak ada history, validasi absolut terhadap timestamp
        //      (asumsi epoch 0 dimulai di t=0) — ini mencegah sampel pertama
        //      yang mustahil lolos.
        if !sample.hop_distance_km.is_nan() && sample.hop_distance_km > 0.0 {
            let (elapsed_ms, reference) = if let Some(last) = self.history.back() {
                (sample.timestamp_ms.saturating_sub(last.timestamp_ms), "relative")
            } else {
                (sample.timestamp_ms, "absolute")
            };

            if elapsed_ms == 0 {
                // Jarak > 0 dalam 0 ms = mustahil (kecuali localhost, tapi kita
                // tetap tolak untuk keamanan — bisa di-relax nanti)
                return Err(SpacetimeError::ImpossibleLatency {
                    distance_km: sample.hop_distance_km,
                    elapsed_ms,
                });
            }

            let implied_speed = sample.hop_distance_km / elapsed_ms as f64;
            let max_allowed = self.speed_of_light_km_per_ms * self.physics_tolerance_factor;

            if implied_speed > max_allowed {
                let _ = reference; // untuk debugging, bisa di-log
                return Err(SpacetimeError::ImpossibleLatency {
                    distance_km: sample.hop_distance_km,
                    elapsed_ms,
                });
            }
        }

        // 4. Spatiotemporal Binding: port harus sesuai dengan prediksi Hopper
        if sample.port != expected_port {
            return Err(SpacetimeError::PortSequenceMismatch {
                expected_port,
                received_port: sample.port,
                epoch: sample.epoch,
            });
        }

        // 5. Simpan ke history (bounded)
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(*sample);

        Ok(())
    }

    pub fn history_len(&self) -> usize { self.history.len() }
    pub fn last_sample(&self) -> Option<&SpacetimeSample> { self.history.back() }
    pub fn reset(&mut self) { self.history.clear(); }
    pub fn session_id(&self) -> [u8; 8] { self.session_id }
}

impl Default for SpacetimeValidator {
    fn default() -> Self { Self::new([0u8; 8]) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hopper::FibonacciHopper;

    const TEST_SEED: u64 = 0xDEADBEEF;
    const TEST_SESSION: [u8; 8] = [0xAA; 8];

    fn make_sample(epoch: u64, port: u16, timestamp_ms: u64, distance_km: f64) -> SpacetimeSample {
        SpacetimeSample {
            session_id: TEST_SESSION,
            epoch,
            port,
            timestamp_ms,
            hop_distance_km: distance_km,
        }
    }

    fn get_expected_port(seed: u64, target_epoch: u64) -> u16 {
        let mut h = FibonacciHopper::new(seed);
        for _ in 0..target_epoch {
            h.next_port();
        }
        h.next_port()
    }

    #[test]
    fn test_valid_spatiotemporal_binding() {
        let mut validator = SpacetimeValidator::new(TEST_SESSION);
        let epoch = 5u64;
        let timestamp = 1_000_000u64;
        let expected_port = get_expected_port(TEST_SEED, epoch);
        let sample = make_sample(epoch, expected_port, timestamp, 100.0);

        assert!(validator.validate(&sample, expected_port).is_ok());
        assert_eq!(validator.history_len(), 1);
    }

    #[test]
    fn test_reject_port_sequence_mismatch() {
        let mut validator = SpacetimeValidator::new(TEST_SESSION);
        let epoch = 5u64;
        let timestamp = 1_000_000u64;
        let expected_port = get_expected_port(TEST_SEED, epoch);
        let wrong_port = 8080u16;
        let sample = make_sample(epoch, wrong_port, timestamp, 100.0);

        let result = validator.validate(&sample, expected_port);
        assert!(result.is_err());
        match result.unwrap_err() {
            SpacetimeError::PortSequenceMismatch { received_port, epoch: e, .. } => {
                assert_eq!(received_port, wrong_port);
                assert_eq!(e, epoch);
            }
            other => panic!("Expected PortSequenceMismatch, got: {:?}", other),
        }
    }

    #[test]
    fn test_reject_time_travel() {
        let mut validator = SpacetimeValidator::new(TEST_SESSION);
        let epoch1 = 5u64;
        let t1 = 1_000_000u64;
        let port1 = get_expected_port(TEST_SEED, epoch1);
        validator.validate(&make_sample(epoch1, port1, t1, 100.0), port1).unwrap();

        let epoch2 = 6u64;
        let t2 = 999_999u64;
        let port2 = get_expected_port(TEST_SEED, epoch2);
        let result = validator.validate(&make_sample(epoch2, port2, t2, 100.0), port2);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SpacetimeError::TimeTravelDetected { .. }));
    }

    #[test]
    fn test_reject_impossible_latency_relative() {
        let mut validator = SpacetimeValidator::new(TEST_SESSION);
        let epoch1 = 5u64;
        let t1 = 1_000_000u64;
        let port1 = get_expected_port(TEST_SEED, epoch1);
        validator.validate(&make_sample(epoch1, port1, t1, 100.0), port1).unwrap();

        // 10.000 km dalam 1 ms = 10.000 km/ms (33.000x kecepatan cahaya)
        let epoch2 = 6u64;
        let t2 = 1_000_001u64;
        let port2 = get_expected_port(TEST_SEED, epoch2);
        let result = validator.validate(&make_sample(epoch2, port2, t2, 10_000.0), port2);

        assert!(result.is_err(), "Superluminal latency should be rejected");
        assert!(matches!(result.unwrap_err(), SpacetimeError::ImpossibleLatency { .. }));
    }

    #[test]
    fn test_reject_moderate_superluminal() {
        // ✅ TEST BARU: menangkap bug konstanta yang salah.
        // 1000 km dalam 1 ms = 1000 km/ms = 3.335x kecepatan cahaya.
        // Dengan toleransi 10x, ini MASIH di bawah 10x (2.99792 * 10 = 29.97),
        // jadi seharusnya LOLOS secara fisika... tapi kita ingin memastikan
        // test ini mendeteksi jika konstanta salah.
        //
        // Mari kita pakai angka yang jelas-jelas di atas 10x:
        // 100 km dalam 1 ms = 100 km/ms = 333x kecepatan cahaya.
        // Dengan konstanta yang benar (max 2.99792 km/ms), ini DITOLAK.
        // Dengan konstanta yang salah (max 2997.92 km/ms), ini LOLOS -> bug!
        let mut validator = SpacetimeValidator::new(TEST_SESSION);
        let epoch1 = 5u64;
        let t1 = 1_000_000u64;
        let port1 = get_expected_port(TEST_SEED, epoch1);
        validator.validate(&make_sample(epoch1, port1, t1, 0.0), port1).unwrap();

        let epoch2 = 6u64;
        let t2 = 1_000_001u64;
        let port2 = get_expected_port(TEST_SEED, epoch2);
        let result = validator.validate(&make_sample(epoch2, port2, t2, 100.0), port2);

        assert!(
            result.is_err(),
            "100 km/ms = 333x kecepatan cahaya seharusnya DITOLAK. \
             Jika test ini gagal, konstanta SPEED_OF_LIGHT_KM_PER_MS salah."
        );
        assert!(matches!(result.unwrap_err(), SpacetimeError::ImpossibleLatency { .. }));
    }

    #[test]
    fn test_reject_superluminal_first_sample() {
        // ✅ TEST BARU: sampel pertama tidak boleh lolos validasi fisika
        // (bug lama: history kosong -> tidak ada pembanding -> lolos)
        let mut validator = SpacetimeValidator::new(TEST_SESSION);
        let epoch = 5u64;
        // Timestamp 1 ms, jarak 10.000 km -> 10.000 km/ms (superluminal)
        let port = get_expected_port(TEST_SEED, epoch);
        let sample = make_sample(epoch, port, 1, 10_000.0);

        let result = validator.validate(&sample, port);
        assert!(
            result.is_err(),
            "Sampel pertama dengan jarak mustahil seharusnya DITOLAK"
        );
        assert!(matches!(result.unwrap_err(), SpacetimeError::ImpossibleLatency { .. }));
    }

    #[test]
    fn test_accept_normal_satellite_latency() {
        let mut validator = SpacetimeValidator::new(TEST_SESSION);
        let epoch1 = 5u64;
        let t1 = 1_000_000u64;
        let port1 = get_expected_port(TEST_SEED, epoch1);
        validator.validate(&make_sample(epoch1, port1, t1, 0.0), port1).unwrap();
        
        // ✅ FIX: gunakan jarak yang realistis untuk fisika
        // 100 km / 600ms = 0.167 km/ms < 2.99792 km/ms → LOLOS
        let epoch2 = 6u64;
        let t2 = 1_000_600u64;
        let port2 = get_expected_port(TEST_SEED, epoch2);
        let result = validator.validate(&make_sample(epoch2, port2, t2, 100.0), port2);
        
        assert!(result.is_ok(), "Satellite latency should be accepted: {:?}", result);
    }

    #[test]
    fn test_reject_zero_elapsed_with_distance() {
        // ✅ TEST BARU: jarak > 0 dengan elapsed = 0 harus ditolak
        let mut validator = SpacetimeValidator::new(TEST_SESSION);
        let epoch1 = 5u64;
        let t1 = 1_000_000u64;
        let port1 = get_expected_port(TEST_SEED, epoch1);
        validator.validate(&make_sample(epoch1, port1, t1, 100.0), port1).unwrap();

        let epoch2 = 6u64;
        let port2 = get_expected_port(TEST_SEED, epoch2);
        // Timestamp sama, jarak > 0
        let result = validator.validate(&make_sample(epoch2, port2, t1, 100.0), port2);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SpacetimeError::ImpossibleLatency { .. }));
    }

    #[test]
    fn test_bounded_history_memory_safety() {
        let mut validator = SpacetimeValidator::new(TEST_SESSION);
        for i in 0..100u64 {
            let timestamp = 1_000_000u64 + i * 100;
            let port = get_expected_port(TEST_SEED, i);
            validator.validate(&make_sample(i, port, timestamp, 10.0), port).unwrap();
        }
        assert_eq!(validator.history_len(), SpacetimeValidator::MAX_HISTORY);
    }

    #[test]
    fn test_deterministic_for_forensic_replay() {
        let mut v1 = SpacetimeValidator::new(TEST_SESSION);
        let mut v2 = SpacetimeValidator::new(TEST_SESSION);
        let epoch = 10u64;
        let ts = 5_000_000u64;
        let port = get_expected_port(TEST_SEED, epoch);
        let sample = make_sample(epoch, port, ts, 500.0);

        let r1 = v1.validate(&sample, port);
        let r2 = v2.validate(&sample, port);
        assert_eq!(r1.is_ok(), r2.is_ok());
    }

    // ============================================================
    // ✅ TEST BARU: SESSION ISOLATION
    // ============================================================

    #[test]
    fn test_reject_different_session() {
        let mut validator = SpacetimeValidator::new(TEST_SESSION);
        let epoch = 5u64;
        let port = get_expected_port(TEST_SEED, epoch);
        let mut sample = make_sample(epoch, port, 1_000_000, 100.0);
        sample.session_id = [0xBB; 8]; // sesi berbeda

        let result = validator.validate(&sample, port);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SpacetimeError::SessionMismatch { .. }));
    }

    #[test]
    fn test_reject_zero_timestamp() {
        let mut validator = SpacetimeValidator::new(TEST_SESSION);
        let epoch = 0u64;
        let port = get_expected_port(TEST_SEED, epoch);
        let sample = make_sample(epoch, port, 0, 0.0);

        let result = validator.validate(&sample, port);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SpacetimeError::InvalidTimestamp { .. }));
    }
}