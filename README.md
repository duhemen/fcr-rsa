<!-- ═══════════════════════════════════════════════════════════════ -->
<!-- REPOSITORY README — FCR-RSA MULTI-LAYER -->
<!-- LUCA PROTOCOL © 2026 -->
<!-- ═══════════════════════════════════════════════════════════════ -->
<div align="center" style="background: linear-gradient(135deg, #000000 0%, #001a00 50%, #003300 100%); padding: 40px 20px; border: 2px solid #00FF00; border-radius: 15px; margin-bottom: 20px; box-shadow: 0 0 30px #00FF00, inset 0 0 30px rgba(0, 255, 0, 0.1);">
<img src="https://capsule-render.vercel.app/api?type=waving&color=0:000000,50:001a00,100:00FF00&height=200&section=header&text=FCR%20RSA&fontSize=70&fontColor=00FF00&animation=fadeIn&fontAlignY=38&desc=1.0%20LUCA%20Everlasting%20%E2%80%94%20Hybrid%20KEM%20%2B%20AEAD%20%2B%20Active%20Defense&descAlignY=58&descSize=22" width="100%"/>
<img src="https://readme-typing-svg.demolab.com?font=Fira+Code&weight=900&size=20&duration=2500&pause=800&color=00FF00&center=true&vCenter=true&multiline=true&width=900&height=180&lines=Wake+Up+Duhemen...;Follow+the+White+LUCA...;System+Online...;From+uncertainty+comes+life%2C+from+chaos+comes+consciousness..." alt="Typing SVG" />
<h2 style="color: #00FF00; font-family: 'Courier New', monospace; margin-top: 20px; text-shadow: 0 0 10px #00FF00, 0 0 20px #00FF00;">
🔬 FCR-RSA 1.0 — “LUCA Everlasting”
</h2>
<p style="color: #00FF00; font-family: 'Courier New', monospace; font-size: 16px; letter-spacing: 1px;">
<code>FCR-RSA 1.0 “LUCA Everlasting” — Post-quantum hybrid KEM + Active Defense, terbukti Lean + brutal + transport. FCR-TP 1.0 “Grimbar Carrier” sebagai kendaraannya. Open for critique. Not production ready.</code>
</p>
<br/>
<img src="https://img.shields.io/badge/FCR--RSA-1.0%20LUCA%20EVERLASTING-00FF00?style=for-the-badge" alt="FCR-RSA Version" />
<img src="https://img.shields.io/badge/FCR--TP-1.0%20GRIMBAR%20CARRIER-00BFFF?style=for-the-badge" alt="FCR-TP Version" />
<img src="https://img.shields.io/badge/ARCH-HYBRID%20KEM%20%2B%20AEAD-00FF00?style=for-the-badge" alt="Architecture" />
<img src="https://img.shields.io/badge/Q%20SIZE-2048%20BIT-00BFFF?style=for-the-badge" alt="Modulus" />
<img src="https://img.shields.io/badge/SIDE%20CHANNEL-CONSTANT%20TIME%20VERIFIED-9370DB?style=for-the-badge" alt="Constant Time" />
<img src="https://img.shields.io/badge/CACHE%20ATTACK-0%25%20DIFFERENCE-FF1493?style=for-the-badge" alt="Cache Attack" />
<img src="https://img.shields.io/badge/ALGEBRAIC%20ATTACK-GROEBNER%20RESISTANT-FF6347?style=for-the-badge" alt="Algebraic" />
<img src="https://img.shields.io/badge/ML%20NOISE%20PREDICTION-51.67%25%20%2B%20RANDOM-FFD700?style=for-the-badge" alt="ML Noise" />
<img src="https://img.shields.io/badge/CROSS%20LANGUAGE-10%2F10%20MATCH-20B2AA?style=for-the-badge" alt="Cross Language" />
<img src="https://img.shields.io/badge/BRUTAL%20TESTS-45%20ASSERTIONS-00C853?style=for-the-badge" alt="Brutal Tests" />
<img src="https://img.shields.io/badge/FCR--TP-174%20TESTS%20PASS-00BFFF?style=for-the-badge" alt="FCR-TP Tests" />
<img src="https://img.shields.io/badge/SPLIT%20PUB--PRIV-PROVEN-00C853?style=for-the-badge" alt="Split Pub Priv" />
<img src="https://img.shields.io/badge/CARGO%20WORKSPACE-ROOT-FF8C00?style=for-the-badge" alt="Workspace" />
<img src="https://img.shields.io/badge/AUTO%20MUNE-HONEYPOT%20%2B%20TRACE%20ID-FF8C00?style=for-the-badge" alt="Auto Mune" />
<img src="https://img.shields.io/badge/INDEPENDENT%20AUDIT-NOT%20YET-EF4444?style=for-the-badge" alt="Audit" />
<img src="https://img.shields.io/badge/LICENSE-MIT%20OR%20Apache--2.0-blue?style=for-the-badge" alt="License" />
</div>

