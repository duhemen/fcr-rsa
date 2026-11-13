"""FCR-RSA: post-quantum cryptographic protocol prototype.

WARNING: RESEARCH PROTOTYPE — NOT PRODUCTION READY.
"""
from .core_math import generate_fibonacci, noise_from_seed
from .tier_a import encrypt as encrypt_tier_a, decrypt as decrypt_tier_a
from .tier_b import encrypt as encrypt_tier_b, decrypt as decrypt_tier_b
from .auto_immune import SpatiotemporalToken, check_anomaly

__version__ = "0.1.0"
__all__ = [
    "generate_fibonacci",
    "noise_from_seed",
    "encrypt_tier_a",
    "decrypt_tier_a",
    "encrypt_tier_b",
    "decrypt_tier_b",
    "SpatiotemporalToken",
    "check_anomaly",
]