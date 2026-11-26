// src/crypto.rs - VERSI FINAL (Error Kompilasi & Bug Test Diperbaiki)
//
// Perbaikan dari versi sebelumnya:
// 1. validate_plaintext_len dipanggil sebagai FcrTpSession:: di test
// 2. session_key_for_test() ditambahkan
// 3. build_aad() diubah ke pub(crate)
// 4. Test lama test_decrypt_rejects_plaintext_len_overflow dihapus
// 5. Duplikasi use di test dihapus
// 6. protected_len dihitung eksplisit

use rand::RngCore;
use zeroize::Zeroize;
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};

use crate::packet::{FcrTpFrame, HeaderMode, BUCKET_SIZES, TOKEN_SIZE};

// ============================================================
// KONSTANTA
// ============================================================

pub const CURRENT_CRYPTO_VERSION: u8 = 1;
pub const MAX_CHUNKS_PER_MESSAGE: u16 = 64;
pub const CHUNK_METADATA_SIZE: usize = 14;
pub const PLAINTEXT_LEN_SIZE: usize = 2;
pub const NONCE_SIZE: usize = 12;
pub const AEAD_TAG_SIZE: usize = 16;
pub const AAD_SIZE: usize = 16;

pub const DEFAULT_BUCKET_SIZE: usize = 1200;

pub const FRAME_OVERHEAD: usize = TOKEN_SIZE
    + NONCE_SIZE
    + CHUNK_METADATA_SIZE
    + PLAINTEXT_LEN_SIZE
    + AEAD_TAG_SIZE;

// ============================================================
// CHUNK METADATA
// ============================================================

#[derive(Debug, Clone, Copy, Zeroize)]
pub struct ChunkMetadata {
    pub message_id: u64,
    pub version: u8,
    pub chunk_index: u16,
    pub total_chunks: u16,
    pub is_final: bool,
}

impl ChunkMetadata {
    pub fn to_bytes(&self) -> [u8; CHUNK_METADATA_SIZE] {
        let mut buf = [0u8; CHUNK_METADATA_SIZE];
        buf[0..8].copy_from_slice(&self.message_id.to_be_bytes());
        buf[8] = self.version;
        buf[9..11].copy_from_slice(&self.chunk_index.to_be_bytes());
        buf[11..13].copy_from_slice(&self.total_chunks.to_be_bytes());
        buf[13] = if self.is_final { 1 } else { 0 };
        buf
    }

    pub fn from_bytes(buf: &[u8; CHUNK_METADATA_SIZE]) -> Result<Self, &'static str> {
        let message_id = u64::from_be_bytes(buf[0..8].try_into().unwrap());
        let version = buf[8];
        let chunk_index = u16::from_be_bytes(buf[9..11].try_into().unwrap());
        let total_chunks = u16::from_be_bytes(buf[11..13].try_into().unwrap());
        // FIX: tolak byte is_final liar (sebelumnya 2..=255 diam-diam jadi false)
        if buf[13] > 1 {
            return Err("Invalid is_final byte: must be 0 or 1");
        }
        let is_final = buf[13] == 1;

        if total_chunks == 0 || total_chunks > MAX_CHUNKS_PER_MESSAGE {
            return Err("Invalid total_chunks: potential DoS attempt");
        }
        if chunk_index >= total_chunks {
            return Err("Invalid chunk_index: out of bounds");
        }

        Ok(Self {
            message_id,
            version,
            chunk_index,
            total_chunks,
            is_final,
        })
    }
}

// ============================================================
// CRYPTO SESSION
// ============================================================

pub struct FcrTpSession {
    pub session_id: [u8; 8],
    pub crypto_version: u8,
    session_key: [u8; 32],
    current_epoch: u64,
    pub max_chunk_payload_size: usize,
}

