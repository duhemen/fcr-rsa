# red_team_dictionary_attack.py
# Simulasi Serangan Kamus (Dictionary Attack) pada Enkripsi Per-Byte

SEED = b"fcr-rsa-demo-seed"
N = 10
Q = 1000000007
BLINDING = 12345 # Blinding global yang sama untuk semua byte (KELEMAHAN SISTEM LAMA)

# --- FUNGSI SIMULASI ENKRIPSI PER-BYTE (CELAH LAMA) ---
def simulate_old_encrypt_byte(byte_val, seed, n, q, blinding):
    # Simulasi logika FCR-RSA Tier B yang disederhanakan
    # C = (M * E_vec) + Blinding mod Q
    e_vec = 71 
    c = ((byte_val * e_vec) + blinding) % q
    return c

# --- SERANGAN RED TEAM ---
def dictionary_attack(ciphertext_list):
    print("[*] Membangun Kamus Enkripsi (Mencoba 256 Kemungkinan Byte)...")
    dictionary = {}
    for i in range(256):
        ct = simulate_old_encrypt_byte(i, SEED, N, Q, BLINDING)
        # Simpan mapping: Ciphertext -> Karakter Asli
        dictionary[ct] = chr(i) if 32 <= i <= 126 else f"[NON-PRINTABLE:{i}]"
    
    print("[*] Mencocokkan Ciphertext Target dengan Kamus...")
    recovered_text = ""
    for ct in ciphertext_list:
        if ct in dictionary:
            recovered_text += dictionary[ct]
        else:
            recovered_text += "?"
            
    return recovered_text

# --- EKSEKUSI ---
if __name__ == "__main__":
    target_message = "HALO"
    print(f"\n[+] Pesan Asli Target: '{target_message}'")
    
    # 1. Attacker menyadap ciphertext dari pesan "HALO"
    # PERHATIAN: Dalam sistem lama, huruf 'L' yang muncul 2 kali akan memiliki ciphertext yang SAMA PERSIS.
    target_cts = [simulate_old_encrypt_byte(ord(char), SEED, N, Q, BLINDING) for char in target_message]
    print(f"[+] Ciphertext yang Disadap: {target_cts}")
    
    # 2. Attacker melakukan dictionary attack
    recovered = dictionary_attack(target_cts)
    
    print(f"\n[🚨 HASIL SERANGAN] Pesan berhasil direkonstruksi: '{recovered}'")
    print("\n[!] KESIMPULAN: Enkripsi per-byte dengan blinding global SANGAT RENTAN terhadap Dictionary Attack.")
    print("[!] SOLUSI: Gunakan Block-Based Encryption (Chunking) seperti yang sudah diperbaiki di fcr-engine/src/crypto.rs.")