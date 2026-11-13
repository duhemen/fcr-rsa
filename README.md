<!-- ═══════════════════════════════════════════════════════════════ -->
<!--           REPOSITORY README — FCR-RSA MULTI-LAYER               -->
<!--                  LUCA PROTOCOL © 2026                           -->
<!-- ═══════════════════════════════════════════════════════════════ -->

<div align="center" style="background: linear-gradient(135deg, #000000 0%, #001a00 50%, #003300 100%); padding: 40px 20px; border: 2px solid #00FF00; border-radius: 15px; margin-bottom: 20px; box-shadow: 0 0 30px #00FF00, inset 0 0 30px rgba(0, 255, 0, 0.1);">

<img src="https://capsule-render.vercel.app/api?type=waving&color=0:000000,50:001a00,100:00FF00&height=200&section=header&text=FCR%20RSA&fontSize=70&fontColor=00FF00&animation=fadeIn&fontAlignY=38&desc=Research%20Prototype%20%E2%80%94%20Not%20Production%20Ready&descAlignY=58&descSize=24" width="100%"/>

<img src="https://readme-typing-svg.demolab.com?font=Fira+Code&weight=900&size=20&duration=2500&pause=800&color=00FF00&center=true&vCenter=true&multiline=true&width=900&height=180&lines=Wake+Up+Duhemen...;Follow+the+White+LUCA...;System+Online...;From+uncertainty+comes+life%2C+from+chaos+comes+consciousness..." alt="Typing SVG" />

<h2 style="color: #00FF00; font-family: 'Courier New', monospace; margin-top: 20px; text-shadow: 0 0 10px #00FF00, 0 0 20px #00FF00;">
  🔬 FCR-RSA — Research Prototype
</h2>

<p style="color: #00FF00; font-family: 'Courier New', monospace; font-size: 16px; letter-spacing: 1px;">
  <code>Post-quantum cryptographic protocol prototype. Open for critique.</code>
</p>

<br/>

