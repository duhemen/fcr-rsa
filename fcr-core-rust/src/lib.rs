//! FCR-RSA: Post-quantum cryptographic protocol prototype.
//!
//! ⚠️ RESEARCH PROTOTYPE — NOT PRODUCTION READY.
//! See LIMITATIONS.md before use.
//!
//! # Modules
//! - [`fib`]      : Non-linear Fibonacci sequence
//! - [`gaussian`] : Gaussian integer mapping
//! - [`encrypt`]  : Encryption (with IND-CPA patch)
//! - [`decrypt`]  : Decryption
//! - [`red_team`] : Internal red team simulations

pub mod fib;
pub mod gaussian;
pub mod encrypt;
pub mod decrypt;
pub mod red_team;

pub use encrypt::{encrypt, encrypt_tier_a};
pub use decrypt::{decrypt, decrypt_tier_a};