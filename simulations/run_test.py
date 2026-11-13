import sys
import os

# Menambahkan folder src ke dalam path agar bisa import
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

from src.core_math import FCR_Core
from src.auto_immune import AutoImmuneSystem

def main():
    print("=== SIMULASI FCR-RSA MULTI-LAYER ===")
    
    # Inisialisasi
    fcr = FCR_Core()
    immune_system = AutoImmuneSystem()
    
    # 1. Uji Matematika (Keygen, Encrypt, Decrypt)
    print("\n--- 1. UJI MATEMATIKA INTI ---")
    secret_index = 10 
    public_key, private_key = fcr.generate_keys(secret_index)
    
    pesan = "Transfer_Dana_Rp_1_Miliar"
    ciphertext, original_M, R_blind = fcr.encrypt(pesan, public_key)
    decrypted_M = fcr.decrypt(ciphertext, R_blind, private_key)
    
    if original_M == decrypted_M:
        print("✅ SUKSES: Enkripsi & Dekripsi Berhasil 100%.")
    else:
        print("❌ GAGAL: Pesan tidak cocok.")

    # 2. Uji Auto-Imun & Anti-MITM
    print("\n--- 2. UJI SISTEM AUTO-IMUN ---")
    
    # Kasus A: Normal
    status, msg = immune_system.check_transaction(1.2, 10, 1.0, (8, 17), 50)
    print(f"Status: {status} | {msg}")
    
    # Kasus B: Kunci Bocor (GPS Anomali)
    status, msg = immune_system.check_transaction(95.0, 3, 1.0, (8, 17), 50)
    print(f"Status: {status} | {msg}")
    
    # Kasus C: Serangan MITM (Delay Tinggi)
    status, msg = immune_system.check_transaction(1.1, 10, 1.0, (8, 17), 250)
    print(f"Status: {status} | {msg}")

if __name__ == "__main__":
    main()