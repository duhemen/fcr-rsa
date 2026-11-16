# Quantum Simulation — Grover & Shor

Folder ini berisi simulasi **ancaman pasca-kuantum** terhadap FCR-RSA menggunakan **Qiskit**.

## 🎯 Tujuan

| Simulasi | Tujuan | Relevansi ke FCR-RSA |
|----------|--------|----------------------|
| **Grover** | Quantum search (√N speedup) | Menguji ketahanan terhadap brute-force kuantum |
| **Shor** | Quantum factorization | **Membuktikan FCR-RSA tidak bergantung pada faktorisasi** |

## ⚠️ Batasan (Jujur)

1. **Simulasi ≠ Bukti Formal.** Grover pada 2^10 hanya menunjukkan *speedup* kuadratik, bukan klaim keamanan.
2. **Shor pada N kecil** (15, 21, 35) hanya demonstrasi—bukan ancaman nyata.
3. **FCR-RSA tidak menggunakan faktorisasi**, jadi Shor **tidak relevan** secara langsung. Ini poin penting untuk whitepaper.
4. **Qiskit di Windows bisa jalan**, tapi untuk qubit > 20 butuh RAM besar → gunakan Colab.

## 📂 Isi Folder

| File | Deskripsi |
|------|-----------|
| `README.md` | File ini |
| `grover_simulation.ipynb` | Notebook Grover (2^10 + oracle FCR-RSA) |
| `shor_simulation.ipynb` | Notebook Shor (demonstrasi pada N kecil) |
| `requirements.txt` | Dependensi Qiskit |

## 🔬 Metodologi

### Grover
- **Ruang kunci:** 2^10 = 1024 kemungkinan plaintext (0..1023)
- **Oracle:** `f(x) = 1` jika `x == target`, else `0`
- **Iterasi optimal:** `⌊π/4 · √N⌋ ≈ 25` untuk N=1024
- **Output:** Probabilitas target terukur vs teori

### Shor
- **Target:** N = 15, 21, 35 (faktorisasi kecil)
- **Tujuan:** Menunjukkan Shor **hanya** bekerja pada struktur perkalian prima
- **Kesimpulan:** FCR-RSA (Fibonacci non-linear + LWE) **tidak memiliki struktur periodik** yang dibutuhkan Shor

## ⚡ Cara Menjalankan

### Di Google Colab (Direkomendasikan)
1. Buka [colab.research.google.com](https://colab.research.google.com)
2. Upload `grover_simulation.ipynb` atau `shor_simulation.ipynb`
3. Runtime → Run all
4. Lihat output di sel terakhir

### Di Lokal (Windows/Linux)
```bash
pip install -r requirements.txt
jupyter notebook grover_simulation.ipynb
```

## 📊 Hasil yang Diharapkan

### Grover
- Setelah ~25 iterasi, probabilitas mengukur `target` mendekati **1.0** (teori: ~0.999).
- Grafik: probabilitas vs jumlah iterasi (osilasi khas Grover).

### Shor
- Berhasil memfaktorkan N=15 → (3, 5), N=21 → (3, 7), N=35 → (5, 7).
- **Tidak ada** "faktorisasi" yang bisa dilakukan pada FCR-RSA karena bukan produk prima.

## 📝 Catatan untuk Whitepaper

> "Grover's algorithm provides a quadratic speedup for unstructured search, reducing FCR-RSA's effective key space by half. However, FCR-RSA's security is not based on search space size alone—it relies on the hardness of inverting non-linear Fibonacci recurrence with LWE noise. Shor's algorithm, which breaks RSA/ECC, is inapplicable to FCR-RSA because the protocol does not rely on integer factorization or discrete logarithms."

---

**Terakhir diperbarui:** 16 November 2026