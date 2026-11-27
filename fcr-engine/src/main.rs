//! FCR-RSA Engine: CLI + REST API (Research Prototype).
//!
//! Subcommand:
//! - `keygen`, `list-keys`, `encrypt`, `decrypt` - operasi CLI klasik
//! - `apikey create|list|revoke` - manajemen API key
//! - `serve` - jalankan REST API server
//! - `tp-send`, `tp-listen` - transport FCR-TP (TANPA database)
//!
//! Prinsip arsitektur yang dijaga file ini:
//! 1. Transport tidak bergantung pada persistence (Postgres).
//!    Alasan: transport harus jalan di edge / lapangan / tanpa DB,
//!    dan harus tetap jalan saat backend ASI / layanan lain down.
//! 2. Crypto agility: main.rs tidak tahu detail KEM / HKDF / hopper.
//!    Semua penurunan kunci ada di crypto.rs + transport.rs.
//!    Ganti algoritma di masa depan = ganti modul itu, bukan main.rs.
//! 3. Human-centered: perintah transport bisa dipakai enduser
//!    tanpa .env, tanpa DATABASE_URL, tanpa Postgres jalan.

mod api;
mod cli;
mod crypto;
mod db;
mod transport;

use clap::Parser;
use cli::{ApikeyAction, Cli, Commands};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    dotenvy::dotenv().ok();

    // Init tracing (untuk log request API)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into()),
        )
        .init();

    // LANGKAH 1: Parse CLI DULU, sebelum sentuh database.
    // Ini kunci Prioritas 1. Perintah transport harus bisa jalan
    // tanpa DATABASE_URL dan tanpa Postgres.
    let cli = Cli::parse();

    // LANGKAH 2: Tangani perintah transport SEBELUM koneksi DB.
    // Transport adalah jalur komunikasi stateless: secret + peer
    // + session key sudah cukup. Tidak ada baca/tulis DB di sini.
    match &cli.command {
        Commands::TpSend {
            peer,
            text,
            secret,
            tier,
            bind,
        } => match transport::run_send(bind, *peer, secret, tier, text) {
            Ok(n) => {
                println!("Terkirim {} frame FCR-TP ke {}", n, peer);
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("Gagal kirim FCR-TP: {}", e);
                return ExitCode::FAILURE;
            }
        },

        Commands::TpListen {
            bind,
            secret,
            tier,
            count,
        } => match transport::run_listen(bind, secret, tier, *count) {
            Ok(()) => {
                println!("Selesai mendengarkan.");
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("Gagal listen FCR-TP: {}", e);
                return ExitCode::FAILURE;
            }
        },
        Commands::TpHsSend {
            peer,
            text,
            keyseed,
            tier,
            bind,
            offered,
        } => match transport::run_handshake_send(&bind, *peer, &keyseed, &tier, &text, &offered) {
            Ok(n) => {
                println!("Handshake + kirim selesai, {} frame.", n);
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("Gagal handshake-send: {}", e);
                return ExitCode::FAILURE;
            }
        },

        Commands::TpHsListen {
            bind,
            keyseed,
            tier,
            count,
            offered,
        } => match transport::run_handshake_listen(&bind, &keyseed, &tier, *count, &offered) {
            Ok(()) => {
                println!("Selesai mendengarkan handshake.");
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("Gagal handshake-listen: {}", e);
                return ExitCode::FAILURE;
            }
        },

        // Semua perintah lain BUTUH database -> lanjut ke bawah.
        _ => {}
    }

    // LANGKAH 3: Baru di sini kita butuh database.
    // keygen, list-keys, encrypt, decrypt, apikey, serve
    // semuanya memakai Postgres, jadi wajar gagal kalau
    // DATABASE_URL tidak ada atau Postgres tidak jalan.
    let db_url = match std::env::var("DATABASE_URL") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("[X] DATABASE_URL must be set in .env");
            return ExitCode::FAILURE;
        }
    };

    let pool = match db::connect_pool(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[X] Gagal koneksi ke PostgreSQL: {}", e);
            return ExitCode::FAILURE;
        }
    };

    match cli.command {
        // ===== CLI klasik =====
        Commands::Keygen { name, seed, n, q } => {
            if n < 4 {
                eprintln!("[X] n harus >= 4");
                return ExitCode::FAILURE;
            }
            match db::upsert_key(&pool, &name, &seed, n as i32, &q).await {
                Ok(_) => println!("[OK] Kunci '{}' berhasil disimpan ke PostgreSQL.", name),
                Err(e) => {
                    eprintln!("[X] Gagal menyimpan kunci: {}", e);
                    return ExitCode::FAILURE;
                }
            }
        }

        Commands::ListKeys => match db::list_keys(&pool).await {
            Ok(records) => {
                println!("Daftar Kunci di Database:");
                for (i, r) in records.iter().enumerate() {
                    println!("{}. Seed: {}, N: {}, Q: {}", i + 1, r.seed, r.n, r.q);
                }
            }
            Err(e) => {
                eprintln!("[X] Gagal mengambil daftar kunci: {}", e);
                return ExitCode::FAILURE;
            }
        },

        Commands::Encrypt {
            text,
            keyname,
            blinding,
            tier,
        } => {
            let key = match db::get_key_by_name(&pool, &keyname).await {
                Ok(k) => k,
                Err(sqlx::Error::RowNotFound) => {
                    eprintln!("[X] Kunci '{}' tidak ditemukan di database", keyname);
                    return ExitCode::FAILURE;
                }
                Err(e) => {
                    eprintln!("[X] DB error: {}", e);
                    return ExitCode::FAILURE;
                }
            };

            match crypto::encrypt_text(&text, &key, &tier, blinding.as_deref()) {
                Ok(ct) => {
                    println!("Plaintext  : {}", text);
                    println!("Ciphertext (Tier {}): {}", tier.to_uppercase(), ct);
                }
                Err(e) => {
                    eprintln!("[X] Enkripsi gagal: {}", e);
                    return ExitCode::FAILURE;
                }
            }
        }

        Commands::Decrypt {
            ciphertext,
            keyname,
            blinding,
            tier,
        } => {
            let key = match db::get_key_by_name(&pool, &keyname).await {
                Ok(k) => k,
                Err(sqlx::Error::RowNotFound) => {
                    eprintln!("[X] Kunci '{}' tidak ditemukan di database", keyname);
                    return ExitCode::FAILURE;
                }
                Err(e) => {
                    eprintln!("[X] DB error: {}", e);
                    return ExitCode::FAILURE;
                }
            };

            match crypto::decrypt_text(&ciphertext, &key, &tier, blinding.as_deref()) {
                Ok(pt) => println!("Decrypted Text: {}", pt),
                Err(e) => {
                    eprintln!("[X] Dekripsi gagal: {}", e);
                    return ExitCode::FAILURE;
                }
            }
        }

        // ===== API key management =====
        Commands::Apikey { action } => match action {
            ApikeyAction::Create { user, label } => {
                let (plaintext, hash, prefix) = crypto::generate_api_key();
                match db::insert_api_key(&pool, &user, label.as_deref(), &prefix, &hash).await {
                    Ok(_) => {
                        println!("[OK] API key berhasil dibuat untuk user '{}'", user);
                        println!();
                        println!("[!] SIMPAN SEKARANG (tidak akan ditampilkan lagi):");
                        println!("{}", plaintext);
                        println!();
                        println!("Prefix: {} | Label: {:?}", prefix, label);
                    }
                    Err(e) => {
                        eprintln!("[X] Gagal menyimpan API key: {}", e);
                        return ExitCode::FAILURE;
                    }
                }
            }

            ApikeyAction::List => match db::list_api_keys(&pool).await {
                Ok(rows) => {
                    println!("Daftar API Key:");
                    for (id, user, label, prefix, active) in rows {
                        println!(
                            "{}. user={} label={:?} prefix={} active={}",
                            id, user, label, prefix, active
                        );
                    }
                }
                Err(e) => {
                    eprintln!("[X] Gagal mengambil daftar API key: {}", e);
                    return ExitCode::FAILURE;
                }
            },

            ApikeyAction::Revoke { prefix } => {
                match db::revoke_api_key_by_prefix(&pool, &prefix).await {
                    Ok(n) if n > 0 => println!("[OK] {} API key dinonaktifkan (prefix: {})", n, prefix),
                    Ok(_) => {
                        eprintln!("[!] Tidak ada API key aktif dengan prefix '{}'", prefix);
                        return ExitCode::FAILURE;
                    }
                    Err(e) => {
                        eprintln!("[X] Gagal revoke API key: {}", e);
                        return ExitCode::FAILURE;
                    }
                }
            }
        },

        // ===== REST API server =====
        Commands::Serve { port } => {
            let port = port
                .or_else(|| std::env::var("FCR_PORT").ok().and_then(|p| p.parse().ok()))
                .unwrap_or(8080);

            if let Err(e) = api::serve(pool, port).await {
                eprintln!("[X] Server error: {}", e);
                return ExitCode::FAILURE;
            }
        }

        // Sudah ditangani di atas, sebelum DB. Arm ini tidak akan
        // tercapai karena kita sudah return lebih awal.
        // Tetap ada agar match exhaustive.
        Commands::TpSend { .. } | Commands::TpListen { .. } => {
            unreachable!("transport sudah ditangani sebelum koneksi DB")
        }
        Commands::TpHsSend { .. } | Commands::TpHsListen { .. } => {
            unreachable!("handshake sudah ditangani sebelum koneksi DB")
        }
    }

    ExitCode::SUCCESS
}