pub mod fib;
pub mod gaussian;
pub mod encrypt;
pub mod decrypt;
pub mod red_team;

// Ekspor fungsi lama (jika masih dipakai, tapi akan deprecated di masa depan)
// pub use encrypt::{encrypt, encrypt_with_blinding}; // <-- Hapus atau comment jika tidak dipakai

// 🆕 Ekspor fungsi baru v0.6.0 (API Utama)
pub use fib::{
    generate_key_pair,
    derive_blinding,
    encrypt_with_blinding_ct,
    decrypt_with_blinding_ct,
    FcrKeyPair,
    Q_2048,
    max_seed_bytes,
    bytes_to_u2048,
    u2048_to_bytes,
    fibonacci_nonlinear_with_lwe,
    gaussian_map,
};

pub use encrypt::{encrypt, encrypt_u2048};
pub use decrypt::{decrypt, decrypt_u2048};