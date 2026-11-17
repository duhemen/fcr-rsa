# Limitations — Apa yang TIDAK Dijamin oleh FCR-RSA

Dokumen ini jujur tentang batasan FCR-RSA. Membacanya **sebelum** menggunakan
FCR-RSA adalah wajib.

## 1. Status Keamanan

- **Belum diaudit secara independen.** Semua klaim keamanan berasal dari
  pengujian internal (red team) yang dilakukan oleh tim yang sama yang
  merancang protokol. Ini bukan jaminan yang cukup untuk produksi.
- **Belum diuji terhadap BKZ 2.0** dengan parameter agresif. Red team yang
  ada hanya menggunakan LLL dasar.
- **Belum diuji terhadap algebraic cryptanalysis** dan teknik analisis
  modern lainnya.
- **Belum diuji terhadap quantum simulation** (Shor, Grover).

## 2. Asumsi Kriptografi

- Keamanan FCR-RSA bergantung pada asumsi bahwa deret Fibonacci non-linear
  dengan noise LWE deterministik sulit dibalik. **Asumsi ini belum
  diverifikasi secara formal.**
- Jika asumsi LWE dengan parameter yang digunakan runtuh, FCR-RSA runtuh.
- Pemilihan modulus Gaussian integer `Q` belum melalui analisis
  kriptanalitik yang ketat.

## 3. Implementasi

- **Rust:** Operasi `BigInt` belum diverifikasi constant-time.  Ada kemungkinan side-channel timing.
- **Python:** Implementasi Python adalah untuk referensi dan pengujian,
  **bukan** untuk produksi. Python tidak cocok untuk operasi kriptografi
  yang sensitif terhadap waktu.
- **Zeroization:** Belum ada jaminan bahwa secret di memori dihapus dengan
  aman setelah digunakan.
- **Fuzzing:** Belum ada fuzzing yang komprehensif terhadap parser input.

## 4. Fitur yang Diklaim tapi Belum Teruji

- **Auto-imun (token spatiotemporal):** Logika deteksi anomali belum diuji
  terhadap data dunia nyata. False positive dan false negative belum diukur.
- **Kill-switch matematika:** Belum diuji dalam skenario serangan nyata.
  Risiko: kill-switch yang salah picu bisa menyebabkan denial of service.
- **Honeytoken:** Konsep ada, implementasi belum ada. Belum jelas bagaimana
  honeytoken berinteraksi dengan transaksi sah.

## 5. Interoperabilitas

- **Belum ada standar.** FCR-RSA bukan bagian dari NIST, IEEE, 3GPP, atau
  standar lain. Tidak ada jaminan interoperabilitas dengan sistem lain.
- **Belum ada test vector resmi** yang diverifikasi oleh pihak ketiga.
- **API belum stabil.** Antara versi 0.x, API bisa berubah tanpa notice.

## 6. Performa

- Benchmark yang ada (Tier A: ~0.5ms dekripsi, Tier B: ~0.05ms) diukur
  pada perangkat pengembangan, **bukan** pada perangkat target (server
  bank, IoT, mobile).
- Belum ada benchmark pada perangkat dengan memori terbatas (<64KB).
- Belum ada benchmark pada beban konkuren (multi-thread, multi-user).

## 7. Cakupan Penggunaan

FCR-RSA **tidak dirancang** untuk:
- Melindungi data yang harus tetap aman selama >30 tahun (butuh analisis
  umur simpan kunci).
- Menggantikan protokol yang sudah terstandar (TLS, SSH, Signal) tanpa
  kajian interoperabilitas.
- Digunakan oleh pengguna akhir tanpa pendampingan teknisi.

## 8. Vektor Serangan yang Tidak Dicakup

FCR-RSA **tidak** mengklaim bisa melawan:
- Serangan yang mengeksploitasi bug implementasi (buffer overflow, RCE)
  — ini tanggung jawab bahasa dan runtime, bukan protokol.
- Serangan physical (side-channel daya, EM, fault injection) — belum
  diuji sama sekali.
- Serangan supply chain (kompromi dependensi, build server).
- Serangan humaniora yang tidak terdeteksi oleh token spatiotemporal
  (misalnya insider yang punya akses sah).

## 9. Batasan Filosofis

FCR-RSA dibangun atas prinsip "kekacauan yang teratur". Tapi kekacauan
yang tidak dipahami bisa menjadi **celah**, bukan **fitur**. Setiap klaim
tentang "keamanan dari kekacauan" harus diuji, bukan dipercaya.

## 10. Komitmen Kami

Kami akan memperbarui dokumen ini setiap kali ada temuan baru. Jika Anda
menemukan klaim di README atau whitepaper yang **tidak** tercakup di sini,
itu adalah bug dokumentasi—laporkan.

---

