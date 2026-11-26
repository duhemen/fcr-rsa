//! FCR-TP Crypto Wrapper & Chunking Engine
//! 
//! Padding sekarang di dalam plaintext untuk AEAD, bukan di luar.
//! Format plaintext untuk AEAD: [metadata:14][plaintext_len:2][data:N][padding_acak:P]
//! Receiver baca plaintext_len dari plaintext terdekripsi, ambil hanya data asli.

use rand::RngCore;
use zeroize::Zeroize;
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};

use crate::packet::{FcrTpFrame, HeaderMode, BUCKET_SIZES};

pub const CURRENT_CRYPTO_VERSION: u8 = 1;
pub const MAX_CHUNKS_PER_MESSAGE: u16 = 64;
pub const CHUNK_METADATA_SIZE: usize = 14;
pub const PLAINTEXT_LEN_SIZE: usize = 2;
pub const NONCE_SIZE: usize = 12;
pub const AEAD_TAG_SIZE: usize = 16;
pub const AAD_SIZE: usize = 16;

/// Default bucket size untuk enkripsi (bucket terbesar)
pub const DEFAULT_BUCKET_SIZE: usize = 1200;

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
        hasher.update(&session_id);
        let derived = hasher.finalize();
        
        let mut session_key = [0u8; 32];
        session_key.copy_from_slice(&derived);
        
        let max_chunk_payload_size = DEFAULT_BUCKET_SIZE 
            - crate::packet::TOKEN_SIZE 
            - NONCE_SIZE 
            - AEAD_TAG_SIZE 
            - CHUNK_METADATA_SIZE 
            - PLAINTEXT_LEN_SIZE;
        
        Self {
            session_id,
            crypto_version,
            session_key,
            current_epoch: 0,
            max_chunk_payload_size,
        }
    }

    pub fn advance_epoch(&mut self) {
        self.current_epoch = self.current_epoch.wrapping_add(1);
    }

    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    pub fn set_epoch(&mut self, epoch: u64) {
        self.current_epoch = epoch;
    }

    fn build_aad(&self) -> [u8; AAD_SIZE] {
        let mut aad = [0u8; AAD_SIZE];
        aad[0..8].copy_from_slice(&self.session_id);
        aad[8..16].copy_from_slice(&self.current_epoch.to_be_bytes());
        aad
    }

    /// Enkripsi plaintext menjadi Vec<FcrTpFrame> dengan padding di dalam AEAD.
    pub fn encrypt_to_frames(&self, plaintext: &[u8]) -> Result<Vec<FcrTpFrame>, &'static str> {
        self.encrypt_to_frames_with_bucket(plaintext, DEFAULT_BUCKET_SIZE)
    }

    /// Enkripsi dengan bucket size spesifik.
    pub fn encrypt_to_frames_with_bucket(&self, plaintext: &[u8], bucket_size: usize) -> Result<Vec<FcrTpFrame>, &'static str> {
        if !BUCKET_SIZES.contains(&bucket_size) {
            return Err("Invalid bucket size");
        }

        let cipher = ChaCha20Poly1305::new_from_slice(&self.session_key)
            .map_err(|_| "Failed to initialize cipher")?;

        let mut message_id_bytes = [0u8; 8];
        rand::thread_rng().fill_bytes(&mut message_id_bytes);
        let message_id = u64::from_be_bytes(message_id_bytes);

        let total_chunks = ((plaintext.len() + self.max_chunk_payload_size - 1) / self.max_chunk_payload_size) as u16;
        
        if total_chunks > MAX_CHUNKS_PER_MESSAGE {
            return Err("Plaintext too large for single message batch");
        }

        let mut frames = Vec::with_capacity(total_chunks as usize);

        for (index, chunk_data) in plaintext.chunks(self.max_chunk_payload_size).enumerate() {
            let chunk_index = index as u16;
            let is_final = index == (total_chunks - 1) as usize;

            let metadata = ChunkMetadata {
                message_id,
                version: self.crypto_version,
                chunk_index,
                total_chunks,
                is_final,
            };

            let plaintext_len = chunk_data.len() as u16;
            
            // Hitung padding supaya total wire size = bucket_size
            // wire_size = TOKEN_SIZE + NONCE_SIZE + ciphertext_len + AEAD_TAG_SIZE
            // ciphertext_len = CHUNK_METADATA_SIZE + PLAINTEXT_LEN_SIZE + plaintext_len + padding_len
            // Jadi: bucket_size = 8 + 12 + 14 + 2 + plaintext_len + padding_len + 16
            // padding_len = bucket_size - 52 - plaintext_len
            let overhead = crate::packet::TOKEN_SIZE + NONCE_SIZE + CHUNK_METADATA_SIZE + PLAINTEXT_LEN_SIZE + AEAD_TAG_SIZE;
            let padding_len = bucket_size as isize - overhead as isize - plaintext_len as isize;
            
            if padding_len < 0 {
                return Err("Plaintext too large for bucket size");
            }
            let padding_len = padding_len as usize;

            // Format plaintext untuk AEAD: [metadata:14][plaintext_len:2][data:N][padding:P]
            let mut protected_payload = Vec::with_capacity(CHUNK_METADATA_SIZE + PLAINTEXT_LEN_SIZE + chunk_data.len() + padding_len);
            protected_payload.extend_from_slice(&metadata.to_bytes());
            protected_payload.extend_from_slice(&plaintext_len.to_be_bytes());
            protected_payload.extend_from_slice(chunk_data);
            
            // Padding acak (CSPRNG)
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

            let estimated_size = 8 + encrypted_blob.len() + AEAD_TAG_SIZE;
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

    /// Dekripsi FcrTpFrame menjadi plaintext asli.
    pub fn decrypt_from_frame(&self, frame: &FcrTpFrame) -> Result<(ChunkMetadata, Vec<u8>), &'static str> {
        if frame.encrypted_header_blob.len() < NONCE_SIZE + CHUNK_METADATA_SIZE + PLAINTEXT_LEN_SIZE {
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

        // Baca panjang plaintext asli dari field `len`
        let plaintext_len = u16::from_be_bytes(
            decrypted[CHUNK_METADATA_SIZE..CHUNK_METADATA_SIZE + PLAINTEXT_LEN_SIZE]
                .try_into()
                .unwrap()
        ) as usize;

        if decrypted.len() < CHUNK_METADATA_SIZE + PLAINTEXT_LEN_SIZE + plaintext_len {
            return Err("Decrypted payload shorter than declared length");
        }

        // Extract plaintext chunk (hanya data asli, tanpa padding)
        let plaintext_chunk = decrypted[CHUNK_METADATA_SIZE + PLAINTEXT_LEN_SIZE..CHUNK_METADATA_SIZE + PLAINTEXT_LEN_SIZE + plaintext_len].to_vec();

        Ok((metadata, plaintext_chunk))
    }
}

