class AutoImmuneSystem:
    def __init__(self):
        pass

    def check_transaction(self, device_gps, transaction_time, expected_gps, allowed_hours, network_delay_ms):
        """Mendeteksi Anomali Spatiotemporal dan Serangan MITM"""
        
        print(f"\n[AUTO-IMUN] Memindai Transaksi...")
        print(f" -> Info: GPS={device_gps}, Jam={transaction_time}:00, Delay Jaringan={network_delay_ms}ms")
        
        # A. Deteksi Anomali Lokasi (Rekayasa Sosial)
        gps_diff = abs(device_gps - expected_gps)
        if gps_diff > 5.0:
            return "LOCKDOWN", "🚨 ANOMALI LOKASI: Kunci cabang diduga bocor!"
            
        # B. Deteksi Anomali Waktu
        if transaction_time < allowed_hours[0] or transaction_time > allowed_hours[1]:
            return "LOCKDOWN", "🚨 ANOMALI WAKTU: Transaksi di luar jam operasional!"
            
        # C. Deteksi Serangan MITM (Berdasarkan Latensi)
        if network_delay_ms > 150:
            return "LOCKDOWN", "🚨 DETEKSI MITM: Latensi jaringan tidak wajar! Fase Kompleks tidak sinkron."
            
        return "AMAN", "✅ Transaksi Terverifikasi (Aman & Sinkron)."