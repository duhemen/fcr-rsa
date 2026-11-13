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

- **Serangan matematis dasar:** LLL reduction, noise attack pada SHA-256
  (diuji, gagal menembus)
- **Serangan CPA:** via IND-CPA patch (diuji, gagal menembus)
- **MITM dengan delay tidak wajar:** via token latensi
- **Transaksi di luar jam operasional:** via token timestamp
- **Anomali lokasi:** via token GPS
- **Isolasi cabang bocor:** via kill-switch endemi

## 4. Yang TIDAK Dilawan (Saat Ini)

- **BKZ 2.0** dengan parameter agresif — belum diuji
- **Algebraic cryptanalysis** — belum diuji
- **Quantum simulation** — belum diuji
- **Side-channel timing** — implementasi Rust belum constant-time
- **Fault injection** — belum diuji
- **Supply chain attack** — belum ada mitigasi
- **Insider dengan akses sah** — di luar cakupan protokol
- **Serangan physical** — di luar cakupan

## 5. Asumsi

- SHA-256 tetap aman (untuk `noise_seed`)
- Modulus Gaussian `Q` cukup besar untuk mencegah faktorisasi
- Parameter Tier A/B dipilih dengan benar
- Pengguna tidak membocorkan kunci secara sengaja

## 6. Pertanyaan Terbuka

- Apakah Fibonacci non-linear benar-benar menghasilkan kekacauan yang
  cukup untuk mencegah LLL/BKZ? (perlu pengujian lebih lanjut)
- Apakah auto-imun bisa false positive dalam produksi?
- Bagaimana honeytoken berinteraksi dengan transaksi sah?
- Apakah kill-switch bisa disalahgunakan untuk DoS?

---

**Terakhir diperbarui:** 13 November 2026