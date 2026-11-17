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

- **Rust:** Operasi `BigInt` belum diverifikasi constant-time untuk seluruh
  jalur enkripsi/dekripsi (meskipun verifikasi HMAC-SHA256 pada middleware
  sudah menggunakan perbandingan *constant-time*). Ada kemungkinan
  side-channel timing pada operasi aritmetika besar.
- **Python:** Implementasi Python adalah untuk referensi dan pengujian,
  **bukan** untuk produksi. Python tidak cocok untuk operasi kriptografi
  yang sensitif terhadap waktu.
- **Zeroization:** Belum ada jaminan bahwa secret di memori dihapus dengan
  aman setelah digunakan.
- **Fuzzing:** Fuzzing tingkat dasar pada signature header telah diuji,
  namun belum ada fuzzing yang komprehensif terhadap parser payload utama.

## 4. Fitur yang Diklaim & Status Pengujian

- **Auto-imun (token spatiotemporal):** Mekanisme *Risk Scoring*, *Privacy-Preserving
  Location Hashing*, *HMAC-SHA256 Signature Verification*, *Deceptive Honeypot*,
  dan *Auto-Cooldown* 5 menit telah diimplementasikan dan terverifikasi penuh
  pada pengujian *end-to-end red team* (7 skenario serangan terkoordinasi).
  Namun, rasio *false positive* dan *false negative* pada trafik pengguna dunia
  nyata berskala besar belum diukur.
- **Kill-switch matematika:** Mekanisme penghentian darurat tingkat kunci (*hard lock*)
  telah digantikan dengan *adaptive cooldown* 5 menit untuk mencegah *denial of service*.
- **Deceptive Honeypot:** Telah diimplementasikan pada lapisan middleware (`auth.rs`)
  dengan format keluaran tersamar (`ciphertext_b64` palsu yang identik secara
  struktural dengan keluaran asli).

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
- Belum ada benchmark pada beban konkuren ekstrem (multi-thread, ribuan
  pengguna serentak).

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
  (misalnya insider yang menggunakan perangkat sah di lokasi sah).

## 9. Batasan Filosofis

FCR-RSA dibangun atas prinsip "kekacauan yang teratur". Tapi kekacauan
yang tidak dipahami bisa menjadi **celah**, bukan **fitur**. Setiap klaim
tentang "keamanan dari kekacauan" harus diuji, bukan dipercaya.

## 10. Komitmen Kami

Kami akan memperbarui dokumen ini setiap kali ada temuan baru. Jika Anda
menemukan klaim di README atau whitepaper yang **tidak** tercakup di sini,
itu adalah bug dokumentasi—laporkan.

---

## 🔬 Security Architecture & Limitations: Auto-Mune (Fase 10B → Fase 11 Hardened)

Fitur Auto-Mune telah berevolusi dari prototipe deteksi statis menjadi sistem **Zero-Trust Adaptive Threat Response** yang terverifikasi. Berikut adalah pemetaan risiko yang telah berhasil ditutup dan target penyempurnaan skala produksi berikutnya:

### ✅ Kerentanan yang Sudah Berhasil Dimitigasi (Verified)
1. **Header Spoofing & Tampering (Ditutup via HMAC-SHA256):**
   * Seluruh parameter telemetri (`X-FCR-GPS-Lat`, `X-FCR-GPS-Lon`, `X-FCR-Timestamp`, `X-FCR-Latency`) wajib ditandatangani menggunakan **HMAC-SHA256** (`X-FCR-Signature`) dan diverifikasi secara *constant-time* sebelum diproses. Manipulasi 1 karakter langsung ditolak (`HTTP 401`).
2. **Kepatuhan Privasi UU PDP No. 27/2022 (Ditutup via Grid Hashing):**
   * Sistem menerapkan prinsip *Privacy by Design*. Koordinat GPS mentah **tidak pernah disimpan** ke database (`api_key_states` maupun `anomaly_events`). Lokasi dikuantisasi ke grid zona (~11 km) dan di-hash menggunakan **Keyed SHA-256** (`last_loc_hash`) di memori sebelum disimpan.
3. **Risiko Permanent Denial of Service / DoS (Ditutup via Adaptive Cooldown):**
   * Mengganti *permanent lock* dengan *temporary cooldown* selama 5 menit (`apply_cooldown`) sehingga kunci pulih otomatis (*self-healing*) tanpa memblokir pengguna sah selamanya.
4. **GPS Drifting & Clock Skew (Ditutup via Tolerance Threshold):**
   * Kuantisasi grid ~11 km mencegah *false positive* akibat *noise* GPS dalam ruangan, disertai toleransi sinkronisasi jam mundur hingga `-300 detik`.
5. **Deceptive Honeypot & Fail-Closed Enforcement:**
   * Permintaan tanpa telemetri ditolak seketika (`HTTP 400`), sedangkan serangan anomali tinggi (`risk_score >= 70`) dialihkan ke *honeypot* dengan keluaran `ciphertext_b64` palsu yang menyerupai struktur enkripsi asli.

### ⚠️ Sisa Batasan Skala Produksi (Target Fase 12)
1. **Skalabilitas State Management (Database I/O):**
   * *Batasan:* Penyimpanan `last_loc_hash` dan `last_timestamp` masih menggunakan PostgreSQL (`api_key_states`) pada setiap request.
   * *Rencana Fase 12:* Migrasikan *state* spatiotemporal ke **Redis** (*in-memory cache* dengan TTL otomatis) untuk mendukung ribuan transaksi per detik tanpa *I/O bottleneck*.
2. **Server-Side Latency Verification:**
   * *Batasan:* Nilai `X-FCR-Latency` meskipun sudah dilindungi integritasnya oleh HMAC dari klien, tetap berasal dari pengukuran sisi klien/SDK.
   * *Rencana Fase 12:* Gabungkan laporan latensi klien dengan pengukuran *round-trip time* (RTT) tingkat TCP/TLS di sisi server.
3. **Retensi Log Otomatis (Data Retention Policy):**
   * *Batasan:* Tabel `anomaly_events` (meskipun sudah bebas koordinat mentah) belum memiliki *cron job* pembersihan otomatis.
   * *Rencana Fase 12:* Tambahkan kebijakan *auto-purge* log anomali setiap 30 hari.

---

**Terakhir diperbarui:** 17 November 2026
**Versi:** 0.2.0-hardened
