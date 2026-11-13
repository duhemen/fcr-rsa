"""Auto-imun: spatiotemporal token + anomaly detection (prototype)."""
from dataclasses import dataclass
from datetime import datetime, timedelta


@dataclass
class SpatiotemporalToken:
    gps_lat: float
    gps_lon: float
    timestamp: datetime
    latency_ms: float


def check_anomaly(
    token: SpatiotemporalToken,
    last_token: SpatiotemporalToken | None,
    max_speed_kmh: float = 900.0,
    max_latency_ms: float = 500.0,
) -> list[str]:
    """Return list of anomaly reasons (empty if OK)."""
    reasons: list[str] = []
    if last_token is None:
        return reasons

    dt = (token.timestamp - last_token.timestamp).total_seconds()
    if dt <= 0:
        reasons.append("non-monotonic timestamp")
        return reasons

    # Haversine (approx)
    from math import radians, sin, cos, asin, sqrt
    r = 6371.0
    lat1, lon1 = radians(last_token.gps_lat), radians(last_token.gps_lon)
    lat2, lon2 = radians(token.gps_lat), radians(token.gps_lon)
    dlat, dlon = lat2 - lat1, lon2 - lon1
    a = sin(dlat/2)**2 + cos(lat1)*cos(lat2)*sin(dlon/2)**2
    dist_km = 2 * r * asin(sqrt(a))
    speed_kmh = dist_km / (dt / 3600.0) if dt else 0.0
    if speed_kmh > max_speed_kmh:
        reasons.append(f"speed anomaly: {speed_kmh:.1f} km/h")

    if token.latency_ms > max_latency_ms:
        reasons.append(f"latency anomaly: {token.latency_ms:.1f} ms")

    return reasons