// src/reassembler.rs - VERSI FINAL (Hutang Lunas & Terintegrasi)
use bytes::{Bytes, BytesMut};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::crypto::ChunkMetadata;
use crate::survival::{SurvivalPolicy, ControlTier, NetworkMetrics};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageId([u8; 32]);

impl MessageId {
    pub fn new(bytes: [u8; 32]) -> Self { Self(bytes) }
    pub fn as_bytes(&self) -> &[u8; 32] { &self.0 }
}

impl From<u64> for MessageId {
    fn from(value: u64) -> Self {
        let mut bytes = [0u8; 32];
        bytes[..8].copy_from_slice(&value.to_be_bytes());
        Self(bytes)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForensicData {
    pub trace_id: String,
    pub session_id: u64,
    pub timestamp: Instant,
    pub rtt_ms: Option<u64>,
    pub packet_loss: Option<f32>,
}

#[derive(Debug, PartialEq)]
pub struct ReassemblyResult {
    pub message_id: [u8; 32], // HUTANG DILUNASI: Jejak audit eksplisit di hasil
    pub payload: Bytes,
    pub is_complete: bool,
    pub missing_chunks: Vec<u16>,
    pub forensic_metadata: ForensicData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReassemblyError {
    IncompleteMessage { message_id: [u8; 32] },
    InvalidChunk,
    Timeout { message_id: [u8; 32] },
    BufferFull,
    DuplicateMessage,
}

#[derive(Debug, Clone)]
struct BufferedChunk {
    index: u16, // Akan divalidasi saat reassembly
    data: Bytes,
    received_at: Instant,
}

#[derive(Debug)]
struct PendingMessage {
    message_id: MessageId, // Akan digunakan untuk tracking forensik
    total_chunks: u16,
    chunks: HashMap<u16, BufferedChunk>,
    first_chunk_at: Instant,
    timeout: Duration,
    forensic: ForensicData,
}

pub struct Reassembler {
    pending: HashMap<MessageId, PendingMessage>,
    max_buffered_messages: usize,
    default_timeout: Duration,
}

impl Reassembler {
    pub const MAX_BUFFERED_MESSAGES: usize = 256;
    pub const DEFAULT_TIMEOUT_MS: u64 = 5000;
    
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
            max_buffered_messages: Self::MAX_BUFFERED_MESSAGES,
            default_timeout: Duration::from_millis(Self::DEFAULT_TIMEOUT_MS),
        }
    }

    // Getter untuk testing dan observabilitas
    pub fn current_timeout(&self) -> Duration {
        self.default_timeout
    }
    
    pub fn calculate_timeout(rtt_ms: u64, packet_loss: f32) -> Duration {
        let base = Duration::from_millis(rtt_ms.saturating_mul(3));
        let loss_penalty = Duration::from_millis((packet_loss * 1000.0) as u64);
        base.saturating_add(loss_penalty)
    }

    pub fn apply_policy(&mut self, policy: &SurvivalPolicy) {
        let base = Duration::from_millis(Self::DEFAULT_TIMEOUT_MS);
        self.default_timeout = base.mul_f32(policy.timeout_multiplier);

        if policy.tier == ControlTier::Enforce {
            self.cleanup(); 
        }
    }

    pub fn extract_assembly_metrics(&self, message_id: &MessageId) -> Option<NetworkMetrics> {
        let pending = self.pending.get(message_id)?;
        if pending.chunks.len() < 2 { return None; }

        let mut arrival_times: Vec<Instant> = pending.chunks.values()
            .map(|c| c.received_at)
            .collect();
        arrival_times.sort();

        let mut total_diff_ms = 0.0;
        for i in 1..arrival_times.len() {
            total_diff_ms += arrival_times[i].duration_since(arrival_times[i-1]).as_millis() as f64;
        }
        let avg_jitter = total_diff_ms / (arrival_times.len() - 1) as f64;
        let loss_rate = 1.0 - (pending.chunks.len() as f64 / pending.total_chunks as f64);

        Some(NetworkMetrics {
            rtt_ms: 0.0,
            jitter_ms: avg_jitter,
            loss_rate,
            throughput_bps: 0.0,
        })
    }
    
    pub fn push_chunk(
        &mut self,
        metadata: &ChunkMetadata,
        data: Bytes,
        forensic: ForensicData,
    ) -> Result<Option<ReassemblyResult>, ReassemblyError> {
        if metadata.chunk_index >= metadata.total_chunks {
            return Err(ReassemblyError::InvalidChunk);
        }
        
        let message_id = MessageId::from(metadata.message_id);
        
        if let Some(pending) = self.pending.get_mut(&message_id) {
            if pending.total_chunks != metadata.total_chunks {
                return Err(ReassemblyError::InvalidChunk);
            }
            if pending.first_chunk_at.elapsed() > pending.timeout {
                self.pending.remove(&message_id);
                return Err(ReassemblyError::Timeout { message_id: message_id.0 }); 
            }
            if pending.chunks.contains_key(&metadata.chunk_index) {
                return Ok(None); 
            }
            
            pending.chunks.insert(
                metadata.chunk_index,
                BufferedChunk { index: metadata.chunk_index, data, received_at: Instant::now() },
            );
            
            if pending.chunks.len() == pending.total_chunks as usize {
                let result = self.assemble_complete(&message_id)?;
                self.pending.remove(&message_id);
                return Ok(Some(result));
            }
            Ok(None)
        } else {
            if self.pending.len() >= self.max_buffered_messages {
                return Err(ReassemblyError::BufferFull);
            }
            
            let mut chunks = HashMap::new();
            chunks.insert(
                metadata.chunk_index,
                BufferedChunk { index: metadata.chunk_index, data, received_at: Instant::now() },
            );
            
            let pending = PendingMessage {
                message_id,
                total_chunks: metadata.total_chunks,
                chunks,
                first_chunk_at: Instant::now(),
                timeout: self.default_timeout,
                forensic,
            };
            self.pending.insert(message_id, pending);
            
            if metadata.total_chunks == 1 {
                let result = self.assemble_complete(&message_id)?;
                self.pending.remove(&message_id);
                return Ok(Some(result));
            }
            Ok(None)
        }
    }
    
    fn assemble_complete(&self, message_id: &MessageId) -> Result<ReassemblyResult, ReassemblyError> {
        let pending = self.pending.get(message_id).ok_or(ReassemblyError::IncompleteMessage { message_id: message_id.0 })?;
        let mut payload = BytesMut::with_capacity(pending.chunks.len() * 1024);
        let mut missing_chunks = Vec::new();
        
        for i in 0..pending.total_chunks {
            if let Some(chunk) = pending.chunks.get(&i) {
                // HUTANG DILUNASI: Validasi integritas index secara aktif (membaca field `index`)
                debug_assert_eq!(chunk.index, i, "Forensic mismatch: chunk index tidak sesuai dengan urutan");
                payload.extend_from_slice(&chunk.data);
            } else {
                missing_chunks.push(i);
            }
        }
        
        Ok(ReassemblyResult {
            message_id: pending.message_id.0, // HUTANG DILUNASI: Menggunakan field message_id
            payload: payload.freeze(),
            is_complete: missing_chunks.is_empty(),
            missing_chunks,
            forensic_metadata: pending.forensic.clone(),
        })
    }
    
    pub fn cleanup(&mut self) -> Vec<MessageId> {
        let mut timed_out = Vec::new();
        self.pending.retain(|&message_id, pending| {
            if pending.first_chunk_at.elapsed() > pending.timeout {
                timed_out.push(message_id);
                false
            } else { true }
        });
        timed_out
    }
    
    pub fn pending_count(&self) -> usize { self.pending.len() }
    
    pub fn try_partial_assembly(&self, message_id: &MessageId) -> Option<ReassemblyResult> {
        let pending = self.pending.get(message_id)?;
        let mut payload = BytesMut::with_capacity(pending.chunks.len() * 1024);
        let mut missing_chunks = Vec::new();
        
        for i in 0..pending.total_chunks {
            if let Some(chunk) = pending.chunks.get(&i) {
                debug_assert_eq!(chunk.index, i, "Forensic mismatch: chunk index tidak sesuai");
                payload.extend_from_slice(&chunk.data);
            } else {
                missing_chunks.push(i);
            }
        }
        
        Some(ReassemblyResult {
            message_id: pending.message_id.0,
            payload: payload.freeze(),
            is_complete: missing_chunks.is_empty(),
            missing_chunks,
            forensic_metadata: pending.forensic.clone(),
        })
    }
}

impl Default for Reassembler { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::ChunkMetadata;
    use crate::survival::{SurvivalPolicy, ControlTier, BucketSize};
    
