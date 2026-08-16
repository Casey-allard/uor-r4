import WasmGemmGnaf.Foundation.Finite
import WasmGemmGnaf.Foundation.Identity
set_option autoImplicit false

/-!
# GNAF: the typed semantic universe (SPEC §8.2, §11.1; authority §3.1–§3.3)

Authority §3.1 requires a kind descriptor to bind, for every admitted type
instance, its carrier, validity predicate, equivalence and observation family,
and §3.2 requires typed content to be a *pair* of a type instance and a value
that is valid at it.  This file fixes the closed released set of scalar kinds
(SPEC §8.1), the closed set of arithmetic modes and the closed compatibility
table (SPEC §8.2), the closed status set (SPEC §8.3), and the typed content
built over them.

Everything here is finite and decidable: the compatibility table is a total
Boolean function on a finite product, so every statement about it is discharged
by `decide` against the *table*, never assumed.
-/

namespace WasmGemmGnaf.GNAF

open WasmGemmGnaf.Foundation

/-! ## Scalar kinds -/

/-- The closed set of released stored scalar encodings together with the
accumulator-only exact dyadic mode (SPEC §8.1, §8.3 tag table). -/
inductive ScalarKind
  | i8 | u8 | i16 | u16 | i32 | u32 | i64 | u64
  | binary16 | bfloat16 | binary32 | binary64 | exactDyadic
  deriving DecidableEq, Repr, Inhabited

namespace ScalarKind

/-- The ABI kind tag (SPEC §8.3). -/
def tag : ScalarKind → Nat
  | i8 => 0 | u8 => 1 | i16 => 2 | u16 => 3
  | i32 => 4 | u32 => 5 | i64 => 6 | u64 => 7
  | binary16 => 8 | bfloat16 => 9 | binary32 => 10 | binary64 => 11
  | exactDyadic => 12

/-- Stored width in bytes; the accumulator-only exact dyadic mode has no stored
width. -/
def byteWidth : ScalarKind → Nat
  | i8 | u8 => 1
  | i16 | u16 | binary16 | bfloat16 => 2
  | i32 | u32 | binary32 => 4
  | i64 | u64 | binary64 => 8
  | exactDyadic => 0

/-- The complete finite enumeration. -/
def all : List ScalarKind :=
  [i8, u8, i16, u16, i32, u32, i64, u64,
   binary16, bfloat16, binary32, binary64, exactDyadic]

theorem mem_all (k : ScalarKind) : k ∈ all := by cases k <;> simp [all]

theorem all_nodup : all.Nodup := by decide

instance : Foundation.Fintype ScalarKind where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

/-- Bounded quantification over the finite kind set is decidable. -/
instance decidableForall (p : ScalarKind → Prop) [DecidablePred p] :
    Decidable (∀ k, p k) :=
  decidable_of_iff (∀ k ∈ all, p k) ⟨fun h k => h k (mem_all k), fun h k _ => h k⟩

theorem tag_injective : Function.Injective tag := by
  intro a b h; cases a <;> cases b <;> simp_all [tag]

/-- A kind that can appear as a stored A/B/C encoding. -/
def isStorable : ScalarKind → Bool
  | exactDyadic => false
  | _ => true

/-- The IEEE-format kinds. -/
def isFloat : ScalarKind → Bool
  | binary16 | bfloat16 | binary32 | binary64 => true
  | _ => false

/-- The signed two's-complement integer kinds. -/
def isSignedInt : ScalarKind → Bool
  | i8 | i16 | i32 | i64 => true
  | _ => false

/-- The unsigned integer kinds. -/
def isUnsignedInt : ScalarKind → Bool
  | u8 | u16 | u32 | u64 => true
  | _ => false

/-- The modulus of the stored bit pattern space, `2 ^ (8 · byteWidth)`. -/
def modulus (k : ScalarKind) : Nat := 2 ^ (8 * k.byteWidth)

theorem modulus_pos (k : ScalarKind) : 0 < k.modulus :=
  Nat.two_pow_pos _

theorem storable_byteWidth_pos : ∀ k : ScalarKind, k.isStorable = true → 0 < k.byteWidth := by
  decide

