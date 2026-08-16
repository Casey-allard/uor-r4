/-
  Wasm/Numeric.lean --- set-valued numeric relations and the deterministic
  core operations.

  Normative source: SPEC.md section 7.1 ("`Numeric` and `Vector` own set-valued
  numeric relations") and section 7.2 ("permitted numeric nondeterminism:
  exactly the relation in the pinned semantics, with correctness and cost
  quantified over every permitted result and trace").

  A Core numeric instruction denotes a *relation* between operands and results,
  not a function.  For the integer operations of the released subset the
  relation is single valued, and that is proved here rather than assumed: every
  integer relation is exhibited as the graph of a total or partial function and
  `Deterministic` is discharged.  The genuinely set-valued cases are the
  floating-point NaN results, whose permitted set is every NaN bit pattern of
  the right width.  For those the relation is proved nonempty and proved to
  contain only NaN payloads, and it is proved *not* single valued, so a later
  file cannot silently replace it by an evaluator choice.

  Every declaration in this file is proved.  Nothing is assumed.
-/
import WasmGemmGnaf.Wasm.Types

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm
namespace Num

open WasmGemmGnaf.Foundation

/-! ## Set-valued results

A numeric relation on results of type `α` is a predicate: the set of results
the pinned semantics permits.  `Deterministic` says the set is a singleton. -/

/-- The set of results a numeric relation permits. -/
def Results (α : Type) : Type := α → Prop

/-- The relation permitting exactly one result. -/
def det {α : Type} (x : α) : Results α := fun y => y = x

/-- The empty relation: no result is permitted (a trapping operation). -/
def none' {α : Type} : Results α := fun _ => False

/-- A relation is deterministic when exactly one result is permitted. -/
def Deterministic {α : Type} (r : Results α) : Prop :=
  ∃ x : α, r x ∧ ∀ y : α, r y → y = x

/-- A relation is inhabited when at least one result is permitted. -/
def Permitted {α : Type} (r : Results α) : Prop := ∃ x : α, r x

theorem det_deterministic {α : Type} (x : α) : Deterministic (det x) :=
  ⟨x, rfl, fun _ h => h⟩

theorem det_permitted {α : Type} (x : α) : Permitted (det x) := ⟨x, rfl⟩

theorem mem_det_iff {α : Type} (x y : α) : det x y ↔ y = x := Iff.rfl

theorem deterministic_permitted {α : Type} {r : Results α}
    (h : Deterministic r) : Permitted r :=
  ⟨h.choose, h.choose_spec.1⟩

/-- A deterministic relation identifies all its permitted results. -/
theorem eq_of_deterministic {α : Type} {r : Results α} (h : Deterministic r)
    {x y : α} (hx : r x) (hy : r y) : x = y := by
  obtain ⟨z, _, huniq⟩ := h
  rw [huniq x hx, huniq y hy]

theorem not_permitted_none' {α : Type} : ¬ Permitted (@none' α) := by
  rintro ⟨_, h⟩
  exact h

/-- The relation obtained from a partial function: `none` is a trap. -/
def ofOption {α : Type} (o : Option α) : Results α := fun y => o = some y

theorem ofOption_some {α : Type} (x : α) : ofOption (some x) = det x := by
  funext y
  simp [ofOption, det, eq_comm]

theorem ofOption_none_not_permitted {α : Type} : ¬ Permitted (@ofOption α none) := by
  rintro ⟨_, h⟩
  simp [ofOption] at h

theorem ofOption_deterministic {α : Type} (x : α) :
    Deterministic (ofOption (some x)) := by
  rw [ofOption_some]; exact det_deterministic x

/-! ## Deterministic core integer operations

These are the exact Core 3.0 `i32`/`i64` operations of the released subset.
Lean's `UInt32`/`UInt64` are the wrap-around words of the specification, so the
total operations are the machine operations and the proofs are definitional. -/

/-- `i32.add`. -/
def i32Add (a b : UInt32) : UInt32 := a + b
/-- `i32.sub`. -/
def i32Sub (a b : UInt32) : UInt32 := a - b
/-- `i32.mul`. -/
def i32Mul (a b : UInt32) : UInt32 := a * b
/-- `i32.and`. -/
def i32And (a b : UInt32) : UInt32 := a &&& b
/-- `i32.or`. -/
def i32Or (a b : UInt32) : UInt32 := a ||| b
/-- `i32.xor`. -/
def i32Xor (a b : UInt32) : UInt32 := a ^^^ b
/-- `i32.shl`, with the shift count taken modulo 32. -/
def i32Shl (a b : UInt32) : UInt32 := a <<< (b % 32)
/-- `i32.shr_u`, with the shift count taken modulo 32. -/
def i32ShrU (a b : UInt32) : UInt32 := a >>> (b % 32)
/-- `i32.eqz`. -/
def i32Eqz (a : UInt32) : UInt32 := if a = 0 then 1 else 0
/-- `i32.eq`. -/
def i32Eq (a b : UInt32) : UInt32 := if a = b then 1 else 0
/-- `i32.ne`. -/
def i32Ne (a b : UInt32) : UInt32 := if a = b then 0 else 1
/-- `i32.lt_u`. -/
def i32LtU (a b : UInt32) : UInt32 := if a < b then 1 else 0
/-- `i32.gt_u`. -/
def i32GtU (a b : UInt32) : UInt32 := if b < a then 1 else 0
/-- `i32.le_u`. -/
def i32LeU (a b : UInt32) : UInt32 := if a ≤ b then 1 else 0
/-- `i32.ge_u`. -/
def i32GeU (a b : UInt32) : UInt32 := if b ≤ a then 1 else 0

