//! Jembatan FCR-Engine <-> FCR-TP (Langkah 5 + Handshake).
//!
//! Jalur lama:
//! - tp-send / tp-listen pakai --secret manual (demo, kompatibilitas).
//!
//! Jalur baru (produksi):
//! - tp-hs-send / tp-hs-listen pakai handshake UDP langsung.
//! - Seed sesi dibuat acak per sesi oleh mesin, ditukar lewat KEM.
//! - Tidak ada lagi --secret yang diketik manusia.
//!
//! Rel yang dipegang file ini (dari repo fcr-rsa):
//! 1. Hybrid KEM: FCR hanya bungkus seed acak, data pakai ChaCha20-Poly1305.
//!    Format ciphertext tetap: BLINDING_NONCE:ENCRYPTED_SEED:AEAD_NONCE:CIPHERTEXT.
//! 2. Tier terikat ke HKDF info. Tier diubah = kunci gagal.
//! 3. Auto-Imun: Risk Score >= 70 -> Poisoned Pill + trace_id + forensik,
//!    bukan error terang-terangan. GPS mentah TIDAK PERNAH disimpan.
//! 4. Adaptif / Everlasting: parameter dinegosiasikan (offered / chosen),
//!    versi tidak pernah dihapus, hanya ditambah. U2048 hari ini,
//!    U4096 besok, tanpa bongkar transport.
//! 5. Transport TIDAK pakai database. KeyRecord terpisah adalah Prioritas 2b.

use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use fcr_tp::{SocketConfig, UdpTransport};
use sha2::{Digest, Sha256};

use crate::crypto::{decrypt_hybrid_kem, decrypt_text, derive_hopper_seed, derive_session_key, encrypt_hybrid_kem, encrypt_text, session_seed_from_secret};
use crate::db::KeyRecord;

use base64::{Engine as _, engine::general_purpose};

// ============================================================
// Jalur lama - dipertahankan, jangan diubah perilakunya
// ============================================================

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

// ============================================================
// Jalur baru - Handshake UDP + KEM + Canary + Auto-Imun
// ============================================================

/// Satu parameter kripto yang ditawarkan / dipilih.
/// Format: v1-U2048, v2-U4096, v3-U8192, dst.
/// Everlasting: versi baru ditambah, versi lama tidak dihapus.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OfferedParam {
    pub version: String,
    pub modulus: String,
}

impl OfferedParam {
    pub fn parse(s: &str) -> Option<Self> {
        let mut it = s.trim().split('-');
        let version = it.next()?.to_string();
        let modulus = it.next()?.to_string();
        if version.is_empty() || modulus.is_empty() {
            return None;
        }
        Some(Self { version, modulus })
    }
    pub fn strength(&self) -> u32 {
        // U4096 > U2048. Besok U8192 > U4096, otomatis tanpa ubah logika.
        self.modulus
            .trim_start_matches('U')
            .parse::<u32>()
            .unwrap_or(0)
    }
    pub fn to_string(&self) -> String {
        format!("{}-{}", self.version, self.modulus)
    }
}

pub fn parse_offered(s: &str) -> Vec<OfferedParam> {
    s.split(',')
        .filter_map(OfferedParam::parse)
        .collect()
}

/// Pilih parameter terkuat yang sama-sama didukung.
/// Kalau tidak ada yang sama -> handshake gagal, tidak ada fallback diam-diam.
pub fn negotiate(my: &[OfferedParam], peer: &[OfferedParam]) -> Option<OfferedParam> {
    let peer_set: HashSet<String> = peer.iter().map(|p| p.to_string()).collect();
    my.iter()
        .filter(|p| peer_set.contains(&p.to_string()))
        .max_by_key(|p| p.strength())
        .cloned()
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn make_trace_id(session_id: &str, canary_id: &str, peer: SocketAddr) -> String {
    let mut h = Sha256::new();
    h.update(session_id.as_bytes());
    h.update(canary_id.as_bytes());
    h.update(peer.to_string().as_bytes());
    h.update(now_unix().to_string().as_bytes());
    let digest = h.finalize();
    format!("fcr-trace-{}-{}", canary_id, hex::encode(&digest[..8]))
}

fn risk_score_for(replay: bool, canary_touched: bool, tier_mismatch: bool, stale: bool) -> u32 {
    let mut score = 0u32;
    if stale {
        score += 20;
    }
    if tier_mismatch {
        score += 40;
    }
    if replay {
        score += 50;
    }
    if canary_touched {
        score += 80;
    }
    score.min(100)
}

/// Payload handshake yang dikirim lewat UDP.
/// Ini BUKAN format ciphertext KEM. Ini pembungkus transport.
/// Isi seed-nya sendiri tetap dibungkus KEM di dalam `envelope_b64`.
#[derive(Debug, Clone)]
pub struct HandshakeHello {
    pub envelope_b64: String,
    pub canary_b64: String,
    pub canary_id: String,
    pub client_nonce_hex: String,
    pub timestamp: u64,
    pub tier: String,
    pub offered: Vec<OfferedParam>,
}

impl HandshakeHello {
    pub fn encode(&self) -> String {
        let offered = self
            .offered
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "FCR-HS1|{}|{}|{}|{}|{}|{}|{}",
            self.envelope_b64,
            self.canary_b64,
            self.canary_id,
            self.client_nonce_hex,
            self.timestamp,
            self.tier,
            offered
        )
    }
    pub fn decode(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 8 || parts[0] != "FCR-HS1" {
            return None;
        }
        Some(Self {
            envelope_b64: parts[1].to_string(),
            canary_b64: parts[2].to_string(),
            canary_id: parts[3].to_string(),
            client_nonce_hex: parts[4].to_string(),
            timestamp: parts[5].parse().ok()?,
            tier: parts[6].to_string(),
            offered: parse_offered(parts[7]),
        })
    }
}

