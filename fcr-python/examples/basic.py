"""Basic example."""
from fcr_rsa.tier_b import encrypt_with_blinding, decrypt

q = 1_000_000_007
seed = b"fcr-rsa-demo-seed"
n = 10
blinding = 12345
msg = 42

print("Plaintext :", msg)
ct = encrypt_with_blinding(msg, seed, n, q, blinding)
print("Ciphertext:", ct)
pt = decrypt(ct, seed, n, q, blinding)
print("Decrypted :", pt)
assert msg == pt, "round-trip failed"
print("Round-trip OK")
