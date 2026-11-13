"""Tier A: server-side, large index (prototype)."""
from .core_math import generate_fibonacci


def encrypt(message: int, seed: bytes, n: int, q: int, blinding: int) -> int:
    if n < 4:
        raise ValueError("n must be >= 4")
    seq = generate_fibonacci(n, seed, q)
    e_vec = seq[n - 2]
    n_comp = seq[n - 1]
    return ((message * e_vec) + n_comp + blinding) % q


def decrypt(ciphertext: int, seed: bytes, n: int, q: int, blinding: int) -> int:
    if n < 4:
        raise ValueError("n must be >= 4")
    seq = generate_fibonacci(n, seed, q)
    e_vec = seq[n - 2]
    n_comp = seq[n - 1]
    c = (ciphertext - n_comp - blinding) % q
    inv = pow(e_vec, -1, q)
    return (c * inv) % q