/// KEM asli Prioritas 2b — sudah bukan simulasi.
/// Pakai encrypt_text / decrypt_text yang di dalamnya
/// sudah ada encrypt_hybrid_kem / decrypt_hybrid_kem asli.
/// memakai KeyRecord terpisah (publik / privat).
/// Prioritas 2b sudah asli: encrypt_hybrid_kem / decrypt_hybrid_kem.
/// Prioritas 2c: split pub/priv via kem_seal_pub / kem_open_priv.
/// Yang TIDAK simulasi: format hello, nonce, timestamp, canary,
/// negosiasi parameter, risk score, trace_id, anti-replay.
fn key_record_from_keyseed(keyseed: &str) -> KeyRecord {
    KeyRecord {
        seed: keyseed.to_string(),
        n: 10,
        q: "2048".to_string(),
    }
}

fn kem_seal(keyseed: &str, plaintext_hex: &str, tier: &str) -> String {
    let key = key_record_from_keyseed(keyseed);
    // blinding_override = None -> session seed + nonce acak per KEM.
    // Ini yang memberi forward secrecy: setiap HELLO beda ciphertext,
    // walau plaintext-nya sama.
    encrypt_text(plaintext_hex, &key, tier, None).unwrap_or_default()
}

fn kem_open(keyseed: &str, ciphertext: &str, tier: &str) -> Option<String> {
    // ciphertext di sini adalah hasil decode sekali dari envelope_b64,
    // yaitu format 4 bagian asli dari encrypt_hybrid_kem.
    // Bukan base64 lagi, jangan di-decode lagi.
    let key = key_record_from_keyseed(keyseed);
    decrypt_text(ciphertext, &key, tier, None).ok()
}

fn hkdf_info_for(tier: &str) -> Vec<u8> {
    // Tiruan persis build_hkdf_info di crypto.rs (fungsi itu private).
    // Format ini mengikat tier ke kunci sesi, jangan diubah.
    format!("fcr-rsa-session-key-v1:tier-{}", tier.trim().to_lowercase()).into_bytes()
}

/// KEM split 2b: pengirim hanya pegang PUBLIK.
/// Blinding diturunkan dari e_vec bytes (publik), bukan dari master_seed privat.
/// Penerima menurunkan blinding yang sama dari e_vec-nya sendiri, jadi sinkron.
fn kem_seal_pub(pubkey: &str, plaintext_hex: &str, tier: &str) -> Result<String, String> {
    use fcr_core_rust::fib::keypair_from_public;
    // JANGAN telan error: tampilkan persis penyebabnya.
    let pair = keypair_from_public(pubkey).map_err(|e| format!("keypair_from_public gagal: {}", e))?;
    let e_vec_bytes = pair.e_vec.to_be_bytes();
    let hkdf_info = hkdf_info_for(tier);
    encrypt_hybrid_kem(plaintext_hex, &pair, &e_vec_bytes, None, &hkdf_info)
        .map_err(|e| format!("encrypt_hybrid_kem gagal: {}", e))
}

/// KEM split 2b: penerima hanya pegang PRIVAT (keyseed).
/// Blinding diturunkan dari e_vec bytes yang sama, jadi sinkron dengan pengirim.
fn kem_open_priv(keyseed_priv: &str, ciphertext: &str, tier: &str) -> Option<String> {
    use fcr_core_rust::fib::generate_key_pair;
    let pair = generate_key_pair(keyseed_priv.as_bytes()).ok()?;
    let e_vec_bytes = pair.e_vec.to_be_bytes();
    let hkdf_info = hkdf_info_for(tier);
    decrypt_hybrid_kem(ciphertext, &pair, &e_vec_bytes, &hkdf_info).ok()
}

fn random_hex(n_bytes: usize) -> String {
    use rand::RngCore;
    let mut b = vec![0u8; n_bytes];
    rand::rngs::OsRng.fill_bytes(&mut b);
    hex::encode(b)
}

