# KERANGKA ACUAN KERJA (KAK) / TERMS OF REFERENCE (TOR)
## Pengembangan Protokol Transpor Kriptografi Pasca-Kuantum, Adaptif ASI, & Efisien Jaringan: FCR-TP (*FCR Transport Protocol*)

**Kode Proyek:** FCR-TP-v1.0-Baseline (Integrasi Ekosistem `fcr-rsa`)  
**Inisiator & Peneliti Utama:** Emen (`duhemen`)  
**Kolaborator Riset AI:** Google Mode AI, DeepSeek, Qwen  
**Status Proyek:** Riset Independen & *Open-Source* (Non-Komersial)  
**Target Integrasi:** `fcr-core-rust` (`v0.6.1-hardened`+) & `fcr-engine`  
**Posisi Roadmap Utama:** Sub-Proyek Fase 4.5 / Fase 5 (Pasca-Kunci Audit Eksternal `v0.6.1-audit-ready`)

---

## 1. Latar Belakang
Proyek kriptografi pasca-kuantum **FCR-RSA** (`v0.6.1-hardened`) telah berhasil membangun fondasi pertahanan berlapis melalui kombinasi Deret Fibonacci Non-Linear, Bilangan Imajiner (*Gaussian Integers*), *Learning With Errors* (LWE) dengan modulus **U2048 Montgomery** (berbasis `crypto-bigint v0.7+`), skema *Hybrid KEM + ChaCha20-Poly1305*, serta lapisan pertahanan aktif **Auto-Imun** (*Spatiotemporal Token*, *Deceptive Honeypot*, dan *Poisoned Pill*).

Namun, untuk menghadapi dua realitas sekaligus—yakni transisi menuju **era komputasi kuantum (superposisi)** dan ancaman **ASI (*Artificial Superintelligence*)** di satu sisi, serta **ketimpangan kualitas infrastruktur jaringan** (mulai dari fiber optik berkecepatan tinggi hingga sinyal seluler/satelit lemah di daerah terpencil) di sisi lain—lapisan transpor jaringan tidak dapat lagi bergantung pada protokol statis seperti HTTP REST API dengan pengkodean Base64. Terdapat empat tantangan strategis yang harus diatasi:
1. **Overhead & Keterbatasan MTU:** Pengkodean Base64 memperbesar ukuran muatan (~33%) dan rentan memicu *IP Fragmentation*.
2. **Kerentanan *Traffic Fingerprinting* oleh ASI:** Protokol dengan *header* tidak terenkripsi (*plaintext*) dan pola distribusi ukuran paket yang tetap mudah dideteksi dan dipetakan oleh kecerdasan buatan tingkat lanjut.
3. **Kebutuhan Pertahanan Aktif Otonom di Lapisan Jaringan:** Serangan skala masif (DDoS terkoordinasi, *timing probe*, dan *fuzzing*) harus ditangani secara otomatis dan terukur di level paket sebelum membebani mesin kriptografi maupun aplikasi.
4. **Ketimpangan Lebar Pita & Latensi (*Bandwidth Asymmetry*):** Fitur kamuflase lalu lintas (*chaffing* dan *padding* besar) berisiko melumpuhkan koneksi pengguna di jaringan berkecepatan rendah (*self-DoS*) jika tidak dilengkapi mekanisme adaptasi jalur.

Oleh karena itu, dikembangkan **FCR-TP (*FCR Transport Protocol*)**—sebuah protokol transpor biner polimorfik berbasis **UDP** yang *crypto-agile*, dilengkapi sistem imun jaringan otonom, serta mampu menyesuaikan diri secara elastis terhadap kondisi bandwidth jaringan dan ramah bagi pengguna akhir (*end-user*).

---

