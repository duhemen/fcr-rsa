# Arsitektur FCR-RSA

## 1. Fondasi Matematika

### A. Deret Fibonacci Non-Linear

Alih-alih penjumlahan, FCR-RSA menggunakan perkalian untuk pertumbuhan
eksponensial yang kacau:

```
F_n = (F_{n-1} * F_{n-2}) + e_n  mod Q
```

- `e_n` : noise LWE deterministik dari `noise_seed` (SHA-256)
- `Q`   : modulus kompleks (Gaussian integer)

### B. Pemetaan Gaussian

Setiap elemen dipetakan ke ruang kompleks:

```
Z_n = Re(F_n) + Im(F_{n+1}) * i
```

### C. IND-CPA Patch (v1.1)

Random blinding factor ditambahkan pada enkripsi:

```
C = (M * E_vec) + N_comp + R_blind
```

Ini mencegah serangan chosen-plaintext attack (CPA).

## 2. Arsitektur Dua Tier

### Tier A — Enterprise / Banking

- Indeks Fibonacci besar (N > 500)
- Big Integer untuk presisi mutlak
- Keamanan maksimal, cocok untuk server

### Tier B — End-User / IoT

- Matriks Fibonacci ringan (2x2 atau 3x3)
- Efisiensi baterai & CPU untuk perangkat mobile

### Bridge Protocol

- Sinkronisasi session key antara Tier B dan Tier A
- Fibonacci heartbeat untuk memastikan komunikasi aman

## 3. Sistem Auto-Imun (Zero-Trust Hardened — Fase 11)

### A. Token Spatiotemporal & Integritas Kriptografis (HMAC-SHA256)

Setiap transaksi API wajib menyertakan telemetri fisik yang dilindungi integritasnya:
- **GPS (`X-FCR-GPS-Lat`, `X-FCR-GPS-Lon`)**: Dikuantisasi ke grid zona (~11 km) dan di-hash dengan **Keyed SHA-256** (`last_loc_hash`) di memori. Koordinat mentah tidak pernah disimpan ke database (kepatuhan UU PDP No. 27/2022).
- **Timestamp (`X-FCR-Timestamp`)**: Mendeteksi *replay attack* dan *clock skew* (toleransi mundur hingga 5 menit).
- **Latensi (`X-FCR-Latency`)**: Mendeteksi anomali proksi atau penyadapan MITM.
- **Tanda Tangan (`X-FCR-Signature`)**: Tanda tangan **HMAC-SHA256** atas payload `{lat}:{lon}:{timestamp}:{latency}` yang diverifikasi secara *constant-time* untuk mencegah *header spoofing*.

### B. Adaptive Threat Response & Deceptive Honeypot

Alih-alih mengunci kunci secara permanen (yang berisiko memicu *Denial of Service*), sistem menerapkan respons adaptif berbasis **Risk Score (0–100)**:
- **Fail-Closed Enforcement**: Request tanpa telemetri lengkap ditolak seketika (`HTTP 400`), sedangkan tanda tangan HMAC palsu langsung diblokir (`HTTP 401`).
- **Deceptive Honeypot (`Score >= 70`)**: Jika terdeteksi *impossible zone jump* atau *latency spike* ekstrem, server tetap mengembalikan `HTTP 200 OK` berisi `ciphertext_b64` palsu yang menyerupai struktur enkripsi asli agar penyerang tidak menyadari bahwa sesi mereka telah dialihkan.
- **Self-Healing Cooldown**: Kunci yang terkompromi dibekukan sementara selama **5 menit** (`apply_cooldown`) agar pengguna sah dapat pulih otomatis tanpa intervensi manual.

## 4. Alur Data (REST API & Middleware)

```text
[Klien / SDK]
   |  (Sign payload telemetri via HMAC-SHA256)
   v
[Middleware: require_api_key]
   |-- 1. Cek API Key & Status Cooldown DB -----> [401 jika terkunci]
   |-- 2. Validasi Header & HMAC Constant-Time -> [400 / 401 jika spoofed]
   |-- 3. Quantize GPS -> Keyed SHA-256 Hash
   |-- 4. Evaluasi Risk Score (0-100)
   |       └── Skor >= 70: Log Anomali + Set Cooldown 5m + [200 Deceptive Honeypot]
   v
[Handler: /keygen | /encrypt | /decrypt]
   |  (Operasi Fibonacci Non-Linear + LWE + Random Blinding)
   v
[Ciphertext / Plaintext Asli]
```

## 4. Alur Data

```
[Pengirim]
   |
   v
[Tier B / Tier A] --enkripsi--> [Ciphertext]
   |
   v
[Auto-Imun: token spatiotemporal]
   |
   v
[Jaringan / Channel]
   |
   v
[Auto-Imun: verifikasi anomali]
   |
   v
[Tier A / Tier B] --dekripsi--> [Plaintext]
```

## 5. Batasan Arsitektur

Lihat [LIMITATIONS.md](../LIMITATIONS.md). Arsitektur ini belum melalui
audit independen dan **tidak boleh** dipakai untuk produksi.

## Catatan: Blinding Factor

Pada contoh (`examples/basic.rs`, `examples/basic.py`), `blinding` sengaja
dibuat konstan (`12345`) agar ciphertext **reproduktif** dan bisa
dibandingkan lintas-bahasa.

**Di produksi, `blinding` HARUS di-generate secara kriptografis acak**
(misalnya dari `OsRng` di Rust, `secrets.token_bytes` di Python) untuk
setiap enkripsi. Tanpa ini, FCR-RSA tidak memenuhi IND-CPA, karena
plaintext yang sama akan menghasilkan ciphertext yang sama.

---

**Terakhir diperbarui:** 17 November 2026
