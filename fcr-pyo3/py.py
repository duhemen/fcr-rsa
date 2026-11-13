import fcr_pyo3

# Enkripsi
ct = fcr_pyo3.py_encrypt_tier_a(42, b'fcr-rsa-demo-seed', 10, 1000000007, 12345)
print("Rust (via Python) Ciphertext:", ct)

# Dekripsi
pt = fcr_pyo3.py_decrypt_tier_a(ct, b'fcr-rsa-demo-seed', 10, 1000000007, 12345)
print("Decrypted:", pt)