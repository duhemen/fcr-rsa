# Contributing to FCR-RSA

Terima kasih telah tertarik berkontribusi. FCR-RSA adalah **riset terbuka**
dan kami menyambut kritik, uji coba, dan perbaikan.

## Yang Paling Kami Butuhkan

1. **Uji klaim keamanan.** Coba tembus FCR-RSA. Jika berhasil, laporkan
   via [SECURITY.md](SECURITY.md). Jika gagal, dokumentasikan usahamu—
   kegagalan yang terdokumentasi sama berharganya dengan keberhasilan.
2. **Audit implementasi.** Cari bug, side-channel, kesalahan logika.
3. **Test vector.** Tambah kasus uji baru, verifikasi yang sudah ada.
4. **Dokumentasi.** Perbaiki yang kurang jelas, terjemahkan, tambah contoh.

## Yang TIDAK Kami Butuhkan (saat ini)

- Klaim "aman" tanpa bukti
- Benchmark tanpa metodologi
- Fitur baru sebelum yang ada teruji
- Perubahan API besar sebelum v1.0

## Alur Kontribusi

1. Buka Issue untuk diskusi (kecuali perbaikan typo)
2. Fork repo, buat branch dari `main`
3. Tulis kode + test
4. Pastikan `cargo test` dan `pytest` lulus
5. Pastikan `cargo clippy -- -D warnings` bersih
6. Kirim Pull Request dengan deskripsi jelas

## Standar Kode

### Rust
- `rustfmt` (default config)
- `cargo clippy -- -D warnings`
- Test untuk setiap fungsi publik
- Dokumentasi `///` untuk API publik

### Python
- `black` (line length 100)
- `ruff` untuk linting
- `pytest` untuk test
- Type hints untuk fungsi publik

## Commit Message

Format: `<tipe>: <deskripsi singkat>`

Tipe: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`, `security`

Contoh:
```
feat: tambah test vector untuk Tier A N=100
fix: perbaiki overflow pada fib non-linear N>500
docs: jelaskan format test vector di docs/test-vectors.md
```

## Review Process

- Setiap PR direview oleh minimal 1 maintainer
- PR yang mengubah logika kriptografi butuh 2 reviewer
- PR yang mengubah klaim keamanan butuh diskusi Issue dulu

## Lisensi Kontribusi

Dengan mengirim PR, Anda setuju kontribusi Anda dilisensikan di bawah
dual MIT/Apache-2.0, sama seperti proyek ini.

## Pertanyaan?

Buka Issue dengan label `question`.

