"""Basic tests."""
from fcr_rsa import encrypt_tier_a, decrypt_tier_a, encrypt_tier_b, decrypt_tier_b


def test_tier_b_round_trip():
    q = 1_000_000_007
    seed = b"test-seed"
    n = 10
    b = 7
    m = 123
    c = encrypt_tier_b(m, seed, n, q, b)
    d = decrypt_tier_b(c, seed, n, q, b)
    assert m == d


def test_tier_a_round_trip():
    q = 1_000_000_007
    seed = b"test-seed"
    n = 10
    b = 7
    m = 456
    c = encrypt_tier_a(m, seed, n, q, b)
    d = decrypt_tier_a(c, seed, n, q, b)
    assert m == d


def test_invalid_n():
    import pytest
    q = 1_000_000_007
    seed = b"test"
    with pytest.raises(ValueError):
        encrypt_tier_b(1, seed, 2, q, 1)