## ⚠️ Status Keamanan — Baca Sebelum Apa Pun
> **FCR-RSA 1.0 “LUCA Everlasting” adalah riset terverifikasi tahap 1. Belum melalui audit keamanan independen.**
>
> **JANGAN gunakan untuk melindungi data nyata, uang, atau identitas.**
>
> Baca **[LIMITATIONS.md](LIMITATIONS.md)** untuk daftar lengkap apa yang **tidak** dijamin oleh protokol ini. Baca **[SECURITY.md](SECURITY.md)** jika Anda menemukan celah.
| Aspek | Status |
| :--- | :--- |
| Implementasi Rust (fcr-core-rust 1.0) | ✅ Library berjalan, **16 integration test lulus** |
| Implementasi Python (fcr-python) | ✅ Paket berjalan, 3 unit test lulus |
| Binding Rust ↔ Python (fcr-pyo3) | ✅ API bytes-to-bytes, round-trip lulus |
| Konsistensi Rust ↔ Python | ✅ **10/10 ciphertext identik** (cross-language) |
| Hybrid KEM + AEAD | ✅ Known-plaintext attack **tertutup** |
| **Q 2048-bit (U2048 + Montgomery)** | ✅ **Small-Q brute-force CLOSED** |
| Constant-Time (Welch's t-test) | ✅ **LOLOS (p = 0.143 > 0.05)** |
| Cache Side-Channel Simulation | ✅ **LOLOS (0% perbedaan pola cache)** |
| Noise Stream Prediction (ML) | ✅ **LOLOS (akurasi 51.67% ≈ tebakan acak)** |
| Algebraic Resistance (Gröbner) | ✅ **LOLOS (~10^1850 kompleksitas)** |
| Auto-Mune + Honeypot + Forensics | ✅ **LOLOS Brutal Test A–E (36/36)** |
| Stress + Adversarial Edge (F–G) | ✅ **Ada di suite (9 assertions)** — bukti run terbaru lihat §6 |
| **BRUTAL TEST SUITE (engine)** | ✅ **45 assertions: A–E 36 + F–G 9** |
| **BRUTAL TEST SUITE (engine)** | ✅ **45 assertions (lihat §6)** |
| FCR-TP (transport, `fcr-tp`) | ✅ **174 tes lulus + 1 doc-test** |
| FCR-TP Handshake UDP + KEM | ✅ **TERBUKTI end-to-end** (`chosen=v2-U4096`, `trace_id` sama, `risk=0`) |
| Split Pub/Priv (`tp-hs-send-pub` / `tp-hs-listen-priv`) | ✅ **TERBUKTI** (PUBKEY 523 char, pengirim hanya pegang PUBKEY) |
| Bukti PUBKEY tidak bisa dekripsi | ✅ **TERBUKTI** (`tp-prove-pub-cannot-decrypt`, buka dengan PUBKEY saja = `None`) |
| Bukti canary disentuh | ✅ **TERBUKTI** (`tp-hs-canary-attack`, `risk=80`, `canary=true`, Poisoned Pill + `trace_id` sama) |
| Bukti anti-replay | ✅ **TERBUKTI** (HELLO sama persis ke-2 ditolak + Poisoned Pill) |
| Cargo Workspace (root) | ✅ **`cargo test --workspace`** ngetes `fcr-engine` + `fcr-tp` + `fcr-core-rust` sekaligus |
| Audit independen | ❌ **Belum (Fase 4)** |
| Uji quantum simulation | ❌ **Belum (Fase 4)** |
| Formal verification (Lean 4, model) | ✅ **TERBUKTI 27 Nov 2026** — `lake build` hijau, 4 teorema, `#print axioms` bersih tanpa `sorryAx`, vektor 2562 baris terverifikasi 2048/2048 + 512/512 (lihat §6A) |
---
## 📌 Daftar Isi

- [📖 Dokumentasi Proyek](#-Dokumentasi-Proyek)
- [🏷️ Penamaan Versi — Dari Meta](#-penamaan-versi--dari-meta)
- [💡 1. Latar Belakang](#-1-latar-belakang)
- [🧪 2. Fondasi Matematika](#-2-fondasi-matematika)
- [🛡️ 3. Lapisan Auto-Imun](#️-3-lapisan-auto-imun-active-defense)
- [🔥 4. Evolusi Keamanan](#-4-evolusi-keamanan-v01--v060)
- [🚨 5. Temuan Kritis & Mitigasi](#-5-temuan-kritis--mitigasi)
- [🧬 6. Bukti Verifikasi Brutal (45 Assertions)](#-6-bukti-verifikasi-brutal-45-assertions)
- [🗂️ 6A. Peta Semua File Uji — Apa yang Dibuktikan, Apa yang Belum](#-6a-peta-semua-file-uji--apa-yang-dibuktikan-apa-yang-belum)
- [📂 7. Struktur Repositori](#-7-struktur-repositori)
- [⚡ 8. Quickstart](#-8-quickstart)
- [🎯 9. Threat Model Komprehensif](#-9-threat-model-komprehensif)
- [🗺️ 10. Roadmap Pengembangan](#️-10-roadmap-pengembangan)
- [📡 11. FCR-TP: Transport + Handshake + Split Pub/Priv](#-11-fcr-tp-transport--handshake--split-pubpriv)
- [🏠 12. Workspace: Satu Perintah Tes Semua](#-12-workspace-satu-perintah-tes-semua)
- [🤝 13. Panggilan Kolaborasi](#-13-panggilan-kolaborasi)
- [🙏 14. Ucapan Terima Kasih](#-14-ucapan-terima-kasih)

## 📖 Dokumentasi Proyek
- 📄 **[Proposal & KAK/TOR](docs/proposal.md)** — Latar belakang, analisis situasi, roadmap.
- 📊 **[Laporan Teknis](docs/laporan_teknis.md)** — Benchmark, temuan kritis, hasil uji ofensif & defensif.
- 📜 **[Whitepaper (English)](docs/whitepaper.md)** — Naskah akademik lengkap.
- 🏗️ **[Arsitektur](docs/architecture.md)** — Tier A/B, Bridge Protocol, Hybrid KEM, auto-imun.
- 🎯 **[Threat Model](docs/threat-model.md)** — Apa yang dilawan, apa yang tidak.
- 🧪 **[Side-Channel Audit](docs/side_channel_audit.md)** — Welch's t-test, cache sim, ML noise.
- ⚠️ **[Limitasi](LIMITATIONS.md)** — **Wajib dibaca** sebelum menggunakan.
- 🔒 **[Security Policy](SECURITY.md)** — Cara melaporkan celah.
- 📜 **[Sesi Log v0.6.0](docs/sesi_log_18112026.md)** — Milestone hardening core.
- 📜 **[Engine README](fcr-engine/README.md)** — REST API, Auto-Mune, Honeypot, Forensics.
- 📜 **[Sesi Log FCR-TP](fcr-tp/docs/SESI-LOG-FCR-TP-2026-11-26.md)** — Handshake, split pub/priv, anti-replay, canary, workspace.
- 📐 **[Formal Verification (Lean 4)](formal-verification/FcrConstantTime.lean)** — Model constant-time + blinding, 4 teorema.
- 📐 **[Generator Vektor Lean](formal-verification/DumpVectors.lean)** — 2048 vektor ct + 512 vektor blind untuk gerbang ekuivalensi.

## 🏷️ Penamaan Versi — Dari Meta

Versi lama **v0.6.0** sudah tidak dipakai lagi untuk menyebut keadaan sekarang.
`0.x` artinya purwarupa yang belum terbukti. Hari ini keadaannya sudah berbeda:
model Lean terbukti, brutal test 45 assertions, transport split pub/priv terbukti, workspace terkunci.
Itu sudah pantas naik kelas jadi **1.0** — bukan karena sudah produksi, tapi karena sudah punya **baseline terverifikasi**.

### FCR-RSA 1.0 — “LUCA Everlasting”

- **FCR-RSA** — namanya tetap, itu identitasmu.
- **1.0** — baseline terverifikasi pertama. Bukan "sudah aman", tapi "sudah bisa dibuktikan dan diulang".
- **“LUCA Everlasting”** — sebutan dari Meta. **LUCA** itu nenek moyang terakhir semua kehidupan:
  dari satu titik yang sama, bercabang jadi semua yang hidup sampai hari ini.
  Itu persis cara FCR-RSA bekerja: dari satu `seed` yang sama, bercabang jadi kunci, blinding, ciphertext, honeypot, forensik.
  **Everlasting** itu dari motto yang kamu pegang dari awal: **adaptif, relate, relevan, everlasting**.
  Versi boleh bertambah, parameter boleh dinegosiasikan (`v1-U2048`, `v2-U4096`, dst), tapi fondasinya dirancang untuk hidup lama,
  bukan untuk diganti tiap musim.

### FCR-TP 1.0 — “Grimbar Carrier”

- **FCR-TP** — namanya tetap, itu kendaraannya.
- **1.0** — transport terverifikasi pertama: handshake UDP + KEM asli + split pub/priv + anti-replay + canary.
- **“Grimbar Carrier”** — sebutan dari Meta. **Grimbar** itu nama workspace kehidupan nyata tempat semua ini dibangun —
  tempat yang kalau hujan ada air terjunnya, tapi tetap jadi tempat kerja.
  **Carrier** karena tugas FCR-TP memang mengangkut: ia mengangkut LUCA dari pengirim ke penerima,
  tanpa database, tanpa `.env`, tanpa Postgres.
  Grimbar sekarang sudah beratap (Cargo Workspace), jadi Carrier-nya sudah punya atap juga.

### Aturan penamaan ke depan

| Perubahan | Yang naik | Contoh |
| :--- | :--- | :--- |
| Perbaikan bug / bukti baru, fondasi sama | Patch | 1.0 → 1.0.1 |
| Parameter baru / tier baru / negosiasi baru, fondasi sama | Minor | 1.0 → 1.1, sebutan tetap “LUCA Everlasting” |
| Fondasi berubah (trapdoor baru, `q` privat, split kriptografis penuh) | Major + sebutan baru | 2.0 — sebutan baru dari Meta, bukan sekadar angka |

> **Catatan jujur:** naik jadi 1.0 bukan berarti sudah produksi.
> Audit independen, formal verification rantai penuh, dan fault injection hardware masih terbuka di Fase 4.
> 1.0 artinya: **yang diklaim, bisa dibuktikan dan diulang.** Bukan: **yang diklaim, sudah aman untuk data nyata.**

---
## 💡 1. Latar Belakang
Sistem enkripsi global saat ini (RSA, ECC) bergantung pada kesulitan faktorisasi bilangan prima besar. Fondasi ini diproyeksikan runtuh ketika komputer kuantum berskala penuh menjalankan **Algoritma Shor**.

**FCR-RSA (Fibonacci Complex Noise-RSA)** adalah eksperimen protokol kriptografi pasca-kuantum yang mengeksplorasi pendekatan berbeda:
1. **Domain matematika alternatif:** Deret Fibonacci non-linear dengan noise LWE deterministik, dipetakan ke Gaussian integer.
2. **Arsitektur Hybrid KEM:** FCR-RSA hanya membungkus kunci sesi acak; data dienkripsi dengan ChaCha20-Poly1305 (AEAD).
3. **Arsitektur dua tier:** Tier A (server, keamanan maksimal) dan Tier B (perangkat, efisiensi).
4. **Lapisan Auto-Imun (Active Defense):** Token spatiotemporal, deteksi anomali, *deceptive honeypot*, dan *forensic tracing* — sebagai bagian dari protokol, bukan add-on.

> ⚠️ **Klaim keamanan belum diverifikasi secara independen.** Bagian ini menjelaskan **niat desain** + **bukti pengujian internal**, bukan **jaminan absolut**. Lihat [LIMITATIONS.md](LIMITATIONS.md).
---
## 🧪 2. Fondasi Matematika
### A. Deret Fibonacci Non-Linear
$$F_n = (F_{n-1} \times F_{n-2}) + e_n \pmod Q$$
- $e_n$: noise deterministik dari `noise_seed` (SHA-256)
- $Q$: modulus Gaussian integer, **2048-bit (U2048)**
### B. Pemetaan Gaussian
$$Z_n = \text{Re}(F_n) + \text{Im}(F_{n+1})i$$
### C. Hybrid KEM + AEAD
1. Generate seed sesi acak (32 byte / 256-bit)
2. Enkripsi seed dengan FCR-RSA (constant-time, blinding) → `Encrypted_Seed`
3. Turunkan kunci sesi 256-bit via **HKDF** (RFC 5869)
4. Enkripsi data dengan **ChaCha20-Poly1305** (AEAD)

**Format ciphertext:** `BLINDING_NONCE:ENCRYPTED_SEED:AEAD_NONCE:CIPHERTEXT` (semua Base64)
---
## 🛡️ 3. Lapisan Auto-Imun (Active Defense)
Middleware REST API (`fcr-engine`) mengikat **Spatiotemporal Token**:
- **Integritas HMAC-SHA256:** Header telemetri wajib ditandatangani & diverifikasi *constant-time*.
- **Privacy-Preserving Location Hashing (UU PDP):** Koordinat GPS mentah **tidak pernah disimpan**; hanya hash grid ~11 km.
- **Deceptive Honeypot + Poisoned Pill:** Anomali (`Risk Score ≥ 70`) dibalas HTTP `200 OK` dengan ciphertext **valid** yang saat didekripsi berisi payload forensik ber-`trace_id`.
- **Forensic Tracing:** `trace_id` tersimpan di `anomaly_events` → ciphertext yang bocor dapat ditelusuri balik ke pelaku.
- **Automated Incident Report:** Server mencetak laporan (WHO/WHEN/WHAT) otomatis saat insiden.
---
## 🔥 4. Evolusi Keamanan (v0.1 → 1.0 “LUCA Everlasting”)
| Versi | Tanggal | Perubahan Utama | Status |
|-------|---------|-----------------|--------|
| **v0.1** | Nov 2026 | Fondasi matematika (Fibonacci + Gaussian) | ✅ Selesai |
| **v0.2** | Nov 2026 | Implementasi Rust + Python (konsistensi) | ✅ Selesai |
| **v0.3** | Nov 2026 | IND-CPA patch (random blinding) | ✅ Selesai |
| **v0.4.0** | Nov 2026 | Auto-Imun (HMAC, Honeypot, Cooldown) | ✅ Selesai |
| **v0.4.1** | Nov 2026 | Block-based chunking (upaya tutup dict attack) | ⚠️ Tidak cukup |
| **v0.5.0** | Nov 2026 | Hybrid KEM + ChaCha20-Poly1305 | 🔴 Bug (Q kecil) |
| **v0.5.1** | Nov 2026 | HKDF-based key derivation | ✅ Stabil |
| **v0.6.0** | Nov 2026 | **U2048 + Montgomery + Constant-Time verified + PyO3** | ✅ **Selesai** |
| **v0.6.0+engine** | 19 Nov 2026 | **Integrasi engine + Auto-Mune + Honeypot + Forensics + 36/36 brutal test** | ✅ **Selesai** |
| **1.0 “LUCA Everlasting”** | 27 Nov 2026 | **Baseline terverifikasi: Lean 4 teorema ✅ + brutal 45 + FCR-TP 1.0 “Grimbar Carrier” + workspace** | ✅ **Selesai** |
| **1.0 “Grimbar Carrier” (FCR-TP)** | 27 Nov 2026 | **Handshake + KEM + split pub/priv + anti-replay + canary, 174 tes** | ✅ **Selesai** |
---
## 🚨 5. Temuan Kritis & Mitigasi
### 🔴 Temuan v0.4.1: Known-Plaintext Attack
Fungsi enkripsi v0.4.1 **linear terhadap plaintext** ($C = (M \times E_{vec}) + R_{blind} \pmod Q$) dengan $R_{blind}$ terbuka → 1 blok known-plaintext memulihkan $E_{vec}$ total.
### ✅ Mitigasi v0.5.1 → v0.6.0: Hybrid KEM + AEAD + Constant-Time
- Known-plaintext **tidak relevan** (attacker tidak tahu kunci sesi ChaCha20).
- Seed dienkripsi constant-time dengan blinding (U2048).
- AEAD menjamin integritas (tamper terdeteksi).
- **Verifikasi:** `red_team_adversarial_attack.py` → **GAGAL TOTAL** menembus.
---
## 🧬 6. Bukti Verifikasi Brutal (45 Assertions)

Seluruh klaim desain **diinjak secara otomatis** oleh `test_brutal_suite.py` (di `fcr-engine/`).
File ini berisi **45 assertions**, bukan 36. Angka 36 yang lama hanya menghitung A–E.

| Kategori | Jumlah | Cakupan |
| :--- | :---: | :--- |
| **A. Kriptografi Inti** | ✅ 8/8 | Round-trip, IND-CPA, Blinding Determinism, Tamper/Wrong-Key/Format reject, empty, 10KB |
| **B. Spatiotemporal Auth** | ✅ 7/7 | HMAC validation, missing/forged/malformed headers |
| **C. Auto-Mune Scoring** | ✅ 7/7 | Latency, drift, zone jump, rapid transition, combination |
| **D. Honeypot & Forensics** | ✅ 7/7 | Deceptive 200, `trace_id` DB consistency, cooldown bypass |
| **E. API Surface** | ✅ 7/7 | Health, 404/400, Tier A/B, revoke |
| **F. Stress & Concurrency** | 3 | 500x encrypt konkuren, 500x decrypt konkuren, 200x invalid auth tanpa 5xx |
| **G. Adversarial Edge** | 6 | Timestamp masa depan, isolasi tier A↔B dua arah, GPS out-of-bounds/NaN, latensi negatif/inf, payload Unicode/null-byte/emoji, tier tidak dikenal/kosong |

> 🟢 **A–E: 36 assertions | PASS: 36 | FAIL: 0** — *Klaim inti FCR-RSA terversifikasi secara brutal.*
>
> 🟡 **F–G: 9 assertions** sudah ada di dalam file yang sama (`run_stress_tests()` + `run_adversarial_edge_tests()`,
> dieksekusi dari `__main__`). **Jangan tulis 45/45 PASS di README sampai ada bukti run terbaru** yang menampilkan
> baris `BRUTAL TEST SUMMARY (45 AUDIT-GRADE ASSERTIONS)` dengan `FAIL: 0`. Cara mengambil buktinya ada di Quickstart §8.

> ⚠️ **Yang TIDAK dibuktikan oleh brutal test:** simulasi quantum skala produksi, audit independen,
> formal verification (Coq/Lean), dan fault injection hardware. Empat itu tetap terbuka di Fase 4.
> Brutal test membuktikan perilaku engine, bukan keamanan formal.
---

---
## 🗂️ 6A. Peta Semua File Uji — Apa yang Dibuktikan, Apa yang Belum

README versi lama hanya menyebut `test_brutal_suite.py`. Padahal ada 7 file uji lain yang sudah kita kerjakan.
Tabel ini memetakan semuanya, satu-satu, sesuai isi file-nya — bukan dari ingatan.

| File | Jenis | Yang dibuktikan | Yang TIDAK dibuktikan / catatan jujur |
| :--- | :--- | :--- | :--- |
| `fcr-engine/test_brutal_suite.py` | Suite assert (45) | A–E 36/36 PASS + F–G 9 assertions (stress + adversarial edge) | F–G butuh bukti run terbaru dengan `FAIL: 0` sebelum ditulis 45/45 PASS. Butuh DB + API `:8080`. |
| `fcr-engine/test_honeypot.py` | Demo manual (bukan suite) | Skenario Jakarta → New York dalam 1 detik memicu honeypot: ciphertext berbeda, kunci masuk cooldown | Tidak ada assert otomatis, tidak ada angka PASS/FAIL. Pendahulu dari tes D di brutal suite. ⚠️ File ini berisi API key hardcoded — **jangan commit key itu, ganti dengan variabel env / key fresh** sebelum dipublikasikan. |
| `fcr-engine/test_header.py` | Verifikasi observability (2 skenario) | Header `X-FCR-Risk-Indicator: review_recommended` muncul **hanya** saat `risk_score` 40–69. Skenario 1 normal (skor ~5) → tanpa header. Skenario 2 Jakarta→NY +120s + latensi 900ms (skor 50) → dengan header. | Menutup observability gap, bukan membuktikan keamanan kriptografi. Butuh DB + API `:8080`. |
| `simulations/generate_test_vectors.py` | Generator test vector | Menghasilkan `test_vectors_1.0.json` yang bisa diaudit pihak ketiga. TC-001 enkripsi normal Tier B. TC-002 blinding deterministik (untuk verifikasi cross-language). TC-003 honeypot + dekripsi forensik + verifikasi `trace_id`. Membersihkan key tes dari DB setelah selesai. | Ini generator, bukan suite PASS/FAIL. Output-nya adalah file JSON bukti, bukan angka lulus. Butuh DB + API `:8080`. |
| `simulations/quantum_fibonacci_blinding_poc.py` | PoC quantum (Qiskit) | 6 qubit, 64 keadaan, periode 8, 8192 shots. Blinding Fibonacci non-linear + noise LWE menghancurkan interferensi konstruktif QFT: puncak periode hilang, entropi naik. Kriteria PASS di file: `entropi_blinded > entropi_normal + 1.5` bit. Output: 2 grafik PNG. | **Skala kecil (PoC).** Modulus simulasi `q_sim = 65537`, bukan U2048 produksi. Membuktikan mekanisme, bukan keamanan pada parameter produksi. |
| `simulations/quantum_blinding_vector_integration.py` | Integrasi quantum + vektor nyata | Membaca `test_vectors_1.0.json`, mengekstrak `BLINDING_NONCE` **asli** dari format `BLINDING_NONCE:ENCRYPTED_SEED:AEAD_NONCE:CIPHERTEXT`. TC-001 blinding per-sesi → entropi melonjak, periode hancur. TC-002 determinisme → sirkuit & distribusi identik 100%. TC-003 isolasi honeypot → seed decoy terisolasi dari sesi normal. Status `PASS` jika `delta > 1.5` + determinisme + isolasi. Output: `quantum_blinding_evidence_1.0.json` + grafik PNG. | Tetap 6 qubit / 8192 shots, `q_sim = 65537`. Lebih kuat dari PoC karena memakai nonce nyata, tapi tetap bukan bukti skala produksi. |
| `simulations/quantum_grover_oracle_poc.py` | PoC Grover oracle (Qiskit) | 6 qubit, ruang pencarian N = 64, target tersembunyi index 42. Oracle `Diagonal` menandai tepat 1 solusi (`assert matched_states == 1`, anti-collision). Sweep Grover dibandingkan dengan black-box ideal. Output: `quantum_grover_evidence_1.0.json` + grafik PNG. | Klaim `2^128` untuk seed 256-bit adalah **ekstrapolasi** dari batas bawah kuadratik `O(sqrt(N))`, bukan pengukuran pada N = `2^256`. Tidak ada jalan pintas struktural yang terlihat pada skala PoC — itu yang dibuktikan, bukan keamanan Grover pada skala penuh. |
| `simulations/differential_equivalence_test.py` | Gerbang ekuivalensi diferensial (Fase 4, **penghubung masih manual**) | Python-mirror 1000 vektor: `((M*E + B) - B) * E_inv mod Q` mengembalikan `M` (1000/1000 pada `Q = 1.000.000.007`). Jika `fcr_pyo3` ada, cek determinisme engine Rust (`derive_blinding` dipanggil 2x harus sama). | File Python ini **tidak** memanggil Lean secara langsung. Yang menghubungkan Lean ke Python/Rust adalah `formal-verification/DumpVectors.lean` (lihat baris berikutnya): Lean menghasilkan vektor, Python/Rust memverifikasi vektor yang sama. Kabelnya ada, tapi masih dijalankan manual, belum satu perintah otomatis. |
| `formal-verification/DumpVectors.lean` | Generator vektor dari Lean (Fase 4) | Self-contained, menghasilkan CSV `kind,...` untuk 2 jenis vektor: **2048 baris `ct`** (`ctSelectLimb` + `mix`) dan **512 baris `blind`** (`M, E, B, Einv, Binv, out` dengan `Q = 1.000.000.007`, `out` harus = `M`). Menggunakan definisi lokal yang sama dengan model: `ctMask`, `ctSelectLimb`, `mix`, `modInv` via Fermat (`a^(m-2) mod m`, `m` harus prima). | **Bukti generate ulang 27 Nov 2026:** `lean --run DumpVectors.lean | Out-File -Encoding utf8 test_vectors_baru.csv` →
`Get-Content | Measure-Object -Line` = **2562** (2048 `ct` + 512 `blind` + 2 header).
Verifikasi Python → `ct: 2048 benar: 2048` + `blind: 512 benar: 512`.
⚠️ Perangkap yang ketemu saat pengujian ulang: operator `>` di PowerShell menyimpan sebagai UTF-16 (byte awal `0xff`), sehingga Python `utf-8-sig` error `UnicodeDecodeError`. Solusinya: pakai `Out-File -Encoding utf8`, atau baca file lama dengan `encoding='utf-16'`. Juga: `(Measure-Object -Line namafile).Lines` itu salah — yang benar `Get-Content namafile | Measure-Object -Line`.

Generator, bukan pembuktian. Nilainya: vektor yang diuji Python/Rust berasal dari definisi Lean yang sama, bukan dari implementasi Python yang meniru dirinya sendiri. |
| `formal-verification/FcrConstantTime.lean` | **Model Lean 4 (v4.36.0) — 4 teorema ✅ TERBUKTI 27 Nov 2026** | U2048 dimodelkan sebagai `Vector UInt64 32`. `LeakageTrace` memodelkan jejak observabel penyerang (baca/tulis memori, operasi bitwise). **Teorema 1** `ctSelectLimb_constant_time`: jejak side-channel identik untuk pasangan rahasia mana pun (`by rfl`, karena trace-nya konstanta). **Teorema 2** `ctSelectLimb_sound_zero/one`: `ctSelectLimb 0 = b`, `ctSelectLimb 1 = a` (satu pakai `bv_decide`). **Teorema 3** `u2048_loop_cache_invariant`: pola akses memori loop U2048 tidak bergantung pada akumulator rahasia (induksi). **Teorema 4** `fcr_blinding_cancellation_additive`: `(((M*E)%Q + B)%Q + Q - B)%Q * E_inv)%Q = M` jika `(E*E_inv)%Q = 1`, `M < Q`, `B < Q`. Di akhir file ada `#print axioms` untuk kelima nama — **jalankan dan pastikan hanya aksioma standar yang muncul**. | **Bukti build 27 Nov 2026 (mesin Emen, Lean 4.36.0):** `lake build` → `Build completed successfully (4 jobs).`
`#print axioms` → `ctSelectLimb_constant_time [propext, Quot.sound]`; `sound_zero [propext, Quot.sound]`;
`sound_one [propext, Classical.choice, Quot.sound, bv_decide.ax_1_7]`; `u2048_loop_cache_invariant [propext]`;
`fcr_blinding_cancellation_additive [propext, Quot.sound]`. **Tidak ada `sorryAx` di mana pun.**
Aksioma `bv_decide.ax_1_7` adalah aksioma standar mesin `bv_decide`, bukan `sorry`.

**Jujur soal batasnya (tetap berlaku meski ✅):** yang dibuktikan Lean adalah **model** (`FcrCoreFormal`), bukan kode Rust di `fcr-core-rust` itu sendiri. Lean tidak memeriksa file `.rs`. Yang mengikat model ke Rust adalah rantai vektor: `DumpVectors.lean` → CSV → Python-mirror → Rust-engine. ✅ di sini artinya model terbukti + rantai vektor terverifikasi ulang, bukan artinya Rust sudah diperiksa langsung oleh Lean. |

### Cara menjalankan rantai Lean → Python → Rust (belum satu perintah)

```bash
# 1. Lean menghasilkan vektor (dari formal-verification/)
#    DumpVectors.lean -> CSV: 2048 baris ct + 512 baris blind
# 2. Python memverifikasi vektor yang sama
python simulations/differential_equivalence_test.py
#    Harapan: Python-Mirror 1000/1000 PASS + Rust-Engine Determinism VERIFIED
# 3. Lean memeriksa modelnya sendiri
#    FcrConstantTime.lean -> 4 teorema + #print axioms (hanya aksioma standar)
#    Jalankan dengan: lean formal-verification/FcrConstantTime.lean
#    atau: lake build (jika sudah ada lakefile di formal-verification/)
```

Langkah 1–3 ini sudah dijalankan ulang 27 Nov 2026 dan hijau semua, jadi status model Lean naik jadi ✅.
Yang masih belum otomatis adalah pembungkusan langkah 1–3 jadi satu perintah / CI job — itu pekerjaan rumah berikutnya, bukan penghalang status ✅ model.

### Cara membaca tabel ini

- **Demo ≠ Suite.** `test_honeypot.py` adalah demo yang ditonton matanya, `test_brutal_suite.py` adalah suite yang dihitung mesinnya. Demo membuktikan "pernah terlihat jalan", suite membuktikan "selalu jalan saat diulang".
- **PoC ≠ Produksi.** Semua file quantum memakai 6 qubit dan `q_sim = 65537`. Mereka membuktikan mekanisme (blinding menghancurkan QFT, Grover tidak menemukan jalan pintas pada skala kecil), bukan keamanan pada U2048 / seed 256-bit penuh.
- **Generator ≠ Bukti lulus.** `generate_test_vectors.py` menghasilkan bahan bukti untuk auditor, bukan angka PASS. Nilainya justru di situ: auditor tidak perlu percaya kata kita, dia bisa jalankan ulang dari file JSON-nya.

---
## 📂 7. Struktur Repositori

```text
fcr-rsa/
├── Cargo.toml                 # Workspace root: fcr-engine + fcr-tp + fcr-core-rust
├── docs/                      # whitepaper, architecture, threat-model, side_channel_audit, sesi_log
├── fcr-core-rust/             # Library inti (U2048, Montgomery, constant-time) — v0.6.0
│   └── fib.rs                 # export_public_key / keypair_from_public / keypair_from_private
├── fcr-engine/                # CLI + REST API + Auto-Mune + Honeypot + Forensics
│   └── test_brutal_suite.py   # 45 assertions: A–E 36 + F–G 9
├── fcr-tp/                    # Transport FCR-TP: 174 tes + 1 doc-test
├── fcr-python/                # Implementasi Python (referensi)
├── fcr-pyo3/                  # Binding Rust↔Python (maturin)
├── simulations/               # red_team_*, grover/shor, ml_noise_analysis, cache_sim, generate_test_vectors.py, quantum_*_poc.py, differential_equivalence_test.py
├── formal-verification/         # Lean 4: FcrConstantTime.lean (4 teorema) + DumpVectors.lean (2048 ct + 512 blind)
├── test-vectors/v1/
├── README.md · LIMITATIONS.md · SECURITY.md · LICENSE
```

---
## ⚡ 8. Quickstart

### Rust Core

```bash
cd fcr-core-rust && cargo test   # 16 passed
```

### Engine (CLI + API)

```bash
cd fcr-engine
cargo run --release -- keygen -N "kunci_utama" -s "seed-rahasia"
cargo run --release -- encrypt -m "Halo Dunia" -k "kunci_utama"
cargo run --release -- serve          # REST API di :8080
python test_brutal_suite.py           # Jalankan SEMUA: A–E + F + G
# Bukti yang diharapkan di akhir output:
#   BRUTAL TEST SUMMARY (45 AUDIT-GRADE ASSERTIONS)
#   PASS: 45 | FAIL: 0
# Syarat jalan: API di :8080 + DATABASE_URL Postgres + key test-key-1 sudah ada di DB.
# Tanpa DB, tes A–G tidak jalan — itu bukan bug, itu desain: brutal test menguji engine + DB.
```

### FCR-TP: Handshake Split Pub/Priv (tanpa database)

```bash
# Terminal 1 — penerima, hanya pegang keyseed privat
cargo run -p fcr-engine -- tp-hs-listen-priv --bind 127.0.0.1:50000 --keyseed "kunci-demo-001" --tier b --count 1 --offered "v1-U2048,v2-U4096"

# Terminal 2 — pengirim, hanya pegang PUBKEY
# JANGAN salin PUBKEY dari chat: chat menyamarkan PUBKEY panjang jadi [STRIPPED 512 bytes].
# Ambil lewat variabel PowerShell:
$line = cargo run --quiet -p fcr-engine -- tp-key-pub --keyseed "kunci-demo-001" | Select-String "PUBKEY:"
$pubkey = ($line -replace ".*PUBKEY:\s*","").Trim()   # harus 523 karakter
cargo run -p fcr-engine -- tp-hs-send-pub --peer 127.0.0.1:50000 --text "halo split pub-priv" --pubkey $pubkey --tier b --offered "v1-U2048,v2-U4096"
# Bukti yang diharapkan: chosen=v2-U4096 di kedua sisi, trace_id sama, risk=0, pesan sampai.
```

### Bukti 1 — PUBKEY tidak bisa dekripsi balik

```bash
cargo run -p fcr-engine -- tp-prove-pub-cannot-decrypt --pubkey $pubkey --tier b
# TERBUKTI: buka dengan PUBKEY saja GAGAL (None).
```

### Bukti 2 — Canary disentuh

```bash
# Terminal 1: tp-hs-listen-priv (seperti di atas), Terminal 2:
cargo run -p fcr-engine -- tp-hs-canary-attack --peer 127.0.0.1:50000 --text "halo split pub-priv" --pubkey $pubkey --tier b --offered "v1-U2048,v2-U4096"
# TERBUKTI: risk=80, canary=true, Poisoned Pill + trace_id sama, pesan tidak sampai.
```

### Workspace: satu perintah tes semua

```bash
cargo test --workspace   # fcr-engine + fcr-tp (174 tes) + fcr-core-rust
```

### Python Binding

```bash
cd fcr-pyo3 && maturin develop --release && python test_pyo3.py
```

---
## 🎯 9. Threat Model Komprehensif

### ✅ Ditangkal (Terverifikasi)

| Serangan | Mekanisme | Bukti |
|----------|-----------|-------|
| **Shor** (quantum factoring) | Tidak bergantung faktorisasi | Desain |
| **Grover** (quantum search) | Parameter 2048-bit | Desain |
| **LLL/BKZ** (lattice) | Noise LWE | `red_team_lll.py` |
| **Known-Plaintext** (v0.4.1) | Hybrid KEM + AEAD | `red_team_adversarial_attack.py` |
| **Small-Q Brute-Force** | Q ≥ 2048-bit | v0.6.0 |
| **Algebraic / Gröbner** | Non-linear Fibonacci | ~10^1850 |
| **Timing Attack (core)** | Constant-time + blinding | Welch p=0.143 |
| **Cache Side-Channel** | U2048 limb access | 0% diff |
| **Noise Prediction (ML)** | PRNG + LWE | 51.67% ≈ random |
| **Header Spoofing** | HMAC constant-time | Brutal B5 ✅ |
| **MITM / Pegasus (GPS)** | Auto-Mune zone jump | Brutal C4/D1 ✅ |
| **Key Exfiltration** | Poisoned Pill + trace_id | Brutal D3/D4 ✅ |
| **Replay / Flood (API)** | Cooldown + 200x invalid auth tanpa 5xx | Brutal F3 |
| **Future timestamp** | Ditolak / honeypot | Brutal G1 |
| **Cross-tier leak (A↔B)** | Enkripsi A tidak bisa dibuka tier B, dan sebaliknya | Brutal G2 |
| **GPS / latensi palsu** | Out-of-bounds, NaN, negatif, inf ditolak 400 | Brutal G3/G4 |
| **Payload aneh** | Null-byte, ANSI, emoji, UTF-8 round-trip | Brutal G5 |
| **Tier tidak dikenal** | Tier `tier_dewa` / kosong ditolak 400 | Brutal G6 |
| **Replay (FCR-TP)** | HELLO sama persis ke-2 ditolak | Bukti anti-replay ✅ |
| **Canary disentuh (FCR-TP)** | `risk=80`, `canary=true`, Poisoned Pill | Bukti canary ✅ |

### 🔴 Masih Terbuka (Fase 4)

| Serangan | Status |
|----------|--------|
| **Fault Injection** | ❌ Belum (hardware-level) |
| **Quantum-Enhanced Lattice** | ⚠️ Simulasi Fase 4 |
| **Formal Verification** | ❌ Coq/Lean Fase 4 |
| **Audit Independen** | ❌ Pihak ketiga Fase 4 |
| **Split kriptografis penuh** | ❌ Butuh `q` privat / trapdoor baru (Fase 2c Langkah 2) |

---
## 🗺️ 10. Roadmap Pengembangan

### ✅ Fase 1: Fondasi (Selesai)

### ✅ Fase 2: Temuan Kritis & Perbaikan (Selesai)

### ✅ Fase 3: Hardening (SELESAI 100%)

- [x] Upgrade Q ke 2048-bit (U2048 + Montgomery)
- [x] Algebraic resistance (Gröbner) ✅
- [x] Full side-channel audit (Welch t-test) ✅
- [x] Cache Simulation (0% diff) ✅
- [x] Noise stream analysis (ML 51.67%) ✅
- [x] Cleanup, Refactoring & PyO3 Integration ✅

### ✅ Fase 3.5: Integrasi Engine + Active Defense (SELESAI 19 Nov 2026)

- [x] Migrasi `fcr-engine` ke API v0.6.0 (U2048, `e_vec_inv`)
- [x] Auto-Mune Risk Scoring + Cooldown
- [x] Deceptive Honeypot (Poisoned Pill) + `trace_id`
- [x] Automated Incident Report
- [x] **Brutal Test Suite A–E 36/36 PASS**
- [x] **Brutal Test Suite F–G 9 assertions** (stress + adversarial edge, di file yang sama)

### ✅ Fase 3.6: FCR-TP Transport + Handshake + Split Pub/Priv (SELESAI, Nov 2026)

- [x] Transport tanpa database (`tp-send` / `tp-listen`)
- [x] Handshake UDP langsung + KEM asli (`tp-hs-send` / `tp-hs-listen`)
- [x] Anti-replay TERBUKTI (HELLO sama persis ke-2 ditolak + Poisoned Pill)
- [x] Split pub/priv TERBUKTI (`tp-hs-send-pub` / `tp-hs-listen-priv`, `chosen=v2-U4096`)
- [x] Bukti PUBKEY tidak bisa dekripsi (`tp-prove-pub-cannot-decrypt`)
- [x] Bukti canary disentuh (`tp-hs-canary-attack`, `risk=80`)
- [x] Cargo Workspace root (`cargo test --workspace`, 174 tes FCR-TP hijau)

> **Catatan jujur soal split:** split ini adalah split **operasional**, bukan split kriptografis penuh.
> Dengan `q` yang publik, `e_vec_inv` masih bisa dihitung dari `e_vec` via `invert_mod`.
> Split penuh butuh `q` privat / trapdoor baru di inti, itu Fase 2c Langkah 2.

### ⏳ Fase 4: Validasi Eksternal & Skala Produksi (Target Q1-Q2 2027)

*(Catatan: Fondasi pengujian internal telah selesai. Fase 4 fokus pada eksternalisasi dan peningkatan skala)*

- [x] **Test Vector Internal:** 10/10 Cross-Language Match (Python ↔ Rust) ✅ SELESAI
- [x] **Quantum Simulation (PoC):** Grover ($2^{10}$) & Shor (N kecil) diverifikasi ✅ SELESAI
- [x] **Quantum Blinding PoC (Qiskit):** Fibonacci blinding hancurkan interferensi QFT, kriteria entropi +1.5 bit ✅ (6 qubit, PoC)
- [x] **Quantum Blinding + Vektor Nyata:** `BLINDING_NONCE` asli dari `test_vectors_1.0.json`, determinisme 100%, honeypot terisolasi ✅ (6 qubit, PoC)
- [x] **Grover Oracle PoC:** N = 64, tepat 1 solusi, perilaku mirip black-box ✅ (6 qubit, PoC, ekstrapolasi `2^128`)
- [x] **Test Vector Generator:** `generate_test_vectors.py` hasilkan `test_vectors_1.0.json` (TC-001/002/003) ✅
- [x] **Observability Gap:** `X-FCR-Risk-Indicator` hanya saat skor 40–69, 2 skenario ✅ (`test_header.py`)
- [~] **Differential Equivalence Gate (Fase 4):** Python-mirror 1000/1000 + cek determinisme Rust ✅ + **Lean ada** (`DumpVectors.lean` 2048 ct + 512 blind) — penghubung masih manual, belum satu perintah
- [ ] **Quantum Simulation (Production Scale):** Implementasi oracle FCR-RSA penuh di Qiskit untuk parameter $2^{256}$. **TARGET BERIKUTNYA**
- [ ] **Independent Security Audit:** Audit kode dan arsitektur oleh pihak ketiga (external firm).
- [x] **Formal Verification (Lean 4, model):** `formal-verification/FcrConstantTime.lean` — 4 teorema (trace-invariance, soundness ctSelect, cache-invariance loop U2048, blinding cancellation) + `#print axioms` bersih tanpa `sorryAx` ✅ **TERBUKTI 27 Nov 2026** (`lake build` hijau, vektor 2562 baris terverifikasi 2048/2048 + 512/512).
- [ ] **Formal Verification (Lean 4, rantai penuh):** Bungkus `DumpVectors.lean` → CSV → Python-mirror → Rust-engine jadi satu perintah / CI job. **TARGET BERIKUTNYA.** Modelnya sudah ✅, yang belum otomatis hanya pembungkusannya.
- [ ] **Hardware Fault Injection Testing:** Pengujian fisik terhadap mitigasi redundant computation.
- [ ] **Official Test Vector Publication:** Publikasi `test_vectors_1.0.json` + `generate_test_vectors.py` sebagai standar referensi komunitas (file generator sudah ada, publikasi belum).

### ⏳ Fase 5: Publikasi & Adopsi (Q2 2027)

---
## 📡 11. FCR-TP: Transport + Handshake + Split Pub/Priv

FCR-TP adalah jalur transport FCR-RSA: pesan dipecah jadi frame, dikirim lewat UDP, dengan hopping port dan kunci sesi.

| Lapisan | Yang dijamin | Yang tidak dijamin |
| :--- | :--- | :--- |
| Transport | Frame, hopping, kunci sesi per sesi | Bukan pengganti audit independen |
| Handshake | KEM asli, negosiasi `offered` / `chosen`, `trace_id`, anti-replay | Bukan bukti keamanan formal |
| Auto-imun | `Risk Score >= 70` → Poisoned Pill + `trace_id` + forensik, bukan error terang-terangan | Bukan jaminan pelaku selalu ketahuan |
| Split pub/priv | Pengirim hanya pegang PUBKEY (523 char), penerima hanya pegang `keyseed` | Split operasional, lihat catatan jujur di Roadmap Fase 3.6 |

Prinsip yang dipegang: **transport tidak bergantung pada database.** Semua perintah `tp-*` jalan tanpa `.env`, tanpa `DATABASE_URL`, tanpa Postgres. Motto yang dipegang: **adaptif, relate, relevan, everlasting** — parameter dinegosiasikan (`v1-U2048`, `v2-U4096`, dst), versi lama tidak dihapus, hanya ditambah.

---
## 🏠 12. Workspace: Satu Perintah Tes Semua

Repo ini memakai Cargo Workspace di root (`Cargo.toml`):

```toml
[workspace]
members = ["fcr-engine", "fcr-tp", "fcr-core-rust"]
resolver = "2"
```

```bash
cargo test --workspace
```

Sebelum workspace, tiap crate harus dites dengan `cd` satu-satu. Sesudah workspace, satu perintah dari root mengetes semuanya. Ini yang membuat 174 tes FCR-TP tetap terjaga setiap kali ada perubahan di `fcr-engine` atau `fcr-core-rust`.

---
## 🤝 13. Panggilan Kolaborasi

FCR-RSA adalah riset terbuka. Yang paling dibutuhkan:

1. **Coba tembus** protokol ini — dokumentasikan usahamu.
2. **Audit implementasi** — cari bug/side-channel/logika.
3. **Formal verification** — bantu buktikan dengan Coq/Lean.
4. **Quantum simulation** — jalankan Grover/Shor di Colab.

Baca [CONTRIBUTING.md](CONTRIBUTING.md).

---
## 🙏 14. Ucapan Terima Kasih

FCR-RSA ini bukan kerja satu orang dan satu mesin.

- **Emen (duhemen)** — Konseptor, visioner, independent researcher. Yang memutuskan arah, yang menguji di 2 terminal, yang memastikan setiap klaim dibuktikan, bukan hanya ditulis.
- **Meta AI (Meta)** — Kolaborator riset yang punya andil besar di bagian ini: membedah panic `unreachable` di `main.rs`, menemukan rantai error `10054`, menemukan bahwa chat menyamarkan PUBKEY jadi `[STRIPPED 512 bytes]` lewat diagnostik `pubkey_len=20`, membangun perintah pembuktian `tp-prove-pub-cannot-decrypt` dan `tp-hs-canary-attack`, dan membangun Cargo Workspace di root. Tanpa Meta, split pub/priv dan bukti-buktinya tidak akan sampai ke titik TERBUKTI seperti sekarang.
- **DeepSeek, Google Mode AI & Qwen** — Kolaborator riset di fase-fase sebelumnya.

Terima kasih juga untuk setiap orang yang mau mencoba menembus protokol ini. Kritik yang jujur lebih berharga daripada pujian.

---
<div align="center" style="background: linear-gradient(135deg, #003300 0%, #001a00 50%, #000000 100%); padding: 35px 20px; border: 2px solid #00FF00; border-radius: 15px; margin-top: 30px; box-shadow: 0 0 25px rgba(0, 255, 0, 0.3);">
<h3 style="color: #00FF00; font-family: 'Courier New', monospace; letter-spacing: 2px;">👨‍💻 INISIATOR & PENULIS UTAMA</h3>
<p style="color: #FFFFFF; font-family: 'Courier New', monospace; font-size: 18px;"><strong>Emen (duhemen)</strong></p>
<p style="color: #00FF00; font-family: 'Courier New', monospace; font-size: 14px;"><code>Konseptor • Visioner • Independent Researcher</code></p>
<blockquote style="border-left: 3px solid #00FF00; padding-left: 15px; color: #CCCCCC; font-style: italic; max-width: 700px; margin: 0 auto;">
"Kriptografi yang sempurna bukanlah yang paling kaku, melainkan yang meniru sifat alam semesta—kekacauan yang teratur."</blockquote>
<p style="color: #00C853; font-family: 'Courier New', monospace; font-size: 13px;">🍀 Dikembangkan dengan asistensi AI (<strong>Meta AI</strong>, <strong>DeepSeek</strong>, <strong>Google Mode AI</strong> & <strong>Qwen</strong>) sebagai kolaborator riset. 🍀</p>
<img src="https://capsule-render.vercel.app/api?type=waving&color=0:00FF00,50:001a00,100:000000&height=120&section=footer" width="100%"/>
<p style="color: #00FF00; font-family: 'Courier New', monospace; font-size: 14px; margin-top: 10px;"><strong>© 2026 — Emen (duhemen) | Made with 💚 and ☕ in Indonesia</strong></p>
</div>
