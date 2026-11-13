import sys
import os
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

from src.fcr_tier_a import TierA_Enterprise
from src.fcr_tier_b import TierB_EndUser
from src.auto_immune import AutoImmuneSystem

def main():
    print("=== SIMULASI FCR-RSA BRIDGE PROTOCOL (TIER A & TIER B) ===\n")

    # 1. Inisialisasi Bank (Tier A)
    bank = TierA_Enterprise()
    bank_master_key = bank.generate_master_key()

    # 2. Inisialisasi HP Nasabah (Tier B)
    hp_budi = TierB_EndUser(device_id="HP-BUDI-123")
    
    # 3. Nasabah Membuat Session Token untuk Transaksi
    session_token = hp_budi.create_session_token(bank_master_key['public_key'])

    # 4. Jembatan Komunikasi (Bridge)
    # Bank menerima token dan memverifikasinya
    print("\n--- [JEMBATAN KOMUNIKASI] ---")
    is_valid = bank.verify_session_key(session_token)

    if is_valid:
        print("✅ Bank menerima token. Transaksi diizinkan.")
        
        # 5. Sistem Auto-Imun Memeriksa Konteks Transaksi
        immune = AutoImmuneSystem()
        
        # Skenario A: Transaksi Normal dari Jakarta
        print("\n[Simulasi Transaksi Normal]")
        status, msg = immune.check_transaction(
            device_gps=1.2,       # GPS Jakarta
            transaction_time=10,  # Jam 10 Pagi
            expected_gps=1.0,     # Sesuai ekspektasi
            allowed_hours=(8, 17),
            network_delay_ms=45   # Jaringan normal
        )
        print(f"Hasil: {msg}")

        # Skenario B: Serangan MITM (Peretas Menyadap)
        print("\n[Simulasi Serangan MITM]")
        status, msg = immune.check_transaction(
            device_gps=1.1, 
            transaction_time=10, 
            expected_gps=1.0, 
            allowed_hours=(8, 17),
            network_delay_ms=300  # Jaringan melambat karena disadap
        )
        print(f"Hasil: {msg}")

    else:
        print("❌ Token tidak valid. Transaksi ditolak.")

if __name__ == "__main__":
    main()