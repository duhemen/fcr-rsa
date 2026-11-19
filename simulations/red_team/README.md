<div align="center">
<img src="https://capsule-render.vercel.app/api?type=waving&color=0:000000,50:001a00,100:00FF00&height=180&section=header&text=Red%20Team-Simulations&fontSize=60&fontColor=00FF00&animation=fadeIn&fontAlignY=38&desc=Plaintext%20Attack%20Adversarial%20Attack&descAlignY=58&descSize=20" width="100%"/>
<img src="https://img.shields.io/badge/STATUS-VULNERABILITY%20CLOSED-00C853?style=for-the-badge&labelColor=000000"/>
</div>

# 🚨 Red Team Attack Simulations

Folder ini berisi skrip simulasi serangan ofensif untuk menguji ketahanan FCR-RSA, khususnya fokus pada temuan kritis di versi lama dan verifikasinya di versi terbaru.

## 📂 Daftar Simulasi
| File | Deskripsi | Status |
| :--- | :--- | :---: |
| `red_team_key_recovery.py` | Simulasi Known-Plaintext Attack pada v0.4.1 | 🔴 **BERHASIL** (Menemukan celah) |
| `red_team_adversarial_attack.py` | Simulasi penyerang eksternal pada v0.5.1+ (Hybrid KEM) | ✅ **GAGAL** (Celah tertutup) |

## 🔬 1. Known-Plaintext Attack (v0.4.1)
**Masalah:** Enkripsi v0.4.1 bersifat linear: `C = (M × E_vec) + R_blind mod Q`. Karena `R_blind` dikirim sebagai prefix plaintext, penyerang yang mengetahui 1 blok plaintext (`M`) dapat memulihkan `E_vec` secara langsung:
```python
E_vec = ((C_known - R_blind) * modInverse(M_known, Q)) % Q
```
**Hasil:** Skrip `red_team_key_recovery.py` berhasil memulihkan kunci dan mendekripsi seluruh pesan rahasia. **TOTAL BREAK.**

## 🛡️ 2. Adversarial Attack Verification (v0.5.1+)
**Mitigasi:** FCR-RSA beralih ke arsitektur **Hybrid KEM + AEAD**. FCR-RSA hanya membungkus *session seed* acak, dan data dienkripsi menggunakan ChaCha20-Poly1305 dengan kunci yang diturunkan via HKDF.

**Hasil:** Skrip `red_team_adversarial_attack.py` mensimulasikan penyerang yang menyadap ciphertext. Tanpa akses ke database atau `noise_seed`, penyerang **TIDAK DAPAT**:
1. Memulihkan session seed (terenkripsi U2048 constant-time).
2. Menurunkan kunci sesi ChaCha20 (memerlukan seed).
3. Memalsukan ciphertext (AEAD Poly1305 tag akan ditolak).

> **Kesimpulan Kriptografis:** Migrasi ke Hybrid KEM + AEAD berhasil menutup celah known-plaintext attack secara total. Serangan adversarial gagal 100%.

## ⚡ Cara Menjalankan
```bash
# Uji serangan v0.4.1 (Hanya untuk tujuan edukasi/sejarah)
python red_team_key_recovery.py

# Uji verifikasi penutupan celah v0.5.1+
python red_team_adversarial_attack.py
```

## 📝 Catatan untuk Whitepaper
"A known-plaintext attack successfully recovered the encryption vector in v0.4.1 due to linear operations with exposed blinding. This critical vulnerability was entirely mitigated in v0.5.1 by adopting a Hybrid KEM + AEAD architecture, rendering adversarial ciphertext-only and known-plaintext attacks computationally infeasible."

---
<div align="center" style="background: linear-gradient(135deg, #003300 0%, #001a00 50%, #000000 100%); padding: 25px 20px; border: 2px solid #00FF00; border-radius: 15px; margin-top: 30px;">
<p style="color: #00FF00; font-family: 'Courier New', monospace; font-size: 14px;"><strong>© 2026 — Emen (duhemen) | Made with 💚 and ☕ in Indonesia</strong></p>
</div>