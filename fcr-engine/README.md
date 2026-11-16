# FCR-RSA Engine CLI

Engine *Command Line* untuk protokol FCR-RSA. Terhubung ke PostgreSQL untuk manajemen kunci dan mendukung enkripsi teks nyata.

## Fitur
- **PostgreSQL Key Management**: Simpan dan kelola kunci di database.
- **Text I/O**: Enkripsi teks (Base-256 per byte).
- **IND-CPA (Auto-Embedding)**: *Blinding factor* acak disematkan otomatis ke *ciphertext*.
- **Tier A & B**: Mendukung mode enterprise (A) dan mobile (B).

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
  ```

## Setup
Buat file `.env` di folder ini:
```env
DATABASE_URL=postgres://postgres:PASSWORD_ANDA@127.0.0.1:5432/fcr_rsa
```

## Cara Penggunaan

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