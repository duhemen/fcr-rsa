<div align="center">

<img src="https://capsule-render.vercel.app/api?type=waving&color=0:000000,50:001a00,100:00FF00&height=180&section=header&text=PYO3%20BRIDGE&fontSize=60&fontColor=00FF00&animation=fadeIn&fontAlignY=38&desc=Rust%20%E2%86%94%20Python%20Binding&descAlignY=58&descSize=20" width="100%"/>

![Status](https://img.shields.io/badge/STATUS-RESEARCH%20PROTOTYPE-F59E0B?style=for-the-badge&labelColor=000000)
![Build](https://img.shields.io/badge/BUILD-MATURIN-00C853?style=for-the-badge&labelColor=000000)

</div>

# fcr-pyo3

Binding Rust ↔ Python untuk FCR-RSA menggunakan **PyO3** + **maturin**.

Memungkinkan Python memanggil logika inti Rust untuk performa native.

> ⚠️ **RESEARCH PROTOTYPE — NOT PRODUCTION READY.**
> Lihat [../LIMITATIONS.md](../LIMITATIONS.md).

## Status

| Item | Status |
| :--- | :--- |
| Compile | ✅ |
| `maturin develop` | ✅ |
| Ciphertext konsisten dengan Rust & Python murni | ✅ |
| Constant-time | ❌ Belum |

## Build

```bash
pip install maturin
maturin develop --release
```

## Contoh

```python
import fcr_pyo3

# Tier B
c = fcr_pyo3.encrypt(42)
print(c)  # 722957369

# Tier A
c_a = fcr_pyo3.encrypt_tier_a(42)
print(c_a)  # 758780528

# Dekripsi
m = fcr_pyo3.decrypt(c)
print(m)  # 42
```

Atau jalankan:

```bash
python py.py
```

## Catatan

- Binding ini memanggil `fcr-core-rust` (library inti).
- Untuk performa produksi, gunakan Rust langsung (hindari *overhead* Python).
- Lihat [../fcr-core-rust/README.md](../fcr-core-rust/README.md) untuk API inti.

## Lisensi

Dual MIT / Apache-2.0. Lihat [../LICENSE](../LICENSE).

<div align="center" style="background: linear-gradient(135deg, #003300 0%, #001a00 50%, #000000 100%); padding: 25px 20px; border: 2px solid #00FF00; border-radius: 15px; margin-top: 30px; box-shadow: 0 0 25px rgba(0, 255, 0, 0.3);">

<p style="color: #00FF00; font-family: 'Courier New', monospace; font-size: 14px; margin-top: 10px;">
  <strong>© 2026 — Emen (duhemen) | Made with 💚 and ☕ in Indonesia</strong>
</p>

</div>