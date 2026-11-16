use clap::{Parser, Subcommand};
use fcr_rsa::{encrypt, encrypt_with_blinding, encrypt_tier_a, encrypt_tier_a_with_blinding, decrypt, decrypt_tier_a};
use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};
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
    Encrypt {
        #[arg(short = 'm', long)]
        text: String,
        #[arg(short, long, default_value = "fcr-rsa-demo-seed")]
        seed: String,
        #[arg(short, long, default_value_t = 10)]
        n: usize,
        #[arg(short, long, default_value = "1000000007")]
        q: String,
        #[arg(short, long)]
        blinding: Option<String>,
        #[arg(short, long, default_value = "b")]
        tier: String,
    },
    Decrypt {
        /// Ciphertext yang dipisahkan koma (contoh: "123,456,789")
        #[arg(short = 'c', long)]
        ciphertext: String,
        #[arg(short, long, default_value = "fcr-rsa-demo-seed")]
        seed: String,
        #[arg(short, long, default_value_t = 10)]
        n: usize,
        #[arg(short, long, default_value = "1000000007")]
        q: String,
        #[arg(short, long)]
        blinding: String,
        #[arg(short, long, default_value = "b")]
        tier: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Encrypt { text, seed, n, q, blinding, tier } => {
            let q_big = BigInt::from_str(q).expect("Invalid Q");
            let seed_bytes = seed.as_bytes();
            let is_tier_a = tier.to_lowercase() == "a";
            let b_big = blinding.as_ref().map(|b| BigInt::from_str(b).expect("Invalid blinding"));

            let mut ciphertexts = Vec::new();

            // Enkripsi per karakter (byte)
            for byte in text.as_bytes() {
                let m_big = BigInt::from_u8(*byte).unwrap();
                let ct = if let Some(ref b) = b_big {
                    if is_tier_a {
                        encrypt_tier_a_with_blinding(&m_big, seed_bytes, *n, &q_big, b).expect("Encryption failed")
                    } else {
                        encrypt_with_blinding(&m_big, seed_bytes, *n, &q_big, b).expect("Encryption failed")
                    }
                } else {
                    if is_tier_a {
                        encrypt_tier_a(&m_big, seed_bytes, *n, &q_big).expect("Encryption failed")
                    } else {
                        encrypt(&m_big, seed_bytes, *n, &q_big).expect("Encryption failed")
                    }
                };
                ciphertexts.push(ct.to_string());
            }

            println!("Plaintext  : {}", text);
            println!("Ciphertext (Tier {}): {}", tier.to_uppercase(), ciphertexts.join(","));
        }
        
        Commands::Decrypt { ciphertext, seed, n, q, blinding, tier } => {
            let q_big = BigInt::from_str(q).expect("Invalid Q");
            let b_big = BigInt::from_str(blinding).expect("Invalid blinding");
            let seed_bytes = seed.as_bytes();
            let is_tier_a = tier.to_lowercase() == "a";

            // Pecah string ciphertext berdasarkan koma
            let ct_strings: Vec<&str> = ciphertext.split(',').collect();
            let mut decrypted_bytes = Vec::new();

            for ct_str in ct_strings {
                let c_big = BigInt::from_str(ct_str.trim()).expect("Invalid ciphertext part");
                
                let result = if is_tier_a {
                    decrypt_tier_a(&c_big, seed_bytes, *n, &q_big, &b_big)
                } else {
                    decrypt(&c_big, seed_bytes, *n, &q_big, &b_big)
                };

                match result {
                    Ok(m) => {
                        // Konversi kembali ke byte (u8)
                        let byte = m.to_u8().expect("Decrypted value is not a valid byte");
                        decrypted_bytes.push(byte);
                    },
                    Err(e) => {
                        eprintln!("Decryption Error pada bagian {}: {}", ct_str, e);
                        return;
                    }
                }
            }

            // Ubah kumpulan byte menjadi String
            let decrypted_text = String::from_utf8_lossy(&decrypted_bytes).to_string();
            println!("Decrypted Text: {}", decrypted_text);
        }
    }
}