/-- The signed interpretation of a 32-bit word. -/
def toSigned (a : UInt32) : Int :=
  if a.toNat < 2147483648 then (a.toNat : Int) else (a.toNat : Int) - 4294967296

/-- `i32.lt_s`. -/
def i32LtS (a b : UInt32) : UInt32 := if toSigned a < toSigned b then 1 else 0
/-- `i32.gt_s`. -/
def i32GtS (a b : UInt32) : UInt32 := if toSigned b < toSigned a then 1 else 0
/-- `i32.le_s`. -/
def i32LeS (a b : UInt32) : UInt32 := if toSigned a ≤ toSigned b then 1 else 0
/-- `i32.ge_s`. -/
def i32GeS (a b : UInt32) : UInt32 := if toSigned b ≤ toSigned a then 1 else 0

theorem i32LtS_irrefl (a : UInt32) : i32LtS a a = 0 := by
  simp [i32LtS]

theorem i32LeS_refl (a : UInt32) : i32LeS a a = 1 := by
  simp [i32LeS]

/-- The signed reading is injective: it loses no information. -/
theorem toSigned_injective : Function.Injective toSigned := by
  intro a b h
  unfold toSigned at h
  have ha : a.toNat < 2 ^ 32 := UInt32.toNat_lt a
  have hb : b.toNat < 2 ^ 32 := UInt32.toNat_lt b
  have hab : a.toNat = b.toNat := by
    simp only [Nat.pow_succ] at ha hb
    split at h <;> split at h <;> omega
  exact UInt32.toNat_inj.mp hab

/-- `i32.div_u`: a partial operation, undefined (trapping) at divisor zero. -/
def i32DivU (a b : UInt32) : Option UInt32 := if b = 0 then none else some (a / b)
/-- `i32.rem_u`: a partial operation, undefined (trapping) at divisor zero. -/
def i32RemU (a b : UInt32) : Option UInt32 := if b = 0 then none else some (a % b)

/-! ### Laws of the deterministic operations -/

theorem i32Add_comm (a b : UInt32) : i32Add a b = i32Add b a := by
  simp [i32Add, UInt32.add_comm]

theorem i32Add_assoc (a b c : UInt32) :
    i32Add (i32Add a b) c = i32Add a (i32Add b c) := by
  simp [i32Add, UInt32.add_assoc]

theorem i32Add_zero (a : UInt32) : i32Add a 0 = a := by
  simp [i32Add]

theorem i32Sub_self (a : UInt32) : i32Sub a a = 0 := by
  simp [i32Sub]

theorem i32Add_sub_cancel (a b : UInt32) : i32Sub (i32Add a b) b = a := by
  simp [i32Sub, i32Add]

theorem i32Mul_comm (a b : UInt32) : i32Mul a b = i32Mul b a := by
  simp [i32Mul, UInt32.mul_comm]

theorem i32Mul_one (a : UInt32) : i32Mul a 1 = a := by
  simp [i32Mul]

theorem i32And_self (a : UInt32) : i32And a a = a := by
  simp [i32And]

theorem i32Xor_self (a : UInt32) : i32Xor a a = 0 := by
  simp [i32Xor]

theorem i32Eqz_zero : i32Eqz 0 = 1 := rfl

theorem i32Eqz_eq_one_iff (a : UInt32) : i32Eqz a = 1 ↔ a = 0 := by
  unfold i32Eqz
  split <;> simp_all

theorem i32Eq_self (a : UInt32) : i32Eq a a = 1 := by simp [i32Eq]

theorem i32Eq_eq_one_iff (a b : UInt32) : i32Eq a b = 1 ↔ a = b := by
  unfold i32Eq
  split <;> simp_all

theorem i32Ne_eq_zero_iff (a b : UInt32) : i32Ne a b = 0 ↔ a = b := by
  unfold i32Ne
  split <;> simp_all

/-- `i32.div_u` traps exactly at divisor zero. -/
theorem i32DivU_eq_none_iff (a b : UInt32) : i32DivU a b = none ↔ b = 0 := by
  unfold i32DivU
  split <;> simp_all

