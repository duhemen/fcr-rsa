# Security Policy

## Status Proyek

FCR-RSA adalah **purwarupa riset**. Belum melalui audit keamanan independen.
**Jangan gunakan untuk melindungi data nyata, uang, atau identitas.**

## Melaporkan Celah Keamanan

Jika Anda menemukan celah keamanan, **jangan** membuka Issue publik.
Laporkan secara privat melalui salah satu jalur berikut:

- Email: (isi dengan alamat email kontak Emen)
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

Terima kasih telah membantu membuat FCR-RSA lebih aman.