## 2. Tujuan Pengembangan
1. **Membangun Transpor Biner Polimorfik & *Zero-Plaintext Header*:** Menghilangkan ketergantungan pada HTTP/Base64 serta mengenkripsi seluruh *header* transpor agar lalu lintas jaringan terlihat seperti derau acak (*uniform random noise*) bagi pengamat eksternal maupun ASI.
2. **Mewujudkan *Crypto-Agility* Native (Era Superposisi):** Memungkinkan pergantian ukuran modulus dan profil algoritma PQC (U2048, U4096, atau hibrida masa depan) secara dinamis di tengah sesi berjalan (*In-Band Algorithmic Hopping*) termasuk dukungan *Chunked Handshake* untuk parameter besar di jalur sempit.
3. **Menanamkan Sistem Imun Jaringan Otonom (*Autonomous Network Immune System*):** Mengintegrasikan sensor telemetri paket dan 4 level respon terukur untuk menghadapi serangan DDoS, pemindaian *timing*, dan upaya peretasan secara mandiri tanpa merugikan pengguna sah di balik NAT publik.
4. **Menjamin Efisiensi & Keadilan Akses (*Survival Mode* & *Invisible UX*):** Menyediakan mesin adaptasi jaringan otomatis serta opsi manual hemat data/baterai yang bekerja senyap di latar belakang tanpa membebani pengguna dengan kerumitan teknis.
5. **Mempertahankan Jaminan *Constant-Time* & *Formal Verification*:** Menjaga ketahanan *side-channel* yang telah teruji pada `fcr-core-rust` (bebas dari celah *timing* `RUSTSEC-2026-0387`) serta memodelkan keamanan *Cryptographic Core State Machine* menggunakan **Lean 4**.

---

## 3. Ruang Lingkup Pekerjaan
Pengembangan ini dilakukan secara mandiri (riset personal *open-source*) pada *branch* eksperimental terpisah (`feat/fcr-tp-transport`) agar tidak mengganggu *pipeline* CI/CD audit `v0.6.1`, dengan cakupan:
* **Perancangan Spesifikasi Protokol Adaptif:** Definisi *frame* biner polimorfik, *Short Header* terkompresi, *Chunked Handshake*, mekanisme *chaffing*, dan *Client-Initiated Fibonacci Port Hopping*.
* **Implementasi Mesin Transpor, Sensor Imun, & *Congestion Engine* (Rust):** Pembuatan pustaka jaringan asinkron berbasis `tokio` yang terhubung langsung ke `fcr-core-rust` dan *Shared Immune Bus* milik `fcr-engine`.
* **Verifikasi Formal Terfokus & Simulasi Adversarial:** Pemodelan keamanan transisi *state* kriptografi inti (manajemen kunci, *nonce*, dan *epoch*) di **Lean 4** (`formal-verification/`), serta pengujian brutal terhadap simulasi DDoS, *Replay Attack*, *Timing Probe*, *Packet Tampering*, CGNAT *traversal*, dan emulasi jaringan berkecepatan rendah.
* *Catatan: Proyek ini bersifat non-komersial sehingga tidak mencakup pengadaan infrastruktur komersial maupun pembiayaan operasional.*

---

## 4. Struktur Desain & Arsitektur Sistem

### 4.1 Empat Pilar Adaptif (Era Superposisi & ASI)
1. ***Crypto-Agility* & *Chunked Handshake* (`ProfileID`):** Protokol mendukung negosiasi dan rotasi profil kriptografi secara *in-band* (misalnya transisi dari `U2048` ke `U4096`). Jika profil besar (`U4096`) dijalankan di jaringan dengan MTU terbatas, protokol otomatis memecah muatan *handshake* menjadi 2–3 fragmen terurut di lapisan FCR-TP (*Chunked Handshake*) untuk mencegah *IP Fragmentation*.
2. ***Zero-Plaintext Header*:** Hanya 8-byte *Ephemeral Routing Token* acak yang terlihat di jaringan; seluruh metadata (`Ver`, `Tier`, `PktType`, `Sequence`, dan `Grid Hash`) dienkripsi penuh menggunakan kunci sesi sementara.
3. ***Chaffing & Adaptive Traffic Shaping*:** Penyisipan paket umpan berisi derau acak (*chaff packets*) serta penyesuaian pola ukuran/waktu paket (*protocol mimicry* menyerupai lalu lintas VoIP/WebRTC) untuk mengecoh analisis pola oleh ASI.
4. ***Client-Initiated Fibonacci Port Hopping* (Kompatibel CGNAT):** Koneksi terikat pada identitas kriptografis sesi. Saat jadwal lompatan *port* berdasarkan deret Fibonacci sesi tiba, **klien selalu bertindak sebagai pengirim pertama** ke *port* baru *server* (*UDP Hole Punching*) agar *router* CGNAT operator seluler membuka jalur balik secara sah tanpa memblokir paket *server*.

### 4.2 Struktur *Frame* Biner Polimorfik FCR-TP
Setiap paket UDP membawa *frame* biner dengan struktur adaptif sebagai berikut:

