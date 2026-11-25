//! FCR-TP Crypto Wrapper & Chunking Engine
//! 
//! Menjembatani FCR-RSA dengan FCR-TP. Mendukung chunking dinamis untuk payload besar 
//! (misal: U4096 atau Post-Quantum KEM) sambil mempertahankan ukuran paket yang ramah jaringan.
//! 
//! IMPLEMENTASI KRIPTOGRAFI NYATA:
//! - ChaCha20-Poly1305 untuk AEAD encryption
//! - Associated Data (AAD) untuk mengikat frame ke session_id + epoch + chunk_index
//! - CSPRNG untuk padding acak dan nonce generation
//! - Session key derivation dari master key

use rand::RngCore;
use zeroize::Zeroize;
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};

use crate::packet::{FcrTpFrame, HeaderMode};

/// Versi kriptografi saat ini. 
pub const CURRENT_CRYPTO_VERSION: u8 = 1;

/// Batas maksimum chunk per pesan untuk mencegah serangan Memory Exhaustion (DoS).
pub const MAX_CHUNKS_PER_MESSAGE: u16 = 64;

/// Ukuran metadata chunking di dalam payload terenkripsi (14 bytes).
pub const CHUNK_METADATA_SIZE: usize = 14;

/// Ukuran nonce ChaCha20-Poly1305 (96 bits = 12 bytes)
pub const NONCE_SIZE: usize = 12;

/// Ukuran AEAD tag (Poly1305 = 16 bytes)
pub const AEAD_TAG_SIZE: usize = 16;

/// Ukuran AAD (Associated Data): session_id (8) + epoch (8) + chunk_index (2) = 18 bytes
pub const AAD_SIZE: usize = 18;

/// Metadata internal yang disisipkan ke dalam setiap chunk sebelum enkripsi.
#[derive(Debug, Clone, Copy, Zeroize)]
pub struct ChunkMetadata {
    pub message_id: u64,
    pub version: u8,
    pub chunk_index: u16,
    pub total_chunks: u16,
    pub is_final: bool,
}

impl ChunkMetadata {
    /// Serialize ke byte array (constant-time friendly)
    pub fn to_bytes(&self) -> [u8; CHUNK_METADATA_SIZE] {
        let mut buf = [0u8; CHUNK_METADATA_SIZE];
        buf[0..8].copy_from_slice(&self.message_id.to_be_bytes());
        buf[8] = self.version;
        buf[9..11].copy_from_slice(&self.chunk_index.to_be_bytes());
        buf[11..13].copy_from_slice(&self.total_chunks.to_be_bytes());
        buf[13] = if self.is_final { 1 } else { 0 };
        buf
    }

    /// Deserialize dari byte array (constant-time friendly)
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

/// Sesi FCR-TP yang terikat dengan kunci sesi ChaCha20-Poly1305.
pub struct FcrTpSession {
    pub session_id: [u8; 8],
    pub crypto_version: u8,
    /// Kunci sesi 256-bit untuk ChaCha20-Poly1305
    session_key: [u8; 32],
    /// Epoch saat ini — di-advance saat port hopping terjadi
    /// Setiap frame diikat ke epoch melalui AAD, mencegah replay lintas epoch
    current_epoch: u64,
    /// Ukuran maksimum payload mentah per chunk (sebelum padding FCR-TP).
    pub max_chunk_payload_size: usize,
}

impl FcrTpSession {
    /// Buat sesi baru dengan master key.
    pub fn new(session_id: [u8; 8], master_key: &[u8; 32], crypto_version: u8) -> Self {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(master_key);
        hasher.update(&session_id);
        let derived = hasher.finalize();
        
        let mut session_key = [0u8; 32];
        session_key.copy_from_slice(&derived);
        
        let max_chunk_payload_size = 1200 - crate::packet::TOKEN_SIZE - NONCE_SIZE - AEAD_TAG_SIZE - CHUNK_METADATA_SIZE;
        
        Self {
            session_id,
            crypto_version,
            session_key,
            current_epoch: 0,
            max_chunk_payload_size,
        }
    }

