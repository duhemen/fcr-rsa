//! Jembatan FCR-Engine <-> FCR-TP (Langkah 5).
//!
//! Kedua pihak harus memakai `--secret` dan `--tier` yang SAMA.
//! Dari secret yang sama -> session seed sama -> session_key (HKDF)
//! sama -> seed hopper sama -> urutan port hopper sama.
//!
//! PROTOTIPE: secret diketik manual. Produksi harus menukar seed
//! lewat KEM FCR-RSA (encrypt_hybrid_kem), bukan lewat argumen CLI.

use std::net::SocketAddr;

use fcr_tp::{SocketConfig, UdpTransport};

use crate::crypto::{derive_hopper_seed, derive_session_key, session_seed_from_secret};

fn build_transport(
    bind: &str,
    secret: &str,
    tier: &str,
) -> Result<UdpTransport, Box<dyn std::error::Error>> {
    let seed_bytes = session_seed_from_secret(secret);
    let session_key = derive_session_key(&seed_bytes, tier)?;
    let hopper_seed = derive_hopper_seed(&seed_bytes);
    let transport = UdpTransport::bind(bind, hopper_seed, session_key, SocketConfig::default())?;
    Ok(transport)
}

/// Kirim satu pesan teks ke peer lewat FCR-TP. Return: jumlah frame terkirim.
pub fn run_send(
    bind: &str,
    peer: SocketAddr,
    secret: &str,
    tier: &str,
    text: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut transport = build_transport(bind, secret, tier)?;
    let sent = transport
        .send_payload(peer, text.as_bytes())
        .map_err(|e| format!("send_payload gagal: {}", e))?;
    Ok(sent)
}

/// Dengarkan dan cetak pesan lengkap yang diterima, sampai `max_messages`.
pub fn run_listen(
    bind: &str,
    secret: &str,
    tier: &str,
    max_messages: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut transport = build_transport(bind, secret, tier)?;
    println!("Mendengarkan FCR-TP di {} ...", transport.local_addr()?);
    let mut received = 0usize;
    while received < max_messages {
        let (result, anomaly) = transport
            .recv_chunk()
            .map_err(|e| format!("recv_chunk gagal: {}", e))?;
        if let Some(report) = anomaly {
            eprintln!("Anomali terdeteksi: {:?}", report);
        }
        if let Some(msg) = result {
            println!("Pesan diterima: {}", String::from_utf8_lossy(&msg.payload));
            received += 1;
        }
    }
    Ok(())
}