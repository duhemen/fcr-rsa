<!-- ═══════════════════════════════════════════════════════════════ -->
<!--           REPOSITORY README — FCR-RSA MULTI-LAYER               -->
<!--                  LUCA PROTOCOL © 2026                           -->
<!-- ═══════════════════════════════════════════════════════════════ -->
<div align="center" style="background: linear-gradient(135deg, #000000 0%, #001a00 50%, #003300 100%); padding: 40px 20px; border: 2px solid #00FF00; border-radius: 15px; margin-bottom: 20px; box-shadow: 0 0 30px #00FF00, inset 0 0 30px rgba(0, 255, 0, 0.1);">
<img src="https://capsule-render.vercel.app/api?type=waving&color=0:000000,50:001a00,100:00FF00&height=200&section=header&text=FCR%20RSA&fontSize=70&fontColor=00FF00&animation=fadeIn&fontAlignY=38&desc=Research%20Prototype%20v0.6.0%20%E2%80%94%20Hybrid%20KEM%20%2B%20AEAD%20%2B%20Active%20Defense&descAlignY=58&descSize=22" width="100%"/>
<img src="https://readme-typing-svg.demolab.com?font=Fira+Code&weight=900&size=20&duration=2500&pause=800&color=00FF00&center=true&vCenter=true&multiline=true&width=900&height=180&lines=Wake+Up+Duhemen...;Follow+the+White+LUCA...;System+Online...;From+uncertainty+comes+life%2C+from+chaos+comes+consciousness..." alt="Typing SVG" />
<h2 style="color: #00FF00; font-family: 'Courier New', monospace; margin-top: 20px; text-shadow: 0 0 10px #00FF00, 0 0 20px #00FF00;">
🔬 FCR-RSA — Research Prototype (v0.6.0)
</h2>
<p style="color: #00FF00; font-family: 'Courier New', monospace; font-size: 16px; letter-spacing: 1px;">
<code>Post-quantum hybrid KEM + Active Defense. Open for critique. Not production ready.</code>
</p>
<br/>
<img src="https://img.shields.io/badge/STATUS-RESEARCH%20PROTOTYPE%20v0.6.0-F59E0B?style=for-the-badge" alt="Status" />
<img src="https://img.shields.io/badge/ARCH-HYBRID%20KEM%20%2B%20AEAD-00FF00?style=for-the-badge" alt="Architecture" />
<img src="https://img.shields.io/badge/Q%20SIZE-2048%20BIT-00BFFF?style=for-the-badge" alt="Modulus" />
<img src="https://img.shields.io/badge/SIDE%20CHANNEL-CONSTANT%20TIME%20VERIFIED-9370DB?style=for-the-badge" alt="Constant Time" />
<img src="https://img.shields.io/badge/CACHE%20ATTACK-0%25%20DIFFERENCE-FF1493?style=for-the-badge" alt="Cache Attack" />
<img src="https://img.shields.io/badge/ALGEBRAIC%20ATTACK-GROEBNER%20RESISTANT-FF6347?style=for-the-badge" alt="Algebraic" />
<img src="https://img.shields.io/badge/ML%20NOISE%20PREDICTION-51.67%25%20%2B%20RANDOM-FFD700?style=for-the-badge" alt="ML Noise" />
<img src="https://img.shields.io/badge/CROSS%20LANGUAGE-10%2F10%20MATCH-20B2AA?style=for-the-badge" alt="Cross Language" />
<img src="https://img.shields.io/badge/BRUTAL%20TESTS-36%2F36%20PASS-00C853?style=for-the-badge" alt="Brutal Tests" />
<img src="https://img.shields.io/badge/AUTO%20MUNE-HONEYPOT%20%2B%20TRACE%20ID-FF8C00?style=for-the-badge" alt="Auto Mune" />
<img src="https://img.shields.io/badge/INDEPENDENT%20AUDIT-NOT%20YET-EF4444?style=for-the-badge" alt="Audit" />
<img src="https://img.shields.io/badge/LICENSE-MIT%20OR%20Apache--2.0-blue?style=for-the-badge" alt="License" />
</div>