![Status](https://img.shields.io/badge/STATUS-RESEARCH%20PROTOTYPE-F59E0B?style=for-the-badge&logo=statuspage&logoColor=black&labelColor=000000)
![Audit](https://img.shields.io/badge/INDEPENDENT%20AUDIT-NOT%20YET-EF4444?style=for-the-badge&logo=shield&logoColor=white&labelColor=000000)
![License](https://img.shields.io/badge/LICENSE-MIT%20OR%20Apache--2.0-blue?style=for-the-badge&logo=opensourceinitiative&logoColor=white&labelColor=000000)
![Rust](https://img.shields.io/badge/RUST-LIBRARY%20%2B%20TESTS-000000?style=for-the-badge&logo=rust&logoColor=white&labelColor=000000)
![Python](https://img.shields.io/badge/PYTHON-REFERENCE%20IMPL-3776AB?style=for-the-badge&logo=python&logoColor=white&labelColor=000000)

</div>

---

## ⚠️ Status Keamanan — Baca Sebelum Apa Pun

> **FCR-RSA adalah purwarupa riset. Belum melalui audit keamanan independen.**
>
> **JANGAN gunakan untuk melindungi data nyata, uang, atau identitas.**
>
> Baca **[LIMITATIONS.md](LIMITATIONS.md)** untuk daftar lengkap apa yang
> **tidak** dijamin oleh protokol ini. Baca **[SECURITY.md](SECURITY.md)**
> jika Anda menemukan celah.

| Aspek | Status |
| :--- | :--- |
| Implementasi Rust | ✅ Library berjalan, 3 integration test lulus |
| Implementasi Python | ✅ Paket berjalan, 3 unit test lulus |
| Konsistensi Rust ↔ Python | ✅ Ciphertext identik untuk parameter uji |
| Audit independen | ❌ **Belum** |
| Uji BKZ 2.0 | ❌ **Belum** |
| Uji quantum simulation | ❌ **Belum** |
| Constant-time verification | ❌ **Belum** |
| Test vector resmi | ⚠️ Struktur ada, isi menyusul |

---

## 📌 Daftar Isi

<details open>
<summary>Klik untuk melihat / menyembunyikan</summary>

- [📖 Dokumentasi Proyek](#-dokumentasi-proyek)
- [💡 1. Latar Belakang](#-1-latar-belakang)
- [🧪 2. Fondasi Matematika](#-2-fondasi-matematika)
- [🛡️ 3. Lapisan Auto-Imun](#️-3-lapisan-auto-imun)
- [📂 4. Struktur Repositori](#-4-struktur-repositori)
- [⚡ 5. Quickstart](#-5-quickstart)
- [🧪 6. Pengujian & Simulasi](#-6-pengujian--simulasi)
- [🤝 7. Panggilan Kolaborasi](#-7-panggilan-kolaborasi)

</details>

---

## 📖 Dokumentasi Proyek

- 📄 **[Proposal & KAK/TOR](docs/proposal.md)** — Latar belakang, analisis situasi, roadmap.
- 📊 **[Laporan Teknis](docs/laporan_teknis.md)** — Benchmark, hasil uji ofensif & defensif.
- 📜 **[Whitepaper (English)](docs/whitepaper.md)** — Naskah akademik lengkap.
- 🏗️ **[Arsitektur](docs/architecture.md)** — Tier A/B, Bridge Protocol, auto-imun.
- 🎯 **[Threat Model](docs/threat-model.md)** — Apa yang dilawan, apa yang tidak.
- ⚠️ **[Limitasi](LIMITATIONS.md)** — **Wajib dibaca** sebelum menggunakan.
- 🔒 **[Security Policy](SECURITY.md)** — Cara melaporkan celah.

---

## 💡 1. Latar Belakang

Sistem enkripsi global saat ini (RSA, ECC) bergantung pada kesulitan
faktorisasi bilangan prima besar. Fondasi ini diproyeksikan runtuh
ketika komputer kuantum berskala penuh menjalankan **Algoritma Shor**.

**FCR-RSA (Fibonacci Complex Noise-RSA)** adalah eksperimen protokol
kriptografi pasca-kuantum yang mengeksplorasi pendekatan berbeda:

1. **Domain matematika alternatif:** Deret Fibonacci non-linear dengan
   noise LWE deterministik, dipetakan ke Gaussian integer.
2. **Arsitektur dua tier:** Tier A (server, keamanan maksimal) dan
   Tier B (perangkat, efisiensi baterai/CPU).
3. **Lapisan auto-imun:** Token spatiotemporal, deteksi anomali,
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

### C. Enkripsi Tier B (Ringan)

$$C = (M \times E_{vec}) + R_{blind} \pmod Q$$

`R_blind` adalah **blinding factor** untuk IND-CPA. Di contoh, `R_blind`
sengaja konstan agar ciphertext reproduktif untuk pengujian lintas-bahasa.
**Di produksi, `R_blind` harus di-generate secara kriptografis acak.**

### D. Enkripsi Tier A (Berat)

$$C = (M \times E_{vec}) + N_{comp} + R_{blind} \pmod Q$$

Menambahkan `N_comp` sebagai noise tambahan. Saat ini hanya
diimplementasikan di Python (`tier_a.py`).

---

## 🛡️ 3. Lapisan Auto-Imun

FCR-RSA tidak berasumsi lapisan manusia aman. Protokol mengikat
**Spatiotemporal Token** (GPS + Timestamp + Latensi) ke dalam fase
transformasi:

- **Lockdown Endemi:** Jika kredensial bocor dan dipakai di luar
  parameter koordinat yang sah, sistem mengisolasi cabang tersebut
  tanpa melumpuhkan jaringan pusat.
- **Deteksi MITM:** Latensi tidak wajar (≥300ms) memicu alarm.
- **Honeytoken:** Konsep ada, implementasi belum.

> ⚠️ Logika ini **belum diuji terhadap data dunia nyata**. False positive
> dan false negative belum diukur. Lihat [LIMITATIONS.md](LIMITATIONS.md).

---

## 📂 4. Struktur Repositori

```text
fcr-rsa/
├── docs/                  # Dokumentasi teknis & whitepaper
│   ├── whitepaper.md
│   ├── architecture.md
│   ├── threat-model.md
│   ├── test-vectors.md
│   ├── laporan_teknis.md
│   └── proposal.md
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
├── fcr-python/            # Implementasi Python (paket)
│   ├── fcr-pyo3/
│   ├── fcr_rsa/
│   │   ├── core_math.py
│   │   ├── tier_a.py
│   │   ├── tier_b.py
│   │   └── auto_immune.py
│   ├── examples/basic.py
│   └── tests/test_basic.py
│
├── simulations/           # Simulasi serangan & benchmark
│   ├── red_team_lll.py
│   ├── red_team_noise_attack.py
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

## ⚡ 5. Quickstart

### Rust

```bash
cd fcr-core-rust
cargo test
cargo run --example basic
```

Yang diharapkan:
```
running 3 tests
test invalid_n_rejected ... ok
test round_trip_multiple ... ok
test round_trip_small ... ok

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

### Rust via Python (PyO3 - Super Cepat)

Jika Anda ingin menggunakan kecepatan *native* Rust langsung dari Python:

```bash
cd fcr-pyo3
pip install maturin
maturin develop
python py.py

```

> **Catatan:** Ciphertext `722957369` identik antara Rust dan Python
> untuk parameter uji yang sama. Ini bukti konsistensi lintas-bahasa.
> **Yang di harapkan** Rust (via Python) Ciphertext: 758780528,  Decrypted: 42

---

## 🧪 6. Pengujian & Simulasi

### Uji Defensif (Internal)

| File | Deskripsi | Status |
| :--- | :--- | :--- |
| `fcr-core-rust/tests/integration.rs` | Round-trip Rust | ✅ 3 passed |
| `fcr-python/tests/test_basic.py` | Round-trip Python | ✅ 3 passed |
| `simulations/run_test.py` | Enkripsi/dekripsi dasar | ✅ |
| `simulations/bridge_protocol.py` | Komunikasi Tier A/B | ✅ |

### Uji Ofensif (Red Team — Internal, Bukan Audit Independen)

| File | Deskripsi | Status |
| :--- | :--- | :--- |
| `simulations/red_team_lll.py` | Serangan LLL dasar | ✅ Gagal menembus |
| `simulations/red_team_noise_attack.py` | Serangan noise_seed | ✅ Gagal menembus |
| `simulations/ordal_attack_simulation.py` | Simulasi Pegasus/LST | ✅ Terdeteksi auto-imun |

> ⚠️ **Penting:** Red team ini dilakukan oleh tim yang sama yang merancang
> protokol. Ini **bukan** audit independen. Auditor eksternal masih
> dibutuhkan sebelum klaim keamanan bisa dianggap valid.

### Uji yang **Belum** Dilakukan

- BKZ 2.0 dengan parameter agresif
- Algebraic cryptanalysis
- Quantum simulation (Shor, Grover)
- Side-channel timing analysis
- Fuzzing parser input
- Formal verification

---

## 🤝 7. Panggilan Kolaborasi

FCR-RSA adalah riset terbuka. Kami **mengundang** kritik, uji coba, dan
kontribusi. Yang paling dibutuhkan saat ini:

1. **Uji klaim keamanan** — coba tembus FCR-RSA. Jika berhasil, laporkan
   via [SECURITY.md](SECURITY.md). Jika gagal, dokumentasikan usahamu.
2. **Audit implementasi** — cari bug, side-channel, kesalahan logika.
3. **Test vector** — tambah kasus uji, verifikasi yang sudah ada.
4. **Dokumentasi** — perbaiki yang kurang jelas.

Baca [CONTRIBUTING.md](CONTRIBUTING.md) untuk panduan.

### Roadmap (Jujur)

| Fase | Target | Status |
| :--- | :--- | :--- |
| 1 | Fondasi matematika + implementasi Rust/Python | ✅ Selesai |
| 2 | Test vector resmi + Tier A di Rust | 🚧 Sedang berjalan |
| 3 | Audit independen + BKZ 2.0 | ⏳ Belum |
| 4 | Publikasi akademik + library siap pakai | ⏳ Belum |

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
  🍀 Dikembangkan dengan asistensi AI (<strong>DeepSeek</strong> & <strong>Google Mode AI</strong>) sebagai kolaborator riset. 🍀
</p>

<img src="https://capsule-render.vercel.app/api?type=waving&color=0:00FF00,50:001a00,100:000000&height=120&section=footer" width="100%"/>

<p style="color: #00FF00; font-family: 'Courier New', monospace; font-size: 14px; margin-top: 10px;">
  <strong>© 2026 — Emen (duhemen) | Made with 💚 and ☕ in Indonesia</strong>
</p>

</div>