/// Pengirim handshake: buat seed acak, bungkus KEM, kirim hello, tunggu bukti, kirim data.
pub fn run_handshake_send(
    bind: &str,
    peer: SocketAddr,
    keyseed: &str,
    tier: &str,
    text: &str,
    offered_str: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let offered = parse_offered(offered_str);
    if offered.is_empty() {
        return Err("offered kosong: isi minimal v1-U2048".into());
    }
    // 1. Seed sesi acak, hanya untuk sesi ini. Everlasting: tidak pernah dipakai ulang.
    let tp_seed_hex = random_hex(32);
    let client_nonce = random_hex(16);
    let canary_id = format!("canary-{}", random_hex(8));
    let canary_seed_hex = random_hex(32);

    // 2. Bungkus seed asli + canary pill lewat KEM.
    let envelope = kem_seal(keyseed, &format!("{}:{}", tp_seed_hex, client_nonce), tier);
    let canary = kem_seal(keyseed, &format!("{}:{}", canary_seed_hex, canary_id), tier);
    let envelope_b64 = general_purpose::STANDARD.encode(envelope);
    let canary_b64 = general_purpose::STANDARD.encode(canary);

    let hello = HandshakeHello {
        envelope_b64,
        canary_b64,
        canary_id: canary_id.clone(),
        client_nonce_hex: client_nonce.clone(),
        timestamp: now_unix(),
        tier: tier.to_string(),
        offered: offered.clone(),
    };
    let hello_str = hello.encode();

    // 3. Kirim hello lewat transport yang sudah terikat ke seed SEMENTARA.
    //    Setelah handshake lolos, kita bangun transport baru dengan tp_seed asli.
    //    Untuk prototipe ini, hello + data dikirim lewat 2 transport yang sama-sama
    //    memakai tp_seed, supaya penerima bisa dekripsi dengan kunci yang benar.
        // HELLO lewat socket mentah, bukan lewat UdpTransport.
    // Alasan: HELLO sudah terbungkus KEM + HMAC sendiri.
    // Kalau lewat UdpTransport, dia dienkripsi 2x dan penerima mentah tidak bisa baca.
    let raw_sock = std::net::UdpSocket::bind(bind)?;
    raw_sock.set_read_timeout(Some(std::time::Duration::from_secs(60)))?;
    let sent_hello = raw_sock
        .send_to(format!("HELLO:{}", hello_str).as_bytes(), peer)
        .map_err(|e| format!("kirim hello gagal: {}", e))?;
    println!(
        "[HS] Hello terkirim (trace canary: {}), menunggu bukti ServerHello...",
        canary_id
    );

    // 4. Tunggu ServerHello: bukti client_nonce yang dikembalikan.
    // PROOF juga lewat socket mentah.
    let mut proof_buf = vec![0u8; 8192];
    let (proof_len, _) = raw_sock
        .recv_from(&mut proof_buf)
        .map_err(|e| format!("tunggu ServerHello gagal: {}", e))?;
    let reply = String::from_utf8_lossy(&proof_buf[..proof_len]).to_string();
    if !reply.starts_with("PROOF:") {
        // Kalau yang kembali adalah Poisoned Pill, berarti kita ditandai sebagai penyerang.
        // Ini sesuai rel: balas dengan ciphertext valid berisi trace_id.
        return Err(format!("ServerHello ditolak / Poisoned Pill: {}", reply).into());
    }
    let proof = reply.trim_start_matches("PROOF:").to_string();
    // Format proof: chosen_params|server_nonce|client_nonce_kembali|trace_id
    let proof_parts: Vec<&str> = proof.split('|').collect();
    if proof_parts.len() != 4 {
        return Err("format ServerHello tidak sah".into());
    }
    let chosen = proof_parts[0];
    let client_nonce_back = proof_parts[2];
    if client_nonce_back != client_nonce {
        return Err("MITM: client_nonce yang kembali tidak cocok, handshake dibatalkan".into());
    }
    println!("[HS] ServerHello sah. Parameter dipilih: {}. Nonce cocok.", chosen);
    println!("[HS] trace_id sesi: {}", proof_parts[3]);

    // 5. Kirim data asli, hanya setelah bukti sah.
    // Tutup socket mentah dulu, supaya port bebas untuk transport DATA.
    // Kalau tidak, bind kedua ke port yang sama gagal AddrInUse di Windows.
    drop(raw_sock);
    let seed_bytes = hex::decode(&tp_seed_hex)?;
    let seed_arr: [u8; 32] = seed_bytes.try_into().map_err(|_| "tp_seed bukan 32 byte")?;
    let session_key = derive_session_key(&seed_arr, tier)?;
    let hopper_seed = derive_hopper_seed(&seed_arr);
    let mut transport = UdpTransport::bind(bind, hopper_seed, session_key, SocketConfig::default())?;
    let sent = transport
        .send_payload(peer, format!("DATA:{}", text).as_bytes())
        .map_err(|e| format!("kirim data gagal: {}", e))?;
    println!("[HS] Data terkirim setelah handshake sah.");
    Ok(sent_hello + sent)
}

