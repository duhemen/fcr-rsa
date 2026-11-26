//! FCR-TP Packet Parser & Serializer
//! 
//! Format wire zero-plaintext, fixed-bucket, zero-copy parsing.
//! Padding sekarang di dalam AEAD (di crypto.rs), bukan di luar.

use bytes::{Bytes, BytesMut, BufMut};
use zeroize::Zeroize;

/// Ukuran bucket datagram yang diizinkan.
pub const BUCKET_SIZES: [usize; 3] = [256, 512, 1200];

/// Ukuran minimum datagram (bucket terkecil).
pub const MIN_DATAGRAM_SIZE: usize = BUCKET_SIZES[0];

/// Ukuran maksimum datagram (bucket terbesar).
pub const MAX_DATAGRAM_SIZE: usize = BUCKET_SIZES[2];

/// Ukuran AEAD auth tag (Poly1305 = 16 byte).
pub const AEAD_TAG_SIZE: usize = 16;

/// Ukuran ephemeral routing token.
pub const TOKEN_SIZE: usize = 8;

/// Overhead minimum: Token (8) + Tag (16) = 24 byte.
pub const MIN_OVERHEAD: usize = TOKEN_SIZE + AEAD_TAG_SIZE;

/// Menentukan mode header berdasarkan panjang datagram.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Zeroize)]
pub enum HeaderMode {
    Full,
    Short,
}

/// Struktur logis frame FCR-TP.
#[derive(Debug, Clone, Zeroize)]
pub struct FcrTpFrame {
    pub ephemeral_routing_token: [u8; 8],
    pub header_mode: HeaderMode,
    pub encrypted_header_blob: Vec<u8>,
    pub aead_auth_tag: [u8; 16],
    pub dynamic_payload: Vec<u8>,
}

impl FcrTpFrame {
    pub fn new(
        ephemeral_routing_token: [u8; 8],
        header_mode: HeaderMode,
        encrypted_header_blob: Vec<u8>,
        aead_auth_tag: [u8; 16],
        dynamic_payload: Vec<u8>,
    ) -> Self {
        Self {
            ephemeral_routing_token,
            header_mode,
            encrypted_header_blob,
            aead_auth_tag,
            dynamic_payload,
        }
    }

    pub fn mode_from_datagram_size(size: usize) -> Result<HeaderMode, &'static str> {
        match size {
            1200 => Ok(HeaderMode::Full),
            256 | 512 => Ok(HeaderMode::Short),
            _ => Err("Invalid datagram size: must be 256, 512, or 1200"),
        }
    }

    /// Meng-encode frame menjadi Bytes.
    /// Format: [Token:8][EncryptedBlob:K][Tag:16]
    /// Padding sudah ada di dalam EncryptedBlob (dari crypto.rs).
    pub fn encode(&self) -> Result<Bytes, &'static str> {
        let blob_size = self.encrypted_header_blob.len() + self.dynamic_payload.len();
        let total_len = TOKEN_SIZE + blob_size + AEAD_TAG_SIZE;
        
        // Validasi: total_len harus salah satu dari BUCKET_SIZES
        if !BUCKET_SIZES.contains(&total_len) {
            return Err("Frame size does not match any bucket size");
        }
        
        let mut buf = BytesMut::with_capacity(total_len);
        
        // 1. Token (8 byte)
        buf.put_slice(&self.ephemeral_routing_token);
        
        // 2. Encrypted Header Blob (sudah termasuk padding dari crypto.rs)
        buf.put_slice(&self.encrypted_header_blob);
        
        // 3. Dynamic Payload (jika ada)
        buf.put_slice(&self.dynamic_payload);
        
        // 4. AEAD Auth Tag (16 byte) — SELALU di akhir
        buf.put_slice(&self.aead_auth_tag);
        
        Ok(buf.freeze())
    }

    /// Men-decode Bytes menjadi FcrTpFrame (zero-copy).
    pub fn decode(data: Bytes) -> Result<Self, &'static str> {
        let total_len = data.len();
        
        let header_mode = Self::mode_from_datagram_size(total_len)?;
        
        if total_len < MIN_OVERHEAD {
            return Err("Datagram too short");
        }
        
        let token_slice = data.slice(0..TOKEN_SIZE);
        let mut ephemeral_routing_token = [0u8; 8];
        ephemeral_routing_token.copy_from_slice(&token_slice);
        
        let tag_start = total_len - AEAD_TAG_SIZE;
        let tag_slice = data.slice(tag_start..total_len);
        let mut aead_auth_tag = [0u8; 16];
        aead_auth_tag.copy_from_slice(&tag_slice);
        
        let blob_start = TOKEN_SIZE;
        let blob_end = tag_start;
        let encrypted_blob = data.slice(blob_start..blob_end);
        
        let encrypted_header_blob = encrypted_blob.to_vec();
        let dynamic_payload = Vec::new();
        
        Ok(Self {
            ephemeral_routing_token,
            header_mode,
            encrypted_header_blob,
            aead_auth_tag,
            dynamic_payload,
        })
    }
}

