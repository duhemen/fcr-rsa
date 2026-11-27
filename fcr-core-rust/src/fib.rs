//! FCR Core - Mathematical Foundation (v0.6.1-hardened)
//! Upgrade: Q 2048-bit dengan constant-time operations via Montgomery Arithmetic (crypto-bigint v0.7.11)

use crypto_bigint::modular::{FixedMontyForm, FixedMontyParams};
use crypto_bigint::{NonZero, Odd, U2048};
use hkdf::Hkdf;
use sha2::Sha256;

pub const Q_2048: U2048 = U2048::from_be_hex(
    "FFFFFFFFFFFFFFFFC90FDAA22168C234C4C6628B80DC1CD129024E088A67CC74\
     020BBEA63B139B22514A08798E3404DDEF9519B3CD3A431B302B0A6DF25F1437\
     4FE1356D6D51C245E485B576625E7EC6F44C42E9A637ED6B0BFF5CB6F406B7ED\
     EE386BFB5A899FA5AE9F24117C4B1FE649286651ECE45B3DC2007CB8A163BF05\
     98DA48361C55D39A69163FA8FD24CF5F83655D23DCA3AD961C62F356208552BB\
     9ED529077096966D670C354E4ABC9804F1746C08CA18217C32905E462E36CE3B\
     E39E772C180E86039B2783A2EC07A28FB5C55DF06F4C52C9DE2BCBF695581718\
     3995497CEA956AE515D2261898FA051015728E5A8AACAA68FFFFFFFFFFFFFFFF",
);

fn q_nz() -> NonZero<U2048> {
    NonZero::new(Q_2048).unwrap()
}

fn q_odd() -> Odd<U2048> {
    Odd::new(Q_2048).unwrap()
}

/// Masking constant-time: mengubah bit kontrol (0 atau 1) menjadi mask 0x0 atau 0xFFFFFFFFFFFFFFFF.
/// Memakai `wrapping_sub` dan `core::hint::black_box` agar kompiler tidak mengubahnya menjadi percabangan (branch).
#[inline(always)]
pub fn ct_mask(choice: u64) -> u64 {
    let bit = core::hint::black_box(choice & 1);
    0u64.wrapping_sub(bit)
}

/// Pemilihan constant-time per-limb tanpa percabangan (branchless).
/// Identik secara matematis dengan definisi `ctSelectLimb` di Lean 4 (`FcrConstantTime.lean`).
#[inline(always)]
pub fn ct_select_limb(choice: u64, a: u64, b: u64) -> u64 {
    let mask = ct_mask(choice);
    b ^ (mask & (a ^ b))
}

pub struct FcrKeyPair {
    pub e_vec: U2048,
    pub e_vec_inv: U2048,
}

pub fn max_seed_bytes() -> usize {
    255
}

pub fn generate_key_pair(master_seed: &[u8]) -> Result<FcrKeyPair, &'static str> {
    let hk = Hkdf::<Sha256>::new(None, master_seed);
    let mut e_vec_bytes = [0u8; 256];
    hk.expand(b"fcr-rsa-e-vec-v1", &mut e_vec_bytes)
        .map_err(|_| "HKDF expand failed")?;

    let e_vec = U2048::from_be_slice(&e_vec_bytes).rem(&q_nz());

    if bool::from(e_vec.is_zero()) {
        return Err("Generated E_vec is zero");
    }

    let inv_opt = e_vec.invert_mod(&q_nz());
    if !bool::from(inv_opt.is_some()) {
        return Err("Modular inverse does not exist");
    }
    let e_vec_inv = inv_opt.unwrap();

    Ok(FcrKeyPair { e_vec, e_vec_inv })
}

/// Ekspor kunci PUBLIK saja: hanya e_vec, tanpa e_vec_inv, tanpa master_seed.
/// Ini yang boleh dikirim ke pengirim / disimpan di tempat publik.
/// Everlasting: format v1, besok v2 tinggal tambah prefix versi.
pub fn export_public_key(pair: &FcrKeyPair) -> String {
    // Hex manual, tanpa crate baru. Inti tetap ringan.
    let bytes = pair.e_vec.to_be_bytes();
    let mut s = String::with_capacity(512);
    for b in bytes.iter() {
        s.push_str(&format!("{:02x}", b));
    }
    format!("fcr-pub-v1:{}", s)
}

/// Bangun KeyPair untuk DEKRIPSI dari privat (master_seed) + publik (e_vec).
/// Penerima pakai ini: dia pegang master_seed (privat), dan e_vec (publik)
/// yang sudah dia ekspor. Dia tidak perlu terima e_vec_inv dari siapa pun,
/// karena e_vec_inv diturunkan lagi dari master_seed-nya sendiri.
/// Ini yang menjaga relate: publik dan privat tetap dari seed yang sama.
pub fn keypair_from_private(master_seed: &[u8]) -> Result<FcrKeyPair, &'static str> {
    generate_key_pair(master_seed)
}

