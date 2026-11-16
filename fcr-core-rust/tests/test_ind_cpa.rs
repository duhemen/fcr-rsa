use fcr_rsa::encrypt;
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use std::collections::HashSet;

#[test]
fn test_ind_cpa_randomness() {
    let q = BigInt::from_u64(1_000_000_007).unwrap();
    let seed = b"fcr-rsa-demo-seed";
    let n = 10;
    let m = BigInt::from_u64(42).unwrap();

    let mut ciphertexts = HashSet::new();
    let iterations = 100;

    // Enkripsi 100 kali, simpan di HashSet
    for _ in 0..iterations {
        let c = encrypt(&m, seed, n, &q).expect("Encryption failed");
        ciphertexts.insert(c);
    }

    // Pastikan semua ciphertext unik (IND-CPA)
    assert_eq!(
        ciphertexts.len(),
        iterations,
        "IND-CPA FAILED: Ada ciphertext yang duplikat! R_blind tidak acak."
    );

    println!("✅ IND-CPA PASSED: {} ciphertext unik dihasilkan dari {} enkripsi.", iterations, iterations);
}