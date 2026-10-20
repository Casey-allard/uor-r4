import WasmGemmGnaf.Gemm.Problem
import WasmGemmGnaf.Wasm.Run
set_option autoImplicit false
set_option exponentiation.threshold 400
set_option maxRecDepth 8000

/-!
# Gemm: the reference relation (SPEC §8.4)

> `Accepts` is the conjunction of
> `ReferenceSemanticAccepts problem raw (Gemm.semanticFor problem raw observation)`
> and `Wasm.CandidateCallMemoryWritesWithin observation
> (problem.SanctionedWriteRegions raw (Gemm.semanticFor problem raw observation))`.
> … For valid input it SHALL express `C ← alpha · op(A) · op(B) + beta · C` under
> the descriptor's exact arithmetic, the exact returned status, the complete
> final C byte region, exact status-detail bytes, and the absence of every
> forbidden effect. For invalid input it SHALL express the exact typed status and
> permitted memory effects. Trapped and uncaught-exception observations are
> rejected by every released problem case. It SHALL be total over raw
> invocations.

This module first completes the *exact* arithmetic of SPEC §8.2 on top of
`Gemm/Arithmetic.lean` and `Gemm/FloatBits.lean` — the modular ring at the
accumulator width, the checked mathematical-integer evaluation with its
first-overflow scan, the strict round-at-each-declared-operation floating
evaluation, and the exact-dyadic round-once evaluation — and then reads that
arithmetic off the raw bytes to give one *total* function

  `raw ↦ (referenceStatus, referenceObservableStore)`.

`Accepts` says exactly that an execution observation returned that status and
left that store, and wrote nothing outside SPEC §8.3's closed sanctioned set.
-/

namespace WasmGemmGnaf.Gemm

/-! ## Bit length

Used by the rounding function to locate the binade of an exact dyadic.  The
fuel argument makes the recursion structural, so the definition reduces in the
kernel; `bitLen_lt` and `le_bitLen` prove it really is the binary length. -/

/-- Fuelled binary length. -/
def bitLenAux : Nat → Nat → Nat
  | 0, _ => 0
  | fuel + 1, n => if n = 0 then 0 else bitLenAux fuel (n / 2) + 1

/-- Binary length: the least `w` with `n < 2 ^ w`. -/
def bitLen (n : Nat) : Nat := bitLenAux n n

theorem bitLenAux_zero : ∀ fuel : Nat, bitLenAux fuel 0 = 0 := by
  intro fuel; cases fuel <;> simp [bitLenAux]

theorem bitLenAux_pos : ∀ (fuel n : Nat), n ≤ fuel → 0 < n → 0 < bitLenAux fuel n := by
  intro fuel
  cases fuel with
  | zero => intro n h hp; omega
  | succ f =>
    intro n _ hp
    have hn : ¬ n = 0 := by omega
    simp [bitLenAux, hn]

theorem bitLenAux_lt : ∀ (fuel n : Nat), n ≤ fuel → n < 2 ^ bitLenAux fuel n := by
  intro fuel
  induction fuel with
  | zero =>
    intro n h
    have hz : n = 0 := by omega
    subst hz
    simp [bitLenAux]
  | succ f ih =>
    intro n h
    by_cases hn : n = 0
    · subst hn; simp [bitLenAux]
    · have hhalf : n / 2 ≤ f := by omega
      have := ih (n / 2) hhalf
      have hb : bitLenAux (f + 1) n = bitLenAux f (n / 2) + 1 := by
        simp [bitLenAux, hn]
      rw [hb, Nat.pow_succ]
      omega