impl FcrTpSession {
    pub fn new(session_id: [u8; 8], master_key: &[u8; 32], crypto_version: u8) -> Self {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(master_key);
        hasher.update(session_id);
        let derived = hasher.finalize();
        
        let mut session_key = [0u8; 32];
        session_key.copy_from_slice(&derived);
        
        let max_chunk_payload_size = DEFAULT_BUCKET_SIZE.saturating_sub(FRAME_OVERHEAD);
        
        Self {
            session_id,
            crypto_version,
            session_key,
            current_epoch: 0,
            max_chunk_payload_size,
        }
    }

    pub fn max_payload_for_bucket(bucket_size: usize) -> Result<usize, &'static str> {
        bucket_size
            .checked_sub(FRAME_OVERHEAD)
            .filter(|&n| n > 0)
            .ok_or("Bucket size too small for frame overhead")
    }

    // FIX: zeroize kunci saat session drop (sebelumnya kunci 32 byte tertinggal di memori)
    // Catatan: Zeroize derive di struct tidak cukup karena session_key field privat di-impl manual.
    pub fn advance_epoch(&mut self) {
        self.current_epoch = self.current_epoch.wrapping_add(1);
    }

    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    pub fn set_epoch(&mut self, epoch: u64) {
        self.current_epoch = epoch;
    }

    /// ✅ FIX: pub(crate) agar bisa dipakai di test helper
    pub(crate) fn build_aad(&self) -> [u8; AAD_SIZE] {
        let mut aad = [0u8; AAD_SIZE];
        aad[0..8].copy_from_slice(&self.session_id);
        aad[8..16].copy_from_slice(&self.current_epoch.to_be_bytes());
        aad
    }

    /// ✅ FIX: test-only accessor untuk crafting frame
    #[cfg(test)]
    pub(crate) fn session_key_for_test(&self) -> &[u8; 32] {
        &self.session_key
    }

    pub fn encrypt_to_frames(&self, plaintext: &[u8]) -> Result<Vec<FcrTpFrame>, &'static str> {
        self.encrypt_to_frames_with_bucket(plaintext, DEFAULT_BUCKET_SIZE)
    }

    pub fn encrypt_to_frames_with_bucket(
        &self,
        plaintext: &[u8],
        bucket_size: usize,
    ) -> Result<Vec<FcrTpFrame>, &'static str> {
        if !BUCKET_SIZES.contains(&bucket_size) {
            return Err("Invalid bucket size");
        }

        let chunk_payload_size = Self::max_payload_for_bucket(bucket_size)?;

        let total_chunks = if plaintext.is_empty() {
            1
        } else {
            plaintext.len().div_ceil(chunk_payload_size) as u16
        };
        
        if total_chunks > MAX_CHUNKS_PER_MESSAGE {
            return Err("Plaintext too large for single message batch");
        }

        let cipher = ChaCha20Poly1305::new_from_slice(&self.session_key)
            .map_err(|_| "Failed to initialize cipher")?;

        let mut message_id_bytes = [0u8; 8];
        rand::thread_rng().fill_bytes(&mut message_id_bytes);
        let message_id = u64::from_be_bytes(message_id_bytes);

        let mut frames = Vec::with_capacity(total_chunks as usize);

        let chunks: Vec<&[u8]> = if plaintext.is_empty() {
            vec![&[]]
        } else {
            plaintext.chunks(chunk_payload_size).collect()
        };

        for (index, chunk_data) in chunks.iter().enumerate() {
            let chunk_index = index as u16;
            let is_final = index == (total_chunks - 1) as usize;

            let metadata = ChunkMetadata {
                message_id,
                version: self.crypto_version,
                chunk_index,
                total_chunks,
                is_final,
            };

            let plaintext_len = chunk_data.len();
            
            let padding_len = bucket_size
                .checked_sub(FRAME_OVERHEAD)
                .and_then(|avail| avail.checked_sub(plaintext_len))
                .ok_or("Plaintext too large for bucket size")?;

            let mut protected_payload = Vec::with_capacity(
                CHUNK_METADATA_SIZE + PLAINTEXT_LEN_SIZE + chunk_data.len() + padding_len
            );
            protected_payload.extend_from_slice(&metadata.to_bytes());
            protected_payload.extend_from_slice(&(plaintext_len as u16).to_be_bytes());
            protected_payload.extend_from_slice(chunk_data);
            
            let mut padding = vec![0u8; padding_len];
            rand::thread_rng().fill_bytes(&mut padding);
            protected_payload.extend_from_slice(&padding);

            let mut nonce_bytes = [0u8; NONCE_SIZE];
            rand::thread_rng().fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);

            let aad = self.build_aad();

            let payload = Payload {
                msg: protected_payload.as_ref(),
                aad: &aad,
            };
            let ciphertext_with_tag = cipher.encrypt(nonce, payload)
                .map_err(|_| "Encryption failed")?;

            let ciphertext_len = ciphertext_with_tag.len() - AEAD_TAG_SIZE;
            let ciphertext = ciphertext_with_tag[..ciphertext_len].to_vec();
            let mut tag = [0u8; AEAD_TAG_SIZE];
            tag.copy_from_slice(&ciphertext_with_tag[ciphertext_len..]);

            let mut token = [0u8; 8];
            rand::thread_rng().fill_bytes(&mut token);

            let mut encrypted_blob = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
            encrypted_blob.extend_from_slice(&nonce_bytes);
            encrypted_blob.extend_from_slice(&ciphertext);

            let estimated_size = TOKEN_SIZE + encrypted_blob.len() + AEAD_TAG_SIZE;
            let header_mode = if estimated_size > 512 {
                HeaderMode::Full
            } else {
                HeaderMode::Short
            };

            let frame = FcrTpFrame::new(
                token,
                header_mode,
                encrypted_blob,
                tag,
                Vec::new(),
            );

            frames.push(frame);
        }

        Ok(frames)
    }

    pub fn decrypt_from_frame(
        &self,
        frame: &FcrTpFrame,
    ) -> Result<(ChunkMetadata, Vec<u8>), &'static str> {
        let min_blob_size = NONCE_SIZE + CHUNK_METADATA_SIZE + PLAINTEXT_LEN_SIZE;
        if frame.encrypted_header_blob.len() < min_blob_size {
            return Err("Encrypted blob too short");
        }

        let nonce_bytes = &frame.encrypted_header_blob[0..NONCE_SIZE];
        let nonce = Nonce::from_slice(nonce_bytes);
        let ciphertext = &frame.encrypted_header_blob[NONCE_SIZE..];

        let mut ciphertext_with_tag = ciphertext.to_vec();
        ciphertext_with_tag.extend_from_slice(&frame.aead_auth_tag);

        let cipher = ChaCha20Poly1305::new_from_slice(&self.session_key)
            .map_err(|_| "Failed to initialize cipher")?;

        let aad = self.build_aad();
        let payload = Payload {
            msg: ciphertext_with_tag.as_ref(),
            aad: &aad,
        };
        let decrypted = cipher.decrypt(nonce, payload)
            .map_err(|_| "Decryption failed: invalid tag, corrupted data, or AAD mismatch")?;

        if decrypted.len() < CHUNK_METADATA_SIZE + PLAINTEXT_LEN_SIZE {
            return Err("Decrypted payload too short for metadata and length");
        }

        let mut meta_buf = [0u8; CHUNK_METADATA_SIZE];
        meta_buf.copy_from_slice(&decrypted[0..CHUNK_METADATA_SIZE]);
        let metadata = ChunkMetadata::from_bytes(&meta_buf)?;

        if metadata.version != self.crypto_version {
            return Err("Crypto version mismatch");
        }

        let plaintext_len = u16::from_be_bytes(
            decrypted[CHUNK_METADATA_SIZE..CHUNK_METADATA_SIZE + PLAINTEXT_LEN_SIZE]
                .try_into()
                .unwrap()
        ) as usize;

        // ✅ FIX: validasi via fungsi murni
        Self::validate_plaintext_len(&decrypted, plaintext_len)?;

        let data_start = CHUNK_METADATA_SIZE + PLAINTEXT_LEN_SIZE;
        let plaintext_chunk = decrypted[data_start..data_start + plaintext_len].to_vec();

        Ok((metadata, plaintext_chunk))
    }
    
    /// ✅ FIX: pub(crate) agar bisa dipanggil dari test
    pub(crate) fn validate_plaintext_len(
        decrypted: &[u8],
        declared_len: usize,
    ) -> Result<(), &'static str> {
        let min_len = CHUNK_METADATA_SIZE + PLAINTEXT_LEN_SIZE;
        if decrypted.len() < min_len {
            return Err("Decrypted payload too short for metadata and length");
        }
        
        let data_start = min_len;
        let available_data = decrypted.len().saturating_sub(data_start);
        
        if declared_len > available_data {
            return Err("Declared plaintext_len exceeds available data");
        }
        
        Ok(())
    }
}

