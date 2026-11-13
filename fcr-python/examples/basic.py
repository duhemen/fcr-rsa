"""Basic example."""
from fcr_rsa import encrypt_tier_b, decrypt_tier_b

q = 1_000_000_007
seed = b"fcr-rsa-demo-seed"
n = 10
blinding = 12345
msg = 42

print("Plaintext :", msg)
ct = encrypt_tier_b(msg, seed, n, q, blinding)
print("Ciphertext:", ct)
pt = decrypt_tier_b(ct, seed, n, q, blinding)
print("Decrypted :", pt)
assert msg == pt, "round-trip failed"
print("Round-trip OK")