# Arsitektur FCR-RSA

## 1. Fondasi Matematika

### A. Deret Fibonacci Non-Linear

Alih-alih penjumlahan, FCR-RSA menggunakan perkalian untuk pertumbuhan
eksponensial yang kacau:

```
F_n = (F_{n-1} * F_{n-2}) + e_n  mod Q
```

- `e_n` : noise LWE deterministik dari `noise_seed` (SHA-256)
- `Q`   : modulus kompleks (Gaussian integer)

### B. Pemetaan Gaussian

Setiap elemen dipetakan ke ruang kompleks:

```
Z_n = Re(F_n) + Im(F_{n+1}) * i
```

### C. IND-CPA Patch (v1.1)

Random blinding factor ditambahkan pada enkripsi:

```
C = (M * E_vec) + N_comp + R_blind
```

Ini mencegah serangan chosen-plaintext attack (CPA).

## 2. Arsitektur Dua Tier

### Tier A — Enterprise / Banking

- Indeks Fibonacci besar (N > 500)
- Big Integer untuk presisi mutlak
- Keamanan maksimal, cocok untuk server

### Tier B — End-User / IoT

- Matriks Fibonacci ringan (2x2 atau 3x3)
- Efisiensi baterai & CPU untuk perangkat mobile

### Bridge Protocol

- Sinkronisasi session key antara Tier B dan Tier A
- Fibonacci heartbeat untuk memastikan komunikasi aman

## 3. Sistem Auto-Imun

### A. Token Spatiotemporal

Setiap transaksi terikat pada:
- **GPS**: mendeteksi anomali lokasi (kunci bocor)
- **Timestamp**: mendeteksi transaksi di luar jam operasional
- **Latensi**: mendeteksi MITM (delay tidak wajar)

### B. Matematical Kill-Switch

Jika anomali terdeteksi:
- Ubah session key secara otomatis
- Blokir IP / Device ID
- Kirim honeytoken untuk melacak penyerang
- **Endemi**: isolasi hanya pada cabang yang bocor, tanpa mengganggu
  jaringan pusat

## 4. Alur Data

```
[Pengirim]
   |
   v
[Tier B / Tier A] --enkripsi--> [Ciphertext]
   |
   v
[Auto-Imun: token spatiotemporal]
   |
   v
[Jaringan / Channel]
   |
   v
[Auto-Imun: verifikasi anomali]
   |
   v
[Tier A / Tier B] --dekripsi--> [Plaintext]
```

## 5. Batasan Arsitektur

Lihat [LIMITATIONS.md](../LIMITATIONS.md). Arsitektur ini belum melalui
audit independen dan **tidak boleh** dipakai untuk produksi.

## Catatan: Blinding Factor

Pada contoh (`examples/basic.rs`, `examples/basic.py`), `blinding` sengaja
dibuat konstan (`12345`) agar ciphertext **reproduktif** dan bisa
dibandingkan lintas-bahasa.

**Di produksi, `blinding` HARUS di-generate secara kriptografis acak**
(misalnya dari `OsRng` di Rust, `secrets.token_bytes` di Python) untuk
setiap enkripsi. Tanpa ini, FCR-RSA tidak memenuhi IND-CPA, karena
plaintext yang sama akan menghasilkan ciphertext yang sama.

---

**Terakhir diperbarui:** 13 November 2026
