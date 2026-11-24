"""
FCR-RSA: Quantum Fibonacci Blinding Proof-of-Concept (V2.1 - Perfected)
Tujuan: Membuktikan secara visual dan matematis bahwa phase masking berbasis
Deret Fibonacci Non-Linear + Noise LWE menghancurkan interferensi konstruktif pada QFT.
"""

import hashlib
import matplotlib.pyplot as plt
import numpy as np
from qiskit import QuantumCircuit, transpile
from qiskit.visualization import plot_histogram
from qiskit_aer import AerSimulator

# --- 1. Parameter Simulasi ---
NUM_QUBITS = 6  # 6 qubit = 64 keadaan dasar (Entropi maks = 6.00 bits)
PERIOD = 8  # Simulasi periode r = 8 (seharusnya muncul 8 puncak tajam di QFT normal)
SHOTS = 8192
NOISE_SEED = b"fcr-rsa-v0.6.0-quantum-oracle-seed"


def generate_nonlinear_fibonacci_phases(
    n_states: int, seed: bytes
) -> np.ndarray:
  """
    Menghasilkan phase shift berdasarkan formula inti FCR-RSA:
    F_n = (F_{n-1} * F_{n-2}) + e_n (mod Q_sim)
    Kemudian dipetakan menjadi sudut fase kuantum [0, 2π).
    """
  # Gunakan modulus prima besar untuk simulasi state agar tidak terjebak siklus genap
  q_sim = 65537
  f_prev2, f_prev1 = 3, 5
  phases = np.zeros(n_states, dtype=float)

  for i in range(n_states):
    # Noise deterministik e_n dari SHA-256 (mensimulasikan noise LWE deterministik)
    digest = hashlib.sha256(seed + i.to_bytes(4, "little")).digest()
    e_n = int.from_bytes(digest[:4], "little") % q_sim

    # Persamaan Non-Linear Fibonacci FCR-RSA
    f_curr = ((f_prev1 * f_prev2) + e_n) % q_sim
    f_prev2, f_prev1 = f_prev1, f_curr

    # Petakan ke sudut fase penuh [0, 2π)
    phases[i] = (f_curr / q_sim) * (2 * np.pi)

  return phases


