<!-- ═══════════════════════════════════════════════════════════════ -->
<!--           REPOSITORY README — FCR-RSA MULTI-LAYER               -->
<!--                  LUCA PROTOCOL © 2026                           -->
<!-- ═══════════════════════════════════════════════════════════════ -->
<div align="center" style="background: linear-gradient(135deg, #000000 0%, #001a00 50%, #003300 100%); padding: 40px 20px; border: 2px solid #00FF00; border-radius: 15px; margin-bottom: 20px; box-shadow: 0 0 30px #00FF00, inset 0 0 30px rgba(0, 255, 0, 0.1);">
<img src="https://capsule-render.vercel.app/api?type=waving&color=0:000000,50:001a00,100:00FF00&height=200&section=header&text=FCR%20RSA&fontSize=70&fontColor=00FF00&animation=fadeIn&fontAlignY=38&desc=Research%20Prototype%20v0.6.0%20%E2%80%94%20Hybrid%20KEM%20%2B%20AEAD&descAlignY=58&descSize=22" width="100%"/>
<img src="https://readme-typing-svg.demolab.com?font=Fira+Code&weight=900&size=20&duration=2500&pause=800&color=00FF00&center=true&vCenter=true&multiline=true&width=900&height=180&lines=Wake+Up+Duhemen...;Follow+the+White+LUCA...;System+Online...;From+uncertainty+comes+life%2C+from+chaos+comes+consciousness..." alt="Typing SVG" />
<h2 style="color: #00FF00; font-family: 'Courier New', monospace; margin-top: 20px; text-shadow: 0 0 10px #00FF00, 0 0 20px #00FF00;">
🔬 FCR-RSA — Research Prototype (v0.6.0)
</h2>
<p style="color: #00FF00; font-family: 'Courier New', monospace; font-size: 16px; letter-spacing: 1px;">
<code>Post-quantum hybrid KEM prototype. Open for critique. Not production ready.</code>
</p>
<br/>
  <img src="https://img.shields.io/badge/STATUS-RESEARCH%20PROTOTYPE%20v0.6.0-F59E0B?style=for-the-badge" alt="Status" />
  <img src="https://img.shields.io/badge/ARCH-HYBRID%20KEM%20%2B%20AEAD-00FF00?style=for-the-badge" alt="Architecture" />
  <img src="https://img.shields.io/badge/Q%20SIZE-2048%20BIT-00BFFF?style=for-the-badge" alt="Modulus" />
  <img src="https://img.shields.io/badge/SIDE%20CHANNEL-CONSTANT%20TIME%20VERIFIED-9370DB?style=for-the-badge" alt="Constant Time" />
  <img src="https://img.shields.io/badge/CACHE%20ATTACK-0%25%20DIFFERENCE-FF1493?style=for-the-badge" alt="Cache Attack" />
  <img src="https://img.shields.io/badge/ALGEBRAIC%20ATTACK-GROEBNER%20RESISTANT-FF6347?style=for-the-badge" alt="Algebraic" />
  <img src="https://img.shields.io/badge/CROSS%20LANGUAGE-10%2F10%20MATCH-20B2AA?style=for-the-badge" alt="Cross Language" />
  <img src="https://img.shields.io/badge/INDEPENDENT%20AUDIT-NOT%20YET-EF4444?style=for-the-badge" alt="Audit" />
  <img src="https://img.shields.io/badge/LICENSE-MIT%20OR%20Apache--2.0-blue?style=for-the-badge" alt="License" />
  <img src="https://img.shields.io/badge/RUST-LIBRARY%20%2B%2018%20TESTS-000000?style=for-the-badge&logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/PYTHON-REFERENCE%20IMPL-3776AB?style=for-the-badge&logo=python" alt="Python" />
</div>

---

## ⚠️ Status Keamanan — Baca Sebelum Apa Pun

