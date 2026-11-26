// src/spacetime.rs - Spatiotemporal Integrity Validator (Diselaraskan dengan FibonacciHopper)
use std::collections::VecDeque;

/// Sampel satu event paket untuk validasi ruang-waktu
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpacetimeSample {
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
    HistoryFull,
}

/// Validator utama untuk konsistensi ruang-waktu
pub struct SpacetimeValidator {
    history: VecDeque<SpacetimeSample>,
    max_history: usize,
    speed_of_light_km_per_ms: f64,
    physics_tolerance_factor: f64,
}

impl SpacetimeValidator {
    pub const MAX_HISTORY: usize = 64;
    pub const SPEED_OF_LIGHT_KM_PER_MS: f64 = 299.792;
    /// Toleransi 10x kecepatan cahaya — cukup untuk overhead routing/processing,
    /// tapi tetap menolak superluminal murni (warp drive).
    pub const PHYSICS_TOLERANCE_FACTOR: f64 = 10.0;

    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(Self::MAX_HISTORY),
            max_history: Self::MAX_HISTORY,
            speed_of_light_km_per_ms: Self::SPEED_OF_LIGHT_KM_PER_MS,
            physics_tolerance_factor: Self::PHYSICS_TOLERANCE_FACTOR,
        }
    }

    pub fn validate(
        &mut self,
        sample: &SpacetimeSample,
        expected_port: u16,
    ) -> Result<(), SpacetimeError> {
        // 1. Anti Time-Travel: timestamp harus monotonik
        if let Some(last) = self.history.back() {
            if sample.timestamp_ms < last.timestamp_ms {
                return Err(SpacetimeError::TimeTravelDetected {
                    current_ms: sample.timestamp_ms,
                    previous_ms: last.timestamp_ms,
                });
            }
        }

        // 2. Anti Superluminal: cek fisika dasar
        if !sample.hop_distance_km.is_nan() && sample.hop_distance_km > 0.0 {
            if let Some(last) = self.history.back() {
                let elapsed_ms = sample.timestamp_ms.saturating_sub(last.timestamp_ms);
                if elapsed_ms > 0 {
                    let implied_speed = sample.hop_distance_km / elapsed_ms as f64;
                    let max_allowed = self.speed_of_light_km_per_ms * self.physics_tolerance_factor;
                    if implied_speed > max_allowed {
                        return Err(SpacetimeError::ImpossibleLatency {
                            distance_km: sample.hop_distance_km,
                            elapsed_ms,
                        });
                    }
                }
            }
        }

        // 3. Spatiotemporal Binding: port harus sesuai dengan prediksi Hopper
        if sample.port != expected_port {
            return Err(SpacetimeError::PortSequenceMismatch {
                expected_port,
                received_port: sample.port,
                epoch: sample.epoch,
            });
        }

        // 4. Simpan ke history (bounded)
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(*sample);

        Ok(())
    }

    pub fn history_len(&self) -> usize { self.history.len() }
    pub fn last_sample(&self) -> Option<&SpacetimeSample> { self.history.back() }
    pub fn reset(&mut self) { self.history.clear(); }
}

impl Default for SpacetimeValidator {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hopper::FibonacciHopper; // ✅ Dipindahkan ke dalam mod tests

    const TEST_SEED: u64 = 0xDEADBEEF;

    fn make_sample(epoch: u64, port: u16, timestamp_ms: u64, distance_km: f64) -> SpacetimeSample {
        SpacetimeSample { epoch, port, timestamp_ms, hop_distance_km: distance_km }
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
        let mut validator = SpacetimeValidator::new();
        let epoch = 5u64;
        let timestamp = 1_000_000u64;
        let expected_port = get_expected_port(TEST_SEED, epoch);
        let sample = make_sample(epoch, expected_port, timestamp, 100.0);

        assert!(validator.validate(&sample, expected_port).is_ok());
        assert_eq!(validator.history_len(), 1);
    }

    #[test]
    fn test_reject_port_sequence_mismatch() {
        let mut validator = SpacetimeValidator::new();
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
        let mut validator = SpacetimeValidator::new();
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
    fn test_reject_impossible_latency() {
        let mut validator = SpacetimeValidator::new();
        let epoch1 = 5u64;
        let t1 = 1_000_000u64;
        let port1 = get_expected_port(TEST_SEED, epoch1);
        validator.validate(&make_sample(epoch1, port1, t1, 100.0), port1).unwrap();

        // Paket kedua: 10.000 km dalam 1 ms = 10.000 km/ms (33x kecepatan cahaya)
        // Dengan toleransi 10x (max ~3.000 km/ms), ini harus ditolak
        let epoch2 = 6u64;
        let t2 = 1_000_001u64;
        let port2 = get_expected_port(TEST_SEED, epoch2);
        let result = validator.validate(&make_sample(epoch2, port2, t2, 10_000.0), port2);

        assert!(result.is_err(), "Superluminal latency should be rejected");
        assert!(matches!(result.unwrap_err(), SpacetimeError::ImpossibleLatency { .. }));
    }

    #[test]
    fn test_accept_normal_satellite_latency() {
        let mut validator = SpacetimeValidator::new();
        let epoch1 = 5u64;
        let t1 = 1_000_000u64;
        let port1 = get_expected_port(TEST_SEED, epoch1);
        validator.validate(&make_sample(epoch1, port1, t1, 0.0), port1).unwrap();

        // Satelit GEO: 36.000 km dalam 600ms = 60 km/ms (normal, di bawah 3.000 km/ms)
        let epoch2 = 6u64;
        let t2 = 1_000_600u64;
        let port2 = get_expected_port(TEST_SEED, epoch2);
        let result = validator.validate(&make_sample(epoch2, port2, t2, 36_000.0), port2);

        assert!(result.is_ok(), "Satellite latency should be accepted: {:?}", result);
    }

    #[test]
    fn test_bounded_history_memory_safety() {
        let mut validator = SpacetimeValidator::new();
        for i in 0..100u64 {
            let timestamp = 1_000_000u64 + i * 100;
            let port = get_expected_port(TEST_SEED, i);
            validator.validate(&make_sample(i, port, timestamp, 10.0), port).unwrap();
        }
        assert_eq!(validator.history_len(), SpacetimeValidator::MAX_HISTORY);
    }

    #[test]
    fn test_deterministic_for_forensic_replay() {
        let mut v1 = SpacetimeValidator::new();
        let mut v2 = SpacetimeValidator::new();
        let epoch = 10u64;
        let ts = 5_000_000u64;
        let port = get_expected_port(TEST_SEED, epoch);
        let sample = make_sample(epoch, port, ts, 500.0);

        let r1 = v1.validate(&sample, port);
        let r2 = v2.validate(&sample, port);
        assert_eq!(r1.is_ok(), r2.is_ok());
    }
}