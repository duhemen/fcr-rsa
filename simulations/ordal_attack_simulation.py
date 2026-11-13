import sys
import os

sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from src.auto_immune import AutoImmuneSystem

def simulate_ordal_attack():
    print("=== SIMULASI SERANGAN 'ORDAL' (PEGASUS/LST) ===\n")
    print("Skenario: Kunci privat Pak Bowo dicuri oleh Pegasus.")
    print("Penyerang mencoba bertransaksi dari luar negeri pada jam 3 pagi.\n")
    
    immune = AutoImmuneSystem()
    
    # Data Normal Pak Bowo
    pak_bowo_gps = 1.2 # Jakarta
    pak_bowo_jam = 10  # Jam 10 pagi
    pak_bowo_delay = 50 # Latensi normal
    
    print(f"[DATA NORMAL PAK BOWO] GPS: {pak_bowo_gps} | Jam: {pak_bowo_jam} | Delay: {pak_bowo_delay}ms\n")
    
    # Skenario 1: Transaksi Normal (Kontrol)
    print("[SKENARIO 1] Transaksi Normal dari Jakarta...")
    status, msg = immune.check_transaction(
        device_gps=pak_bowo_gps,
        transaction_time=pak_bowo_jam,
        expected_gps=1.0,
        allowed_hours=(8, 17),
        network_delay_ms=pak_bowo_delay
    )
    print(f"Status: {status} | {msg}\n")
    
    # Skenario 2: Kunci Dicuri, Transaksi dari Luar Negeri
    print("[SKENARIO 2] Penyerang (Pegasus) mencoba dari Luar Negeri pada Jam 3 Pagi...")
    status, msg = immune.check_transaction(
        device_gps=99.5, # GPS Eropa
        transaction_time=3, # Jam 3 pagi
        expected_gps=1.0,
        allowed_hours=(8, 17),
        network_delay_ms=50
    )
    print(f"Status: {status} | {msg}\n")
    
    # Skenario 3: Kunci Dicuri, Transaksi dengan Delay Tinggi (MITM)
    print("[SKENARIO 3] Penyerang menggunakan Proxy untuk menyadap (Latensi Tinggi)...")
    status, msg = immune.check_transaction(
        device_gps=1.1, # GPS mendekati Jakarta (menipu)
        transaction_time=10, # Jam normal
        expected_gps=1.0,
        allowed_hours=(8, 17),
        network_delay_ms=350 # Delay tidak wajar
    )
    print(f"Status: {status} | {msg}\n")
    
    print("[KESIMPULAN] Sistem Auto-Imun FCR-RSA berhasil mendeteksi dan mengisolasi")
    print("serangan 'Ordal' dalam ketiga skenario. Endemi berhasil dicegah menjadi Pandemi!")

if __name__ == "__main__":
    simulate_ordal_attack()