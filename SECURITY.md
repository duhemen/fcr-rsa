# Security Policy

## Status Proyek

FCR-RSA adalah **purwarupa riset**. Belum melalui audit keamanan independen.
**Jangan gunakan untuk melindungi data nyata, uang, atau identitas.**

## Melaporkan Celah Keamanan

Jika Anda menemukan celah keamanan, **jangan** membuka Issue publik.
Laporkan secara privat melalui salah satu jalur berikut:

- Email: (muhammadharamein@gmail.com)
- GitHub Security Advisory: buka tab "Security" di repo ini

Sertakan dalam laporan:
1. Deskripsi celah
2. Langkah reproduksi (kode atau perintah)
3. Dampak yang mungkin terjadi
4. Saran perbaikan (opsional)

## Yang Kami Anggap Sebagai Celah

- Kesalahan matematika pada deret Fibonacci non-linear atau pemetaan Gaussian
- Kegagalan IND-CPA pada mode enkripsi yang diklaim aman
- Kebocoran informasi melalui timing, cache, atau side-channel lain
- Kesalahan pada logika auto-imun atau kill-switch yang bisa dieksploitasi
- Ketidakcocokan antara implementasi Rust dan Python pada input yang sama

## Yang Bukan Celah (untuk Versi Riset Ini)

- Ketidakmampuan melawan serangan yang belum diuji (BKZ 2.0, algebraic, dll)
  — ini keterbatasan yang diketahui, lihat `LIMITATIONS.md`
- Serangan yang membutuhkan akses fisik ke perangkat
- Serangan yang membutuhkan kunci yang sudah bocor sebelumnya

## Komitmen Kami

- Konfirmasi penerimaan laporan: maksimal 7 hari
- Update status: maksimal 30 hari
- Kredit publik untuk pelapor (jika diinginkan)

## Audit Independen

FCR-RSA **membutuhkan** audit independen sebelum bisa dipertimbangkan
untuk produksi. Jika Anda seorang kriptografer yang tertarik melakukan
audit, silakan hubungi kontak di atas.

### ✅ Constant-Time Verification

FCR-RSA v0.6.0 telah melalui **Full Side-Channel Audit (A4)** dengan hasil:

| Aspek | Status | Metode Verifikasi |
|-------|--------|-------------------|
| **Timing Attack Resistance** | ✅ VERIFIED | Welch's t-test (n=10,000, p=0.143) |
| **Cache Attack Resistance** | ✅ VERIFIED | Direct-mapped cache simulation (0% diff) |
| **Cross-Language Consistency** | ✅ VERIFIED | Python Oracle vs Rust (10/10 match) |

### Implementasi Constant-Time

Semua operasi kriptografi inti menggunakan `crypto-bigint::DynResidue` (Montgomery arithmetic) yang menjamin:
- **Tidak ada secret-dependent branching:** Semua limb diproses, terlepas dari nilai bit
- **Tidak ada early exit:** Loop selalu berjalan jumlah iterasi tetap
- **Memory access pattern konstan:** Pola akses cache identik untuk semua input

### Keterbatasan yang Diketahui

1. **Hardware-Level Testing:** Audit dilakukan di level aplikasi. Untuk verifikasi cycle-accurate, diperlukan `dudect_bencher` dengan instruksi `rdtsc` atau `valgrind --tool=callgrind`.
2. **Power Analysis:** Belum diuji. Memerlukan peralatan lab fisik (oscilloscope, EM probe).
3. **Fault Injection:** Belum diuji. Memerlukan clock glitching atau voltage fault injection.

### Rekomendasi Penggunaan

- ✅ **AMAN** untuk riset, prototyping, dan aplikasi non-kritis
- ⚠️ **PERLU AUDIT LANJUTAN** untuk aplikasi keuangan, militer, atau production-grade
- ❌ **JANGAN** digunakan untuk melindungi data sensitif tanpa audit independen

Lihat `docs/side_channel_audit.md` untuk laporan teknis lengkap.

Terima kasih telah membantu membuat FCR-RSA lebih aman.
