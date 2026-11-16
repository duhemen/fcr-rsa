<div align="center">

<img src="https://capsule-render.vercel.app/api?type=waving&color=0:000000,50:001a00,100:00FF00&height=180&section=header&text=SIMULATIONS%20REF&fontSize=60&fontColor=00FF00&animation=fadeIn&fontAlignY=38&desc=Reference%20Implementation%20%E2%80%94%20Research%20Prototype&descAlignY=58&descSize=20" width="100%"/>

![Status](https://img.shields.io/badge/STATUS-INTERNAL%20TESTING-F59E0B?style=for-the-badge&labelColor=000000)
![Type](https://img.shields.io/badge/TYPE-SELF--TEST%20(Not%20Audit)-EF4444?style=for-the-badge&labelColor=000000)

</div>

# Simulasi & Pengujian

Folder ini berisi skrip untuk mensimulasikan serangan (Red Team) dan mengukur performa (Benchmark) dari protokol FCR-RSA.

> ⚠️ **PENTING:** Semua simulasi di sini adalah **uji internal (self-test)** yang dilakukan oleh pengembang. Ini **BUKAN** audit keamanan independen. Klaim ketahanan terhadap serangan belum divalidasi oleh pihak ketiga.

## 📂 Daftar Simulasi

| File | Deskripsi | Tipe | Platform |
| :--- | :--- | :--- | :--- |
| `run_test.py` | Uji dasar enkripsi dan dekripsi | Defensif | Semua |
| `bridge_protocol.py` | Simulasi *handshake* Tier A & Tier B | Defensif | Semua |
| `stress_test.py` | Uji beban tinggi dan konsistensi *BigInt* | Performa | Semua |
| `red_team_lll.py` | Serangan Lattice Reduction (LLL) dasar | Ofensif | Semua |
| `red_team_noise_attack.py` | Serangan brute-force terhadap `noise_seed` | Ofensif | Semua |
| `ordal_attack_simulation.py` | Skenario Pegasus/LST & respons Auto-Imun | Ofensif | Semua |
| `visualize_attack.py` | Visualisasi sebaran *noise* & kegagalan LLL | Analisis | Semua |
| `bkz/bkz_attack.py` | **BKZ 2.0** lattice attack | Ofensif | **Linux/Colab** ⚠️ |

## ⚠️ Catatan: BKZ 2.0 Hanya Bisa Jalan di Linux/Colab

`fpylll` (library untuk LLL/BKZ) **tidak tersedia untuk Windows** (tidak ada wheel resmi; kompilasi butuh GMP). Karena itu, **BKZ 2.0 dijalankan di Google Colab** (Linux), bukan di Windows lokal.

Lihat [`bkz/README.md`](bkz/README.md) untuk instruksi lengkap.

## 🖼️ Aset Visual

Semua grafik dan hasil visualisasi benchmark disimpan di dalam folder `assets/`.

## ⚡ Cara Menjalankan

Pastikan dependensi Python sudah terinstal (terutama `numpy`, `matplotlib`, dan `scipy`):

```bash
pip install -r requirements.txt
```

## 🐧 Menjalankan BKZ di Colab (Linux)

1. Buka [colab.research.google.com](https://colab.research.google.com)
2. Upload `simulations/bkz/bkz_attack.ipynb` (atau *copy* isi `bkz_attack.py` ke sel baru)
3. Jalankan: `!pip install fpylll` (sel pertama)
4. Runtime → Run all
5. Lihat output: harusnya berakhir dengan `[KESIMPULAN] ❌ BKZ 2.0 GAGAL menemukan pesan asli (M)`

## 📊 Hasil yang Diharapkan

- **Uji Defensif:** Menghasilkan pesan dekripsi yang sama persis dengan pesan asli (*Round-trip OK*).
- **Uji Ofensif (Red Team):** Algoritma LLL, Noise Attack, dan BKZ 2.0 diharapkan **gagal** menemukan kunci.
- **Visualisasi:** Menghasilkan plot grafik yang menunjukkan kekacauan (*chaos*) deret Fibonacci non-linear.

## 📋 Status Jujur per-File

| File | Bisa Dijalankan? | Bukti Output |
| :--- | :--- | :--- |
| `run_test.py` | ✅ | Round-trip OK |
| `bridge_protocol.py` | ✅ | Handshake OK |
| `stress_test.py` | ✅ | BigInt konsisten |
| `red_team_lll.py` | ✅ | LLL gagal menembus |
| `red_team_noise_attack.py` | ✅ | Noise attack gagal |
| `ordal_attack_simulation.py` | ✅ | LOCKDOWN terpicu |
| `visualize_attack.py` | ✅ | Grafik chaos |
| `bkz/bkz_attack.py` | ✅ (Linux/Colab) | BKZ gagal menembus |

---

<div align="center" style="background: linear-gradient(135deg, #003300 0%, #001a00 50%, #000000 100%); padding: 25px 20px; border: 2px solid #00FF00; border-radius: 15px; margin-top: 30px; box-shadow: 0 0 25px rgba(0, 255, 0, 0.3);">

<p style="color: #00FF00; font-family: 'Courier New', monospace; font-size: 14px; margin-top: 10px;">
  <strong>© 2026 — Emen (duhemen) | Made with 💚 and ☕ in Indonesia</strong>
</p>

</div>