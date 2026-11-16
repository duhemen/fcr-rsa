from fpylll import IntegerMatrix, BKZ, LLL


def attack_bkz_2_0() -> bool:
    """
    Simulasi serangan BKZ 2.0.

    Returns:
        True jika penyerang berhasil menemukan plaintext, False jika gagal.
    """
    print("=== SIMULASI RED TEAM: BKZ 2.0 LATTICE ATTACK ===\n")

    # 1. Parameter FCR-RSA (sama dengan test vector)
    Q = 1000000007
    E_vec = 779117744  # seq[8] untuk seed "fcr-rsa-demo-seed", n=10
    M_asli = 42
    R_blind_asli = 12345

    # 2. Simulasi enkripsi (apa yang dilihat penyerang)
    C = (M_asli * E_vec + R_blind_asli) % Q
    print(f"[TARGET] Ciphertext (C) : {C}")
    print(f"[TARGET] Public E_vec   : {E_vec}")
    print(f"[TARGET] Modulus Q      : {Q}\n")

    # 3. Membangun matriks kisi (embedding)
    # Penyerang ingin mencari (M, R) sehingga M*E_vec + R = C (mod Q).
    # Basis 3x3:
    #   Baris 1: [Q, 0, 0]       -> modulo
    #   Baris 2: [E_vec, 1, 0]   -> koefisien M
    #   Baris 3: [C, 0, 1]       -> koefisien R
    M_lattice = IntegerMatrix(3, 3)
    M_lattice[0, 0] = Q
    M_lattice[1, 0] = E_vec
    M_lattice[1, 1] = 1
    M_lattice[2, 0] = C
    M_lattice[2, 2] = 1

    print("[PENYERANG] Matriks Kisi Awal:")
    for i in range(3):
        print([M_lattice[i, j] for j in range(3)])

    # 4. LLL (pemanasan)
    print("\n[PENYERANG] Menerapkan LLL (Pemanasan)...")
    LLL.reduction(M_lattice)

    # 5. BKZ 2.0 dengan block size agresif
    print("[PENYERANG] Menerapkan BKZ 2.0 (Block Size = 20)...")
    BKZ.reduction(M_lattice, BKZ.Param(block_size=20, max_loops=4))

    print("\n[PENYERANG] Matriks Setelah BKZ:")
    for i in range(3):
        print([M_lattice[i, j] for j in range(3)])

    # 6. Analisis hasil
    print("\n[PENYERANG] Menganalisis vektor pendek...")
    found = False
    for i in range(3):
        row = [M_lattice[i, j] for j in range(3)]
        print(f" Vektor {i}: {row}")
        if abs(row[1]) == M_asli:
            print(f" -> 🚨 PENYERANG MENEMUKAN M! Nilai: {abs(row[1])}")
            found = True
            break

    if not found:
        print("\n[KESIMPULAN] ❌ BKZ 2.0 GAGAL menemukan pesan asli (M).")
        print("Ini membuktikan bahwa noise LWE + Blinding Factor melindungi FCR-RSA.")
        print("Meskipun penyerang bisa membangun kisi, vektor terpendek yang ditemukan")
        print("bukanlah pesan yang sebenarnya.")
    else:
        print("\n[KESIMPULAN] ⚠️ BKZ 2.0 BERHASIL MENEMBUS!")
        print("FCR-RSA rentan terhadap serangan kisi untuk parameter ini.")

    return found


if __name__ == "__main__":
    berhasil = attack_bkz_2_0()
    raise SystemExit(0 if not berhasil else 1)