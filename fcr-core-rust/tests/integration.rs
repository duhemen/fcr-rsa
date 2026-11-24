//! FCR-RSA v0.6.1 Integration Tests
//! Test suite komprehensif untuk Q 2048-bit

use crypto_bigint::modular::{FixedMontyForm, FixedMontyParams};
use crypto_bigint::{Odd, U2048};
use fcr_core_rust::{
    bytes_to_u2048, decrypt_with_blinding_ct, derive_blinding, encrypt_with_blinding_ct,
    fibonacci_nonlinear_with_lwe, gaussian_map, generate_key_pair, max_seed_bytes, u2048_to_bytes,
    Q_2048,
};

/// Helper: Verifikasi perkalian modular menggunakan DynResidue (sama seperti di fib.rs)
fn verify_mul_mod(a: &U2048, b: &U2048, expected: &U2048) -> bool {
    let odd_q = Odd::new(Q_2048).unwrap();
    let params = FixedMontyParams::new(odd_q);
    let a_res = FixedMontyForm::new(a, &params);
    let b_res = FixedMontyForm::new(b, &params);
    let product = a_res.mul(&b_res).retrieve();
    &product == expected
}

#[test]
fn test_max_seed_bytes() {
    assert_eq!(max_seed_bytes(), 255);
}

#[test]
fn test_bytes_to_u2048_small() {
    let bytes = [1u8, 2, 3, 4];
    let value = bytes_to_u2048(&bytes).unwrap();
    assert!(!bool::from(value.is_zero()));
}

#[test]
fn test_bytes_to_u2048_max() {
    let bytes = [0xFFu8; 255];
    let value = bytes_to_u2048(&bytes).unwrap();
    assert!(!bool::from(value.is_zero()));
}

#[test]
fn test_bytes_to_u2048_too_large() {
    let bytes = [0u8; 257];
    assert!(bytes_to_u2048(&bytes).is_err());
}

#[test]
fn test_u2048_to_bytes_roundtrip() {
    let original = [42u8; 100];
    let value = bytes_to_u2048(&original).unwrap();
    let bytes = u2048_to_bytes(&value);
    assert_eq!(&bytes[156..], &original[..]);
}

#[test]
fn test_generate_key_pair() {
    let master_seed = b"test master seed for key generation";
    let key_pair = generate_key_pair(master_seed).unwrap();

    assert!(!bool::from(key_pair.e_vec.is_zero()));
    assert!(!bool::from(key_pair.e_vec_inv.is_zero()));

    // VERIFIKASI: E_vec * E_vec_inv ≡ 1 (mod Q)
    // PERBAIKAN: Gunakan DynResidue (konsisten dengan fib.rs)
    let is_valid = verify_mul_mod(&key_pair.e_vec, &key_pair.e_vec_inv, &U2048::ONE);
    assert!(is_valid, "E_vec * E_vec_inv harus ≡ 1 (mod Q)");
}

#[test]
fn test_generate_key_pair_deterministic() {
    let master_seed = b"deterministic test";
    let key_pair1 = generate_key_pair(master_seed).unwrap();
    let key_pair2 = generate_key_pair(master_seed).unwrap();

    assert_eq!(key_pair1.e_vec, key_pair2.e_vec);
    assert_eq!(key_pair1.e_vec_inv, key_pair2.e_vec_inv);
}

#[test]
fn test_derive_blinding() {
    let master_seed = b"master seed";
    let nonce = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    let blinding = derive_blinding(master_seed, &nonce).unwrap();
    assert!(blinding < Q_2048);
}

#[test]
fn test_derive_blinding_deterministic() {
    let master_seed = b"master seed";
    let nonce = [42u8; 12];
    let blinding1 = derive_blinding(master_seed, &nonce).unwrap();
    let blinding2 = derive_blinding(master_seed, &nonce).unwrap();
    assert_eq!(blinding1, blinding2);
}

