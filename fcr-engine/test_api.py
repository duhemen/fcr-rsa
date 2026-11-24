import requests
import hmac
import hashlib
import base64
import json
from datetime import datetime, timezone

# KONFIGURASI
API_URL = "http://localhost:8080"
API_KEY = "fcr_9494bc460cac45ada117b3140b9f6074" # Ganti dengan output dari langkah 1
KEY_NAME = "test-key-1"

# 1. Siapkan Spatiotemporal Header
lat = "-6.2088"
lon = "106.8456"
timestamp = datetime.now(timezone.utc).isoformat()
latency = "50.0"

# 2. Buat HMAC Signature
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

# 3. Request Enkripsi
plaintext = "Rahasia Negara: Serangan FCR-RSA berhasil!"
text_b64 = base64.b64encode(plaintext.encode()).decode()

payload_json = {
    "key_name": KEY_NAME,
    "text_b64": text_b64,
    "tier": "b"
}

print("🚀 Mengirim request ke /encrypt...")
response = requests.post(f"{API_URL}/encrypt", headers=headers, json=payload_json)

print(f"Status Code: {response.status_code}")
print("Response JSON:")
print(json.dumps(response.json(), indent=2))

if response.status_code == 200:
    ct_b64 = response.json().get("ciphertext_b64")
    # Decode ciphertext_b64 untuk mendapatkan format 4 bagian yang sebenarnya
    actual_ciphertext = base64.b64decode(ct_b64).decode()
    print(f"\n✅ Ciphertext Asli (4 bagian): {actual_ciphertext}")
else:
    print("❌ Gagal enkripsi. Cek server logs.")