    fn make_metadata(message_id: u64, chunk_index: u16, total_chunks: u16) -> ChunkMetadata {
        ChunkMetadata { message_id, version: 1, chunk_index, total_chunks, is_final: chunk_index == total_chunks - 1 }
    }
    
    fn make_forensic() -> ForensicData {
        ForensicData { trace_id: "test-trace-123".to_string(), session_id: 42, timestamp: Instant::now(), rtt_ms: None, packet_loss: None }
    }

    #[test]
    fn test_single_chunk_message() {
        let mut reassembler = Reassembler::new();
        let metadata = make_metadata(1, 0, 1);
        let data = Bytes::from("hello");
        let result = reassembler.push_chunk(&metadata, data.clone(), make_forensic()).unwrap();
        assert!(result.is_some());
        let res = result.unwrap();
        assert_eq!(res.payload, data);
        assert_eq!(res.message_id, MessageId::from(1u64).0); // Validasi message_id baru
    }

    #[test]
    fn test_multi_chunk_in_order() {
        let mut reassembler = Reassembler::new();
        reassembler.push_chunk(&make_metadata(2, 0, 3), Bytes::from("hel"), make_forensic()).unwrap();
        reassembler.push_chunk(&make_metadata(2, 1, 3), Bytes::from("lo "), make_forensic()).unwrap();
        let result = reassembler.push_chunk(&make_metadata(2, 2, 3), Bytes::from("world"), make_forensic()).unwrap();
        assert_eq!(result.unwrap().payload, Bytes::from("hello world"));
    }

