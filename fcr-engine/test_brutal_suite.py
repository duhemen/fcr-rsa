#!/usr/bin/env python3
"""
FCR-RSA ENGINE — BRUTAL TEST SUITE v1.0
Menginjak SELURUH klaim desain: Kriptografi, Spatiotemporal Auth,
Auto-Mune Scoring, Honeypot/Forensics, API Surface.
Setiap tes = API key FRESH (deterministik, tanpa state silang).
"""
import os, sys, json, hmac, hashlib, base64, secrets, subprocess
from datetime import datetime, timezone, timedelta

try:
    import requests
except ImportError:
    sys.exit("❌ pip install requests")
try:
    import psycopg2
    import psycopg2.extras
except ImportError:
    sys.exit("❌ pip install psycopg2-binary")

# ============================================================
# KONFIGURASI — SESUAIKAN
# ============================================================
API_URL   = "http://localhost:8080"
DB_URL    = os.environ.get("DATABASE_URL",
            "postgresql://postgres:PASSWORD_ANDA@127.0.0.1:5432/fcr_rsa")
KEY_NAME  = "test-key-1"          # pastikan sudah ada di DB (keygen kemarin)
ENGINE_DIR= os.environ.get("ENGINE_DIR", os.path.dirname(os.path.abspath(__file__)))
NORMAL_TEXT = "Halo Dunia, ini adalah rahasia!"   # 42 byte
NORMAL_LAT, NORMAL_LON = "-6.2088", "106.8456"    # Jakarta
NY_LAT,   NY_LON       = "40.7128", "-74.0060"    # New York

# ============================================================
# FRAMEWORK ASSERT
# ============================================================
RESULTS = {"PASS": 0, "FAIL": 0, "DETAILS": []}
def section(t): print(f"\n{'='*62}\n  {t}\n{'='*62}")
def t(name, cond, note=""):
    ok = bool(cond)
    RESULTS["PASS" if ok else "FAIL"] += 1
    RESULTS["DETAILS"].append((ok, name, note))
    print(f"  [{'✅ PASS' if ok else '❌ FAIL'}] {name}" + (f"  ({note})" if note and not ok else ""))

# ============================================================
# DB HELPERS
# ============================================================
def db():
    return psycopg2.connect(DB_URL)

def make_api_key(user="brutal", label="auto-test"):
    """Insert API key FRESH ke DB (meniru crypto::generate_api_key + db::insert_api_key)."""
    plaintext = "fcr_" + secrets.token_hex(16)
    prefix = plaintext[:8]
    key_hash = hashlib.sha256(plaintext.encode()).hexdigest()
    with db() as conn, conn.cursor() as cur:
        cur.execute(
            "INSERT INTO api_keys (user_name, label, key_prefix, key_hash) VALUES (%s,%s,%s,%s) RETURNING id",
            (user, label, prefix, key_hash))
        kid = cur.fetchone()[0]
    return plaintext, key_hash, kid

def reset_cooldown(key_hash):
    with db() as conn, conn.cursor() as cur:
        cur.execute("UPDATE api_keys SET lockout_until=NULL, is_locked=FALSE WHERE key_hash=%s", (key_hash,))

def get_last_anomaly(key_hash):
    with db() as conn, conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as cur:
        cur.execute("""
            SELECT ae.trace_id, ae.risk_score, ae.details, ae.event_type
            FROM anomaly_events ae JOIN api_keys ak ON ae.api_key_id=ak.id
            WHERE ak.key_hash=%s ORDER BY ae.id DESC LIMIT 1""", (key_hash,))
        return cur.fetchone()

# ============================================================
# HTTP HELPERS
# ============================================================
def headers_for(key, lat, lon, ts, latency):
    payload = f"{lat}:{lon}:{ts}:{latency}"
    sig = hmac.new(key.encode(), payload.encode(), hashlib.sha256).hexdigest()
    return {
        "Authorization": f"Bearer {key}",
        "X-FCR-GPS-Lat": lat, "X-FCR-GPS-Lon": lon,
        "X-FCR-Timestamp": ts, "X-FCR-Latency": latency,
        "X-FCR-Signature": sig, "Content-Type": "application/json",
    }

def now_ts(offset_sec=0):
    return (datetime.now(timezone.utc) + timedelta(seconds=offset_sec)).isoformat()

