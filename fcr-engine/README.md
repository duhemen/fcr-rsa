# FCR-RSA Engine

**Engine kriptografi hibrida FCR-RSA dengan REST API, Auto-Mune (Active Defense), dan Forensic Tracing.**

> 🛡️ **Production-Ready** | 🧠 **Privacy-Preserving** | 🍯 **Deceptive Honeypot**

FCR-RSA Engine adalah implementasi referensi dari protokol FCR-RSA (Fibonacci-Cascade RSA) yang terintegrasi dengan `fcr-core-rust` v0.6.0. Engine ini menyediakan CLI untuk manajemen kunci, REST API dengan autentikasi Spatiotemporal HMAC, dan sistem pertahanan aktif (*Active Defense*) yang mampu mendeteksi anomali, menjebak penyerang dengan *Poisoned Pill*, dan melacak pelaku secara forensik melalui `trace_id`.

---

## 🌟 Fitur Utama

### 🔐 Kriptografi Inti
- **Hybrid KEM + AEAD**: Enkripsi kunci sesi via FCR-RSA (U2048, constant-time) + data via ChaCha20-Poly1305.
- **Side-Channel Resistant**: Blinding factor deterministik, operasi modular constant-time.
- **IND-CPA Secure**: Session seed acak per-opsi, nonce unik per-plaintext.

### 🌍 Spatiotemporal Authentication
- **HMAC-SHA256 Headers**: Setiap request wajib ditandatangani dengan koordinat GPS, timestamp, dan latency.
- **Privacy-Preserving**: Hanya hash grid lokasi (~11km) yang disimpan — **100% UU PDP compliant**, tidak ada koordinat mentah yang keluar dari memori.
- **Anti-Spoofing**: Signature diverifikasi constant-time; header palsu ditolak dengan `401 Unauthorized`.

### 🛡️ Auto-Mune (Sistem Imun Aktif)
- **Risk Scoring (0–100)**: Menganalisis latency, perpindahan zona lokasi, dan drift timestamp.
- **Threshold Adaptif**:
  - `< 40` → Request normal.
  - `40–69` → Suspicious (dicatat, tidak diblokir).
  - `≥ 70` → **Honeypot aktif** + Cooldown 5 menit.

### 🍯 Poisoned Pill Honeypot
- **Deceptive Response**: Penyerang mendapat HTTP `200 OK` dengan ciphertext yang terlihat valid.
- **Forensic Payload**: Saat didekripsi, penyerang menemukan JSON peringatan dengan `trace_id` unik.
- **Psychological Ops**: Penyerang tidak tahu mereka telah terdeteksi hingga mereka membuka "kotak hadiah" tersebut.

### 🔍 Forensic Tracing
- **Trace ID**: Setiap insiden tercatat di `anomaly_events` dengan `trace_id` yang sama persis dengan yang disuntikkan ke ciphertext penyerang.
- **Reverse Tracing**: Ciphertext honeypot yang bocor dapat dilacak kembali ke pelaku, kunci, dan waktu serangan.
- **Automated Incident Report**: Server mencetak laporan forensik lengkap (WHO, WHEN, WHAT) secara otomatis di terminal.

---

## 🏗️ Arsitektur

```
┌─────────────────────────────────────────────────────────────┐
│                        Client (Attacker)                    │
│  [GPS + Timestamp + Latency] → HMAC-SHA256 → HTTP Request   │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                  fcr-engine (Axum 0.7)                      │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Middleware: require_api_key                          │  │
│  │  • Bearer Token Verification                          │  │
│  │  • Spatiotemporal HMAC Validation                     │  │
│  │  • Auto-Mune Risk Scoring                             │  │
│  │  • Honeypot Injection (if score ≥ 70)                 │  │
│  └───────────────────────────────────────────────────────┘  │
│                            │                                │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Handlers: /keygen, /encrypt, /decrypt                │  │
│  │  • Poisoned Pill substitution (if honeypot)           │  │
│  │  • Hybrid KEM + AEAD (fcr-core-rust v0.6.0)           │  │
│  └───────────────────────────────────────────────────────┘  │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                  PostgreSQL (fcr_rsa)                       │
│  • keys              • api_keys (with lockout_until)        │
│  • anomaly_events    • api_key_states (location hash)       │
└─────────────────────────────────────────────────────────────┘
```

---

## 📦 Prasyarat

- **Rust** (1.75+) dengan `cargo`
- **PostgreSQL** 14+ (berjalan di port 5432)
- **Python 3.10+** (opsional, untuk script testing)

---

## ⚙️ Setup

### 1. Clone & Konfigurasi

```bash
cd fcr-engine
cp .env.example .env
# Edit .env:
# DATABASE_URL=postgres://postgres:PASSWORD@127.0.0.1:5432/fcr_rsa
```

