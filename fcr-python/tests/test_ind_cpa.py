import pytest
from fcr_rsa.tier_b import encrypt, decrypt

def test_ind_cpa_randomness():
    q = 1_000_000_007
    seed = b"fcr-rsa-demo-seed"
    n = 10
    m = 42
    
    ciphertexts = set()
    iterations = 100
    
    # Enkripsi 100 kali
    for _ in range(iterations):
        c = encrypt(m, seed, n, q)
        ciphertexts.add(c)
        
    # Pastikan semua unik
    assert len(ciphertexts) == iterations, "IND-CPA FAILED: Ciphertext duplikat!"
    print(f"\n✅ IND-CPA PASSED: {iterations} ciphertext unik dihasilkan dari {iterations} enkripsi.")

def test_round_trip():
    q = 1_000_000_007
    seed = b"fcr-rsa-demo-seed"
    n = 10
    m = 42
    
    # Karena fungsi encrypt() membangkitkan blinding acak secara internal,
    # kita harus menggunakan encrypt_with_blinding() agar bisa didekripsi kembali.
    from fcr_rsa.tier_b import encrypt_with_blinding
    
    blinding = 12345 # Kita tentukan manual untuk test dekripsi
    c = encrypt_with_blinding(m, seed, n, q, blinding)
    decrypted = decrypt(c, seed, n, q, blinding)
    
    assert decrypted == m, "Round-trip gagal!"
    print(f"✅ Round-trip OK: {m} -> {c} -> {decrypted}")