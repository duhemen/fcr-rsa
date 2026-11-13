//! Basic example of FCR-RSA usage.
use fcr_rsa::{decrypt, encrypt};
use num_bigint::BigInt;
use num_traits::FromPrimitive;

fn main() {
    let q = BigInt::from_u64(1_000_000_007).unwrap();
    let seed = b"fcr-rsa-demo-seed";
    let n = 10usize;
    let blinding = BigInt::from_u64(12345).unwrap();
    let msg = BigInt::from_u64(42).unwrap();

    println!("Plaintext : {}", msg);
    let ct = encrypt(&msg, seed, n, &q, &blinding).expect("encrypt");
    println!("Ciphertext: {}", ct);
    let pt = decrypt(&ct, seed, n, &q, &blinding).expect("decrypt");
    println!("Decrypted : {}", pt);
    assert_eq!(msg, pt, "round-trip failed");
    println!("Round-trip OK");
}