//! FCR-TP (FCR Transport Protocol)
//! 
//! Protokol lapisan transport yang aman, adaptif, dan ramah pengguna
//! untuk ekosistem FCR-RSA. Dirancang untuk tahan terhadap analisis lalu lintas
//! (Traffic Analysis) di era superposisi dan ASI.

pub mod packet;
pub mod hopper;
pub mod crypto;
pub mod defense;
pub mod survival;

// Ekspor tipe utama agar mudah diakses dari luar crate
pub use packet::FcrTpFrame;
pub use packet::HeaderMode;