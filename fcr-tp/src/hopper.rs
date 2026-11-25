/// Fibonacci Port Hopper
/// Menghasilkan urutan port yang deterministik namun sulit diprediksi oleh pihak ketiga
/// yang tidak mengetahui session_seed. Dirancang khusus untuk menembus CGNAT dengan aman.

const MIN_PORT: u16 = 1024;
const MAX_PORT: u16 = 65535;
const PORT_RANGE: u32 = (MAX_PORT - MIN_PORT + 1) as u32; // 64512 port yang tersedia

// Sebuah bilangan prima besar (2^64 - 59) untuk modulus, 
// memastikan periode urutan port sangat panjang dan tidak mudah dipecahkan.
const LARGE_PRIME: u64 = 0xFFFFFFFFFFFFFFC5; 

pub struct FibonacciHopper {
    state_1: u64,
    state_2: u64,
}

impl FibonacciHopper {
    /// Inisialisasi hopper dengan seed sesi.
    /// Seed dipecah dan diacak (mixed) menjadi dua state awal agar tidak trivial.
    pub fn new(session_seed: u64) -> Self {
        // Simple seed mixing untuk menginisialisasi dua state
        let s1 = session_seed ^ 0x517cc1b727220a95;
        let s2 = session_seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        
        Self {
            state_1: s1.max(1), // Pastikan tidak nol agar deret tidak degenerat
            state_2: s2.max(1),
        }
    }

    /// Hitung port berikutnya dalam urutan Fibonacci termodifikasi.
    pub fn next_port(&mut self) -> u16 {
        // Langkah Fibonacci: F(n) = (F(n-1) + F(n-2)) mod P
        let next_state = self.state_1.wrapping_add(self.state_2) % LARGE_PRIME;
        
        // Update state untuk langkah selanjutnya
        self.state_1 = self.state_2;
        self.state_2 = next_state;

        // Petakan (map) hasil ke dalam rentang port yang valid
        let port_offset = (next_state % PORT_RANGE as u64) as u16;
        MIN_PORT + port_offset
    }
}

// === UJI COBA (TEST) ===
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hopper_deterministic() {
        let mut h1 = FibonacciHopper::new(12345);
        let mut h2 = FibonacciHopper::new(12345);

        // Urutan port HARUS identik untuk seed yang sama (Klien & Server sinkron)
        for _ in 0..100 {
            assert_eq!(h1.next_port(), h2.next_port());
        }
    }

    #[test]
    fn test_hopper_bounds() {
        let mut h = FibonacciHopper::new(9999);
        // Pastikan TIDAK ADA port yang keluar dari batas aman (1024 - 65535)
        for _ in 0..1000 {
            let port = h.next_port();
            assert!(port >= MIN_PORT && port <= MAX_PORT, "Port {} out of bounds!", port);
        }
    }

    #[test]
    fn test_hopper_different_seeds() {
        let mut h1 = FibonacciHopper::new(111);
        let mut h2 = FibonacciHopper::new(222);

        // Urutan port HARUS berbeda untuk seed yang berbeda (Anti-fingerprinting)
        let mut same_count = 0;
        for _ in 0..100 {
            if h1.next_port() == h2.next_port() {
                same_count += 1;
            }
        }
        assert!(same_count < 5, "Seeds berbeda menghasilkan terlalu banyak port yang sama!");
    }
}