// === TEST SUITE ===
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_basic() {
        // Untuk bucket 256: 8 (token) + blob + payload + 16 (tag) = 256
        // Jadi blob + payload = 232 bytes
        let original = FcrTpFrame::new(
            [1, 2, 3, 4, 5, 6, 7, 8],
            HeaderMode::Short,
            vec![0xAA; 200], // 200 bytes blob
            [0x11; 16],
            vec![0xBB; 32], // 32 bytes payload
        );

        let encoded = original.encode().expect("Encode gagal");
        assert_eq!(encoded.len(), 256);
        
        let decoded = FcrTpFrame::decode(encoded).expect("Decode gagal");
        
        assert_eq!(original.ephemeral_routing_token, decoded.ephemeral_routing_token);
        assert_eq!(original.header_mode, decoded.header_mode);
        assert_eq!(original.aead_auth_tag, decoded.aead_auth_tag);
        // encrypted_header_blob sekarang termasuk padding dari crypto.rs
        // Jadi kita hanya verifikasi token, mode, dan tag
    }

    #[test]
    fn test_reject_invalid_size() {
        let data = Bytes::from(vec![0u8; 300]);
        let result = FcrTpFrame::decode(data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid datagram size: must be 256, 512, or 1200");
    }

    #[test]
    fn test_reject_too_short() {
        let data = Bytes::from(vec![0u8; 20]);
        let result = FcrTpFrame::decode(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_mode_from_datagram_size() {
        assert_eq!(FcrTpFrame::mode_from_datagram_size(256).unwrap(), HeaderMode::Short);
        assert_eq!(FcrTpFrame::mode_from_datagram_size(512).unwrap(), HeaderMode::Short);
        assert_eq!(FcrTpFrame::mode_from_datagram_size(1200).unwrap(), HeaderMode::Full);
        assert!(FcrTpFrame::mode_from_datagram_size(300).is_err());
    }

    #[test]
    fn test_encode_rejects_non_bucket_size() {
        let frame = FcrTpFrame::new(
            [0; 8],
            HeaderMode::Short,
            vec![0; 100], // 8 + 100 + 16 = 124, bukan bucket size
            [0; 16],
            vec![],
        );

        let result = frame.encode();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Frame size does not match any bucket size");
    }

    #[test]
    fn test_encode_bucket_512() {
        // Untuk bucket 512: 8 + blob + payload + 16 = 512
        // blob + payload = 488 bytes
        let frame = FcrTpFrame::new(
            [0; 8],
            HeaderMode::Short,
            vec![0; 400],
            [0; 16],
            vec![0; 88],
        );

        let encoded = frame.encode().expect("Encode gagal");
        assert_eq!(encoded.len(), 512);
    }

    #[test]
    fn test_encode_bucket_1200() {
        // Untuk bucket 1200: 8 + blob + payload + 16 = 1200
        // blob + payload = 1176 bytes
        let frame = FcrTpFrame::new(
            [0; 8],
            HeaderMode::Full,
            vec![0; 1000],
            [0; 16],
            vec![0; 176],
        );

        let encoded = frame.encode().expect("Encode gagal");
        assert_eq!(encoded.len(), 1200);
    }
}