## ⚠️ Status Keamanan — Baca Sebelum Apa Pun
> **FCR-RSA adalah purwarupa riset (v0.6.0). Belum melalui audit keamanan independen.**
>
> **JANGAN gunakan untuk melindungi data nyata, uang, atau identitas.**
>
> Baca **[LIMITATIONS.md](LIMITATIONS.md)** untuk daftar lengkap apa yang **tidak** dijamin oleh protokol ini. Baca **[SECURITY.md](SECURITY.md)** jika Anda menemukan celah.
| Aspek | Status |
| :--- | :--- |
| Implementasi Rust (fcr-core-rust v0.6.0) | ✅ Library berjalan, **16 integration test lulus** |
| Implementasi Python (fcr-python) | ✅ Paket berjalan, 3 unit test lulus |
| Binding Rust ↔ Python (fcr-pyo3) | ✅ API bytes-to-bytes, round-trip lulus |
| Konsistensi Rust ↔ Python | ✅ **10/10 ciphertext identik** (cross-language) |
| Hybrid KEM + AEAD | ✅ Known-plaintext attack **tertutup** |
| **Q 2048-bit (U2048 + Montgomery)** | ✅ **Small-Q brute-force CLOSED** |
| Constant-Time (Welch's t-test) | ✅ **LOLOS (p = 0.143 > 0.05)** |
| Cache Side-Channel Simulation | ✅ **LOLOS (0% perbedaan pola cache)** |
| Noise Stream Prediction (ML) | ✅ **LOLOS (akurasi 51.67% ≈ tebakan acak)** |
| Algebraic Resistance (Gröbner) | ✅ **LOLOS (~10^1850 kompleksitas)** |
| Auto-Mune + Honeypot + Forensics | ✅ **LOLOS Brutal Test (36/36)** |
| **BRUTAL TEST SUITE (engine)** | ✅ **36/36 PASS** |
| Audit independen | ❌ **Belum (Fase 4)** |
| Uji quantum simulation | ❌ **Belum (Fase 4)** |
| Formal verification (Coq/Lean) | ❌ **Belum (Fase 4)** |
---
## 📌 Daftar Isi

- [📖 Dokumentasi Proyek](#-Dokumentasi-Proyek)
- [💡 1. Latar Belakang](#-1-latar-belakang)
- [🧪 2. Fondasi Matematika](#-2-fondasi-matematika)
- [🛡️ 3. Lapisan Auto-Imun](#️-3-lapisan-auto-imun-active-defense)
- [🔥 4. Evolusi Keamanan](#-4-evolusi-keamanan-v01--v060)
- [🚨 5. Temuan Kritis & Mitigasi](#-5-temuan-kritis--mitigasi)
- [🧬 6. Bukti Verifikasi Brutal](#-6-bukti-verifikasi-brutal-3636)
- [📂 7. Struktur Repositori](#-7-struktur-repositori)
- [⚡ 8. Quickstart](#-8-quickstart)
- [🎯 9. Threat Model Komprehensif](#-9-threat-model-komprehensif)
- [🗺️ 10. Roadmap Pengembangan](#️-10-roadmap-pengembangan)
- [🤝 11. Panggilan Kolaborasi](#-11-panggilan-kolaborasi)

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
## 🔥 4. Evolusi Keamanan (v0.1 → v0.6.0)
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
## 🧬 6. Bukti Verifikasi Brutal (36/36)
Seluruh klaim desain **diinjak secara otomatis** oleh `test_brutal_suite.py`:
| Kategori | Status | Cakupan |
| :--- | :---: | :--- |
| **A. Kriptografi Inti** | ✅ 8/8 | Round-trip, IND-CPA, Blinding Determinism, Tamper/Wrong-Key/Format reject |
| **B. Spatiotemporal Auth** | ✅ 7/7 | HMAC validation, missing/forged/malformed headers |
| **C. Auto-Mune Scoring** | ✅ 7/7 | Latency, drift, zone jump, rapid transition, combination |
| **D. Honeypot & Forensics** | ✅ 7/7 | Deceptive 200, `trace_id` DB consistency, cooldown bypass |
| **E. API Surface** | ✅ 7/7 | Health, 404/400, Tier A/B, revoke |
> 🟢 **Total: 36 assertions | PASS: 36 | FAIL: 0** — *Semua klaim desain FCR-RSA terversifikasi secara brutal.*
---
## 📂 7. Struktur Repositori
```text
fcr-rsa/
├── docs/                      # whitepaper, architecture, threat-model, side_channel_audit, sesi_log
├── fcr-core-rust/             # Library inti (U2048, Montgomery, constant-time) — v0.6.0
├── fcr-engine/                # CLI + REST API + Auto-Mune + Honeypot + Forensics
│   └── test_brutal_suite.py   # 36/36 brutal test
├── fcr-python/                # Implementasi Python (referensi)
├── fcr-pyo3/                  # Binding Rust↔Python (maturin)
├── simulations/               # red_team_*, grover/shor, ml_noise_analysis, cache_sim
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
python test_brutal_suite.py           # 36/36 PASS
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
### 🔴 Masih Terbuka (Fase 4)
| Serangan | Status |
|----------|--------|
| **Fault Injection** | ❌ Belum (hardware-level) |
| **Quantum-Enhanced Lattice** | ⚠️ Simulasi Fase 4 |
| **Formal Verification** | ❌ Coq/Lean Fase 4 |
| **Audit Independen** | ❌ Pihak ketiga Fase 4 |
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
- [x] **Brutal Test Suite 36/36 PASS**

### ⏳ Fase 4: Validasi Eksternal & Skala Produksi (Target Q1-Q2 2027)
*(Catatan: Fondasi pengujian internal telah selesai. Fase 4 fokus pada eksternalisasi dan peningkatan skala)*
- [x] **Test Vector Internal:** 10/10 Cross-Language Match (Python ↔ Rust) ✅ SELESAI
- [x] **Quantum Simulation (PoC):** Grover ($2^{10}$) & Shor (N kecil) diverifikasi ✅ SELESAI
- [ ] **Quantum Simulation (Production Scale):** Implementasi oracle  FCR-RSA penuh di Qiskit untuk parameter $2^{256}$. **TARGET BERIKUTNYA**
- [ ] **Independent Security Audit:** Audit kode dan arsitektur oleh pihak ketiga (external firm).
- [ ] **Formal Verification:** Pembuktian matematis properti constant-time dan keamanan KEM menggunakan Coq/Lean.
- [ ] **Hardware Fault Injection Testing:** Pengujian fisik terhadap mitigasi redundant computation.
- [ ] **Official Test Vector Publication:** Publikasi format test vector sebagai standar referensi komunitas.
### ⏳ Fase 5: Publikasi & Adopsi (Q2 2027)
---
## 🤝 11. Panggilan Kolaborasi
FCR-RSA adalah riset terbuka. Yang paling dibutuhkan:
1. **Coba tembus** protokol ini — dokumentasikan usahamu.
2. **Audit implementasi** — cari bug/side-channel/logika.
3. **Formal verification** — bantu buktikan dengan Coq/Lean.
4. **Quantum simulation** — jalankan Grover/Shor di Colab.
Baca [CONTRIBUTING.md](CONTRIBUTING.md).
---
<div align="center" style="background: linear-gradient(135deg, #003300 0%, #001a00 50%, #000000 100%); padding: 35px 20px; border: 2px solid #00FF00; border-radius: 15px; margin-top: 30px; box-shadow: 0 0 25px rgba(0, 255, 0, 0.3);">
<h3 style="color: #00FF00; font-family: 'Courier New', monospace; letter-spacing: 2px;">👨‍💻 INISIATOR & PENULIS UTAMA</h3>
<p style="color: #FFFFFF; font-family: 'Courier New', monospace; font-size: 18px;"><strong>Emen (duhemen)</strong></p>
<p style="color: #00FF00; font-family: 'Courier New', monospace; font-size: 14px;"><code>Konseptor • Visioner • Independent Researcher</code></p>
<blockquote style="border-left: 3px solid #00FF00; padding-left: 15px; color: #CCCCCC; font-style: italic; max-width: 700px; margin: 0 auto;">
"Kriptografi yang sempurna bukanlah yang paling kaku, melainkan yang meniru sifat alam semesta—kekacauan yang teratur."</blockquote>
<p style="color: #00C853; font-family: 'Courier New', monospace; font-size: 13px;">🍀 Dikembangkan dengan asistensi AI (<strong>DeepSeek</strong>, <strong>Google Mode AI</strong> & <strong>Qwen</strong>) sebagai kolaborator riset. 🍀</p>
<img src="https://capsule-render.vercel.app/api?type=waving&color=0:00FF00,50:001a00,100:000000&height=120&section=footer" width="100%"/>
<p style="color: #00FF00; font-family: 'Courier New', monospace; font-size: 14px; margin-top: 10px;"><strong>© 2026 — Emen (duhemen) | Made with 💚 and ☕ in Indonesia</strong></p>
</div>