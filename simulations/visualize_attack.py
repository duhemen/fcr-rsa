import sys
import os
import matplotlib.pyplot as plt

sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from src.core_math import FCR_Core

def generate_linear_fibonacci(n):
    """Deret Fibonacci Linear Murni (Rentan LLL)"""
    F = [complex(1, 1), complex(1, 2)]
    for i in range(2, n):
        F_next = F[-1] + F[-2]
        F.append(F_next)
    return F

def main():
    print("=== VISUALISASI KRIPTANALISIS FCR-RSA ===")
    fcr = FCR_Core()
    
    n_points = 50 # Jumlah titik untuk digambar
    secret_index = 50
    noise_seed = 12345 # Seed tetap untuk visualisasi
    
    # 1. Generate Data
    # A. Fibonacci Linear Murni
    linear_data = generate_linear_fibonacci(n_points)
    
    # B. Fibonacci Non-Linear + Noise (FCR-RSA)
    noisy_data = []
    # Kita modifikasi fungsi generate untuk mengembalikan list, bukan hanya nilai akhir
    F_prev2 = complex(1, 1)
    F_prev1 = complex(1, 2)
    noisy_data.append(F_prev2)
    noisy_data.append(F_prev1)
    
    import numpy as np
    for i in range(2, n_points):
        seed_val = (noise_seed + i) % (2**32)
        np.random.seed(seed_val)
        noise_real = np.random.randint(-5, 5)
        noise_imag = np.random.randint(-5, 5)
        e_n = complex(noise_real, noise_imag)
        
        F_current = (F_prev1 * F_prev2) + e_n
        F_current = complex(F_current.real % 100, F_current.imag % 100) # Modulo kecil untuk visualisasi
        
        noisy_data.append(F_current)
        F_prev2 = F_prev1
        F_prev1 = F_current

    # 2. Plotting
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 6))
    
    # Plot A: Linear (Mudah ditebak LLL)
    x_linear = [p.real for p in linear_data]
    y_linear = [p.imag for p in linear_data]
    ax1.plot(x_linear, y_linear, 'o-', color='red', markersize=4, label='Fibonacci Linear')
    ax1.set_title("Rentan LLL/BKZ: Pola Teratur\n(Mudah Dipetakan sebagai Kisi)", fontsize=12, fontweight='bold')
    ax1.set_xlabel("Real")
    ax1.set_ylabel("Imajiner")
    ax1.grid(True, linestyle='--', alpha=0.6)
    ax1.legend()
    
    # Plot B: Non-Linear + Noise (Anti-LLL)
    x_noisy = [p.real for p in noisy_data]
    y_noisy = [p.imag for p in noisy_data]
    ax2.plot(x_noisy, y_noisy, 'x-', color='blue', markersize=6, alpha=0.7, label='FCR-RSA (Non-Linear + LWE)')
    ax2.set_title("Anti-LLL/BKZ: Pola Rusak (Noise)\n(Sulit Dipetakan sebagai Kisi)", fontsize=12, fontweight='bold')
    ax2.set_xlabel("Real")
    ax2.set_ylabel("Imajiner")
    ax2.grid(True, linestyle='--', alpha=0.6)
    ax2.legend()
    
    plt.suptitle("Perbandingan Ruang Kompleks: Fibonacci Murni vs FCR-RSA Multi-Layer", fontsize=14, fontweight='bold')
    plt.tight_layout()
    
    # Simpan gambar ke folder simulations
    output_path = os.path.join(os.path.dirname(__file__), 'hasil_visualisasi_lll.png')
    plt.savefig(output_path, dpi=300)
    print(f"\n✅ Visualisasi berhasil disimpan di: {output_path}")
    
    # Tampilkan plot
    plt.show()

if __name__ == "__main__":
    main()