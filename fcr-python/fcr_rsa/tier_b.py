"""Tier B: end-user / IoT, lightweight (prototype)."""
import secrets
from .core_math import generate_fibonacci


def generate_random_blinding(q: int) -> int:
    """Generate a cryptographically secure random blinding factor."""
    return secrets.randbelow(q)


def encrypt(message: int, seed: bytes, n: int, q: int) -> int:
    """
    Fungsi Produksi: R_blind dibangkitkan secara acak (IND-CPA aman).
    """
    if n < 4:
        raise ValueError("n must be >= 4")
    blinding = generate_random_blinding(q)
    return encrypt_with_blinding(message, seed, n, q, blinding)


def encrypt_with_blinding(message: int, seed: bytes, n: int, q: int, blinding: int) -> int:
    """
    Fungsi Deterministik: Digunakan untuk test vector & verifikasi.
    """
    if n < 4:
        raise ValueError("n must be >= 4")
    seq = generate_fibonacci(n, seed, q)
    e_vec = seq[n - 2]
    return ((message * e_vec) + blinding) % q


def decrypt(ciphertext: int, seed: bytes, n: int, q: int, blinding: int) -> int:
    """Dekripsi membutuhkan blinding yang sama dengan saat enkripsi."""
    if n < 4:
        raise ValueError("n must be >= 4")
    seq = generate_fibonacci(n, seed, q)
    e_vec = seq[n - 2]
    c = (ciphertext - blinding) % q
    inv = pow(e_vec, -1, q)
    return (c * inv) % q