    /// Advance epoch (dipanggil saat port hopping terjadi).
    /// Frame yang dienkripsi setelah ini akan terikat ke epoch baru.
    pub fn advance_epoch(&mut self) {
        self.current_epoch = self.current_epoch.wrapping_add(1);
    }

    /// Dapatkan epoch saat ini
    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    /// Set epoch secara eksplisit (untuk sinkronisasi dengan defense engine)
    pub fn set_epoch(&mut self, epoch: u64) {
        self.current_epoch = epoch;
    }

    /// Bangun AAD (Associated Data) untuk frame saat ini.
    /// AAD mengikat frame ke session_id + epoch + chunk_index.
    /// 
    /// Format: [session_id:8][epoch:8][chunk_index:2] = 18 bytes
    /// 
    /// Manfaat:
    /// - Mencegah replay lintas sesi (session_id berbeda → AAD berbeda → dekripsi gagal)
    /// - Mencegah replay lintas epoch (epoch berbeda → AAD berbeda → dekripsi gagal)
    /// - Mencegah chunk mixing (chunk_index berbeda → AAD berbeda → dekripsi gagal)
    fn build_aad(&self, chunk_index: u16) -> [u8; AAD_SIZE] {
        let mut aad = [0u8; AAD_SIZE];
        aad[0..8].copy_from_slice(&self.session_id);
        aad[8..16].copy_from_slice(&self.current_epoch.to_be_bytes());
        aad[16..18].copy_from_slice(&chunk_index.to_be_bytes());
        aad
    }

    /// Enkripsi plaintext menjadi Vec<FcrTpFrame> dengan AEAD + AAD.
    pub fn encrypt_to_frames(&self, plaintext: &[u8]) -> Result<Vec<FcrTpFrame>, &'static str> {
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

            let mut protected_payload = Vec::with_capacity(CHUNK_METADATA_SIZE + chunk_data.len());
            protected_payload.extend_from_slice(&metadata.to_bytes());
            protected_payload.extend_from_slice(chunk_data);

            let mut nonce_bytes = [0u8; NONCE_SIZE];
            rand::thread_rng().fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);

            // PENTING: Bangun AAD untuk frame ini
            let aad = self.build_aad(chunk_index);

            // Enkripsi dengan AAD (Associated Data)
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
    /// 
    /// PENTING: Dekripsi akan GAGAL jika:
    /// - Tag tidak valid (data corrupted)
    /// - Session ID di AAD tidak cocok (replay dari sesi lain)
    /// - Epoch di AAD tidak cocok (replay dari epoch lain)
    /// - Chunk index di AAD tidak cocok (chunk mixing)
    pub fn decrypt_from_frame(&self, frame: &FcrTpFrame, expected_chunk_index: u16) -> Result<(ChunkMetadata, Vec<u8>), &'static str> {
        if frame.encrypted_header_blob.len() < NONCE_SIZE + CHUNK_METADATA_SIZE {
            return Err("Encrypted blob too short");
        }

        let nonce_bytes = &frame.encrypted_header_blob[0..NONCE_SIZE];
        let nonce = Nonce::from_slice(nonce_bytes);

        let ciphertext = &frame.encrypted_header_blob[NONCE_SIZE..];

        let mut ciphertext_with_tag = ciphertext.to_vec();
        ciphertext_with_tag.extend_from_slice(&frame.aead_auth_tag);

        let cipher = ChaCha20Poly1305::new_from_slice(&self.session_key)
            .map_err(|_| "Failed to initialize cipher")?;

        // PENTING: Bangun AAD dengan expected_chunk_index
        let aad = self.build_aad(expected_chunk_index);

        // Dekripsi dengan AAD — akan gagal jika AAD tidak cocok
        let payload = Payload {
            msg: ciphertext_with_tag.as_ref(),
            aad: &aad,
        };
        let decrypted = cipher.decrypt(nonce, payload)
            .map_err(|_| "Decryption failed: invalid tag, corrupted data, or AAD mismatch")?;