theorem isFloat_storable : ∀ k : ScalarKind, k.isFloat = true → k.isStorable = true := by
  decide

theorem int_not_float : ∀ k : ScalarKind,
    k.isSignedInt = true ∨ k.isUnsignedInt = true → k.isFloat = false := by
  decide

/-- Canonical one-byte encoding of a kind tag. -/
def bytes (k : ScalarKind) : List UInt8 := [UInt8.ofNat k.tag]

theorem bytes_injective : Function.Injective bytes := by
  intro a b h
  simp only [bytes, List.cons.injEq, and_true] at h
  rw [Bytes.uint8_ofNat_eq_iff] at h
  apply tag_injective
  cases a <;> cases b <;> simp_all [tag]

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Bytes.prefixFree_of_constLength bytes 1 (fun k => by cases k <;> rfl) bytes_injective

end ScalarKind

/-! ## Arithmetic modes and the closed compatibility table -/

/-- The closed set of released arithmetic modes (SPEC §8.2, §8.3 tag table). -/
inductive ArithmeticMode
  | modular | checked | strictFloat | exactDyadicRoundOnce
  deriving DecidableEq, Repr, Inhabited

namespace ArithmeticMode

def tag : ArithmeticMode → Nat
  | modular => 0 | checked => 1 | strictFloat => 2 | exactDyadicRoundOnce => 3

def all : List ArithmeticMode := [modular, checked, strictFloat, exactDyadicRoundOnce]

theorem mem_all (m : ArithmeticMode) : m ∈ all := by cases m <;> simp [all]

theorem all_nodup : all.Nodup := by decide

instance : Foundation.Fintype ArithmeticMode where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

instance decidableForall (p : ArithmeticMode → Prop) [DecidablePred p] :
    Decidable (∀ m, p m) :=
  decidable_of_iff (∀ m ∈ all, p m) ⟨fun h m => h m (mem_all m), fun h m _ => h m⟩

theorem tag_injective : Function.Injective tag := by
  intro a b h; cases a <;> cases b <;> simp_all [tag]

def bytes (m : ArithmeticMode) : List UInt8 := [UInt8.ofNat m.tag]

theorem bytes_injective : Function.Injective bytes := by
  intro a b h
  simp only [bytes, List.cons.injEq, and_true] at h
  rw [Bytes.uint8_ofNat_eq_iff] at h
  apply tag_injective
  cases a <;> cases b <;> simp_all [tag]

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Bytes.prefixFree_of_constLength bytes 1 (fun m => by cases m <;> rfl) bytes_injective

end ArithmeticMode

/-- The compatibility row index of SPEC §8.2, in the table's displayed order.
`none` means the triple appears in no row and therefore classifies as invalid;
there is no implicit conversion. -/
def compatibleRow :
    ArithmeticMode → ScalarKind → ScalarKind → Option Nat
  | .modular, .i8, .u32 | .modular, .u8, .u32 | .modular, .i16, .u32
  | .modular, .u16, .u32 | .modular, .i32, .u32 | .modular, .u32, .u32 => some 0
  | .modular, .i64, .u64 | .modular, .u64, .u64 => some 1
  | .checked, .i8, .i64 | .checked, .i16, .i64 | .checked, .i32, .i64
  | .checked, .i64, .i64 => some 2
  | .checked, .u8, .u64 | .checked, .u16, .u64 | .checked, .u32, .u64
  | .checked, .u64, .u64 => some 3
  | .strictFloat, .binary16, .binary32 | .strictFloat, .binary16, .binary64
  | .strictFloat, .bfloat16, .binary32 | .strictFloat, .bfloat16, .binary64 => some 4
  | .strictFloat, .binary32, .binary32 | .strictFloat, .binary32, .binary64 => some 5
  | .strictFloat, .binary64, .binary64 => some 6
  | .exactDyadicRoundOnce, .binary16, .exactDyadic
  | .exactDyadicRoundOnce, .bfloat16, .exactDyadic
  | .exactDyadicRoundOnce, .binary32, .exactDyadic
  | .exactDyadicRoundOnce, .binary64, .exactDyadic => some 7
  | _, _, _ => none

