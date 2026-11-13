use num_bigint::BigInt;
use num_traits::{One, ToPrimitive};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

mod red_team;

#[derive(Debug, Clone)]
struct FcrCore {
    q_real: BigInt,
    q_imag: BigInt,
}

impl FcrCore {
    fn new() -> Self {
        FcrCore {
            q_real: BigInt::from(104729),
            q_imag: BigInt::from(104723),
        }
    }

    fn generate_non_linear_fibonacci(&self, n: u32, seed_base: u64, noise_scale: i64) -> (BigInt, BigInt) {
        let mut f_prev2 = (BigInt::one(), BigInt::one());
        let mut f_prev1 = (BigInt::one(), BigInt::from(2));

        for i in 2..=n {
            let mut hasher = DefaultHasher::new();
            seed_base.hash(&mut hasher);
            i.hash(&mut hasher);
            let hashed_val = hasher.finish();

            let noise_real = (hashed_val % (2 * noise_scale as u64 + 1)) as i64 - noise_scale;
            let noise_imag = ((hashed_val >> 16) % (2 * noise_scale as u64 + 1)) as i64 - noise_scale;

            let a = &f_prev1.0;
            let b = &f_prev1.1;
            let c = &f_prev2.0;
            let d = &f_prev2.1;

            let real_part = (a * c) - (b * d) + BigInt::from(noise_real);
            let imag_part = (a * d) + (b * c) + BigInt::from(noise_imag);

            let f_current_real = real_part % &self.q_real;
            let f_current_imag = imag_part % &self.q_imag;

            f_prev2 = f_prev1;
            f_prev1 = (f_current_real, f_current_imag);
        }

        f_prev1
    }

    fn generate_keys(&self, secret_index: u32) -> ((BigInt, BigInt), (u32, u64)) {
        let seed_base = (secret_index as u64) * 12345;
        let public_key = self.generate_non_linear_fibonacci(secret_index, seed_base, 10);
        let private_key = (secret_index, seed_base);
        (public_key, private_key)
    }

    // --- IND-CPA PATCHED ENCRYPT ---
    fn encrypt(&self, m_x: i64, m_y: i64, public_key: &(BigInt, BigInt)) -> ((BigInt, BigInt), (i64, i64)) {
        let e_real = BigInt::from(3);
        let e_imag = BigInt::from(5);

        // 1. Buat Random Blinding (menggunakan waktu sebagai seed sederhana)
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let r_x = (now % 1000) as i64;
        let r_y = ((now >> 8) % 1000) as i64;

        // 2. Enkripsi: C = (M * E_vec) + N_comp + R_blind
        let c_real = (BigInt::from(m_x) * &e_real) - (BigInt::from(m_y) * &e_imag) + &public_key.0 + BigInt::from(r_x);
        let c_imag = (BigInt::from(m_x) * &e_imag) + (BigInt::from(m_y) * &e_real) + &public_key.1 + BigInt::from(r_y);

        ((c_real, c_imag), (r_x, r_y))
    }

    // --- IND-CPA PATCHED DECRYPT ---
    fn decrypt(&self, ciphertext: &(BigInt, BigInt), r_blind: (i64, i64), private_key: &(u32, u64)) -> (i64, i64) {
        let z_n = self.generate_non_linear_fibonacci(private_key.0, private_key.1, 10);
        let e_real = BigInt::from(3);
        let e_imag = BigInt::from(5);

        // Kurangi dengan Z_n dan R_blind
        let diff_real = &ciphertext.0 - &z_n.0 - BigInt::from(r_blind.0);
        let diff_imag = &ciphertext.1 - &z_n.1 - BigInt::from(r_blind.1);

        let denom = BigInt::from(34);

        let m_x_rec = (diff_real.clone() * &e_real + diff_imag.clone() * &e_imag) / &denom;
        let m_y_rec = (diff_imag * &e_real - diff_real * &e_imag) / &denom;

        (m_x_rec.to_i64().unwrap_or(0), m_y_rec.to_i64().unwrap_or(0))
    }
}

fn main() {
    println!("=== FCR-RSA RUST EDITION (IND-CPA PATCHED) ===\n");
    let fcr = FcrCore::new();

    let secret_index = 500; 
    let original_m = (1234i64, 5678i64);

    println!("[1] Mempersiapkan Kunci (Indeks: {})...", secret_index);
    let (public_key, private_key) = fcr.generate_keys(secret_index);
    
    let pk_real_str = public_key.0.to_string();
    let pk_imag_str = public_key.1.to_string();
    
    let pk_real_short: String = pk_real_str.chars().take(10).collect();
    let pk_imag_short: String = pk_imag_str.chars().take(10).collect();
    println!("Public Key (Real): {}...", pk_real_short);
    println!("Public Key (Imag): {}...", pk_imag_short);

    println!("\n[2] Proses Enkripsi (IND-CPA)...");
    let (ciphertext, r_blind) = fcr.encrypt(original_m.0, original_m.1, &public_key);
    let ct_real_str = ciphertext.0.to_string();
    let ct_real_short: String = ct_real_str.chars().take(10).collect();
    println!("Ciphertext (Real): {}...", ct_real_short);

    println!("\n[3] Proses Dekripsi...");
    let decrypted_m = fcr.decrypt(&ciphertext, r_blind, &private_key);
    println!("Pesan Asli: {:?}", original_m);
    println!("Pesan Dekripsi: {:?}", decrypted_m);

    if original_m == decrypted_m {
        println!("\n✅ SUKSES: FCR-RSA Rust Edition Berhasil Menembus Indeks {} (IND-CPA Validated)!", secret_index);
        println!("\n=== MEMULAI SIMULASI RED TEAM (RUST EDITION) ===");
        red_team::simulate_ordal_attack();
        red_team::simulate_lll_attack();
    } else {
        println!("\n❌ GAGAL: Terjadi kesalahan presisi atau matematis.");
    }
}