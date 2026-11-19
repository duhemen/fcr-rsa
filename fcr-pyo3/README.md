<div align="center">
<img src="https://capsule-render.vercel.app/api?type=waving&color=0:000000,50:001a00,100:00FF00&height=180&section=header&text=PYO3%20BRIDGE&fontSize=60&fontColor=00FF00&animation=fadeIn&fontAlignY=38&desc=Rust%20%E2%86%94%20Python%20Binding%20(bytes-to-bytes)&descAlignY=58&descSize=20" width="100%"/>
<img src="https://img.shields.io/badge/STATUS-RESEARCH%20PROTOTYPE-F59E0B?style=for-the-badge&labelColor=000000"/>
<img src="https://img.shields.io/badge/BUILD-MATURIN-00C853?style=for-the-badge&labelColor=000000"/>
<img src="https://img.shields.io/badge/CONSTANT%20TIME-INHERITED%20from%20core-9370DB?style=for-the-badge&labelColor=000000"/>
</div>
# fcr-pyo3
Binding Rust ↔ Python untuk FCR-RSA menggunakan **PyO3 + maturin**. Memungkinkan Python memanggil logika inti Rust (U2048 constant-time) dengan performa native.
⚠️ **RESEARCH PROTOTYPE.** Lihat [../LIMITATIONS.md](../LIMITATIONS.md).
## Status
| Item | Status |
| --- | --- |
| Compile | ✅ |
| `maturin develop --release` | ✅ |
| Ciphertext konsisten Rust ↔ Python | ✅ **10/10 match** |
| API `bytes-to-bytes` | ✅ (menyembunyikan U2048 dari Python) |
| Constant-time | ✅ **Inherited dari core v0.6.0** |
## Build
```bash
pip install maturin
maturin develop --release
python test_pyo3.py
```
## Contoh
```python
import fcr_pyo3
ct = fcr_pyo3.encrypt(b"Halo Dunia")   # bytes-to-bytes
pt = fcr_pyo3.decrypt(ct)              # b"Halo Dunia"
```
## Catatan
Binding ini memanggil `fcr-core-rust` v0.6.0. Untuk performa produksi, gunakan Rust langsung (hindari overhead Python). Lihat [../fcr-core-rust/README.md](../fcr-core-rust/README.md).
## Lisensi
Dual MIT / Apache-2.0. Lihat [../LICENSE](../LICENSE).
<div align="center" style="background: linear-gradient(135deg, #003300 0%, #001a00 50%, #000000 100%); padding: 25px 20px; border: 2px solid #00FF00; border-radius: 15px; margin-top: 30px; box-shadow: 0 0 25px rgba(0, 255, 0, 0.3);">
<p style="color: #00FF00; font-family: 'Courier New', monospace;"><strong>© 2026 — Emen (duhemen) | Made with 💚 and ☕ in Indonesia</strong></p>
</div>