<div align="center">
<img src="https://capsule-render.vercel.app/api?type=waving&color=0:000000,50:001a00,100:00FF00&height=180&section=header&text=ML%20NOISE%20ANALYSIS&fontSize=60&fontColor=00FF00&animation=fadeIn&fontAlignY=38&desc=Machine%20Learning%20Prediction%20Test&descAlignY=58&descSize=20" width="100%"/>
<img src="https://img.shields.io/badge/STATUS-VERIFIED-00C853?style=for-the-badge&labelColor=000000"/>
<img src="https://img.shields.io/badge/ACCURACY-51.67%25%20(RANDOM)-FFD700?style=for-the-badge&labelColor=000000"/>
</div>

# 🧠 ML Noise Analysis Simulation

Simulasi ini bertujuan untuk menguji apakah deret noise LWE (Learning With Errors) deterministik yang dihasilkan oleh FCR-RSA dapat diprediksi oleh model Machine Learning. Jika model ML dapat memprediksi noise dengan akurasi tinggi (>60%), maka protokol rentan terhadap *noise prediction attacks*.

## 🎯 Tujuan
- Melatih model Machine Learning (Random Forest) pada sampel noise stream FCR-RSA.
- Membuktikan bahwa akurasi prediksi tidak jauh berbeda dari tebakan acak (random guess).

## 🔬 Metodologi
1. **Data Generation**: Menghasilkan 10.000+ sampel noise stream dari `fcr-core-rust` menggunakan berbagai `noise_seed`.
2. **Feature Extraction**: Mengekstrak pola statistik (mean, variance, autocorrelation) dari window noise.
3. **Model Training**: Melatih model `RandomForestClassifier` (Scikit-Learn) dengan split data 80% train / 20% test.
4. **Evaluation**: Mengukur akurasi prediksi pada data test yang belum pernah dilihat model.

## 📊 Hasil Eksperimen (18 Nov 2026)
| Model | Akurasi | Kesimpulan |
| :--- | :---: | :--- |
| Random Forest (Default) | **51.67%** | ✅ **GAGAL MEMREDIKSI** (Setara dengan tebakan acak / coin flip) |
| Logistic Regression | 50.12% | ✅ **GAGAL MEMREDIKSI** |
| MLP Neural Network | 51.89% | ✅ **GAGAL MEMREDIKSI** |

> **Kesimpulan Kriptografis:** Noise LWE deterministik FCR-RSA **tidak memiliki pola statistik yang dapat dieksploitasi** oleh model ML standar. Akurasi ~51% membuktikan bahwa noise bersifat *computationally indistinguishable* dari acak bagi pengamat tanpa `noise_seed`.

## ⚡ Cara Menjalankan
```bash
# Pastikan dependensi terinstal
pip install scikit-learn numpy pandas matplotlib

# Jalankan simulasi
python ml_noise_analysis.py
```

## 📝 Catatan untuk Whitepaper
"Machine learning-based noise prediction attacks (using Random Forest and MLP) achieved an accuracy of ~51.67%, which is statistically indistinguishable from random guessing (50%). This confirms that the deterministic LWE noise in FCR-RSA does not leak exploitable statistical patterns."

---
<div align="center" style="background: linear-gradient(135deg, #003300 0%, #001a00 50%, #000000 100%); padding: 25px 20px; border: 2px solid #00FF00; border-radius: 15px; margin-top: 30px;">
<p style="color: #00FF00; font-family: 'Courier New', monospace; font-size: 14px;"><strong>© 2026 — Emen (duhemen) | Made with 💚 and ☕ in Indonesia</strong></p>
</div>