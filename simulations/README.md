<div align="center">

<img src="https://capsule-render.vercel.app/api?type=waving&color=0:000000,50:001a00,100:00FF00&height=180&section=header&text=SIMULATIONS%20REF&fontSize=60&fontColor=00FF00&animation=fadeIn&fontAlignY=38&desc=Reference%20Implementation%20%E2%80%94%20Research%20Prototype&descAlignY=58&descSize=20" width="100%"/>

![Status](https://img.shields.io/badge/STATUS-INTERNAL%20TESTING-F59E0B?style=for-the-badge&labelColor=000000)
![Type](https://img.shields.io/badge/TYPE-SELF--TEST%20(Not%20Audit)-EF4444?style=for-the-badge&labelColor=000000)

</div>

# Simulasi & Pengujian

Folder ini berisi skrip untuk mensimulasikan serangan (Red Team) dan mengukur performa (Benchmark) dari protokol FCR-RSA.

> ⚠️ **PENTING:** Semua simulasi di sini adalah **uji internal (self-test)** yang dilakukan oleh pengembang. Ini **BUKAN** audit keamanan independen. Klaim ketahanan terhadap serangan belum divalidasi oleh pihak ketiga.

## 📂 Daftar Simulasi

| File | Deskripsi | Tipe |
| :--- | :--- | :--- |
| `run_test.py` | Uji dasar enkripsi dan dekripsi | Defensif |
| `bridge_protocol.py` | Simulasi komunikasi dan *handshake* antara Tier A & Tier B | Defensif |
| `stress_test.py` | Pengujian beban tinggi dan konsistensi *BigInt* | Performa |
| `red_team_lll.py` | Simulasi serangan Lattice Reduction (LLL) dasar | Ofensif |
| `red_team_noise_attack.py` | Simulasi serangan brute-force terhadap `noise_seed` | Ofensif |
| `ordal_attack_simulation.py` | Simulasi skenario pencurian kunci (Pegasus/LST) & respons Auto-Imun | Ofensif |
| `visualize_attack.py` | Visualisasi sebaran *noise* dan kegagalan LLL | Analisis |

## 🖼️ Aset Visual

Semua grafik dan hasil visualisasi benchmark disimpan di dalam folder `assets/`. 
*(Contoh: `hasil_benchmark_fcr_rsa.png`, `hasil_visualisasi_lll.png`)*

## ⚡ Cara Menjalankan

Pastikan dependensi Python sudah terinstal (terutama `numpy`, `matplotlib`, dan `scipy`):

```bash
pip install -r requirements.txt
```

Jalankan skrip simulasi dari direktori *root* repositori (bukan dari dalam folder `simulations/`):

```bash
# Contoh menjalankan uji LLL
python simulations/red_team_lll.py

# Contoh menjalankan uji fungsionalitas dasar
python simulations/run_test.py
```

## 📊 Hasil yang Diharapkan

- **Uji Defensif:** Menghasilkan pesan dekripsi yang sama persis dengan pesan asli (*Round-trip OK*).
- **Uji Ofensif (Red Team):** Algoritma LLL dan Noise Attack diharapkan **gagal** menemukan kunci, membuktikan bahwa distorsi *noise* LWE bekerja pada parameter uji saat ini.
- **Visualisasi:** Menghasilkan plot grafik yang menunjukkan kekacauan (*chaos*) dari deret Fibonacci non-linear.

---

<div align="center" style="background: linear-gradient(135deg, #003300 0%, #001a00 50%, #000000 100%); padding: 25px 20px; border: 2px solid #00FF00; border-radius: 15px; margin-top: 30px; box-shadow: 0 0 25px rgba(0, 255, 0, 0.3);">

<p style="color: #00FF00; font-family: 'Courier New', monospace; font-size: 14px; margin-top: 10px;">
  <strong>© 2026 — Emen (duhemen) | Made with 💚 and ☕ in Indonesia</strong>
</p>

</div>