//! FCR-TP Packet Parser & Serializer
//! 
//! Format wire zero-plaintext, fixed-bucket padding, zero-copy parsing.
//! Semua metadata terenkripsi di dalam blob, tidak ada field plaintext yang bisa di-fingerprint.

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
    /// Datagram 1200 byte → Handshake atau Risk Score tinggi.
    Full,
    /// Datagram 256 atau 512 byte → Data stream normal atau Survival Lite.
    Short,
}

/// Struktur logis frame FCR-TP (sebelum enkripsi/padding).
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

    pub fn bucket_for_size(payload_size: usize) -> Result<usize, &'static str> {
        for &bucket in &BUCKET_SIZES {
            if payload_size + MIN_OVERHEAD <= bucket {
                return Ok(bucket);
            }
        }
        Err("Payload too large for maximum bucket size")
    }

    /// Meng-encode frame menjadi Bytes dengan padding ke bucket size.
    /// Format: [Token:8][Blob+Payload:K][Padding:P][Tag:16]
    /// Tag SELALU di akhir buffer.
    pub fn encode(&self) -> Result<Bytes, &'static str> {
        let blob_size = self.encrypted_header_blob.len() + self.dynamic_payload.len();
        let total_unpadded = TOKEN_SIZE + blob_size + AEAD_TAG_SIZE;
        
        let bucket_size = Self::bucket_for_size(blob_size)?;
        let padding_size = bucket_size - total_unpadded;
        
        let mut buf = BytesMut::with_capacity(bucket_size);
        
        // 1. Token (8 byte)
        buf.put_slice(&self.ephemeral_routing_token);
        
        // 2. Encrypted Header Blob (N byte)
        buf.put_slice(&self.encrypted_header_blob);
        
        // 3. Dynamic Payload (M byte)
        buf.put_slice(&self.dynamic_payload);
        
        // 4. Padding (isi dengan zero, akan diganti random di crypto layer)
        buf.put_slice(&vec![0u8; padding_size]);
        
        // 5. AEAD Auth Tag (16 byte) — SELALU di akhir
        buf.put_slice(&self.aead_auth_tag);
        
        Ok(buf.freeze())
    }

    /// Men-decode Bytes menjadi FcrTpFrame (zero-copy).
    /// Format: [Token:8][EncryptedData:N-24][Tag:16]
    pub fn decode(data: Bytes) -> Result<Self, &'static str> {
        let total_len = data.len();
        
        // 1. Validasi ukuran bucket
        let header_mode = Self::mode_from_datagram_size(total_len)?;
        
        // 2. Validasi panjang minimum
        if total_len < MIN_OVERHEAD {
            return Err("Datagram too short");
        }
        
        // 3. Extract token (zero-copy)
        let token_slice = data.slice(0..TOKEN_SIZE);
        let mut ephemeral_routing_token = [0u8; 8];
        ephemeral_routing_token.copy_from_slice(&token_slice);
        
        // 4. Extract tag (16 byte terakhir)
        let tag_start = total_len - AEAD_TAG_SIZE;
        let tag_slice = data.slice(tag_start..total_len);
        let mut aead_auth_tag = [0u8; 16];
        aead_auth_tag.copy_from_slice(&tag_slice);
        
        // 5. Extract encrypted data (antara token dan tag, termasuk padding)
        let blob_start = TOKEN_SIZE;
        let blob_end = tag_start;
        let encrypted_blob = data.slice(blob_start..blob_end);
        
        // Untuk sekarang, seluruh encrypted_blob dianggap sebagai header
        // Pemisahan header vs payload akan dilakukan di crypto layer setelah dekripsi
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
        let original = FcrTpFrame::new(
            [1, 2, 3, 4, 5, 6, 7, 8],
            HeaderMode::Short,
            vec![0xAA, 0xBB, 0xCC],
            [0x11; 16],
            vec![1, 2, 3, 4, 5],
        );

        let encoded = original.encode().expect("Encode gagal");
        assert_eq!(encoded.len(), 256);
        
        let decoded = FcrTpFrame::decode(encoded).expect("Decode gagal");
        
        assert_eq!(original.ephemeral_routing_token, decoded.ephemeral_routing_token);
        assert_eq!(original.header_mode, decoded.header_mode);
        assert_eq!(original.aead_auth_tag, decoded.aead_auth_tag);
        // dynamic_payload tidak dibandingkan karena decode tidak mengisinya
    }

    #[test]
    fn test_padding_to_512() {
        let frame = FcrTpFrame::new(
            [0; 8],
            HeaderMode::Short,
            vec![0; 300], // 300 byte blob → total 324 byte → tidak muat di 256, harus 512
            [0; 16],
            vec![],
        );

        let encoded = frame.encode().expect("Encode gagal");
        assert_eq!(encoded.len(), 512);
    }

    #[test]
    fn test_padding_to_1200() {
        let frame = FcrTpFrame::new(
            [0; 8],
            HeaderMode::Full,
            vec![0; 1000],
            [0; 16],
            vec![],
        );

        let encoded = frame.encode().expect("Encode gagal");
        assert_eq!(encoded.len(), 1200);
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
    fn test_payload_too_large() {
        let frame = FcrTpFrame::new(
            [0; 8],
            HeaderMode::Full,
            vec![0; 2000],
            [0; 16],
            vec![],
        );

        let result = frame.encode();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Payload too large for maximum bucket size");
    }

    #[test]
    fn test_mode_from_datagram_size() {
        assert_eq!(FcrTpFrame::mode_from_datagram_size(256).unwrap(), HeaderMode::Short);
        assert_eq!(FcrTpFrame::mode_from_datagram_size(512).unwrap(), HeaderMode::Short);
        assert_eq!(FcrTpFrame::mode_from_datagram_size(1200).unwrap(), HeaderMode::Full);
        assert!(FcrTpFrame::mode_from_datagram_size(300).is_err());
    }

    #[test]
    fn test_zero_copy_encode() {
        let frame = FcrTpFrame::new(
            [1, 2, 3, 4, 5, 6, 7, 8],
            HeaderMode::Short,
            vec![0xAA, 0xBB],
            [0x11; 16],
            vec![1, 2, 3],
        );

        let encoded = frame.encode().expect("Encode gagal");
        
        // Token di posisi awal
        assert_eq!(&encoded[0..8], &[1, 2, 3, 4, 5, 6, 7, 8]);
        
        // Blob
        assert_eq!(&encoded[8..10], &[0xAA, 0xBB]);
        
        // Payload
        assert_eq!(&encoded[10..13], &[1, 2, 3]);
        
        // Padding (zero untuk sekarang)
        assert_eq!(&encoded[13..240], &vec![0u8; 227]);
        
        // Tag SELALU di akhir [240..256]
        assert_eq!(&encoded[240..256], &[0x11; 16]);
    }
}