    #[test]
    fn test_multi_chunk_out_of_order() {
        let mut reassembler = Reassembler::new();
        reassembler.push_chunk(&make_metadata(3, 2, 3), Bytes::from("world"), make_forensic()).unwrap();
        reassembler.push_chunk(&make_metadata(3, 0, 3), Bytes::from("hel"), make_forensic()).unwrap();
        let result = reassembler.push_chunk(&make_metadata(3, 1, 3), Bytes::from("lo "), make_forensic()).unwrap();
        assert_eq!(result.unwrap().payload, Bytes::from("hello world"));
    }

    #[test]
    fn test_duplicate_chunk_idempotent() {
        let mut reassembler = Reassembler::new();
        let meta = make_metadata(4, 0, 2);
        let data = Bytes::from("test");
        reassembler.push_chunk(&meta, data.clone(), make_forensic()).unwrap();
        reassembler.push_chunk(&meta, data, make_forensic()).unwrap(); // Harus diabaikan, bukan error
        assert_eq!(reassembler.pending_count(), 1);
    }

    #[test]
    fn test_invalid_chunk_index() {
        let mut reassembler = Reassembler::new();
        let meta = make_metadata(5, 5, 3); // chunk_index >= total_chunks
        let result = reassembler.push_chunk(&meta, Bytes::from("test"), make_forensic());
        assert_eq!(result, Err(ReassemblyError::InvalidChunk));
    }

    #[test]
    fn test_inconsistent_total_chunks() {
        let mut reassembler = Reassembler::new();
        reassembler.push_chunk(&make_metadata(6, 0, 3), Bytes::from("test"), make_forensic()).unwrap();
        let result = reassembler.push_chunk(&make_metadata(6, 1, 5), Bytes::from("test2"), make_forensic()); // Total chunks berubah
        assert_eq!(result, Err(ReassemblyError::InvalidChunk));
    }

