//! Basic example of FCR-RSA v0.6.0 usage.
use fcr_core_rust::{
    bytes_to_u2048,
    derive_blinding,
    decrypt_with_blinding_ct,
    encrypt_with_blinding_ct,
    generate_key_pair,
    u2048_to_bytes,
};

fn main() {
    let master_seed = b"fcr-rsa-demo-seed";
    
    println!("=== FCR-RSA v0.6.0 (Q 2048-bit) ===\n");
    
    // 1. Generate key pair
    println!("[1] Generating key pair...");
    let key_pair = generate_key_pair(master_seed).expect("Key generation failed");
    println!("✅ Key pair generated\n");
    
    // 2. Prepare message
    let msg_bytes = b"Hello, FCR-RSA v0.6.0!";
    println!("[2] Plaintext: {:?}", String::from_utf8_lossy(msg_bytes));
    
    let message = bytes_to_u2048(msg_bytes).expect("Message too large");
    
    // 3. Generate nonce (simulasi nonce acak)
    let nonce = [42u8; 12];
    
    // 4. Derive blinding deterministically
    let blinding = derive_blinding(master_seed, &nonce).expect("Blinding derivation failed");
    
    // 5. Encrypt
    println!("[3] Encrypting...");
    let ciphertext = encrypt_with_blinding_ct(&message, &key_pair.e_vec, &blinding);
    println!("✅ Encrypted\n");
    
    // 6. Decrypt
    println!("[4] Decrypting...");
    let decrypted = decrypt_with_blinding_ct(&ciphertext, &key_pair.e_vec_inv, &blinding);
    let decrypted_bytes = u2048_to_bytes(&decrypted);
    
    // 7. Extract original message (skip leading zeros)
    let first_non_zero = decrypted_bytes.iter().position(|&x| x != 0).unwrap_or(decrypted_bytes.len());
    let original = &decrypted_bytes[first_non_zero..];
    
    println!("[5] Decrypted: {:?}", String::from_utf8_lossy(original));
    
    if msg_bytes == original {
        println!("\n✅ Round-trip OK! FCR-RSA v0.6.0 works perfectly.");
    } else {
        println!("\n❌ Round-trip failed!");
    }
}