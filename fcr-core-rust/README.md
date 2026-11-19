<div align="center">
<img src="https://capsule-render.vercel.app/api?type=waving&color=0:000000,50:001a00,100:00FF00&height=180&section=header&text=RUST%20CORE&fontSize=60&fontColor=00FF00&animation=fadeIn&fontAlignY=38&desc=Library%20Implementation%20%E2%80%94%20v0.6.0%20U2048%20Constant-Time&descAlignY=58&descSize=20" width="100%"/>
<img src="https://img.shields.io/badge/STATUS-RESEARCH%20PROTOTYPE%20v0.6.0-F59E0B?style=for-the-badge&labelColor=000000"/>
<img src="https://img.shields.io/badge/TESTS-16%20PASSED-00C853?style=for-the-badge&labelColor=000000"/>
<img src="https://img.shields.io/badge/CONSTANT%20TIME-VERIFIED-9370DB?style=for-the-badge&labelColor=000000"/>
</div>
# fcr-core-rust
Implementasi Rust dari FCR-RSA — library inti (v0.6.0).
⚠️ **RESEARCH PROTOTYPE — NOT PRODUCTION READY.** Lihat [../LIMITATIONS.md](../LIMITATIONS.md).
## Status
| Item | Status |
| --- | --- |
| Compile | ✅ |
| Integration test | ✅ **16 passed** |
| Example round-trip | ✅ |
| Konsistensi dengan Python | ✅ **10/10 ciphertext identik** |
| **Constant-time (Welch's t-test)** | ✅ **LOLOS (p = 0.143 > 0.05)** |
| **Cache side-channel sim** | ✅ **LOLOS (0% perbedaan)** |
| **Algebraic (Gröbner)** | ✅ **RESISTANT (~10^1850)** |
| Audit independen | ❌ Belum (Fase 4) |
## Arsitektur v0.6.0
- **`crypto-bigint::U2048` + Montgomery arithmetic** — operasi modular *constant-time*.
- API publik: `generate_key_pair`, `derive_blinding`, `encrypt_with_blinding_ct`, `decrypt_with_blinding_ct`, `bytes_to_u2048`, `u2048_to_bytes`, `FcrKeyPair`, `max_seed_bytes`.
## Build & Test
```bash
cargo build --release
cargo test
cargo clippy --all-targets -- -D warnings
```
## Contoh
```bash
cargo run --example basic
```
## Modul
```text
src/lib.rs      — entry point, re-export API publik
src/fib.rs      — Fibonacci non-linear + noise LWE + U2048 CT ops
src/gaussian.rs — pemetaan Gaussian integer
```
## Catatan
Library ini adalah fondasi. Untuk CLI/REST API, lihat [../fcr-engine](../fcr-engine/README.md). Untuk binding Python, lihat [../fcr-pyo3](../fcr-pyo3/README.md).
## Lisensi
Dual MIT / Apache-2.0. Lihat [../LICENSE](../LICENSE).
<div align="center" style="background: linear-gradient(135deg, #003300 0%, #001a00 50%, #000000 100%); padding: 25px 20px; border: 2px solid #00FF00; border-radius: 15px; margin-top: 30px; box-shadow: 0 0 25px rgba(0, 255, 0, 0.3);">
<p style="color: #00FF00; font-family: 'Courier New', monospace;"><strong>© 2026 — Emen (duhemen) | Made with 💚 and ☕ in Indonesia</strong></p>
</div>