def build_qft_circuit(num_qubits: int) -> QuantumCircuit:
  """Membangun sirkuit Quantum Fourier Transform (QFT) standar yang akurat (MSB -> LSB)."""
  qc = QuantumCircuit(num_qubits, name="QFT")
  for qubit in range(num_qubits - 1, -1, -1):
    qc.h(qubit)
    for target_qubit in range(qubit - 1, -1, -1):
      angle = np.pi / float(2 ** (qubit - target_qubit))
      qc.cp(angle, target_qubit, qubit)

  # Swap qubits untuk menyesuaikan konvensi little-endian Qiskit
  for i in range(num_qubits // 2):
    qc.swap(i, num_qubits - i - 1)
  return qc


def run_simulation(
    title: str, initial_statevector: np.ndarray, filename_suffix: str
) -> dict:
  """Menjalankan simulasi QFT dan memplot hasilnya dengan register klasik yang bersih."""
  simulator = AerSimulator()

  # Gunakan QuantumCircuit(NUM_QUBITS) tanpa register klasik ganda
  qc = QuantumCircuit(NUM_QUBITS)
  qc.initialize(initial_statevector, range(NUM_QUBITS))

  qft_circ = build_qft_circuit(NUM_QUBITS)
  qc.compose(qft_circ, inplace=True)
  qc.measure_all()  # Otomatis membuat 1 register klasik berukuran NUM_QUBITS

  transpiled_qc = transpile(qc, simulator)
  job = simulator.run(transpiled_qc, shots=SHOTS)
  counts = job.result().get_counts()

  # Konversi bitstring biner ke nilai integer desimal agar grafik mudah dibaca
  int_counts = {str(int(k, 2)).zfill(2): v for k, v in counts.items()}
  sorted_int_counts = dict(sorted(int_counts.items(), key=lambda x: int(x[0])))

  plt.figure(figsize=(14, 6))
  plot_histogram(sorted_int_counts, title=title, bar_labels=False)
  plt.xlabel("State Pengukuran (Desimal 0 - 63)")
  plt.ylabel("Jumlah Kemunculan (Shots)")
  plt.savefig(
      f"qft_interference_{filename_suffix}_v2.png", dpi=300, bbox_inches="tight"
  )
  plt.close()

  return counts


def calculate_entropy(counts: dict, shots: int) -> float:
  probs = np.array(list(counts.values())) / shots
  probs = probs[probs > 0]
  return float(-np.sum(probs * np.log2(probs)))


# --- 2. Eksekusi Simulasi ---
if __name__ == "__main__":
  print(
      "🚀 Memulai Simulasi Kuantum FCR-RSA Fibonacci Blinding (V2.1"
      " Perfected)..."
  )

  n_states = 2**NUM_QUBITS
  # Statevector Awal: Superposisi periodik (Peaks di 0, 8, 16, 24, 32, 40, 48, 56)
  periodic_state = np.zeros(n_states, dtype=complex)
  num_peaks = n_states // PERIOD
  for i in range(0, n_states, PERIOD):
    periodic_state[i] = 1.0 / np.sqrt(num_peaks)

  print("📊 [1/2] Menjalankan Sirkuit A: QFT Standar (Tanpa Blinding)...")
  counts_normal = run_simulation(
      "QFT Output: Standard Periodic State (Sharp Constructive Interference)",
      periodic_state,
      "normal",
  )

  print(
      "📊 [2/2] Menjalankan Sirkuit B: QFT dengan Non-Linear Fibonacci Phase"
      " Blinding..."
  )
  fib_phases = generate_nonlinear_fibonacci_phases(n_states, NOISE_SEED)
  blinded_state = periodic_state * np.exp(1j * fib_phases)

  counts_blinded = run_simulation(
      "QFT Output: FCR-RSA Non-Linear Fibonacci Blinded State (Period"
      " Destroyed)",
      blinded_state,
      "blinded",
  )

  # --- 3. Analisis Hasil ---
  print("\n" + "=" * 72)
  print("📈 HASIL SIMULASI & ANALISIS ENTROPI SHANNON (MAKS = 6.00 BITS)")
  print("=" * 72)

  entropy_normal = calculate_entropy(counts_normal, SHOTS)
  entropy_blinded = calculate_entropy(counts_blinded, SHOTS)

  # Konversi top 5 ke angka desimal agar jelas terlihat kelipatan 8-nya
  top_normal = sorted(
      [(int(k, 2), v) for k, v in counts_normal.items()],
      key=lambda x: x[1],
      reverse=True,
  )[:5]
  top_blinded = sorted(
      [(int(k, 2), v) for k, v in counts_blinded.items()],
      key=lambda x: x[1],
      reverse=True,
  )[:5]

  print(f"🔹 Top 5 State Normal (Desimal, Shots)  : {top_normal}")
  print(f"🔹 Top 5 State Blinded (Desimal, Shots) : {top_blinded}")
  print("-" * 72)
  print(
      f"🔹 Entropi Distribusi (Normal)   : {entropy_normal:.4f} bits (Target: ~"
      f" {np.log2(PERIOD):.2f} bits)"
  )
  print(
      f"🔹 Entropi Distribusi (Blinded)  : {entropy_blinded:.4f} bits (Target:"
      f" mendekati {NUM_QUBITS:.2f} bits)"
  )
  print(
      "🔹 Delta Peningkatan Entropi     :"
      f" +{entropy_blinded - entropy_normal:.4f} bits"
  )

  print("\n✅ KESIMPULAN:")
  if entropy_blinded > entropy_normal + 1.5:
    print(
        "   [✅ PASS] Non-Linear Fibonacci Blinding BERHASIL menghancurkan"
        " interferensi konstruktif QFT!"
    )
    print(
        "   Puncak periode hilang dan menyebar mendekati distribusi seragam"
        " (noise)."
    )
  else:
    print("   [⚠️ WARN] Perbedaan entropi masih di bawah ambang batas 1.5 bits.")

  print("\n💾 Plot visual disimpan sebagai:")
  print("   - qft_interference_normal_v2.png")
  print("   - qft_interference_blinded_v2.png")