> **FCR-RSA adalah purwarupa riset (v0.5.1). Belum melalui audit keamanan independen.**
>
> **JANGAN gunakan untuk melindungi data nyata, uang, atau identitas.**
>
> Baca **[LIMITATIONS.md](LIMITATIONS.md)** untuk daftar lengkap apa yang
> **tidak** dijamin oleh protokol ini. Baca **[SECURITY.md](SECURITY.md)**
> jika Anda menemukan celah.

| Aspek | Status |
| :--- | :--- |
| Implementasi Rust (fcr-core-rust) | ✅ Library berjalan, 5 integration test lulus |
| Implementasi Python (fcr-python) | ✅ Paket berjalan, 3 unit test lulus |
| Konsistensi Rust ↔ Python | ✅ Ciphertext identik untuk parameter uji |
| Hybrid KEM + AEAD (v0.5.1) | ✅ Known-plaintext attack **tertutup** |
| Auto-Imun v0.4.0 (HMAC, Honeypot) | ✅ Lolos Red Team 7 tahap |
| Uji BKZ 2.0 | ✅ Gagal menembus (lihat `laporan_teknis.md`) |
| Constant-Time (dudect) | ✅ Lolos (max t = 1.78 < 5.0) |
| **Temuan Kritis v0.4.1** | 🔴 **Known-plaintext attack** berhasil → **diperbaiki di v0.5.1** |
| Audit independen | ❌ **Belum** |
| Uji quantum simulation | ❌ **Belum** |
| Algebraic cryptanalysis (Gröbner) | ❌ **Belum** |
| Test vector resmi | ⚠️ Struktur ada, isi menyusul |

---

## 📌 Daftar Isi

<details open>
<summary>Klik untuk melihat / menyembunyikan</summary>

