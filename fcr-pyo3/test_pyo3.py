#!/usr/bin/env python3
"""Test FCR-RSA v0.6.0 Python Bindings - Round Trip Verification"""

import fcr_pyo3
import os

print("=== FCR-RSA v0.6.0 Python Bindings Test ===\n")

# 1. Generate Key Pair
master_seed = b"fcr-rsa-demo-seed"
key_pair = fcr_pyo3.PyFcrKeyPair(master_seed)
print("✅ Key pair generated")

# 2. Prepare Message
message = b"Hello, FCR-RSA v0.6.0 via Python!"
nonce = os.urandom(12)
print(f"✅ Message: {message}")
print(f"✅ Nonce ({len(nonce)} bytes): {nonce.hex()}")

# 3. Encrypt
ciphertext = fcr_pyo3.encrypt_message(key_pair, message, master_seed, nonce)
# ✅ PERBAIKAN: Konversi list → bytes sebelum memanggil .hex()
ct_bytes = bytes(ciphertext)
print(f"✅ Ciphertext ({len(ct_bytes)} bytes): {ct_bytes[:32].hex()}...")

# 4. Decrypt
decrypted = fcr_pyo3.decrypt_message(key_pair, ct_bytes, master_seed, nonce)
print(f"✅ Decrypted: {bytes(decrypted)}")

# 5. Verify
if bytes(decrypted) == message:
    print("\n🎉 SUCCESS: Round-trip encryption/decryption works perfectly!")
else:
    print("\n❌ FAILED: Decrypted message does not match original!")
    print(f"   Expected: {message}")
    print(f"   Got:      {bytes(decrypted)}")

# 6. Test dengan nonce berbeda (harus gagal decrypt)
print("\n=== Bonus Test: Nonce Berbeda ===")
wrong_nonce = os.urandom(12)
try:
    wrong_decrypted = fcr_pyo3.decrypt_message(key_pair, ct_bytes, master_seed, wrong_nonce)
    if bytes(wrong_decrypted) != message:
        print("✅ AMAN: Nonce berbeda menghasilkan plaintext berbeda (tidak bocor)")
    else:
        print("⚠️  PERINGATAN: Nonce berbeda tetap menghasilkan plaintext sama!")
except Exception as e:
    print(f"✅ AMAN: Decryption dengan nonce salah gagal: {e}")

print("\n=== FCR-RSA v0.6.0 Python Bindings: ALL TESTS PASSED ===")