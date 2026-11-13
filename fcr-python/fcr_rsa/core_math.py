"""Core math: non-linear Fibonacci + deterministic LWE noise."""
import hashlib


def noise_from_seed(seed: bytes, counter: int) -> int:
    """Deterministic noise byte stream (32 bytes) from SHA-256(seed || counter)."""
    h = hashlib.sha256()
    h.update(seed)
    h.update(counter.to_bytes(8, "little"))
    return int.from_bytes(h.digest(), "little")


def generate_fibonacci(n: int, seed: bytes, q: int) -> list[int]:
    """F_n = (F_{n-1} * F_{n-2}) + e_n mod Q."""
    if n <= 0:
        return []
    seq = [1]
    if n == 1:
        return seq
    seq.append(1)
    counter = 0
    for _ in range(2, n):
        prod = seq[-1] * seq[-2]
        e = noise_from_seed(seed, counter) % q
        counter += 1
        seq.append((prod + e) % q)
    return seq