### 2. Inisialisasi Database

Buat database `fcr_rsa` dan jalankan schema berikut:

```sql
-- Tabel kunci kriptografi
CREATE TABLE keys (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    seed TEXT NOT NULL,
    n INTEGER NOT NULL,
    q TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tabel API key dengan fitur lockout & cooldown
CREATE TABLE api_keys (
    id BIGSERIAL PRIMARY KEY,
    user_name TEXT NOT NULL,
    label TEXT,
    key_prefix TEXT NOT NULL,
    key_hash TEXT NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_locked BOOLEAN NOT NULL DEFAULT FALSE,
    lockout_until TIMESTAMPTZ,
    lock_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ
);

-- Tabel state lokasi (Privacy-Preserving)
CREATE TABLE api_key_states (
    api_key_id BIGINT PRIMARY KEY REFERENCES api_keys(id),
    last_loc_hash TEXT NOT NULL,
    last_timestamp TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tabel anomali & insiden forensik
CREATE TABLE anomaly_events (
    id BIGSERIAL PRIMARY KEY,
    api_key_id BIGINT NOT NULL REFERENCES api_keys(id),
    event_type TEXT NOT NULL,
    details TEXT,
    risk_score INTEGER,
    distance_km DOUBLE PRECISION,
    trace_id VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_anomaly_trace ON anomaly_events(trace_id);
CREATE INDEX idx_anomaly_api_key ON anomaly_events(api_key_id);
```

### 3. Build

```bash
cargo build --release
```

---

## 🖥️ Penggunaan CLI

### Generate Kunci Baru

```bash
cargo run --release -- keygen -N "kunci_utama" -s "super-secret-seed" -i 10 -q "1000000007"
```

### Enkripsi Teks

```bash
cargo run --release -- encrypt -m "Rahasia Negara" -k "kunci_utama" -t b
```

**Output**: Ciphertext format 4-bagian (Base64):
```
BLINDING_NONCE:ENCRYPTED_SEED:AEAD_NONCE:CIPHERTEXT
```

### Dekripsi Teks

```bash
cargo run --release -- decrypt -c "<CIPHERTEXT_4_BAGIAN>" -k "kunci_utama" -t b
```

### Manajemen API Key

```bash
# Buat API key baru (plaintext hanya muncul SEKALI)
cargo run --release -- apikey create -u emen -l "laptop-emen"

# Daftar semua API key
cargo run --release -- apikey list

# Nonaktifkan API key
cargo run --release -- apikey revoke -p "fcr_8b10"
```

---

## 🌐 Penggunaan REST API

### 1. Jalankan Server

```bash
cargo run --release -- serve
# Override port: FCR_PORT=9000 cargo run --release -- serve
```

Server berjalan di `http://0.0.0.0:8080`.

### 2. Health Check (Publik)

```bash
curl http://localhost:8080/health
```

### 3. Enkripsi via API (Dengan Spatiotemporal Headers)

Setiap request terproteksi **WAJIB** menyertakan 5 header spatiotemporal yang ditandatangani dengan HMAC-SHA256 menggunakan API key sebagai secret.

**Contoh dengan Python** (lihat `test_api.py`):

```python
import hmac, hashlib, requests, base64
from datetime import datetime, timezone

API_KEY = "fcr_xxxxxxxxxxxxxxxx"
lat, lon = "-6.2088", "106.8456"
timestamp = datetime.now(timezone.utc).isoformat()
latency = "50.0"

# Buat HMAC signature
payload = f"{lat}:{lon}:{timestamp}:{latency}"
signature = hmac.new(API_KEY.encode(), payload.encode(), hashlib.sha256).hexdigest()

headers = {
    "Authorization": f"Bearer {API_KEY}",
    "X-FCR-GPS-Lat": lat,
    "X-FCR-GPS-Lon": lon,
    "X-FCR-Timestamp": timestamp,
    "X-FCR-Latency": latency,
    "X-FCR-Signature": signature,
    "Content-Type": "application/json"
}

resp = requests.post("http://localhost:8080/encrypt", headers=headers, json={
    "key_name": "kunci_utama",
    "text_b64": base64.b64encode(b"Rahasia Negara").decode(),
    "tier": "b"
})
print(resp.json())
```

### 4. Dekripsi via API

```bash
curl -X POST http://localhost:8080/decrypt \
  -H "Authorization: Bearer <API_KEY>" \
  -H "X-FCR-GPS-Lat: -6.2088" \
  -H "X-FCR-GPS-Lon: 106.8456" \
  -H "X-FCR-Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  -H "X-FCR-Latency: 50.0" \
  -H "X-FCR-Signature: <HMAC_HEX>" \
  -H "Content-Type: application/json" \
  -d '{"key_name":"kunci_utama","ciphertext_b64":"<BASE64_CT>","tier":"b"}'
```