/-- The closed compatibility relation of SPEC §8.2: a mode, one stored kind used
for A, B and C, and the required accumulator kind. -/
def compatible (m : ArithmeticMode) (stored acc : ScalarKind) : Bool :=
  (compatibleRow m stored acc).isSome

theorem compatible_iff_row (m : ArithmeticMode) (stored acc : ScalarKind) :
    compatible m stored acc = true ↔ ∃ r, compatibleRow m stored acc = some r := by
  simp [compatible, Option.isSome_iff_exists]

theorem compatible_row_lt_eight : ∀ (m : ArithmeticMode) (stored acc : ScalarKind)
    (r : Nat), compatibleRow m stored acc = some r → r < 8 := by decide

/-- Compatibility forces a stored kind: the exact dyadic mode is accumulator
only. -/
theorem compatible_stored_storable : ∀ (m : ArithmeticMode) (stored acc : ScalarKind),
    compatible m stored acc = true → stored.isStorable = true := by decide

/-- The exact-dyadic round-once mode admits exactly the exact dyadic
accumulator. -/
theorem compatible_exactDyadic_acc : ∀ (stored acc : ScalarKind),
    compatible .exactDyadicRoundOnce stored acc = true → acc = .exactDyadic := by decide

/-- Both floating modes require a floating stored kind. -/
theorem compatible_float_stored : ∀ (m : ArithmeticMode) (stored acc : ScalarKind),
    compatible m stored acc = true →
    (m = .strictFloat ∨ m = .exactDyadicRoundOnce) → stored.isFloat = true := by decide

/-- Both integer modes require an integer stored kind. -/
theorem compatible_int_stored : ∀ (m : ArithmeticMode) (stored acc : ScalarKind),
    compatible m stored acc = true →
    (m = .modular ∨ m = .checked) →
    (stored.isSignedInt = true ∨ stored.isUnsignedInt = true) := by decide

/-- An integer accumulator is never narrower than the stored kind. -/
theorem compatible_int_acc_wide : ∀ (m : ArithmeticMode) (stored acc : ScalarKind),
    compatible m stored acc = true → stored.isFloat = false →
    stored.byteWidth ≤ acc.byteWidth := by decide

/-- The modular mode's accumulator is one of the two released unsigned widths. -/
theorem compatible_modular_acc : ∀ (stored acc : ScalarKind),
    compatible .modular stored acc = true → acc = .u32 ∨ acc = .u64 := by decide

/-- The compatibility table is nonvacuous: every mode has a witness. -/
theorem compatible_modular_witness : compatible .modular .i32 .u32 = true := by decide
theorem compatible_checked_witness : compatible .checked .i32 .i64 = true := by decide
theorem compatible_strictFloat_witness : compatible .strictFloat .binary32 .binary32 = true := by
  decide
theorem compatible_exactDyadic_witness :
    compatible .exactDyadicRoundOnce .binary64 .exactDyadic = true := by decide

/-- No mixed-kind row exists: a mode never relates a float stored kind to an
integer accumulator width of a different family. -/
theorem compatible_no_mixed_family : ∀ (m : ArithmeticMode) (stored acc : ScalarKind),
    compatible m stored acc = true → stored.isFloat = true →
    (acc.isFloat = true ∨ acc = .exactDyadic) := by decide

/-! ## Arithmetic contracts

SPEC §8.2: no ring, semiring, associativity or reassociation law is implicit.
An `ArithmeticContract` is the first-order record of the mode and the two kinds,
carrying the *checked* compatibility bit rather than a class instance. -/

/-- The first-order arithmetic contract of a reduction (SPEC §8.2). -/
structure ArithmeticContract where
  mode : ArithmeticMode
  stored : ScalarKind
  accumulator : ScalarKind
  deriving DecidableEq, Repr, Inhabited

namespace ArithmeticContract

/-- The contract's compatibility check; decidable by the closed table. -/
def compatibleB (c : ArithmeticContract) : Bool :=
  compatible c.mode c.stored c.accumulator

