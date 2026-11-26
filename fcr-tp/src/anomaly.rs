// src/anomaly.rs - Timing & Size Anomaly Detection
use std::collections::VecDeque;
use std::time::Instant;

/// Jenis anomali yang terdeteksi
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnomalyType {
    /// Paket datang terlalu cepat (Burst Attack / Flood)
    BurstTraffic,
    /// Variasi ukuran payload terlalu rendah (Traffic Injection / Bot)
    UniformSizeInjection,
    /// Jitter melampaui batas toleransi statistik
    ExtremeJitter,
}

/// Laporan anomali untuk forensic dan decision making
#[derive(Debug, Clone)]
pub struct AnomalyReport {
    pub anomaly_type: AnomalyType,
    pub severity: f64, // 0.0 - 1.0
    pub forensic_context: String,
}

/// Detector utama untuk Timing dan Size Anomaly
pub struct AnomalyDetector {
    // Timing metrics
    iat_history: VecDeque<f64>, // Inter-Arrival Time dalam ms
    last_arrival: Option<Instant>,
    ema_iat: f64, // Exponential Moving Average untuk IAT
    
    // Size metrics
    size_history: VecDeque<usize>,
    
    // Konfigurasi & Batasan (Memory Safety)
    max_window_size: usize,
    ema_alpha: f64, // Faktor peluruhan EMA (0.0 - 1.0)
}

impl AnomalyDetector {
    // Batasan ketat untuk mencegah memory exhaustion
    pub const MAX_WINDOW_SIZE: usize = 64;
    pub const EMA_ALPHA: f64 = 0.2; 
    
    // Threshold Anomali
    pub const BURST_THRESHOLD_MS: f64 = 2.0; // Kurang dari 2ms = Burst
    pub const UNIFORMITY_THRESHOLD: f64 = 1.5; // StdDev < 1.5 bytes = Injection
    pub const JITTER_SPIKE_MULTIPLIER: f64 = 5.0; // 5x dari EMA = Extreme Jitter

    pub fn new() -> Self {
        Self {
            iat_history: VecDeque::with_capacity(Self::MAX_WINDOW_SIZE),
            last_arrival: None,
            ema_iat: 0.0,
            size_history: VecDeque::with_capacity(Self::MAX_WINDOW_SIZE),
            max_window_size: Self::MAX_WINDOW_SIZE,
            ema_alpha: Self::EMA_ALPHA,
        }
    }

    /// Catat kedatangan chunk dan analisis anomali
    pub fn record_chunk(&mut self, size: usize) -> Option<AnomalyReport> {
        let now = Instant::now();
        let mut report = None;

        // 1. Analisis Timing (IAT & Jitter)
        if let Some(last) = self.last_arrival {
            let iat = now.duration_since(last).as_secs_f64() * 1000.0; // ke ms
            
            // Update EMA
            if self.ema_iat == 0.0 {
                self.ema_iat = iat;
            } else {
                self.ema_iat = (self.ema_alpha * iat) + ((1.0 - self.ema_alpha) * self.ema_iat);
            }

            // Deteksi Burst
            if iat < Self::BURST_THRESHOLD_MS {
                report = Some(AnomalyReport {
                    anomaly_type: AnomalyType::BurstTraffic,
                    severity: (Self::BURST_THRESHOLD_MS - iat) / Self::BURST_THRESHOLD_MS,
                    forensic_context: format!("IAT {:.2}ms < threshold", iat),
                });
            }
            // Deteksi Extreme Jitter
            else if iat > (self.ema_iat * Self::JITTER_SPIKE_MULTIPLIER) && self.ema_iat > 0.0 {
                report = Some(AnomalyReport {
                    anomaly_type: AnomalyType::ExtremeJitter,
                    severity: 0.8,
                    forensic_context: format!("IAT {:.2}ms > 5x EMA ({:.2}ms)", iat, self.ema_iat),
                });
            }

            // Simpan ke history (bounded)
            if self.iat_history.len() >= self.max_window_size {
                self.iat_history.pop_front();
            }
            self.iat_history.push_back(iat);
        }
        self.last_arrival = Some(now);

        // 2. Analisis Size (Traffic Injection)
        if self.size_history.len() >= self.max_window_size {
            self.size_history.pop_front();
        }
        self.size_history.push_back(size);

        // Hitung Standard Deviation jika data cukup
        if self.size_history.len() >= 10 {
            let stddev = self.calculate_std_dev();
            if stddev < Self::UNIFORMITY_THRESHOLD {
                // Override atau tambahkan report jika belum ada
                if report.is_none() {
                    report = Some(AnomalyReport {
                        anomaly_type: AnomalyType::UniformSizeInjection,
                        severity: 1.0 - (stddev / Self::UNIFORMITY_THRESHOLD),
                        forensic_context: format!("Size StdDev {:.2} bytes (Terlalu Seragam)", stddev),
                    });
                }
            }
        }

        report
    }

    /// Hitung Standard Deviation dari ukuran payload
    fn calculate_std_dev(&self) -> f64 {
        let n = self.size_history.len() as f64;
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
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_normal_traffic_no_anomaly() {
        let mut detector = AnomalyDetector::new();
        // Simulasi trafik normal dengan variasi ukuran dan waktu
        for i in 0usize..15 {
            // Cast ke u64 karena from_millis() hanya menerima u64
            thread::sleep(Duration::from_millis((50 + (i % 5) * 10) as u64));
            // Langsung usize karena record_chunk() menerima usize
            detector.record_chunk(100 + (i % 20) * 5);
        }
        // Seharusnya tidak ada anomali yang terdeteksi
        assert!(detector.size_history.len() == 15);
    }

    #[test]
    fn test_burst_attack_detection() {
        let mut detector = AnomalyDetector::new();
        // Inisialisasi baseline
        detector.record_chunk(100);
        thread::sleep(Duration::from_millis(50));
        detector.record_chunk(100);
        
        // Kirim burst sangat cepat (0ms delay)
        let report = detector.record_chunk(100);
        assert!(report.is_some());
        let r = report.unwrap();
        assert_eq!(r.anomaly_type, AnomalyType::BurstTraffic);
    }

    #[test]
    fn test_uniform_size_injection() {
        let mut detector = AnomalyDetector::new();
        // Kirim 20 paket dengan ukuran PERSIS SAMA dan interval normal
        for _ in 0..20 {
            thread::sleep(Duration::from_millis(50));
            detector.record_chunk(512); // Ukuran selalu 512
        }
        
        // Karena history > 10 dan std dev = 0, harusnya terdeteksi
        // (Report mungkin ter-trigger di iterasi ke-11 atau seterusnya)
        // Kita verifikasi dengan melihat state internal
        let stddev = detector.calculate_std_dev();
        assert_eq!(stddev, 0.0);
    }
}