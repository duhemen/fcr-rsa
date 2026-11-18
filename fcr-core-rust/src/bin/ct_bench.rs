//! Constant-Time Benchmark for FCR-RSA v0.6.0
//! Mengukur performa enkripsi 2048-bit dengan deterministic blinding.

use fcr_core_rust::{
    bytes_to_u2048,
    derive_blinding,
    encrypt_with_blinding_ct,
    generate_key_pair,
};

fn main() {
    println!("=== FCR-RSA v0.6.0 Constant-Time Benchmark ===\n");
    
    let master_seed = b"fcr-rsa-demo-seed";
    
    // 1. Setup Key Pair (Dilakukan di luar loop)
    println!("[1] Generating Key Pair...");
    let key_pair = generate_key_pair(master_seed).expect("Key generation failed");
    
    // 2. Setup Message (2048-bit)
    let msg_bytes = [42u8; 100]; // Pesan 100 byte
    let message = bytes_to_u2048(&msg_bytes).expect("Message conversion failed");
    
    println!("[2] Running 10,000 Constant-Time Encryption Cycles...\n");
    
    let iterations = 10_000;
    let start = std::time::Instant::now();
    
    // 3. Benchmark Loop (Full cycle: derive blinding + encrypt per iterasi)
    for i in 0..iterations {
        // Simulasi perubahan nonce kecil untuk memastikan blinding deterministik
        let mut current_nonce = [0u8; 12];
        current_nonce[0] = (i % 256) as u8;
        current_nonce[1] = ((i / 256) % 256) as u8;
        
        // Derive blinding + Encrypt dalam satu siklus
        let current_blinding = derive_blinding(master_seed, &current_nonce).unwrap();
        let _ct = encrypt_with_blinding_ct(&message, &key_pair.e_vec, &current_blinding);
    }
    
    let duration = start.elapsed();
    let avg = duration / iterations;
    
    println!("✅ Completed {} iterations.", iterations);
    println!("Total Time   : {:?}", duration);
    println!("Average Time : {:?} per full cycle (derive + encrypt)", avg);
    println!("\n[NOTE] To verify constant-time property, run with `dudect` or `valgrind --tool=massif`.");
}