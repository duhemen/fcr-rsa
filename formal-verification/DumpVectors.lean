import Std.Tactic.BVDecide

-- Definisi lokal (self-contained) untuk generator vektor
def ctMask (choice : UInt64) : UInt64 :=
  0 - (choice &&& 1)

def ctSelectLimb (choice : UInt64) (a b : UInt64) : UInt64 :=
  let mask := ctMask choice
  b ^^^ (mask &&& (a ^^^ b))

def mix (s : UInt64) : UInt64 :=
  let x := s * 2654435761 + 1013904223
  (x >>> 16) ^^^ (x >>> 8) ^^^ x

def modInv (a m : Nat) : Nat := Nat.powMod a (m - 2) m  -- m harus prima

def main : IO Unit := do
  let Q : Nat := 1000000007  -- prima, > 2
  IO.println "kind,choice,a,b,res"
  for i in List.range 2048 do
    let i_u64 : UInt64 := i.toUInt64
    let c : UInt64 := (mix i_u64) &&& 1
    let a : UInt64 := mix (i_u64 * 3 + 1)
    let b : UInt64 := mix (i_u64 * 3 + 2)
    let res := ctSelectLimb c a b
    IO.println s!"ct,{UInt64.toNat c},{UInt64.toNat a},{UInt64.toNat b},{UInt64.toNat res}"
  IO.println "kind,M,E,B,Einv,Binv,out"
  for i in List.range 512 do
    let M := UInt64.toNat (mix (UInt64.ofNat i)) % Q
    let E := (UInt64.toNat (mix (UInt64.ofNat (i + 7))) % (Q - 2)) + 2
    let B := (UInt64.toNat (mix (UInt64.ofNat (i + 13))) % (Q - 2)) + 2
    let Einv := modInv E Q
    let Binv := modInv B Q
    let out := (((M * E * B) % Q) * ((Einv * Binv) % Q)) % Q
    IO.println s!"blind,{M},{E},{B},{Einv},{Binv},{out}"
