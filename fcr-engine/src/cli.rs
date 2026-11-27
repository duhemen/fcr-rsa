use clap::{Parser, Subcommand};
use std::net::SocketAddr;

#[derive(Parser, Debug)]
#[command(name = "fcr-engine", about = "FCR-RSA Engine: CLI + REST API + FCR-TP")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Buat kunci baru
    Keygen {
        #[arg(short = 'N', long)]
        name: String,
        #[arg(short = 's', long)]
        seed: String,
        #[arg(short = 'n', long, default_value_t = 10)]
        n: usize,
        #[arg(short = 'q', long, default_value = "2048")]
        q: String,
    },
    /// Daftar kunci
    ListKeys,
    /// Enkripsi teks
    Encrypt {
        #[arg(short = 'm', long)]
        text: String,
        #[arg(short = 'k', long)]
        keyname: String,
        #[arg(long)]
        blinding: Option<String>,
        #[arg(long, default_value = "b")]
        tier: String,
    },
    /// Dekripsi teks
    Decrypt {
        #[arg(short = 'c', long)]
        ciphertext: String,
        #[arg(short = 'k', long)]
        keyname: String,
        #[arg(long)]
        blinding: Option<String>,
        #[arg(long, default_value = "b")]
        tier: String,
    },
    /// Manajemen API key
    Apikey {
        #[command(subcommand)]
        action: ApikeyAction,
    },
    /// Jalankan REST API server
    Serve {
        #[arg(long)]
        port: Option<u16>,
    },
    // ===== FCR-TP lama, tetap dipertahankan untuk demo =====
    /// Kirim lewat FCR-TP dengan secret manual (demo / kompatibilitas)
    TpSend {
        #[arg(long)]
        peer: SocketAddr,
        #[arg(long)]
        text: String,
        #[arg(long)]
        secret: String,
        #[arg(long, default_value = "b")]
        tier: String,
        #[arg(long, default_value = "0.0.0.0:0")]
        bind: String,
    },
    /// Dengarkan FCR-TP dengan secret manual (demo / kompatibilitas)
    TpListen {
        #[arg(long, default_value = "0.0.0.0:50000")]
        bind: String,
        #[arg(long)]
        secret: String,
        #[arg(long, default_value = "b")]
        tier: String,
        #[arg(long, default_value_t = 1)]
        count: usize,
    },
    // ===== FCR-TP baru: Handshake UDP + KEM + Canary + Auto-Imun =====
    /// Handshake UDP langsung: tukar seed lewat KEM, tanpa --secret ketik manual
    TpHsSend {
        #[arg(long)]
        peer: SocketAddr,
        #[arg(long)]
        text: String,
        /// Seed kunci bersama untuk KEM demo (produksi: ganti dengan KeyRecord terpisah).
        /// Ini BUKAN secret sesi. Secret sesi dibuat acak per sesi oleh mesin.
        #[arg(long)]
        keyseed: String,
        #[arg(long, default_value = "b")]
        tier: String,
        #[arg(long, default_value = "0.0.0.0:0")]
        bind: String,
        /// Parameter yang ditawarkan, dipisah koma. Contoh: v1-U2048,v2-U4096
        /// Mesin pilih yang terkuat yang sama-sama didukung. Everlasting: tambah v baru tanpa bongkar lama.
        #[arg(long, default_value = "v1-U2048,v2-U4096")]
        offered: String,
    },

    /// Uji replay: kirim HELLO yang sama persis 2x, yang kedua harus ditolak
    TpHsReplay {
        #[arg(long)]
        peer: SocketAddr,
        #[arg(long)]
        text: String,
        #[arg(long)]
        keyseed: String,
        #[arg(long, default_value = "b")]
        tier: String,
        #[arg(long, default_value = "0.0.0.0:0")]
        bind: String,
        #[arg(long, default_value = "v1-U2048,v2-U4096")]
        offered: String,
    },

    /// Dengarkan handshake UDP: verifikasi KEM + canary + anti-replay
    TpHsListen {
        #[arg(long, default_value = "0.0.0.0:50000")]
        bind: String,
        #[arg(long)]
        keyseed: String,
        #[arg(long, default_value = "b")]
        tier: String,
        #[arg(long, default_value_t = 1)]
        count: usize,
        #[arg(long, default_value = "v1-U2048,v2-U4096")]
        offered: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ApikeyAction {
    Create {
        #[arg(long)]
        user: String,
        #[arg(long)]
        label: Option<String>,
    },
    List,
    Revoke {
        #[arg(long)]
        prefix: String,
    },
}