// === TEST SUITE ===
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let session_id = [1, 2, 3, 4, 5, 6, 7, 8];
        let master_key = [0xAA; 32];
        let session = FcrTpSession::new(session_id, &master_key, CURRENT_CRYPTO_VERSION);
        
        let original_plaintext = b"Rahasia FCR-RSA untuk masa depan";

        let frames = session.encrypt_to_frames(original_plaintext).expect("Encrypt gagal");
        assert_eq!(frames.len(), 1);
        
        let frame = &frames[0];
        
        let (_metadata, decrypted_chunk) = session.decrypt_from_frame(frame).expect("Decrypt gagal");
        
        assert_eq!(decrypted_chunk, original_plaintext.to_vec());
    }

    #[test]
    fn test_chunking_large_payload() {
        let session = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0xBB; 32], CURRENT_CRYPTO_VERSION);
        
        let large_plaintext = vec![0xCC; 3000];
        
        let frames = session.encrypt_to_frames(&large_plaintext).expect("Encrypt gagal");
        assert_eq!(frames.len(), 3);
        
        let mut decrypted_all = Vec::new();
        for frame in &frames {
            let (_metadata, chunk) = session.decrypt_from_frame(frame).expect("Decrypt gagal");
            decrypted_all.extend_from_slice(&chunk);
        }
        
        assert_eq!(decrypted_all, large_plaintext);
    }

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
        let mut malicious_buf = [0u8; CHUNK_METADATA_SIZE];
        malicious_buf[11..13].copy_from_slice(&1000u16.to_be_bytes());
        malicious_buf[9..11].copy_from_slice(&5u16.to_be_bytes());

        let result = ChunkMetadata::from_bytes(&malicious_buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_invalid_chunk_index() {
        let mut malicious_buf = [0u8; CHUNK_METADATA_SIZE];
        malicious_buf[11..13].copy_from_slice(&5u16.to_be_bytes());
        malicious_buf[9..11].copy_from_slice(&5u16.to_be_bytes());

        let result = ChunkMetadata::from_bytes(&malicious_buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_tampered_tag() {
        let session = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0xDD; 32], CURRENT_CRYPTO_VERSION);
        let plaintext = b"Data Rahasia";
        
        let frames = session.encrypt_to_frames(plaintext).expect("Encrypt gagal");
        let mut frame = frames[0].clone();
        
        frame.aead_auth_tag[0] ^= 0xFF;
        
        let result = session.decrypt_from_frame(&frame);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_wrong_session_key() {
        let session1 = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0xEE; 32], CURRENT_CRYPTO_VERSION);
        let session2 = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0xFF; 32], CURRENT_CRYPTO_VERSION);
        
        let plaintext = b"Data Rahasia";
        let frames = session1.encrypt_to_frames(plaintext).expect("Encrypt gagal");
        let frame = &frames[0];
        
        let result = session2.decrypt_from_frame(frame);
        assert!(result.is_err());
    }

    #[test]
    fn test_nonce_uniqueness() {
        let session = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0x11; 32], CURRENT_CRYPTO_VERSION);
        let plaintext = b"Data Rahasia";
        
        let mut nonces = Vec::new();
        for _ in 0..10 {
            let frames = session.encrypt_to_frames(plaintext).expect("Encrypt gagal");
            let frame = &frames[0];
            let nonce = &frame.encrypted_header_blob[0..NONCE_SIZE];
            nonces.push(nonce.to_vec());
        }
        
        for i in 0..nonces.len() {
            for j in (i+1)..nonces.len() {
                assert_ne!(nonces[i], nonces[j], "Nonce collision detected!");
            }
        }
    }

    #[test]
    fn test_reject_replay_across_epochs() {
        let mut session = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0x22; 32], CURRENT_CRYPTO_VERSION);
        
        let plaintext = b"Data Rahasia Epoch 0";
        let frames = session.encrypt_to_frames(plaintext).expect("Encrypt gagal");
        let frame = frames[0].clone();
        
        let result = session.decrypt_from_frame(&frame);
        assert!(result.is_ok());
        
        session.advance_epoch();
        
        let result = session.decrypt_from_frame(&frame);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_replay_across_sessions() {
        let session_a = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0x33; 32], CURRENT_CRYPTO_VERSION);
        let session_b = FcrTpSession::new([9, 10, 11, 12, 13, 14, 15, 16], &[0x33; 32], CURRENT_CRYPTO_VERSION);
        
        let plaintext = b"Data Rahasia Sesi A";
        let frames = session_a.encrypt_to_frames(plaintext).expect("Encrypt gagal");
        let frame = &frames[0];
        
        let result = session_a.decrypt_from_frame(frame);
        assert!(result.is_ok());
        
        let result = session_b.decrypt_from_frame(frame);
        assert!(result.is_err());
    }

    #[test]
    fn test_epoch_advancement() {
        let mut session = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0x55; 32], CURRENT_CRYPTO_VERSION);
        
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

    #[test]
    fn test_wire_roundtrip_encrypt_encode_decode_decrypt() {
        let session = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0x77; 32], CURRENT_CRYPTO_VERSION);
        let plaintext = b"Data yang benar-benar dikirim lewat jaringan";

        let frames = session.encrypt_to_frames(plaintext).expect("Encrypt gagal");
        assert_eq!(frames.len(), 1);
        
        let wire = frames[0].encode().expect("Encode gagal");
        assert!(BUCKET_SIZES.contains(&wire.len()));
        
        let received = FcrTpFrame::decode(wire).expect("Decode gagal");
        
        let (metadata, decrypted) = session.decrypt_from_frame(&received).expect("Decrypt gagal");
        
        assert_eq!(metadata.version, CURRENT_CRYPTO_VERSION);
        assert_eq!(decrypted, plaintext.to_vec());
    }
    // === TEST BARU: Wire Roundtrip Multi-Chunk ===
    #[test]
    fn test_wire_roundtrip_multi_chunk_large_payload() {
        let session = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0x88; 32], CURRENT_CRYPTO_VERSION);
        
        // Payload besar: 3000 bytes → akan jadi 3 chunk
        let large_plaintext = vec![0xDD; 3000];
        
        // Encrypt
        let frames = session.encrypt_to_frames(&large_plaintext).expect("Encrypt gagal");
        assert_eq!(frames.len(), 3, "Harus ada 3 chunk");
        
        // Encode semua chunk ke wire format
        let mut wire_frames = Vec::new();
        for frame in &frames {
            let wire = frame.encode().expect("Encode gagal");
            assert!(BUCKET_SIZES.contains(&wire.len()), "Ukuran wire harus bucket valid");
            wire_frames.push(wire);
        }
        
        // Decode semua chunk dari wire format
        let mut received_frames = Vec::new();
        for wire in wire_frames {
            let received = FcrTpFrame::decode(wire).expect("Decode gagal");
            received_frames.push(received);
        }
        
        // Decrypt semua chunk dan rakit kembali
        let mut decrypted_all = Vec::new();
        for (i, received) in received_frames.iter().enumerate() {
            let (metadata, chunk) = session.decrypt_from_frame(received).expect("Decrypt gagal");
            
            // Verifikasi metadata chunk
            assert_eq!(metadata.version, CURRENT_CRYPTO_VERSION);
            assert_eq!(metadata.chunk_index, i as u16, "Chunk index harus berurutan");
            assert_eq!(metadata.total_chunks, 3, "Total chunks harus 3");
            
            if i == 2 {
                assert_eq!(metadata.is_final, true, "Chunk terakhir harus is_final");
            } else {
                assert_eq!(metadata.is_final, false, "Chunk bukan terakhir harus is_final=false");
            }
            
            decrypted_all.extend_from_slice(&chunk);
        }
        
        // Verifikasi plaintext lengkap
        assert_eq!(decrypted_all.len(), 3000, "Panjang plaintext harus sama");
        assert_eq!(decrypted_all, large_plaintext, "Isi plaintext harus identik");
    }

    #[test]
    fn test_wire_roundtrip_chunk_out_of_order() {
        let session = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0x99; 32], CURRENT_CRYPTO_VERSION);
        
        let large_plaintext = vec![0xEE; 3000];
        
        let frames = session.encrypt_to_frames(&large_plaintext).expect("Encrypt gagal");
        assert_eq!(frames.len(), 3);
        
        // Encode semua chunk
        let mut wire_frames = Vec::new();
        for frame in &frames {
            let wire = frame.encode().expect("Encode gagal");
            wire_frames.push(wire);
        }
        
        // Decode dalam urutan terbalik (chunk 2, 1, 0)
        let mut received_frames = Vec::new();
        for i in (0..3).rev() {
            let received = FcrTpFrame::decode(wire_frames[i].clone()).expect("Decode gagal");
            received_frames.push(received);
        }
        
        // Decrypt dalam urutan terbalik
        let mut decrypted_chunks = Vec::new();
        for received in &received_frames {
            let (metadata, chunk) = session.decrypt_from_frame(received).expect("Decrypt gagal");
            decrypted_chunks.push((metadata.chunk_index, chunk));
        }
        
        // Sort berdasarkan chunk_index
        decrypted_chunks.sort_by_key(|(idx, _)| *idx);
        
        // Rakit plaintext
        let mut decrypted_all = Vec::new();
        for (_, chunk) in decrypted_chunks {
            decrypted_all.extend_from_slice(&chunk);
        }
        
        // Verifikasi plaintext lengkap
        assert_eq!(decrypted_all, large_plaintext, "Plaintext harus identik meskipun chunk datang out-of-order");
    }

    #[test]
    fn test_wire_roundtrip_different_bucket_sizes() {
        let session = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0xAA; 32], CURRENT_CRYPTO_VERSION);
        
        // Test dengan bucket 256 (pesan kecil)
        let small_plaintext = b"Pesan kecil";
        let frames_small = session.encrypt_to_frames_with_bucket(small_plaintext, 256).expect("Encrypt gagal");
        assert_eq!(frames_small.len(), 1);
        
        let wire_small = frames_small[0].encode().expect("Encode gagal");
        assert_eq!(wire_small.len(), 256);
        
        let received_small = FcrTpFrame::decode(wire_small).expect("Decode gagal");
        let (_, decrypted_small) = session.decrypt_from_frame(&received_small).expect("Decrypt gagal");
        assert_eq!(decrypted_small, small_plaintext.to_vec());
        
        // Test dengan bucket 512 (pesan sedang)
        let medium_plaintext = vec![0xBB; 400];
        let frames_medium = session.encrypt_to_frames_with_bucket(&medium_plaintext, 512).expect("Encrypt gagal");
        assert_eq!(frames_medium.len(), 1);
        
        let wire_medium = frames_medium[0].encode().expect("Encode gagal");
        assert_eq!(wire_medium.len(), 512);
        
        let received_medium = FcrTpFrame::decode(wire_medium).expect("Decode gagal");
        let (_, decrypted_medium) = session.decrypt_from_frame(&received_medium).expect("Decrypt gagal");
        assert_eq!(decrypted_medium, medium_plaintext);
        
        // Test dengan bucket 1200 (pesan besar)
        let large_plaintext = vec![0xCC; 1000];
        let frames_large = session.encrypt_to_frames_with_bucket(&large_plaintext, 1200).expect("Encrypt gagal");
        assert_eq!(frames_large.len(), 1);
        
        let wire_large = frames_large[0].encode().expect("Encode gagal");
        assert_eq!(wire_large.len(), 1200);
        
        let received_large = FcrTpFrame::decode(wire_large).expect("Decode gagal");
        let (_, decrypted_large) = session.decrypt_from_frame(&received_large).expect("Decrypt gagal");
        assert_eq!(decrypted_large, large_plaintext);
    }
}