/// Bangun KeyPair untuk ENKRIPSI dari publik saja.
/// Pengirim pakai ini: dia hanya pegang e_vec, tidak pegang master_seed,
/// tidak pegang e_vec_inv yang asli. e_vec_inv di sini diisi dummy,
/// TIDAK BOLEH dipakai untuk dekripsi. Kalau ada yang pakai KeyPair ini
/// untuk dekripsi, dia akan gagal, dan itu yang kita mau.
/// Tantangan yang masih ada dan harus kita hadapi jujur:
/// dengan q yang publik, e_vec_inv bisa dihitung dari e_vec via invert_mod.
/// Jadi split ini adalah split OPERASIONAL, bukan split kriptografis penuh.
/// Split penuh butuh q privat / trapdoor baru di inti, itu Fase 2c Langkah 2.
pub fn keypair_from_public(pub_str: &str) -> Result<FcrKeyPair, &'static str> {
    let hex_str = pub_str.strip_prefix("fcr-pub-v1:").ok_or("format kunci publik tidak sah")?;
    if hex_str.len() != 512 {
        return Err("panjang kunci publik tidak sah");
    }
    let mut bytes = [0u8; 256];
    let hex_bytes = hex_str.as_bytes();
    for i in 0..256 {
        let hi = hex_val(hex_bytes[i * 2]).ok_or("hex kunci publik tidak sah")?;
        let lo = hex_val(hex_bytes[i * 2 + 1]).ok_or("hex kunci publik tidak sah")?;
        bytes[i] = (hi << 4) | lo;
    }
    let e_vec = U2048::from_be_slice(&bytes);
    // Dummy, tidak boleh dipakai dekripsi.
    Ok(FcrKeyPair { e_vec, e_vec_inv: U2048::ONE })
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

pub fn fibonacci_nonlinear_with_lwe(n: u64, seed: &U2048, q: &U2048) -> U2048 {
    let mut a = U2048::ONE;
    let mut b = U2048::ONE;
    let q_nonzero = NonZero::new(*q).unwrap();

    for _ in 2..n {
        let next = a.add_mod(&b, &q_nonzero);
        a = b;
        b = next;
    }

    b.add_mod(seed, &q_nonzero)
}

pub fn gaussian_map(value: &U2048, q: &U2048) -> U2048 {
    let odd_q = Option::from(Odd::new(*q)).expect("Modulus must be odd");
    let params = FixedMontyParams::new(odd_q);
    let val_res = FixedMontyForm::new(value, &params);
    val_res.square().retrieve()
}

pub fn derive_blinding(master_seed: &[u8], nonce: &[u8]) -> Result<U2048, &'static str> {
    let mut ikm = Vec::with_capacity(master_seed.len() + nonce.len());
    ikm.extend_from_slice(master_seed);
    ikm.extend_from_slice(nonce);

    let hk = Hkdf::<Sha256>::new(None, &ikm);
    let mut blinding_bytes = [0u8; 256];
    hk.expand(b"fcr-rsa-blinding-v1", &mut blinding_bytes)
        .map_err(|_| "HKDF expand failed")?;

    let blinding = U2048::from_be_slice(&blinding_bytes);
    Ok(blinding.rem(&q_nz()))
}

pub fn encrypt_with_blinding_ct(message: &U2048, e_vec: &U2048, blinding: &U2048) -> U2048 {
    let params = FixedMontyParams::new(q_odd());
    let msg_res = FixedMontyForm::new(message, &params);
    let e_res = FixedMontyForm::new(e_vec, &params);
    let prod = msg_res.mul(&e_res).retrieve();

    prod.add_mod(blinding, &q_nz())
}

pub fn decrypt_with_blinding_ct(ciphertext: &U2048, e_vec_inv: &U2048, blinding: &U2048) -> U2048 {
    let q_nonzero = q_nz();
    let unblinded = ciphertext.sub_mod(blinding, &q_nonzero);

    let params = FixedMontyParams::new(q_odd());
    let unblinded_res = FixedMontyForm::new(&unblinded, &params);
    let inv_res = FixedMontyForm::new(e_vec_inv, &params);
    unblinded_res.mul(&inv_res).retrieve()
}

pub fn decrypt_u2048(
    ciphertext: &U2048,
    key_pair: &FcrKeyPair,
    master_seed: &[u8],
    nonce: &[u8],
) -> Result<U2048, &'static str> {
    let blinding = derive_blinding(master_seed, nonce)?;
    Ok(decrypt_with_blinding_ct(
        ciphertext,
        &key_pair.e_vec_inv,
        &blinding,
    ))
}

pub fn bytes_to_u2048(bytes: &[u8]) -> Result<U2048, &'static str> {
    if bytes.len() > 256 {
        return Err("Input too large for U2048");
    }
    let mut padded = [0u8; 256];
    let offset = 256 - bytes.len();
    padded[offset..].copy_from_slice(bytes);
    Ok(U2048::from_be_slice(&padded))
}

pub fn u2048_to_bytes(value: &U2048) -> [u8; 256] {
    value.to_be_bytes().into()
}
