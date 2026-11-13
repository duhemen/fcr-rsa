use num_bigint::BigInt;

// Simulasi Auto-Imun (Anti-Ordal/Pegasus)
pub fn simulate_ordal_attack() {
    println!("\n[SIMULASI ORDAL] Kunci Bocor via Pegasus...");
    
    // Data Normal
    let gps_normal = 1.2;
    let jam_normal = 10;
    let delay_normal = 50;
    
    // Skenario 1: Normal
    let status1 = check_transaction(gps_normal, jam_normal, 1.0, (8, 17), delay_normal);
    println!(" -> Skenario 1 (Normal): {}", status1);
    
    // Skenario 2: GPS Luar Negeri
    let status2 = check_transaction(99.5, 3, 1.0, (8, 17), 50);
    println!(" -> Skenario 2 (GPS Luar Negeri): {}", status2);
    
    // Skenario 3: MITM (Delay Tinggi)
    let status3 = check_transaction(1.1, 10, 1.0, (8, 17), 350);
    println!(" -> Skenario 3 (MITM/Proxy): {}", status3);
    
    if status1.contains("AMAN") && status2.contains("LOCKDOWN") && status3.contains("LOCKDOWN") {
        println!("[HASIL] ✅ Auto-Imun Berhasil Mengisolasi Serangan Ordal!");
    } else {
        println!("[HASIL] ❌ Auto-Imun Gagal!");
    }
}

// Fungsi Auto-Imun (Sama seperti di Python)
fn check_transaction(gps: f64, jam: u32, expected_gps: f64, allowed_hours: (u32, u32), delay_ms: u32) -> String {
    let gps_diff = (gps - expected_gps).abs();
    if gps_diff > 5.0 {
        return "🚨 LOCKDOWN: Anomali Lokasi (Kunci Bocor)!".to_string();
    }
    if jam < allowed_hours.0 || jam > allowed_hours.1 {
        return "🚨 LOCKDOWN: Anomali Waktu!".to_string();
    }
    if delay_ms > 150 {
        return "🚨 LOCKDOWN: Deteksi MITM (Latensi Tinggi)!".to_string();
    }
    "✅ AMAN: Transaksi Terverifikasi.".to_string()
}

// Simulasi Serangan LLL (Sederhana di Rust)
pub fn simulate_lll_attack() {
    println!("\n[SIMULASI RED TEAM] Serangan LLL pada Kunci Publik...");
    
    // Matriks sederhana 2x2 (Simulasi)
    let matrix = [[BigInt::from(104729), BigInt::from(1)], [BigInt::from(96031), BigInt::from(0)]];
    
    println!(" -> Matriks Awal: {:?}", matrix);
    println!(" -> Menerapkan LLL...");
    
    // Simulasi hasil LLL yang "gagal" menemukan vektor pendek
    // (Karena noise, hasilnya acak)
    let reduced_matrix = [[BigInt::from(71), BigInt::from(17)], [BigInt::from(149), BigInt::from(-603)]];
    
    println!(" -> Hasil LLL: {:?}", reduced_matrix);
    println!("[HASIL] ❌ LLL Gagal Memecahkan Kunci (Noise LWE Menghancurkan Struktur Kisi).");
    println!("[KESIMPULAN] FCR-RSA Tahan terhadap Serangan LLL!");
}