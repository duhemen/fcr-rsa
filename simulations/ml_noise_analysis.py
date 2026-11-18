#!/usr/bin/env python3
"""
FCR-RSA v0.6.0 - Noise Stream ML Analysis
Membuktikan bahwa ciphertext tidak memiliki pola yang dapat dipelajari 
oleh model Machine Learning untuk memprediksi properti pesan (Light vs Heavy).
"""

import os
import hashlib
import hmac
import numpy as np
from sklearn.ensemble import RandomForestClassifier
from sklearn.model_selection import train_test_split
from sklearn.metrics import accuracy_score

# Konstanta Q 2048-bit (RFC 3526 MODP Group 14)
Q_HEX = (
    "FFFFFFFFFFFFFFFFC90FDAA22168C234C4C6628B80DC1CD129024E088A67CC74"
    "020BBEA63B139B22514A08798E3404DDEF9519B3CD3A431B302B0A6DF25F1437"
    "4FE1356D6D51C245E485B576625E7EC6F44C42E9A637ED6B0BFF5CB6F406B7ED"
    "EE386BFB5A899FA5AE9F24117C4B1FE649286651ECE45B3DC2007CB8A163BF05"
    "98DA48361C55D39A69163FA8FD24CF5F83655D23DCA3AD961C62F356208552BB"
    "9ED529077096966D670C354E4ABC9804F1746C08CA18217C32905E462E36CE3B"
    "E39E772C180E86039B2783A2EC07A28FB5C55DF06F4C52C9DE2BCBF695581718"
    "3995497CEA956AE515D2261898FA051015728E5A8AACAA68FFFFFFFFFFFFFFFF"
)
Q = int(Q_HEX, 16)

def hkdf_expand(prk: bytes, info: bytes, length: int) -> bytes:
    t = b""
    okm = b""
    for i in range(1, (length // 32) + 2):
        t = hmac.new(prk, t + info + bytes([i]), hashlib.sha256).digest()
        okm += t
    return okm[:length]

def generate_ml_dataset(num_samples=1000):
    print(f"[1] Generating {num_samples} synthetic ciphertexts (v0.6.0 logic)...")
    
    X = [] # Features (Ciphertext bytes)
    y = [] # Labels (0 = Light, 1 = Heavy)
    
    for i in range(num_samples):
        master_seed = os.urandom(32)
        nonce = os.urandom(12)
        
        # 50% Light, 50% Heavy
        is_heavy = np.random.rand() > 0.5
        if is_heavy:
            msg_bytes = bytes([0xFF] * 32)
            label = 1
        else:
            msg_bytes = bytes([0x01] + [0x00] * 31)
            label = 0
            
        msg_int = int.from_bytes(msg_bytes, 'big') % Q
        
        # Simulasi generate_key_pair (E_vec)
        prk_e = hmac.new(b'', master_seed, hashlib.sha256).digest()
        e_vec_bytes = hkdf_expand(prk_e, b"fcr-rsa-e-vec-v1", 256)
        e_vec = int.from_bytes(e_vec_bytes, 'big') % Q
        
        # Simulasi derive_blinding
        ikm = master_seed + nonce
        prk_b = hmac.new(b'', ikm, hashlib.sha256).digest()
        blinding_bytes = hkdf_expand(prk_b, b"fcr-rsa-blinding-v1", 256)
        blinding = int.from_bytes(blinding_bytes, 'big') % Q
        
        # Enkripsi: C = (M * E_vec + Blinding) mod Q
        ciphertext = (msg_int * e_vec + blinding) % Q
        ct_bytes = ciphertext.to_bytes(256, 'big')
        
        X.append(list(ct_bytes))
        y.append(label)
        
    return np.array(X), np.array(y)

def run_ml_analysis():
    print("="*70)
    print("🧠 FCR-RSA v0.6.0 - NOISE STREAM ML ANALYSIS (Random Forest)")
    print("="*70)
    
    X, y = generate_ml_dataset(num_samples=1000)
    
    print("\n[2] Dataset Shape:")
    print(f"    Features (X): {X.shape}  (256 bytes per ciphertext)")
    print(f"    Labels (y):   {np.bincount(y)} (0=Light, 1=Heavy)")
    
    # Split data 70% train, 30% test
    X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.3, random_state=42)
    
    print("\n[3] Training Random Forest Classifier (100 trees, max_depth=10)...")
    clf = RandomForestClassifier(n_estimators=100, max_depth=10, random_state=42, n_jobs=-1)
    clf.fit(X_train, y_train)
    
    print("\n[4] Evaluating Model on Test Set...")
    y_pred = clf.predict(X_test)
    accuracy = accuracy_score(y_test, y_pred)
    
    print(f"\n📊 HASIL:")
    print(f"    Model Accuracy : {accuracy:.4f}")
    print(f"    Random Guessing: 0.5000")
    
    print("\n" + "="*70)
    if accuracy < 0.55:
        print("✅ AMAN: Model ML GAGAL mempelajari pola dari ciphertext.")
        print("   Akurasi mendekati 50% (tebakan acak), membuktikan tidak ada")
        print("   korelasi yang dapat dieksploitasi antara ciphertext dan tipe pesan.")
        print("   FCR-RSA v0.6.0 TAHAN terhadap ML-based Noise Analysis.")
    else:
        print("🚨 BAHAYA: Model ML berhasil mempelajari pola!")
        print(f"   Akurasi {accuracy:.4f} > 0.55 menunjukkan ada kebocoran informasi")
        print("   dalam noise stream yang bisa diprediksi oleh AI.")
    print("="*70)

if __name__ == "__main__":
    run_ml_analysis()