/-- The accumulator modulus used by modular and checked evaluation. -/
def accModulus (c : ArithmeticContract) : Nat := c.accumulator.modulus

theorem accModulus_pos (c : ArithmeticContract) : 0 < c.accModulus :=
  ScalarKind.modulus_pos _

/-- The exact multiply-accumulate step of the contract, on stored bit patterns.

SPEC §8.2 fixes the modular mode to work modulo the accumulator width; the
checked mode computes over mathematical naturals and reports its own overflow
separately (see `overflows`); the two floating modes are *not* given a bit-level
equation here — the plan language treats a floating reduction as an opaque
declared step and this file provides only its accumulator arithmetic. -/
def step (c : ArithmeticContract) (acc a b : Nat) : Nat :=
  match c.mode with
  | .modular => (acc + a * b) % c.accModulus
  | .checked => acc + a * b
  | .strictFloat => (acc + a * b) % c.accModulus
  | .exactDyadicRoundOnce => acc + a * b

/-- The checked mode's overflow predicate (SPEC §8.2: status 4 with C
unchanged). -/
def overflows (c : ArithmeticContract) (acc a b : Nat) : Bool :=
  match c.mode with
  | .checked => decide (c.accModulus ≤ acc + a * b)
  | _ => false

theorem step_modular_lt (c : ArithmeticContract) (h : c.mode = .modular)
    (acc a b : Nat) : c.step acc a b < c.accModulus := by
  simp only [step, h]
  exact Nat.mod_lt _ c.accModulus_pos

theorem step_checked_eq (c : ArithmeticContract) (h : c.mode = .checked)
    (acc a b : Nat) : c.step acc a b = acc + a * b := by
  simp [step, h]

theorem step_checked_no_overflow (c : ArithmeticContract) (h : c.mode = .checked)
    (acc a b : Nat) (hov : c.overflows acc a b = false) :
    c.step acc a b < c.accModulus := by
  simp only [overflows, h, decide_eq_false_iff_not, Nat.not_le] at hov
  simpa [step, h] using hov

/-- A zero multiplicand leaves a modular accumulator that is already reduced
unchanged; this is a *proved* consequence of the equation, not an assumed zero
law. -/
theorem step_modular_zero (c : ArithmeticContract) (h : c.mode = .modular)
    (acc : Nat) (hacc : acc < c.accModulus) (b : Nat) :
    c.step acc 0 b = acc := by
  simp only [step, h, Nat.zero_mul, Nat.add_zero]
  exact Nat.mod_eq_of_lt hacc

end ArithmeticContract

/-! ## Status codes -/

/-- The closed released status set (SPEC §8.3): `0=success`, `1=invalid`,
`2=unsupported`, `3=resource-exhausted`, `4=checked-overflow`,
`5=arithmetic-exception`. -/
inductive Status
  | success | invalid | unsupported | resourceExhausted
  | checkedOverflow | arithmeticException
  deriving DecidableEq, Repr, Inhabited

namespace Status

def code : Status → Nat
  | success => 0 | invalid => 1 | unsupported => 2
  | resourceExhausted => 3 | checkedOverflow => 4 | arithmeticException => 5

def ofCode : Nat → Status
  | 0 => success | 1 => invalid | 2 => unsupported
  | 3 => resourceExhausted | 4 => checkedOverflow | 5 => arithmeticException
  | _ => invalid

def all : List Status :=
  [success, invalid, unsupported, resourceExhausted, checkedOverflow,
   arithmeticException]

theorem mem_all (s : Status) : s ∈ all := by cases s <;> simp [all]

theorem all_nodup : all.Nodup := by decide

instance : Foundation.Fintype Status where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

instance decidableForall (p : Status → Prop) [DecidablePred p] :
    Decidable (∀ s, p s) :=
  decidable_of_iff (∀ s ∈ all, p s) ⟨fun h s => h s (mem_all s), fun h s _ => h s⟩

theorem code_injective : Function.Injective code := by
  intro a b h; cases a <;> cases b <;> simp_all [code]

@[simp] theorem ofCode_code : ∀ s : Status, ofCode s.code = s := by decide