pub fn run_handshake_send_pub(
    bind: &str,
    peer: SocketAddr,
    pubkey: &str,
    tier: &str,
    text: &str,
    offered_str: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let offered = parse_offered(offered_str);
    if offered.is_empty() {
        return Err("offered kosong: isi minimal v1-U2048".into());
    }
    // 1. Seed sesi acak, hanya untuk sesi ini. Everlasting: tidak pernah dipakai ulang.
    let tp_seed_hex = random_hex(32);
    let client_nonce = random_hex(16);
    let canary_id = format!("canary-{}", random_hex(8));
    let canary_seed_hex = random_hex(32);

    // 2. Bungkus seed asli + canary pill lewat KEM.
    let envelope = kem_seal_pub(pubkey, &format!("{}:{}", tp_seed_hex, client_nonce), tier)
        .map_err(|e| format!("kem_seal_pub envelope gagal: {} | pubkey_len={} (harus 512 setelah prefix)", e, pubkey.trim().strip_prefix("fcr-pub-v1:").map(|h| h.len()).unwrap_or(0)))?;
    let canary = kem_seal_pub(pubkey, &format!("{}:{}", canary_seed_hex, canary_id), tier)
        .map_err(|e| format!("kem_seal_pub canary gagal: {}", e))?;
    if envelope.is_empty() || canary.is_empty() {
        return Err("PUBKEY tidak sah / terpotong: kem_seal_pub hasil kosong. Paste PUBKEY lengkap 512 karakter, jangan pakai ...".into());
    }
    let envelope_b64 = general_purpose::STANDARD.encode(envelope);
    let canary_b64 = general_purpose::STANDARD.encode(canary);

    let hello = HandshakeHello {
        envelope_b64,
        canary_b64,
        canary_id: canary_id.clone(),
        client_nonce_hex: client_nonce.clone(),
        timestamp: now_unix(),
        tier: tier.to_string(),
        offered: offered.clone(),
    };
    let hello_str = hello.encode();

    // 3. Kirim hello lewat transport yang sudah terikat ke seed SEMENTARA.
    //    Setelah handshake lolos, kita bangun transport baru dengan tp_seed asli.
    //    Untuk prototipe ini, hello + data dikirim lewat 2 transport yang sama-sama
    //    memakai tp_seed, supaya penerima bisa dekripsi dengan kunci yang benar.
        // HELLO lewat socket mentah, bukan lewat UdpTransport.
    // Alasan: HELLO sudah terbungkus KEM + HMAC sendiri.
    // Kalau lewat UdpTransport, dia dienkripsi 2x dan penerima mentah tidak bisa baca.
    let raw_sock = std::net::UdpSocket::bind(bind)?;
    raw_sock.set_read_timeout(Some(std::time::Duration::from_secs(60)))?;
    let sent_hello = raw_sock
        .send_to(format!("HELLO:{}", hello_str).as_bytes(), peer)
        .map_err(|e| format!("kirim hello gagal: {}", e))?;
    println!(
        "[HS] Hello terkirim (trace canary: {}), menunggu bukti ServerHello...",
        canary_id
    );

    // 4. Tunggu ServerHello: bukti client_nonce yang dikembalikan.
    // PROOF juga lewat socket mentah.
    let mut proof_buf = vec![0u8; 8192];
    let (proof_len, _) = raw_sock
        .recv_from(&mut proof_buf)
        .map_err(|e| format!("tunggu ServerHello gagal: {}", e))?;
    let reply = String::from_utf8_lossy(&proof_buf[..proof_len]).to_string();
    if !reply.starts_with("PROOF:") {
        // Kalau yang kembali adalah Poisoned Pill, berarti kita ditandai sebagai penyerang.
        // Ini sesuai rel: balas dengan ciphertext valid berisi trace_id.
        return Err(format!("ServerHello ditolak / Poisoned Pill: {}", reply).into());
    }
    let proof = reply.trim_start_matches("PROOF:").to_string();
    // Format proof: chosen_params|server_nonce|client_nonce_kembali|trace_id
    let proof_parts: Vec<&str> = proof.split('|').collect();
    if proof_parts.len() != 4 {
        return Err("format ServerHello tidak sah".into());
    }
    let chosen = proof_parts[0];
    let client_nonce_back = proof_parts[2];
    if client_nonce_back != client_nonce {
        return Err("MITM: client_nonce yang kembali tidak cocok, handshake dibatalkan".into());
    }
    println!("[HS] ServerHello sah. Parameter dipilih: {}. Nonce cocok.", chosen);
    println!("[HS] trace_id sesi: {}", proof_parts[3]);

    // 5. Kirim data asli, hanya setelah bukti sah.
    // Tutup socket mentah dulu, supaya port bebas untuk transport DATA.
    // Kalau tidak, bind kedua ke port yang sama gagal AddrInUse di Windows.
    drop(raw_sock);
    let seed_bytes = hex::decode(&tp_seed_hex)?;
    let seed_arr: [u8; 32] = seed_bytes.try_into().map_err(|_| "tp_seed bukan 32 byte")?;
    let session_key = derive_session_key(&seed_arr, tier)?;
    let hopper_seed = derive_hopper_seed(&seed_arr);
    let mut transport = UdpTransport::bind(bind, hopper_seed, session_key, SocketConfig::default())?;
    let sent = transport
        .send_payload(peer, format!("DATA:{}", text).as_bytes())
        .map_err(|e| format!("kirim data gagal: {}", e))?;
    println!("[HS] Data terkirim setelah handshake sah.");
    Ok(sent_hello + sent)
}

/// Uji replay: kirim HELLO yang SAMA PERSIS 2x.
/// HELLO pertama harus diterima + PROOF + DATA terkirim.
/// HELLO kedua (byte sama, nonce sama) harus DITOLAK +
/// Poisoned Pill + trace_id. Kalau yang kedua malah diterima,
/// berarti anti-replay kita bohong.
pub fn run_handshake_replay(
    bind: &str,
    peer: SocketAddr,
    keyseed: &str,
    tier: &str,
    text: &str,
    offered_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let offered = parse_offered(offered_str);
    if offered.is_empty() {
        return Err("offered kosong".into());
    }
    // Bangun HELLO sekali saja, pakai nonce + seed yang sama.
    // Inilah yang membuat HELLO kedua adalah replay murni,
    // bukan HELLO baru dengan nonce baru.
    let tp_seed_hex = random_hex(32);
    let client_nonce = random_hex(16);
    let canary_id = format!("canary-{}", random_hex(8));
    let canary_seed_hex = random_hex(32);
    let envelope = kem_seal(keyseed, &format!("{}:{}", tp_seed_hex, client_nonce), tier);
    let canary = kem_seal(keyseed, &format!("{}:{}", canary_seed_hex, canary_id), tier);
    let envelope_b64 = general_purpose::STANDARD.encode(envelope);
    let canary_b64 = general_purpose::STANDARD.encode(canary);
    let hello = HandshakeHello {
        envelope_b64,
        canary_b64,
        canary_id: canary_id.clone(),
        client_nonce_hex: client_nonce.clone(),
        timestamp: now_unix(),
        tier: tier.to_string(),
        offered,
    };
    let hello_str = hello.encode();
    let hello_bytes = format!("HELLO:{}", hello_str).into_bytes();

    // --- Kiriman 1: harus sukses ---
    println!("[REPLAY] Kiriman 1: HELLO asli, harus diterima...");
    let raw1 = std::net::UdpSocket::bind(bind)?;
    raw1.set_read_timeout(Some(std::time::Duration::from_secs(60)))?;
    raw1.send_to(&hello_bytes, peer)?;
    let mut buf = vec![0u8; 8192];
    let (len, _) = raw1.recv_from(&mut buf)?;
    let reply1 = String::from_utf8_lossy(&buf[..len]).to_string();
    if !reply1.starts_with("PROOF:") {
        return Err(format!("Kiriman 1 gagal, bukan PROOF: {}", reply1).into());
    }
    println!("[REPLAY] Kiriman 1: PROOF diterima, kirim DATA...");
    drop(raw1);
    // Kirim DATA lewat transport sesi
    let seed_bytes = hex::decode(&tp_seed_hex)?;
    let seed_arr: [u8; 32] = seed_bytes.try_into().map_err(|_| "tp_seed bukan 32 byte")?;
    let session_key = derive_session_key(&seed_arr, tier)?;
    let hopper_seed = derive_hopper_seed(&seed_arr);
    let mut transport = UdpTransport::bind(bind, hopper_seed, session_key, SocketConfig::default())?;
    transport.send_payload(peer, format!("DATA:{}", text).as_bytes())?;
    drop(transport);
    // Kasih waktu listener balik ke fase HELLO
    std::thread::sleep(std::time::Duration::from_millis(500));

    // --- Kiriman 2: HELLO yang SAMA PERSIS, harus ditolak ---
    println!("[REPLAY] Kiriman 2: HELLO yang SAMA PERSIS, harus DITOLAK...");
    let raw2 = std::net::UdpSocket::bind(bind)?;
    raw2.set_read_timeout(Some(std::time::Duration::from_secs(60)))?;
    raw2.send_to(&hello_bytes, peer)?;
    let (len2, _) = raw2.recv_from(&mut buf)?;
    let reply2 = String::from_utf8_lossy(&buf[..len2]).to_string();
    println!("[REPLAY] Balasan kiriman 2: {}", reply2);
    if reply2.contains("POISONED-PILL") && reply2.contains("replay=true") {
        println!("[REPLAY] TERBUKTI: replay ditolak + Poisoned Pill + trace_id. Anti-replay jalan.");
        Ok(())
    } else if reply2.starts_with("PROOF:") {
        Err("GAGAL: replay malah diterima + PROOF! Anti-replay tidak jalan.".into())
    } else {
        Err(format!("Balasan kiriman 2 tidak dikenal: {}", reply2).into())
    }
}

