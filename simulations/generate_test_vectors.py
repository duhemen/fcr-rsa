#!/usr/bin/env python3
"""
FCR-RSA v0.6.0 - Cross-Language Test Vector Generator (Python Oracle)
Script ini mereplikasi logika Rust v0.6.0 untuk menghasilkan test vectors JSON.
"""

import hmac
import hashlib
import json
import os

# Konstanta Q 2048-bit (RFC 3526 MODP Group 14)
Q_HEX = (
    "FFFFFFFFFFFFFFFFC90FDAA22168C234C4C6628B80DC1CD129024E088A67CC74"
    "020BBEA63B139B22514A08798E3404DDEF9519B3CD3A431B302B0A6DF25F1437"
    "4FE1356D6D51C245E485B576625E7EC6F44C42E9A637ED6B0BFF5CB6F406B7ED"
    "EE386BFB5A899FA5AE9F24117C4B1FE649286651ECE45B3DC2007CB8A163BF05"
    "98DA48361C55D39A69163FA8FD24CF5F83655D23DCA3AD961C62F356208552BB"
    "9ED529077096966D670C354E4ABC9804F1746C08CA18217C32905E462E36CE3B"
    "E39E772C180E86039B2783A2EC07A28FB5C55DF06F4C52C9DE2BCBF695581718"
    "3995497CEA956AE515D2261898FA051015728E5A8AACAA68FFFFFFFFFFFFFFFF"
)
Q = int(Q_HEX, 16)

def hkdf_extract(salt: bytes, ikm: bytes) -> bytes:
    """HKDF-Extract (RFC 5869) menggunakan SHA-256."""
    if not salt:
        salt = b'\x00' * 32 # Hash length for SHA-256
    return hmac.new(salt, ikm, hashlib.sha256).digest()

def hkdf_expand(prk: bytes, info: bytes, length: int) -> bytes:
    """HKDF-Expand (RFC 5869) menggunakan SHA-256."""
    t = b""
    okm = b""
    for i in range(1, (length // 32) + 2):
        t = hmac.new(prk, t + info + bytes([i]), hashlib.sha256).digest()
        okm += t
    return okm[:length]

def generate_key_pair(master_seed: bytes):
    """Replikasi generate_key_pair dari Rust."""
    prk = hkdf_extract(None, master_seed)
    e_vec_bytes = hkdf_expand(prk, b"fcr-rsa-e-vec-v1", 256)
    e_vec = int.from_bytes(e_vec_bytes, 'big') % Q
    
    # Modular inverse menggunakan Fermat's Little Theorem (karena Q prima): a^-1 = a^(Q-2) mod Q
    e_vec_inv = pow(e_vec, Q - 2, Q)
    return e_vec, e_vec_inv

def derive_blinding(master_seed: bytes, nonce: bytes):
    """Replikasi derive_blinding dari Rust."""
    ikm = master_seed + nonce
    prk = hkdf_extract(None, ikm)
    blinding_bytes = hkdf_expand(prk, b"fcr-rsa-blinding-v1", 256)
    return int.from_bytes(blinding_bytes, 'big') % Q

def encrypt(message: int, e_vec: int, blinding: int):
    """Replikasi encrypt_with_blinding_ct dari Rust."""
    return (message * e_vec + blinding) % Q

def main():
    print("🐍 Generating Cross-Language Test Vectors (Python Oracle)...")
    
    vectors = []
    for i in range(10):
        master_seed = os.urandom(32)
        nonce = os.urandom(12)
        message_bytes = os.urandom(32)
        
        message_int = int.from_bytes(message_bytes, 'big') % Q
        
        e_vec, _ = generate_key_pair(master_seed)
        blinding = derive_blinding(master_seed, nonce)
        ciphertext = encrypt(message_int, e_vec, blinding)
        
        # ✅ PERBAIKAN KRITIS: Gunakan to_bytes() yang selalu menghasilkan 256 byte
        # Ini menjamin hex string SELALU 512 karakter, termasuk leading zeros
        ct_bytes = ciphertext.to_bytes(256, 'big')
        
        vectors.append({
            "id": i + 1,
            "master_seed_hex": master_seed.hex(),
            "nonce_hex": nonce.hex(),
            "message_hex": message_bytes.hex(),
            "expected_ciphertext_hex": ct_bytes.hex()  # Selalu 512 karakter!
        })
    
    output_dir = "fcr-core-rust/tests"
    output_file = os.path.join(output_dir, "test_vectors.json")
    os.makedirs(output_dir, exist_ok=True)
    
    with open(output_file, "w") as f:
        json.dump(vectors, f, indent=2)
    
    print(f"✅ Successfully generated {len(vectors)} test vectors -> {output_file}")
    print("👉 File ini sekarang siap dibaca oleh Rust Integration Test!")

if __name__ == "__main__":
    main()