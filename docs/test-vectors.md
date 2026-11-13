# Test Vectors FCR-RSA

## Format

Setiap file test vector adalah JSON dengan struktur:

```json
{
  "version": "1.0",
  "tier": "A" | "B",
  "description": "...",
  "cases": [
    {
      "id": "case-001",
      "params": {
        "n": 100,
        "seed": "hex...",
        "q": "hex..."
      },
      "input": {
        "plaintext": "hex...",
        "key": "hex..."
      },
      "expected": {
        "ciphertext": "hex...",
        "decrypted": "hex..."
      }
    }
  ]
}
```

## File yang Tersedia

| File | Deskripsi |
| :--- | :--- |
| `v1/basic.json` | Enkripsi/dekripsi dasar, Tier B, N kecil |
| `v1/tier-a.json` | Parameter Tier A (N > 500) |
| `v1/tier-b.json` | Parameter Tier B (matriks kecil) |
| `v1/edge-cases.json` | Input kosong, N=0, overflow, dll |

## Cara Menggunakan

### Rust

```bash
cd fcr-core-rust
cargo test --test integration
```

### Python

```bash
cd fcr-python
pytest tests/
```

## Cara Memverifikasi Test Vector

Untuk memverifikasi bahwa implementasi Anda benar:

1. Baca `params` dari kasus uji
2. Jalankan enkripsi pada `input.plaintext` dengan `input.key`
3. Bandingkan hasil dengan `expected.ciphertext`
4. Jalankan dekripsi pada `expected.ciphertext`
5. Bandingkan hasil dengan `expected.decrypted`

Jika hasil tidak cocok, ada bug di implementasi.

## Menambah Test Vector Baru

1. Buat kasus uji di salah satu file yang ada
2. Beri `id` unik
3. Verifikasi hasilnya dengan **dua implementasi** (Rust dan Python)
4. Kirim Pull Request

## Status

Test vector masih dalam penyusunan. Target: 100+ kasus sebelum v1.0.

---

**Terakhir diperbarui:** 13 November 2026