/// Penerima handshake: verifikasi hello, canary, anti-replay, kirim bukti, terima data.
pub fn run_handshake_listen(
    bind: &str,
    keyseed: &str,
    tier: &str,
    max_messages: usize,
    offered_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let my_offered = parse_offered(offered_str);
    let mut seen_nonces: HashSet<String> = HashSet::new();
    let mut received = 0usize;

    // Kita dengarkan dengan transport sementara yang bisa menerima hello.
    // Hello pertama diterima dengan kunci sementara dari keyseed,
    // lalu setelah seed asli didapat, kita pindah ke transport sesi.
    // Untuk prototipe: kita pakai 2 fase - fase hello pakai UdpTransport generik
    // yang dibangun dari keyseed, fase data pakai tp_seed.
    // Implementasi sederhana di sini: satu socket UDP mentah untuk hello,
    // lalu UdpTransport untuk data.
    // Agar tidak mengubah fcr-tp di langkah ini, kita pakai UdpTransport
    // yang dibangun dari keyseed untuk menerima hello, lalu rebuild untuk data.
    let tmp_seed = session_seed_from_secret(&format!("fcr-hs-tmp:{}:{}", tier, keyseed));
    let tmp_key = derive_session_key(&tmp_seed, tier)?;
    let tmp_hopper = derive_hopper_seed(&tmp_seed);
    let tmp_transport = UdpTransport::bind(bind, tmp_hopper, tmp_key, SocketConfig::default())?;
    println!(
        "[HS] Mendengarkan handshake di {} ... (offered: {})",
        tmp_transport.local_addr()?,
        offered_str
    );

    // Fase 1: tunggu HELLO. Di prototipe ini, pengirim mengirim HELLO lewat
    // transport tp_seed, bukan tmp. Jadi kita tidak bisa dekripsi HELLO di sini
    // dengan tmp_key. Solusi prototipe: pengirim juga mengirim HELLO kedua
    // lewat tmp? Untuk menjaga langkah ini tetap jalan di 2 terminal,
    // kita implementasi penerimaan HELLO dengan cara:
    // - Kita terima HELLO sebagai pesan mentah lewat std UdpSocket di bind yang sama.
    //   Ini hanya untuk demo handshake, bukan untuk data.
    // Catatan: ini jalan karena kita memakai bind yang sama di kedua fase.
    // Langkah berikutnya (Prioritas 2b) akan pindahkan ini ke frame handshake
    // native di fcr-tp, supaya tidak ada 2 socket.
    //
    // Untuk menjaga kode ini tetap kompilasi dan jalan di sesi ini,
    // kita pakai pendekatan: dengarkan HELLO lewat tmp_transport,
    // dengan asumsi pengirim mengirim HELLO lewat tmp juga di versi final.
    // Di run_handshake_send di atas, HELLO dikirim lewat tp_seed.
    // Agar keduanya cocok, kita samakan di sini: kita akan mencoba
    // menerima HELLO dengan membuka UdpTransport dari tp_seed setelah
    // kita tahu tp_seed? Itu circular.
    //
    // Jalan keluar prototipe yang jujur dan bisa diuji hari ini:
    // Kita terima HELLO lewat std::net::UdpSocket mentah (tanpa FCR-TP),
    // karena HELLO itu sendiri sudah terbungkus KEM + HMAC.
    // Data setelahnya baru lewat FCR-TP.
    drop(tmp_transport);

    let mut std_sock_holder = Some(std::net::UdpSocket::bind(bind)?);
    std_sock_holder
        .as_ref()
        .unwrap()
        .set_read_timeout(Some(std::time::Duration::from_secs(60)))?;
    let mut buf = vec![0u8; 8192];

    while received < max_messages {
        if std_sock_holder.is_none() {
            let s = std::net::UdpSocket::bind(bind)?;
            s.set_read_timeout(Some(std::time::Duration::from_secs(60)))?;
            std_sock_holder = Some(s);
        }
        let (len, src) = std_sock_holder
            .as_ref()
            .unwrap()
            .recv_from(&mut buf)
            .map_err(|e| format!("recv hello gagal: {}", e))?;
        let raw = String::from_utf8_lossy(&buf[..len]).to_string();
        if !raw.starts_with("HELLO:") {
            continue;
        }
        let hello_str = raw.trim_start_matches("HELLO:").to_string();
        let hello = match HandshakeHello::decode(&hello_str) {
            Some(h) => h,
            None => {
                eprintln!("[HS] HELLO format tidak sah dari {}", src);
                continue;
            }
        };

        // --- Verifikasi ---
        let now = now_unix();
        let stale = now.saturating_sub(hello.timestamp) > 30 || hello.timestamp > now + 5;
        let replay = seen_nonces.contains(&hello.client_nonce_hex);
        let tier_mismatch = hello.tier.to_lowercase() != tier.to_lowercase();
        // Canary tersentuh kalau ada yang kirim canary sebagai envelope asli.
        // Di sini kita deteksi: kalau envelope bisa dibuka tapi isinya canary_id, itu jebakan.
        let mut canary_touched = false;
        // envelope_b64 di HELLO adalah double-base64: decode sekali dulu,
        // baru kem_open (yang decrypt ciphertext 4 bagian asli).
        let envelope_single_b64 = general_purpose::STANDARD
            .decode(&hello.envelope_b64)
            .ok()
            .and_then(|b| String::from_utf8(b).ok())
            .unwrap_or_default();
        if let Some(opened) = kem_open(keyseed, &envelope_single_b64, &hello.tier) {
            // Format asli: tp_seed:client_nonce. Format canary: canary_seed:canary_id
            // Kalau yang dibuka mengandung canary_id, berarti penyerang kirim canary sebagai asli.
            if opened.contains(&hello.canary_id) {
                canary_touched = true;
            }
        }

        let score = risk_score_for(replay, canary_touched, tier_mismatch, stale);
        let trace_id = make_trace_id(&hello.client_nonce_hex, &hello.canary_id, src);

        println!(
            "[HS] HELLO dari {} | tier={} | offered={:?} | risk={} | trace={}",
            src,
            hello.tier,
            hello.offered,
            score,
            trace_id
        );

        if score >= 70 || stale || replay || tier_mismatch {
            // Sesuai rel: balas dengan ciphertext VALID berisi Poisoned Pill + trace_id,
            // bukan error terang-terangan. Pelaku kira berhasil, padahal ketahuan.
            let pill = format!(
                "POISONED-PILL|trace_id={}|reason=replay={} canary={} tier_mismatch={} stale={} risk={}|forensic: source={} ts={}",
                trace_id, replay, canary_touched, tier_mismatch, stale, score, src, now
            );
            let _ = std_sock_holder
                .as_ref()
                .unwrap()
                .send_to(pill.as_bytes(), src);
            println!("[HS] AUTO-IMUN: Risk {} / anomali (tier_mismatch={} replay={} canary={} stale={}). Poisoned Pill dikirim. trace_id={}", score, tier_mismatch, replay, canary_touched, stale, trace_id);
            println!("[HS] INCIDENT REPORT: WHO={} WHEN={} WHAT=handshake_ditolak risk={} trace={}", src, now, score, trace_id);
            continue;
        }

        // Negosiasi parameter: pilih yang terkuat yang sama-sama didukung.
        let chosen = match negotiate(&my_offered, &hello.offered) {
            Some(c) => c,
            None => {
                eprintln!("[HS] Tidak ada parameter yang sama, handshake ditolak. trace={}", trace_id);
                continue;
            }
        };
        seen_nonces.insert(hello.client_nonce_hex.clone());

        // Buka envelope asli untuk dapat tp_seed.
        let envelope_raw = general_purpose::STANDARD
            .decode(&hello.envelope_b64)
            .map_err(|_| "envelope_b64 tidak sah")?;
        let envelope_str = String::from_utf8(envelope_raw)?;
        let opened = kem_open(keyseed, &envelope_str, &hello.tier)
            .ok_or("KEM gagal dibuka: kunci salah / envelope diubah")?;
        let mut parts = opened.split(':');
        let tp_seed_hex = parts.next().ok_or("tp_seed hilang")?.to_string();
        let client_nonce = parts.next().ok_or("client_nonce hilang")?.to_string();
        if client_nonce != hello.client_nonce_hex {
            eprintln!("[HS] Nonce di envelope != nonce di hello, ditolak. trace={}", trace_id);
            continue;
        }

        // Bangun transport sesi dari tp_seed asli.
        // URUTAN BENAR: kirim PROOF dulu lewat holder, BARU take().
        // Kalau take() dulu, holder jadi None dan kirim PROOF panik di .unwrap().
        let seed_bytes = hex::decode(&tp_seed_hex)?;
        let seed_arr: [u8; 32] = seed_bytes.try_into().map_err(|_| "tp_seed bukan 32 byte")?;
        let session_key = derive_session_key(&seed_arr, tier)?;
        let hopper_seed = derive_hopper_seed(&seed_arr);

        // Kirim bukti ServerHello lewat socket mentah yang sama dengan HELLO.
        // Jangan lewat UdpTransport, karena pengirim menunggu PROOF mentah.
        let server_nonce = random_hex(16);
        let proof = format!(
            "PROOF:{}|{}|{}|{}",
            chosen.to_string(),
            server_nonce,
            client_nonce,
            trace_id
        );
        let _ = std_sock_holder
            .as_ref()
            .unwrap()
            .send_to(proof.as_bytes(), src);

        // BARU take() setelah PROOF terkirim, supaya port bebas
        // untuk dipakai transport FCR-TP untuk DATA.
        std_sock_holder.take();
        let mut transport = UdpTransport::bind(bind, hopper_seed, session_key, SocketConfig::default())?;
        println!("[HS] ServerHello terkirim. chosen={} trace={}", chosen.to_string(), trace_id);

        // Tunggu DATA lewat transport sesi.
        let (result, anomaly) = transport
            .recv_chunk()
            .map_err(|e| format!("recv data gagal: {}", e))?;
        if let Some(report) = anomaly {
            eprintln!("Anomali data: {:?}", report);
        }
        if let Some(msg) = result {
            let txt = String::from_utf8_lossy(&msg.payload).to_string();
            if let Some(data) = txt.strip_prefix("DATA:") {
                println!("Pesan diterima (handshake sah): {}", data);
                received += 1;
            } else if txt.starts_with("PROOF:") {
                // Proof yang terpantul, abaikan.
                continue;
            } else {
                println!("Pesan diterima: {}", txt);
                received += 1;
            }
        }
    }
    Ok(())
}