impl Drop for FcrTpSession { fn drop(&mut self) { self.session_key.zeroize(); } }

// ============================================================
// TEST SUITE
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // Helper
    // ============================================================

    fn make_session() -> FcrTpSession {
        FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0xAA; 32], CURRENT_CRYPTO_VERSION)
    }

    // ============================================================
    // Test: max_payload_for_bucket
    // ============================================================

    #[test]
    fn test_max_payload_for_each_bucket() {
        assert_eq!(FcrTpSession::max_payload_for_bucket(256).unwrap(), 204);
        assert_eq!(FcrTpSession::max_payload_for_bucket(512).unwrap(), 460);
        assert_eq!(FcrTpSession::max_payload_for_bucket(1200).unwrap(), 1148);
    }

    #[test]
    fn test_max_payload_rejects_too_small_bucket() {
        assert!(FcrTpSession::max_payload_for_bucket(52).is_err());
        assert!(FcrTpSession::max_payload_for_bucket(10).is_err());
        assert!(FcrTpSession::max_payload_for_bucket(0).is_err());
    }

    // ============================================================
    // Test: Roundtrip dasar
    // ============================================================

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let session = make_session();
        let original = b"Rahasia FCR-RSA untuk masa depan";

        let frames = session.encrypt_to_frames(original).expect("Encrypt gagal");
        assert_eq!(frames.len(), 1);

        let (metadata, decrypted) = session.decrypt_from_frame(&frames[0]).expect("Decrypt gagal");
        assert_eq!(decrypted, original.to_vec());
        assert_eq!(metadata.version, CURRENT_CRYPTO_VERSION);
        assert!(metadata.is_final);
    }

    #[test]
    fn test_chunking_large_payload() {
        let session = make_session();
        let large = vec![0xCC; 3000];

        let frames = session.encrypt_to_frames(&large).expect("Encrypt gagal");
        assert_eq!(frames.len(), 3);

        let mut decrypted_all = Vec::new();
        for frame in &frames {
            let (_, chunk) = session.decrypt_from_frame(frame).expect("Decrypt gagal");
            decrypted_all.extend_from_slice(&chunk);
        }
        assert_eq!(decrypted_all, large);
    }

    // ============================================================
    // Test: Bucket 256 boundary
    // ============================================================

    #[test]
    fn test_bucket_256_exact_max_payload() {
        let session = make_session();
        let plaintext = vec![0xAA; 204];

        let frames = session.encrypt_to_frames_with_bucket(&plaintext, 256).expect("Encrypt gagal");
        assert_eq!(frames.len(), 1);

        let wire = frames[0].encode().expect("Encode gagal");
        assert_eq!(wire.len(), 256);

        let received = FcrTpFrame::decode(wire).expect("Decode gagal");
        let (_, decrypted) = session.decrypt_from_frame(&received).expect("Decrypt gagal");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_bucket_256_one_byte_over() {
        let session = make_session();
        let plaintext = vec![0xBB; 205];

        let frames = session.encrypt_to_frames_with_bucket(&plaintext, 256).expect("Encrypt gagal");
        assert_eq!(frames.len(), 2);

        let mut decrypted_all = Vec::new();
        for frame in &frames {
            let wire = frame.encode().expect("Encode gagal");
            assert_eq!(wire.len(), 256);
            let received = FcrTpFrame::decode(wire).expect("Decode gagal");
            let (_, chunk) = session.decrypt_from_frame(&received).expect("Decrypt gagal");
            decrypted_all.extend_from_slice(&chunk);
        }
        assert_eq!(decrypted_all, plaintext);
    }

    #[test]
    fn test_bucket_256_300_bytes() {
        let session = make_session();
        let plaintext = vec![0xCC; 300];

        let frames = session.encrypt_to_frames_with_bucket(&plaintext, 256).expect("Encrypt gagal");
        assert_eq!(frames.len(), 2);

        let mut decrypted_all = Vec::new();
        for frame in &frames {
            let wire = frame.encode().expect("Encode gagal");
            assert_eq!(wire.len(), 256);
            let received = FcrTpFrame::decode(wire).expect("Decode gagal");
            let (_, chunk) = session.decrypt_from_frame(&received).expect("Decrypt gagal");
            decrypted_all.extend_from_slice(&chunk);
        }
        assert_eq!(decrypted_all, plaintext);
    }

    #[test]
    fn test_bucket_512_boundary() {
        let session = make_session();
        let plaintext = vec![0xDD; 460];

        let frames = session.encrypt_to_frames_with_bucket(&plaintext, 512).expect("Encrypt gagal");
        assert_eq!(frames.len(), 1);

        let wire = frames[0].encode().expect("Encode gagal");
        assert_eq!(wire.len(), 512);

        let received = FcrTpFrame::decode(wire).expect("Decode gagal");
        let (_, decrypted) = session.decrypt_from_frame(&received).expect("Decrypt gagal");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_bucket_1200_boundary() {
        let session = make_session();
        let plaintext = vec![0xEE; 1148];

        let frames = session.encrypt_to_frames_with_bucket(&plaintext, 1200).expect("Encrypt gagal");
        assert_eq!(frames.len(), 1);

        let wire = frames[0].encode().expect("Encode gagal");
        assert_eq!(wire.len(), 1200);
    }

    // ============================================================
    // Test: Multi-chunk wire roundtrip
    // ============================================================

    #[test]
    fn test_wire_roundtrip_multi_chunk_large_payload() {
        let session = make_session();
        let large = vec![0xDD; 3000];

        let frames = session.encrypt_to_frames(&large).expect("Encrypt gagal");
        assert_eq!(frames.len(), 3);

        let mut wire_frames = Vec::new();
        for frame in &frames {
            let wire = frame.encode().expect("Encode gagal");
            assert!(BUCKET_SIZES.contains(&wire.len()));
            wire_frames.push(wire);
        }

        let mut decrypted_all = Vec::new();
        for (i, wire) in wire_frames.into_iter().enumerate() {
            let received = FcrTpFrame::decode(wire).expect("Decode gagal");
            let (metadata, chunk) = session.decrypt_from_frame(&received).expect("Decrypt gagal");
            assert_eq!(metadata.chunk_index, i as u16);
            assert_eq!(metadata.total_chunks, 3);
            decrypted_all.extend_from_slice(&chunk);
        }
        assert_eq!(decrypted_all, large);
    }

    #[test]
    fn test_wire_roundtrip_chunk_out_of_order() {
        let session = make_session();
        let large = vec![0xEE; 3000];

        let frames = session.encrypt_to_frames(&large).expect("Encrypt gagal");
        let wire_frames: Vec<_> = frames.iter()
            .map(|f| f.encode().expect("Encode gagal"))
            .collect();

        let mut decrypted_chunks = Vec::new();
        for i in (0..3).rev() {
            let received = FcrTpFrame::decode(wire_frames[i].clone()).expect("Decode gagal");
            let (metadata, chunk) = session.decrypt_from_frame(&received).expect("Decrypt gagal");
            decrypted_chunks.push((metadata.chunk_index, chunk));
        }

        decrypted_chunks.sort_by_key(|(idx, _)| *idx);
        let mut decrypted_all = Vec::new();
        for (_, chunk) in decrypted_chunks {
            decrypted_all.extend_from_slice(&chunk);
        }
        assert_eq!(decrypted_all, large);
    }

    // ============================================================
    // Test: Metadata & security
    // ============================================================

    #[test]
    fn test_metadata_serialization_roundtrip() {
        let original = ChunkMetadata {
            message_id: 0x123456789ABCDEF0,
            version: 2,
            chunk_index: 5,
            total_chunks: 10,
            is_final: false,
        };

        let bytes = original.to_bytes();
        let decoded = ChunkMetadata::from_bytes(&bytes).expect("Deserialize gagal");

        assert_eq!(original.message_id, decoded.message_id);
        assert_eq!(original.version, decoded.version);
        assert_eq!(original.chunk_index, decoded.chunk_index);
        assert_eq!(original.total_chunks, decoded.total_chunks);
        assert_eq!(original.is_final, decoded.is_final);
    }

    #[test]
    fn test_reject_dos_attack_large_total_chunks() {
        let mut malicious = [0u8; CHUNK_METADATA_SIZE];
        malicious[11..13].copy_from_slice(&1000u16.to_be_bytes());
        malicious[9..11].copy_from_slice(&5u16.to_be_bytes());

        assert!(ChunkMetadata::from_bytes(&malicious).is_err());
    }

    #[test]
    fn test_reject_invalid_chunk_index() {
        let mut malicious = [0u8; CHUNK_METADATA_SIZE];
        malicious[11..13].copy_from_slice(&5u16.to_be_bytes());
        malicious[9..11].copy_from_slice(&5u16.to_be_bytes());

        assert!(ChunkMetadata::from_bytes(&malicious).is_err());
    }

    #[test]
    fn test_reject_tampered_tag() {
        let session = make_session();
        let frames = session.encrypt_to_frames(b"Data Rahasia").expect("Encrypt gagal");
        let mut frame = frames[0].clone();
        frame.aead_auth_tag[0] ^= 0xFF;

        assert!(session.decrypt_from_frame(&frame).is_err());
    }

    #[test]
    fn test_reject_wrong_session_key() {
        let session1 = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0xEE; 32], CURRENT_CRYPTO_VERSION);
        let session2 = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0xFF; 32], CURRENT_CRYPTO_VERSION);

        let frames = session1.encrypt_to_frames(b"Data Rahasia").expect("Encrypt gagal");
        assert!(session2.decrypt_from_frame(&frames[0]).is_err());
    }

    #[test]
    fn test_nonce_uniqueness() {
        let session = make_session();

        let mut nonces = Vec::new();
        for _ in 0..10 {
            let frames = session.encrypt_to_frames(b"Data").expect("Encrypt gagal");
            let nonce = &frames[0].encrypted_header_blob[0..NONCE_SIZE];
            nonces.push(nonce.to_vec());
        }

        for i in 0..nonces.len() {
            for j in (i+1)..nonces.len() {
                assert_ne!(nonces[i], nonces[j], "Nonce collision!");
            }
        }
    }

    #[test]
    fn test_reject_replay_across_epochs() {
        let mut session = make_session();

        let frames = session.encrypt_to_frames(b"Data Epoch 0").expect("Encrypt gagal");
        let frame = frames[0].clone();

        assert!(session.decrypt_from_frame(&frame).is_ok());

        session.advance_epoch();
        assert!(session.decrypt_from_frame(&frame).is_err());
    }

    #[test]
    fn test_reject_replay_across_sessions() {
        let session_a = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0x33; 32], CURRENT_CRYPTO_VERSION);
        let session_b = FcrTpSession::new([9, 10, 11, 12, 13, 14, 15, 16], &[0x33; 32], CURRENT_CRYPTO_VERSION);

        let frames = session_a.encrypt_to_frames(b"Data Sesi A").expect("Encrypt gagal");
        assert!(session_a.decrypt_from_frame(&frames[0]).is_ok());
        assert!(session_b.decrypt_from_frame(&frames[0]).is_err());
    }

    #[test]
    fn test_epoch_advancement() {
        let mut session = make_session();
        assert_eq!(session.current_epoch(), 0);
        session.advance_epoch();
        assert_eq!(session.current_epoch(), 1);
        session.set_epoch(100);
        assert_eq!(session.current_epoch(), 100);
    }

    #[test]
    fn test_aad_structure() {
        let session = FcrTpSession::new([0xAA; 8], &[0x66; 32], CURRENT_CRYPTO_VERSION);
        let aad = session.build_aad();
        assert_eq!(&aad[0..8], &[0xAA; 8]);
        assert_eq!(&aad[8..16], &0u64.to_be_bytes());
    }

    // ============================================================
    // Test: Wire roundtrip semua bucket
    // ============================================================

    #[test]
    fn test_wire_roundtrip_all_buckets() {
        let session = make_session();

        for &bucket in &BUCKET_SIZES {
            let payload_size = FcrTpSession::max_payload_for_bucket(bucket).unwrap();
            let plaintext = vec![0xAA; payload_size];

            let frames = session.encrypt_to_frames_with_bucket(&plaintext, bucket)
                .expect("Encrypt gagal");
            assert_eq!(frames.len(), 1, "Bucket {} harus 1 chunk", bucket);

            let wire = frames[0].encode().expect("Encode gagal");
            assert_eq!(wire.len(), bucket);

            let received = FcrTpFrame::decode(wire).expect("Decode gagal");
            let (_, decrypted) = session.decrypt_from_frame(&received).expect("Decrypt gagal");
            assert_eq!(decrypted, plaintext, "Bucket {} roundtrip gagal", bucket);
        }
    }

    #[test]
    fn test_empty_plaintext() {
        let session = make_session();
        let frames = session.encrypt_to_frames(b"").expect("Encrypt gagal");
        assert_eq!(frames.len(), 1);

        let (metadata, decrypted) = session.decrypt_from_frame(&frames[0]).expect("Decrypt gagal");
        assert_eq!(decrypted.len(), 0);
        assert!(metadata.is_final);
    }

    // ============================================================
    // ✅ HELPER: Craft frame dengan plaintext_len yang dimanipulasi
    // ============================================================

    fn craft_frame_with_declared_len(
        session: &FcrTpSession,
        actual_data_len: usize,
        declared_len: usize,
        bucket_size: usize,
    ) -> FcrTpFrame {
        let metadata = ChunkMetadata {
            message_id: 0xDEADBEEF,
            version: CURRENT_CRYPTO_VERSION,
            chunk_index: 0,
            total_chunks: 1,
            is_final: true,
        };
        
        let padding_len = bucket_size
            .checked_sub(FRAME_OVERHEAD)
            .and_then(|a| a.checked_sub(actual_data_len))
            .expect("bucket too small untuk test");
        
        // ✅ FIX: hitung protected_len eksplisit
        let protected_len = CHUNK_METADATA_SIZE
            + PLAINTEXT_LEN_SIZE
            + actual_data_len
            + padding_len;
        let mut protected = Vec::with_capacity(protected_len);
        protected.extend_from_slice(&metadata.to_bytes());
        protected.extend_from_slice(&(declared_len as u16).to_be_bytes());
        protected.extend_from_slice(&vec![0xAA; actual_data_len]);
        protected.extend_from_slice(&vec![0x00; padding_len]);
        
        let cipher = ChaCha20Poly1305::new_from_slice(session.session_key_for_test())
            .expect("cipher init");
        
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let aad = session.build_aad();
        let payload = Payload { msg: &protected, aad: &aad };
        let ciphertext_with_tag = cipher.encrypt(nonce, payload).expect("encrypt");
        
        let ciphertext_len = ciphertext_with_tag.len() - AEAD_TAG_SIZE;
        let ciphertext = ciphertext_with_tag[..ciphertext_len].to_vec();
        let mut tag = [0u8; AEAD_TAG_SIZE];
        tag.copy_from_slice(&ciphertext_with_tag[ciphertext_len..]);
        
        let mut encrypted_blob = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        encrypted_blob.extend_from_slice(&nonce_bytes);
        encrypted_blob.extend_from_slice(&ciphertext);
        
        FcrTpFrame::new(
            [0x11; 8],
            HeaderMode::Short,
            encrypted_blob,
            tag,
            Vec::new(),
        )
    }
    
    // ============================================================
    // ✅ TEST: Validasi plaintext_len (unit test)
    // ============================================================

    #[test]
    fn test_validate_plaintext_len_exact() {
        let decrypted = vec![0u8; CHUNK_METADATA_SIZE + PLAINTEXT_LEN_SIZE + 10];
        assert!(FcrTpSession::validate_plaintext_len(&decrypted, 10).is_ok());
    }
    
    #[test]
    fn test_validate_plaintext_len_over_by_one() {
        let decrypted = vec![0u8; CHUNK_METADATA_SIZE + PLAINTEXT_LEN_SIZE + 10];
        assert!(FcrTpSession::validate_plaintext_len(&decrypted, 11).is_err());
    }
    
    #[test]
    fn test_validate_plaintext_len_over_by_much() {
        let decrypted = vec![0u8; CHUNK_METADATA_SIZE + PLAINTEXT_LEN_SIZE + 10];
        assert!(FcrTpSession::validate_plaintext_len(&decrypted, 1000).is_err());
    }
    
    #[test]
    fn test_validate_plaintext_len_too_short() {
        let too_short = vec![0u8; 5];
        assert!(FcrTpSession::validate_plaintext_len(&too_short, 0).is_err());
    }
    
    #[test]
    fn test_validate_plaintext_len_zero_declared() {
        let decrypted = vec![0u8; CHUNK_METADATA_SIZE + PLAINTEXT_LEN_SIZE];
        assert!(FcrTpSession::validate_plaintext_len(&decrypted, 0).is_ok());
    }
    
    // ============================================================
    // ✅ TEST: Overflow yang sebenarnya (crafted frame)
    // ============================================================
    
    #[test]
    fn test_decrypt_rejects_real_overflow() {
        let session = make_session();
        // ✅ FIX: declared_len = 300 > available_data = 204
        let frame = craft_frame_with_declared_len(&session, 10, 300, 256);
        
        let result = session.decrypt_from_frame(&frame);
        assert!(result.is_err(), "Overflow harus ditolak");
        assert!(
            result.unwrap_err().contains("exceeds available data"),
            "Error harus menyebut 'exceeds available data'"
        );
    }
    
    #[test]
    fn test_decrypt_accepts_exact_boundary() {
        let session = make_session();
        let frame = craft_frame_with_declared_len(&session, 204, 204, 256);
        let result = session.decrypt_from_frame(&frame);
        assert!(result.is_ok(), "Boundary harus lolos: {:?}", result.err());
    }
    
    #[test]
    fn test_decrypt_rejects_one_byte_overflow() {
        let session = make_session();
        let frame = craft_frame_with_declared_len(&session, 204, 205, 256);
        let result = session.decrypt_from_frame(&frame);
        assert!(result.is_err(), "1 byte overflow harus ditolak");
    }
    
    #[test]
    fn test_decrypt_rejects_huge_overflow() {
        let session = make_session();
        let frame = craft_frame_with_declared_len(&session, 10, 65000, 1200);
        let result = session.decrypt_from_frame(&frame);
        assert!(result.is_err(), "Huge overflow harus ditolak");
    }
}