    #[test]
    fn test_buffer_full() {
        let mut reassembler = Reassembler::new();
        reassembler.max_buffered_messages = 2;
        reassembler.push_chunk(&make_metadata(1, 0, 2), Bytes::from("test"), make_forensic()).unwrap();
        reassembler.push_chunk(&make_metadata(2, 0, 2), Bytes::from("test"), make_forensic()).unwrap();
        
        let result = reassembler.push_chunk(&make_metadata(100, 0, 2), Bytes::from("test"), make_forensic());
        assert_eq!(result, Err(ReassemblyError::BufferFull));
    }

    #[test]
    fn test_cleanup_timeout() {
        let mut reassembler = Reassembler::new();
        reassembler.default_timeout = Duration::from_millis(100);
        reassembler.push_chunk(&make_metadata(7, 0, 2), Bytes::from("test"), make_forensic()).unwrap();
        assert_eq!(reassembler.pending_count(), 1);
        
        std::thread::sleep(Duration::from_millis(150));
        let timed_out = reassembler.cleanup();
        assert_eq!(timed_out.len(), 1);
        assert_eq!(reassembler.pending_count(), 0);
    }

    #[test]
    fn test_adaptive_timeout_calculation() {
        let timeout = Reassembler::calculate_timeout(100, 0.1); // 100ms RTT, 10% loss
        assert!(timeout > Duration::from_millis(300)); // 3x RTT + penalty
    }

    #[test]
    fn test_partial_assembly() {
        let mut reassembler = Reassembler::new();
        let msg_id = MessageId::from(8u64);
        reassembler.push_chunk(&make_metadata(8, 0, 3), Bytes::from("hel"), make_forensic()).unwrap();
        reassembler.push_chunk(&make_metadata(8, 1, 3), Bytes::from("lo"), make_forensic()).unwrap();
        
        let partial = reassembler.try_partial_assembly(&msg_id).unwrap();
        assert_eq!(partial.payload, Bytes::from("hello"));
        assert!(!partial.is_complete);
        assert_eq!(partial.missing_chunks, vec![2]);
    }

    #[test]
    fn test_forensic_metadata_preserved() {
        let mut reassembler = Reassembler::new();
        let mut forensic = make_forensic();
        forensic.trace_id = "custom-trace-456".to_string();
        forensic.session_id = 99;
        
        let result = reassembler.push_chunk(&make_metadata(9, 0, 1), Bytes::from("test"), forensic.clone()).unwrap().unwrap();
        assert_eq!(result.forensic_metadata.trace_id, "custom-trace-456");
        assert_eq!(result.forensic_metadata.session_id, 99);
    }

    #[test]
    fn test_policy_application_enforce() {
        let mut reassembler = Reassembler::new();
        let policy = SurvivalPolicy {
            tier: ControlTier::Enforce,
            suggested_bucket: BucketSize::B256,
            timeout_multiplier: 5.0,
            enable_partial_assembly: true,
        };
        reassembler.apply_policy(&policy);
        assert_eq!(reassembler.current_timeout(), Duration::from_millis(25000)); // 5000 * 5.0
    }

    #[test]
    fn test_extract_jitter_metrics() {
        let mut reassembler = Reassembler::new();
        let msg_id = MessageId::from(99u64);
        
        reassembler.push_chunk(&make_metadata(99, 0, 3), Bytes::from("A"), make_forensic()).unwrap();
        std::thread::sleep(Duration::from_millis(50));
        reassembler.push_chunk(&make_metadata(99, 1, 3), Bytes::from("B"), make_forensic()).unwrap();
        
        let metrics = reassembler.extract_assembly_metrics(&msg_id).unwrap();
        assert!(metrics.jitter_ms >= 40.0, "Jitter should be around 50ms, got: {}", metrics.jitter_ms);
        
        let expected_loss = 1.0 / 3.0;
        assert!(
            (metrics.loss_rate - expected_loss).abs() < 1e-10, 
            "Loss rate mismatch: expected ~{}, got {}", 
            expected_loss, 
            metrics.loss_rate
        );
    }
}