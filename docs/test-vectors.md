# Test Vectors FCR-RSA (v0.6.0)

Dokumen ini menjelaskan format dan cara menggunakan test vector untuk memverifikasi konsistensi implementasi FCR-RSA lintas bahasa (Rust ↔ Python).

## 🎯 Status Verifikasi
- **Versi Protokol:** v0.6.0 (U2048 + Constant-Time + Hybrid KEM)
- **Cross-Language Match:** ✅ **10/10 Vektor Cocok** (Python Oracle ↔ Rust Implementation)
- **Format Ciphertext:** `BLINDING_NONCE:ENCRYPTED_SEED:AEAD_NONCE:CIPHERTEXT` (semua Base64)

## 📂 Struktur File JSON
Setiap file test vector menggunakan format standar:
```json
{
  "version": "0.6.0",
  "tier": "B",
  "cases": [
    {
      "id": "basic-001",
      "params": { "n": 10, "seed": "fcr-rsa-demo-seed", "q": "1000000007" },
      "input": { "plaintext": "Halo Dunia", "blinding_override": "deterministic-vector-001" },
      "expected": { 
        "ciphertext_4_parts": "bn_b64:es_b64:an_b64:ct_b64",
        "decrypted": "Halo Dunia" 
      }
    }
  ]
}
```

## 📋 File yang Tersedia
| File | Deskripsi | Status |
| :--- | :--- | :---: |
| `v1/basic.json` | Enkripsi/dekripsi dasar Tier B (parameter standar) | ✅ Tersedia |
| `v1/edge-cases.json` | Kasus batas (plaintext kosong, oversized 10KB) | ✅ Tersedia |
| `v1/cross_language.json` | 10 vektor untuk verifikasi Python ↔ Rust | ✅ **10/10 MATCH** |

## 🔬 Cara Memverifikasi
1. Baca `params` dan `input` dari file JSON.
2. Jalankan fungsi enkripsi pada implementasi Anda (Rust atau Python).
3. Pastikan output ciphertext memiliki **4 bagian** yang dipisahkan oleh `:`.
4. Bandingkan output Anda dengan `expected.ciphertext_4_parts`.
5. Jalankan dekripsi pada ciphertext tersebut dan bandingkan dengan `expected.decrypted`.

## ⚠️ Catatan Penting
- **Blinding Factor:** Pada test vector, `blinding_override` sengaja dibuat konstan (misal: `"deterministic-vector-001"`) agar output deterministik dan bisa dibandingkan lintas-bahasa. 
- **Produksi:** Di lingkungan produksi, parameter ini **TIDAK DIGUNAKAN**. Blinding nonce di-generate secara kriptografis acak (`OsRng`) untuk setiap sesi guna menjamin properti IND-CPA.
- **Error Handling:** Untuk kasus di `edge-cases.json`, implementasi harus mengembalikan error yang jelas (Rust: `Result::Err`, Python: `ValueError`), bukan menghasilkan ciphertext rusak.
