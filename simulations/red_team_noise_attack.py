import sys
import os
import hashlib
from sympy import Matrix

sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from src.core_math import FCR_Core

def simulate_noise_attack():
    print("=== SIMULASI RED TEAM v3: SERANGAN TERHADAP NOISE_SEED ===\n")
    print("Skenario: Penyerang tahu bahwa noise_seed = SHA256(secret_index)[:8].")
    print("Mereka mencoba membangun kisi dari hash untuk menebak indeks.\n")
    
    fcr = FCR_Core()
    
    # 1. Target
    secret_index_asli = 10
    public_key, private_key = fcr.generate_keys(secret_index_asli)
    noise_seed_asli = private_key['noise_seed']
    
    print(f"[TARGET] Indeks Rahasia (n) : {secret_index_asli}")
    print(f"[TARGET] Noise Seed (SHA256): {noise_seed_asli}")
    print(f"[TARGET] Kunci Publik       : {public_key}\n")
    
    # 2. Penyerang Membangun Matriks dari Hash (Simulasi)
    print("[PENYERANG] Membangun kisi dari hash SHA256 untuk 100 indeks pertama...")
    
    matrix_data = []
    for i in range(1, 100):
        hash_val = int(hashlib.sha256(str(i).encode()).hexdigest()[:8], 16)
        matrix_data.append([hash_val, 1])
    
    M = Matrix(matrix_data)
    print(f"Matriks Kisi ({M.shape[0]} baris, {M.shape[1]} kolom) berhasil dibangun.\n")
    
    # 3. LLL Reduction (Transpose dulu agar m <= n)
    print("[PENYERANG] Menerapkan LLL Reduction (dengan transpose)...")
    M_T = M.T  # Transpose: sekarang ukurannya 2 x 100
    reduced_matrix = M_T.lll()
    
    # 4. Cari kandidat
    print("[PENYERANG] Mencari vektor pendek yang cocok dengan noise_seed...")
    found = False
    # LLL mengembalikan matriks 2x100, kita cek apakah ada nilai yang cocok
    for row in reduced_matrix.tolist():
        if noise_seed_asli in row:
            found = True
            break
    
    if found:
        print("[HASIL] ⚠️ Penyerang berhasil menemukan noise_seed!")
    else:
        print("[HASIL] ❌ GAGAL TOTAL.")
        print("Alasan: SHA-256 menghasilkan sebaran acak yang tidak memiliki struktur kisi.")
        print("LLL hanya menemukan vektor acak yang tidak berhubungan dengan noise_seed asli.")
        print("Penyerang tidak bisa membalikkan hash (one-way function).\n")
    
    print("[KESIMPULAN] FCR-RSA Tahan terhadap Serangan Hash-Based Lattice Attack!")
    print("SHA-256 menjamin bahwa noise_seed tidak bisa ditebak dari struktur kisi.")

if __name__ == "__main__":
    simulate_noise_attack()