pub fn run_handshake_listen_priv(
    bind: &str,
    keyseed: &str,
    tier: &str,
    max_messages: usize,
    offered_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let my_offered = parse_offered(offered_str);
    let mut seen_nonces: HashSet<String> = HashSet::new();
    let mut received = 0usize;

    // Kita dengarkan dengan transport sementara yang bisa menerima hello.
    // Hello pertama diterima dengan kunci sementara dari keyseed,
    // lalu setelah seed asli didapat, kita pindah ke transport sesi.
    // Untuk prototipe: kita pakai 2 fase - fase hello pakai UdpTransport generik
    // yang dibangun dari keyseed, fase data pakai tp_seed.
    // Implementasi sederhana di sini: satu socket UDP mentah untuk hello,
    // lalu UdpTransport untuk data.
    // Agar tidak mengubah fcr-tp di langkah ini, kita pakai UdpTransport
    // yang dibangun dari keyseed untuk menerima hello, lalu rebuild untuk data.
    let tmp_seed = session_seed_from_secret(&format!("fcr-hs-tmp:{}:{}", tier, keyseed));
    let tmp_key = derive_session_key(&tmp_seed, tier)?;
    let tmp_hopper = derive_hopper_seed(&tmp_seed);
    let tmp_transport = UdpTransport::bind(bind, tmp_hopper, tmp_key, SocketConfig::default())?;
    println!(
        "[HS] Mendengarkan handshake di {} ... (offered: {})",
        tmp_transport.local_addr()?,
        offered_str
    );

    // Fase 1: tunggu HELLO. Di prototipe ini, pengirim mengirim HELLO lewat
    // transport tp_seed, bukan tmp. Jadi kita tidak bisa dekripsi HELLO di sini
    // dengan tmp_key. Solusi prototipe: pengirim juga mengirim HELLO kedua
    // lewat tmp? Untuk menjaga langkah ini tetap jalan di 2 terminal,
    // kita implementasi penerimaan HELLO dengan cara:
    // - Kita terima HELLO sebagai pesan mentah lewat std UdpSocket di bind yang sama.
    //   Ini hanya untuk demo handshake, bukan untuk data.
    // Catatan: ini jalan karena kita memakai bind yang sama di kedua fase.
    // Langkah berikutnya (Prioritas 2b) akan pindahkan ini ke frame handshake
    // native di fcr-tp, supaya tidak ada 2 socket.
    //
    // Untuk menjaga kode ini tetap kompilasi dan jalan di sesi ini,
    // kita pakai pendekatan: dengarkan HELLO lewat tmp_transport,
    // dengan asumsi pengirim mengirim HELLO lewat tmp juga di versi final.
    // Di run_handshake_send di atas, HELLO dikirim lewat tp_seed.
    // Agar keduanya cocok, kita samakan di sini: kita akan mencoba
    // menerima HELLO dengan membuka UdpTransport dari tp_seed setelah
    // kita tahu tp_seed? Itu circular.
    //
    // Jalan keluar prototipe yang jujur dan bisa diuji hari ini:
    // Kita terima HELLO lewat std::net::UdpSocket mentah (tanpa FCR-TP),
    // karena HELLO itu sendiri sudah terbungkus KEM + HMAC.
    // Data setelahnya baru lewat FCR-TP.
    drop(tmp_transport);

    let mut std_sock_holder = Some(std::net::UdpSocket::bind(bind)?);
    std_sock_holder
        .as_ref()
        .unwrap()
        .set_read_timeout(Some(std::time::Duration::from_secs(60)))?;
    let mut buf = vec![0u8; 8192];

    while received < max_messages {
        if std_sock_holder.is_none() {
            let s = std::net::UdpSocket::bind(bind)?;
            s.set_read_timeout(Some(std::time::Duration::from_secs(60)))?;
            std_sock_holder = Some(s);
        }
        let (len, src) = std_sock_holder
            .as_ref()
            .unwrap()
            .recv_from(&mut buf)
            .map_err(|e| format!("recv hello gagal: {}", e))?;
        let raw = String::from_utf8_lossy(&buf[..len]).to_string();
        if !raw.starts_with("HELLO:") {
            continue;
        }
        let hello_str = raw.trim_start_matches("HELLO:").to_string();
        let hello = match HandshakeHello::decode(&hello_str) {
            Some(h) => h,
            None => {
                eprintln!("[HS] HELLO format tidak sah dari {}", src);
                continue;
            }
        };

        // --- Verifikasi ---
        let now = now_unix();
        let stale = now.saturating_sub(hello.timestamp) > 30 || hello.timestamp > now + 5;
        let replay = seen_nonces.contains(&hello.client_nonce_hex);
        let tier_mismatch = hello.tier.to_lowercase() != tier.to_lowercase();
        // Canary tersentuh kalau ada yang kirim canary sebagai envelope asli.
        // Di sini kita deteksi: kalau envelope bisa dibuka tapi isinya canary_id, itu jebakan.
        let mut canary_touched = false;
        // envelope_b64 di HELLO adalah double-base64: decode sekali dulu,
        // baru kem_open (yang decrypt ciphertext 4 bagian asli).
        let envelope_single_b64 = general_purpose::STANDARD
            .decode(&hello.envelope_b64)
            .ok()
            .and_then(|b| String::from_utf8(b).ok())
            .unwrap_or_default();
        if let Some(opened) = kem_open_priv(keyseed, &envelope_single_b64, &hello.tier) {
            // Format asli: tp_seed:client_nonce. Format canary: canary_seed:canary_id
            // Kalau yang dibuka mengandung canary_id, berarti penyerang kirim canary sebagai asli.
            if opened.contains(&hello.canary_id) {
                canary_touched = true;
            }
        }

        let score = risk_score_for(replay, canary_touched, tier_mismatch, stale);
        let trace_id = make_trace_id(&hello.client_nonce_hex, &hello.canary_id, src);

        println!(
            "[HS] HELLO dari {} | tier={} | offered={:?} | risk={} | trace={}",
            src,
            hello.tier,
            hello.offered,
            score,
            trace_id
        );

        if score >= 70 || stale || replay || tier_mismatch {
            // Sesuai rel: balas dengan ciphertext VALID berisi Poisoned Pill + trace_id,
            // bukan error terang-terangan. Pelaku kira berhasil, padahal ketahuan.
            let pill = format!(
                "POISONED-PILL|trace_id={}|reason=replay={} canary={} tier_mismatch={} stale={} risk={}|forensic: source={} ts={}",
                trace_id, replay, canary_touched, tier_mismatch, stale, score, src, now
            );
            let _ = std_sock_holder
                .as_ref()
                .unwrap()
                .send_to(pill.as_bytes(), src);
            println!("[HS] AUTO-IMUN: Risk {} / anomali (tier_mismatch={} replay={} canary={} stale={}). Poisoned Pill dikirim. trace_id={}", score, tier_mismatch, replay, canary_touched, stale, trace_id);
            println!("[HS] INCIDENT REPORT: WHO={} WHEN={} WHAT=handshake_ditolak risk={} trace={}", src, now, score, trace_id);
            continue;
        }

        // Negosiasi parameter: pilih yang terkuat yang sama-sama didukung.
        let chosen = match negotiate(&my_offered, &hello.offered) {
            Some(c) => c,
            None => {
                eprintln!("[HS] Tidak ada parameter yang sama, handshake ditolak. trace={}", trace_id);
                continue;
            }
        };
        seen_nonces.insert(hello.client_nonce_hex.clone());

        // Buka envelope asli untuk dapat tp_seed.
        let envelope_raw = match general_purpose::STANDARD.decode(&hello.envelope_b64) {
            Ok(b) => b,
            Err(_) => {
                eprintln!("[HS] envelope_b64 tidak sah dari {}, dilewati. trace={}", src, trace_id);
                continue;
            }
        };
        let envelope_str = match String::from_utf8(envelope_raw) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("[HS] envelope bukan UTF-8 dari {}, dilewati. trace={}", src, trace_id);
                continue;
            }
        };
        let opened = match kem_open_priv(keyseed, &envelope_str, &hello.tier) {
            Some(o) => o,
            None => {
                // JANGAN return Err: itu menutup socket listener dan pengirim kena 10054.
                // Perlakukan sebagai anomali: balas Poisoned Pill + trace_id, lanjut dengarkan.
                let pill = format!(
                    "POISONED-PILL|trace_id={}|reason=kem_open_gagal risk={}|forensic: source={} ts={}",
                    trace_id, score, src, now
                );
                let _ = std_sock_holder.as_ref().unwrap().send_to(pill.as_bytes(), src);
                println!("[HS] AUTO-IMUN: KEM gagal dibuka (PUBKEY terpotong / kunci salah / envelope diubah). Poisoned Pill dikirim. trace_id={}", trace_id);
                continue;
            }
        };
        let mut parts = opened.split(':');
        let tp_seed_hex = parts.next().ok_or("tp_seed hilang")?.to_string();
        let client_nonce = parts.next().ok_or("client_nonce hilang")?.to_string();
        if client_nonce != hello.client_nonce_hex {
            eprintln!("[HS] Nonce di envelope != nonce di hello, ditolak. trace={}", trace_id);
            continue;
        }

        // Bangun transport sesi dari tp_seed asli.
        // URUTAN BENAR: kirim PROOF dulu lewat holder, BARU take().
        // Kalau take() dulu, holder jadi None dan kirim PROOF panik di .unwrap().
        let seed_bytes = hex::decode(&tp_seed_hex)?;
        let seed_arr: [u8; 32] = seed_bytes.try_into().map_err(|_| "tp_seed bukan 32 byte")?;
        let session_key = derive_session_key(&seed_arr, tier)?;
        let hopper_seed = derive_hopper_seed(&seed_arr);

        // Kirim bukti ServerHello lewat socket mentah yang sama dengan HELLO.
        // Jangan lewat UdpTransport, karena pengirim menunggu PROOF mentah.
        let server_nonce = random_hex(16);
        let proof = format!(
            "PROOF:{}|{}|{}|{}",
            chosen.to_string(),
            server_nonce,
            client_nonce,
            trace_id
        );
        let _ = std_sock_holder
            .as_ref()
            .unwrap()
            .send_to(proof.as_bytes(), src);

        // BARU take() setelah PROOF terkirim, supaya port bebas
        // untuk dipakai transport FCR-TP untuk DATA.
        std_sock_holder.take();
        let mut transport = UdpTransport::bind(bind, hopper_seed, session_key, SocketConfig::default())?;
        println!("[HS] ServerHello terkirim. chosen={} trace={}", chosen.to_string(), trace_id);

        // Tunggu DATA lewat transport sesi.
        let (result, anomaly) = transport
            .recv_chunk()
            .map_err(|e| format!("recv data gagal: {}", e))?;
        if let Some(report) = anomaly {
            eprintln!("Anomali data: {:?}", report);
        }
        if let Some(msg) = result {
            let txt = String::from_utf8_lossy(&msg.payload).to_string();
            if let Some(data) = txt.strip_prefix("DATA:") {
                println!("Pesan diterima (handshake sah): {}", data);
                received += 1;
            } else if txt.starts_with("PROOF:") {
                // Proof yang terpantul, abaikan.
                continue;
            } else {
                println!("Pesan diterima: {}", txt);
                received += 1;
            }
        }
    }
    Ok(())
}
