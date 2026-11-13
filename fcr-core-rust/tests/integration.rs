//! Integration tests.
use fcr_rsa::{decrypt, encrypt};
use num_bigint::BigInt;
use num_traits::FromPrimitive;

#[test]
fn round_trip_small() {
    let q = BigInt::from_u64(1_000_000_007).unwrap();
    let seed = b"test-seed";
    let n = 10;
    let b = BigInt::from_u64(7).unwrap();
    let m = BigInt::from_u64(123).unwrap();
    let c = encrypt(&m, seed, n, &q, &b).unwrap();
    let d = decrypt(&c, seed, n, &q, &b).unwrap();
    assert_eq!(m, d);
}

#[test]
fn round_trip_multiple() {
    let q = BigInt::from_u64(1_000_000_007).unwrap();
    let seed = b"test-seed";
    let n = 10;
    let b = BigInt::from_u64(7).unwrap();
    for m_val in [1u64, 2, 10, 100, 1000] {
        let m = BigInt::from_u64(m_val).unwrap();
        let c = encrypt(&m, seed, n, &q, &b).unwrap();
        let d = decrypt(&c, seed, n, &q, &b).unwrap();
        assert_eq!(m, d, "failed for m={}", m_val);
    }
}

#[test]
fn invalid_n_rejected() {
    let q = BigInt::from_u64(1_000_000_007).unwrap();
    let seed = b"test";
    let b = BigInt::from_u64(1).unwrap();
    let m = BigInt::from_u64(1).unwrap();
    assert!(encrypt(&m, seed, 2, &q, &b).is_err());
}

#[test]
fn round_trip_tier_a() {
    use fcr_rsa::{encrypt_tier_a, decrypt_tier_a};
    let q = BigInt::from_u64(1_000_000_007).unwrap();
    let seed = b"fcr-rsa-demo-seed";
    let n = 10;
    let b = BigInt::from_u64(12345).unwrap();
    let m = BigInt::from_u64(42).unwrap();
    
    let c = encrypt_tier_a(&m, seed, n, &q, &b).unwrap();
    assert_eq!(c.to_string(), "758780528"); // Memastikan cocok dengan Python
    
    let d = decrypt_tier_a(&c, seed, n, &q, &b).unwrap();
    assert_eq!(m, d);
}