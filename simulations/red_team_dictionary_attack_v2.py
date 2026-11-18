# red_team_dictionary_attack_v2.py
# Uji Dictionary Attack pada FCR-Engine yang SUDAH DIPERBAIKI (Block-Based Encryption)

import requests
import hmac, hashlib
import json
from datetime import datetime, timezone

# Konfigurasi API
BASE_URL = "http://localhost:8080"
API_KEY = "MFHYNfwzYrbKuohfbahNQhlp4xCeyraK1UWTfjTlnHw"

def generate_hmac_signature(key, lat, lon, ts, latency):
    payload = f"{lat}:{lon}:{ts}:{latency}"
    return hmac.new(key.encode(), payload.encode(), hashlib.sha256).hexdigest()

def get_signed_headers(lat="-7.2575", lon="112.7521", latency="30"):
    """Generate header dengan signature HMAC-SHA256."""
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    sig = generate_hmac_signature(API_KEY, lat, lon, ts, latency)
    return {
        "Authorization": f"Bearer {API_KEY}",
        "Content-Type": "application/json",
        "X-FCR-GPS-Lat": lat,
        "X-FCR-GPS-Lon": lon,
        "X-FCR-Timestamp": ts,
        "X-FCR-Latency": latency,
        "X-FCR-Signature": sig
    }

def encrypt_via_api(text, key_name="vault_omega", tier="b"):
    """Enkripsi teks via API endpoint /encrypt."""
    import base64
    text_b64 = base64.b64encode(text.encode('utf-8')).decode('utf-8')
    
    headers = get_signed_headers()
    payload = {
        "key_name": key_name,
        "text_b64": text_b64,
        "tier": tier
    }
    
    response = requests.post(f"{BASE_URL}/encrypt", headers=headers, json=payload)
    if response.status_code == 200:
        return response.json()["ciphertext_b64"]
    else:
        raise Exception(f"Encryption failed: {response.status_code} - {response.text}")

def analyze_ciphertext_pattern(ciphertext_b64):
    """Analisis pola pada ciphertext (mencari pengulangan)."""
    import base64
    raw = base64.b64decode(ciphertext_b64).decode('utf-8')
    
    # Format: BLINDING:CT1,CT2,CT3,...
    parts = raw.split(':')
    if len(parts) != 2:
        return None, None
    
    blinding = parts[0]
    cts = parts[1].split(',')
    
    # Hitung frekuensi setiap ciphertext block
    from collections import Counter
    freq = Counter(cts)
    
    return blinding, freq

def dictionary_attack_attempt(ciphertext_b64):
    """
    Coba lakukan dictionary attack.
    Dalam sistem LAMA, huruf yang sama akan memiliki ciphertext yang sama.
    Dalam sistem BARU (block-based), pola ini hilang.
    """
    blinding, freq = analyze_ciphertext_pattern(ciphertext_b64)
    
    if freq is None:
        return "Format ciphertext tidak valid"
    
    # Cek apakah ada ciphertext block yang berulang
    repeated_blocks = {ct: count for ct, count in freq.items() if count > 1}
    
    if repeated_blocks:
        print(f"\n[🚨 PERINGATAN] Ditemukan {len(repeated_blocks)} blok ciphertext yang berulang!")
        print("Ini mengindikasikan sistem MASIH RENTAN terhadap dictionary attack.")
        print("Blok yang berulang:", repeated_blocks)
        return "GAGAL - Sistem masih rentan"
    else:
        print(f"\n[✅ AMAN] Tidak ada blok ciphertext yang berulang.")
        print(f"Total blok: {len(freq)} (semua unik)")
        print("Ini mengindikasikan sistem TAHAN terhadap dictionary attack.")
        return "BERHASIL - Sistem aman"

# ============================================================================
# EKSEKUSI UJI
# ============================================================================
if __name__ == "__main__":
    print("=" * 70)
    print("UJI DICTIONARY ATTACK PADA FCR-ENGINE (BLOCK-BASED ENCRYPTION)")
    print("=" * 70)
    
    # Pesan dengan pola berulang (huruf 'L' muncul 2x, 'O' muncul 2x)
    test_messages = [
        "HALO",
        "AAAA",
        "HELLO WORLD",
        "Dokumen Rahasia Pertahanan Nasional 2026!"
    ]
    
    for msg in test_messages:
        print(f"\n{'='*70}")
        print(f"Menguji pesan: '{msg}'")
        print(f"{'='*70}")
        
        try:
            # 1. Enkripsi via API
            ct_b64 = encrypt_via_api(msg)
            print(f"[1] Ciphertext (Base64): {ct_b64[:50]}...")
            
            # 2. Analisis pola
            blinding, freq = analyze_ciphertext_pattern(ct_b64)
            print(f"[2] Blinding: {blinding}")
            print(f"[3] Jumlah blok ciphertext: {len(freq)}")
            
            # 3. Coba dictionary attack
            result = dictionary_attack_attempt(ct_b64)
            print(f"\n[HASIL] {result}")
            
        except Exception as e:
            print(f"[ERROR] {e}")
    
    print(f"\n{'='*70}")
    print("KESIMPULAN AKHIR")
    print(f"{'='*70}")
    print("Jika semua pesan menunjukkan 'TIDAK ADA BLOK YANG BERULANG',")
    print("maka perbaikan Block-Based Encryption BERHASIL menutup celah dictionary attack.")
    print("Pola huruf (seperti 'L' yang muncul 2x di 'HALO') telah diacak sepenuhnya.")