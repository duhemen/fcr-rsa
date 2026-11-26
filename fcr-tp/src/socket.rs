//! Lapisan socket UDP nyata untuk FCR-TP (Langkah 4).
//!
//! Ini SATU-SATUNYA tempat `source_port` / `source_ip` boleh berasal:
//! dari `UdpSocket::recv_from`, yaitu alamat peer yang benar-benar
//! mengirim datagram. Tidak ada lagi pengisian manual dari field
//! kiriman peer (lihat INTEGRATION_CONTRACT.md butir 4).
//!
//! Alur kirim:  payload FCR-RSA -> orchestrator.encrypt() -> frame.encode() -> send_to()
//! Alur terima: recv_from() -> FcrTpFrame::decode() -> orchestrator.decrypt()
//!               -> orchestrator.process_chunk_at() dengan forensic dari socket.

use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use bytes::Bytes;

use crate::packet::FcrTpFrame;
use crate::reassembler::{ForensicData, ReassemblyResult, ReassemblyError};
use crate::session::TransportOrchestrator;
use crate::anomaly::AnomalyReport;
use crate::crypto::ChunkMetadata;

/// Konfigurasi socket. `hop_distance_km`: jarak ke peer (0.0 = tidak dicek
/// fisika jarak; isi dari konfigurasi deployment, BUKAN dari peer).
#[derive(Debug, Clone, Copy)]
pub struct SocketConfig {
    pub hop_distance_km: f64,
}

impl Default for SocketConfig {
    fn default() -> Self { Self { hop_distance_km: 0.0 } }
}

/// Socket UDP + orchestrator dalam satu pegangan.
pub struct UdpTransport {
    socket: UdpSocket,
    pub orchestrator: TransportOrchestrator,
    config: SocketConfig,
}