/-- `i32.rem_u` traps exactly at divisor zero. -/
theorem i32RemU_eq_none_iff (a b : UInt32) : i32RemU a b = none ↔ b = 0 := by
  unfold i32RemU
  split <;> simp_all

theorem i32DivU_of_ne_zero {a b : UInt32} (h : b ≠ 0) :
    i32DivU a b = some (a / b) := by
  simp [i32DivU, h]

theorem i32RemU_of_ne_zero {a b : UInt32} (h : b ≠ 0) :
    i32RemU a b = some (a % b) := by
  simp [i32RemU, h]

/-! ### The relational reading of the integer operations

Every integer operation of the subset is exhibited as a `Results` relation, and
each is proved deterministic where defined.  This is the interface `Step` uses,
so a nondeterministic rule and a deterministic rule are treated uniformly. -/

/-- The permitted results of a total binary integer operation. -/
def binRel (f : UInt32 → UInt32 → UInt32) (a b : UInt32) : Results UInt32 :=
  det (f a b)

/-- The permitted results of a partial binary integer operation. -/
def partialBinRel (f : UInt32 → UInt32 → Option UInt32) (a b : UInt32) :
    Results UInt32 :=
  ofOption (f a b)

theorem binRel_deterministic (f : UInt32 → UInt32 → UInt32) (a b : UInt32) :
    Deterministic (binRel f a b) :=
  det_deterministic _

theorem partialBinRel_deterministic_of_some
    {f : UInt32 → UInt32 → Option UInt32} {a b : UInt32} {x : UInt32}
    (h : f a b = some x) : Deterministic (partialBinRel f a b) := by
  unfold partialBinRel
  rw [h]
  exact ofOption_deterministic x

theorem partialBinRel_not_permitted_of_none
    {f : UInt32 → UInt32 → Option UInt32} {a b : UInt32}
    (h : f a b = none) : ¬ Permitted (partialBinRel f a b) := by
  unfold partialBinRel
  rw [h]
  exact ofOption_none_not_permitted

/-- The `i32.div_u` relation is empty exactly at divisor zero, i.e. exactly
where the instruction traps. -/
theorem divU_rel_not_permitted_iff (a b : UInt32) :
    ¬ Permitted (partialBinRel i32DivU a b) ↔ b = 0 := by
  constructor
  · intro h
    rcases Decidable.em (b = 0) with hb | hb
    · exact hb
    · exact absurd
        (⟨a / b, by simp [partialBinRel, ofOption, i32DivU_of_ne_zero hb]⟩ :
          Permitted (partialBinRel i32DivU a b)) h
  · intro hb
    exact partialBinRel_not_permitted_of_none (by simp [i32DivU, hb])

/-! ## Floating-point bit patterns and the genuinely set-valued relations

An `f32` value is its 32-bit pattern.  The bit-level operations `abs`, `neg`
and `copysign` are exactly defined by the specification and are deterministic;
they are given here as functions.  A NaN *result* is the set-valued case: the
pinned semantics permits any NaN payload. -/

/-- The exponent-and-mantissa mask isolating everything but the sign bit. -/
def f32SignMask : UInt32 := 0x80000000

/-- `f32.abs`: clear the sign bit. -/
def f32Abs (x : UInt32) : UInt32 := x &&& 0x7fffffff

/-- `f32.neg`: flip the sign bit. -/
def f32Neg (x : UInt32) : UInt32 := x ^^^ f32SignMask

/-- `f32.copysign`: the magnitude of `x` with the sign of `y`. -/
def f32Copysign (x y : UInt32) : UInt32 := f32Abs x ||| (y &&& f32SignMask)

/-- A 32-bit pattern is a NaN when the exponent field is all ones and the
significand is nonzero. -/
def isNaN32 (x : UInt32) : Bool := 0x7f800000 < (x &&& 0x7fffffff)

/-- The canonical positive quiet NaN. -/
def canonicalNaN32 : UInt32 := 0x7fc00000

/-- The canonical negative quiet NaN. -/
def negCanonicalNaN32 : UInt32 := 0xffc00000

theorem canonicalNaN32_isNaN : isNaN32 canonicalNaN32 = true := by decide

theorem negCanonicalNaN32_isNaN : isNaN32 negCanonicalNaN32 = true := by decide

theorem zero_not_isNaN : isNaN32 0 = false := by decide

theorem f32Abs_idem (x : UInt32) : f32Abs (f32Abs x) = f32Abs x := by
  simp [f32Abs, UInt32.and_assoc]

theorem f32Neg_involutive (x : UInt32) : f32Neg (f32Neg x) = x := by
  simp [f32Neg, UInt32.xor_assoc]

theorem f32Abs_zero : f32Abs 0 = 0 := by decide

