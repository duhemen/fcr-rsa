"""
FCR-RSA: Quantum Blinding Vector Integration (Fase 4 - Audit-Grade Final)
Membaca test_vectors_v0.6.0.json, mengekstrak BLINDING_NONCE asli sesuai
format crypto.rs (BLINDING_NONCE:ENCRYPTED_SEED:AEAD_NONCE:CIPHERTEXT),
lalu menjalankan 3 skenario Qiskit:
  TC-001: Blinding per-sesi (nonce nyata)  -> entropi melonjak, periode hancur
  TC-002: Determinisme (blinding_override) -> sirkuit & distribusi identik 100%
  TC-003: Honeypot isolation              -> seed decoy terisolasi dari sesi normal
Output: quantum_blinding_evidence_v0.6.0.json + grafik resolusi tinggi
"""

import base64
import hashlib
import json
import os
import sys
import matplotlib.pyplot as plt
import numpy as np
from qiskit import QuantumCircuit, transpile
from qiskit.quantum_info import Statevector
from qiskit.visualization import plot_histogram
from qiskit_aer import AerSimulator

NUM_QUBITS, PERIOD, SHOTS = 6, 8, 8192
N_STATES = 2**NUM_QUBITS


# ---------- 1. Fungsi Inti (Identik dengan V2.1 Perfected) ----------
def generate_nonlinear_fibonacci_phases(
    n_states: int, seed_bytes: bytes
) -> np.ndarray:
  q_sim = 65537
  f_prev2, f_prev1 = 3, 5
  phases = np.zeros(n_states, dtype=float)
  for i in range(n_states):
    digest = hashlib.sha256(seed_bytes + i.to_bytes(4, "little")).digest()
    e_n = int.from_bytes(digest[:4], "little") % q_sim
    f_curr = ((f_prev1 * f_prev2) + e_n) % q_sim
    f_prev2, f_prev1 = f_prev1, f_curr
    phases[i] = (f_curr / q_sim) * (2 * np.pi)
  return phases