impl UdpTransport {
    /// Bind ke alamat lokal (mis. `"0.0.0.0:0"` untuk port acak, atau
    /// `"127.0.0.1:50000"` untuk test). `session_key` = output HKDF FCR-RSA.
    pub fn bind(bind_addr: &str, seed: u64, session_key: [u8; 32], config: SocketConfig) -> io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr)?;
        Ok(Self { socket, orchestrator: TransportOrchestrator::from_session_key(seed, session_key), config })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> { self.socket.local_addr() }

    /// Kirim payload FCR-RSA utuh ke `peer`.
    ///
    /// PENTING: setiap frame dikirim dari PORT HOPPER-nya, bukan dari port
    /// acak OS. Itulah yang divalidasi penerima (`SpacetimeValidator`:
    /// `sample.port == expected_port`). Karena socket utama ter-bind ke
    /// port listen (`:0` acak), pengiriman memakai socket sementara yang
    /// di-bind ke port hopper untuk frame itu, lalu dibuang.
    /// `&mut self` karena hopper pengirim maju satu langkah per frame.
    pub fn send_payload(&mut self, peer: SocketAddr, payload: &[u8]) -> Result<usize, &'static str> {
        let frames = self.orchestrator.encrypt(payload)?;
        let local_ip = self.socket.local_addr().map(|a| a.ip()).map_err(|_| "UDP local_addr gagal")?;
        let mut sent = 0;
        for frame in &frames {
            let send_port = self.orchestrator.next_send_port();
            let bind_addr = SocketAddr::new(local_ip, send_port);
            let send_socket = UdpSocket::bind(bind_addr).map_err(|_| "UDP bind ke port hopper gagal (port dipakai?)")?;
            let wire = frame.encode()?;
            send_socket.send_to(&wire, peer).map_err(|_| "UDP send gagal")?;
            sent += 1;
        }
        Ok(sent)
    }

    /// Terima SATU datagram, dekripsi, dan dorong ke reassembler.
    /// `source_port`/`source_ip` diambil dari `recv_from` â€” bukan dari peer.
    /// Return: (hasil reassembly bila lengkap, laporan anomali bila ada).
    pub fn recv_chunk(&mut self) -> Result<(Option<ReassemblyResult>, Option<AnomalyReport>), &'static str> {
        let mut buf = [0u8; 1200];
        let (len, peer) = self.socket.recv_from(&mut buf).map_err(|_| "UDP recv gagal")?;
        let wire = Bytes::copy_from_slice(&buf[..len]);
        let frame = FcrTpFrame::decode(wire)?;
        let (metadata, data) = self.orchestrator.decrypt(&frame)?;
        self.process_decrypted(metadata, data, peer)
    }

    /// Varian test: terima datagram yang sudah dibaca dari socket lain.
    /// Tetap wajib melewatkan `peer` dari `recv_from` si penerima.
    pub fn process_wire(&mut self, wire: Bytes, peer: SocketAddr) -> Result<(Option<ReassemblyResult>, Option<AnomalyReport>), &'static str> {
        let frame = FcrTpFrame::decode(wire)?;
        let (metadata, data) = self.orchestrator.decrypt(&frame)?;
        self.process_decrypted(metadata, data, peer)
    }

    fn process_decrypted(&mut self, metadata: ChunkMetadata, data: Vec<u8>, peer: SocketAddr) -> Result<(Option<ReassemblyResult>, Option<AnomalyReport>), &'static str> {
        let source_ip = match peer.ip() {
            std::net::IpAddr::V4(v4) => Some(v4.octets()),
            std::net::IpAddr::V6(_) => None, // IPv6 belum didukung forensic v1
        };
        let forensic = ForensicData {
            trace_id: format!("{}-{}", self.orchestrator.session_id().iter().map(|b| format!("{:02x}", b)).collect::<String>(), metadata.message_id),
            session_id: self.orchestrator.session_id(),
            source_port: peer.port(), // <-- DARI SOCKET, bukan dari peer
            source_ip,
            timestamp: Instant::now(),
            rtt_ms: None,
            packet_loss: None,
        };
        let timestamp_ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0);
        self.orchestrator.process_chunk_at(&metadata, Bytes::from(data), forensic, self.config.hop_distance_km, timestamp_ms)
            .map_err(|e| match e { ReassemblyError::InvalidChunk => "InvalidChunk", ReassemblyError::Timeout { .. } => "Timeout", ReassemblyError::BufferFull => "BufferFull", ReassemblyError::IncompleteMessage { .. } => "IncompleteMessage", ReassemblyError::DuplicateMessage => "DuplicateMessage" })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_transports() -> (UdpTransport, UdpTransport) {
        // Dua pihak memakai kunci sesi HKDF yang SAMA (dari FCR-RSA).
        let key = [0xAA; 32];
        let a = UdpTransport::bind("127.0.0.1:0", 0xDEADBEEF, key, SocketConfig::default()).unwrap();
        let b = UdpTransport::bind("127.0.0.1:0", 0xDEADBEEF, key, SocketConfig::default()).unwrap();
        (a, b)
    }

    #[test]
    fn test_udp_loopback_roundtrip_proves_source_port_from_socket() {
        let (mut a, mut b) = two_transports();
        let peer_b = b.local_addr().unwrap();
        // Port hopper yang diharapkan penerima untuk chunk ini.
        let expected_port = a.orchestrator.expected_port();
        // a mengirim; b menerima. b TIDAK PERNAH diberi tahu port a secara manual.
        let sent = a.send_payload(peer_b, b"halo lewat socket nyata").unwrap();
        assert_eq!(sent, 1);
        // Set timeout agar test tidak menggantung bila gagal
        b.socket.set_read_timeout(Some(std::time::Duration::from_secs(2))).unwrap();
        let (result, _) = b.recv_chunk().unwrap();
        let result = result.expect("harus lengkap 1 chunk");
        assert_eq!(&result.payload[..], b"halo lewat socket nyata");
        // Bukti source_port = port hopper pengirim (bukan port acak OS,
        // bukan port listen penerima): sama dengan yang diharapkan penerima.
        assert_eq!(result.forensic_metadata.source_port, expected_port);
        assert_eq!(result.forensic_metadata.source_ip, Some([127, 0, 0, 1]));
    }

    #[test]
    fn test_spoofed_peer_port_is_rejected() {
        // Attacker meneruskan datagram asli tapi mengaku dari port lain.
        // process_wire WAJIB dipanggil dengan peer dari recv_from; di sini
        // kita simulasikan peer palsu dan buktikan validator menolaknya.
        let (a, mut b) = two_transports();
        let frames = a.orchestrator.encrypt(b"data").unwrap();
        let wire = frames[0].encode().unwrap();
        let fake_peer: SocketAddr = "127.0.0.1:9999".parse().unwrap();
        let result = b.process_wire(wire, fake_peer);
        assert!(result.is_err(), "port palsu harus ditolak SpacetimeValidator");
    }
}
