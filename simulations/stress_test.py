import sys
import os
import time
import matplotlib.pyplot as plt

sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from src.core_math import FCR_Core

def benchmark_encryption(index, noise_scale, label, iterations=10):
    fcr = FCR_Core()
    total_keygen = total_encrypt = total_decrypt = 0
    valid_count = 0
    
    for _ in range(iterations):
        start_time = time.perf_counter()
        public_key, private_key = fcr.generate_keys(index)
        total_keygen += (time.perf_counter() - start_time) * 1000
        
        # Enkripsi
        pesan = "Transaksi_Rahasia_Negara"
        start_time = time.perf_counter()
        ciphertext, original_M, r_blind = fcr.encrypt(pesan, public_key) # <-- Tambah r_blind
        total_encrypt += (time.perf_counter() - start_time) * 1000
        
        # Dekripsi
        start_time = time.perf_counter()
        decrypted_M = fcr.decrypt(ciphertext, r_blind, private_key) # <-- Tambah r_blind
        total_decrypt += (time.perf_counter() - start_time) * 1000
        
        if original_M == decrypted_M:
            valid_count += 1
            
    avg_keygen = total_keygen / iterations
    avg_encrypt = total_encrypt / iterations
    avg_decrypt = total_decrypt / iterations
    
    print(f"[{label}] (Rata-rata dari {iterations}x percobaan)")
    print(f" -> Indeks Fibonacci : {index}")
    print(f" -> Waktu Keygen     : {avg_keygen:.4f} ms")
    print(f" -> Waktu Enkripsi   : {avg_encrypt:.4f} ms")
    print(f" -> Waktu Dekripsi   : {avg_decrypt:.4f} ms")
    print(f" -> Validasi         : {valid_count}/{iterations} VALID\n")
    
    return avg_encrypt, avg_decrypt

if __name__ == "__main__":
    print("=== STRESS TEST AKADEMIS FCR-RSA (BIG INT EDITION) ===\n")
    
    labels = ["Tier B (Mobile)", "Tier A (Bank)", "Maksimal (Quantum)"]
    indices = [10, 100, 500]
    
    encrypt_times = []
    decrypt_times = []
    
    for i in range(len(labels)):
        enc, dec = benchmark_encryption(indices[i], noise_scale=indices[i]//2, label=labels[i])
        encrypt_times.append(enc)
        decrypt_times.append(dec)
        
    print("Membuat grafik benchmark...")
    x = range(len(labels))
    width = 0.35
    
    fig, ax = plt.subplots(figsize=(10, 6))
    rects1 = ax.bar([p - width/2 for p in x], encrypt_times, width, label='Enkripsi (ms)', color='skyblue')
    rects2 = ax.bar([p + width/2 for p in x], decrypt_times, width, label='Dekripsi (ms)', color='salmon')
    
    ax.set_ylabel('Waktu (milidetik)')
    ax.set_title('Benchmark Kinerja FCR-RSA (Big Integer Edition)')
    ax.set_xticks(x)
    ax.set_xticklabels(labels)
    ax.legend()
    ax.grid(axis='y', linestyle='--', alpha=0.7)
    
    ax.bar_label(rects1, fmt='%.3f ms', padding=3)
    ax.bar_label(rects2, fmt='%.3f ms', padding=3)
    
    plt.tight_layout()
    output_path = os.path.join(os.path.dirname(__file__), 'hasil_benchmark_fcr_rsa_fixed.png')
    plt.savefig(output_path, dpi=300)
    print(f"✅ Grafik berhasil disimpan di: {output_path}")
    
    plt.show()