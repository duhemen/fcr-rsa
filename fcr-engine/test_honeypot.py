import requests
import hmac
import hashlib
import base64
import json
import time
from datetime import datetime, timezone

# KONFIGURASI
API_URL = "http://localhost:8080"
API_KEY = "fcr_8b1066b290c68dc896858c27681a4dd5" # Ganti dengan API Key Anda
KEY_NAME = "test-key-1"

def make_request(lat, lon, label):
    print(f"\n--- 🧪 TEST: {label} ---")
    timestamp = datetime.now(timezone.utc).isoformat()
    latency = "50.0"
    
    # Buat HMAC Signature
    payload = f"{lat}:{lon}:{timestamp}:{latency}"
    signature = hmac.new(API_KEY.encode(), payload.encode(), hashlib.sha256).hexdigest()

    headers = {
        "Authorization": f"Bearer {API_KEY}",
        "X-FCR-GPS-Lat": lat,
        "X-FCR-GPS-Lon": lon,
        "X-FCR-Timestamp": timestamp,
        "X-FCR-Latency": latency,
        "X-FCR-Signature": signature,
        "Content-Type": "application/json"
    }

    plaintext = "Rahasia Negara: Serangan FCR-RSA berhasil!"
    text_b64 = base64.b64encode(plaintext.encode()).decode()
    
    payload_json = {
        "key_name": KEY_NAME,
        "text_b64": text_b64,
        "tier": "b"
    }

    response = requests.post(f"{API_URL}/encrypt", headers=headers, json=payload_json)
    
    if response.status_code == 200:
        res_json = response.json()
        ct_b64 = res_json.get("ciphertext_b64")
        actual_ct = base64.b64decode(ct_b64).decode()
        print(f"✅ Status: {response.status_code}")
        print(f"📦 Blocks: {res_json.get('blocks')}")
        print(f"🔐 Ciphertext UTUH:\n{actual_ct}\n")
        return actual_ct
    else:
        print(f"❌ Status: {response.status_code}")
        print(f"Error: {response.json()}")
        return None

# 1. Request Normal (Jakarta) - SEHARUSNYA SUKSES
ct_jakarta = make_request("-6.2088", "106.8456", "Normal Request (Jakarta)")

# Tunggu 1 detik (untuk memastikan timestamp berbeda tapi masih dalam window < 60s)
time.sleep(1)

# 2. Request Spoofing (Pindah ke New York secara instan) - SEHARUSNYA KENA HONEYPOT
# Koordinat New York: Lat 40.7128, Lon -74.0060
ct_newyork = make_request("40.7128", "-74.0060", "Spoofing Request (Jakarta -> New York dalam 1 detik!)")

# 3. Analisis Hasil
print("\n" + "="*50)
print("🕵️‍♂️ HASIL ANALISIS AUTO-MUNE:")
print("="*50)
if ct_jakarta and ct_newyork:
    if ct_jakarta == ct_newyork:
        print("⚠️  Ciphertext identik. (Honeypot mungkin belum terpicu)")
    else:
        print("🚨 TERDETEKSI! Ciphertext berbeda!")
        print("Server mengembalikan HONEYPOT (Ciphertext Palsu) untuk menipu penyerang.")
        print("API Key Anda kemungkinan besar sedang dalam masa Cooldown (5 menit).")