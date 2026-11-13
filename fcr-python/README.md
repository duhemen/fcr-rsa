
<div align="center">

<img src="https://capsule-render.vercel.app/api?type=waving&color=0:000000,50:001a00,100:00FF00&height=180&section=header&text=PYTHON%20REF&fontSize=60&fontColor=00FF00&animation=fadeIn&fontAlignY=38&desc=Reference%20Implementation%20%E2%80%94%20Research%20Prototype&descAlignY=58&descSize=20" width="100%"/>

![Status](https://img.shields.io/badge/STATUS-RESEARCH%20PROTOTYPE-F59E0B?style=for-the-badge&labelColor=000000)
![Tests](https://img.shields.io/badge/TESTS-3%20PASSED-00C853?style=for-the-badge&labelColor=000000)
![Package](https://img.shields.io/badge/PACKAGE-fcr--rsa%20v0.1.0-3776AB?style=for-the-badge&labelColor=000000)

</div>

# fcr-python

Implementasi Python dari FCR-RSA — untuk referensi & pengujian.

> ⚠️ **RESEARCH PROTOTYPE — NOT PRODUCTION READY.**
> Implementasi Python **tidak** cocok untuk operasi kriptografi sensitif
> terhadap waktu. Gunakan Rust untuk eksperimen performa.
> Lihat [../LIMITATIONS.md](../LIMITATIONS.md).

## Instalasi (editable)

```bash
cd fcr-python
pip install -e .
```

## Contoh

```bash
python examples/basic.py
```

Output yang diharapkan:
```
Plaintext : 42
Ciphertext: 722957369
Decrypted : 42
Round-trip OK
```

## Test

```bash
pytest tests/
```

Yang diharapkan: `3 passed`.

## Modul

- `fcr_rsa/core_math.py`   — Fibonacci non-linear + noise LWE
- `fcr_rsa/tier_a.py`      — enkripsi/dekripsi Tier A
- `fcr_rsa/tier_b.py`      — enkripsi/dekripsi Tier B
- `fcr_rsa/auto_immune.py` — token spatiotemporal + deteksi anomali

## Catatan

Implementasi Python adalah **referensi**, bukan produksi. Perbedaan
performa dengan Rust signifikan (Python interpreter overhead). Untuk
benchmark, gunakan Rust.

## Lisensi

Dual MIT / Apache-2.0. Lihat [../LICENSE](../LICENSE).
---

<div align="center" style="background: linear-gradient(135deg, #003300 0%, #001a00 50%, #000000 100%); padding: 35px 20px; border: 2px solid #00FF00; border-radius: 15px; margin-top: 30px; box-shadow: 0 0 25px rgba(0, 255, 0, 0.3);">

<h3 style="color: #00FF00; font-family: 'Courier New', monospace; letter-spacing: 2px;">
  👨‍💻 INISIATOR & PENULIS UTAMA
</h3>

<p style="color: #FFFFFF; font-family: 'Courier New', monospace; font-size: 18px;">
  <strong>Emen (duhemen)</strong>
</p>

<p style="color: #00FF00; font-family: 'Courier New', monospace; font-size: 14px;">
  <code>Konseptor • Visioner • Independent Researcher</code>
</p>

<blockquote style="border-left: 3px solid #00FF00; padding-left: 15px; color: #CCCCCC; font-style: italic; max-width: 700px; margin: 0 auto;">
  "Kriptografi yang sempurna bukanlah yang paling kaku, melainkan yang
  meniru sifat alam semesta—kekacauan yang teratur."
</blockquote>

<p style="color: #00C853; font-family: 'Courier New', monospace; font-size: 13px;">
  🍀 Dikembangkan dengan asistensi AI (<strong>DeepSeek</strong> & <strong>Google Mode AI</strong>) sebagai kolaborator riset. 🍀
</p>

<img src="https://capsule-render.vercel.app/api?type=waving&color=0:00FF00,50:001a00,100:000000&height=120&section=footer" width="100%"/>

<p style="color: #00FF00; font-family: 'Courier New', monospace; font-size: 14px; margin-top: 10px;">
  <strong>© 2026 — Emen (duhemen) | Made with 💚 and ☕ in Indonesia</strong>
</p>

</div>