```text
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

|         Ephemeral Routing Token (Obfuscated Session ID)       | (8 Bytes - Plaintext Acak)
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

|                                                               |
|   ENCRYPTED HEADER (Ver | Tier | ProfileID | PktType | Seq    | (32 Bytes / 4 Bytes Short)
|   | Spatiotemporal Grid Hash | Flags)                         |
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

|              Header & Telemetry AEAD Auth Tag                 | (16 Bytes)
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

|                                                               |
|   DYNAMIC KEM PAYLOAD (U2048 / U4096 / Chunked Hybrid PQC)    | (Panjang Dinamis sesuai ProfileID)
|   [Hanya muncul pada PktType = HANDSHAKE / KEY_HOP]           |
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

|                                                               |
|   AEAD STREAM PAYLOAD + POLYMORPHIC CHAFF PADDING             | (Ukuran Elastis sesuai Profil Jaringan)
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

**Penjelasan Komponen Utama:**
* **`Ephemeral Routing Token` (64-bit):** Penanda rute sementara yang terus berubah tiap rotasi kunci untuk mencegah pelacakan sesi dan membedakan pengguna sah dari penyerang di dalam satu IP CGNAT yang sama.
* **`Encrypted Header` (256-bit Full / 32-bit Short Mode):** Memuat versi protokol, profil *Tier A/B*, `ProfileID` (penentu ukuran KEM), tipe paket (`HANDSHAKE`, `DATA`, `KEY_HOP`, `POW_CHALLENGE`, `PORT_MUTATE`), nomor urut monotonik, dan `Spatiotemporal Grid Hash` (grid privasi ~11 km sesuai UU PDP).
* **`Dynamic KEM Payload`:** Memuat `BLINDING_NONCE` (16-Byte) dan `ENCRYPTED_SEED` (256-Byte untuk U2048 atau 512-Byte untuk U4096, mendukung fragmentasi internal).
* **`AEAD Stream + Chaff Padding`:** Muatan terenkripsi **ChaCha20-Poly1305** dengan ukuran *padding* yang menyesuaikan profil kualitas jaringan secara dinamis.

---

## 5. Sistem Imun Jaringan Otonom (*Autonomous Network Immune System*)

### 5.1 Sensor Telemetri Lapisan Paket & Pencegahan *False Positive*
Sebelum paket diteruskan ke dekripsi inti, mesin transpor mengevaluasi empat parameter secara mikrodetik:
Gunakan kode dengan hati-hati.
1. Anomali Volumetrik (Rate & Burst): Lonjakan paket per detik dari sumber yang belum terautentikasi (indikasi DDoS / UDP Flood). Catatan: Lonjakan paket dari sesi dengan Routing Token tervalidasi setelah putus sinyal sementara (reconnection burst) dikecualikan dari penalti DDoS dan dialihkan ke SURVIVAL_LITE.
2. Anomali Entropi (Entropy Drift): Upaya fuzzing atau injeksi paket acak yang gagal melewati autentikasi header.
3. Anomali Ritme Waktu (Timing Probe): Interval pengiriman paket yang terlalu presisi secara mekanis (indikasi pemetaan side-channel oleh AI).
4. Anomali Spasial & Hop (Desync): Ketidaksesuaian jadwal lompatan port Fibonacci atau lonjakan Grid Hash yang mustahil secara fisik.

5.2 Matriks Respon Bertingkat & Terukur (Graduated Countermeasures)

FCR-TP menghindari pemblokiran total berbasis IP (hard-drop) yang berisiko memutus akses warga biasa di balik Wi-Fi publik/CGNAT, dan menggantinya dengan empat tingkat respon otonom berbasis skor ancaman (Transport Threat Level / TTL):

Level AncamanKategori SeranganTindakan Otomatis FCR-TP (Countermeasure)Dampak Terhadap Penyerang (ASI / Botnet)
Level 1
(Skor 30–59)Probing Ringan / Timing Scan1. Adaptive Jitter & Chaff Injection
Mengacak jeda waktu balasan secara non-linear dan menyuntikkan paket derau palsu.Merusak akurasi model analisis waktu dan pola lalu lintas milik ASI.
Level 2
(Skor 60–79)DDoS / Volumetric Flood2. Stateless Proof-of-Work (PoW) Puzzle
Server berhenti mengalokasikan memori sesi bagi pengirim tak dikenal dan membalas dengan teka-teki matematis ringan.Memaksa penyerang membakar daya komputasi CPU/GPU mereka sendiri untuk setiap paket (asymmetric cost), sementara beban memori server mendekati 0 byte.
Level 3
(Skor 80–94)Targeted Exploit / Spoofing3. Silent Rerouting ke Transport Honeypot
Koneksi dialihkan secara senyap ke virtual socket terisolasi yang membalas dengan Poisoned Pill (trace_id) berformat U2048 palsu.Penyerang mengira berhasil menembus sistem, padahal sedang mendekripsi data umpan yang melacak balik identitasnya.
Level 4
(Skor 95–100)Serangan Skala Masif (Critical Siege)4. Autonomous Fibonacci Port Mutation
Klien sah menerima sinyal in-band untuk melompat ke port UDP baru sesuai deret Fibonacci sesi, sementara port lama dijadikan sinkhole.Komunikasi klien sah tetap berjalan mulus tanpa putus, sementara seluruh banjir DDoS terisolasi di port lama.


5.3 Koordinasi Shared Immune Bus

Lapisan transpor FCR-TP dan fcr-engine terhubung melalui jalur memori internal dua arah: anomali jaringan yang ditangkap FCR-TP otomatis menaikkan Risk Score pada database forensik fcr-engine, dan sebaliknya anomali aplikasi memerintahkan transpor mengaktifkan Silent Rerouting atau Port Mutation.

6. Mesin Adaptasi Efisiensi Jaringan & Perspektif Humaniora (End-User)


6.1 Mekanisme Adaptasi Jalur (Adaptive Bandwidth & Survival Mode)

Untuk menjamin protokol tetap berjalan andal di wilayah dengan infrastruktur jaringan terbatas (sinyal seluler lemah, latensi tinggi, atau packet loss), FCR-TP menerapkan empat mekanisme efisiensi berbasis metrik RTT, Jitter, dan Packet Loss:
1. Adaptive Chaff & Padding Throttling: Sistem otomatis mengurangi hingga menghentikan penyuntikan paket umpan (chaff = 0%) saat bandwidth menyempit, sehingga 100% kapasitas jalur diprioritaskan bagi data asli pengguna.
2. Dynamic Path MTU Sizing: Ukuran paket disesuaikan secara otomatis—menggunakan paket besar (1.200–1.400 byte) di jaringan stabil untuk memaksimalkan throughput, dan memecah aliran menjadi paket mikro (256–512 byte) di jaringan buruk agar peluang paket tembus tetap tinggi.
3. Zero-Overhead Short Header Mode: Setelah fase Handshake selesai dan sesi dinyatakan aman (Risk Score < 30), paket data selanjutnya menggunakan header terkompresi sehingga overhead protokol turun menjadi < 12 byte per paket.
4. Asimetri Beban Tier A / Tier B: Perangkat klien (Tier B) dibebaskan dari beban komputasi berat seperti pemrosesan chaff masif atau Proof-of-Work berulang; seluruh beban pertahanan aktif diambil alih oleh server pusat (Tier A).

6.2 Prinsip Humaniora & Pengalaman Pengguna (Invisible UX)

• Keamanan Tanpa Friksi (Invisible Security): Seluruh peralihan profil jaringan maupun lompatan port berjalan 100% senyap di latar belakang tanpa menampilkan pesan galat teknis yang membingungkan pengguna awam.
• Keadilan Digital & Kontrol Sadar (Manual Data-Saver Toggle): Selain perpindahan otomatis berdasarkan kualitas sinyal, lapisan API menyediakan sakelar manual bagi pengguna untuk mengunci mode SURVIVAL_LITE kapan saja guna menghemat kuota internet dan daya baterai perangkat.

Tabel Profil Operasi Otomatis FCR-TP


Kondisi Jaringan TerdeteksiProfil Transpor OtomatisUkuran PaketKamuflase (Chaff & Padding)Tujuan Utama
Optimal (RTT < 50ms, Loss < 0.5%)STEALTH_MAX1.200 – 1.400 B100% Aktif (Ukuran seragam + paket umpan)Keamanan maksimal & anti-analisis ASI
Menengah (RTT 50–200ms, Loss 0.5–3%)BALANCED_AGILE800 – 1.000 BAdaptif (Hanya padding minimal, 0% chaff)Menjaga kecepatan & keamanan seimbang
Kritis / Pelosok / Hemat Kuota (RTT > 200ms, Loss > 3%, atau Manual)SURVIVAL_LITE256 – 512 B (Mikro)Nonaktif (Short Header < 12B, 0% Chaff)Menjamin koneksi tidak putus & hemat kuota/baterai


7. Kebutuhan Teknis & Spesifikasi Sistem


7.1 Kebutuhan Perangkat Lunak (Software & Crates)

• Bahasa Pemrograman Utama: Rust (Edisi 2021+, wajib lolos cargo clippy -- -D warnings dan cargo audit).
• Runtime Asinkron Jaringan: tokio (tokio::net::UdpSocket, tokio::time, dan tokio::sync untuk Shared Immune Bus).
• Serialisasi Biner: bytes (manipulasi zero-copy buffer) tanpa dependensi serde_json.
• Kriptografi Inti: fcr-core-rust (v0.6.1-hardened dengan crypto-bigint >= 0.7.6), chacha20poly1305, hkdf, sha2, hmac, dan subtle (constant-time operations).
• Verifikasi Formal Terfokus & Pengujian: Lean 4 (std4 v4.36.0) khusus untuk membuktikan integritas Cryptographic Core State Machine (siklus kunci, nonce unik, dan transisi epoch), serta Python 3.10+ (dengan lingkungan terisolasi .venv sesuai PEP 668) untuk simulasi adversarial, fuzzing, dan emulasi jaringan buruk (netem / packet loss & CGNAT simulation).

7.2 Kebutuhan Lingkungan Pengembangan (Hardware Minimal)

• Prosesor: Arsitektur x86_64 atau ARM64 dengan dukungan instruksi SIMD.
• Memori (RAM): Minimal 4 GB (untuk kompilasi Rust, build Lean 4, dan simulasi ribuan sesi UDP konkuren).
• Sistem Operasi: Linux (disarankan untuk pengukuran latensi jaringan presisi dan simulasi traffic control) atau Windows/WSL2.

8. Roadmap Pengembangan (FCR-TP)

Pengembangan dilaksanakan pada branch feat/fcr-tp-transport setelah branch main mencapai status v0.6.1-audit-ready, dibagi ke dalam empat tahap terukur:

FaseNama TahapanTarget Keluaran (Deliverables)Indikator Keberhasilan
Tahap 1Spesifikasi & Polymorphic Parser• Dokumen spesifikasi docs/transport_protocol.md
• Modul fcr-tp-packet di Rust (encode/decode biner, Encrypted Header, Short Header < 12B, & Chunked Handshake)• 100% unit test parsing biner lulus (zero-copy).
• Paket Handshake U2048 < 1.200 byte (pas 1 MTU) dan U4096 terfragmentasi mulus di mode mikro.
Tahap 2Inti UDP, Crypto-Agility & Congestion Engine• Implementasi server & client asinkron (tokio UDP)
• Dukungan ProfileID (U2048/U4096), Client-Initiated Fibonacci Port Hopping (CGNAT-safe), dan transisi STEALTH_MAX $\leftrightarrow$ SURVIVAL_LITE• Pertukaran kunci PQC dan streaming ChaCha20 berjalan mulus lewat UDP & NAT.
• Koneksi tetap bertahan stabil pada simulasi packet loss tinggi dan perpindahan port dinamis.
Tahap 3Sistem Imun Otonom & Bukti Lean 4• Sensor telemetri paket (bebas false positive saat reconnection burst) & 4 Level Respon Terukur
• Chaffing, Adaptive Jitter, dan bukti Lean 4 untuk Cryptographic Core State Machine• Simulasi DDoS memicu Stateless PoW tanpa lonjakan memori server dan tanpa memutus pengguna sah di IP yang sama.
• Serangan spoofing masuk ke Honeypot & lake build Lean 4 hijau.
Tahap 4Uji Brutal (45+ Assertions) & CI Gate• Ekstensi test_brutal_suite_tp.py (uji DDoS flood, packet loss, timing probe, CGNAT hop, survival mode, dan kompatibilitas 45 assertions engine)
• Integrasi job baru di .github/workflows/• Lulus 100% skenario uji brutal jaringan (≥ 45 assertions PASS).
• Lolos cargo audit (0 vulnerabilities) & siap diaudit komunitas.


9. Penutup

Dokumen KAK/TOR (FCR-TP-v1.0-Baseline) ini menjadi cetak biru teknis resmi dalam merancang dan mengimplementasikan FCR-TP sebagai lapisan transpor pasca-kuantum yang adaptif terhadap era superposisi dan ASI, mandiri secara imunologis, efisien di seluruh spektrum infrastruktur jaringan, serta berpihak pada kemudahan pengguna manusia bagi ekosistem FCR-RSA (v0.6.1-hardened+). Seluruh pengembangan bersifat terbuka untuk tinjauan sejawat (peer-review), kritik konstruktif, dan kolaborasi riset keamanan siber.