import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

# Arahkan path ke direktori 'fcr-python' agar paket 'fcr_rsa' dapat diimpor
ROOT_DIR = Path(__file__).resolve().parent.parent
FCR_PYTHON_DIR = ROOT_DIR / "fcr-python"

if str(FCR_PYTHON_DIR) not in sys.path:
    sys.path.insert(0, str(FCR_PYTHON_DIR))

from fcr_rsa import tier_a, tier_b
from fcr_rsa.auto_immune import SpatiotemporalToken, check_anomaly


def main():
    print("=== SIMULASI FCR-RSA BRIDGE PROTOCOL (TIER A & TIER B) ===\n")

    # 1. Konfigurasi Parameter Kriptografi Bersama
    seed = b"fcr-rsa-shared-master-seed-2026"
    n = 16
    q = 104729  # Bilangan prima modulus
    pesan_transaksi = 42069

    # 2. Tier B (HP Nasabah) Melakukan Enkripsi Session Token
    blinding_b = tier_b.generate_random_blinding(q)
    cipher_b = tier_b.encrypt_with_blinding(pesan_transaksi, seed, n, q, blinding_b)
    print(f"[Tier B - Nasabah] Pesan asli: {pesan_transaksi} -> Ciphertext: {cipher_b}")

    # 3. Tier A (Server Bank) Melakukan Enkripsi & Dekripsi Verifikasi
    blinding_a = 12345
    cipher_a = tier_a.encrypt(pesan_transaksi, seed, n, q, blinding_a)
    decrypted_a = tier_a.decrypt(cipher_a, seed, n, q, blinding_a)
    decrypted_b = tier_b.decrypt(cipher_b, seed, n, q, blinding_b)

    print("\n--- [JEMBATAN KOMUNIKASI] ---")
    if decrypted_a == pesan_transaksi and decrypted_b == pesan_transaksi:
        print("✅ Bank memverifikasi kriptografi Tier A & Tier B valid. Transaksi diizinkan.")

        # 4. Sistem Auto-Imun Memeriksa Konteks Spatiotemporal
        waktu_awal = datetime.now(timezone.utc)
        token_sebelumnya = SpatiotemporalToken(
            gps_lat=-6.2088,  # Jakarta
            gps_lon=106.8456,
            timestamp=waktu_awal,
            latency_ms=45.0,
        )

        # Skenario A: Transaksi Normal (10 menit kemudian, lokasi bergeser wajar)
        print("\n[Simulasi Transaksi Normal]")
        token_normal = SpatiotemporalToken(
            gps_lat=-6.2150,
            gps_lon=106.8500,
            timestamp=waktu_awal + timedelta(minutes=10),
            latency_ms=50.0,
        )
        anomali_a = check_anomaly(token_normal, token_sebelumnya)
        print(f"Status: {'AMAN' if not anomali_a else anomali_a}")

        # Skenario B: Serangan MITM / Impossible Travel (1 menit kemudian pindah ke Surabaya & latency tinggi)
        print("\n[Simulasi Serangan MITM / Anomali Lokasi]")
        token_serangan = SpatiotemporalToken(
            gps_lat=-7.2575,  # Surabaya (~660 km dalam 1 menit)
            gps_lon=112.7521,
            timestamp=waktu_awal + timedelta(minutes=1),
            latency_ms=650.0,  # Melebihi batas 500 ms
        )
        anomali_b = check_anomaly(token_serangan, token_sebelumnya)
        print(f"Status: {'AMAN' if not anomali_b else f'DITOLAK -> {anomali_b}'}")
    else:
        print("❌ Verifikasi kriptografi gagal. Transaksi ditolak.")


if __name__ == "__main__":
    main()
