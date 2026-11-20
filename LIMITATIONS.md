# Limitations — Apa yang TIDAK Dijamin oleh FCR-RSA

Dokumen ini jujur tentang batasan FCR-RSA. Membacanya sebelum menggunakan
FCR-RSA adalah wajib.

**Versi:** 0.6.0 + Fase 3.5 (Engine Integration)  
**Terakhir diperbarui:** 20 November 2026

---

## 1. Status Keamanan

### ✅ Yang Sudah Diverifikasi (Internal)
- **Hybrid KEM + AEAD (v0.5.1+):** Known-plaintext attack **tertutup total**.
- **Q 2048-bit (v0.6.0):** Small-Q brute-force **tertutup permanen**.
- **Constant-Time (v0.6.0):** Welch's t-test (p=0.143 > 0.05) + Cache simulation (0% difference).
- **Cross-Language Consistency:** 10/10 test vectors match (Python ↔ Rust).
- **Algebraic Resistance:** Gröbner basis complexity ~10^1850.
- **ML Noise Prediction:** Random Forest accuracy 51.67% ≈ random guess.
- **Auto-Mune + Honeypot + Forensics:** Brutal Test Suite **36/36 PASS**.
- **BKZ 2.0 (Parameter Kecil):** Gagal menembus pada matriks 3×3, Q=10^9.

### ❌ Yang Belum Diverifikasi
- **Independent Security Audit:** Belum ada pihak ketiga (Cure53, NCC Group) yang mengaudit kode.
- **BKZ 2.0 (Parameter Besar):** Belum diuji dengan matriks >10×10 (butuh Colab/Linux).
- **Quantum Simulation (Skala Produksi):** Simulasi Grover/Shor baru pada parameter kecil (2^10, N=15/21/35). Belum pada 2^256.
- **Formal Verification:** Belum ada pembuktian matematis menggunakan Coq/Lean.
- **Hardware Fault Injection:** Belum diuji di level fisik (glitching, laser fault injection).

---

## 2. Asumsi Kriptografi

### ✅ Yang Sudah Divalidasi
- **Non-Linear Fibonacci + LWE Noise:** Telah diuji terhadap LLL/BKZ dasar, algebraic cryptanalysis, dan ML noise prediction. Semua gagal menembus.
- **Q 2048-bit:** Ruang pencarian 2^2048, tahan terhadap brute-force klasik dan Grover's algorithm (2^1024).
- **Hybrid KEM + AEAD:** IND-CPA secure, known-plaintext attack tidak relevan.

### ⚠️ Yang Masih Berbasis Asumsi
- **Security Reduction:** Belum ada pembuktian formal bahwa memecahkan FCR-RSA setara dengan memecahkan masalah kisi (SVP/CVP) atau faktorisasi. Ini adalah **asumsi keamanan**, bukan **bukti matematis**.
- **LWE Deterministik:** Noise di-generate dari `SHA-256(seed || counter)`, bukan true random sampling. Ini bukan LWE standar, dan belum ada analisis formal tentang implikasi keamanannya.

---

## 3. Implementasi

### ✅ Yang Sudah Diverifikasi
- **Constant-Time (Rust):** Welch's t-test (p=0.143) + Cache simulation (0% difference). Menggunakan `crypto-bigint::DynResidue` yang selalu memproses semua limbs.
- **HMAC-SHA256 Verification:** Constant-time comparison untuk mencegah timing attack pada signature verification.
- **Cross-Language Consistency:** Python Oracle dan Rust implementation 100% sinkron (10/10 vectors match).

### ⚠️ Yang Perlu Diperhatikan
- **Python Implementation:** Hanya untuk referensi dan pengujian. Tidak cocok untuk operasi kriptografi sensitif terhadap waktu (timing). Gunakan Rust untuk produksi.
- **Zeroization:** Belum ada jaminan eksplisit bahwa secret di memori dihapus dengan aman setelah digunakan (belum implementasi `zeroize` crate).
- **Fuzzing:** Fuzzing tingkat dasar pada signature header telah diuji, namun belum ada fuzzing komprehensif terhadap parser payload utama.

---

## 4. Fitur yang Diklaim & Status Pengujian

