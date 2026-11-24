"""Basic tests."""
from fcr_rsa import encrypt_tier_a, decrypt_tier_a, decrypt_tier_b
from fcr_rsa.tier_b import encrypt_with_blinding


def test_tier_b_round_trip():
    q = 1_000_000_007
    seed = b"test-seed"
    n = 10
    m = 123
    blinding = 54321
    c = encrypt_with_blinding(m, seed, n, q, blinding)
    d = decrypt_tier_b(c, seed, n, q, blinding)
    assert m == d


def test_tier_a_round_trip():
    q = 1_000_000_007
    seed = b"test-seed"
    n = 10
    m = 456
    blinding = 54321
    c = encrypt_tier_a(m, seed, n, q, blinding)
    d = decrypt_tier_a(c, seed, n, q, blinding)
    assert m == d


def test_invalid_n():
    import pytest
    q = 1_000_000_007
    seed = b"test"
    with pytest.raises(ValueError):
        encrypt_with_blinding(1, seed, 2, q, 123)