        if decrypted.len() < CHUNK_METADATA_SIZE {
            return Err("Decrypted payload too short for metadata");
        }

        let mut meta_buf = [0u8; CHUNK_METADATA_SIZE];
        meta_buf.copy_from_slice(&decrypted[0..CHUNK_METADATA_SIZE]);
        let metadata = ChunkMetadata::from_bytes(&meta_buf)?;

        if metadata.version != self.crypto_version {
            return Err("Crypto version mismatch");
        }

        // Verifikasi chunk_index konsisten dengan yang diharapkan
        if metadata.chunk_index != expected_chunk_index {
            return Err("Chunk index mismatch: potential chunk mixing attack");
        }

        let plaintext_chunk = decrypted[CHUNK_METADATA_SIZE..].to_vec();

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
        
        // Dekripsi dengan chunk_index = 0 (sesuai)
        let (metadata, decrypted_chunk) = session.decrypt_from_frame(frame, 0).expect("Decrypt gagal");
        
        assert_eq!(metadata.version, CURRENT_CRYPTO_VERSION);
        assert_eq!(metadata.chunk_index, 0);
        assert_eq!(metadata.total_chunks, 1);
        assert_eq!(metadata.is_final, true);
        assert_eq!(decrypted_chunk, original_plaintext.to_vec());
    }

    #[test]
    fn test_chunking_large_payload() {
        let session = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0xBB; 32], CURRENT_CRYPTO_VERSION);
        
        let large_plaintext = vec![0xCC; 3000];
        
        let frames = session.encrypt_to_frames(&large_plaintext).expect("Encrypt gagal");
        assert_eq!(frames.len(), 3);
        
        // Dekripsi semua chunk dengan chunk_index yang benar
        let mut decrypted_all = Vec::new();
        for (i, frame) in frames.iter().enumerate() {
            let (metadata, chunk) = session.decrypt_from_frame(frame, i as u16).expect("Decrypt gagal");
            assert_eq!(metadata.chunk_index, i as u16);
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
        assert_eq!(result.unwrap_err(), "Invalid total_chunks: potential DoS attempt");
    }

    #[test]
    fn test_reject_invalid_chunk_index() {
        let mut malicious_buf = [0u8; CHUNK_METADATA_SIZE];
        malicious_buf[11..13].copy_from_slice(&5u16.to_be_bytes());
        malicious_buf[9..11].copy_from_slice(&5u16.to_be_bytes());

        let result = ChunkMetadata::from_bytes(&malicious_buf);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid chunk_index: out of bounds");
    }

    #[test]
    fn test_reject_tampered_tag() {
        let session = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0xDD; 32], CURRENT_CRYPTO_VERSION);
        let plaintext = b"Data Rahasia";
        
        let frames = session.encrypt_to_frames(plaintext).expect("Encrypt gagal");
        let mut frame = frames[0].clone();
        
        frame.aead_auth_tag[0] ^= 0xFF;
        
        let result = session.decrypt_from_frame(&frame, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_wrong_session_key() {
        let session1 = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0xEE; 32], CURRENT_CRYPTO_VERSION);
        let session2 = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0xFF; 32], CURRENT_CRYPTO_VERSION);
        
        let plaintext = b"Data Rahasia";
        let frames = session1.encrypt_to_frames(plaintext).expect("Encrypt gagal");
        let frame = &frames[0];
        
        let result = session2.decrypt_from_frame(frame, 0);
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

    // === TEST BARU: Verifikasi AAD mencegah replay ===

    #[test]
    fn test_reject_replay_across_epochs() {
        // Skenario: Frame dari epoch 5 tidak bisa di-replay di epoch 6
        let mut session = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0x22; 32], CURRENT_CRYPTO_VERSION);
        
        assert_eq!(session.current_epoch(), 0);
        
        // Enkripsi di epoch 0
        let plaintext = b"Data Rahasia Epoch 0";
        let frames = session.encrypt_to_frames(plaintext).expect("Encrypt gagal");
        let frame = frames[0].clone();
        
        // Dekripsi di epoch 0 harus berhasil
        let result = session.decrypt_from_frame(&frame, 0);
        assert!(result.is_ok(), "Decrypt di epoch yang sama harus berhasil");
        
        // Advance ke epoch 1
        session.advance_epoch();
        assert_eq!(session.current_epoch(), 1);
        
        // Dekripsi frame lama di epoch 1 harus GAGAL (AAD mismatch)
        let result = session.decrypt_from_frame(&frame, 0);
        assert!(result.is_err(), "Decrypt di epoch berbeda harus gagal (AAD mismatch)");
    }

    #[test]
    fn test_reject_replay_across_sessions() {
        // Skenario: Frame dari Sesi A tidak bisa dipakai di Sesi B
        let session_a = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0x33; 32], CURRENT_CRYPTO_VERSION);
        let session_b = FcrTpSession::new([9, 10, 11, 12, 13, 14, 15, 16], &[0x33; 32], CURRENT_CRYPTO_VERSION);
        
        let plaintext = b"Data Rahasia Sesi A";
        let frames = session_a.encrypt_to_frames(plaintext).expect("Encrypt gagal");
        let frame = &frames[0];
        
        // Dekripsi dengan sesi A harus berhasil
        let result = session_a.decrypt_from_frame(frame, 0);
        assert!(result.is_ok(), "Decrypt dengan sesi yang sama harus berhasil");
        
        // Dekripsi dengan sesi B harus GAGAL (session_id di AAD berbeda)
        let result = session_b.decrypt_from_frame(frame, 0);
        assert!(result.is_err(), "Decrypt dengan sesi berbeda harus gagal (AAD mismatch)");
    }

    #[test]
    fn test_reject_chunk_mixing() {
        // Skenario: Chunk dari pesan lain tidak bisa disisipkan
        let session = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0x44; 32], CURRENT_CRYPTO_VERSION);
        
        let plaintext = vec![0xAA; 3000]; // Akan jadi 3 chunk
        let frames = session.encrypt_to_frames(&plaintext).expect("Encrypt gagal");
        assert_eq!(frames.len(), 3);
        
        // Frame 0 dienkripsi dengan chunk_index=0, AAD-nya mengandung chunk_index=0
        let frame_0 = &frames[0];
        
        // Coba dekripsi frame 0 dengan expected_chunk_index=1 (mismatch!)
        let result = session.decrypt_from_frame(frame_0, 1);
        assert!(result.is_err(), "Chunk mixing harus ditolak (AAD mismatch)");
        
        // Dekripsi dengan chunk_index yang benar harus berhasil
        let result = session.decrypt_from_frame(frame_0, 0);
        assert!(result.is_ok(), "Decrypt dengan chunk_index yang benar harus berhasil");
    }

    #[test]
    fn test_epoch_advancement() {
        let mut session = FcrTpSession::new([1, 2, 3, 4, 5, 6, 7, 8], &[0x55; 32], CURRENT_CRYPTO_VERSION);
        
        assert_eq!(session.current_epoch(), 0);
        
        session.advance_epoch();
        assert_eq!(session.current_epoch(), 1);
        
        session.advance_epoch();
        assert_eq!(session.current_epoch(), 2);
        
        session.set_epoch(100);
        assert_eq!(session.current_epoch(), 100);
    }

    #[test]
    fn test_aad_structure() {
        let session = FcrTpSession::new([0xAA; 8], &[0x66; 32], CURRENT_CRYPTO_VERSION);
        
        let aad = session.build_aad(5);
        
        // Verifikasi struktur AAD: [session_id:8][epoch:8][chunk_index:2]
        assert_eq!(&aad[0..8], &[0xAA; 8]); // session_id
        assert_eq!(&aad[8..16], &0u64.to_be_bytes()); // epoch = 0
        assert_eq!(&aad[16..18], &5u16.to_be_bytes()); // chunk_index = 5
    }
}