### ✅ Auto-Imun (Spatiotemporal Token) — VERIFIED
Mekanisme **Risk Scoring**, **Privacy-Preserving Location Hashing**, **HMAC-SHA256 Signature Verification**, **Deceptive Honeypot (Poisoned Pill)**, **Forensic Tracing (trace_id)**, dan **Auto-Cooldown 5 menit** telah diimplementasikan dan terverifikasi penuh pada **Brutal Test Suite (36/36 PASS)**.

**Batasan:**
- Rasio **false positive** dan **false negative** pada trafik pengguna dunia nyata berskala besar belum diukur.
- Skalabilitas state management (PostgreSQL I/O) belum diuji pada ribuan transaksi per detik.

### ✅ Deceptive Honeypot (Poisoned Pill) — VERIFIED
Ketika Auto-Mune mendeteksi serangan (Risk Score ≥ 70), server mengembalikan HTTP `200 OK` dengan ciphertext yang terlihat valid, namun plaintext-nya diganti dengan **payload forensik** berisi `trace_id`. Payload ini dienkripsi dengan kunci yang sama, sehingga ciphertext terlihat 100% valid.

**Batasan:**
- Penyerang yang canggih mungkin menyadari bahwa ciphertext honeypot memiliki panjang yang tidak biasa (blocks > 100) dan mencurigai adanya honeypot.

### ✅ Forensic Tracing — VERIFIED
`trace_id` tercatat di tabel `anomaly_events` dengan format yang sama persis dengan yang disuntikkan ke ciphertext penyerang. Ciphertext honeypot yang bocor dapat ditelusuri balik ke pelaku via query SQL.

**Batasan:**
- Memerlukan akses ke database untuk melakukan tracing. Jika database dikompromikan, forensic tracing tidak berguna.

---

## 5. Interoperabilitas

### ⚠️ Yang Perlu Diperhatikan
- **Belum Ada Standar:** FCR-RSA bukan bagian dari NIST, IEEE, 3GPP, atau standar lain. Tidak ada jaminan interoperabilitas dengan sistem lain.
- **Test Vector Internal:** Sudah ada 10/10 cross-language match (Python ↔ Rust), namun belum diverifikasi oleh pihak ketiga.
- **API Belum Stabil:** Antara versi 0.x, API bisa berubah tanpa notice. v0.6.0 tidak kompatibel dengan ciphertext v0.5.1.

---

## 6. Performa

### ✅ Yang Sudah Diukur (Internal)
- **Enkripsi (42 bytes):** ~35 ms
- **Dekripsi (42 bytes):** ~15 ms
- **Auto-Mune Scoring:** < 1 ms
- **HMAC Verification:** < 1 ms

> **⚙️ Environment Pengujian:**
> *   **Perangkat:** Laptop High-Performance
> *   **CPU:** Intel Core i9 Gen 13 (Mobile)
> *   **RAM:** 16 GB DDR5
> *   **GPU:** NVIDIA RTX 4060 8 GB
> *   **Database:** PostgreSQL 14
> *   **Build Mode:** `cargo build --release`
>
> *Catatan: Hasil benchmark pada perangkat Desktop (misal: i7-12700K) atau Server mungkin menunjukkan latensi yang lebih rendah karena perbedaan arsitektur cache dan thermal headroom.*

### ⚠️ Yang Belum Diukur
- **Benchmark pada Perangkat Target:** Belum diuji pada server bank, IoT, atau mobile devices.
- **Benchmark pada Beban Konkuren Ekstrem:** Belum diuji pada multi-thread, ribuan pengguna serentak.
- **Benchmark pada Memori Terbatas:** Belum diuji pada perangkat dengan memori <64KB.

---

## 7. Cakupan Penggunaan

FCR-RSA **tidak dirancang** untuk:
- Melindungi data yang harus tetap aman selama >30 tahun (butuh analisis umur simpan kunci).
- Mengganti protokol yang sudah terstandar (TLS, SSH, Signal) tanpa kajian interoperabilitas.
- Digunakan oleh pengguna akhir tanpa pendampingan teknisi.

---

## 8. Vektor Serangan yang Tidak Dicakup