---

## 🛡️ Auto-Mune: Active Defense

### Mekanisme Deteksi

Auto-Mune menganalisis setiap request berdasarkan:

| Faktor | Threshold | Skor |
|--------|-----------|------|
| Latency ekstrem | > 1500 ms | +80 |
| Latency tinggi | > 800 ms | +20 |
| Timestamp drift | < -300s | +30 |
| **Impossible zone jump** | **< 60s antar grid** | **+75** |
| Rapid zone transition | < 1800s antar grid | +30 |
| First request | — | +5 |

### Respon Berbasis Skor

- **`< 40`** → Request diproses normal.
- **`40 – 69`** → Suspicious, dicatat di `anomaly_events`, request tetap diproses.
- **`≥ 70`** → **HONEYPOT AKTIF**:
  1. API key dikunci sementara (cooldown 5 menit).
  2. Plaintext diganti dengan **Poisoned Pill** (JSON forensik).
  3. Ciphertext dienkripsi seperti biasa (penyerang tidak sadar).
  4. `trace_id` dicatat di database.

---

## 🍯 Poisoned Pill & Forensic Tracing

### Apa itu Poisoned Pill?

Ketika Auto-Mune mendeteksi serangan (skor ≥ 70), server **tidak memblokir** request. Sebaliknya, server:

1. Mengembalikan HTTP `200 OK` (penyerang mengira berhasil).
2. Mengganti plaintext dengan **payload forensik**:

```json
{
  "status": "HONEYPOT_TRIGGERED",
  "message": "⚠️ AKTIVITAS MENCURIGAKAN TERDETEKSI ⚠️",
  "trace_id": "TRC-1795068093299",
  "warning": "Ciphertext ini adalah umpan (Canary Token). Identitas sesi Anda telah dicatat dalam sistem forensik FCR-RSA.",
  "original_attempt_reason": "impossible zone jump in 3s",
  "server_timestamp": "2026-11-19T06:01:33.308Z",
  "beacon_note": "Jika sistem Anda secara otomatis memproses teks ini, pertimbangkan untuk melaporkan trace_id ke admin keamanan."
}
```

3. Payload ini **dienkripsi dengan kunci yang sama**, sehingga ciphertext terlihat 100% valid.

### Cara Melacak Pelaku dari Ciphertext yang Bocor

Jika ciphertext honeypot ditemukan di forum gelap, dark web, atau dilaporkan oleh pihak ketiga:

**Langkah 1**: Dekripsi ciphertext tersebut (dengan kunci yang sesuai) untuk mendapatkan `trace_id`.

**Langkah 2**: Jalankan query forensik di PostgreSQL:

```sql
SELECT 
    ae.trace_id,
    ae.created_at AS waktu_serangan,
    ak.user_name AS pelaku,
    ak.key_prefix AS kunci_yang_digunakan,
    ak.label AS tujuan_kunci,
    ae.details AS modus_operandi,
    ae.risk_score
FROM anomaly_events ae
JOIN api_keys ak ON ae.api_key_id = ak.id
WHERE ae.trace_id = 'TRC-1795068093299';
```

**Hasil**: Identitas pelaku terungkap dalam hitungan milidetik.

```
     trace_id      |    waktu_serangan     | pelaku | kunci_yang_digunakan |       modus_operandi       
-------------------+-----------------------+--------+----------------------+----------------------------
 TRC-1795068093299 | 2026-11-19 13:01:33   | emen   | fcr_8b10             | impossible zone jump in 3s
```

### Automated Incident Report

Setiap kali honeypot terpicu, server **secara otomatis** mencetak laporan forensik di terminal:

```
============================================================
🚨 [SECURITY INCIDENT REPORT] AUTO-GENERATED 🚨
============================================================
⏰ WHEN       : 2026-11-19T05:51:15.966968600+00:00
👤 WHO        : User 'emen', Key Prefix 'fcr_8b10'
🎯 WHAT       : impossible zone jump in 3s
⚠️  RISK SCORE : 75/100
🔗 TRACE ID   : TRC-1795067475955
🛡️  ACTION     : Cooldown applied (5 mins) + Honeypot served
============================================================
```

---

## 🔒 Catatan Keamanan

### Yang Dilindungi
- ✅ **Side-channel attacks**: Constant-time modular exponentiation dengan blinding.
- ✅ **Replay attacks**: Nonce unik + timestamp HMAC.
- ✅ **Location spoofing**: Grid hashing + impossible jump detection.
- ✅ **Key exfiltration**: Poisoned pill + forensic tracing.
- ✅ **Privacy (UU PDP)**: Tidak ada koordinat GPS mentah yang disimpan.

