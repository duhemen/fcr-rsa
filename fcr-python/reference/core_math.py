import numpy as np
import hashlib

class FCR_Core:
    def __init__(self):
        self.Q_real = 104729
        self.Q_imag = 104723

    def generate_non_linear_fibonacci(self, n, seed_base, noise_scale=10):
        F_prev2 = (1, 1)
        F_prev1 = (1, 2)
        
        for i in range(2, n + 1):
            seed_val = (seed_base + i) % (2**32)
            np.random.seed(seed_val)
            
            noise_real = np.random.randint(-noise_scale, noise_scale)
            noise_imag = np.random.randint(-noise_scale, noise_scale)
            
            a, b = F_prev1
            c, d = F_prev2
            
            real_part = (a * c - b * d) + noise_real
            imag_part = (a * d + b * c) + noise_imag
            
            # Modulo hanya diterapkan pada pembentukan Kunci (Z_n)
            F_current = (real_part % self.Q_real, imag_part % self.Q_imag)
            
            F_prev2 = F_prev1
            F_prev1 = F_current
            
        return F_prev1

    def generate_keys(self, secret_index):
        noise_seed = int(hashlib.sha256(str(secret_index).encode()).hexdigest()[:8], 16)
        private_key = {
            'secret_index': secret_index,
            'noise_seed': noise_seed
        }
        public_key = self.generate_non_linear_fibonacci(secret_index, seed_base=noise_seed)
        return public_key, private_key

    def encrypt(self, message_str, public_key):
        """Proses Enkripsi dengan IND-CPA Patch (Random Blinding)"""
        hash_val = int(hashlib.sha256(message_str.encode()).hexdigest(), 16)
        M_x = hash_val % 10000
        M_y = (hash_val // 10000) % 10000
        
        E_vec = (3, 5)
        N_comp = public_key
        
        # 1. Buat Random Blinding Factor (R_blind)
        # Menggunakan os.urandom untuk randomness kriptografis
        import os
        r_seed = int.from_bytes(os.urandom(4), 'big')
        R_x = r_seed % 1000
        R_y = (r_seed // 1000) % 1000
        
        # 2. Enkripsi: C = (M * E_vec) + N_comp + R_blind
        C_real = (M_x * E_vec[0] - M_y * E_vec[1]) + N_comp[0] + R_x
        C_imag = (M_x * E_vec[1] + M_y * E_vec[0]) + N_comp[1] + R_y
        
        C = (C_real % self.Q_real, C_imag % self.Q_imag)
        M = (M_x, M_y)
        R_blind = (R_x, R_y)
        
        # Ciphertext sekarang mengembalikan 3 nilai: C, M, dan R_blind
        return C, M, R_blind

    def decrypt(self, ciphertext, R_blind, private_key):
        """Proses Dekripsi dengan membuang R_blind"""
        secret_index = private_key['secret_index']
        noise_seed = private_key['noise_seed']
        
        Z_n = self.generate_non_linear_fibonacci(secret_index, seed_base=noise_seed)
        E_vec = (3, 5)
        
        # 1. Kurangi C dengan Z_n dan R_blind
        diff_real = ciphertext[0] - Z_n[0] - R_blind[0]
        diff_imag = ciphertext[1] - Z_n[1] - R_blind[1]
        
        # 2. Bagi dengan E_vec
        denom = E_vec[0]**2 + E_vec[1]**2
        M_x_rec = round((diff_real * E_vec[0] + diff_imag * E_vec[1]) / denom)
        M_y_rec = round((diff_imag * E_vec[0] - diff_real * E_vec[1]) / denom)
        
        return (M_x_rec, M_y_rec)