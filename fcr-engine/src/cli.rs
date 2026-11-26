//! CLI definition (Clap) — dipisah dari main.rs agar rapi.

use clap::{Parser, Subcommand};
use std::net::SocketAddr;

#[derive(Parser)]
#[command(name = "fcr-engine")]
#[command(version = "0.1.0")]
#[command(about = "FCR-RSA Engine: CLI + REST API (Research Prototype)", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a new key and save it to PostgreSQL
    Keygen {
        #[arg(short = 'N', long)]
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
        #[arg(short = 'm', long)]
        text: String,
        #[arg(short = 'k', long)]
        keyname: String,
        /// (Opsional) Jika tidak diisi, akan menggunakan mode acak (IND-CPA)
        #[arg(short = 'b', long)]
        blinding: Option<String>,
        #[arg(short = 't', long, default_value = "b")]
        tier: String,
    },

    /// Decrypt a ciphertext message
    Decrypt {
        /// Ciphertext (Format: BLINDING:CT1,CT2,... atau CT1,CT2,...)
        #[arg(short = 'c', long)]
        ciphertext: String,
        #[arg(short = 'k', long)]
        keyname: String,
        /// (Opsional) Hanya diperlukan jika ciphertext tidak menyertakan blinding
        #[arg(short = 'b', long)]
        blinding: Option<String>,
        #[arg(short = 't', long, default_value = "b")]
        tier: String,
    },

    /// Kelola API key untuk REST API
    Apikey {
        #[command(subcommand)]
        action: ApikeyAction,
    },

    /// Jalankan REST API server
    Serve {
        /// Port server (default 8080, override via env FCR_PORT)
        #[arg(short, long)]
        port: Option<u16>,
    },
    /// Kirim satu pesan teks ke peer lewat FCR-TP (UDP + port hopping)
    TpSend {
        /// Alamat peer, mis. 127.0.0.1:50000
        #[arg(long)]
        peer: SocketAddr,
        /// Teks yang dikirim
        #[arg(short = 'm', long)]
        text: String,
        /// Secret sesi bersama (HARUS sama di kedua pihak)
        #[arg(short = 's', long)]
        secret: String,
        /// Tier: a atau b (harus sama di kedua pihak)
        #[arg(short = 't', long, default_value = "b")]
        tier: String,
        /// Alamat bind lokal (default: port acak)
        #[arg(long, default_value = "0.0.0.0:0")]
        bind: String,
    },

    /// Dengarkan pesan masuk lewat FCR-TP
    TpListen {
        /// Alamat bind lokal, mis. 0.0.0.0:50000
        #[arg(long, default_value = "0.0.0.0:50000")]
        bind: String,
        /// Secret sesi bersama (HARUS sama di kedua pihak)
        #[arg(short = 's', long)]
        secret: String,
        /// Tier: a atau b (harus sama di kedua pihak)
        #[arg(short = 't', long, default_value = "b")]
        tier: String,
        /// Berapa pesan lengkap yang ditunggu sebelum berhenti
        #[arg(long, default_value_t = 1)]
        count: usize,
    },
}

#[derive(Subcommand)]
pub enum ApikeyAction {
    /// Buat API key baru (plaintext hanya ditampilkan sekali)
    Create {
        #[arg(short = 'u', long)]
        user: String,
        #[arg(short = 'l', long)]
        label: Option<String>,
    },
    /// Daftar API key (tanpa plaintext, hanya prefix & metadata)
    List,
    /// Nonaktifkan API key berdasarkan prefix
    Revoke {
        #[arg(short = 'p', long)]
        prefix: String,
    },
}