#[test]
fn test_derive_blinding_different_nonce() {
    let master_seed = b"master seed";
    let nonce1 = [1u8; 12];
    let nonce2 = [2u8; 12];
    let blinding1 = derive_blinding(master_seed, &nonce1).unwrap();
    let blinding2 = derive_blinding(master_seed, &nonce2).unwrap();
    assert_ne!(blinding1, blinding2);
}

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let master_seed = b"test master seed";
    let key_pair = generate_key_pair(master_seed).unwrap();

    let session_seed = [42u8; 255];
    let message = bytes_to_u2048(&session_seed).unwrap();

    let nonce = [99u8; 12];
    let blinding = derive_blinding(master_seed, &nonce).unwrap();

    let encrypted = encrypt_with_blinding_ct(&message, &key_pair.e_vec, &blinding);
    let decrypted = decrypt_with_blinding_ct(&encrypted, &key_pair.e_vec_inv, &blinding);

    assert_eq!(message, decrypted);
}

#[test]
fn test_encrypt_decrypt_random_data() {
    let master_seed = b"random test";
    let key_pair = generate_key_pair(master_seed).unwrap();

    let mut session_seed = [0u8; 255];
    for (i, byte) in session_seed.iter_mut().enumerate() {
        *byte = (i * 7 + 13) as u8;
    }

    let message = bytes_to_u2048(&session_seed).unwrap();
    let nonce = [77u8; 12];
    let blinding = derive_blinding(master_seed, &nonce).unwrap();

    let encrypted = encrypt_with_blinding_ct(&message, &key_pair.e_vec, &blinding);
    let decrypted = decrypt_with_blinding_ct(&encrypted, &key_pair.e_vec_inv, &blinding);

    assert_eq!(message, decrypted);
}

#[test]
fn test_encrypt_produces_different_ciphertext() {
    let master_seed = b"ind-cpa test";
    let key_pair = generate_key_pair(master_seed).unwrap();
    let message = bytes_to_u2048(&[1u8; 100]).unwrap();

    let nonce1 = [1u8; 12];
    let blinding1 = derive_blinding(master_seed, &nonce1).unwrap();
    let encrypted1 = encrypt_with_blinding_ct(&message, &key_pair.e_vec, &blinding1);

    let nonce2 = [2u8; 12];
    let blinding2 = derive_blinding(master_seed, &nonce2).unwrap();
    let encrypted2 = encrypt_with_blinding_ct(&message, &key_pair.e_vec, &blinding2);

    assert_ne!(encrypted1, encrypted2);
}

#[test]
fn test_fibonacci_nonlinear() {
    let seed = bytes_to_u2048(b"fibonacci seed").unwrap();
    let result = fibonacci_nonlinear_with_lwe(10, &seed, &Q_2048);
    assert!(result < Q_2048);
}

#[test]
fn test_gaussian_map() {
    let value = bytes_to_u2048(b"gaussian test").unwrap();
    let result = gaussian_map(&value, &Q_2048);
    assert!(result < Q_2048);
}

#[test]
fn test_full_hybrid_kem_simulation() {
    let master_seed = b"full simulation test";
    let key_pair = generate_key_pair(master_seed).unwrap();

    let mut session_seed = [0u8; 255];
    for (i, byte) in session_seed.iter_mut().enumerate() {
        *byte = (i * 13 + 7) as u8;
    }

    let nonce = [55u8; 12];
    let blinding = derive_blinding(master_seed, &nonce).unwrap();

    let message = bytes_to_u2048(&session_seed).unwrap();
    let encrypted_seed = encrypt_with_blinding_ct(&message, &key_pair.e_vec, &blinding);

    let decrypted_seed = decrypt_with_blinding_ct(&encrypted_seed, &key_pair.e_vec_inv, &blinding);

    assert_eq!(message, decrypted_seed);

    let decrypted_bytes = u2048_to_bytes(&decrypted_seed);
    assert_eq!(&decrypted_bytes[1..256], &session_seed[..]);
}
