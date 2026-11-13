# Limitations — Apa yang TIDAK Dijamin oleh FCR-RSA

Dokumen ini jujur tentang batasan FCR-RSA. Membacanya **sebelum** menggunakan
FCR-RSA adalah wajib.

## 1. Status Keamanan

- **Belum diaudit secara independen.** Semua klaim keamanan berasal dari
  pengujian internal (red team) yang dilakukan oleh tim yang sama yang
  merancang protokol. Ini bukan jaminan yang cukup untuk produksi.
- **Belum diuji terhadap BKZ 2.0** dengan parameter agresif. Red team yang
  ada hanya menggunakan LLL dasar.
- **Belum diuji terhadap algebraic cryptanalysis** dan teknik analisis
  modern lainnya.
- **Belum diuji terhadap quantum simulation** (Shor, Grover).

## 2. Asumsi Kriptografi

- Keamanan FCR-RSA bergantung pada asumsi bahwa deret Fibonacci non-linear
  dengan noise LWE deterministik sulit dibalik. **Asumsi ini belum
  diverifikasi secara formal.**
- Jika asumsi LWE dengan parameter yang digunakan runtuh, FCR-RSA runtuh.
- Pemilihan modulus Gaussian integer `Q` belum melalui analisis
  kriptanalitik yang ketat.

## 3. Implementasi

- **Rust:** Operasi `BigInt` belum diverifikasi constant-time.  Ada kemungkinan side-channel timing.
- **Python:** Implementasi Python adalah untuk referensi dan pengujian,
  **bukan** untuk produksi. Python tidak cocok untuk operasi kriptografi
  yang sensitif terhadap waktu.
- **Zeroization:** Belum ada jaminan bahwa secret di memori dihapus dengan
  aman setelah digunakan.
- **Fuzzing:** Belum ada fuzzing yang komprehensif terhadap parser input.

## 4. Fitur yang Diklaim tapi Belum Teruji

- **Auto-imun (token spatiotemporal):** Logika deteksi anomali belum diuji
  terhadap data dunia nyata. False positive dan false negative belum diukur.
- **Kill-switch matematika:** Belum diuji dalam skenario serangan nyata.
  Risiko: kill-switch yang salah picu bisa menyebabkan denial of service.
- **Honeytoken:** Konsep ada, implementasi belum ada. Belum jelas bagaimana
  honeytoken berinteraksi dengan transaksi sah.

## 5. Interoperabilitas

- **Belum ada standar.** FCR-RSA bukan bagian dari NIST, IEEE, 3GPP, atau
  standar lain. Tidak ada jaminan interoperabilitas dengan sistem lain.
- **Belum ada test vector resmi** yang diverifikasi oleh pihak ketiga.
- **API belum stabil.** Antara versi 0.x, API bisa berubah tanpa notice.

## 6. Performa

- Benchmark yang ada (Tier A: ~0.5ms dekripsi, Tier B: ~0.05ms) diukur
  pada perangkat pengembangan, **bukan** pada perangkat target (server
  bank, IoT, mobile).
- Belum ada benchmark pada perangkat dengan memori terbatas (<64KB).
- Belum ada benchmark pada beban konkuren (multi-thread, multi-user).

## 7. Cakupan Penggunaan

FCR-RSA **tidak dirancang** untuk:
- Melindungi data yang harus tetap aman selama >30 tahun (butuh analisis
  umur simpan kunci).
- Menggantikan protokol yang sudah terstandar (TLS, SSH, Signal) tanpa
  kajian interoperabilitas.
- Digunakan oleh pengguna akhir tanpa pendampingan teknisi.

## 8. Vektor Serangan yang Tidak Dicakup

FCR-RSA **tidak** mengklaim bisa melawan:
- Serangan yang mengeksploitasi bug implementasi (buffer overflow, RCE)
  — ini tanggung jawab bahasa dan runtime, bukan protokol.
- Serangan physical (side-channel daya, EM, fault injection) — belum
  diuji sama sekali.
- Serangan supply chain (kompromi dependensi, build server).
- Serangan humaniora yang tidak terdeteksi oleh token spatiotemporal
  (misalnya insider yang punya akses sah).

## 9. Batasan Filosofis

FCR-RSA dibangun atas prinsip "kekacauan yang teratur". Tapi kekacauan
yang tidak dipahami bisa menjadi **celah**, bukan **fitur**. Setiap klaim
tentang "keamanan dari kekacauan" harus diuji, bukan dipercaya.

## 10. Komitmen Kami

Kami akan memperbarui dokumen ini setiap kali ada temuan baru. Jika Anda
menemukan klaim di README atau whitepaper yang **tidak** tercakup di sini,
itu adalah bug dokumentasi—laporkan.

---

**Terakhir diperbarui:** 13 November 2026
**Versi:** 0.1.0-research
