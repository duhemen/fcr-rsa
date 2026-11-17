# Threat Model FCR-RSA

## 1. Aset yang Dilindungi

- Pesan / data pengguna (plaintext)
- Kunci sesi (session key)
- Kunci privat jangka panjang
- Metadata transaksi (waktu, lokasi, latensi)

## 2. Aktor Ancaman

| Aktor | Kemampuan | Motivasi |
| :--- | :--- | :--- |
| Kriptografer penasaran | Analisis matematis, tools LLL/BKZ | Riset, reputasi |
| Penyerang jaringan | MITM, replay, delay injection | Data, uang |
| Penyerang humaniora | Rekayasa sosial, phishing | Akses kredensial |
| Adversari pasca-kuantum | Komputer kuantum (asumsi) | Dekripsi massal |
| Insider | Akses sah ke sistem | Bermacam-macam |

## 3. Yang Dilawan

- **Serangan matematis dasar:** LLL & BKZ 2.0 (3×3) reduction, noise attack pada SHA-256 (diuji, gagal menembus).
- **Serangan CPA:** via IND-CPA patch dengan random blinding (diuji, gagal menembus).
- **Header Spoofing & Tampering:** via tanda tangan **HMAC-SHA256** *constant-time* pada header telemetri (diuji, ditolak `HTTP 401`).
- **Blind Key Usage (Pencurian Kunci Tanpa SDK):** via kebijakan *fail-closed* telemetri wajib (diuji, ditolak `HTTP 400`).
- **Impossible Travel & MITM Delay:** via evaluasi *Risk Score* berbasis perpindahan grid lokasi dan anomali latensi (diuji, memicu *honeypot* + *cooldown*).
- **Permanent DoS pada Kunci Sah:** dimitigasi dengan mengganti *hard lock* menjadi *self-healing cooldown* 5 menit.
- **Kebocoran Privasi Geolokasi (UU PDP):** dimitigasi dengan *Keyed Geospatial Grid Hashing* (tidak ada koordinat mentah di database).

## 4. Yang TIDAK Dilawan (Saat Ini)

- **BKZ 2.0 dengan parameter besar (matriks > 10×10)** — belum diuji.
- **Algebraic cryptanalysis** — belum diuji.
- **Side-channel timing pada BigInt Rust** — baru verifikasi HMAC yang *constant-time*; operasi aritmetika besar belum diaudit penuh.
- **Fault injection & Physical side-channel** — belum diuji.
- **Supply chain attack** — di luar cakupan protokol.
- **Insider dengan perangkat & lokasi sah** — tidak terdeteksi oleh anomali spatiotemporal.

## 6. Pertanyaan Terbuka & Status Resolusi

- ✅ **Apakah kill-switch bisa disalahgunakan untuk DoS?** *Terjawab (17 Nov 2026):* Ya, jika menggunakan *lock* permanen. Telah diganti dengan *adaptive cooldown* 5 menit sehingga sistem pulih otomatis.
- ✅ **Bagaimana honeypot berinteraksi dengan penyerang?** *Terjawab (17 Nov 2026):* Server mengembalikan JSON `ciphertext_b64` palsu yang identik secara struktural dengan keluaran asli sambil mengaktifkan *cooldown* di latar belakang.
- ✅ **Bagaimana menjaga privasi GPS pengguna?** *Terjawab (17 Nov 2026):* Koordinat dikuantisasi ke grid ~11 km dan di-hash dengan Keyed SHA-256 sebelum menyentuh PostgreSQL.
- ⏳ **Apakah Fibonacci non-linear tetap tahan terhadap BKZ 2.0 dimensi tinggi?** *(Masih terbuka — target pengujian Fase 12).*
- ⏳ **Berapa rasio false positive pada trafik jaringan seluler/satelit dunia nyata?** *(Masih terbuka — butuh uji lapangan).*

---

**Terakhir diperbarui:** 17 November 2026