## 🔬 Known Issues & Limitations: Auto-Mune (Fase 10B)

Fitur Auto-Mune (deteksi anomali spatiotemporal + *kill switch* + honeypot) yang diimplementasikan pada Fase 10B adalah **prototipe demonstrasi**. Analisis kritis menemukan beberapa titik rawan yang **harus** diselesaikan sebelum digunakan di lingkungan produksi:

### 1. Risiko False Positive (Pemblokiran Pengguna Sah)
*   **Latency Threshold (500ms)**: Terlalu rentan terhadap koneksi 3G/4G lambat atau jaringan satelit (Starlink). Pengguna sah bisa salah di-*lock*.
*   **GPS Drifting**: Perpindahan GPS akibat *noise* perangkat (terutama di dalam gedung) bisa memicu *speed anomaly* palsu.
*   **Timestamp Non-Monotonic**: Jam klien yang tidak sinkron (misal setelah baterai habis) bisa dianggap sebagai anomali.
*   **Mitigasi (Fase 11)**: Terapkan *Dynamic Baseline*, toleransi jarak (abaikan perpindahan < 5km), dan toleransi waktu (±5 menit).

### 2. Risiko Bypass (Penyerang Lolos)
*   **Header Spoofing**: Middleware saat ini **mempercayai mentah-mentah** header `X-FCR-Latency` dan `X-FCR-GPS` dari klien. Penyerang bisa memanipulasi header agar terlihat "normal".
*   **Replay Attack**: Middleware belum menyimpan *state* `last_token` per API Key (masih `None`), sehingga penyerang bisa mengirim ulang request yang sama.
*   **Mitigasi (Fase 11)**: Token spatiotemporal harus ditandatangani (HMAC/JWT). Server harus mengukur latency sendiri (TCP handshake) atau memvalidasi token sesi.

### 3. Risiko Denial of Service (DoS)
*   **Lockdown Permanen**: Jika API Key penyerang bocor, penyerang bisa sengaja mengirim request dengan latency tinggi untuk memicu *lockdown* pada key korban. Korban menjadi tidak bisa bertransaksi.
*   **Mitigasi (Fase 11)**: Jangan langsung *lock* permanen. Gunakan *Rate Limiting*, *Cooldown* (blokir sementara 5 menit), atau minta verifikasi tambahan (MFA/OTP).

### 4. Skalabilitas & Privasi
*   **State Management**: Menyimpan `last_token` di PostgreSQL untuk setiap request akan membebani database (I/O bottleneck).
*   **Privasi GPS**: Menyimpan koordinat mentah (Lat/Lon) di tabel `anomaly_events` berpotensi melanggar privasi (UU PDP/GDPR).
*   **Mitigasi (Fase 11)**: Gunakan Redis (in-memory) untuk state spatiotemporal. Simpan hanya *hash* koordinat atau jarak (distance) di log, bukan koordinat mentah.

### 5. Rekomendasi Arsitektur (Hardening)
*   Ubah pendekatan dari **"Percaya Header"** menjadi **"Verifikasi Kriptografis"** (JWT/HMAC).
*   Ubah dari **"Lockdown Permanen"** menjadi **"Adaptive Threat Response"** (Risk Scoring: Latency 30% + GPS 40% + Timestamp 30%).

### 6. Kepatuhan Hukum (UU PDP No. 27/2022) - Kritis

Auto-Mune saat ini berpotensi menyimpan data lokasi (`X-FCR-GPS-Lat`, `X-FCR-GPS-Lon`) ke dalam tabel `anomaly_events`. Dalam konteks hukum Indonesia, ini adalah pelanggaran serius jika dilakukan tanpa dasar pemrosesan yang sah.

*   **Pasal 20 & 21 UU PDP**: Pemrosesan data pribadi (termasuk geolokasi) **wajib** memiliki persetujuan (consent) eksplisit dan informasi yang jelas tentang tujuan serta jangka waktu penyimpanan.
*   **Sanksi**: Pelanggaran dapat dikenai sanksi administratif (denda hingga 2% pendapatan tahunan) dan **sanksi pidana** (Pasal 67: penjara maksimal 5 tahun dan/atau denda hingga Rp5 miliar untuk individu, atau 10x lipat untuk korporasi).
*   **Mitigasi Wajib (Fase 11)**:
    1.  **Jangan simpan koordinat mentah**. Simpan hanya **jarak (distance)** atau **hash** dari koordinat di tabel `anomaly_events`.
    2.  Jika tetap ingin menyimpan, API harus memiliki mekanisme *opt-in* (`X-FCR-Consent: true`) yang didokumentasikan jelas.
    3.  Terapkan **retensi data** (misal: auto-delete log anomali setelah 30 hari).

---

**Terakhir diperbarui:** 13 November 2026
**Versi:** 0.1.0-research
