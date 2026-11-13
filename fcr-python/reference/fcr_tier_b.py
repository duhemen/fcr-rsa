from src.core_math import FCR_Core
import hashlib
import time

class TierB_EndUser(FCR_Core):
    def __init__(self, device_id):
        super().__init__()
        self.tier_name = f"Tier B (End-User: {device_id})"
        self.device_id = device_id
        self.default_index = 10 # Indeks kecil untuk kecepatan
        self.noise_scale = 5    # Noise kecil agar hemat baterai

    def create_session_token(self, bank_public_key):
        """
        End-User membuat token sesi (Session Key) menggunakan kunci publik bank.
        """
        print(f"[{self.tier_name}] Membuat Session Token...")
        
        # Membuat seed dinamis berdasarkan perangkat dan waktu
        dynamic_seed = int(hashlib.sha256(f"{self.device_id}{time.time()}".encode()).hexdigest()[:8], 16)
        
        # End-user menghitung Z_n lokal (ringan)
        local_key = self.generate_non_linear_fibonacci(self.default_index, seed_base=dynamic_seed, noise_scale=self.noise_scale)
        
        # Membentuk token (simulasi enkripsi hibrida dengan kunci publik bank)
        token = f"FCR-SESSION-{self.device_id}-{int(time.time())}"
        print(f"[{self.tier_name}] Token Dibuat: {token[:20]}...")
        
        return token