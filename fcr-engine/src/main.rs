use clap::{Parser, Subcommand};
// Kita impor fungsi Tier A dan fungsi _with_blinding
use fcr_rsa::{encrypt, encrypt_with_blinding, encrypt_tier_a, encrypt_tier_a_with_blinding, decrypt, decrypt_tier_a};
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use std::str::FromStr;

#[derive(Parser)]
#[command(name = "fcr-engine")]
#[command(version = "0.1.0")]
#[command(about = "FCR-RSA Command Line Engine (Research Prototype)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encrypt a plaintext message
    Encrypt {
        #[arg(short, long)]
        message: u64,
        #[arg(short, long, default_value = "fcr-rsa-demo-seed")]
        seed: String,
        #[arg(short, long, default_value_t = 10)]
        n: usize,
        #[arg(short, long, default_value = "1000000007")]
        q: String,
        #[arg(short, long)]
        blinding: Option<String>,
        /// Pilih tier: "b" (default) atau "a" (enterprise)
        #[arg(short, long, default_value = "b")]
        tier: String,
    },
    
    /// Decrypt a ciphertext message
    Decrypt {
        #[arg(short, long)]
        ciphertext: String,
        #[arg(short, long, default_value = "fcr-rsa-demo-seed")]
        seed: String,
        #[arg(short, long, default_value_t = 10)]
        n: usize,
        #[arg(short, long, default_value = "1000000007")]
        q: String,
        #[arg(short, long)]
        blinding: String,
        /// Pilih tier: "b" (default) atau "a" (enterprise)
        #[arg(short, long, default_value = "b")]
        tier: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Encrypt { message, seed, n, q, blinding, tier } => {
            let q_big = BigInt::from_str(q).expect("Invalid Q");
            let m_big = BigInt::from_u64(*message).expect("Invalid message");
            let seed_bytes = seed.as_bytes();
            let is_tier_a = tier.to_lowercase() == "a";

            let ct = if let Some(b_str) = blinding {
                // Mode Deterministik
                let b_big = BigInt::from_str(b_str).expect("Invalid blinding");
                if is_tier_a {
                    encrypt_tier_a_with_blinding(&m_big, seed_bytes, *n, &q_big, &b_big).expect("Encryption failed")
                } else {
                    encrypt_with_blinding(&m_big, seed_bytes, *n, &q_big, &b_big).expect("Encryption failed")
                }
            } else {
                // Mode Produksi (Acak)
                if is_tier_a {
                    encrypt_tier_a(&m_big, seed_bytes, *n, &q_big).expect("Encryption failed")
                } else {
                    encrypt(&m_big, seed_bytes, *n, &q_big).expect("Encryption failed")
                }
            };

            println!("Ciphertext (Tier {}): {}", tier.to_uppercase(), ct);
        }
        
        Commands::Decrypt { ciphertext, seed, n, q, blinding, tier } => {
            let q_big = BigInt::from_str(q).expect("Invalid Q");
            let c_big = BigInt::from_str(ciphertext).expect("Invalid ciphertext");
            let b_big = BigInt::from_str(blinding).expect("Invalid blinding");
            let seed_bytes = seed.as_bytes();
            let is_tier_a = tier.to_lowercase() == "a";

            let result = if is_tier_a {
                decrypt_tier_a(&c_big, seed_bytes, *n, &q_big, &b_big)
            } else {
                decrypt(&c_big, seed_bytes, *n, &q_big, &b_big)
            };

            match result {
                Ok(m) => println!("Decrypted Plaintext (Tier {}): {}", tier.to_uppercase(), m),
                Err(e) => eprintln!("Decryption Error: {}", e),
            }
        }
    }
}