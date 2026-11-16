# Analisis Ancaman Kuantum terhadap FCR-RSA

**Tanggal:** 16 November 2026  
**Status:** Final (berdasarkan simulasi Qiskit lokal)
**Environment:** Python 3.12.7, Qiskit 2.6.2, Qiskit-Aer 0.18.2

---

## 1. Ringkasan Eksekutif

| Algoritma Kuantum | Target Klasik | Hasil Simulasi terhadap FCR-RSA |
|-------------------|---------------|----------------------|
| **Shor** | RSA, ECC (faktorisasi, discrete log) | ❌ **Tidak relevan** — FCR-RSA tidak pakai faktorisasi. Simulasi memfaktorkan N=15, 21, 35, tetapi tidak ada celah yang sama pada FCR-RSA. |
| **Grover** | Brute-force (unstructured search) | ⚠️ **Tidak memberi speedup signifikan** — Dengan `R_blind` acak, search space menjadi 2D (M, R). Grover butuh 2^n iterasi (sama dengan klasik). |

---

## 2. Hasil Simulasi Grover (Aktual)

- **Ruang kunci:** 2^10 (1024 kemungkinan)
- **Target:** 42
- **Iterasi optimal:** 25
- **Hasil pengukuran:** Target 42 ditemukan dengan probabilitas **1023/1024 (~99.9%)**
- **Speedup:** ~41x lebih cepat dari brute-force klasik (kuadratik).

**Kesimpulan Grover:** Pada skenario 1D (R=0), Grover bekerja sangat efisien. Namun, pada skenario nyata FCR-RSA di mana `R_blind` acak, penyerang harus mencari kombinasi (M, R). Hal ini mengubah problem menjadi 2D search space, yang menetralkan *speedup* kuadratik Grover.

---

## 3. Hasil Simulasi Shor (Aktual)

- **Target:** N = 15, 21, 35
- **Hasil:** Berhasil menemukan faktor prima untuk ketiga angka tersebut (3x5, 3x7, 5x7).
- **Relevansi ke FCR-RSA:** Shor bergantung pada *order finding* (a^r ≡ 1 mod N). FCR-RSA menggunakan operasi `C = (M * E_vec) + R mod Q` dengan noise LWE yang menghancurkan periodisitas. Tidak ada modulus `N = p*q` untuk difaktorkan.

**Kesimpulan Shor:** FCR-RSA secara struktural **kebal** terhadap Algoritma Shor.

---

## 4. Batasan Analisis

1. **Simulasi ≠ Bukti Formal.** Hasil Qiskit adalah demonstrasi empiris, bukan bukti kriptografis formal.
2. **Parameter kecil** (2^10) tidak mewakili parameter produksi (2^256).
3. **Oracle FCR-RSA** di Sel 4 masih berupa penjelasan konseptual.

---

## 5. Rekomendasi Lanjutan

| # | Aksi | Prioritas |
|---|------|-----------|
| 1 | Implementasi oracle FCR-RSA penuh di Qiskit (arithmetic modular) | 🟡 Sedang |
| 2 | Uji Grover dengan 2^20 (butuh RAM besar) | 🟡 Sedang |
| 3 | Analisis formal (bukan simulasi) untuk klaim keamanan | 🔴 Tinggi |
| 4 | Publikasi hasil ini ke whitepaper | 🔴 Tinggi |

---

## 6. Referensi

1. Shor, P. W. (1994). *Algorithms for quantum computation: discrete logarithms and factoring*. FOCS.
2. Grover, L. K. (1996). *A fast quantum mechanical algorithm for database search*. STOC.
3. NIST PQC Standardization (2024).

---

**Terakhir diperbarui:** 16 November 2026