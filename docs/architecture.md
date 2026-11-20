# Arsitektur FCR-RSA (v0.6.0)

Dokumen ini menjelaskan arsitektur teknis FCR-RSA Multi-Layer setelah fase hardening (Fase 3) dan integrasi engine (Fase 3.5).

**Versi:** 0.6.0 (U2048 + Hybrid KEM + Active Defense)  
**Tanggal:** 20 November 2026  
**Status:** ✅ Research Prototype — Validated via Brutal Test Suite (36/36 PASS)

---

## 1. Fondasi Matematika

### A. Deret Fibonacci Non-Linear

FCR-RSA menggunakan perkalian modular non-linear dengan injeksi noise LWE deterministik:

$$F_n = (F_{n-1} \times F_{n-2}) + e_n \pmod Q$$

- $e_n$: noise deterministik dari `noise_seed` (SHA-256)
- $Q$: modulus **2048-bit** diimplementasikan sebagai `crypto-bigint::U2048` dengan Montgomery arithmetic (constant-time)

### B. Pemetaan Gaussian

Setiap elemen dipetakan ke ruang kompleks $\mathbb{Z}[i]$:

$$Z_n = \text{Re}(F_n) + \text{Im}(F_{n+1})i$$

### C. Hybrid KEM + AEAD (v0.5.1 → v0.6.0)

> ⚠️ **Catatan Historis:** Versi v0.4.1 mengenkripsi plaintext langsung dengan formula linear `C = (M × E_vec) + R_blind mod Q`. Celah ini **tertutup total** di v0.5.1 dengan arsitektur Hybrid KEM.

**Alur Enkripsi Modern:**
1. Generate session seed acak (32 byte / 256-bit)
2. Enkripsi seed dengan FCR-RSA U2048 (constant-time + deterministic blinding) → `Encrypted_Seed`
3. Turunkan kunci sesi 256-bit dari seed via **HKDF** (RFC 5869)
4. Enkripsi data dengan **ChaCha20-Poly1305** (AEAD)

**Format Ciphertext (4 bagian, semua Base64):**
```
BLINDING_NONCE : ENCRYPTED_SEED : AEAD_NONCE : CIPHERTEXT
```

---

## 2. Arsitektur Dua Tier

### Tier A — Enterprise / Banking
- **Parameter:** Indeks Fibonacci besar ($N > 500$)
- **Implementasi:** `crypto-bigint::U2048` + Montgomery `DynResidue`
- **Peran:** Deep sequence reconstruction, dekripsi, audit anomali global
- **Target:** Server bank, Cloud HSM, high-throughput parallel processing

### Tier B — End-User / IoT
- **Parameter:** Matriks Fibonacci ringan (2×2 atau 3×3)
- **Implementasi:** Binding native via **PyO3 + maturin** (bytes-to-bytes API)
- **Peran:** Enkripsi cepat + generasi token spatiotemporal lokal
- **Target:** Mobile devices, smart cards, resource-constrained endpoints

### Bridge Protocol
Sinkronisasi session key antara Tier B dan Tier A via Fibonacci Heartbeat, memastikan HP pengguna dapat berkomunikasi aman dengan server bank.

---

## 3. Sistem Auto-Imun (Zero-Trust Hardened — v0.6.0)

### A. Token Spatiotemporal & Integritas Kriptografis (HMAC-SHA256)

Setiap transaksi API wajib menyertakan telemetri fisik yang dilindungi integritasnya:

| Header | Fungsi | Proteksi |
|--------|--------|----------|
| `X-FCR-GPS-Lat` / `Lon` | Koordinat endpoint | Dikuantisasi ke grid ~11 km, di-hash Keyed SHA-256 (UU PDP compliant) |
| `X-FCR-Timestamp` | Waktu request | Deteksi replay & clock skew (toleransi mundur 5 menit) |
| `X-FCR-Latency` | Latensi jaringan | Deteksi anomali proksi / MITM delay |
| `X-FCR-Signature` | HMAC-SHA256 | Diverifikasi constant-time; header palsu → `401` |

### B. Adaptive Threat Response & Deceptive Honeypot

Sistem menerapkan respons adaptif berbasis **Risk Score (0–100)**:

| Skor | Respon |
|------|--------|
| `< 40` | Request diproses normal |
| `40 – 69` | Suspicious — dicatat di `anomaly_events`, request tetap diproses |
| `≥ 70` | **HONEYPOT AKTIF** (lihat sub-section C) |

**Faktor Penambah Skor:**
- Extreme latency (>1500 ms): +80
- High latency (>800 ms): +20
- Timestamp drift (< −300 s): +30
- **Impossible zone jump** (< 60 s antar grid): **+75**
- Rapid zone transition (< 1800 s): +30
- First request: +5

