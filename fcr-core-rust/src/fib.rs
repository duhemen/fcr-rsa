//! FCR Core - Mathematical Foundation (v0.6.0)
//! Upgrade: Q 2048-bit dengan constant-time operations via Montgomery Arithmetic (v0.5.5 compatible)

use crypto_bigint::{U2048, NonZero, Zero, Encoding};
use crypto_bigint::modular::runtime_mod::{DynResidue, DynResidueParams};
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
     3995497CEA956AE515D2261898FA051015728E5A8AACAA68FFFFFFFFFFFFFFFF"
);

fn q_nz() -> NonZero<U2048> {
    NonZero::new(Q_2048).unwrap()
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
    
    if e_vec.is_zero().into() {
        return Err("Generated E_vec is zero");
    }
    
    let (e_vec_inv, is_some) = e_vec.inv_mod(&Q_2048);
    if !bool::from(is_some) {
        return Err("Modular inverse does not exist");
    }
    
    Ok(FcrKeyPair { e_vec, e_vec_inv })
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
    // SOLUSI FINAL v0.5.5: Gunakan DynResidue untuk Montgomery reduction
    let params = Option::from(DynResidueParams::new(q)).expect("Modulus must be odd");
    let val_res = DynResidue::new(value, params);
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
    // SOLUSI FINAL v0.5.5: Perkalian modular via DynResidue
    let params = Option::from(DynResidueParams::new(&Q_2048)).expect("Q must be odd");
    let msg_res = DynResidue::new(message, params);
    let e_res = DynResidue::new(e_vec, params);
    let prod = msg_res.mul(&e_res).retrieve();
    
    prod.add_mod(blinding, &q_nz())
}

pub fn decrypt_with_blinding_ct(ciphertext: &U2048, e_vec_inv: &U2048, blinding: &U2048) -> U2048 {
    let q_nonzero = q_nz();
    let unblinded = ciphertext.sub_mod(blinding, &q_nonzero);
    
    // SOLUSI FINAL v0.5.5: Perkalian modular via DynResidue
    let params = Option::from(DynResidueParams::new(&Q_2048)).expect("Q must be odd");
    let unblinded_res = DynResidue::new(&unblinded, params);
    let inv_res = DynResidue::new(e_vec_inv, params);
    unblinded_res.mul(&inv_res).retrieve()
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
    value.to_be_bytes()
}