theorem code_lt_six : ∀ s : Status, s.code < 6 := by decide

theorem code_ofCode : ∀ {n : Nat}, n < 6 → (ofCode n).code = n
  | 0, _ => rfl
  | 1, _ => rfl
  | 2, _ => rfl
  | 3, _ => rfl
  | 4, _ => rfl
  | 5, _ => rfl
  | _ + 6, h => absurd h (by omega)

end Status

/-! ## Typed content (authority §3.2)

Typed content is a pair of a type instance and a value valid at it.  The
validity predicate is decidable, so no content object can be admitted without
the check actually being run. -/

/-- A stored scalar: a kind together with its bit pattern. -/
structure ScalarValue where
  kind : ScalarKind
  bits : Nat
  deriving DecidableEq, Repr, Inhabited

namespace ScalarValue

/-- Validity at the kind: the bit pattern fits the stored width. -/
def Valid (v : ScalarValue) : Prop := v.bits < v.kind.modulus

instance (v : ScalarValue) : Decidable v.Valid := by
  unfold Valid; infer_instance

/-- Canonical reduction of an arbitrary natural into the kind's pattern space. -/
def wrap (k : ScalarKind) (n : Nat) : ScalarValue := ⟨k, n % k.modulus⟩

theorem wrap_valid (k : ScalarKind) (n : Nat) : (wrap k n).Valid :=
  Nat.mod_lt _ (ScalarKind.modulus_pos k)

@[simp] theorem wrap_kind (k : ScalarKind) (n : Nat) : (wrap k n).kind = k := rfl

theorem wrap_of_valid (v : ScalarValue) (h : v.Valid) : wrap v.kind v.bits = v := by
  cases v
  simp only [wrap, ScalarValue.mk.injEq, true_and]
  exact Nat.mod_eq_of_lt h

theorem wrap_idem (k : ScalarKind) (n : Nat) :
    wrap k (wrap k n).bits = wrap k n :=
  wrap_of_valid _ (wrap_valid k n)

end ScalarValue

/-- Typed content of the GNAF universe (authority §3.2).  Equal host naturals in
different kinds remain different typed content. -/
inductive Object
  | scalar (value : ScalarValue)
  | tensor (kind : ScalarKind) (data : List Nat)
  | statusValue (status : Status)
  | bytes (data : List Nat)
  | unit
  deriving DecidableEq, Repr, Inhabited

namespace Object

/-- The decidable validity predicate of authority §3.2. -/
def ValidB : Object → Bool
  | scalar v => decide (v.bits < v.kind.modulus)
  | tensor k data => data.all (fun x => decide (x < k.modulus))
  | statusValue _ => true
  | bytes data => data.all (fun x => decide (x < 256))
  | unit => true

@[simp] theorem validB_unit : ValidB unit = true := rfl

@[simp] theorem validB_statusValue (s : Status) : ValidB (statusValue s) = true := rfl

theorem validB_scalar_iff (v : ScalarValue) : ValidB (scalar v) = true ↔ v.Valid := by
  simp [ValidB, ScalarValue.Valid]

theorem validB_tensor_iff (k : ScalarKind) (data : List Nat) :
    ValidB (tensor k data) = true ↔ ∀ x ∈ data, x < k.modulus := by
  simp [ValidB]

theorem validB_wrap (k : ScalarKind) (n : Nat) :
    ValidB (scalar (ScalarValue.wrap k n)) = true :=
  (validB_scalar_iff _).mpr (ScalarValue.wrap_valid k n)

/-- The canonical wrapping constructor for tensors produces valid content. -/
def wrapTensor (k : ScalarKind) (data : List Nat) : Object :=
  tensor k (data.map (fun x => x % k.modulus))

theorem wrapTensor_valid (k : ScalarKind) (data : List Nat) :
    ValidB (wrapTensor k data) = true := by
  rw [wrapTensor, validB_tensor_iff]
  intro x hx
  obtain ⟨y, _, rfl⟩ := List.mem_map.mp hx
  exact Nat.mod_lt _ (ScalarKind.modulus_pos k)

end Object

end WasmGemmGnaf.GNAF
