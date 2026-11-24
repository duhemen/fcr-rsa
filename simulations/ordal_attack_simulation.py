import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

# Arahkan path ke direktori 'fcr-python' agar paket 'fcr_rsa' dapat diimpor
ROOT_DIR = Path(__file__).resolve().parent.parent
FCR_PYTHON_DIR = ROOT_DIR / "fcr-python"

if str(FCR_PYTHON_DIR) not in sys.path:
    sys.path.insert(0, str(FCR_PYTHON_DIR))

from fcr_rsa.auto_immune import SpatiotemporalToken, check_anomaly


def main():
    print("=== SIMULASI SERANGAN ORDAL (INSIDER THREAT) FCR-RSA ===\n")

    waktu_awal = datetime.now(timezone.utc)

    # 1. Aktivitas Sah Admin dari Kantor Pusat (Jakarta)
    token_admin_sah = SpatiotemporalToken(
        gps_lat=-6.2088,  # Jakarta
        gps_lon=106.8456,
        timestamp=waktu_awal,
        latency_ms=35.0,
    )
    print("[1] Admin sah login dari Kantor Pusat Jakarta (09:00 UTC).")

    # 2. Skenario Serangan Ordal A: Kredensial dicuri/digunakan bersamaan dari lokasi berbeda (Impossible Travel)
    print("\n[2] Skenario Ordal A: Penggunaan kunci oleh oknum di Singapura 2 menit kemudian...")
    token_ordal_lokasi = SpatiotemporalToken(
        gps_lat=1.3521,  # Singapura (~890 km dari Jakarta)
        gps_lon=103.8198,
        timestamp=waktu_awal + timedelta(minutes=2),
        latency_ms=60.0,
    )
    hasil_a = check_anomaly(token_ordal_lokasi, token_admin_sah)
    if hasil_a:
        print(f"🛡️ Sistem Auto-Imun MEMBLOKIR akses ordal: {hasil_a}")
    else:
        print("⚠️ Akses lolos!")

    # 3. Skenario Serangan Ordal B: Replay / Timestamp Mundur (Non-monotonic)
    print("\n[3] Skenario Ordal B: Oknum mencoba memanipulasi timestamp transaksi...")
    token_ordal_replay = SpatiotemporalToken(
        gps_lat=-6.2090,
        gps_lon=106.8460,
        timestamp=waktu_awal - timedelta(seconds=10),  # Timestamp mundur
        latency_ms=40.0,
    )
    hasil_b = check_anomaly(token_ordal_replay, token_admin_sah)
    if hasil_b:
        print(f"🛡️ Sistem Auto-Imun MEMBLOKIR manipulasi waktu: {hasil_b}")
    else:
        print("⚠️ Akses lolos!")


if __name__ == "__main__":
    main()
