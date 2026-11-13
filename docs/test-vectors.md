# Test Vectors FCR-RSA

Dokumen ini menjelaskan format dan cara menggunakan test vector untuk memverifikasi implementasi FCR-RSA Anda.

## Format JSON

Setiap file test vector menggunakan format standar:

```json
{
  "version": "1.0",
  "tier": "B",
  "cases": [
    {
      "id": "basic-001",
      "params": { "n": 10, "seed": "...", "q": "..." },
      "input": { "plaintext": "...", "blinding": "..." },
      "expected": { "ciphertext": "...", "decrypted": "..." }
    }
  ]
}
```

## File yang Tersedia

| File | Deskripsi |
| :--- | :--- |
| `v1/basic.json` | Enkripsi/dekripsi dasar Tier B (parameter standar) |
| `v1/edge-cases.json` | Kasus batas (N terlalu kecil, Q negatif, dll) |
| `v1/tier-a.json` | Parameter Tier A (menyusul) |
| `v1/tier-b.json` | Parameter Tier B varian (menyusul) |

## Cara Memverifikasi

1. Baca `params` dan `input` dari file JSON.
2. Jalankan fungsi enkripsi pada implementasi Anda.
3. Bandingkan output `ciphertext` Anda dengan `expected.ciphertext`.
4. Jalankan dekripsi pada ciphertext tersebut.
5. Bandingkan hasilnya dengan `expected.decrypted`.

Jika hasilnya cocok (100% identik), berarti implementasi Anda konsisten dengan protokol FCR-RSA v0.1.0.
Jika tidak cocok, periksa kembali penanganan *noise* SHA-256 dan operasi modulo BigInt Anda.

## Catatan Penting

- **Blinding Factor:** Pada test vector ini, `blinding` sengaja dibuat konstan (misal `12345`) agar output deterministik dan bisa dibandingkan. Di produksi, `blinding` harus acak.
- **Error Handling:** Untuk kasus di `edge-cases.json`, implementasi Anda harus mengembalikan error (Rust: `Result::Err`, Python: `ValueError`), bukan menghasilkan ciphertext.