### C. Poisoned Pill & Forensic Tracing (v0.6.0)

Ketika skor ≥ 70, server **TIDAK memblokir** request. Sebaliknya:

1. **Deceptive Response:** Server mengembalikan `HTTP 200 OK` dengan ciphertext yang terlihat 100% valid.
2. **Poisoned Pill:** Plaintext di dalam ciphertext diganti dengan **payload forensik JSON**:
   ```json
   {
     "status": "HONEYPOT_TRIGGERED",
     "message": "⚠️ AKTIVITAS MENCURIGAKAN TERDETEKSI ⚠️",
     "trace_id": "TRC-1795068093299",
     "warning": "Ciphertext ini adalah umpan (Canary Token). Identitas sesi Anda telah dicatat.",
     "original_attempt_reason": "impossible zone jump in 3s",
     "server_timestamp": "2026-11-19T06:01:33.308Z"
   }
   ```
3. **Self-Healing Cooldown:** API key dibekukan sementara 5 menit (`apply_cooldown`), pulih otomatis tanpa intervensi manual.
4. **Forensic Tracing:** `trace_id` tercatat di tabel `anomaly_events`. Ciphertext honeypot yang bocor dapat ditelusuri balik ke pelaku via query:
   ```sql
   SELECT ak.user_name, ak.key_prefix, ae.details
   FROM anomaly_events ae
   JOIN api_keys ak ON ae.api_key_id = ak.id
   WHERE ae.trace_id = 'TRC-1795068093299';
   ```

---

## 4. Alur Data (REST API & Middleware)

```
[Klien / SDK]
   |  (Sign payload telemetri via HMAC-SHA256)
   v
[Middleware: require_api_key]
   |-- 1. Cek API Key & Status Cooldown DB -----> [401 jika terkunci]
   |-- 2. Validasi Header & HMAC Constant-Time -> [400 / 401 jika spoofed]
   |-- 3. Quantize GPS -> Keyed SHA-256 Hash
   |-- 4. Evaluasi Risk Score (0-100)
   |       └── Skor >= 70: Log Anomali + Set Cooldown 5m
   |           + Inject HoneypotContext ke request extensions
   v
[Handler: /keygen | /encrypt | /decrypt]
   |  (Cek HoneypotContext → ganti plaintext dengan Poisoned Pill jika ada)
   |  (Operasi Hybrid KEM + AEAD via fcr-core-rust v0.6.0)
   v
[Ciphertext (valid secara kriptografis, berisi payload asli ATAU forensik)]
```

---

## 5. Batasan Arsitektur

Lihat [LIMITATIONS.md](../LIMITATIONS.md). Arsitektur ini **belum melalui audit independen** dan tidak boleh dipakai untuk produksi.

### Catatan: Blinding Factor & Deterministic Override

- **Produksi:** Blinding nonce di-generate acak via `OsRng` untuk setiap sesi (menjamin IND-CPA).
- **Test Vector:** Untuk keperluan verifikasi cross-language, parameter `blinding_override` dapat diisi string konstan (misal `"deterministic-vector-001"`). Dalam mode ini, `session_seed` dan `aead_nonce` juga diturunkan secara deterministik via SHA-256, sehingga ciphertext **reproduktif** dan bisa dibandingkan lintas-bahasa.
- **Peringatan:** Jangan gunakan `blinding_override` di lingkungan produksi — akan menghilangkan properti IND-CPA.

---

## 6. Struktur Komponen (fcr-engine)

```text
fcr-engine/
├── src/
│   ├── main.rs              # CLI entry point + serve command
│   ├── cli.rs               # Clap definitions
│   ├── crypto.rs            # Hybrid KEM + AEAD wrappers (v0.6.0)
│   ├── db.rs                # PostgreSQL helpers (keys, api_keys, anomaly_events)
│   └── api/
│       ├── mod.rs           # Router + AppState
│       ├── auth.rs          # Middleware: Bearer + HMAC + Auto-Mune + HoneypotContext injection
│       ├── auto_immune.rs   # Risk scoring (Privacy-Preserving Location Hash)
│       └── handlers.rs      # /keygen, /encrypt (Poisoned Pill logic), /decrypt
├── test_brutal_suite.py     # 36/36 assertions
└── Cargo.toml               # Link ke fcr-core-rust v0.6.0
```

---

**Terakhir diperbarui:** 20 November 2026  
**Dokumen terkait:** [whitepaper.md](whitepaper.md) · [threat-model.md](threat-model.md) · [side_channel_audit.md](side_channel_audit.md)