FCR-RSA **tidak mengklaim** bisa melawan:
- **Serangan yang mengeksploitasi bug implementasi** (buffer overflow, RCE) — ini tanggung jawab bahasa dan runtime, bukan protokol.
- **Serangan physical level** (side-channel daya, EM, fault injection) — side-channel timing dan cache sudah diuji, tapi fault injection fisik belum.
- **Serangan supply chain** (kompromi dependensi, build server).
- **Serangan humaniora yang tidak terdeteksi oleh token spatiotemporal** (misalnya insider yang menggunakan perangkat sah di lokasi sah).

---

## 9. Batasan Filosofis

FCR-RSA dibangun atas prinsip "kekacauan yang teratur". Tapi kekacauan yang tidak dipahami bisa menjadi celah, bukan fitur. Setiap klaim tentang "keamanan dari kekacauan" harus diuji, bukan dipercaya.

---

## 10. Komitmen Kami

Kami akan memperbarui dokumen ini setiap kali ada temuan baru. Jika Anda menemukan klaim di README atau whitepaper yang tidak tercakup di sini, itu adalah bug dokumentasi—laporkan.

---

## 🛡️ Security Architecture & Limitations: Auto-Mune (v0.6.0 + Fase 3.5)

Fitur Auto-Mune telah berevolusi dari prototipe deteksi statis menjadi sistem **Zero-Trust Adaptive Threat Response** yang terverifikasi melalui **Brutal Test Suite (36/36 PASS)**. Berikut adalah pemetaan risiko yang telah berhasil ditutup dan target penyempurnaan skala produksi berikutnya:

### ✅ Kerentanan yang Sudah Berhasil Dimitigasi (Verified)

1. **Header Spoofing & Tampering** (Ditutup via HMAC-SHA256):
   Seluruh parameter telemetri (`X-FCR-GPS-Lat`, `X-FCR-GPS-Lon`, `X-FCR-Timestamp`, `X-FCR-Latency`) wajib ditandatangani menggunakan HMAC-SHA256 (`X-FCR-Signature`) dan diverifikasi secara constant-time sebelum diproses. Manipulasi 1 karakter langsung ditolak (`HTTP 401`).

2. **Kepatuhan Privasi UU PDP No. 27/2022** (Ditutup via Grid Hashing):
   Sistem menerapkan prinsip Privacy by Design. Koordinat GPS mentah tidak pernah disimpan ke database (`api_key_states` maupun `anomaly_events`). Lokasi dikuantisasi ke grid zona (~11 km) dan di-hash menggunakan Keyed SHA-256 (`last_loc_hash`) di memori sebelum disimpan.

3. **Risiko Permanent Denial of Service / DoS** (Ditutup via Adaptive Cooldown):
   Mengganti permanent lock dengan temporary cooldown selama 5 menit (`apply_cooldown`) sehingga kunci pulih otomatis (self-healing) tanpa memblokir pengguna sah selamanya.

4. **GPS Drifting & Clock Skew** (Ditutup via Tolerance Threshold):
   Kuantisasi grid ~11 km mencegah false positive akibat noise GPS dalam ruangan, disertai toleransi sinkronisasi jam mundur hingga -300 detik.

5. **Deceptive Honeypot & Fail-Closed Enforcement**:
   Permintaan tanpa telemetri ditolak seketika (`HTTP 400`), sedangkan serangan anomali tinggi (`risk_score >= 70`) dialihkan ke honeypot dengan keluaran ciphertext yang menyerupai struktur enkripsi asli namun berisi payload forensik ber-`trace_id`.

6. **Forensic Tracing** (Ditutup via trace_id):
   Setiap insiden honeypot tercatat di `anomaly_events` dengan `trace_id` yang sama persis dengan yang disuntikkan ke ciphertext penyerang. Ciphertext yang bocor dapat ditelusuri balik ke pelaku.

### ⚠️ Sisa Batasan Skala Produksi (Target Fase 4)

1. **Skalabilitas State Management (Database I/O):**
   - **Batasan:** Penyimpanan `last_loc_hash` dan `last_timestamp` masih menggunakan PostgreSQL (`api_key_states`) pada setiap request.
   - **Rencana Fase 4:** Migrasikan state spatiotemporal ke Redis (in-memory cache dengan TTL otomatis) untuk mendukung ribuan transaksi per detik tanpa I/O bottleneck.

