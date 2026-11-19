<div align="center">
<img src="https://capsule-render.vercel.app/api?type=waving&color=0:000000,50:001a00,100:00FF00&height=180&section=header&text=PYTHON%20REF&fontSize=60&fontColor=00FF00&animation=fadeIn&fontAlignY=38&desc=Reference%20Implementation%20%E2%80%94%20Research%20Prototype&descAlignY=58&descSize=20" width="100%"/>
<img src="https://img.shields.io/badge/STATUS-RESEARCH%20PROTOTYPE-F59E0B?style=for-the-badge&labelColor=000000"/>
<img src="https://img.shields.io/badge/TESTS-3%20PASSED-00C853?style=for-the-badge&labelColor=000000"/>
<img src="https://img.shields.io/badge/CROSS%20LANGUAGE-10%2F10%20MATCH-20B2AA?style=for-the-badge&labelColor=000000"/>
</div>
# fcr-python
Implementasi Python dari FCR-RSA — untuk **referensi & oracle cross-language**.
⚠️ **RESEARCH PROTOTYPE.** Tidak cocok untuk operasi sensitif waktu (timing). Gunakan Rust untuk produksi. Lihat [../LIMITATIONS.md](../LIMITATIONS.md).
## Status
| Item | Status |
| --- | --- |
| Paket terpasang | ✅ |
| Unit test | ✅ 3 passed |
| Konsistensi dengan Rust | ✅ **10/10 ciphertext identik** |
| Peran | 🧪 Oracle test-vector & referensi akademik |
## Instalasi & Test
```bash
cd fcr-python && pip install -e .
python examples/basic.py
pytest tests/   # 3 passed
```
## Modul
```text
fcr_rsa/core_math.py   — Fibonacci non-linear + noise LWE
fcr_rsa/tier_a.py      — Tier A
fcr_rsa/tier_b.py      — Tier B
fcr_rsa/auto_immune.py — token spatiotemporal + anomali
```
## Catatan
Implementasi Python = referensi. Untuk binding native, lihat [../fcr-pyo3](../fcr-pyo3/README.md).
## Lisensi
Dual MIT / Apache-2.0. Lihat [../LICENSE](../LICENSE).
<div align="center" style="background: linear-gradient(135deg, #003300 0%, #001a00 50%, #000000 100%); padding: 25px 20px; border: 2px solid #00FF00; border-radius: 15px; margin-top: 30px; box-shadow: 0 0 25px rgba(0, 255, 0, 0.3);">
<p style="color: #00FF00; font-family: 'Courier New', monospace;"><strong>© 2026 — Emen (duhemen) | Made with 💚 and ☕ in Indonesia</strong></p>
</div>