def enc(key, text=NORMAL_TEXT, lat=NORMAL_LAT, lon=NORMAL_LON, ts=None, latency="50.0", tier="b", blinding=None):
    ts = ts or now_ts()
    body = {"key_name": KEY_NAME, "text_b64": base64.b64encode(text.encode()).decode(), "tier": tier}
    if blinding is not None: body["blinding"] = blinding
    return requests.post(f"{API_URL}/encrypt", headers=headers_for(key, lat, lon, ts, latency), json=body)

def dec(key, ct_b64, lat=NORMAL_LAT, lon=NORMAL_LON, ts=None, latency="50.0", tier="b"):
    ts = ts or now_ts()
    body = {"key_name": KEY_NAME, "ciphertext_b64": ct_b64, "tier": tier}
    return requests.post(f"{API_URL}/decrypt", headers=headers_for(key, lat, lon, ts, latency), json=body)

def get_decrypted_text(key, ct_b64):
    """Mendekripsi ciphertext dan mengembalikan string mentah."""
    r = dec(key, ct_b64)
    if r.status_code != 200: 
        return None
    try:
        return base64.b64decode(r.json()["text_b64"]).decode()
    except Exception:
        return None

def decrypt_to_json(key, ct_b64):
    """Mendekripsi ciphertext dan mencoba parse sebagai JSON (untuk Honeypot)."""
    text = get_decrypted_text(key, ct_b64)
    if not text: 
        return None
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        return None  # Bukan JSON, kembalikan None (test akan handle ini)

def is_honeypot(resp):
    """Honeypot terpicu = status 200 TAPI blocks jauh > 42 (payload forensik)."""
    return resp.status_code == 200 and resp.json().get("blocks", 0) > 100

# ============================================================
# KATEGORI A — KRIPTOGRAFI INTI
# ============================================================
section("A. KRIPTOGRAFI INTI (KEM + AEAD + IND-CPA + Tampering)")

# A1: Round-trip (Gunakan get_decrypted_text, BUKAN decrypt_to_json)
k,_,_ = make_api_key("A1")
r = enc(k)
ct = r.json()["ciphertext_b64"]
decrypted_text = get_decrypted_text(k, ct)
t("A1 round-trip encrypt→decrypt==original",
  r.status_code==200 and decrypted_text == NORMAL_TEXT,
  f"got: {repr(decrypted_text)}")

# A2: IND-CPA
k,_,_ = make_api_key("A2")
c1 = enc(k).json()["ciphertext_b64"]; c2 = enc(k).json()["ciphertext_b64"]
t("A2 IND-CPA: ciphertext acak per-opsi", c1 != c2, f"c1==c2? {c1==c2}")

# A3: Blinding determinism
k,_,_ = make_api_key("A3")
b = "deterministic-vector-001"
d1 = enc(k, blinding=b).json()["ciphertext_b64"]
d2 = enc(k, blinding=b).json()["ciphertext_b64"]
t("A3 Blinding determinism: blinding sama → ct sama", d1 == d2)

