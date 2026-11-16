# BKZ 2.0 Lattice Attack Simulation

Folder ini berisi simulasi **Block Korkine-Zolotarev (BKZ) 2.0** terhadap FCR-RSA Tier B.

## ⚠️ Kenapa Harus di Colab/Linux?

`fpylll` (library Python untuk LLL/BKZ) **tidak tersedia untuk Windows** secara resmi. Kompilasi dari source gagal karena butuh `gmp.h` (GMP library) yang tidak ada di Windows default.

**Solusi:** Jalankan notebook ini di:
- **Google Colab** (Linux, gratis) — direkomendasikan
- **WSL2** (Windows Subsystem for Linux)
- **Docker** (`python:3.11-slim` + `apt install libgmp-dev`)
- **Linux/macOS native**

## 📂 Isi Folder

| File | Deskripsi |
|------|-----------|
| `README.md` | File ini |
| `bkz_attack.ipynb` | Notebook Colab (hasil eksperimen 13 Nov 2026) |
| `bkz_attack.py` | Versi script (hasil *download* dari Colab) |

## 🔬 Parameter Uji

| Parameter | Nilai | Catatan |
|-----------|-------|---------|
| `Q` | `1000000007` | Modulus (prime, 30-bit) |
| `E_vec` | `779117744` | `seq[8]` untuk seed `"fcr-rsa-demo-seed"`, `n=10` |
| `M` | `42` | Plaintext asli (disembunyikan dari penyerang) |
| `R_blind` | `12345` | Blinding factor (disembunyikan) |
| `C` | `(M*E_vec + R) mod Q` | Ciphertext (dilihat penyerang) |
| `block_size` | `20` | BKZ parameter (overkill untuk 3×3) |
| `max_loops` | `4` | BKZ parameter |

## 📊 Hasil Eksperimen (13 Nov 2026)

**Status:** ✅ **BKZ 2.0 GAGAL MENEMBUS**

- LLL + BKZ 2.0 dijalankan pada matriks 3×3 embedding.
- Vektor terpendek yang ditemukan **bukan** `(42, 12345)`.
- Penyerang tidak bisa membedakan pesan asli dari *decoy vectors*.

## ⚠️ Batasan (Jujur)

1. **Matriks 3×3 terlalu kecil.** Ini hanya membuktikan *satu instance*, bukan klaim umum.
2. **Parameter `Q` 30-bit** relatif kecil. Untuk produksi, `Q` harus ≥ 256-bit.
3. **Tier A (dengan `N_comp`)** belum diuji dengan BKZ.
4. **Belum ada uji dengan matriks 10×10 atau 20×20** — ini PR untuk Fase 4.

## 🔁 Cara Menjalankan Ulang

### Di Google Colab
1. Buka [colab.research.google.com](https://colab.research.google.com)
2. Upload `bkz_attack.ipynb`
3. Runtime → Run all
4. Lihat output di sel terakhir

### Di WSL2 / Linux
```bash
sudo apt install libgmp-dev python3-dev
pip install fpylll
python bkz_attack.py
```

## 📝 Catatan untuk Whitepaper

Hasil ini **tidak boleh** diklaim sebagai "FCR-RSA tahan BKZ 2.0 secara umum". Yang benar:
> "Pada parameter uji (`Q = 10^9`, matriks 3×3), BKZ 2.0 dengan `block_size=20` gagal menemukan plaintext. Pengujian dengan parameter lebih besar masih diperlukan."

---

**Terakhir diperbarui:** 16 November 2026