### Yang Perlu Diperhatikan
- ⚠️ **Session seed length**: Saat ini 32 byte (256-bit) untuk kompatibilitas HKDF.
- ⚠️ **Cooldown duration**: Default 5 menit; dapat disesuaikan di `auth.rs`.
- ⚠️ **Blinding nonce**: Dikirim plaintext dalam ciphertext (aman, hanya untuk masking side-channel).

---

## 📊 Metrik Performa

| Operasi | Latency (avg) | Throughput |
|---------|---------------|------------|
| Enkripsi (42 bytes) | ~35 ms | ~28 ops/detik |
| Dekripsi (42 bytes) | ~15 ms | ~66 ops/detik |
| Auto-Mune scoring | < 1 ms | — |
| HMAC verification | < 1 ms | — |

*Diukur pada Intel i7-12700K, PostgreSQL 14, release build.*

---

## 🛡️ Verified Security Claims (Brutal Test Suite)

Sistem ini telah melewati **36/36 assertion** dalam *Brutal Test Suite* otomatis (`test_brutal_suite.py`), memverifikasi bahwa setiap klaim desain FCR-RSA benar-benar berfungsi, bukan sekadar teori:

| Kategori Pengujian | Status | Cakupan Verifikasi |
| :--- | :---: | :--- |
| **A. Kriptografi Inti** | ✅ 8/8 PASS | Round-trip, IND-CPA, Blinding Determinism, Tamper Resistance, Wrong-Key Reject, Format Validation |
| **B. Spatiotemporal Auth** | ✅ 7/7 PASS | HMAC-SHA256 Validation, Missing Headers, Forged Signatures, Malformed Data Rejection |
| **C. Auto-Mune Scoring** | ✅ 7/7 PASS | Extreme Latency, Timestamp Drift, Impossible Zone Jump, Rapid Transition, Score Combination |
| **D. Honeypot & Forensics** | ✅ 7/7 PASS | Deceptive 200 OK, Forensic Payload Injection, `trace_id` DB Consistency, Cooldown Enforcement |
| **E. API Surface** | ✅ 7/7 PASS | Health Check, 404/400 Error Handling, Tier A/B Support, API Key Revocation |

> 🟢 **Total: 36 Assertions | PASS: 36 | FAIL: 0**<br>
> *Diverifikasi secara otomatis. Semua klaim desain FCR-RSA telah terbukti secara brutal.*

---

## 🧪 Cara Menjalankan Pengujian

Untuk memverifikasi sendiri integritas sistem ini, jalankan *Brutal Test Suite*:

```bash
# 1. Pastikan server berjalan di terminal lain:
cargo run --release -- serve

# 2. Set environment variable database:
$env:DATABASE_URL="postgresql://postgres:PASSWORD_ANDA@127.0.0.1:5432/fcr_rsa"

# 3. Jalankan pengujian brutal:
python test_brutal_suite.py
```

---

## 📚 Referensi Teknis

- **`fcr-core-rust` v0.6.0**: Library inti dengan U2048, Montgomery arithmetic, constant-time.
- **Sesi Log 18/11/2026**: `docs/sesi_log_18112026.md` — Milestone completion log.
- **Side-Channel Audit**: `docs/side_channel_audit.md` — Welch's t-test & cache simulation.
- **ML Noise Analysis**: `simulations/ml_noise_analysis.py` — Random Forest accuracy 51.67% ≈ random guess.

---

## 🤝 Kontribusi

1. Fork repository.
2. Buat branch fitur (`git checkout -b feature/amazing`).
3. Commit perubahan (`git commit -m 'feat: add amazing feature'`).
4. Push ke branch (`git push origin feature/amazing`).
5. Buka Pull Request.

---

## 📄 Lisensi

Dual-licensed under **MIT** or **Apache-2.0** at your option.

Lihat [LICENSE](../LICENSE) untuk detail.

---

## 🏆 Pengakuan

Dibangun dengan ❤️ oleh **Emen** (`duhemen`) dengan bantuan **Qwen** (AI Assistant).

> *"Sistem yang baik tidak hanya menolak penyerang — ia membuat penyerang menyesal pernah mencoba."*
<div align="center" style="background: linear-gradient(135deg, #003300 0%, #001a00 50%, #000000 100%); padding: 25px 20px; border: 2px solid #00FF00; border-radius: 15px; margin-top: 30px; box-shadow: 0 0 25px rgba(0, 255, 0, 0.3);">
<p style="color: #00FF00; font-family: 'Courier New', monospace;"><strong>© 2026 — Emen (duhemen) | Made with 💚 and ☕ in Indonesia</strong></p>
**Status**: ✅ Production-Ready | **Versi**: 0.1.0 | **Core**: fcr-core-rust v0.6.0 Terakhir diperbarui: 19 November 2026
</div>