/-- The set-valued NaN relation: every NaN bit pattern of the right width is a
permitted result of an operation whose mathematical value is NaN.  This is a
proper set, not a choice. -/
def nanResults32 : Results UInt32 := fun y => isNaN32 y = true

theorem nanResults32_permitted : Permitted nanResults32 :=
  ⟨canonicalNaN32, canonicalNaN32_isNaN⟩

theorem nanResults32_isNaN {y : UInt32} (h : nanResults32 y) : isNaN32 y = true := h

/-- The NaN relation is genuinely set valued: it is *not* deterministic, so no
single evaluator result can stand in for it. -/
theorem nanResults32_not_deterministic : ¬ Deterministic nanResults32 := by
  intro h
  have := eq_of_deterministic h canonicalNaN32_isNaN negCanonicalNaN32_isNaN
  exact absurd this (by decide)

/-- The profile-permitted finite enumeration of NaN results used by the
executable successor enumerator: the two canonical quiet NaNs. -/
def canonicalNaNs32 : List UInt32 := [canonicalNaN32, negCanonicalNaN32]

theorem canonicalNaNs32_nodup : canonicalNaNs32.Nodup := by decide

theorem canonicalNaNs32_isNaN {y : UInt32} (h : y ∈ canonicalNaNs32) :
    isNaN32 y = true := by
  simp only [canonicalNaNs32, List.mem_cons, List.not_mem_nil, or_false] at h
  rcases h with rfl | rfl
  · decide
  · decide

theorem canonicalNaNs32_subset_nanResults32 {y : UInt32}
    (h : y ∈ canonicalNaNs32) : nanResults32 y :=
  canonicalNaNs32_isNaN h

/-! ## The `memory.grow` nondeterminism

`memory.grow` is the released profile's permitted nondeterminism in the
control-visible part of the semantics: the embedder may always refuse to grow.
The relation therefore has two permitted outcomes whenever growth is possible,
and one when it is not. -/

/-- The outcome of a `memory.grow` attempt. -/
inductive GrowOutcome
  /-- Growth succeeded; the previous page count is pushed. -/
  | grown (previousPages : Nat)
  /-- Growth was refused; `-1` is pushed and the memory is unchanged. -/
  | refused
  deriving DecidableEq, Repr, Inhabited

/-- The permitted outcomes of growing a memory of `current` pages by `delta`,
under a maximum of `limit` pages.  Refusal is always permitted; success is
permitted exactly when the resulting size is within the limit. -/
def growResults (current delta limit : Nat) : Results GrowOutcome
  | .refused => True
  | .grown previous => previous = current ∧ current + delta ≤ limit

theorem growResults_refused (current delta limit : Nat) :
    growResults current delta limit .refused := trivial

theorem growResults_permitted (current delta limit : Nat) :
    Permitted (growResults current delta limit) :=
  ⟨.refused, trivial⟩

theorem growResults_grown_iff {current delta limit previous : Nat} :
    growResults current delta limit (.grown previous) ↔
      previous = current ∧ current + delta ≤ limit := Iff.rfl

/-- When growth is possible the relation is genuinely set valued: refusal and
success are both permitted, so a single evaluator path is not a substitute. -/
theorem growResults_not_deterministic {current delta limit : Nat}
    (h : current + delta ≤ limit) :
    ¬ Deterministic (growResults current delta limit) := by
  intro hdet
  have := eq_of_deterministic hdet (x := GrowOutcome.refused)
    (y := GrowOutcome.grown current) trivial ⟨rfl, h⟩
  exact absurd this (by simp)

/-- When growth is impossible the relation is deterministic: only refusal. -/
theorem growResults_deterministic_of_gt {current delta limit : Nat}
    (h : limit < current + delta) :
    Deterministic (growResults current delta limit) := by
  refine ⟨.refused, trivial, ?_⟩
  intro y hy
  cases y with
  | grown previous =>
    simp only [growResults] at hy
    omega
  | refused => rfl

/-- The executable enumeration of permitted `memory.grow` outcomes. -/
def growOutcomes (current delta limit : Nat) : List GrowOutcome :=
  if current + delta ≤ limit then [.grown current, .refused] else [.refused]

theorem growOutcomes_nodup (current delta limit : Nat) :
    (growOutcomes current delta limit).Nodup := by
  unfold growOutcomes
  split <;> simp

/-- The executable enumeration is exactly the permitted set. -/
theorem mem_growOutcomes_iff (current delta limit : Nat) (o : GrowOutcome) :
    o ∈ growOutcomes current delta limit ↔ growResults current delta limit o := by
  unfold growOutcomes
  cases o with
  | refused => split <;> simp [growResults]
  | grown previous =>
    split <;> rename_i h <;> simp [growResults, h] <;> omega

end Num
end WasmGemmGnaf.Wasm
