# FCR-RSA Engine (CLI + REST API)

Engine untuk protokol FCR-RSA. Terhubung ke PostgreSQL untuk manajemen kunci, mendukung enkripsi teks nyata, dan dilengkapi dengan **REST API Server** serta **Auto-Mune (Sistem Imun)** untuk keamanan aktif.

## Fitur Utama
- **PostgreSQL Key Management**: Simpan dan kelola kunci di database.
- **Text I/O**: Enkripsi teks (Base-256 per byte).
- **IND-CPA (Auto-Embedding)**: *Blinding factor* acak disematkan otomatis ke *ciphertext*.
- **Tier A & B**: Mendukung mode enterprise (A) dan mobile (B).
- **REST API Server**: Endpoint `/keygen`, `/encrypt`, `/decrypt` dengan Autentikasi API Key.
- **Auto-Mune & Honeypot**: Deteksi anomali spatiotemporal, *kill switch* otomatis, dan respons jebakan (honeypot).

## Prasyarat
- Rust (Cargo)
- PostgreSQL (berjalan di port 5432)
- Database bernama `fcr_rsa` dengan tabel:
  ```sql
  CREATE TABLE keys (
      id SERIAL PRIMARY KEY,
      name VARCHAR(255) UNIQUE NOT NULL,
      seed TEXT NOT NULL,
      n INTEGER NOT NULL,
      q TEXT NOT NULL,
      created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
  );
  
  CREATE TABLE api_keys (
      id BIGSERIAL PRIMARY KEY,
      user_name TEXT NOT NULL,
      label TEXT,
      key_prefix TEXT NOT NULL,
      key_hash TEXT NOT NULL UNIQUE,
      is_active BOOLEAN NOT NULL DEFAULT TRUE,
      is_locked BOOLEAN NOT NULL DEFAULT FALSE,
      locked_at TIMESTAMPTZ,
      lock_reason TEXT,
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      last_used_at TIMESTAMPTZ
  );
  
  CREATE TABLE anomaly_events (
      id BIGSERIAL PRIMARY KEY,
      api_key_id BIGINT NOT NULL REFERENCES api_keys(id),
      event_type TEXT NOT NULL,
      details TEXT,
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  ```

## Setup
Buat file `.env` di folder ini:
```env
DATABASE_URL=postgres://postgres:PASSWORD_ANDA@127.0.0.1:5432/fcr_rsa
```

## Cara Penggunaan (CLI)

**1. Buat Kunci Baru**
```bash
cargo run -- keygen --name "kunci_utama"
```

**2. Enkripsi Teks**
```bash
cargo run -- encrypt --text "Halo Dunia" --keyname "kunci_utama"
# Output: BLINDING:CT1,CT2,...
```

**3. Dekripsi Teks**
```bash
cargo run -- decrypt --ciphertext "BLINDING:CT1,CT2,..." --keyname "kunci_utama"
# Output: Halo Dunia
```

## Cara Penggunaan (REST API)

**1. Buat API Key (Bootstrap)**
```bash
cargo run -- apikey create --user emen --label "laptop-emen"
# Simpan plaintext key yang muncul (hanya muncul sekali).
```

**2. Jalankan Server**
```bash
cargo run -- serve
# Server berjalan di http://0.0.0.0:8080 (override via FCR_PORT)
```

**3. Test dengan `curl` atau PowerShell**
```powershell
# Health Check
curl http://127.0.0.1:8080/health

# Enkripsi (Base64)
curl -X POST http://127.0.0.1:8080/encrypt \
  -H "Authorization: Bearer <API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{"key_name":"kunci-utama","text_b64":"SGFsbyBEdW5pYQ==","tier":"b"}'
```

## 🛡️ Auto-Mune (Fitur Eksperimental)
Jika klien mengirim header spatiotemporal (`X-FCR-GPS-Lat`, `X-FCR-GPS-Lon`, `X-FCR-Timestamp`, `X-FCR-Latency`) dan terdeteksi anomali (misal latency > 500ms), server akan:
1. Mencatat kejadian di tabel `anomaly_events`.
2. Mengunci API Key (`is_locked = TRUE`).
3. Mengembalikan **Honeypot Response** (`"data": "HONEYPOT_TRAP_DATA"`) agar penyerang merasa berhasil.

*Catatan: Lihat `../LIMITATIONS.md` untuk daftar batasan dan rencana hardening Auto-Mune.*

## Lisensi
Dual MIT / Apache-2.0. Lihat [../LICENSE](../LICENSE).