2. **Server-Side Latency Verification:**
   - **Batasan:** Nilai `X-FCR-Latency` meskipun sudah dilindungi integritasnya oleh HMAC dari klien, tetap berasal dari pengukuran sisi klien/SDK.
   - **Rencana Fase 4:** Gabungkan laporan latensi klien dengan pengukuran round-trip time (RTT) tingkat TCP/TLS di sisi server.

3. **Retensi Log Otomatis (Data Retention Policy):**
   - **Batasan:** Tabel `anomaly_events` (meskipun sudah bebas koordinat mentah) belum memiliki cron job pembersihan otomatis.
   - **Rencana Fase 4:** Tambahkan kebijakan auto-purge log anomali setiap 30 hari.

4. **Observability Gap (Minor):**
   - **Batasan:** Skor intermediate (40–69) hanya di-`eprintln` di server, tidak di-log ke DB dan tidak di-expose di response.
   - **Rencana Fase 4:** Tambah kolom `risk_score` di response `/encrypt` saat suspicious, atau endpoint debug `/admin/last-risk`.

### ⚠️ Batasan Fondasi Matematika (Mathematical Limitations)

Sebagai protokol riset, FCR-RSA memiliki batasan matematis yang harus dipahami sebelum klaim keamanan apa pun dibuat:

1. **Belum Ada Security Reduction ke Masalah Hard Standar:**
   Deret Fibonacci non-linear dengan noise deterministik (`SHA-256(seed || counter)`) bukanlah implementasi Learning With Errors (LWE) yang standar. LWE sejati memerlukan sampling Gaussian yang benar-benar acak (true random), bukan pseudo-random. Saat ini, tidak ada pembuktian matematis (reduction proof) yang menunjukkan bahwa memecahkan FCR-RSA setara dengan memecahkan masalah kisi (SVP/CVP) atau faktorisasi.

2. **Kerentanan Teoretis terhadap Algebraic Cryptanalysis:**
   Karena relasi rekurensi bersifat deterministik dan berbasis perkalian, terdapat risiko teoretis bahwa penyerang dengan sumber daya komputasi tinggi dapat memformulasikan sistem persamaan polinomial dan menyelesaikannya menggunakan algoritma Gröbner Basis atau linearisasi. Namun, pengujian internal menunjukkan kompleksitas ~10^1850, yang membuat serangan ini tidak praktis saat ini.

3. **Ukuran Modulus Q:**
   Implementasi saat ini menggunakan **Q = 2048-bit** (`crypto-bigint::U2048`), yang cukup besar untuk menahan serangan brute-force dan kisi. Namun, pemilihan parameter ini belum melalui analisis kriptanalitik formal yang ketat.

### 🛡️ Mitigasi yang Sudah Diterapkan (Defense in Depth)

Meskipun fondasi matematika di atas masih dalam tahap eksplorasi, FCR-RSA mengompensasinya dengan Lapisan Auto-Imun (v0.6.0 + Fase 3.5) yang kuat di tingkat aplikasi:

- **Hybrid KEM + AEAD:** Menutup celah known-plaintext attack secara total.
- **U2048 + Montgomery Arithmetic:** Constant-time verified (Welch p=0.143, cache 0%).
- **HMAC-SHA256 Telemetry Auth:** Mencegah spoofing dan replay attack pada level jaringan.
- **Privacy-Preserving Location Hashing:** Mematuhi UU PDP dengan tidak pernah menyimpan koordinat GPS mentah.
- **Deceptive Honeypot + Forensic Tracing:** Secara aktif mengisolasi, membingungkan, dan melacak penyerang yang mencoba mengeksploitasi kunci secara massal.
- **Brutal Test Suite (36/36 PASS):** Semua klaim desain telah diverifikasi secara otomatis.

### Kesimpulan

FCR-RSA v0.6.0 + Fase 3.5 adalah **Proof-of-Concept yang aman secara rekayasa (engineering-hardened) dan terverifikasi secara empiris (36/36 brutal test)**, namun **belum terbukti aman secara matematis murni (mathematically unproven)** dan **belum diaudit oleh pihak ketiga**.

---

**Terakhir diperbarui:** 20 November 2026  
**Versi:** 0.6.0 + Fase 3.5 (Engine Integration + Brutal Test 36/36)