# A4: Tampering
k,_,_ = make_api_key("A4")
ct = enc(k).json()["ciphertext_b64"]
raw = bytearray(base64.b64decode(ct)); raw[len(raw)//2] ^= 0x01
tampered = base64.b64encode(bytes(raw)).decode()
rr = dec(k, tampered)
t("A4 Tampering ciphertext → reject (≠200)", rr.status_code != 200, f"got {rr.status_code}")

# A5: Wrong key decrypt
k,_,_ = make_api_key("A5")
ct = enc(k).json()["ciphertext_b64"]
subprocess.run(["cargo","run","--release","--","keygen","-N","wrong-key","-s","other-seed"],
               cwd=ENGINE_DIR, capture_output=True)
body = {"key_name":"wrong-key","ciphertext_b64":ct,"tier":"b"}
ts=now_ts(); payload=f"{NORMAL_LAT}:{NORMAL_LON}:{ts}:50.0"
sig=hmac.new(k.encode(),payload.encode(),hashlib.sha256).hexdigest()
hd={"Authorization":f"Bearer {k}","X-FCR-GPS-Lat":NORMAL_LAT,"X-FCR-GPS-Lon":NORMAL_LON,
    "X-FCR-Timestamp":ts,"X-FCR-Latency":"50.0","X-FCR-Signature":sig,"Content-Type":"application/json"}
rr = requests.post(f"{API_URL}/decrypt", headers=hd, json=body)
t("A5 Wrong-key decrypt → reject", rr.status_code != 200, f"got {rr.status_code}")

# A6: Format corruption
k,_,_ = make_api_key("A6")
bad = ":".join(ct.split(":")[:3])
t("A6 Ciphertext 3-bagian → reject", dec(k, base64.b64encode(bad.encode()).decode()).status_code != 200)

# A7: Empty plaintext
k,_,_ = make_api_key("A7")
r = enc(k, text="")
t("A7 Empty plaintext round-trip",
  r.status_code==200 and get_decrypted_text(k, r.json()["ciphertext_b64"])=="")

# A8: Oversized 10KB
k,_,_ = make_api_key("A8")
big = "X"*10000
r = enc(k, text=big)
t("A8 Oversized 10KB round-trip",
  r.status_code==200 and len(get_decrypted_text(k, r.json()["ciphertext_b64"]))==10000)

# ============================================================
# KATEGORI B — SPATIOTEMPORAL AUTH
# ============================================================
section("B. SPATIOTEMPORAL AUTH (HMAC + Header Validation)")
k,_,_ = make_api_key("B1")
t("B1 Missing Authorization → 401",
  requests.post(f"{API_URL}/encrypt", json={"key_name":KEY_NAME,"text_b64":"aGk=","tier":"b"}).status_code==401)

k,_,_ = make_api_key("B2")
ts=now_ts(); payload=f"{NORMAL_LAT}:{NORMAL_LON}:{ts}:50.0"
sig=hmac.new(k.encode(),payload.encode(),hashlib.sha256).hexdigest()
h={"Authorization":"Bearer ","X-FCR-GPS-Lat":NORMAL_LAT,"X-FCR-GPS-Lon":NORMAL_LON,
   "X-FCR-Timestamp":ts,"X-FCR-Latency":"50.0","X-FCR-Signature":sig,"Content-Type":"application/json"}
t("B2 Empty bearer → 401", requests.post(f"{API_URL}/encrypt",headers=h,json={"key_name":KEY_NAME,"text_b64":"aGk=","tier":"b"}).status_code==401)

k,_,_ = make_api_key("B3")
ts=now_ts(); payload=f"{NORMAL_LAT}:{NORMAL_LON}:{ts}:50.0"
sig=hmac.new(k.encode(),payload.encode(),hashlib.sha256).hexdigest()
h={"Authorization":f"Bearer {k}","X-FCR-GPS-Lon":NORMAL_LON,"X-FCR-Timestamp":ts,
   "X-FCR-Latency":"50.0","X-FCR-Signature":sig,"Content-Type":"application/json"}  # no Lat
t("B3 Missing GPS-Lat → 400", requests.post(f"{API_URL}/encrypt",headers=h,json={"key_name":KEY_NAME,"text_b64":"aGk=","tier":"b"}).status_code==400)

k,_,_ = make_api_key("B4")
ts=now_ts(); h={"Authorization":f"Bearer {k}","X-FCR-GPS-Lat":NORMAL_LAT,"X-FCR-GPS-Lon":NORMAL_LON,
   "X-FCR-Timestamp":ts,"X-FCR-Latency":"50.0","Content-Type":"application/json"}  # no sig
t("B4 Missing Signature → 400", requests.post(f"{API_URL}/encrypt",headers=h,json={"key_name":KEY_NAME,"text_b64":"aGk=","tier":"b"}).status_code==400)

k,_,_ = make_api_key("B5")
ts=now_ts(); h={"Authorization":f"Bearer {k}","X-FCR-GPS-Lat":NORMAL_LAT,"X-FCR-GPS-Lon":NORMAL_LON,
   "X-FCR-Timestamp":ts,"X-FCR-Latency":"50.0","X-FCR-Signature":"deadbeef"*8,"Content-Type":"application/json"}
t("B5 Forged HMAC signature → 401", requests.post(f"{API_URL}/encrypt",headers=h,json={"key_name":KEY_NAME,"text_b64":"aGk=","tier":"b"}).status_code==401)

k,_,_ = make_api_key("B6")
ts="bukan-rfc3339"; payload=f"{NORMAL_LAT}:{NORMAL_LON}:{ts}:50.0"
sig=hmac.new(k.encode(),payload.encode(),hashlib.sha256).hexdigest()
h={"Authorization":f"Bearer {k}","X-FCR-GPS-Lat":NORMAL_LAT,"X-FCR-GPS-Lon":NORMAL_LON,
   "X-FCR-Timestamp":ts,"X-FCR-Latency":"50.0","X-FCR-Signature":sig,"Content-Type":"application/json"}
t("B6 Malformed timestamp → 400", requests.post(f"{API_URL}/encrypt",headers=h,json={"key_name":KEY_NAME,"text_b64":"aGk=","tier":"b"}).status_code==400)

k,_,_ = make_api_key("B7")
ts=now_ts(); payload=f"abc:{NORMAL_LON}:{ts}:50.0"
sig=hmac.new(k.encode(),payload.encode(),hashlib.sha256).hexdigest()
h={"Authorization":f"Bearer {k}","X-FCR-GPS-Lat":"abc","X-FCR-GPS-Lon":NORMAL_LON,
   "X-FCR-Timestamp":ts,"X-FCR-Latency":"50.0","X-FCR-Signature":sig,"Content-Type":"application/json"}
t("B7 Non-float lat → 400", requests.post(f"{API_URL}/encrypt",headers=h,json={"key_name":KEY_NAME,"text_b64":"aGk=","tier":"b"}).status_code==400)

# ============================================================
# KATEGORI C — AUTO-MUNE SCORING (multi-faktor)
# ============================================================
section("C. AUTO-MUNE RISK SCORING (Latency/Drift/Zone/First)")
# C1 Extreme latency >1500 → +80 → honeypot
k,_,_ = make_api_key("C1")
r = enc(k, latency="2000.0")
t("C1 Latency>1500ms → honeypot", is_honeypot(r), f"blocks={r.json().get('blocks')}")

# C2 High latency 900 → +20 → TIDAK honeypot
k,_,_ = make_api_key("C2")
r = enc(k, latency="900.0")
t("C2 Latency 900ms → normal (no honeypot)", r.status_code==200 and not is_honeypot(r), f"blocks={r.json().get('blocks')}")

# C3 Timestamp drift -400s → +30 → TIDAK honeypot
k,_,_ = make_api_key("C3")
enc(k, ts=now_ts(0))                 # set last_state
r = enc(k, ts=now_ts(-400))          # drift
t("C3 Timestamp drift -400s → normal", r.status_code==200 and not is_honeypot(r))

# C4 Impossible zone jump <60s → +75 → honeypot
k,_,_ = make_api_key("C4")
ts1=now_ts(0); enc(k, ts=ts1)        # grid Jakarta
r = enc(k, lat=NY_LAT, lon=NY_LON, ts=now_ts(3))  # grid NY, dt=3
t("C4 Impossible zone jump → honeypot", is_honeypot(r), f"blocks={r.json().get('blocks')}")

# C5 Rapid transition 120s → +30 → TIDAK honeypot
# C5 Rapid transition 120s → +30 → TIDAK honeypot
k, _, _ = make_api_key("C5")
enc(k, ts=now_ts(-120))  # Set state 120 detik lalu (bukan masa depan!)
r = enc(k, lat=NY_LAT, lon=NY_LON, ts=now_ts(0))
t("C5 Rapid transition 120s → normal", r.status_code == 200 and not is_honeypot(r))

# C6 First request → +5 → normal
k,_,_ = make_api_key("C6")
r = enc(k)
t("C6 First request → normal", r.status_code==200 and not is_honeypot(r))

# C7 Combination: high latency + zone jump → 20+75=95 → honeypot
k,_,_ = make_api_key("C7")
enc(k, ts=now_ts(0))
r = enc(k, lat=NY_LAT, lon=NY_LON, ts=now_ts(2), latency="900.0")
t("C7 Combination → honeypot", is_honeypot(r))

# ============================================================
# KATEGORI D — HONEYPOT & FORENSICS (end-to-end)
# ============================================================
section("D. HONEYPOT + FORENSIC CONSISTENCY + COOLDOWN")
k, kh, kid = make_api_key("D1")
enc(k, ts=now_ts(0))
r = enc(k, lat=NY_LAT, lon=NY_LON, ts=now_ts(2))
ct = r.json()["ciphertext_b64"]
# D1 deceptive = 200 (bukan 401/403)
t("D1 Honeypot = deceptive 200 (bukan block)", r.status_code==200)
# D2 payload forensik lebih panjang
t("D2 Honeypot blocks > normal", r.json()["blocks"]>100, f"blocks={r.json()['blocks']}")
# D3 decrypt honeypot → JSON HONEYPOT_TRIGGERED (pakai key FRESH lain untuk auth decrypt)
k2,_,_ = make_api_key("D3")
fp = decrypt_to_json(k2, ct)
t("D3 Honeypot decrypt → status HONEYPOT_TRIGGERED",
  fp is not None and fp.get("status")=="HONEYPOT_TRIGGERED", f"got={fp}")
# D4 forensic consistency: trace_id ciphertext == trace_id DB (untuk key D1)
db_anom = get_last_anomaly(kh)
t("D4 Forensic consistency: trace_id ct == trace_id DB",
  fp and db_anom and fp.get("trace_id")==db_anom["trace_id"],
  f"ct={fp.get('trace_id') if fp else None} db={db_anom['trace_id'] if db_anom else None}")
t("D5 DB mencatat risk_score & reason",
  db_anom and db_anom["risk_score"]>=70 and "zone jump" in (db_anom["details"] or ""),
  f"score={db_anom['risk_score'] if db_anom else None}")
# D6 Cooldown bypass: request lagi dengan key D1 (yang cooldown) → 401
r2 = enc(k, ts=now_ts(100))  # koordinat normal, tapi key masih cooldown
t("D6 Cooldown active → 401", r2.status_code==401, f"got={r2.status_code}")
# D7 Setelah reset cooldown → bisa 200 lagi
reset_cooldown(kh)
r3 = enc(k, ts=now_ts(0))
t("D7 After cooldown reset → 200", r3.status_code==200, f"got={r3.status_code}")

# ============================================================
# KATEGORI E — API SURFACE
# ============================================================
section("E. API SURFACE (404/400/Tier/Revoke)")
t("E1 /health → 200 status ok",
  requests.get(f"{API_URL}/health").json().get("status")=="ok")
k,_,_ = make_api_key("E2")
r = requests.post(f"{API_URL}/encrypt", headers=headers_for(k,NORMAL_LAT,NORMAL_LON,now_ts(),"50.0"),
                  json={"key_name":"key-yang-tidak-ada","text_b64":"aGk=","tier":"b"})
t("E2 Unknown key_name → 404", r.status_code==404, f"got={r.status_code}")
k,_,_ = make_api_key("E3")
r = requests.post(f"{API_URL}/encrypt", headers=headers_for(k,NORMAL_LAT,NORMAL_LON,now_ts(),"50.0"),
                  json={"key_name":KEY_NAME,"text_b64":"!!!bukan-base64!!!","tier":"b"})
t("E3 Invalid base64 → 400", r.status_code==400, f"got={r.status_code}")
# E4 Tier A vs Tier B round-trip
k,_,_ = make_api_key("E4")
for tier in ["a","b"]:
    r = enc(k, tier=tier)
    ok = r.status_code==200 and base64.b64decode(dec(k, r.json()["ciphertext_b64"], tier=tier).json()["text_b64"]).decode()==NORMAL_TEXT
    t(f"E4 Tier {tier.upper()} round-trip", ok)
# E5 Revoke API key
k, kh, kid = make_api_key("E5")
t("E5a Key valid sebelum revoke → 200", enc(k).status_code==200)
with db() as conn, conn.cursor() as cur:
    cur.execute("UPDATE api_keys SET is_active=FALSE WHERE key_hash=%s", (kh,))
t("E5b After revoke → 401", enc(k).status_code==401)

# ==============================================================
#   F. STRESS & CONCURRENCY TESTING (Asyncio + aiohttp)
# ==============================================================
import asyncio
import time
import aiohttp


def get_stress_headers(
    api_key_plaintext, lat="-6.200000", lon="106.816666", latency="50.0"
):
  ts = now_ts()
  payload = f"{lat}:{lon}:{ts}:{latency}"
  sig = hmac.new(
      api_key_plaintext.encode(), payload.encode(), hashlib.sha256
  ).hexdigest()
  return {
      "Authorization": f"Bearer {api_key_plaintext}",
      "X-FCR-GPS-Lat": lat,
      "X-FCR-GPS-Lon": lon,
      "X-FCR-Timestamp": ts,
      "X-FCR-Latency": latency,
      "X-FCR-Signature": sig,
      "Content-Type": "application/json",
  }


async def stress_worker(session, url, endpoint, headers, payload):
  try:
    async with session.post(
        f"{url}/{endpoint}", json=payload, headers=headers
    ) as resp:
      return resp.status
  except Exception:
    return 0


def run_stress_tests():
  section("F. STRESS & CONCURRENCY TESTING (500+ Multi-Key Concurrent)")

  # Buat pool 25 API key agar tidak memicu false-positive rate limit 1-key saat stress test
  key_pool = [make_api_key(f"F_STRESS_{i}")[0] for i in range(25)]
  base_url = API_URL

  plaintext = "Stress test payload for FCR-RSA v0.6.0 concurrent execution"
  enc_payload = {
      "key_name": KEY_NAME,
      "text_b64": base64.b64encode(plaintext.encode()).decode(),
      "tier": "b",
  }

  r = requests.post(
      f"{base_url}/encrypt",
      json=enc_payload,
      headers=get_stress_headers(key_pool[0]),
  )
  if r.status_code != 200:
    t("F0 Prerequisite encrypt", False, f"Failed with {r.status_code}")
    return

  ct_data = r.json()
  dec_payload = {
      "key_name": KEY_NAME,
      "ciphertext_b64": ct_data["ciphertext_b64"],
      "tier": "b",
  }

  async def execute_stress():
    connector = aiohttp.TCPConnector(limit=100)
    timeout = aiohttp.ClientTimeout(total=20)
    async with aiohttp.ClientSession(
        connector=connector, timeout=timeout
    ) as session:

      # F1: 500 Concurrent Encrypts (distribusi across 25 keys)
      start = time.time()
      tasks = [
          stress_worker(
              session,
              base_url,
              "encrypt",
              get_stress_headers(key_pool[i % len(key_pool)]),
              enc_payload,
          )
          for i in range(500)
      ]
      results_enc = await asyncio.gather(*tasks)
      time_enc = time.time() - start
      success_enc = sum(1 for r in results_enc if r == 200)
      t(
          "F1 500x Concurrent Encrypt (25-key pool)",
          success_enc == 500,
          f"{success_enc}/500 success ({time_enc:.2f}s)",
      )

      # F2: 500 Concurrent Decrypts (distribusi across 25 keys)
      start = time.time()
      tasks = [
          stress_worker(
              session,
              base_url,
              "decrypt",
              get_stress_headers(key_pool[i % len(key_pool)]),
              dec_payload,
          )
          for i in range(500)
      ]
      results_dec = await asyncio.gather(*tasks)
      time_dec = time.time() - start
      success_dec = sum(1 for r in results_dec if r == 200)
      t(
          "F2 500x Concurrent Decrypt (25-key pool)",
          success_dec == 500,
          f"{success_dec}/500 success ({time_dec:.2f}s)",
      )

      # F3: 200 Concurrent Invalid Auth (Brute-force resilience)
      start = time.time()
      bad_headers = get_stress_headers("fcr_invalid_key_stress_test_0000")
      tasks = [
          stress_worker(session, base_url, "encrypt", bad_headers, enc_payload)
          for _ in range(200)
      ]
      results_bad = await asyncio.gather(*tasks)
      time_bad = time.time() - start
      auth_reject = sum(1 for r in results_bad if r == 401)
      server_error = sum(1 for r in results_bad if r >= 500)
      t(
          "F3 200x Concurrent Invalid Auth (No 5xx)",
          auth_reject == 200 and server_error == 0,
          f"{auth_reject}/200 rejected, {server_error} 5xx ({time_bad:.2f}s)",
      )

  asyncio.run(execute_stress())


# ==============================================================
#   G. ADVERSARIAL PROTOCOL & EDGE CASES (Audit Readiness)
# ==============================================================
def run_adversarial_edge_tests():
  section("G. ADVERSARIAL PROTOCOL & EDGE CASES (6 Assertions)")

  # G1: Future Timestamp Injection (+600 detik ke masa depan -> wajib 401/400 atau honeypot)
  k, _, _ = make_api_key("G1")
  r_fut = enc(k, ts=now_ts(600))
  is_safe_fut = (r_fut.status_code in (400, 401)) or is_honeypot(r_fut)
  t(
      "G1 Future timestamp (+600s) → blocked or honeypot",
      is_safe_fut,
      f"status={r_fut.status_code}, honeypot={is_honeypot(r_fut)}",
  )

  # G2: Bidirectional Cross-Tier Isolation (Enc A -> Dec B & Enc B -> Dec A)
  k, _, _ = make_api_key("G2")
  r_a = enc(k, tier="a")
  r_b = enc(k, tier="b")
  if r_a.status_code == 200 and r_b.status_code == 200:
    ct_a = r_a.json()["ciphertext_b64"]
    ct_b = r_b.json()["ciphertext_b64"]
    dec_a_as_b = dec(k, ct_a, tier="b")
    dec_b_as_a = dec(k, ct_b, tier="a")
    isolated = (dec_a_as_b.status_code != 200) and (
        dec_b_as_a.status_code != 200
    )
    t(
        "G2 Bidirectional Cross-Tier Isolation (A↔B) → reject",
        isolated,
        f"A->B={dec_a_as_b.status_code}, B->A={dec_b_as_a.status_code}",
    )
  else:
    t("G2 Bidirectional Cross-Tier Isolation", False, "Initial encrypt failed")

  # G3: Out-of-bounds & Non-Finite GPS Coordinates (999.9 & NaN)
  k, _, _ = make_api_key("G3")
  r_gps_oob = enc(k, lat="999.9999", lon="-999.9999")
  r_gps_nan = enc(k, lat="NaN", lon="106.8456")
  t(
      "G3 Invalid GPS bounds (999.9 & NaN) → reject (400)",
      r_gps_oob.status_code == 400 and r_gps_nan.status_code == 400,
      f"oob={r_gps_oob.status_code}, nan={r_gps_nan.status_code}",
  )

  # G4: Negative & Infinite Latency Injection (Spoofed telemetry)
  k, _, _ = make_api_key("G4")
  r_lat_neg = enc(k, latency="-50.0")
  r_lat_inf = enc(k, latency="inf")
  t(
      "G4 Negative/Infinite latency (-50.0 & inf) → reject (400)",
      r_lat_neg.status_code == 400 and r_lat_inf.status_code == 400,
      f"neg={r_lat_neg.status_code}, inf={r_lat_inf.status_code}",
  )

  # G5: Unicode, Control Char & Emoji UTF-8 Payload Integrity
  k, _, _ = make_api_key("G5")
  nasty_payload = "Rahasia\x00NullByte\x07\x1b[31m🔐🚀漢字"
  r_uni = enc(k, text=nasty_payload)
  dec_uni = (
      get_decrypted_text(k, r_uni.json()["ciphertext_b64"])
      if r_uni.status_code == 200
      else None
  )
  t("G5 Null-byte, ANSI & Emoji UTF-8 round-trip", dec_uni == nasty_payload)

  # G6: Invalid Tier Parameter (Empty & Unknown)
  k, _, _ = make_api_key("G6")
  r_bad_tier = enc(k, tier="tier_dewa")
  r_empty_tier = enc(k, tier="   ")
  t(
      "G6 Unknown/Empty tier parameter → reject (400)",
      r_bad_tier.status_code == 400 and r_empty_tier.status_code == 400,
      f"unknown={r_bad_tier.status_code}, empty={r_empty_tier.status_code}",
  )

# ==============================================================
#  EKSEKUSI UTAMA & SUMMARY FINAL
# ==============================================================
if __name__ == "__main__":
  run_stress_tests()
  run_adversarial_edge_tests()

  print("\n==============================================================")
  print("  📊 BRUTAL TEST SUMMARY (45 AUDIT-GRADE ASSERTIONS)")
  print("==============================================================")

  total = RESULTS["PASS"] + RESULTS["FAIL"]
  print(f"  Total : {total} assertions")
  print(f"  PASS  : {RESULTS['PASS']}")
  print(f"  FAIL  : {RESULTS['FAIL']}")

  if RESULTS["FAIL"]:
    print("\n❌ YANG GAGAL:")
    for ok, name, note in RESULTS["DETAILS"]:
      if not ok:
        print(f"     - {name}  ({note})")
    sys.exit(1)

  print(
      "\n  🟢 SEMUA 45 KLAIM DESAIN FCR-RSA TERVERIFIKASI SECARA BRUTAL,"
      " CONCURRENT & ADVERSARIAL."
  )
  sys.exit(0)