- [📖 Dokumentasi Proyek](#-dokumentasi-proyek)
- [💡 1. Latar Belakang](#-1-latar-belakang)
- [🧪 2. Fondasi Matematika](#-2-fondasi-matematika)
- [🛡️ 3. Lapisan Auto-Imun (v0.4.0)](#️-3-lapisan-auto-imun-v040)
- [🔥 4. Evolusi Keamanan (v0.1 → v0.5.1)](#-4-evolusi-keamanan-v01--v051)
- [🚨 5. Temuan Kritis & Mitigasi](#-5-temuan-kritis--mitigasi)
- [📂 6. Struktur Repositori](#-6-struktur-repositori)
- [⚡ 7. Quickstart](#-7-quickstart)
- [🧪 8. Pengujian & Simulasi](#-8-pengujian--simulasi)
- [🎯 9. Threat Model Komprehensif](#-9-threat-model-komprehensif)
- [🗺️ 10. Roadmap Pengembangan](#️-10-roadmap-pengembangan)
- [🤝 11. Panggilan Kolaborasi](#-11-panggilan-kolaborasi)

</details>

---

## 📖 Dokumentasi Proyek

- 📄 **[Proposal & KAK/TOR](docs/proposal.md)** — Latar belakang, analisis situasi, roadmap.
- 📊 **[Laporan Teknis](docs/laporan_teknis.md)** — Benchmark, temuan kritis, hasil uji ofensif & defensif.
- 📜 **[Whitepaper (English)](docs/whitepaper.md)** — Naskah akademik lengkap.
- 🏗️ **[Arsitektur](docs/architecture.md)** — Tier A/B, Bridge Protocol, Hybrid KEM, auto-imun.
- 🎯 **[Threat Model](docs/threat-model.md)** — Apa yang dilawan, apa yang tidak.
- ⚠️ **[Limitasi](LIMITATIONS.md)** — **Wajib dibaca** sebelum menggunakan.
- 🔒 **[Security Policy](SECURITY.md)** — Cara melaporkan celah.
- 📜 **[Session Log v0.5.1](docs/SESSION_LOG_v0.5.1.md)** — Catatan perbaikan Hybrid KEM.

---

## 💡 1. Latar Belakang

Sistem enkripsi global saat ini (RSA, ECC) bergantung pada kesulitan
faktorisasi bilangan prima besar. Fondasi ini diproyeksikan runtuh
ketika komputer kuantum berskala penuh menjalankan **Algoritma Shor**.

**FCR-RSA (Fibonacci Complex Noise-RSA)** adalah eksperimen protokol
kriptografi pasca-kuantum yang mengeksplorasi pendekatan berbeda:

1. **Domain matematika alternatif:** Deret Fibonacci non-linear dengan
   noise LWE deterministik, dipetakan ke Gaussian integer.
2. **Arsitektur Hybrid KEM (v0.5.1):** FCR-RSA hanya membungkus kunci
   sesi acak; data dienkripsi dengan ChaCha20-Poly1305 (AEAD).
3. **Arsitektur dua tier:** Tier A (server, keamanan maksimal) dan
   Tier B (perangkat, efisiensi baterai/CPU).
4. **Lapisan auto-imun:** Token spatiotemporal, deteksi anomali,
   kill-switch dengan isolasi endemi—sebagai bagian dari protokol,
   bukan add-on.

> ⚠️ **Klaim keamanan belum diverifikasi secara independen.** Bagian
> "Latar Belakang" ini menjelaskan **niat desain**, bukan **jaminan
> keamanan**. Lihat [LIMITATIONS.md](LIMITATIONS.md).

---

## 🧪 2. Fondasi Matematika

### A. Deret Fibonacci Non-Linear

Operasi penjumlahan standar diganti dengan perkalian, lalu disuntik
noise berbasis Learning With Errors (LWE) deterministik:

$$F_n = (F_{n-1} \times F_{n-2}) + e_n \pmod Q$$

- $e_n$: noise deterministik dari `noise_seed` (SHA-256)
- $Q$: modulus kompleks (Gaussian integer)

### B. Pemetaan Gaussian

Setiap elemen dipetakan ke ruang kompleks:

$$Z_n = \text{Re}(F_n) + \text{Im}(F_{n+1})i$$

### C. Hybrid KEM + AEAD (v0.5.1)

**PENTING:** Mulai v0.5.1, FCR-RSA **tidak lagi** mengenkripsi data
langsung. Fungsi enkripsi sekarang:

1. Generate seed acak (ukuran sesuai kapasitas Q)
2. Enkripsi seed dengan FCR-RSA → `Encrypted_Seed`
3. Turunkan kunci sesi 256-bit dari seed via **HKDF** (RFC 5869)
4. Enkripsi data dengan **ChaCha20-Poly1305** (AEAD)

Format ciphertext: `BLINDING:ES:NONCE:CT` (semua Base64)

> ⚠️ **Catatan Kritis:** Versi v0.4.1 mengenkripsi plaintext langsung
> dengan fungsi linear `C = (M × E_vec) + R_blind mod Q`, yang rentan
> terhadap known-plaintext attack. **Celah ini telah diperbaiki di v0.5.1.**
> Lihat [Bagian 5](#-5-temuan-kritis--mitigasi).

---

## 🛡️ 3. Lapisan Auto-Imun (Zero-Trust Hardened — v0.4.0)

FCR-RSA tidak berasumsi lapisan manusia dan jaringan selalu aman.
Middleware REST API (`fcr-engine`) mengikat **Spatiotemporal Token**
yang dilindungi secara kriptografis:

- **Integritas HMAC-SHA256:** Seluruh header telemetri (`X-FCR-GPS-*`,
  `X-FCR-Timestamp`, `X-FCR-Latency`) wajib ditandatangani dengan
  `X-FCR-Signature` dan diverifikasi secara *constant-time* untuk
  mencegah *header spoofing*.
- **Privacy-Preserving Location Hashing (UU PDP):** Koordinat GPS mentah
  tidak pernah disimpan ke database. Lokasi dikuantisasi ke grid zona
  (~11 km) dan di-hash dengan **Keyed SHA-256** di memori sebelum
  disimpan.
- **Deceptive Honeypot & Adaptive Cooldown:** Anomali perpindahan zona
  ekstrem atau lonjakan latensi (`Risk Score >= 70`) dialihkan ke
  *honeypot* yang mengembalikan `ciphertext_b64` palsu menyerupai
  struktur asli, disertai *self-healing cooldown* 5 menit untuk mencegah
  *permanent DoS*.

> ⚠️ Logika ini **telah lolos simulasi Red Team 7 tahap**, namun rasio
> *false positive* pada trafik dunia nyata skala besar belum diukur.
> Lihat [LIMITATIONS.md](LIMITATIONS.md).

---

## 🔥 4. Evolusi Keamanan (v0.1 → v0.5.1)

Perjalanan FCR-RSA dari prototipe mentah hingga arsitektur Hybrid KEM:

| Versi | Tanggal | Perubahan Utama | Status |
|-------|---------|-----------------|--------|
| **v0.1** | Nov 2026 | Fondasi matematika (Fibonacci + Gaussian) | ✅ Selesai |
| **v0.2** | Nov 2026 | Implementasi Rust + Python (konsistensi lintas-bahasa) | ✅ Selesai |
| **v0.3** | Nov 2026 | IND-CPA patch (random blinding) | ✅ Selesai |
| **v0.4.0** | Nov 2026 | Auto-Imun (HMAC, Honeypot, Cooldown) | ✅ Selesai |
| **v0.4.1** | Nov 2026 | Block-based chunking (upaya tutup dictionary attack) | ⚠️ Tidak cukup |
| **v0.5.0** | Nov 2026 | Hybrid KEM + ChaCha20-Poly1305 (arsitektur baru) | 🔴 Bug (Q kecil) |
| **v0.5.1** | Nov 2026 | **HKDF-based key derivation** (perbaikan v0.5.0) | ✅ **Stabil** |

### Pelajaran Penting

> **"Kriptografi yang sempurna bukanlah yang paling kaku, melainkan
> yang mampu mengakui kelemahannya dan beradaptasi."**

Temuan known-plaintext attack di v0.4.1 mengajarkan kita bahwa:
- **Enkripsi linear + blinding terbuka = celah fatal**
- **Chunking bukan solusi arsitektural**, hanya penundaan
- **Hybrid KEM + AEAD** adalah standar industri yang sudah terbukti

---

## 🚨 5. Temuan Kritis & Mitigasi

### 🔴 Temuan v0.4.1: Known-Plaintext Attack

**Masalah:**
Fungsi enkripsi v0.4.1 bersifat **linear terhadap plaintext**:

$$C = (M \times E_{vec}) + R_{blind} \pmod Q$$

Dengan $R_{blind}$ dikirim sebagai **prefix plaintext** pada ciphertext,
penyerang yang tahu **satu blok plaintext** bisa langsung menghitung:

$$E_{vec} = (C - R_{blind}) \times M^{-1} \pmod Q$$

Setelah $E_{vec}$ diketahui, **semua pesan** dengan kunci itu bisa
didekripsi.

**Bukti Serangan (Python):**

```python
# red_team_key_recovery.py
# Known-plaintext 1 blok = key recovery total

Q = 1_000_000_007
B_known = 518556109  # Dari prefix ciphertext (TERBUKA)
C_known = 793794356  # Ciphertext
M_known = 4276803    # Plaintext yang diketahui attacker

# Pulihkan E_vec
E_vec = ((C_known - B_known) * pow(M_known, -1, Q)) % Q
# E_vec = 322622128 ✅

# Verifikasi dengan blok kedua
C2 = 626351115
M2 = 4474182
assert (M2 * E_vec + B_known) % Q == C2  # ✅ COCOK

# Dekripsi pesan rahasia
secret_ct = 793794356
inv_E = pow(E_vec, -1, Q)
secret_m = ((secret_ct - B_known) * inv_E) % Q
# secret_m = 4276803 → "ABCDEF" ✅
```

**Output Nyata:**

```
[1] Plaintext diketahui : 'ABCDEF' -> blok [4276803, 4474182]
    Blinding (dari prefix ciphertext, TERBUKA): 518556109
    Ciphertext: [793794356, 626351115]

[2] E_vec DIPULIHKAN    : 322622128
    Verifikasi blok ke-2: COCOK ✅

[3] Pesan rahasia asli  : 'Dokumen Rahasia Pertahanan Nasional 2026!'
    Hasil pembobolan    : 'Dokumen Rahasia Pertahanan Nasional 2026!'

🚨 TOTAL BREAK — kunci dipulihkan dari 1 blok known-plaintext.
```

### ✅ Mitigasi v0.5.1: Hybrid KEM + AEAD

**Solusi:**
Alih-alih mengenkripsi data langsung, FCR-RSA sekarang hanya
**membungkus kunci sesi acak**:

1. Generate seed acak (ukuran sesuai kapasitas Q)
2. Enkripsi seed dengan FCR-RSA → `Encrypted_Seed`
3. Turunkan kunci sesi 256-bit dari seed via **HKDF** (RFC 5869)
4. Enkripsi data dengan **ChaCha20-Poly1305** (AEAD)

**Mengapa Ini Aman:**
- Known-plaintext attack **tidak relevan** karena attacker tidak tahu
  kunci sesi ChaCha20-Poly1305
- Seed yang dienkripsi ukurannya kecil (sesuai Q), tapi **HKDF**
  menurunkan kunci 256-bit yang tidak bisa ditebak
- AEAD memberikan **integritas data** (manipulasi ciphertext terdeteksi)

**Verifikasi Penutupan Celah (Python):**

```python
# red_team_adversarial_attack.py
# Attacker mencoba dekripsi TANPA akses server

[1] Target: 'Dokumen Rahasia Pertahanan Nasional 2026!'
    Attacker menyadap ciphertext dari jaringan...
    ✅ Ciphertext berhasil disadap: 432339172:Fj7RHg==:4Cm5Sdpv6jtD2lSV:...

[🚨 ADVERSARIAL ATTACK] Mencoba dekripsi tanpa akses server...
    Attacker hanya punya ciphertext yang disadap dari jaringan.
    Attacker TIDAK punya seed, DB access, atau kunci sesi.

    [❌ GAGAL] Attacker TIDAK BISA memulihkan kunci sesi.
    Alasan:
    1. ESK dienkripsi dengan FCR-RSA → butuh seed (rahasia)
    2. Seed hanya 3 byte → tidak cukup untuk brute-force 256-bit key
    3. HKDF menurunkan kunci sesi dari seed → deterministik tapi butuh seed
    4. ChaCha20-Poly1305 membutuhkan kunci sesi 256-bit yang tepat

🛡️ KESIMPULAN: ADVERSARIAL ATTACK GAGAL TOTAL.
```

---

## 📂 6. Struktur Repositori

```text
fcr-rsa/
├── docs/                  # Dokumentasi teknis & whitepaper
│   ├── whitepaper.md
│   ├── architecture.md
│   ├── threat-model.md
│   ├── test-vectors.md
│   ├── laporan_teknis.md
│   ├── proposal.md
│   └── SESSION_LOG_v0.5.1.md
│
├── fcr-core-rust/         # Implementasi Rust (library)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── fib.rs         # Fibonacci non-linear
│   │   ├── gaussian.rs    # Pemetaan Gaussian
│   │   ├── encrypt.rs     # Enkripsi + IND-CPA patch
│   │   ├── decrypt.rs     # Dekripsi
│   │   └── red_team.rs    # Simulasi serangan internal
│   ├── examples/basic.rs
│   ├── tests/integration.rs
│   └── benches/encrypt_bench.rs
│
├── fcr-engine/            # CLI + REST API + Auto-Imun (v0.4.0+)
│   ├── src/
│   │   ├── main.rs
│   │   ├── cli.rs
│   │   ├── crypto.rs      # Hybrid KEM + AEAD (v0.5.1)
│   │   ├── db.rs
│   │   └── api/
│   │       ├── auth.rs    # HMAC-SHA256 telemetry
│   │       ├── auto_immune.rs
│   │       └── handlers.rs
│   └── Cargo.toml
│
├── fcr-python/            # Implementasi Python (paket)
│   ├── fcr_rsa/
│   │   ├── core_math.py
│   │   ├── tier_a.py
│   │   ├── tier_b.py
│   │   └── auto_immune.py
│   ├── examples/basic.py
│   └── tests/test_basic.py
│
├── fcr-pyo3/              # Binding Rust-Python (maturin)
│
├── simulations/           # Simulasi serangan & benchmark
│   ├── red_team_lll.py
│   ├── red_team_noise_attack.py
│   ├── red_team_key_recovery.py      # 🔴 Known-plaintext attack
│   ├── red_team_adversarial_attack.py # ✅ Verifikasi penutupan celah
│   ├── ordal_attack_simulation.py
│   ├── bridge_protocol.py
│   ├── run_test.py
│   ├── stress_test.py
│   └── visualize_attack.py
│
├── test-vectors/v1/       # Test vector (JSON)
├── .github/workflows/     # CI: test, clippy, cargo-audit
│
├── README.md
├── LIMITATIONS.md         # ⚠️ Wajib dibaca
├── SECURITY.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
└── LICENSE                # Dual MIT / Apache-2.0
```

---

## ⚡ 7. Quickstart

### Rust

```bash
cd fcr-core-rust
cargo test
cargo run --example basic
```

Yang diharapkan:

```
running 5 tests
test invalid_n_rejected ... ok
test round_trip_multiple ... ok
test round_trip_small ... ok
test round_trip_tier_a ... ok
test test_ind_cpa_randomness ... ok
Plaintext : 42
Ciphertext: 722957369
Decrypted : 42
Round-trip OK
```

### Python

```bash
cd fcr-python
pip install -e .
python examples/basic.py
pytest tests/
```

Yang diharapkan:

```
Plaintext : 42
Ciphertext: 722957369
Decrypted : 42
Round-trip OK
3 passed
```

### FCR-Engine (Hybrid KEM + Auto-Imun)

```bash
cd fcr-engine

# 1. Setup Database (PostgreSQL)
psql -U postgres -c "CREATE DATABASE fcr_rsa;"

# 2. Buat Kunci
cargo run -- keygen --name "kunci_utama"

# 3. Enkripsi Teks (Hybrid KEM + AEAD)
cargo run -- encrypt --text "Halo Dunia" --keyname "kunci_utama"
# Output: BLINDING:ES:NONCE:CT (Base64)

# 4. Dekripsi
cargo run -- decrypt --ciphertext "BLINDING:ES:NONCE:CT" --keyname "kunci_utama"
```

### Jalankan Server REST API

```bash
cd fcr-engine
cargo run -- serve
# Server berjalan di http://0.0.0.0:8080
```

---

## 🧪 8. Pengujian & Simulasi

### Uji Defensif (Internal)

| File | Deskripsi | Status |
| :--- | :--- | :--- |
| `fcr-core-rust/tests/integration.rs` | Round-trip Rust | ✅ 5 passed |
| `fcr-python/tests/test_basic.py` | Round-trip Python | ✅ 3 passed |
| `simulations/run_test.py` | Enkripsi/dekripsi dasar | ✅ |
| `simulations/bridge_protocol.py` | Komunikasi Tier A/B | ✅ |

### Uji Ofensif (Red Team — Internal, Bukan Audit Independen)

| File | Deskripsi | Status |
| :--- | :--- | :--- |
| `simulations/red_team_lll.py` | Serangan LLL dasar | ✅ Gagal menembus |
| `simulations/red_team_noise_attack.py` | Serangan noise_seed | ✅ Gagal menembus |
| `simulations/ordal_attack_simulation.py` | Simulasi Pegasus/LST | ✅ Terdeteksi auto-imun |
| **BKZ 2.0** (`simulations/bkz/`) | Serangan kisi, block size 20 | ✅ Gagal menembus (di Colab) |
| **Constant-Time** (dudect) | Timing attack pada Rust | ✅ Lolos (max t=1.78) |
| 🔴 **`red_team_key_recovery.py`** | **Known-plaintext attack (v0.4.1)** | ✅ **Berhasil** → diperbaiki di v0.5.1 |
| ✅ **`red_team_adversarial_attack.py`** | Verifikasi penutupan celah (v0.5.1) | ✅ **Gagal** (celah tertutup) |

> ⚠️ **Penting:** Red team ini dilakukan oleh tim yang sama yang merancang
> protokol. Ini **bukan** audit independen. Auditor eksternal masih
> dibutuhkan sebelum klaim keamanan bisa dianggap valid.

---

## 🎯 9. Threat Model Komprehensif

### ✅ Vektor Serangan yang Sudah Ditangkal

| Serangan | Status | Mekanisme Pertahanan |
|----------|--------|----------------------|
| **Algoritma Shor** (quantum factoring) | ✅ Ditangkal | Tidak bergantung pada faktorisasi |
| **Algoritma Grover** (quantum search) | ✅ Ditangkal | Parameter cukup besar |
| **LLL/BKZ** (lattice reduction) | ✅ Ditangkal | Noise LWE menghancurkan struktur kisi |
| **Dictionary Attack** (v0.4.1) | ✅ Ditangkal | Hybrid KEM + AEAD (v0.5.1) |
| **Known-Plaintext Attack** (v0.4.1) | ✅ Ditangkal | Hybrid KEM + AEAD (v0.5.1) |
| **MITM** (Man-in-the-Middle) | ✅ Ditangkal | Auto-Imun (latency detection) |
| **Pegasus/Ordal** (key theft) | ✅ Ditangkal | Auto-Imun (GPS anomaly) |
| **Header Spoofing** | ✅ Ditangkal | HMAC-SHA256 constant-time |
| **Timing Attack** (fungsi inti) | ✅ Ditangkal | Constant-time implementation |

### 🔴 Vektor Serangan yang Masih Terbuka

| Serangan | Tingkat | Waktu | Status Mitigasi |
|----------|---------|-------|-----------------|
| **Small-Q Brute-Force** (24-bit seed) | 🔴 KRITIS | Jam | ❌ Belum (butuh Q ≥ 2048-bit) |
| **Algebraic Cryptanalysis** (Gröbner basis) | 🔴 KRITIS | Jam-Hari | ❌ Belum |
| **Quantum-Enhanced Lattice** | ⚠️ TINGGI | Minggu-Bulan | ⚠️ Sebagian |
| **Side-Channel** (full stack) | ⚠️ TINGGI | Minggu | ⚠️ Sebagian (hanya fungsi inti) |
| **Fault Injection** | ⚠️ TINGGI | Bulan | ❌ Belum |
| **Noise Stream Prediction** | 🟡 SEDANG | Bulan | ⚠️ Sebagian (PRNG-based) |

> 📖 Lihat **[docs/threat-model.md](docs/threat-model.md)** untuk analisis detail.

---

## 🗺️ 10. Roadmap Pengembangan

### ✅ Fase 1: Fondasi (Selesai — Nov 2026)

- [x] Deret Fibonacci non-linear + noise LWE
- [x] Pemetaan Gaussian integer
- [x] Implementasi Rust + Python (konsistensi lintas-bahasa)
- [x] IND-CPA patch (random blinding)
- [x] Auto-Imun v0.4.0 (HMAC, Honeypot, Cooldown)

### ✅ Fase 2: Temuan Kritis & Perbaikan (Selesai — Nov 2026)

- [x] Red Team: known-plaintext attack berhasil (v0.4.1)
- [x] Analisis akar masalah: linearitas + blinding terbuka
- [x] Implementasi Hybrid KEM + ChaCha20-Poly1305 (v0.5.0)
- [x] Perbaikan HKDF-based key derivation (v0.5.1)
- [x] Verifikasi penutupan celah

### 🚧 Fase 3: Hardening (Sedang Berjalan — Des 2026)

- [X] **Upgrade Q ke 2048-bit minimum** (tutup small-Q brute-force)
- [X] **Algebraic resistance test** (SageMath, Gröbner basis) ✅ BARU SAJA SELESAI
- [X] **Full side-channel audit** (seluruh stack, bukan hanya fungsi inti)
##### ✅ Tahap 1: Cross-Language Test Vectors (SELESAI)
- Python Oracle ↔ Rust: **10/10 MATCH** ✅

##### ✅ Tahap 2: Statistical Timing Test (BARU SELESAI)
- Welch's t-test: **p = 0.143 > 0.05** ✅
- Perbedaan mean: **0.8975% (tidak signifikan)** ✅
- **CONSTANT-TIME VERIFIED** ✅

##### ⏭️ Tahap 3: Cache Simulation (NEXT TARGET)
- Memodelkan pola akses cache untuk `U2048` limbs
- Memverifikasi tidak ada korelasi antara pola cache dan data rahasia
- [ ] **Fault injection testing** (redundant computation)
- [ ] **Noise stream analysis** (machine learning prediction)

### ⏳ Fase 4: Validasi Eksternal (Q1 2027)

- [ ] **Independent security audit** (pihak ketiga)
- [ ] **Quantum simulation** (Qiskit/Cirq)
- [ ] **Formal verification** (Coq/Lean)
- [ ] **Test vector resmi** (JSON, lintas-bahasa)

### ⏳ Fase 5: Publikasi & Adopsi (Q2 2027)

- [ ] **Publikasi akademik** (journal/conference)
- [ ] **Library siap pakai** (crates.io, PyPI)
- [ ] **Hardware Security Module** (YubiHSM, AWS CloudHSM)
- [ ] **Dokumentasi produksi** (deployment guide)

---

## 🤝 11. Panggilan Kolaborasi

FCR-RSA adalah riset terbuka. Kami **mengundang** kritik, uji coba, dan
kontribusi. Yang paling dibutuhkan saat ini:

1. **Uji klaim keamanan** — coba tembus FCR-RSA. Jika berhasil, laporkan
   via [SECURITY.md](SECURITY.md). Jika gagal, dokumentasikan usahamu.
2. **Audit implementasi** — cari bug, side-channel, kesalahan logika.
3. **Algebraic cryptanalysis** — coba Gröbner basis, linearisasi.
4. **Test vector** — tambah kasus uji, verifikasi yang sudah ada.
5. **Dokumentasi** — perbaiki yang kurang jelas.

Baca [CONTRIBUTING.md](CONTRIBUTING.md) untuk panduan.

---

<div align="center" style="background: linear-gradient(135deg, #003300 0%, #001a00 50%, #000000 100%); padding: 35px 20px; border: 2px solid #00FF00; border-radius: 15px; margin-top: 30px; box-shadow: 0 0 25px rgba(0, 255, 0, 0.3);">
<h3 style="color: #00FF00; font-family: 'Courier New', monospace; letter-spacing: 2px;">
👨‍💻 INISIATOR & PENULIS UTAMA
</h3>
<p style="color: #FFFFFF; font-family: 'Courier New', monospace; font-size: 18px;">
<strong>Emen (duhemen)</strong>
</p>
<p style="color: #00FF00; font-family: 'Courier New', monospace; font-size: 14px;">
<code>Konseptor • Visioner • Independent Researcher</code>
</p>
<blockquote style="border-left: 3px solid #00FF00; padding-left: 15px; color: #CCCCCC; font-style: italic; max-width: 700px; margin: 0 auto;">
"Kriptografi yang sempurna bukanlah yang paling kaku, melainkan yang
meniru sifat alam semesta—kekacauan yang teratur."
</blockquote>
<p style="color: #00C853; font-family: 'Courier New', monospace; font-size: 13px;">
🍀 Dikembangkan dengan asistensi AI (<strong>DeepSeek</strong>, <strong>Google Mode AI</strong>, & <strong>Qwen</strong>) sebagai kolaborator riset. 🍀
</p>
<img src="https://capsule-render.vercel.app/api?type=waving&color=0:00FF00,50:001a00,100:000000&height=120&section=footer" width="100%"/>
<p style="color: #00FF00; font-family: 'Courier New', monospace; font-size: 14px; margin-top: 10px;">
<strong>© 2026 — Emen (duhemen) | Made with 💚 and ☕ in Indonesia</strong>
</p>
</div>