use clap::{Parser, Subcommand};
use fcr_rsa::{encrypt, encrypt_with_blinding, encrypt_tier_a, encrypt_tier_a_with_blinding, decrypt, decrypt_tier_a};
use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};
use sqlx::postgres::PgPoolOptions;
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
    /// Generate a new key and save it to PostgreSQL
    Keygen {
        /// Nama unik untuk kunci ini
        #[arg(short = 'N', long)] // Pakai 'N' besar untuk nama
        name: String,
        #[arg(short, long, default_value = "fcr-rsa-demo-seed")]
        seed: String,
        #[arg(short = 'i', long, default_value_t = 10)]
        n: usize,
        #[arg(short = 'q', long, default_value = "1000000007")]
        q: String,
    },
    /// List all keys in the database
    ListKeys,
    /// Encrypt a plaintext message
    Encrypt {
        #[arg(short = 'm', long)] // Pakai 'm' untuk message (teks)
        text: String,
        #[arg(short = 'k', long)] // Pakai 'k' untuk keyname
        keyname: String,
        #[arg(short = 'b', long)] // Pakai 'b' untuk blinding
        blinding: Option<String>,
        #[arg(short = 't', long, default_value = "b")] // Pakai 't' untuk tier
        tier: String,
    },
    /// Decrypt a ciphertext message
    Decrypt {
        #[arg(short = 'c', long)] // Pakai 'c' untuk ciphertext
        ciphertext: String,
        #[arg(short = 'k', long)] // Pakai 'k' untuk keyname
        keyname: String,
        #[arg(short = 'b', long)] // Pakai 'b' untuk blinding
        blinding: String,
        #[arg(short = 't', long, default_value = "b")] // Pakai 't' untuk tier
        tier: String,
    },
}

// Struct untuk menampung hasil query
#[derive(sqlx::FromRow)]
struct KeyRecord {
    seed: String,
    n: i32,
    q: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

    // Buat koneksi pool ke PostgreSQL
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    let cli = Cli::parse();

    match &cli.command {
        Commands::Keygen { name, seed, n, q } => {
            // Simpan ke database
            sqlx::query(
                "INSERT INTO keys (name, seed, n, q) VALUES ($1, $2, $3, $4)
                 ON CONFLICT (name) DO UPDATE SET seed = $2, n = $3, q = $4"
            )
            .bind(name)
            .bind(seed)
            .bind(*n as i32)
            .bind(q)
            .execute(&pool)
            .await
            .expect("Gagal menyimpan kunci ke database");

            println!("✅ Kunci '{}' berhasil disimpan ke PostgreSQL.", name);
        }

        Commands::ListKeys => {
            let records = sqlx::query_as::<_, KeyRecord>("SELECT seed, n, q FROM keys")
                .fetch_all(&pool)
                .await
                .expect("Gagal mengambil daftar kunci");
            
            println!("📋 Daftar Kunci di Database:");
            for (i, r) in records.iter().enumerate() {
                println!("{}. Seed: {}, N: {}, Q: {}", i+1, r.seed, r.n, r.q);
            }
        }
        
        Commands::Encrypt { text, keyname, blinding, tier } => {
            // Ambil kunci dari database
            let record: KeyRecord = sqlx::query_as("SELECT seed, n, q FROM keys WHERE name = $1")
                .bind(keyname)
                .fetch_one(&pool)
                .await
                .expect("Kunci tidak ditemukan di database");

            let q_big = BigInt::from_str(&record.q).expect("Invalid Q");
            let seed_bytes = record.seed.as_bytes();
            let n_val = record.n as usize;
            let is_tier_a = tier.to_lowercase() == "a";
            let b_big = blinding.as_ref().map(|b| BigInt::from_str(b).expect("Invalid blinding"));

            let mut ciphertexts = Vec::new();

            for byte in text.as_bytes() {
                let m_big = BigInt::from_u8(*byte).unwrap();
                let ct = if let Some(ref b) = b_big {
                    if is_tier_a {
                        encrypt_tier_a_with_blinding(&m_big, seed_bytes, n_val, &q_big, b).expect("Encryption failed")
                    } else {
                        encrypt_with_blinding(&m_big, seed_bytes, n_val, &q_big, b).expect("Encryption failed")
                    }
                } else {
                    if is_tier_a {
                        encrypt_tier_a(&m_big, seed_bytes, n_val, &q_big).expect("Encryption failed")
                    } else {
                        encrypt(&m_big, seed_bytes, n_val, &q_big).expect("Encryption failed")
                    }
                };
                ciphertexts.push(ct.to_string());
            }

            println!("Plaintext  : {}", text);
            println!("Ciphertext (Tier {}): {}", tier.to_uppercase(), ciphertexts.join(","));
        }
        
        Commands::Decrypt { ciphertext, keyname, blinding, tier } => {
            let record: KeyRecord = sqlx::query_as("SELECT seed, n, q FROM keys WHERE name = $1")
                .bind(keyname)
                .fetch_one(&pool)
                .await
                .expect("Kunci tidak ditemukan di database");

            let q_big = BigInt::from_str(&record.q).expect("Invalid Q");
            let b_big = BigInt::from_str(blinding).expect("Invalid blinding");
            let seed_bytes = record.seed.as_bytes();
            let n_val = record.n as usize;
            let is_tier_a = tier.to_lowercase() == "a";

            let ct_strings: Vec<&str> = ciphertext.split(',').collect();
            let mut decrypted_bytes = Vec::new();

            for ct_str in ct_strings {
                let c_big = BigInt::from_str(ct_str.trim()).expect("Invalid ciphertext part");
                
                let result = if is_tier_a {
                    decrypt_tier_a(&c_big, seed_bytes, n_val, &q_big, &b_big)
                } else {
                    decrypt(&c_big, seed_bytes, n_val, &q_big, &b_big)
                };

                match result {
                    Ok(m) => {
                        let byte = m.to_u8().expect("Decrypted value is not a valid byte");
                        decrypted_bytes.push(byte);
                    },
                    Err(e) => {
                        eprintln!("Decryption Error pada bagian {}: {}", ct_str, e);
                        return;
                    }
                }
            }

            let decrypted_text = String::from_utf8_lossy(&decrypted_bytes).to_string();
            println!("Decrypted Text: {}", decrypted_text);
        }
    }
}