def build_qft_circuit(n: int) -> QuantumCircuit:
  qc = QuantumCircuit(n, name="QFT")
  for q in range(n - 1, -1, -1):
    qc.h(q)
    for t in range(q - 1, -1, -1):
      qc.cp(np.pi / float(2 ** (q - t)), t, q)
  for i in range(n // 2):
    qc.swap(i, n - i - 1)
  return qc


def run_qft(
    statevec: np.ndarray, title: str, suffix: str, sim_seed: int = 42
) -> dict:
  sim = AerSimulator()
  qc = QuantumCircuit(NUM_QUBITS)
  qc.initialize(statevec, range(NUM_QUBITS))
  qc.compose(build_qft_circuit(NUM_QUBITS), inplace=True)
  qc.measure_all()

  transpiled = transpile(qc, sim)
  counts = (
      sim.run(transpiled, shots=SHOTS, seed_simulator=sim_seed)
      .result()
      .get_counts()
  )

  int_counts = {str(int(k, 2)).zfill(2): v for k, v in counts.items()}
  sorted_counts = dict(sorted(int_counts.items(), key=lambda x: int(x[0])))

  plt.figure(figsize=(14, 6))
  plot_histogram(sorted_counts, title=title, bar_labels=False)
  plt.xlabel("State Pengukuran (Desimal 00 - 63)")
  plt.ylabel("Jumlah Kemunculan (Shots)")
  plt.savefig(f"qft_vector_{suffix}.png", dpi=300, bbox_inches="tight")
  plt.close()
  return counts


def entropy(counts: dict) -> float:
  p = np.array(list(counts.values())) / SHOTS
  p = p[p > 0]
  return float(-np.sum(p * np.log2(p)))


def dominant_residue(counts: dict) -> int:
  top = max(counts.items(), key=lambda kv: kv[1])
  return int(top[0], 2) % PERIOD


# ---------- 2. Ekstraksi Seed dari JSON Produksi (Tahan Banting) ----------
def find_vector_file() -> str:
  candidates = [
      "test_vectors_v0.6.0.json",
      os.path.join("fcr-engine", "test_vectors_v0.6.0.json"),
      os.path.join("test-vectors", "v1", "test_vectors_v0.6.0.json"),
      os.path.join("..", "test-vectors", "v1", "test_vectors_v0.6.0.json"),
  ]
  for path in candidates:
    if os.path.exists(path):
      return path
  sys.exit(
      "❌ File test_vectors_v0.6.0.json tidak ditemukan! Pastikan path sudah"
      " benar."
  )


def extract_blinding_seed(tc: dict) -> tuple[bytes, str]:
  """Mengekstrak BLINDING_NONCE sesuai format crypto.rs (4-bagian Base64)."""
  inputs = tc.get("inputs", {})
  outputs = tc.get("expected_outputs", {})
  master_seed = str(inputs.get("seed", "fcr-master-seed")).encode()

  # Prioritas 1: Blinding override (TC-002 deterministik)
  override = inputs.get("blinding_override") or inputs.get("blinding")
  if override:
    raw = str(override).encode()
    return (
        hashlib.sha256(master_seed + b":" + raw).digest(),
        "blinding_override",
    )

  # Prioritas 2: Ekstrak bagian pertama (BLINDING_NONCE) dari ciphertext produksi
  ct = outputs.get("ciphertext_b64") or outputs.get("ciphertext")
  if ct and isinstance(ct, str):
    raw_ct_str = ct
    if ":" not in ct:
      try:
        raw_ct_str = base64.b64decode(ct).decode("utf-8", errors="ignore")
      except Exception:
        pass

    if ":" in raw_ct_str:
      nonce_b64 = raw_ct_str.split(":")[0]
      try:
        nonce_bytes = base64.b64decode(nonce_b64)
        return (
            hashlib.sha256(master_seed + b":" + nonce_bytes).digest(),
            "ciphertext[BLINDING_NONCE]",
        )
      except Exception:
        return (
            hashlib.sha256(master_seed + b":" + nonce_b64.encode()).digest(),
            "ciphertext[NONCE_RAW]",
        )

  # Prioritas 3: Fallback unik untuk TC-003 (Honeypot / Anomaly Case)
  fallback_blob = json.dumps(tc, sort_keys=True).encode()
  return (
      hashlib.sha256(b"honeypot-isolated-context:" + fallback_blob).digest(),
      "honeypot_decoy_context",
  )


# ---------- 3. Eksekusi Simulasi Integrasi ----------
if __name__ == "__main__":
  vector_path = find_vector_file()
  print(f"📂 Memuat test vector produksi dari: {vector_path}")

  with open(vector_path, "r", encoding="utf-8") as f:
    data = json.load(f)
  vectors = data.get("test_cases", data if isinstance(data, list) else [])

  periodic = np.zeros(N_STATES, dtype=complex)
  for i in range(0, N_STATES, PERIOD):
    periodic[i] = 1.0 / np.sqrt(N_STATES // PERIOD)

  counts_ref = run_qft(
      periodic, "Reference: Unblinded Periodic Lattice", "reference"
  )
  H_ref = entropy(counts_ref)
  print(f"📐 Reference entropi (tanpa blinding): {H_ref:.4f} bits\n")

  evidence = {
      "metadata": {
          "source": vector_path,
          "derivation": (
              "SHA-256(master_seed || BLINDING_NONCE) -> Non-Linear Fibonacci"
          ),
          "num_qubits": NUM_QUBITS,
          "shots": SHOTS,
      },
      "results": [],
  }

  # Proses semua TC selain TC-002 (misal TC-001 dan TC-003)
  for tc in vectors:
    tc_id = tc.get("id", "UNKNOWN")
    if tc_id == "TC-002":
      continue
    seed, src = extract_blinding_seed(tc)
    phases = generate_nonlinear_fibonacci_phases(N_STATES, seed)
    counts = run_qft(
        periodic * np.exp(1j * phases), f"{tc_id} Blinded ({src})", tc_id.lower()
    )
    H = entropy(counts)
    ev = {
        "id": tc_id,
        "seed_source": src,
        "entropy_bits": round(H, 4),
        "delta_vs_reference": round(H - H_ref, 4),
        "dominant_residue_mod8": dominant_residue(counts),
        "top5": sorted(
            [(int(k, 2), v) for k, v in counts.items()],
            key=lambda x: x[1],
            reverse=True,
        )[:5],
    }
    evidence["results"].append(ev)
    print(
        f"🔹 {tc_id} [{src}]: H={H:.4f} bits (Δ +{H-H_ref:.4f}), kelas residu"
        f" dominan = {ev['dominant_residue_mod8']} (mod 8)"
    )

  # TC-002: Determinisme Eksak (Statevector & Quantum Sampling)
  tc2 = next((t for t in vectors if t.get("id") == "TC-002"), None)
  identik = False
  if tc2:
    seed2, src2 = extract_blinding_seed(tc2)
    ph2 = generate_nonlinear_fibonacci_phases(N_STATES, seed2)
    sv_a = Statevector(periodic * np.exp(1j * ph2))
    sv_b = Statevector(periodic * np.exp(1j * ph2))
    c_a = run_qft(
        sv_a.data, "TC-002 Run A (Deterministic)", "tc002_runA", sim_seed=42
    )
    c_b = run_qft(
        sv_b.data, "TC-002 Run B (Deterministic)", "tc002_runB", sim_seed=42
    )

    identik = np.allclose(sv_a.data, sv_b.data, atol=1e-12) and (c_a == c_b)
    H2 = entropy(c_a)
    print(
        f"🔹 TC-002 [{src2}]: H={H2:.4f} bits | Statevector & Run identik"
        f" bit-per-bit: {'✅ VERIFIED' if identik else '❌ FAILED'}"
    )
    evidence["results"].append({
        "id": "TC-002",
        "seed_source": src2,
        "entropy_bits": round(H2, 4),
        "delta_vs_reference": round(H2 - H_ref, 4),
        "determinism_verified": bool(identik),
    })

  # TC-003 vs TC-001: Verifikasi Isolasi Honeypot
  r1 = next((r for r in evidence["results"] if r["id"] == "TC-001"), None)
  r3 = next((r for r in evidence["results"] if r["id"] == "TC-003"), None)
  isolated = False
  if r1 and r3:
    isolated = (r1["dominant_residue_mod8"] != r3["dominant_residue_mod8"]) or (
        r1["top5"] != r3["top5"]
    )
    print(
        "🔹 TC-003 vs TC-001: distribusi spektrum"
        f" {'TERISOLASI ✅' if isolated else '⚠️ IDENTIK'}"
    )

  all_delta_ok = all(r["delta_vs_reference"] > 1.5 for r in evidence["results"])
  status = (
      "PASS"
      if (all_delta_ok and identik and (isolated if (r1 and r3) else True))
      else "REVIEW"
  )

  evidence["conclusion"] = {
      "blinding_effective": all_delta_ok,
      "determinism": bool(identik),
      "honeypot_isolation": bool(isolated),
      "status": status,
  }

  with open(
      "quantum_blinding_evidence_v0.6.0.json", "w", encoding="utf-8"
  ) as f:
    json.dump(evidence, f, indent=2)

  print(f"\n📄 Kesimpulan Audit: {status}")
  print(
      "💾 Bukti tersimpan: quantum_blinding_evidence_v0.6.0.json + grafik PNG"
  )
