# FCR-RSA Engine CLI

Command Line Interface (CLI) untuk protokol FCR-RSA. 

> ⚠️ **RESEARCH PROTOTYPE — NOT PRODUCTION READY.**
> Jangan gunakan untuk data sensitif. Lihat [../LIMITATIONS.md](../LIMITATIONS.md).

## Fitur
- **Tier B**: Mode ringan untuk IoT/Mobile.
- **Tier A**: Mode berat untuk Server (dengan `N_comp`).
- **Random Blind**: Mode produksi (IND-CPA aman).
- **Deterministic**: Mode uji (dengan *blinding* manual).

## Cara Build
```bash
cd fcr-engine
cargo build --release
```

## Cara Penggunaan

### 1. Enkripsi (Mode Acak / Produksi)
```bash
cargo run -- encrypt --message 42
```
*(Hasil ciphertext akan berubah setiap kali dijalankan)*

### 2. Enkripsi & Dekripsi (Mode Deterministik Tier B)
```bash
# Enkripsi
cargo run -- encrypt --message 42 --blinding 12345

# Dekripsi (Gunakan ciphertext yang didapat)
cargo run -- decrypt --ciphertext 722957369 --blinding 12345
```

### 3. Enkripsi & Dekripsi (Mode Deterministik Tier A)
```bash
# Enkripsi
cargo run -- encrypt --message 42 --blinding 12345 --tier a

# Dekripsi
cargo run -- decrypt --ciphertext 758780528 --blinding 12345 --tier a
```

### 4. Enkripsi Teks Nyata (Fase 6)
```bash
# Enkripsi
cargo run -- encrypt --text "Halo Dunia" --blinding 12345
# Output: 96489521,574432988,...

# Dekripsi (Gunakan deretan angka dari output di atas)
cargo run -- decrypt --ciphertext 96489521,574432988,... --blinding 12345
# Output: Halo Dunia
```
*(Catatan: Karena kita menggunakan mode ECB per-byte, teks panjang akan menghasilkan deretan angka yang dipisahkan koma).*