theorem le_bitLenAux : ∀ (fuel n : Nat), n ≤ fuel → 0 < n →
    2 ^ (bitLenAux fuel n - 1) ≤ n := by
  intro fuel
  induction fuel with
  | zero => intro n h hp; omega
  | succ f ih =>
    intro n h hp
    have hn : ¬ n = 0 := by omega
    have hb : bitLenAux (f + 1) n = bitLenAux f (n / 2) + 1 := by
      simp [bitLenAux, hn]
    rw [hb]
    by_cases hh : n / 2 = 0
    · rw [hh, bitLenAux_zero]
      simpa using hp
    · have hhalf : n / 2 ≤ f := by omega
      have hle := ih (n / 2) hhalf (by omega)
      have hbpos : 0 < bitLenAux f (n / 2) := bitLenAux_pos f (n / 2) hhalf (by omega)
      have hpow : 2 * 2 ^ (bitLenAux f (n / 2) - 1) = 2 ^ bitLenAux f (n / 2) := by
        rw [← Nat.pow_succ']
        congr 1
        omega
      simp only [Nat.add_sub_cancel]
      omega

/-- `n` is strictly below `2 ^ bitLen n`. -/
theorem bitLen_lt (n : Nat) : n < 2 ^ bitLen n := bitLenAux_lt n n (Nat.le_refl n)

/-- `bitLen` is not larger than it must be. -/
theorem le_bitLen {n : Nat} (h : 0 < n) : 2 ^ (bitLen n - 1) ≤ n :=
  le_bitLenAux n n (Nat.le_refl n) h

theorem bitLen_zero : bitLen 0 = 0 := rfl
theorem bitLen_one : bitLen 1 = 1 := rfl

/-- The minimum number of bits a mathematical integer needs in the declared
signedness (SPEC §8.3's checked-overflow detail). -/
def bitsRequired (signed : Bool) (z : Int) : Nat :=
  if signed then
    (if 0 ≤ z then bitLen z.toNat + 1 else bitLen (-z - 1).toNat + 1)
  else
    (if z < 0 then 2 ^ 64 - 1 else bitLen z.toNat)

/-! ## Exact dyadic values

An exact binary value `num · 2 ^ exp`.  Products and sums of dyadics are exact;
this is the carrier both float modes reduce through. -/

/-- An exact dyadic rational `num · 2 ^ exp`. -/
structure Dyadic where
  /-- Signed numerator. -/
  num : Int
  /-- Binary exponent. -/
  exp : Int
  deriving DecidableEq, Repr, Inhabited

namespace Dyadic

/-- Exact zero. -/
def zero : Dyadic := ⟨0, 0⟩

/-- Exact product. -/
def mul (a b : Dyadic) : Dyadic := ⟨a.num * b.num, a.exp + b.exp⟩

/-- Exact sum, at the finer of the two exponents. -/
def add (a b : Dyadic) : Dyadic :=
  let e := min a.exp b.exp
  ⟨a.num * 2 ^ (a.exp - e).toNat + b.num * 2 ^ (b.exp - e).toNat, e⟩

@[simp] theorem zero_num : zero.num = 0 := rfl

theorem mul_num (a b : Dyadic) : (a.mul b).num = a.num * b.num := rfl

theorem add_zero_left (b : Dyadic) (h : 0 ≤ b.exp) :
    (zero.add b).num = b.num * 2 ^ b.exp.toNat := by
  simp only [add, zero, Int.zero_mul, Int.zero_add]
  have : min (0 : Int) b.exp = 0 := by omega
  simp [this]

end Dyadic

/-! ## Exact float values

A decoded operand is either the canonical NaN class, a signed infinity, or an
exact finite dyadic.  The special-value tables are *not* re-stated here: they
are `Gemm.SpecialClass.mul`, `Gemm.SpecialClass.add` and `Gemm.dyadicReduce` of
`Gemm/Arithmetic.lean`, and `evalMul`/`evalAdd` below are proved to agree with
them. -/

/-- An exactly represented floating operand or intermediate. -/
inductive EVal
  /-- A NaN (payload already canonicalized). -/
  | nan
  /-- A signed infinity. -/
  | inf (sign : Bool)
  /-- An exact finite dyadic. -/
  | num (d : Dyadic)
  deriving DecidableEq, Repr, Inhabited

namespace EVal

/-- The coarse class SPEC §8.2's tables react to. -/
def classOf : EVal → SpecialClass
  | .nan => .nan
  | .inf s => .inf s
  | .num d => if d.num = 0 then .zero false else .finite (decide (d.num < 0))

end EVal

/-- Exact product of two operands: the special-value table, then the exact
dyadic product. -/
def evalMul : EVal → EVal → EVal
  | .nan, _ => .nan
  | _, .nan => .nan
  | .inf s, .inf t => .inf (xor s t)
  | .inf s, .num d => if d.num = 0 then .nan else .inf (xor s (decide (d.num < 0)))
  | .num d, .inf t => if d.num = 0 then .nan else .inf (xor (decide (d.num < 0)) t)
  | .num a, .num b => .num (a.mul b)

/-- Exact sum of two operands: the special-value table, then the exact dyadic
sum. -/
def evalAdd : EVal → EVal → EVal
  | .nan, _ => .nan
  | _, .nan => .nan
  | .inf s, .inf t => if s = t then .inf s else .nan
  | .inf s, .num _ => .inf s
  | .num _, .inf t => .inf t
  | .num a, .num b => .num (a.add b)

/-- **`evalMul` is SPEC §8.2's product table.**  Whenever the table of
`Gemm/Arithmetic.lean` prescribes an outcome class, `evalMul` produces exactly
that class. -/
theorem classOf_evalMul {x y : EVal} {c : SpecialClass}
    (h : SpecialClass.mul x.classOf y.classOf = some c) :
    (evalMul x y).classOf = c := by
  cases x with
  | nan => cases y <;> simp_all [EVal.classOf, evalMul, SpecialClass.mul]
  | inf s =>
    cases y with
    | nan => simp_all [EVal.classOf, evalMul, SpecialClass.mul]
    | inf t => simp_all [EVal.classOf, evalMul, SpecialClass.mul]
    | num d =>
      by_cases hd : d.num = 0 <;>
        simp_all [EVal.classOf, evalMul, SpecialClass.mul]
  | num a =>
    cases y with
    | nan =>
      by_cases ha : a.num = 0 <;> simp_all [EVal.classOf, evalMul, SpecialClass.mul]
    | inf t =>
      by_cases ha : a.num = 0 <;>
        simp_all [EVal.classOf, evalMul, SpecialClass.mul]
    | num b =>
      by_cases ha : a.num = 0 <;> by_cases hb : b.num = 0 <;>
        simp_all [EVal.classOf, evalMul, SpecialClass.mul]

/-- **`evalAdd` is SPEC §8.2's sum table.** -/
theorem classOf_evalAdd {x y : EVal} {c : SpecialClass}
    (h : SpecialClass.add x.classOf y.classOf = some c) :
    (evalAdd x y).classOf = c := by
  cases x with
  | nan => cases y <;> simp_all [EVal.classOf, evalAdd, SpecialClass.add]
  | inf s =>
    cases y with
    | nan => simp_all [EVal.classOf, evalAdd, SpecialClass.add]
    | inf t =>
      by_cases hst : s = t <;> simp_all [EVal.classOf, evalAdd, SpecialClass.add]
    | num d =>
      by_cases hd : d.num = 0 <;> simp_all [EVal.classOf, evalAdd, SpecialClass.add]
  | num a =>
    cases y with
    | nan =>
      by_cases ha : a.num = 0 <;> simp_all [EVal.classOf, evalAdd, SpecialClass.add]
    | inf t =>
      by_cases ha : a.num = 0 <;> simp_all [EVal.classOf, evalAdd, SpecialClass.add]
    | num b =>
      by_cases ha : a.num = 0 <;> by_cases hb : b.num = 0 <;>
        simp_all [EVal.classOf, evalAdd, SpecialClass.add]

/-- `0 × infinity` is a NaN, as SPEC §8.2 requires. -/
theorem evalMul_zero_inf (s : Bool) : evalMul (.num Dyadic.zero) (.inf s) = .nan := rfl

/-- Adding opposite infinities is a NaN. -/
theorem evalAdd_opposite_inf : evalAdd (.inf true) (.inf false) = .nan := rfl

/-- Adding equal-signed infinities is that infinity. -/
theorem evalAdd_same_inf (s : Bool) : evalAdd (.inf s) (.inf s) = .inf s := by
  cases s <;> rfl

/-! ## Decoding a stored pattern to an exact value -/

/-- Signed embedding of a magnitude. -/
def signedNat (s : Bool) (v : Nat) : Int := if s then -(v : Int) else (v : Int)

/-- The exponent of the subnormal binade of a format. -/
def subnormalExp (f : FloatFormat) : Int := 1 - (f.bias : Int) - (f.mantBits : Int)

/-- Decode a stored bit pattern of format `f` to its exact value. -/
def evalOfBits (f : FloatFormat) (bits : Nat) : EVal :=
  match f.decode bits with
  | .nan _ _ _ => .nan
  | .infinity s => .inf s
  | .zero _ => .num Dyadic.zero
  | .subnormal s m => .num ⟨signedNat s m, subnormalExp f⟩
  | .normal s e m =>
      .num ⟨signedNat s (2 ^ f.mantBits + m), (e : Int) - (f.bias : Int) - (f.mantBits : Int)⟩

/-! ## Round-to-nearest, ties-to-even, into a format

Gradual underflow, IEEE sign rules for nonzero results, overflow to the signed
infinity, and SPEC §8.2's exact-zero sign rule (`Gemm.exactZeroSign`). -/

/-- `round (N · 2 ^ exp / 2 ^ q)`, ties to even.  Exact when `q ≤ exp`. -/
def roundShift (N : Nat) (exp q : Int) : Nat :=
  if q ≤ exp then N * 2 ^ (exp - q).toNat else rneNat N (2 ^ (q - exp).toNat)

/-- Round an exact dyadic into `f`. -/
def roundDyadic (f : FloatFormat) (d : Dyadic) : Nat :=
  if d.num = 0 then f.pack false 0 0
  else
    let neg := decide (d.num < 0)
    let N := d.num.natAbs
    let emin : Int := subnormalExp f
    let e : Int := (bitLen N : Int) - 1 + d.exp
    let q0 : Int := max emin (e - (f.mantBits : Int))
    let s0 : Nat := roundShift N d.exp q0
    let carried : Bool := decide (s0 = 2 ^ (f.mantBits + 1))
    let sig : Nat := if carried then 2 ^ f.mantBits else s0
    let q : Int := if carried then q0 + 1 else q0
    if sig = 0 then f.pack (exactZeroSign false neg) 0 0
    else if sig < 2 ^ f.mantBits then f.pack neg 0 sig
    else
      let E : Int := q + (f.mantBits : Int) + (f.bias : Int)
      if (f.expMax : Int) ≤ E then f.pack neg f.expMax 0
      else f.pack neg E.toNat (sig - 2 ^ f.mantBits)

/-- Encode an exact value in `f`: the canonical quiet NaN for a NaN, the signed
infinity for an infinity, and the rounded pattern for a finite value. -/
def roundToFormat (f : FloatFormat) : EVal → Nat
  | .nan => f.canonicalQuietNaN
  | .inf s => f.pack s f.expMax 0
  | .num d => roundDyadic f d

/-- Every released deterministic mode canonicalizes its NaN output: no
unobserved payload freedom remains (SPEC §8.2). -/
theorem roundToFormat_nan (f : FloatFormat) :
    roundToFormat f .nan = f.canonicalQuietNaN := rfl

theorem roundToFormat_binary32_nan :
    roundToFormat FloatFormat.binary32 .nan = 0x7fc00000 := by decide

theorem roundToFormat_binary64_nan :
    roundToFormat FloatFormat.binary64 .nan = 0x7ff8000000000000 := by decide

/-- Exact `+0` rounds to `+0`. -/
theorem roundDyadic_zero (f : FloatFormat) : roundDyadic f Dyadic.zero = f.pack false 0 0 :=
  rfl

/-- The format of a floating stored kind.  Total by construction; it is only
ever applied to a floating kind, because a compatible descriptor in a floating
mode has floating stored kinds (`Gemm.compatible_*`). -/
def formatOf (k : ScalarKind) : FloatFormat :=
  (k.floatFormat?).getD FloatFormat.binary64

@[simp] theorem formatOf_binary16 : formatOf .binary16 = FloatFormat.binary16 := rfl
@[simp] theorem formatOf_bfloat16 : formatOf .bfloat16 = FloatFormat.bfloat16 := rfl
@[simp] theorem formatOf_binary32 : formatOf .binary32 = FloatFormat.binary32 := rfl
@[simp] theorem formatOf_binary64 : formatOf .binary64 = FloatFormat.binary64 := rfl

/-- Round a product of two `src`-format patterns into `dst`. -/
def fMulTo (srcA srcB dst : FloatFormat) (x y : Nat) : Nat :=
  roundToFormat dst (evalMul (evalOfBits srcA x) (evalOfBits srcB y))

/-- Round a sum of two `src`-format patterns into `dst`. -/
def fAddTo (src dst : FloatFormat) (x y : Nat) : Nat :=
  roundToFormat dst (evalAdd (evalOfBits src x) (evalOfBits src y))

/-- Round a `src`-format pattern into `dst`. -/
def fConv (src dst : FloatFormat) (x : Nat) : Nat :=
  roundToFormat dst (evalOfBits src x)

/-! ## Reading and writing elements

Addresses are computed over the mathematical integers by `View.addrOf` and only
then converted; a well-formed descriptor guarantees the address is nonnegative
(`View.LayoutOk.inMemory`), so `Int.toNat` loses nothing. -/

/-- Little-endian read of `w` bytes. -/
def readLE (s : Store) (addr w : Nat) : Nat :=
  (List.range w).foldr (fun i acc => acc * 256 + (s (addr + i)).toNat) 0

/-- Little-endian overwrite of `w` bytes. -/
def writeLE (s : Store) (addr w v : Nat) : Store :=
  fun i => if addr ≤ i ∧ i < addr + w then UInt8.ofNat (v / 256 ^ (i - addr) % 256) else s i

theorem writeLE_outside {s : Store} {addr w v i : Nat} (h : ¬ (addr ≤ i ∧ i < addr + w)) :
    writeLE s addr w v i = s i := by simp [writeLE, h]

/-- The byte offset of a logical element. -/
def elemAddr (v : View) (t : Bool) (x : Index) : Nat := (v.addrOf t x).toNat

/-- The stored bit pattern of a logical element. -/
def elemBits (s : Store) (v : View) (t : Bool) (k : ScalarKind) (x : Index) : Nat :=
  readLE s (elemAddr v t x) k.byteWidth

/-! ## `A`, `B` and `C` element access (SPEC §8.1, §8.3)

`op(A)` and `op(B)` are already folded into the address map: `View.addrOf`
swaps the two inner coordinates of a transposed operand, exactly as SPEC §8.3
prescribes.  `A` is indexed `(b, i, t)`, `B` is indexed `(b, t, j)`, `C` is
indexed `(b, i, j)`. -/

/-- `op(A) [b, i, t]`. -/
def aBitsAt (d : DescriptorBody) (s : Store) (b i t : Nat) : Nat :=
  elemBits s d.aView d.transposeA d.aKind ⟨b, i, t⟩

/-- `op(B) [b, t, j]`. -/
def bBitsAt (d : DescriptorBody) (s : Store) (b t j : Nat) : Nat :=
  elemBits s d.bView d.transposeB d.bKind ⟨b, t, j⟩

/-- `C [b, i, j]`. -/
def cBitsAt (d : DescriptorBody) (s : Store) (x : Index) : Nat :=
  elemBits s d.cView false d.cKind x

/-! ## Modular integer mode (SPEC §8.2)

"Modular integer mode converts operands to the declared accumulator width and
performs every multiply, add, alpha scale, beta scale, and output conversion
modulo that width." -/

/-- The running modular sum over `k`, visited in ascending order. -/
def modularSum (d : DescriptorBody) (s : Store) (x : Index) : Nat :=
  let w := d.accumulatorKind.bitWidth
  (List.range d.k).foldl
    (fun acc t =>
      modAdd w acc
        (modMul w (d.aKind.extendTo w (aBitsAt d s x.b x.i t))
                  (d.bKind.extendTo w (bBitsAt d s x.b t x.j)))) 0

/-- `C ← alpha · op(A) · op(B) + beta · C` in the modular ring of the
accumulator width, truncated to the `C` width. -/
def modularElem (d : DescriptorBody) (s : Store) (x : Index) : Nat :=
  let w := d.accumulatorKind.bitWidth
  modTruncate d.cKind.bitWidth
    (modAdd w
      (modMul w (d.cKind.extendTo w d.alphaBits) (modularSum d s x))
      (modMul w (d.cKind.extendTo w d.betaBits)
        (d.cKind.extendTo w (cBitsAt d s x))))

/-! ## Checked integer mode (SPEC §8.2)

"Checked integer mode performs those operations over mathematical integers and
returns `checked-overflow` with C unchanged if any declared accumulator or
output range is exceeded. … every product, running sum, alpha scale, beta
scale, final addition, and output conversion is checked against the declared
accumulator or C interval, and the first overflow in the fixed row-major
`(batch,row,column,k)` order produces status 4 with C unchanged." -/

/-- The mathematical value of a stored integer pattern. -/
def intOfBits (k : ScalarKind) (bits : Nat) : Int :=
  if k.isSignedInteger then toSigned k.bitWidth bits else (bits : Int)

/-- One checked step: the value, or the first offending value together with the
number of bits the violated interval offers. -/
def checkAgainst (k : ScalarKind) (z : Int) : Except (Int × Nat) Int :=
  if k.InRange z then .ok z else .error (z, k.bitWidth)

/-- The checked running sum over `k`, in ascending order; each product and each
running sum is checked against the accumulator interval. -/
def checkedSum (d : DescriptorBody) (s : Store) (x : Index) : Except (Int × Nat) Int :=
  (List.range d.k).foldl
    (fun acc t => do
      let a ← acc
      let p ← checkAgainst d.accumulatorKind
        (intOfBits d.aKind (aBitsAt d s x.b x.i t) *
          intOfBits d.bKind (bBitsAt d s x.b t x.j))
      checkAgainst d.accumulatorKind (a + p))
    (.ok 0)

/-- The checked element evaluation: the output bit pattern, or the first
offending value. -/
def checkedElem (d : DescriptorBody) (s : Store) (x : Index) :
    Except (Int × Nat) Nat := do
  let sum ← checkedSum d s x
  let scaled ← checkAgainst d.accumulatorKind (intOfBits d.cKind d.alphaBits * sum)
  let scaledC ← checkAgainst d.accumulatorKind
    (intOfBits d.cKind d.betaBits * intOfBits d.cKind (cBitsAt d s x))
  let total ← checkAgainst d.accumulatorKind (scaled + scaledC)
  let out ← checkAgainst d.cKind total
  pure (ofSigned d.cKind.bitWidth out)

/-- The first checked overflow in the fixed `(batch, row, column, k)` order. -/
def checkedFirstFailure (d : DescriptorBody) (s : Store) :
    Option (Index × Int × Nat) :=
  (Index.enumerate d.shapeC).findSome?
    (fun x => match checkedElem d s x with
      | .error (z, bits) => some (x, z, bits)
      | .ok _ => none)

/-- The checked output pattern, once the whole matrix is known not to
overflow. -/
def checkedBits (d : DescriptorBody) (s : Store) (x : Index) : Nat :=
  match checkedElem d s x with
  | .ok v => v
  | .error _ => 0

/-! ## Strict floating mode (SPEC §8.2)

"Strict floating mode visits `k` in ascending order, rounds each product to the
accumulator format, rounds every accumulator addition, separately rounds
`alpha·sum` and `beta·C`, then rounds their addition to C." -/

/-- The strict running sum, in the accumulator format. -/
def strictSum (d : DescriptorBody) (s : Store) (x : Index) : Nat :=
  let fa := formatOf d.accumulatorKind
  let fA := formatOf d.aKind
  let fB := formatOf d.bKind
  (List.range d.k).foldl
    (fun acc t =>
      fAddTo fa fa acc (fMulTo fA fB fa (aBitsAt d s x.b x.i t) (bBitsAt d s x.b t x.j)))
    (fa.pack false 0 0)

/-- `C ← alpha · op(A) · op(B) + beta · C` with a rounding at every declared
operation. -/
def strictElem (d : DescriptorBody) (s : Store) (x : Index) : Nat :=
  let fa := formatOf d.accumulatorKind
  let fc := formatOf d.cKind
  fAddTo fa fc
    (fMulTo fc fa fa d.alphaBits (strictSum d s x))
    (fMulTo fc fc fa d.betaBits (cBitsAt d s x))

/-! ## Exact-dyadic round-once mode (SPEC §8.2)

"Exact-dyadic mode decodes finite values to signed dyadics, forms the complete
mathematical `alpha·Σ(A·B)+beta·C`, and rounds once to C. … Exact-dyadic
reduction first applies this product table, yields canonical NaN if both
infinity signs occur among the scaled terms, yields the sole infinity sign if
exactly one occurs, and otherwise rounds the finite exact dyadic.  Alpha and
beta participate in the same table." -/

/-- The complete list of scaled terms of one `C` element. -/
def exactTerms (d : DescriptorBody) (s : Store) (x : Index) : List EVal :=
  let fc := formatOf d.cKind
  let fA := formatOf d.aKind
  let fB := formatOf d.bKind
  ((List.range d.k).map
    (fun t => evalMul (evalOfBits fc d.alphaBits)
      (evalMul (evalOfBits fA (aBitsAt d s x.b x.i t))
               (evalOfBits fB (bBitsAt d s x.b t x.j))))) ++
  [evalMul (evalOfBits fc d.betaBits) (evalOfBits fc (cBitsAt d s x))]

/-- The exact sum of the finite scaled terms. -/
def exactSum (terms : List EVal) : Dyadic :=
  terms.foldl (fun acc tm => match tm with | .num dd => acc.add dd | _ => acc) Dyadic.zero

/-- One rounding, at the end, into the `C` format. -/
def exactElem (d : DescriptorBody) (s : Store) (x : Index) : Nat :=
  let fc := formatOf d.cKind
  let terms := exactTerms d s x
  match dyadicReduce (terms.map EVal.classOf) with
  | some .nan => fc.canonicalQuietNaN
  | some (.inf sgn) => fc.pack sgn fc.expMax 0
  | some _ => fc.canonicalQuietNaN
  | none => roundDyadic fc (exactSum terms)

/-! ## The declared arithmetic of a descriptor

One function of the descriptor's mode tag: no typeclass, no callback. -/

/-- The `C` element prescribed by the descriptor's declared arithmetic, for the
three modes that never fail at runtime, and the checked mode's output once it
is known not to overflow. -/
def referenceElem (d : DescriptorBody) (s : Store) (x : Index) : Nat :=
  match d.mode with
  | .modular => modularElem d s x
  | .checked => checkedBits d s x
  | .strictFloat => strictElem d s x
  | .exactDyadicRoundOnce => exactElem d s x

/-! ## The signaling-NaN scan (SPEC §8.2)

"A signaling NaN in `alpha`, `beta`, A, B, or the initial C is the sole
released trigger for status 5.  Detection precedes arithmetic and scans in that
listed carrier order, with each matrix in canonical `(batch,row,column)` order.
C remains unchanged." -/

/-- The offending `(offset-or-index, raw bits, quiet-bit mask)` triple of the
first signaling NaN in the fixed carrier order, if any.  `alpha` and `beta`
report their header byte offsets; a matrix element reports its logical index in
canonical order. -/
def signalingScan (d : DescriptorBody) (s : Store) : Option (Nat × Nat × Nat) :=
  if d.cKind.isSignalingNaNBits d.alphaBits then
    some (168, d.alphaBits, (formatOf d.cKind).quietBitMask)
  else if d.cKind.isSignalingNaNBits d.betaBits then
    some (184, d.betaBits, (formatOf d.cKind).quietBitMask)
  else
    match (Index.enumerate d.shapeA).findSome? (fun x =>
        let v := elemBits s d.aView d.transposeA d.aKind x
        if d.aKind.isSignalingNaNBits v then
          some (x.linear d.shapeA, v, (formatOf d.aKind).quietBitMask) else none) with
    | some r => some r
    | none =>
      match (Index.enumerate d.shapeB).findSome? (fun x =>
          let v := elemBits s d.bView d.transposeB d.bKind x
          if d.bKind.isSignalingNaNBits v then
            some (x.linear d.shapeB, v, (formatOf d.bKind).quietBitMask) else none) with
      | some r => some r
      | none =>
        (Index.enumerate d.shapeC).findSome? (fun x =>
          let v := cBitsAt d s x
          if d.cKind.isSignalingNaNBits v then
            some (x.linear d.shapeC, v, (formatOf d.cKind).quietBitMask) else none)

theorem findSome?_const_none {α β : Type} (l : List α) :
    l.findSome? (fun _ => (none : Option β)) = none := by
  induction l with
  | nil => rfl
  | cons a t ih => simp only [List.findSome?, ih]

/-- An integer descriptor never signals: `isSignalingNaNBits` is `false` on
every integer kind, so status 5 is a floating-mode phenomenon only. -/
theorem signalingScan_integer (d : DescriptorBody) (s : Store)
    (ha : d.aKind.isInteger = true) (hb : d.bKind.isInteger = true)
    (hc : d.cKind.isInteger = true) : signalingScan d s = none := by
  have hA : ∀ v, d.aKind.isSignalingNaNBits v = false :=
    fun v => ScalarKind.isSignalingNaNBits_of_integer _ v ha
  have hB : ∀ v, d.bKind.isSignalingNaNBits v = false :=
    fun v => ScalarKind.isSignalingNaNBits_of_integer _ v hb
  have hC : ∀ v, d.cKind.isSignalingNaNBits v = false :=
    fun v => ScalarKind.isSignalingNaNBits_of_integer _ v hc
  simp only [signalingScan, hA, hB, hC, if_false, Bool.false_eq_true,
    findSome?_const_none]

/-! ## The reference outcome of a raw invocation

Everything below is a *total function of the raw bytes*: the released status,
the status-detail record, and the complete final ABI-visible store. -/

/-- The six ABI regions a raw invocation declares.  A raw input whose header
does not even decode declares nothing. -/
def regionsOfRaw {P : Wasm.Profile} (raw : RawInvocationBody P) : Regions :=
  match decodeHeader raw.bytes with
  | some h => (descriptorOf h).regions
  | none => ⟨⟨0, 0⟩, ⟨0, 0⟩, ⟨0, 0⟩, ⟨0, 0⟩, ⟨0, 0⟩, ⟨0, 0⟩⟩

/-- The declared status-detail range. -/
def statusRange {P : Wasm.Profile} (raw : RawInvocationBody P) : ByteRange :=
  (regionsOfRaw raw).statusDetail

/-- SPEC §8.3's "separately validated status range": the declared range is a
32-byte record, lies inside the invocation extent, and is disjoint from the
header.  This test uses *no other* part of the descriptor, which is exactly what
lets a descriptor-level rejection still write it. -/
def statusTrustworthy {P : Wasm.Profile} (raw : RawInvocationBody P) : Bool :=
  match decodeHeader raw.bytes with
  | none => false
  | some h =>
    decide ((descriptorOf h).statusDetail.length = statusDetailBytes) &&
      decide ((descriptorOf h).statusDetail.stop ≤ raw.len.toNat) &&
      decide ((⟨0, headerBytes⟩ : ByteRange).Disjoint (descriptorOf h).statusDetail)

/-- The classifier's cascade, as one decidable Boolean.  `refValid_classify`
below proves it really is the classifier's `valid` verdict. -/
def refValid {P : Wasm.Profile} (raw : RawInvocationBody P) : Bool :=
  match decodeHeader raw.bytes with
  | none => false
  | some h =>
    (headerMalformed h == none) && (tagUnsupported h == none) &&
      decide (headerCompatible h) && decide (dimensionsRepresentable h) &&
      decide (DescriptorWellFormed (descriptorOf h) (windowOf P raw)) &&
      decide (ResourceOk P raw)

/-- **`refValid` is the classifier's verdict.**  Direct consequence of
`Gemm.classifier_exact_domain`. -/
theorem refValid_classify {P : Wasm.Profile} {raw : RawInvocationBody P}
    (h : refValid raw = true) : ∃ inv, classify P raw = .valid inv := by
  unfold refValid at h
  split at h
  · exact absurd h (by simp)
  · next hd hdec =>
    simp only [Bool.and_eq_true, beq_iff_eq, decide_eq_true_eq] at h
    obtain ⟨⟨⟨⟨⟨hmal, htag⟩, hcomp⟩, hdim⟩, hwf⟩, hres⟩ := h
    exact classifier_exact_domain raw hd hdec hmal htag hcomp hdim hwf hres

/-- **The converse**: the classifier's `valid` verdict gives back every clause
of `refValid`, so the two agree on every raw invocation. -/
theorem refValid_of_classify {P : Wasm.Profile} {raw : RawInvocationBody P}
    {inv : ValidInvocation P} (h : classify P raw = .valid inv) : refValid raw = true := by
  unfold refValid
  unfold classify at h
  split at h
  · exact absurd h (by simp)
  · next hd hdec =>
    rw [hdec]
    unfold classifyHeader at h
    split at h
    · exact absurd h (by simp)
    · next hmal =>
      unfold classifyTags at h
      split at h
      · exact absurd h (by simp)
      · next htag =>
        unfold classifyDescriptor at h
        split at h
        · next hcomp =>
          split at h
          · next hdim =>
            split at h
            · next hwf =>
              split at h
              · next hres =>
                simp only [Bool.and_eq_true, beq_iff_eq, decide_eq_true_eq]
                exact ⟨⟨⟨⟨⟨hmal, htag⟩, hcomp⟩, hdim⟩, hwf⟩, hres⟩
              · exact absurd h (by simp)
            · exact absurd h (by simp)
          · exact absurd h (by simp)
        · exact absurd h (by simp)

/-- `refValid` and the classifier are the same verdict. -/
theorem refValid_iff_classify {P : Wasm.Profile} (raw : RawInvocationBody P) :
    refValid raw = true ↔ ∃ inv, classify P raw = .valid inv :=
  ⟨refValid_classify, fun ⟨_, h⟩ => refValid_of_classify h⟩

/-- The typed status of a descriptor-level rejection. -/
def rejectStatus {P : Wasm.Profile} (raw : RawInvocationBody P) : Nat :=
  match classify P raw with
  | .valid _ => 0
  | .invalid _ => 1
  | .unsupported _ => 2
  | .resourceExhausted _ => 3

/-- What the reference prescribes: the returned status, the exact 32-byte
status-detail record, and whether `C` is written. -/
structure RefOutcome where
  /-- The exact returned status. -/
  status : Nat
  /-- The exact status-detail record. -/
  detail : StatusDetail
  /-- Whether the phase writes `C`. -/
  writesC : Bool
  deriving DecidableEq, Repr, Inhabited

/-- The status-5 record: `(quiet-bit mask, offending raw bits)`. -/
def arithmeticDetail (off bits mask : Nat) : StatusDetail where
  status := 5
  fieldCode := UInt32.ofNat FieldCode.arithmetic.code
  offendingOffset := sat64 off
  required := sat64 mask
  available := sat64 bits

/-- The status-4 record: `(minimum bits required, bits available)`. -/
def overflowDetail (index required available : Nat) : StatusDetail where
  status := 4
  fieldCode := UInt32.ofNat FieldCode.overflow.code
  offendingOffset := sat64 index
  required := sat64 required
  available := sat64 available

/-- **The total reference outcome of a raw invocation.** -/
def refOutcome {P : Wasm.Profile} (raw : RawInvocationBody P) : RefOutcome :=
  if refValid raw then
    match decodeHeader raw.bytes with
    | none => { status := 1, detail := StatusDetailOf P raw, writesC := false }
    | some h =>
      match signalingScan (descriptorOf h) (entryStore raw) with
      | some (off, bits, mask) =>
          { status := 5, detail := arithmeticDetail off bits mask, writesC := false }
      | none =>
        match (descriptorOf h).mode with
        | .checked =>
          match checkedFirstFailure (descriptorOf h) (entryStore raw) with
          | some (x, z, bits) =>
              { status := 4
              , detail := overflowDetail (x.linear (descriptorOf h).shapeC)
                  (bitsRequired (descriptorOf h).accumulatorKind.isSignedInteger z) bits
              , writesC := false }
          | none => { status := 0, detail := StatusDetail.zero, writesC := true }
        | _ => { status := 0, detail := StatusDetail.zero, writesC := true }
  else
    { status := rejectStatus raw, detail := StatusDetailOf P raw, writesC := false }

/-- **SPEC §8.4**: the exact returned status of a raw invocation. -/
def referenceStatus {P : Wasm.Profile} (raw : RawInvocationBody P) : UInt32 :=
  UInt32.ofNat (refOutcome raw).status

/-- **SPEC §8.3**: the phase the reference terminates in. -/
def referencePhase {P : Wasm.Profile} (raw : RawInvocationBody P) : TerminalPhase :=
  if (refOutcome raw).status = 0 then .success
  else if (refOutcome raw).status = 4 ∨ (refOutcome raw).status = 5 then .runtimeFailure
  else if statusTrustworthy raw then .descriptorRejected
  else .noTrustworthyStatus

/-! ## The prescribed final store -/

/-- Write the 32-byte status-detail record at `start`. -/
def writeRecordAt (start : Nat) (r : StatusDetail) (s : Store) : Store :=
  fun i =>
    if start ≤ i ∧ i < start + statusDetailBytes then
      (encodeStatusDetailList r).getD (i - start) 0
    else s i

/-- Write every logical `C` element.  Every element is evaluated against the
*entry* store, which is SPEC §8.3's "immutable logical snapshot … taken before
the first C write". -/
def writeCAll (d : DescriptorBody) (f : Index → Nat) (s : Store) : Store :=
  (Index.enumerate d.shapeC).foldl
    (fun acc x => writeLE acc (elemAddr d.cView false x) d.cKind.byteWidth (f x)) s

/-- The store after the `C` phase: every logical `C` element is evaluated
against the *entry* store, which is SPEC §8.3's "immutable logical snapshot …
taken before the first C write". -/
def referenceAfterC {P : Wasm.Profile} (raw : RawInvocationBody P) : Store :=
  if (refOutcome raw).writesC then
    match decodeHeader raw.bytes with
    | some h =>
        writeCAll (descriptorOf h) (referenceElem (descriptorOf h) (entryStore raw))
          (entryStore raw)
    | none => entryStore raw
  else entryStore raw

/-- **The complete final ABI-visible store prescribed by the reference.** -/
def referenceFinalStore {P : Wasm.Profile} (raw : RawInvocationBody P) : Store :=
  if statusTrustworthy raw then
    writeRecordAt (statusRange raw).start (refOutcome raw).detail (referenceAfterC raw)
  else referenceAfterC raw

/-- The entry store the harness must have installed: exactly the raw bytes at
`ptr`, and nothing outside the invocation extent. -/
def entryObservable {P : Wasm.Profile} (raw : RawInvocationBody P) : Store :=
  fun i => if i < raw.len.toNat then entryStore raw i else 0

/-- The reference *observation*: SPEC §8.3's scratch masking, applied only
after complete descriptor validation, over the invocation extent. -/
def referenceObservableStore {P : Wasm.Profile} (raw : RawInvocationBody P) : Store :=
  fun i =>
    if i < raw.len.toNat then
      (if refValid raw then maskScratch (regionsOfRaw raw) (referenceFinalStore raw)
       else referenceFinalStore raw) i
    else 0

/-! ## The reference never writes outside the sanctioned set -/

theorem writeRecordAt_outside {start : Nat} {r : StatusDetail} {s : Store} {i : Nat}
    (h : ¬ (start ≤ i ∧ i < start + statusDetailBytes)) :
    writeRecordAt start r s i = s i := by simp [writeRecordAt, h]

/-- A `C` write lands in one logical element's byte range. -/
theorem writeCAll_mem {d : DescriptorBody} {f : Index → Nat} :
    ∀ (l : List Index) (s : Store) (i : Nat),
      l.foldl (fun acc x => writeLE acc (elemAddr d.cView false x) d.cKind.byteWidth (f x))
        s i ≠ s i →
      ∃ x ∈ l, elemAddr d.cView false x ≤ i ∧
        i < elemAddr d.cView false x + d.cKind.byteWidth := by
  intro l
  induction l with
  | nil => intro s i h; exact absurd rfl h
  | cons a t ih =>
    intro s i h
    simp only [List.foldl_cons] at h
    by_cases hstep : writeLE s (elemAddr d.cView false a) d.cKind.byteWidth (f a) i = s i
    · by_cases hrest :
        t.foldl (fun acc x => writeLE acc (elemAddr d.cView false x) d.cKind.byteWidth (f x))
          (writeLE s (elemAddr d.cView false a) d.cKind.byteWidth (f a)) i
          = writeLE s (elemAddr d.cView false a) d.cKind.byteWidth (f a) i
      · exact absurd (hrest.trans hstep) h
      · obtain ⟨x, hx, hmem⟩ := ih _ i hrest
        exact ⟨x, List.mem_cons_of_mem _ hx, hmem⟩
    · refine ⟨a, List.mem_cons_self, ?_⟩
      by_cases hc : elemAddr d.cView false a ≤ i ∧
          i < elemAddr d.cView false a + d.cKind.byteWidth
      · exact hc
      · exact absurd (writeLE_outside hc) hstep

/-- Under the layout predicate every written `C` byte lies in the declared `C`
range. -/
theorem elem_byte_in_c {d : DescriptorBody} {x : Index} {i : Nat}
    (hin : View.ElementsInInterval d.cView d.shapeC false d.cKind.byteWidth)
    (hx : x.Mem d.shapeC)
    (h : elemAddr d.cView false x ≤ i ∧
      i < elemAddr d.cView false x + d.cKind.byteWidth) :
    (ByteRange.ofView d.cView).Mem i := by
  obtain ⟨h1, h2⟩ := hin x hx
  obtain ⟨h3, h4⟩ := h
  simp only [elemAddr] at h3 h4
  simp only [ByteRange.ofView, ByteRange.Mem, ByteRange.stop]
  omega

/-- `refValid` gives the descriptor's well-formedness. -/
theorem wellFormed_of_refValid {P : Wasm.Profile} {raw : RawInvocationBody P}
    {h : RawHeader} (hdec : decodeHeader raw.bytes = some h) (hv : refValid raw = true) :
    DescriptorWellFormed (descriptorOf h) (windowOf P raw) := by
  unfold refValid at hv
  rw [hdec] at hv
  simp only [Bool.and_eq_true, beq_iff_eq, decide_eq_true_eq] at hv
  exact hv.1.2

theorem decodeHeader_of_refValid {P : Wasm.Profile} {raw : RawInvocationBody P}
    (hv : refValid raw = true) : ∃ h, decodeHeader raw.bytes = some h := by
  unfold refValid at hv
  split at hv
  · exact absurd hv (by simp)
  · next hd hdec => exact ⟨hd, hdec⟩

/-- A validated descriptor always has a trustworthy status range. -/
theorem statusTrustworthy_of_refValid {P : Wasm.Profile} {raw : RawInvocationBody P}
    (hv : refValid raw = true) : statusTrustworthy raw = true := by
  unfold refValid at hv
  unfold statusTrustworthy
  split at hv
  · exact absurd hv (by simp)
  · simp only [Bool.and_eq_true, beq_iff_eq, decide_eq_true_eq] at hv
    have hwf := hv.1.2
    obtain ⟨hc, _⟩ := hwf.aliasOk
    obtain ⟨_, hhs, _⟩ := hc
    simp only [Bool.and_eq_true, decide_eq_true_eq]
    exact ⟨⟨hwf.statusIsRecord, hwf.statusInside⟩, hhs⟩

/-- The status of a validated raw input is `0`, `4` or `5`. -/
theorem refOutcome_status_of_refValid {P : Wasm.Profile} {raw : RawInvocationBody P}
    (hv : refValid raw = true) :
    (refOutcome raw).status = 0 ∨ (refOutcome raw).status = 4 ∨
      (refOutcome raw).status = 5 := by
  obtain ⟨hd, hdec⟩ := decodeHeader_of_refValid hv
  unfold refOutcome
  simp only [if_pos hv, hdec]
  repeat' split
  all_goals simp

/-- Only the successful phase writes `C`. -/
theorem writesC_status {P : Wasm.Profile} (raw : RawInvocationBody P) :
    (refOutcome raw).writesC = true → (refOutcome raw).status = 0 := by
  unfold refOutcome
  repeat' split
  all_goals first
    | (intro _; rfl)
    | (intro hcon; exact absurd hcon (by simp))

/-- Every status the reference returns is one of the six sanctioned results. -/
theorem refOutcome_status_lt {P : Wasm.Profile} (raw : RawInvocationBody P) :
    (refOutcome raw).status < 6 := by
  unfold refOutcome rejectStatus
  repeat' split
  all_goals simp

/-- **Status `0` is returned only for a descriptor the classifier accepts.**  A
malformed, unsupported or resource-invalid raw input can never be reported as a
successful GEMM. -/
theorem refOutcome_status_zero_valid {P : Wasm.Profile} {raw : RawInvocationBody P}
    (h : (refOutcome raw).status = 0) : refValid raw = true := by
  by_cases hv : refValid raw = true
  · exact hv
  · simp only [Bool.not_eq_true] at hv
    unfold refOutcome at h
    rw [if_neg (by simp [hv])] at h
    simp only at h
    unfold rejectStatus at h
    split at h
    · next inv hcl => exact absurd (refValid_of_classify hcl) (by simp [hv])
    · exact absurd h (by simp)
    · exact absurd h (by simp)
    · exact absurd h (by simp)

/-- The reference status round-trips through `UInt32`. -/
theorem referenceStatus_toNat {P : Wasm.Profile} (raw : RawInvocationBody P) :
    (referenceStatus raw).toNat = (refOutcome raw).status := by
  have h := refOutcome_status_lt raw
  have hsize : UInt32.size = 4294967296 := rfl
  simp only [referenceStatus]
  exact UInt32.toNat_ofNat_of_lt' (by omega)

/-- The declared status range really is 32 bytes once it is trustworthy. -/
theorem statusRange_length {P : Wasm.Profile} {raw : RawInvocationBody P}
    (h : statusTrustworthy raw = true) : (statusRange raw).length = statusDetailBytes := by
  unfold statusTrustworthy at h
  cases hdec : decodeHeader raw.bytes with
  | none => rw [hdec] at h; exact absurd h (by simp)
  | some hd =>
    rw [hdec] at h
    simp only [Bool.and_eq_true, decide_eq_true_eq] at h
    simp only [statusRange, regionsOfRaw, hdec, DescriptorBody.regions]
    exact h.1.1

theorem statusDetail_mem {P : Wasm.Profile} {raw : RawInvocationBody P} {i : Nat}
    (ht : statusTrustworthy raw = true)
    (h : (statusRange raw).start ≤ i ∧ i < (statusRange raw).start + statusDetailBytes) :
    (regionsOfRaw raw).statusDetail.Mem i := by
  have hlen := statusRange_length ht
  refine ⟨h.1, ?_⟩
  have hstop : (statusRange raw).stop = (statusRange raw).start + statusDetailBytes := by
    simp only [ByteRange.stop, hlen]
  have h2 := h.2
  rw [← hstop] at h2
  exact h2

/-- **The reference stays inside SPEC §8.3's sanctioned write set.** -/
theorem reference_writes_within {P : Wasm.Profile} (raw : RawInvocationBody P) (i : Nat)
    (hne : entryObservable raw i ≠ referenceObservableStore raw i) :
    InRegions (sanctionedWriteRegions (regionsOfRaw raw) (referencePhase raw)) i := by
  by_cases hlen : i < raw.len.toNat
  · simp only [entryObservable, referenceObservableStore, if_pos hlen] at hne
    by_cases hv : refValid raw = true
    · rw [if_pos hv] at hne
      have htrust := statusTrustworthy_of_refValid hv
      have hstatus := refOutcome_status_of_refValid hv
      by_cases hsc : (regionsOfRaw raw).scratch.Mem i
      · rcases hstatus with h0 | h4 | h5
        · exact ⟨_, by simp [referencePhase, h0], hsc⟩
        · exact ⟨_, by simp [referencePhase, h4], hsc⟩
        · exact ⟨_, by simp [referencePhase, h5], hsc⟩
      · rw [maskScratch_of_not_mem hsc] at hne
        by_cases hst : (statusRange raw).start ≤ i ∧
            i < (statusRange raw).start + statusDetailBytes
        · have hmem := statusDetail_mem htrust hst
          rcases hstatus with h0 | h4 | h5
          · exact ⟨_, by simp [referencePhase, h0], hmem⟩
          · exact ⟨_, by simp [referencePhase, h4], hmem⟩
          · exact ⟨_, by simp [referencePhase, h5], hmem⟩
        · rw [referenceFinalStore, if_pos htrust, writeRecordAt_outside hst] at hne
          by_cases hwc : (refOutcome raw).writesC = true
          · have h0 := writesC_status raw hwc
            rw [referenceAfterC, if_pos hwc] at hne
            cases hdec : decodeHeader raw.bytes with
            | none => rw [hdec] at hne; exact absurd rfl hne
            | some hd =>
              rw [hdec] at hne
              have hwf := wellFormed_of_refValid hdec hv
              obtain ⟨x, hx, hbyte⟩ := writeCAll_mem _ _ i (Ne.symm hne)
              have hxm : x.Mem (descriptorOf hd).shapeC :=
                (Index.mem_enumerate_iff _ _).mp hx
              have hmem : (regionsOfRaw raw).c.Mem i := by
                have := elem_byte_in_c hwf.layoutC.elementsInInterval hxm hbyte
                simpa [regionsOfRaw, hdec, DescriptorBody.regions] using this
              exact ⟨_, by simp [referencePhase, h0], hmem⟩
          · simp only [Bool.not_eq_true] at hwc
            rw [referenceAfterC, if_neg (by simp [hwc])] at hne
            exact absurd rfl hne
    · simp only [Bool.not_eq_true] at hv
      rw [if_neg (by simp [hv])] at hne
      have hwc : (refOutcome raw).writesC = false := by
        unfold refOutcome; rw [if_neg (by simp [hv])]
      by_cases htrust : statusTrustworthy raw = true
      · rw [referenceFinalStore, if_pos htrust] at hne
        by_cases hst : (statusRange raw).start ≤ i ∧
            i < (statusRange raw).start + statusDetailBytes
        · have hmem := statusDetail_mem htrust hst
          unfold referencePhase
          by_cases h0 : (refOutcome raw).status = 0
          · exact ⟨_, by simp [h0], hmem⟩
          · by_cases h45 : (refOutcome raw).status = 4 ∨ (refOutcome raw).status = 5
            · exact ⟨_, by simp [h0, h45], hmem⟩
            · exact ⟨_, by simp [h0, h45, htrust], hmem⟩
        · rw [writeRecordAt_outside hst, referenceAfterC, if_neg (by simp [hwc])] at hne
          exact absurd rfl hne
      · simp only [Bool.not_eq_true] at htrust
        rw [referenceFinalStore, if_neg (by simp [htrust]), referenceAfterC,
          if_neg (by simp [hwc])] at hne
        exact absurd rfl hne
  · simp only [entryObservable, referenceObservableStore, if_neg hlen] at hne
    exact absurd rfl hne

/-! ## List helpers for the observation construction -/

theorem getD_replicate_append : ∀ (n : Nat) (l : List UInt8) (i : Nat),
    (List.replicate n 0 ++ l).getD (n + i) 0 = l.getD i 0 := by
  intro n
  induction n with
  | zero => intro l i; simp
  | succ m ih =>
    intro l i
    simp only [List.replicate_succ, List.cons_append, Nat.succ_add, List.getD_cons_succ]
    exact ih l i

theorem getD_replicate_append_lt : ∀ (n : Nat) (l : List UInt8) (i : Nat), i < n →
    (List.replicate n 0 ++ l).getD i 0 = 0 := by
  intro n
  induction n with
  | zero => intro l i h; omega
  | succ m ih =>
    intro l i h
    cases i with
    | zero => simp [List.replicate_succ]
    | succ j =>
      simp only [List.replicate_succ, List.cons_append, List.getD_cons_succ]
      exact ih l j (by omega)

theorem getD_ge {l : List UInt8} {i : Nat} (h : l.length ≤ i) : l.getD i 0 = 0 := by
  simp [List.getD_eq_getElem?_getD, List.getElem?_eq_none h]

theorem getD_map_range {f : Nat → UInt8} {n i : Nat} (h : i < n) :
    ((List.range n).map f).getD i 0 = f i := by
  simp [List.getD_eq_getElem?_getD, h]

theorem toList_length (b : ByteArray) : b.toList.length = b.size := by
  rw [Foundation.Bytes.byteArray_toList]
  simp

/-! ## From an execution observation to a GEMM semantic observation -/

/-- The ABI-visible store an execution observation exposes, as a function of the
byte offset relative to `ptr`, restricted to the invocation extent. -/
def storeOfObservable {P : Wasm.Profile} (raw : RawInvocationBody P)
    (os : Wasm.ObservableStore) : Store :=
  fun i => if i < raw.len.toNat then os.bytes.getD (raw.ptr.toNat + i) 0 else 0

/-- SPEC §8.3's masking: scratch is masked out of the semantic observation, and
only after complete descriptor validation. -/
def maskFor {P : Wasm.Profile} (raw : RawInvocationBody P) (s : Store) : Store :=
  if refValid raw then maskScratch (regionsOfRaw raw) s else s

/-- **SPEC §8.4**, the semantic observation of a run: the returned status
together with the entry and final ABI-visible stores, or the fact that the run
trapped or threw. -/
inductive SemanticOutcome
  /-- The run trapped or threw before the `gemm` entry boundary. -/
  | beforeEntry
  /-- The run trapped or threw inside `gemm`. -/
  | faulted (entry final : Store)
  /-- The run returned a status. -/
  | returned (value : UInt32) (entry final : Store)

/-- **SPEC §8.4**, `Gemm.semanticFor`. -/
def semanticFor {P : Wasm.Profile} (_problem : Problem P) (raw : Gemm.RawInvocation P)
    (o : Wasm.ExecutionObservation) : SemanticOutcome :=
  match o with
  | .returned _ entry v final _ =>
      .returned v (storeOfObservable raw.body entry)
        (maskFor raw.body (storeOfObservable raw.body final))
  | .trappedBeforeEntry _ _ _ _ => .beforeEntry
  | .thrownBeforeEntry _ _ _ _ => .beforeEntry
  | .trappedAfterEntry _ entry _ final _ =>
      .faulted (storeOfObservable raw.body entry)
        (maskFor raw.body (storeOfObservable raw.body final))
  | .thrownAfterEntry _ entry _ final _ =>
      .faulted (storeOfObservable raw.body entry)
        (maskFor raw.body (storeOfObservable raw.body final))

namespace Problem

variable {P : Wasm.Profile}

/-- The terminal phase a semantic observation lands in (SPEC §8.3's four
cases). -/
def terminalPhase (_problem : Problem P) (raw : Gemm.RawInvocation P)
    (out : SemanticOutcome) : TerminalPhase :=
  match out with
  | .returned v _ _ =>
      if v.toNat = 0 then .success
      else if v.toNat = 4 ∨ v.toNat = 5 then .runtimeFailure
      else if statusTrustworthy raw.body then .descriptorRejected
      else .noTrustworthyStatus
  | _ => .noTrustworthyStatus

/-- **SPEC §8.3/§8.4**, `problem.SanctionedWriteRegions`: the closed
phase-dependent function, read out of the problem's own first-order table. -/
def SanctionedWriteRegions (problem : Problem P) (raw : Gemm.RawInvocation P)
    (out : SemanticOutcome) : List ByteRange :=
  (selectorFor problem.body.sanctionedWriteTable (problem.terminalPhase raw out)).regions
    (regionsOfRaw raw.body)

/-- The problem's stored table *is* SPEC §8.3's closed function. -/
theorem sanctionedWriteRegions_eq (problem : Problem P) (raw : Gemm.RawInvocation P)
    (out : SemanticOutcome) :
    problem.SanctionedWriteRegions raw out =
      sanctionedWriteRegions (regionsOfRaw raw.body) (problem.terminalPhase raw out) := by
  simp only [SanctionedWriteRegions, lawful_sanctionedWriteTable problem]
  exact selectorFor_release _ _

/-- `Problem.regionsOf` of `Gemm/Problem.lean` is the raw-level region map. -/
theorem regionsOf_eq (problem : Problem P) (raw : Gemm.RawInvocation P) :
    problem.regionsOf raw = regionsOfRaw raw.body := rfl

/-- The phase of the reference observation is the reference phase. -/
theorem terminalPhase_reference (problem : Problem P) (raw : Gemm.RawInvocation P)
    (e f : Store) :
    problem.terminalPhase raw (.returned (referenceStatus raw.body) e f) =
      referencePhase raw.body := by
  simp only [terminalPhase, referencePhase, referenceStatus_toNat]

end Problem

end WasmGemmGnaf.Gemm

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf

/-!
## Candidate-call write confinement (SPEC §8.4)

> `Wasm.CandidateCallMemoryWritesWithin observation regions` is defined from the
> post-`gemm-entry` candidate write events and the bytewise difference between
> `observation.gemmEntryObservableStore` and the final observable store.  It
> requires every candidate-call write event and every changed ABI-visible byte
> to lie in `regions`; it is false for `trappedBeforeEntry`.

**Scope of the write-event half.**  The released `Wasm.Event` alphabet
(`Wasm/Config.lean`) labels *reductions*, not byte writes: its seven
constructors are `step`, `branch`, `growAttempt`, `enterGemm`, `trapEvent`,
`throwEvent` and `returnEvent`, and `event_names_no_memory_address` below proves
that this list is exhaustive.  No constructor names a memory address, so the
"write event" conjunct of SPEC §8.4 has **no content in this model** and is not
stated here as a separate conjunct; the whole obligation is carried by the
bytewise-difference conjunct, which is stated over the complete memory, not
only over the invocation window.  A future event alphabet that does name
addresses would need this predicate extended — it would *not* be implied by
what is proved here.
-/

/-- The released event alphabet, exhaustively.  No constructor carries a memory
address. -/
theorem event_names_no_memory_address (e : Event) :
    e = .step ∨ e = .branch ∨ (∃ o, e = .growAttempt o) ∨ e = .enterGemm ∨
      (∃ t, e = .trapEvent t) ∨ (∃ v, e = .throwEvent v) ∨ (∃ v, e = .returnEvent v) := by
  cases e with
  | step => exact Or.inl rfl
  | branch => exact Or.inr (Or.inl rfl)
  | growAttempt o => exact Or.inr (Or.inr (Or.inl ⟨o, rfl⟩))
  | enterGemm => exact Or.inr (Or.inr (Or.inr (Or.inl rfl)))
  | trapEvent t => exact Or.inr (Or.inr (Or.inr (Or.inr (Or.inl ⟨t, rfl⟩))))
  | throwEvent v => exact Or.inr (Or.inr (Or.inr (Or.inr (Or.inr (Or.inl ⟨v, rfl⟩)))))
  | returnEvent v =>
      exact Or.inr (Or.inr (Or.inr (Or.inr (Or.inr (Or.inr ⟨v, rfl⟩)))))

/-- **SPEC §8.4.**  Every ABI-visible byte the candidate call changed lies in
`regions`, and no byte outside the invocation window changed at all.  The
region offsets are relative to `window.ptr`, which is why the window is an
explicit argument. -/
def CandidateCallMemoryWritesWithin (o : ExecutionObservation)
    (window : Gemm.MemoryWindow) (regions : List Gemm.ByteRange) : Prop :=
  ∃ entry : ObservableStore,
    o.gemmEntryObservableStore? = some entry ∧
    (∀ i : Nat, i < window.len →
      entry.bytes.getD (window.ptr + i) 0 ≠
        o.finalObservableStore.bytes.getD (window.ptr + i) 0 →
      Gemm.InRegions regions i) ∧
    (∀ j : Nat, (j < window.ptr ∨ window.ptr + window.len ≤ j) →
      entry.bytes.getD j 0 = o.finalObservableStore.bytes.getD j 0)

/-- **SPEC §8.4**: the predicate is false for `trappedBeforeEntry`. -/
theorem not_candidateCallMemoryWritesWithin_trappedBeforeEntry
    (tr : List Event) (t : Trap) (store : ObservableStore) (eff : ObservableEffects)
    (window : Gemm.MemoryWindow) (regions : List Gemm.ByteRange) :
    ¬ CandidateCallMemoryWritesWithin (.trappedBeforeEntry tr t store eff) window regions := by
  rintro ⟨entry, hentry, -, -⟩
  exact absurd hentry (by simp [ExecutionObservation.gemmEntryObservableStore?])

/-- … and for `thrownBeforeEntry`. -/
theorem not_candidateCallMemoryWritesWithin_thrownBeforeEntry
    (tr : List Event) (v : ExceptionValue) (store : ObservableStore)
    (eff : ObservableEffects) (window : Gemm.MemoryWindow)
    (regions : List Gemm.ByteRange) :
    ¬ CandidateCallMemoryWritesWithin (.thrownBeforeEntry tr v store eff) window regions := by
  rintro ⟨entry, hentry, -, -⟩
  exact absurd hentry (by simp [ExecutionObservation.gemmEntryObservableStore?])

end WasmGemmGnaf.Wasm

namespace WasmGemmGnaf.Gemm

/-! ## The reference relation (SPEC §8.4) -/

/--
  **SPEC §8.4**, `ReferenceSemanticAccepts`.

  The run returned; its entry store is exactly the raw bytes the harness
  installed; and its final ABI-visible store is exactly the store the reference
  prescribes — the complete final `C` byte region for a valid descriptor under
  the descriptor's exact arithmetic, the exact 32-byte status-detail record, and
  the entry bytes everywhere else.  A trapped or uncaught-exception observation
  has no `returned` semantic outcome and is therefore rejected.
-/
def ReferenceSemanticAccepts {P : Wasm.Profile} (_problem : Problem P)
    (raw : Gemm.RawInvocation P) (out : SemanticOutcome) : Prop :=
  ∃ entry final : Store,
    out = .returned (referenceStatus raw.body) entry final ∧
    (∀ i, entry i = entryObservable raw.body i) ∧
    (∀ i, final i = referenceObservableStore raw.body i)

namespace Reference

/-- **SPEC §8.4**, `Gemm.Reference.Accepts`. -/
def Accepts {P : Wasm.Profile} (problem : Problem P) (raw : Gemm.RawInvocation P)
    (observation : Wasm.ExecutionObservation) : Prop :=
  ReferenceSemanticAccepts problem raw (semanticFor problem raw observation) ∧
  Wasm.CandidateCallMemoryWritesWithin observation (windowOf P raw.body)
    (problem.SanctionedWriteRegions raw (semanticFor problem raw observation))

end Reference

/-! ## The prescribed observation

`referenceObservation` realizes the reference: it is the observation a correct
candidate must produce.  Its existence is what makes `Accepts` *total*, and it
is what forbids the reference relation from being vacuously unsatisfiable. -/

/-- The entry bytes of the prescribed observation. -/
def prescribedEntryBytes {P : Wasm.Profile} (raw : RawInvocationBody P) : List UInt8 :=
  List.replicate raw.ptr.toNat 0 ++ raw.bytes.toList

/-- The final bytes of the prescribed observation. -/
def prescribedFinalBytes {P : Wasm.Profile} (raw : RawInvocationBody P) : List UInt8 :=
  List.replicate raw.ptr.toNat 0 ++
    (List.range raw.len.toNat).map (referenceObservableStore raw)

/-- **The observation the reference prescribes.** -/
def referenceObservation {P : Wasm.Profile} (raw : Gemm.RawInvocation P) :
    Wasm.ExecutionObservation :=
  .returned [Wasm.Event.enterGemm] ⟨prescribedEntryBytes raw.body⟩
    (referenceStatus raw.body) ⟨prescribedFinalBytes raw.body⟩ Wasm.ObservableEffects.none'

section Prescribed

variable {P : Wasm.Profile} (raw : Gemm.RawInvocation P)

theorem prescribedEntryBytes_length :
    (prescribedEntryBytes raw.body).length = raw.body.ptr.toNat + raw.body.len.toNat := by
  simp only [prescribedEntryBytes, List.length_append, List.length_replicate,
    toList_length]
  rw [raw.lawful.1]

theorem prescribedFinalBytes_length :
    (prescribedFinalBytes raw.body).length = raw.body.ptr.toNat + raw.body.len.toNat := by
  simp [prescribedFinalBytes]

theorem prescribed_entry_store (i : Nat) :
    storeOfObservable raw.body ⟨prescribedEntryBytes raw.body⟩ i =
      entryObservable raw.body i := by
  simp only [storeOfObservable, entryObservable, prescribedEntryBytes, entryStore]
  by_cases h : i < raw.body.len.toNat
  · rw [if_pos h, if_pos h, getD_replicate_append]
  · rw [if_neg h, if_neg h]

theorem prescribed_final_store (i : Nat) :
    storeOfObservable raw.body ⟨prescribedFinalBytes raw.body⟩ i =
      referenceObservableStore raw.body i := by
  simp only [storeOfObservable, prescribedFinalBytes]
  by_cases h : i < raw.body.len.toNat
  · rw [if_pos h, getD_replicate_append, getD_map_range h]
  · rw [if_neg h]
    simp only [referenceObservableStore, if_neg h]

theorem prescribed_masked_final (i : Nat) :
    maskFor raw.body (storeOfObservable raw.body ⟨prescribedFinalBytes raw.body⟩) i =
      referenceObservableStore raw.body i := by
  simp only [maskFor]
  by_cases hv : refValid raw.body = true
  · rw [if_pos hv]
    by_cases hsc : (regionsOfRaw raw.body).scratch.Mem i
    · rw [maskScratch_of_mem hsc]
      by_cases h : i < raw.body.len.toNat
      · simp only [referenceObservableStore, if_pos h, if_pos hv,
          maskScratch_of_mem hsc]
      · simp only [referenceObservableStore, if_neg h]
    · rw [maskScratch_of_not_mem hsc]
      exact prescribed_final_store raw i
  · simp only [Bool.not_eq_true] at hv
    rw [if_neg (by simp [hv])]
    exact prescribed_final_store raw i

theorem semanticFor_referenceObservation (problem : Problem P) :
    semanticFor problem raw (referenceObservation raw) =
      .returned (referenceStatus raw.body)
        (storeOfObservable raw.body ⟨prescribedEntryBytes raw.body⟩)
        (maskFor raw.body (storeOfObservable raw.body ⟨prescribedFinalBytes raw.body⟩)) :=
  rfl

end Prescribed

/-! ## The required theorems (SPEC §8.4) -/

/--
  **SPEC §8.4**, `Gemm.reference_total`.

  `Accepts` is total over raw invocations: *every* raw invocation — malformed,
  truncated, unsupported, resource-invalid or valid — has an accepted
  observation.  The reference relation is therefore nowhere empty, and no
  correctness claim built on it can be vacuous for want of a witness.
-/
theorem reference_total {P : Wasm.Profile} (problem : Problem P)
    (raw : Gemm.RawInvocation P) :
    ∃ observation, Reference.Accepts problem raw observation := by
  refine ⟨referenceObservation raw, ?_, ?_⟩
  · refine ⟨_, _, rfl, ?_, ?_⟩
    · exact prescribed_entry_store raw
    · exact prescribed_masked_final raw
  · refine ⟨⟨prescribedEntryBytes raw.body⟩, rfl, ?_, ?_⟩
    · intro i hi hne
      have hi' : i < raw.body.len.toNat := hi
      have hentry : (prescribedEntryBytes raw.body).getD (raw.body.ptr.toNat + i) 0
          = entryObservable raw.body i := by
        have h := prescribed_entry_store raw i
        simp only [storeOfObservable, if_pos hi'] at h
        exact h
      have hfinal : (prescribedFinalBytes raw.body).getD (raw.body.ptr.toNat + i) 0
          = referenceObservableStore raw.body i := by
        have h := prescribed_final_store raw i
        simp only [storeOfObservable, if_pos hi'] at h
        exact h
      have hne' : (prescribedEntryBytes raw.body).getD (raw.body.ptr.toNat + i) 0 ≠
          (prescribedFinalBytes raw.body).getD (raw.body.ptr.toNat + i) 0 := by
        simpa only [referenceObservation,
          Wasm.ExecutionObservation.finalObservableStore, windowOf] using hne
      have hdiff : entryObservable raw.body i ≠ referenceObservableStore raw.body i := by
        rw [← hentry, ← hfinal]; exact hne'
      have hin := reference_writes_within raw.body i hdiff
      rw [problem.sanctionedWriteRegions_eq raw _,
        semanticFor_referenceObservation raw problem,
        problem.terminalPhase_reference raw _ _]
      exact hin
    · intro j hj
      simp only [referenceObservation, Wasm.ExecutionObservation.finalObservableStore,
        windowOf] at hj ⊢
      rcases hj with hlt | hge
      · simp only [prescribedEntryBytes, prescribedFinalBytes]
        rw [getD_replicate_append_lt _ _ _ hlt, getD_replicate_append_lt _ _ _ hlt]
      · rw [getD_ge (by rw [prescribedEntryBytes_length raw]; exact hge),
          getD_ge (by rw [prescribedFinalBytes_length raw]; exact hge)]

/--
  **SPEC §8.4**, `Gemm.valid_reference_nonempty`.  A valid classification has an
  accepted observation.
-/
theorem valid_reference_nonempty {P : Wasm.Profile} (problem : Problem P)
    {raw : Gemm.RawInvocation P} {invocation : ValidInvocation P}
    (_h : classify P raw.body = .valid invocation) :
    ∃ observation, Reference.Accepts problem raw observation :=
  reference_total problem raw

/--
  **SPEC §8.4**, `Gemm.deterministic_mode_unique`.  Every released mode is
  deterministic, and two accepted observations of one raw invocation have the
  same semantic observation: the same status, the same entry store and the same
  final ABI-visible store.
-/
theorem deterministic_mode_unique {P : Wasm.Profile} (problem : Problem P)
    {raw : Gemm.RawInvocation P} {a b : Wasm.ExecutionObservation}
    (_hmode : problem.ModeDeterministic raw)
    (ha : Reference.Accepts problem raw a) (hb : Reference.Accepts problem raw b) :
    semanticFor problem raw a = semanticFor problem raw b := by
  obtain ⟨ea, fa, hea, hea', hfa⟩ := ha.1
  obtain ⟨eb, fb, heb, heb', hfb⟩ := hb.1
  have h1 : ea = eb := funext fun i => (hea' i).trans (heb' i).symm
  have h2 : fa = fb := funext fun i => (hfa i).trans (hfb i).symm
  rw [hea, heb, h1, h2]

/--
  **SPEC §8.4**, `Gemm.reference_memory_safe`.  Acceptance implies that every
  candidate-call write stayed inside the sanctioned regions.
-/
theorem reference_memory_safe {P : Wasm.Profile} (problem : Problem P)
    {raw : Gemm.RawInvocation P} {observation : Wasm.ExecutionObservation}
    (h : Reference.Accepts problem raw observation) :
    Wasm.CandidateCallMemoryWritesWithin observation (windowOf P raw.body)
      (problem.SanctionedWriteRegions raw (semanticFor problem raw observation)) :=
  h.2

/-! ### Trapped and uncaught-exception observations are rejected -/

theorem reference_rejects_trappedBeforeEntry {P : Wasm.Profile} (problem : Problem P)
    (raw : Gemm.RawInvocation P) (tr : List Wasm.Event) (t : Wasm.Trap)
    (store : Wasm.ObservableStore) (eff : Wasm.ObservableEffects) :
    ¬ Reference.Accepts problem raw (.trappedBeforeEntry tr t store eff) := by
  rintro ⟨⟨e, f, hcon, -, -⟩, -⟩
  exact absurd hcon (by simp [semanticFor])

theorem reference_rejects_thrownBeforeEntry {P : Wasm.Profile} (problem : Problem P)
    (raw : Gemm.RawInvocation P) (tr : List Wasm.Event) (v : Wasm.ExceptionValue)
    (store : Wasm.ObservableStore) (eff : Wasm.ObservableEffects) :
    ¬ Reference.Accepts problem raw (.thrownBeforeEntry tr v store eff) := by
  rintro ⟨⟨e, f, hcon, -, -⟩, -⟩
  exact absurd hcon (by simp [semanticFor])

theorem reference_rejects_trappedAfterEntry {P : Wasm.Profile} (problem : Problem P)
    (raw : Gemm.RawInvocation P) (tr : List Wasm.Event) (entry : Wasm.ObservableStore)
    (t : Wasm.Trap) (store : Wasm.ObservableStore) (eff : Wasm.ObservableEffects) :
    ¬ Reference.Accepts problem raw (.trappedAfterEntry tr entry t store eff) := by
  rintro ⟨⟨e, f, hcon, -, -⟩, -⟩
  exact absurd hcon (by simp [semanticFor])

theorem reference_rejects_thrownAfterEntry {P : Wasm.Profile} (problem : Problem P)
    (raw : Gemm.RawInvocation P) (tr : List Wasm.Event) (entry : Wasm.ObservableStore)
    (v : Wasm.ExceptionValue) (store : Wasm.ObservableStore) (eff : Wasm.ObservableEffects) :
    ¬ Reference.Accepts problem raw (.thrownAfterEntry tr entry v store eff) := by
  rintro ⟨⟨e, f, hcon, -, -⟩, -⟩
  exact absurd hcon (by simp [semanticFor])

/-- The released profile admits no externally visible non-memory effect at all,
so "the absence of every forbidden effect" is a theorem, not a side condition. -/
theorem reference_effects_absent (o : Wasm.ExecutionObservation) :
    o.effects = Wasm.ObservableEffects.none' :=
  Wasm.ObservableEffects.eq_none' _

/-! ## Anti-vacuity, part 1: list and header helpers -/

theorem getD_append_left : ∀ (l r : List UInt8) (i : Nat), i < l.length →
    (l ++ r).getD i 0 = l.getD i 0 := by
  intro l
  induction l with
  | nil => intro r i h; simp at h
  | cons a t ih =>
    intro r i h
    cases i with
    | zero => simp
    | succ j => simpa using ih r j (by simpa using h)

theorem getD_append_right : ∀ (l r : List UInt8) (i : Nat), l.length ≤ i →
    (l ++ r).getD i 0 = r.getD (i - l.length) 0 := by
  intro l
  induction l with
  | nil => intro r i h; simp
  | cons a t ih =>
    intro r i h
    cases i with
    | zero => simp at h
    | succ j =>
      simp only [List.cons_append, List.getD_cons_succ, List.length_cons]
      simpa using ih r j (by simpa using h)

theorem drop_append_left : ∀ (l r : List UInt8) (n : Nat), n ≤ l.length →
    (l ++ r).drop n = l.drop n ++ r := by
  intro l
  induction l with
  | nil => intro r n h; simp only [List.length_nil, Nat.le_zero_eq] at h; simp [h]
  | cons a t ih =>
    intro r n h
    cases n with
    | zero => simp
    | succ j => simpa using ih r j (by simpa using h)

theorem take_append_left : ∀ (l r : List UInt8) (n : Nat), n ≤ l.length →
    (l ++ r).take n = l.take n := by
  intro l
  induction l with
  | nil => intro r n h; simp at h; simp [h]
  | cons a t ih =>
    intro r n h
    cases n with
    | zero => simp
    | succ j => simpa using ih r j (by simpa using h)

theorem fieldValue_append {l r : List UInt8} {off w : Nat} (h : off + w ≤ l.length) :
    fieldValue (l ++ r) off w = fieldValue l off w := by
  simp only [fieldValue]
  rw [drop_append_left l r off (by omega), take_append_left _ _ _ (by simp; omega)]

/-- The header decodes out of any byte string that starts with its encoding. -/
theorem decodeHeaderList_append (h : RawHeader) (r : List UInt8) :
    decodeHeaderList (encodeHeaderList h ++ r) = some h := by
  have hlen : (encodeHeaderList h).length = 256 := encodeHeaderList_length h
  have key : ∀ off w : Nat, off + w ≤ 256 →
      fieldValue (encodeHeaderList h ++ r) off w = fieldValue (encodeHeaderList h) off w := by
    intro off w hw
    exact fieldValue_append (by rw [hlen]; omega)
  rw [decodeHeaderList, if_pos (by simp [hlen])]
  rw [← abi_roundtrip_list h, decodeHeaderList, if_pos (by omega)]
  simp only [key 0 4 (by omega), key 4 2 (by omega), key 6 2 (by omega),
    key 8 1 (by omega), key 9 1 (by omega), key 10 1 (by omega), key 11 1 (by omega),
    key 12 1 (by omega), key 13 1 (by omega), key 14 1 (by omega), key 15 1 (by omega),
    key 16 8 (by omega), key 24 8 (by omega), key 32 8 (by omega), key 40 8 (by omega),
    key 48 8 (by omega), key 56 8 (by omega), key 64 8 (by omega), key 72 8 (by omega),
    key 80 8 (by omega), key 88 8 (by omega), key 96 8 (by omega), key 104 8 (by omega),
    key 112 8 (by omega), key 120 8 (by omega), key 128 8 (by omega), key 136 8 (by omega),
    key 144 8 (by omega), key 152 8 (by omega), key 160 8 (by omega), key 168 8 (by omega),
    key 176 8 (by omega), key 184 8 (by omega), key 192 8 (by omega), key 200 8 (by omega),
    key 208 8 (by omega), key 216 8 (by omega), key 224 8 (by omega), key 232 8 (by omega),
    key 240 8 (by omega), key 248 8 (by omega)]

theorem pack_size (l : List UInt8) : (Foundation.Bytes.pack l).size = l.length := by
  have h := Foundation.Bytes.data_toList_pack l
  have : (Foundation.Bytes.pack l).data.toList.length = l.length := by rw [h]
  simpa using this

/-- Memory-window irrelevance: descriptor well-formedness looks only at the
invocation extent. -/
theorem DescriptorWellFormed.ofLenEq {d : DescriptorBody} {M M' : MemoryWindow}
    (hlen : M.len = M'.len) (h : DescriptorWellFormed d M) : DescriptorWellFormed d M' where
  layoutA := ⟨hlen ▸ h.layoutA.endpointsRepresentable, h.layoutA.elementsInInterval⟩
  layoutB := ⟨hlen ▸ h.layoutB.endpointsRepresentable, h.layoutB.elementsInInterval⟩
  layoutC := ⟨hlen ▸ h.layoutC.endpointsRepresentable, h.layoutC.elementsInInterval⟩
  cElementsDisjoint := h.cElementsDisjoint
  scratchInside := hlen ▸ h.scratchInside
  statusInside := hlen ▸ h.statusInside
  statusIsRecord := h.statusIsRecord
  headerInside := hlen ▸ h.headerInside
  aliasOk := h.aliasOk
  alphaFits := h.alphaFits
  betaFits := h.betaFits

/-! ## Anti-vacuity, part 2: the mandatory witness family (SPEC §8.3)

> Required anti-vacuity theorems SHALL provide a nonzero `1×1×1` witness for
> every mandatory scalar-kind × compatible arithmetic-mode × transpose ×
> layout-class combination, and SHALL prove that the full C range and status are
> observable.

The family below is generated from the compatibility table itself, and
`mandatoryKindRows_complete` proves the enumeration is *exhaustive*: every
compatible `(mode, stored kind, accumulator)` triple appears.  A witness that
covered only some rows would fail that theorem. -/

/-- One mandatory combination. -/
structure MandatoryCase where
  /-- The shared stored kind of `A`, `B` and `C`. -/
  kind : ScalarKind
  /-- The accumulator kind. -/
  acc : ScalarKind
  /-- The arithmetic mode. -/
  mode : ArithmeticMode
  /-- Transpose bit of `A`. -/
  transposeA : Bool
  /-- Transpose bit of `B`. -/
  transposeB : Bool
  /-- Layout class: declared strides are negative rather than zero. -/
  negativeStrides : Bool
  deriving DecidableEq, Repr, Inhabited

/-- Every `(mode, stored kind, accumulator)` triple of SPEC §8.2's table. -/
def mandatoryKindRows : List (ArithmeticMode × ScalarKind × ScalarKind) :=
  [ (.modular, .i8, .u32), (.modular, .u8, .u32), (.modular, .i16, .u32)
  , (.modular, .u16, .u32), (.modular, .i32, .u32), (.modular, .u32, .u32)
  , (.modular, .i64, .u64), (.modular, .u64, .u64)
  , (.checked, .i8, .i64), (.checked, .i16, .i64), (.checked, .i32, .i64)
  , (.checked, .i64, .i64)
  , (.checked, .u8, .u64), (.checked, .u16, .u64), (.checked, .u32, .u64)
  , (.checked, .u64, .u64)
  , (.strictFloat, .binary16, .binary32), (.strictFloat, .binary16, .binary64)
  , (.strictFloat, .bfloat16, .binary32), (.strictFloat, .bfloat16, .binary64)
  , (.strictFloat, .binary32, .binary32), (.strictFloat, .binary32, .binary64)
  , (.strictFloat, .binary64, .binary64)
  , (.exactDyadicRoundOnce, .binary16, .exactDyadic)
  , (.exactDyadicRoundOnce, .bfloat16, .exactDyadic)
  , (.exactDyadicRoundOnce, .binary32, .exactDyadic)
  , (.exactDyadicRoundOnce, .binary64, .exactDyadic) ]

/-- Every listed row really is compatible. -/
theorem mandatoryKindRows_compatible :
    ∀ r ∈ mandatoryKindRows, Compatible r.1 r.2.1 r.2.1 r.2.1 r.2.2 := by decide

/-- **The enumeration is exhaustive**: every compatible tuple of the released
table is a listed row.  This is what makes the witness family *mandatory*
rather than selective. -/
theorem mandatoryKindRows_complete (m : ArithmeticMode) (k acc : ScalarKind)
    (h : Compatible m k k k acc) : (m, k, acc) ∈ mandatoryKindRows := by
  revert h
  cases m <;> cases k <;> cases acc <;> decide

/-- The complete mandatory family: every row, both transpose bits, both layout
classes. -/
def mandatoryCases : List MandatoryCase :=
  mandatoryKindRows.flatMap (fun r =>
    [false, true].flatMap (fun ta =>
      [false, true].flatMap (fun tb =>
        [false, true].map (fun neg =>
          { kind := r.2.1, acc := r.2.2, mode := r.1
          , transposeA := ta, transposeB := tb, negativeStrides := neg }))))

theorem mandatoryCases_length : mandatoryCases.length = 216 := by decide

/-- Every combination of a compatible row with a transpose pair and a layout
class occurs in the family. -/
theorem mandatoryCases_covers (m : ArithmeticMode) (k acc : ScalarKind)
    (ta tb neg : Bool) (h : Compatible m k k k acc) :
    { kind := k, acc := acc, mode := m, transposeA := ta, transposeB := tb
    , negativeStrides := neg : MandatoryCase } ∈ mandatoryCases := by
  have hrow := mandatoryKindRows_complete m k acc h
  simp only [mandatoryCases, List.mem_flatMap, List.mem_map, List.mem_cons,
    List.not_mem_nil, or_false]
  exact ⟨(m, k, acc), hrow, ta, by cases ta <;> simp, tb, by cases tb <;> simp,
    neg, by cases neg <;> simp, rfl⟩

/-! ### The witness bytes -/

/-- The stored `1` of a kind: the integer one, or the IEEE/bfloat16 `1.0`. -/
def oneBits : ScalarKind → Nat
  | .binary16 => 0x3c00
  | .bfloat16 => 0x3f80
  | .binary32 => 0x3f800000
  | .binary64 => 0x3ff0000000000000
  | _ => 1

theorem oneBits_ne_zero (k : ScalarKind) : oneBits k ≠ 0 := by cases k <;> decide

/-- The declared stride of the layout class: zero, or `-8`.  Both are legal for
a `1×1×1` tensor, and neither is ever applied, so the two classes address the
same byte and differ exactly in the declared layout. -/
def witnessStride (c : MandatoryCase) : UInt64 :=
  if c.negativeStrides then 0xfffffffffffffff8 else 0

/-- The witness header: a `1×1×1` descriptor with disjoint `A`, `B`, `C`,
scratch and status ranges inside a 320-byte invocation. -/
def witnessHeader (c : MandatoryCase) : RawHeader where
  magic := 0x474e4757
  version := 1
  headerSize := 256
  aTag := UInt8.ofNat c.kind.tag
  bTag := UInt8.ofNat c.kind.tag
  cTag := UInt8.ofNat c.kind.tag
  accTag := UInt8.ofNat c.acc.tag
  modeTag := UInt8.ofNat c.mode.tag
  transposeBits := (if c.transposeA then 1 else 0) + (if c.transposeB then 2 else 0)
  aliasTag := 0
  reserved15 := 0
  m := 1
  n := 1
  k := 1
  batch := 1
  aOffset := 256
  aByteLength := 8
  aRowStride := witnessStride c
  aColStride := witnessStride c
  aBatchStride := witnessStride c
  bOffset := 264
  bByteLength := 8
  bRowStride := witnessStride c
  bColStride := witnessStride c
  bBatchStride := witnessStride c
  cOffset := 272
  cByteLength := 8
  cRowStride := witnessStride c
  cColStride := witnessStride c
  cBatchStride := witnessStride c
  alphaBits := UInt64.ofNat (oneBits c.kind)
  alphaPad := 0
  betaBits := 0
  betaPad := 0
  scratchOffset := 280
  scratchLength := 8
  statusOffset := 288
  statusLength := 32
  reserved232 := 0
  reserved240 := 0
  reserved248 := 0

/-- The 64 payload bytes: `A = 1`, `B = 1`, `C = 0`, empty scratch and an
all-zero status record. -/
def witnessPayload (c : MandatoryCase) : List UInt8 :=
  natToBytesLE 8 (oneBits c.kind) ++ natToBytesLE 8 (oneBits c.kind) ++
    List.replicate 48 0

theorem witnessPayload_length (c : MandatoryCase) : (witnessPayload c).length = 64 := by
  simp [witnessPayload]

/-- The complete 320 raw bytes of the witness. -/
def witnessBytes (c : MandatoryCase) : List UInt8 :=
  encodeHeaderList (witnessHeader c) ++ witnessPayload c

theorem witnessBytes_length (c : MandatoryCase) : (witnessBytes c).length = 320 := by
  simp [witnessBytes, encodeHeaderList_length, witnessPayload_length]

/-- The raw invocation body of the witness. -/
def witnessRawBody (P : Wasm.Profile) (c : MandatoryCase) : RawInvocationBody P where
  ptr := 0
  len := 320
  bytes := Foundation.Bytes.pack (witnessBytes c)

theorem witnessRawBody_lawful (P : Wasm.Profile) (c : MandatoryCase) :
    RawInvocationLawful P (witnessRawBody P c) := by
  refine ⟨?_, ?_, ?_⟩
  · simp only [witnessRawBody, pack_size, witnessBytes_length]
    rfl
  · rw [P.addressBits_eq]
    simp only [witnessRawBody]
    decide
  · rw [P.maxPages_eq]
    simp only [witnessRawBody]
    decide

/-- The raw invocation of the witness. -/
def witnessRaw (P : Wasm.Profile) (c : MandatoryCase) : Gemm.RawInvocation P :=
  ⟨witnessRawBody P c, witnessRawBody_lawful P c⟩

theorem witness_decodeHeader (P : Wasm.Profile) (c : MandatoryCase) :
    decodeHeader (witnessRawBody P c).bytes = some (witnessHeader c) := by
  simp only [decodeHeader, witnessRawBody, Foundation.Bytes.toList_pack, witnessBytes]
  exact decodeHeaderList_append _ _

/-- The witness store, written so that every read the reference performs
reduces without ever unfolding the 256 header bytes. -/
def witnessStore (c : MandatoryCase) : Store :=
  fun i =>
    if 256 ≤ i then (witnessPayload c).getD (i - 256) 0
    else (encodeHeaderList (witnessHeader c)).getD i 0

theorem witness_entryStore (P : Wasm.Profile) (c : MandatoryCase) :
    entryStore (witnessRawBody P c) = witnessStore c := by
  funext i
  simp only [entryStore, witnessRawBody, Foundation.Bytes.toList_pack, witnessBytes,
    witnessStore]
  by_cases h : 256 ≤ i
  · rw [if_pos h, getD_append_right _ _ _ (by rw [encodeHeaderList_length]; omega),
      encodeHeaderList_length]
  · rw [if_neg h, getD_append_left _ _ _ (by rw [encodeHeaderList_length]; omega)]

/-- Everything the witness must satisfy, as one closed decidable check.  It
mentions no profile: the profile enters only through the resource predicate,
which is discharged from `Wasm.ProfileLawful`. -/
def witnessOk (c : MandatoryCase) : Bool :=
  (headerMalformed (witnessHeader c) == none) &&
  (tagUnsupported (witnessHeader c) == none) &&
  decide (headerCompatible (witnessHeader c)) &&
  decide (dimensionsRepresentable (witnessHeader c)) &&
  decide (DescriptorWellFormed (descriptorOf (witnessHeader c)) ⟨0, 320, 0⟩) &&
  ((descriptorOf (witnessHeader c)).mode == c.mode) &&
  ((witnessHeader c).aTag.toNat == c.kind.tag) &&
  ((witnessHeader c).bTag.toNat == c.kind.tag) &&
  ((witnessHeader c).cTag.toNat == c.kind.tag) &&
  ((witnessHeader c).accTag.toNat == c.acc.tag) &&
  ((witnessHeader c).modeTag.toNat == c.mode.tag) &&
  ((descriptorOf (witnessHeader c)).transposeA == c.transposeA) &&
  ((descriptorOf (witnessHeader c)).transposeB == c.transposeB) &&
  ((descriptorOf (witnessHeader c)).shapeC == ⟨1, 1, 1⟩) &&
  (signalingScan (descriptorOf (witnessHeader c)) (witnessStore c) == none) &&
  decide (c.mode = .checked →
    checkedFirstFailure (descriptorOf (witnessHeader c)) (witnessStore c) = none) &&
  (cBitsAt (descriptorOf (witnessHeader c)) (witnessStore c) ⟨0, 0, 0⟩ == 0) &&
  (referenceElem (descriptorOf (witnessHeader c)) (witnessStore c) ⟨0, 0, 0⟩
    == oneBits c.kind)

/-- **Every mandatory combination passes.**  This is the computational heart of
the anti-vacuity requirement: for each of the 216 combinations the witness
classifies valid, signals nothing, overflows nothing, starts from `C = 0`, and
computes the nonzero result `1` under that combination's exact arithmetic. -/
theorem witnessOk_mandatory : ∀ c ∈ mandatoryCases, witnessOk c = true := by decide

/-! ### The witness is valid and successful under the released profile -/

theorem witness_resourceOk (P : Wasm.Profile) (c : MandatoryCase) :
    ResourceOk P (witnessRawBody P c) := by
  refine ⟨?_, ?_⟩
  · rw [P.maxInvocationBytes_eq]
    simp only [witnessRawBody]
    decide
  · rw [P.maxPages_eq]
    simp only [witnessRawBody]
    decide

/--
  **SPEC §8.3's anti-vacuity requirement.**

  For *every* mandatory scalar-kind × compatible arithmetic-mode × transpose ×
  layout-class combination there is a `1×1×1` raw invocation that

  * classifies `valid`,
  * returns status `0`,
  * starts from `C = 0`, and
  * computes, under that combination's exact declared arithmetic, the **nonzero**
    result `1` of that kind.

  `mandatoryCases_covers` shows the family really does contain every such
  combination, so no compatible row of SPEC §8.2's table is unexercised.
-/
theorem mandatory_family_nonzero_witnesses (P : Wasm.Profile) :
    ∀ c ∈ mandatoryCases,
      ∃ (raw : Gemm.RawInvocation P) (inv : ValidInvocation P) (h : RawHeader),
        classify P raw.body = .valid inv ∧
        decodeHeader raw.body.bytes = some h ∧
        h.aTag.toNat = c.kind.tag ∧ h.bTag.toNat = c.kind.tag ∧
        h.cTag.toNat = c.kind.tag ∧ h.accTag.toNat = c.acc.tag ∧
        h.modeTag.toNat = c.mode.tag ∧
        (descriptorOf h).transposeA = c.transposeA ∧
        (descriptorOf h).transposeB = c.transposeB ∧
        (descriptorOf h).shapeC = ⟨1, 1, 1⟩ ∧
        cBitsAt (descriptorOf h) (entryStore raw.body) ⟨0, 0, 0⟩ = 0 ∧
        referenceElem (descriptorOf h) (entryStore raw.body) ⟨0, 0, 0⟩ = oneBits c.kind ∧
        oneBits c.kind ≠ 0 ∧
        referenceStatus raw.body = 0 := by
  intro c hc
  have hok := witnessOk_mandatory c hc
  simp only [witnessOk, Bool.and_eq_true, beq_iff_eq, decide_eq_true_eq] at hok
  obtain ⟨⟨⟨⟨⟨⟨⟨⟨⟨⟨⟨⟨⟨⟨⟨⟨⟨h1, h2⟩, h3⟩, h4⟩, h5⟩, h6⟩, h7⟩, h8⟩, h9⟩, h10⟩, h11⟩,
    h12⟩, h13⟩, h14⟩, h15⟩, h16⟩, h17⟩, h18⟩ := hok
  have hdec := witness_decodeHeader P c
  have hres := witness_resourceOk P c
  have hwf : DescriptorWellFormed (descriptorOf (witnessHeader c))
      (windowOf P (witnessRawBody P c)) :=
    DescriptorWellFormed.ofLenEq (M := ⟨0, 320, 0⟩) rfl h5
  have hvalid : refValid (witnessRawBody P c) = true := by
    simp [refValid, hdec, h1, h2, h3, h4, hwf, hres]
  obtain ⟨inv, hcl⟩ :=
    classifier_exact_domain (witnessRawBody P c) (witnessHeader c) hdec h1 h2 h3 h4 hwf hres
  have hentry := witness_entryStore P c
  have hstatus : refOutcome (witnessRawBody P c) =
      { status := 0, detail := StatusDetail.zero, writesC := true } := by
    unfold refOutcome
    rw [if_pos hvalid]
    simp only [hdec, hentry, h15, h6]
    cases hm : c.mode with
    | modular => rfl
    | strictFloat => rfl
    | exactDyadicRoundOnce => rfl
    | checked => rw [h16 hm]
  refine ⟨witnessRaw P c, inv, witnessHeader c, hcl, hdec, h7, h8, h9, h10, h11, h12,
    h13, h14, ?_, ?_, oneBits_ne_zero c.kind, ?_⟩
  · show cBitsAt (descriptorOf (witnessHeader c)) (entryStore (witnessRawBody P c))
      ⟨0, 0, 0⟩ = 0
    rw [hentry]; exact h17
  · show referenceElem (descriptorOf (witnessHeader c)) (entryStore (witnessRawBody P c))
      ⟨0, 0, 0⟩ = oneBits c.kind
    rw [hentry]; exact h18
  · show referenceStatus (witnessRawBody P c) = 0
    simp [referenceStatus, hstatus]

/-! ## Anti-vacuity: the full C range and the status range are observable -/

/-- **SPEC §8.3.**  For a validated raw invocation, every byte of the declared
`C` range and every byte of the status-detail range of an accepted observation
is pinned to the reference value: the scratch mask never hides either. -/
theorem observation_covers_status_and_full_c {P : Wasm.Profile} (problem : Problem P)
    (raw : Gemm.RawInvocation P) {o : Wasm.ExecutionObservation}
    (hv : refValid raw.body = true) (h : Reference.Accepts problem raw o) :
    ∃ e f : Store,
      semanticFor problem raw o = .returned (referenceStatus raw.body) e f ∧
      ∀ i, i < raw.body.len.toNat →
        ((regionsOfRaw raw.body).c.Mem i ∨ (regionsOfRaw raw.body).statusDetail.Mem i) →
        f i = referenceFinalStore raw.body i := by
  obtain ⟨e, f, heq, -, hf⟩ := h.1
  refine ⟨e, f, heq, ?_⟩
  intro i hi hmem
  obtain ⟨hd, hdec⟩ := decodeHeader_of_refValid hv
  have hwf := wellFormed_of_refValid hdec hv
  obtain ⟨hsc, hss⟩ := scratch_disjoint_of_wellFormed hwf
  rw [hf i]
  simp only [referenceObservableStore, if_pos hi, if_pos hv]
  apply maskScratch_of_not_mem
  intro hscm
  have hreg : regionsOfRaw raw.body = (descriptorOf hd).regions := by
    simp only [regionsOfRaw, hdec]
  rw [hreg] at hscm hmem
  rcases hmem with hc | hs
  · exact hsc ⟨i, hscm, hc⟩
  · exact hss ⟨i, hscm, hs⟩

/-- The status-detail range of a validated raw invocation is exactly 32 bytes,
so the covering statement above is not vacuous on the status side. -/
theorem status_range_is_record {P : Wasm.Profile} {raw : Gemm.RawInvocation P}
    (hv : refValid raw.body = true) :
    (regionsOfRaw raw.body).statusDetail.length = statusDetailBytes := by
  obtain ⟨hd, hdec⟩ := decodeHeader_of_refValid hv
  have hwf := wellFormed_of_refValid hdec hv
  simp only [regionsOfRaw, hdec, DescriptorBody.regions]
  exact hwf.statusIsRecord

/-- The declared `C` range of a validated raw invocation with a nonempty logical
`C` and a stored (nonzero-width) kind is nonempty, so the covering statement is
not vacuous on the `C` side either. -/
theorem c_range_nonempty {P : Wasm.Profile} {raw : Gemm.RawInvocation P} {hd : RawHeader}
    (hdec : decodeHeader raw.body.bytes = some hd) (hv : refValid raw.body = true)
    (hne : (descriptorOf hd).shapeC.isEmpty = false)
    (hw : 0 < (descriptorOf hd).cKind.byteWidth) :
    (regionsOfRaw raw.body).c.isEmpty = false := by
  have hwf := wellFormed_of_refValid hdec hv
  have horigin : (Index.mk 0 0 0).Mem (descriptorOf hd).shapeC := Index.origin_mem hne
  obtain ⟨h1, h2⟩ := hwf.layoutC.elementsInInterval _ horigin
  simp only [regionsOfRaw, hdec, DescriptorBody.regions, ByteRange.ofView,
    ByteRange.isEmpty, beq_eq_false_iff_ne, ne_eq]
  omega

/-- The witness family exercises both anti-vacuity statements at once: its `C`
range is a real, nonempty, fully observable range. -/
theorem witness_c_range_nonempty (P : Wasm.Profile) (c : MandatoryCase) :
    (regionsOfRaw (witnessRawBody P c)).c.isEmpty = false := by
  simp only [regionsOfRaw, witness_decodeHeader, DescriptorBody.regions,
    ByteRange.ofView, ByteRange.isEmpty]
  rfl

end WasmGemmGnaf.Gemm
