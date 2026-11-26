/// Fibonacci Port Hopper
/// Menghasilkan urutan port yang deterministik namun sulit diprediksi oleh pihak ketiga
/// yang tidak mengetahui session_seed. Dirancang khusus untuk menembus CGNAT dengan aman.

const MIN_PORT: u16 = 1024;
const MAX_PORT: u16 = 65535;
const PORT_RANGE: u32 = (MAX_PORT as u32) - (MIN_PORT as u32) + 1; // 64512 port

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
    /// 
    /// **Mengonsumsi state**: setelah dipanggil, state hopper berubah,
    /// dan panggilan berikutnya akan menghasilkan port yang berbeda.
    pub fn next_port(&mut self) -> u16 {
        let next_state = self.state_1.wrapping_add(self.state_2) % LARGE_PRIME;
        
        self.state_1 = self.state_2;
        self.state_2 = next_state;

        let port_offset = (next_state % PORT_RANGE as u64) as u16;
        MIN_PORT + port_offset
    }

    /// Hitung port berikutnya **tanpa** mengonsumsi state hopper.
    /// 
    /// Method ini berguna untuk:
    /// - **Prediksi**: Mengetahui port yang akan datang tanpa mengubah state
    /// - **Test**: Sinkronisasi antara `expected_port()` dan `process_chunk_at()`
    /// - **Validasi**: Memeriksa port yang diharapkan sebelum benar-benar mengonsumsi
    /// 
    /// # Perbedaan dengan `next_port()`
    /// 
    /// | Method | State berubah? | Penggunaan |
    /// |--------|----------------|------------|
    /// | `next_port()` | ✅ Ya | Konsumsi port untuk chunk berikutnya |
    /// | `peek_next_port()` | ❌ Tidak | Prediksi/validasi port |
    /// 
    /// # Contoh
    /// 
    /// ```
    /// use fcr_tp::FibonacciHopper;
    /// 
    /// let mut hopper = FibonacciHopper::new(12345);
    /// 
    /// // Peek: state tidak berubah
    /// let peeked = hopper.peek_next_port();
    /// assert_eq!(peeked, hopper.peek_next_port());  // masih sama
    /// 
    /// // Next: state berubah
    /// let next = hopper.next_port();
    /// assert_eq!(next, peeked);  // sama dengan peek sebelumnya
    /// assert_ne!(hopper.peek_next_port(), peeked);  // sekarang berbeda
    /// ```
    pub fn peek_next_port(&self) -> u16 {
        let next_state = self.state_1.wrapping_add(self.state_2) % LARGE_PRIME;
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

        for _ in 0..100 {
            assert_eq!(h1.next_port(), h2.next_port());
        }
    }

    #[test]
    fn test_hopper_bounds() {
        let mut h = FibonacciHopper::new(9999);
        for _ in 0..1000 {
            let port = h.next_port();
            assert!(port >= MIN_PORT && port <= MAX_PORT, "Port {} out of bounds!", port);
        }
    }

    #[test]
    fn test_hopper_different_seeds() {
        let mut h1 = FibonacciHopper::new(111);
        let mut h2 = FibonacciHopper::new(222);

        let mut same_count = 0;
        for _ in 0..100 {
            if h1.next_port() == h2.next_port() {
                same_count += 1;
            }
        }
        assert!(same_count < 5, "Seeds berbeda menghasilkan terlalu banyak port yang sama!");
    }

    // ============================================================
    // Test: peek_next_port()
    // ============================================================

    #[test]
    fn test_peek_does_not_consume_state() {
        let h = FibonacciHopper::new(12345);
        
        let peek1 = h.peek_next_port();
        let peek2 = h.peek_next_port();
        let peek3 = h.peek_next_port();
        
        assert_eq!(peek1, peek2, "Peek berulang harus sama (state tidak berubah)");
        assert_eq!(peek2, peek3, "Peek berulang harus sama (state tidak berubah)");
    }

    #[test]
    fn test_peek_matches_next_in_sequence() {
        let mut h = FibonacciHopper::new(12345);
        
        for i in 0..100 {
            let peeked = h.peek_next_port();
            let next = h.next_port();
            assert_eq!(peeked, next,
                "Iterasi {}: peek ({}) harus sama dengan next ({})", i, peeked, next);
        }
    }

    #[test]
    fn test_peek_after_next_is_different() {
        let mut h = FibonacciHopper::new(12345);
        
        let peek1 = h.peek_next_port();
        let _ = h.next_port();
        let peek2 = h.peek_next_port();
        
        assert_ne!(peek1, peek2,
            "Setelah next, peek harus berbeda (state sudah berubah)");
    }

    #[test]
    fn test_peek_bounds() {
        let h = FibonacciHopper::new(9999);
        
        for _ in 0..1000 {
            let port = h.peek_next_port();
            assert!(port >= MIN_PORT && port <= MAX_PORT,
                "Peek port {} out of bounds!", port);
        }
    }

    #[test]
    fn test_peek_deterministic_across_instances() {
        let h1 = FibonacciHopper::new(12345);
        let h2 = FibonacciHopper::new(12345);
        
        assert_eq!(h1.peek_next_port(), h2.peek_next_port(),
            "Peek harus deterministik untuk seed yang sama");
    }
}