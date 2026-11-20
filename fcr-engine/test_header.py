#!/usr/bin/env python3
"""
test_header.py — Verifikasi Observability Gap Fix (X-FCR-Risk-Indicator)
Membuktikan: header muncul HANYA saat risk_score di rentang 40-69.
Self-contained: buat API key fresh via DB, lalu cleanup.
"""
import os, sys, hmac, hashlib, base64, secrets, time
from datetime import datetime, timezone, timedelta
import requests
import psycopg2

API_URL  = "http://127.0.0.1:8080"
DB_URL   = os.environ.get("DATABASE_URL",
           "postgresql://postgres:Bp2jk%40kalteng123@127.0.0.1:5432/fcr_rsa")
KEY_NAME = "test-key-1"  # key kriptografi (seed) — harus sudah ada di tabel `keys`

JAKARTA = ("-6.2088", "106.8456")
NEWYORK = ("40.7128", "-74.0060")

def make_api_key():
    plaintext = "fcr_" + secrets.token_hex(16)
    key_hash  = hashlib.sha256(plaintext.encode()).hexdigest()
    with psycopg2.connect(DB_URL) as conn, conn.cursor() as cur:
        cur.execute(
            "INSERT INTO api_keys (user_name, label, key_prefix, key_hash) "
            "VALUES (%s,%s,%s,%s) RETURNING id",
            ("header-test", "obs-verify", plaintext[:8], key_hash))
        kid = cur.fetchone()[0]
    return plaintext, key_hash, kid

def now_ts(offset_sec=0):
    return (datetime.now(timezone.utc) + timedelta(seconds=offset_sec)).isoformat()

def headers_for(key, lat, lon, ts, latency):
    payload = f"{lat}:{lon}:{ts}:{latency}"
    sig = hmac.new(key.encode(), payload.encode(), hashlib.sha256).hexdigest()
    return {
        "Authorization": f"Bearer {key}",
        "X-FCR-GPS-Lat": lat, "X-FCR-GPS-Lon": lon,
        "X-FCR-Timestamp": ts, "X-FCR-Latency": latency,
        "X-FCR-Signature": sig, "Content-Type": "application/json",
    }

def do_encrypt(key, lat, lon, ts, latency):
    body = {
        "key_name": KEY_NAME,
        "text_b64": base64.b64encode(b"observability test").decode(),
        "tier": "b",
    }
    return requests.post(f"{API_URL}/encrypt",
                         headers=headers_for(key, lat, lon, ts, latency),
                         json=body)

def show(label, r, expect_header):
    got = r.headers.get("X-FCR-Risk-Indicator")
    ok  = (got == "review_recommended") == expect_header
    mark = "✅" if ok else "❌"
    print(f"  {mark} {label}")
    print(f"       status={r.status_code}  header={got or '(tidak ada)'}  "
          f"expected={'review_recommended' if expect_header else 'TIDAK ADA'}")
    return ok

# ============================================================
print("=" * 62)
print("  VERIFIKASI OBSERVABILITY GAP — X-FCR-Risk-Indicator")
print("=" * 62)

key, key_hash, kid = make_api_key()
try:
    # --- Skenario 1 (KONTROL NEGATIF): request normal → skor 5 → TANPA header
    r1 = do_encrypt(key, JAKARTA[0], JAKARTA[1], now_ts(), "50.0")
    ok1 = show("Skenario 1: Normal (Jakarta, 50ms) → skor ~5", r1, expect_header=False)

    # --- Skenario 2 (KONTROL POSITIF): rapid transition + latency tinggi → skor 50
    #     req1 sudah set last_state = Jakarta.
    #     req2 = NY, dt=+120s → rapid transition (+30); latency 900ms → +20. TOTAL=50.
    r2 = do_encrypt(key, NEWYORK[0], NEWYORK[1], now_ts(120), "900.0")
    ok2 = show("Skenario 2: Jakarta→NY +120s, latency 900ms → skor 50", r2, expect_header=True)

    print("-" * 62)
    if ok1 and ok2:
        print("  🟢 OBSERVABILITY GAP RESOLVED: header muncul TEPAT saat skor 40-69.")
    else:
        print("  🔴 MASIH ADA YANG MELESET — cek log server (eprintln/println).")
finally:
    # Cleanup: hapus key tes agar DB tetap bersih
    with psycopg2.connect(DB_URL) as conn, conn.cursor() as cur:
        cur.execute("DELETE FROM api_keys WHERE id = %s", (kid,))
    print("  🧹 API key tes dibersihkan dari DB.")