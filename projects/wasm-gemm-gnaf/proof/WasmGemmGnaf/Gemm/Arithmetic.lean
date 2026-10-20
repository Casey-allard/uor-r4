import WasmGemmGnaf.Gemm.FloatBits
set_option autoImplicit false

/-!
# Gemm: arithmetic contracts (SPEC §8.2)

Four released modes, `modular = 0`, `checked = 1`, `strictFloat = 2`,
`exactDyadicRoundOnce = 3`, and the **closed** compatibility table

| row | Mode | A, B, C stored kinds | Required accumulator |
|---|---|---|---|
| 0 | modular | the same one of `i8 u8 i16 u16 i32 u32` | `u32` |
| 1 | modular | the same one of `i64 u64` | `u64` |
| 2 | checked | the same one of `i8 i16 i32 i64` | `i64` |
| 3 | checked | the same one of `u8 u16 u32 u64` | `u64` |
| 4 | strictFloat | the same one of `binary16 bfloat16` | `binary32` or `binary64` |
| 5 | strictFloat | `binary32` | `binary32` or `binary64` |
| 6 | strictFloat | `binary64` | `binary64` |
| 7 | exactDyadicRoundOnce | the same one of `binary16 bfloat16 binary32 binary64` | `exactDyadic` |

"A row not listed is incompatible and classifies as `invalid`; there is no
implicit mixed-kind or mixed-signedness conversion."  The table is a
first-order list value here, not a typeclass, and every derived quantity — the
compatible-row bitset of SPEC §8.3 included — is computed from it.
-/

namespace WasmGemmGnaf.Gemm

/-! ## Modes -/

/-- The four released arithmetic modes (SPEC §8.2). -/
inductive ArithmeticMode
  | modular | checked | strictFloat | exactDyadicRoundOnce
  deriving DecidableEq, Repr, Inhabited

namespace ArithmeticMode

/-- ABI tag byte, exactly as printed in SPEC §8.3. -/
def tag : ArithmeticMode → Nat
  | modular => 0
  | checked => 1
  | strictFloat => 2
  | exactDyadicRoundOnce => 3

def ofTag : Nat → Option ArithmeticMode
  | 0 => some modular
  | 1 => some checked
  | 2 => some strictFloat
  | 3 => some exactDyadicRoundOnce
  | _ + 4 => none

def all : List ArithmeticMode := [modular, checked, strictFloat, exactDyadicRoundOnce]

theorem mem_all (m : ArithmeticMode) : m ∈ all := by cases m <;> decide

theorem all_nodup : all.Nodup := by decide

theorem all_map_tag : all.map tag = List.range 4 := by decide

theorem tag_lt (m : ArithmeticMode) : m.tag < 4 := by cases m <;> decide

theorem tag_injective : Function.Injective tag := by
  intro a b h; cases a <;> cases b <;> simp_all [tag]

@[simp] theorem ofTag_tag (m : ArithmeticMode) : ofTag m.tag = some m := by
  cases m <;> rfl

theorem tag_ofTag {n : Nat} {m : ArithmeticMode} (h : ofTag n = some m) : m.tag = n := by
  match n with
  | 0 | 1 | 2 | 3 => simp only [ofTag, Option.some.injEq] at h; subst h; rfl
  | _ + 4 => exact absurd h (by simp [ofTag])

theorem ofTag_eq_some_iff {n : Nat} {m : ArithmeticMode} :
    ofTag n = some m ↔ m.tag = n :=
  ⟨tag_ofTag, fun h => h ▸ ofTag_tag m⟩

theorem ofTag_eq_none_iff (n : Nat) : ofTag n = none ↔ 4 ≤ n := by
  match n with
  | 0 | 1 | 2 | 3 => simp [ofTag]
  | _ + 4 => simp [ofTag]

/-- The mode is one of the two integer modes. -/
def isInteger : ArithmeticMode → Bool
  | modular | checked => true
  | _ => false

/-- The mode is one of the two floating modes. -/
def isFloating : ArithmeticMode → Bool
  | strictFloat | exactDyadicRoundOnce => true
  | _ => false

theorem integer_or_floating (m : ArithmeticMode) :
    m.isInteger = true ∨ m.isFloating = true := by cases m <;> decide

theorem not_integer_and_floating (m : ArithmeticMode) :
    ¬ (m.isInteger = true ∧ m.isFloating = true) := by cases m <;> decide

end ArithmeticMode

/-! ## The closed compatibility table -/

/-- One row of SPEC §8.2's compatibility table. -/
structure CompatRow where
  /-- Row number in the table's displayed order (SPEC §8.3: rows `0 … 7`). -/
  index : Nat
  mode : ArithmeticMode
  /-- The stored kinds the row admits; A, B and C must all be *the same one*. -/
  storedKinds : List ScalarKind
  /-- The accumulator kinds the row requires. -/
  accumulatorKinds : List ScalarKind
  deriving DecidableEq, Repr, Inhabited

namespace CompatRow

/-- The row's premise on the stored A/B/C triple, ignoring the accumulator.
SPEC §8.3's compatible-row bitset is built from exactly this predicate. -/
def acceptsStored (r : CompatRow) (mode : ArithmeticMode) (a b c : ScalarKind) : Bool :=
  (mode == r.mode) && (a == b) && (b == c) && r.storedKinds.contains a

/-- The complete row premise, accumulator included. -/
def accepts (r : CompatRow) (mode : ArithmeticMode) (a b c acc : ScalarKind) : Bool :=
  r.acceptsStored mode a b c && r.accumulatorKinds.contains acc

theorem accepts_imp_acceptsStored {r : CompatRow} {mode : ArithmeticMode}
    {a b c acc : ScalarKind} (h : r.accepts mode a b c acc = true) :
    r.acceptsStored mode a b c = true := by
  simp only [accepts, Bool.and_eq_true] at h; exact h.1

theorem acceptsStored_same {r : CompatRow} {mode : ArithmeticMode}
    {a b c : ScalarKind} (h : r.acceptsStored mode a b c = true) :
    a = b ∧ b = c ∧ mode = r.mode := by
  simp only [acceptsStored, Bool.and_eq_true, beq_iff_eq] at h
  exact ⟨h.1.1.2, h.1.2, h.1.1.1⟩

end CompatRow

/-- SPEC §8.2's closed compatibility table, in the displayed order. -/
def compatibilityTable : List CompatRow :=
  [ { index := 0, mode := .modular
    , storedKinds := [.i8, .u8, .i16, .u16, .i32, .u32]
    , accumulatorKinds := [.u32] }
  , { index := 1, mode := .modular
    , storedKinds := [.i64, .u64]
    , accumulatorKinds := [.u64] }
  , { index := 2, mode := .checked
    , storedKinds := [.i8, .i16, .i32, .i64]
    , accumulatorKinds := [.i64] }
  , { index := 3, mode := .checked
    , storedKinds := [.u8, .u16, .u32, .u64]
    , accumulatorKinds := [.u64] }
  , { index := 4, mode := .strictFloat
    , storedKinds := [.binary16, .bfloat16]
    , accumulatorKinds := [.binary32, .binary64] }
  , { index := 5, mode := .strictFloat
    , storedKinds := [.binary32]
    , accumulatorKinds := [.binary32, .binary64] }
  , { index := 6, mode := .strictFloat
    , storedKinds := [.binary64]
    , accumulatorKinds := [.binary64] }
  , { index := 7, mode := .exactDyadicRoundOnce
    , storedKinds := [.binary16, .bfloat16, .binary32, .binary64]
    , accumulatorKinds := [.exactDyadic] } ]

/-- There are exactly eight rows. -/
theorem compatibilityTable_length : compatibilityTable.length = 8 := rfl

/-- SPEC §8.3: "Compatibility rows are numbered `0` through `7` in the table's
displayed order." -/
theorem compatibilityTable_indices :
    compatibilityTable.map CompatRow.index = [0, 1, 2, 3, 4, 5, 6, 7] := rfl

/-- The row numbering matches the position of the row in the table. -/
theorem compatibilityTable_index_eq (i : Nat) :
    (compatibilityTable[i]?).map CompatRow.index = if i < 8 then some i else none := by
  match i with
  | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 => rfl
  | _ + 8 => rfl

/-- The displayed mode column, in order. -/
theorem compatibilityTable_modes :
    compatibilityTable.map CompatRow.mode =
      [.modular, .modular, .checked, .checked,
       .strictFloat, .strictFloat, .strictFloat, .exactDyadicRoundOnce] := rfl

theorem compatibilityTable_index_lt : ∀ r ∈ compatibilityTable, r.index < 8 := by decide

theorem compatibilityTable_index_injective :
    ∀ r ∈ compatibilityTable, ∀ s ∈ compatibilityTable, r.index = s.index → r = s := by
  decide

theorem compatibilityTable_accumulators_nonempty :
    ∀ r ∈ compatibilityTable, r.accumulatorKinds ≠ [] := by decide

/-! ## The compatibility relation -/

/-- Boolean form of SPEC §8.2's compatibility relation. -/
def compatibleB (mode : ArithmeticMode) (a b c acc : ScalarKind) : Bool :=
  compatibilityTable.any (fun r => r.accepts mode a b c acc)

/-- SPEC §8.2's compatibility relation.  A tuple is compatible exactly when some
row of the closed table accepts it. -/
def Compatible (mode : ArithmeticMode) (a b c acc : ScalarKind) : Prop :=
  compatibleB mode a b c acc = true

instance (mode : ArithmeticMode) (a b c acc : ScalarKind) :
    Decidable (Compatible mode a b c acc) :=
  inferInstanceAs (Decidable (compatibleB mode a b c acc = true))

/-- Compatibility is exactly "some row of the closed table accepts the tuple". -/
theorem compatible_iff_exists_row (mode : ArithmeticMode) (a b c acc : ScalarKind) :
    Compatible mode a b c acc ↔
      ∃ r ∈ compatibilityTable, r.accepts mode a b c acc = true := by
  simp [Compatible, compatibleB, List.any_eq_true]

/-- No compatible tuple mixes kinds: A, B and C are literally the same kind. -/
theorem compatible_kinds_equal {mode : ArithmeticMode} {a b c acc : ScalarKind}
    (h : Compatible mode a b c acc) : a = b ∧ b = c := by
  obtain ⟨r, _, hr⟩ := (compatible_iff_exists_row mode a b c acc).mp h
  obtain ⟨h1, h2, _⟩ := CompatRow.acceptsStored_same (CompatRow.accepts_imp_acceptsStored hr)
  exact ⟨h1, h2⟩

/-- Every row of the table is entered by splitting on membership; this is the
shared skeleton of the structural facts below. -/
theorem mem_compatibilityTable_cases {r : CompatRow} (h : r ∈ compatibilityTable) :
    r = compatibilityTable[0]! ∨ r = compatibilityTable[1]! ∨
    r = compatibilityTable[2]! ∨ r = compatibilityTable[3]! ∨
    r = compatibilityTable[4]! ∨ r = compatibilityTable[5]! ∨
    r = compatibilityTable[6]! ∨ r = compatibilityTable[7]! := by
  simp only [compatibilityTable, List.mem_cons, List.not_mem_nil, or_false] at h
  simpa [compatibilityTable] using h

/-- The accumulator of a compatible tuple is one of the five released
accumulators of SPEC §8.1. -/
theorem compatible_accumulator {mode : ArithmeticMode} {a b c acc : ScalarKind}
    (h : Compatible mode a b c acc) :
    acc = .u32 ∨ acc = .u64 ∨ acc = .i64 ∨ acc = .binary32 ∨ acc = .binary64 ∨
      acc = .exactDyadic := by
  obtain ⟨r, hr, ha⟩ := (compatible_iff_exists_row mode a b c acc).mp h
  simp only [CompatRow.accepts, Bool.and_eq_true] at ha
  have hacc := ha.2
  rcases mem_compatibilityTable_cases hr with
    rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl <;>
    (revert hacc; cases acc <;> decide)

/-- `exactDyadic` is admissible only as an accumulator, never as a stored kind. -/
theorem compatible_stored_not_exactDyadic {mode : ArithmeticMode}
    {a b c acc : ScalarKind} (h : Compatible mode a b c acc) :
    a ≠ .exactDyadic ∧ b ≠ .exactDyadic ∧ c ≠ .exactDyadic := by
  obtain ⟨r, hr, ha⟩ := (compatible_iff_exists_row mode a b c acc).mp h
  have hs := CompatRow.accepts_imp_acceptsStored ha
  obtain ⟨hab, hbc, _⟩ := CompatRow.acceptsStored_same hs
  subst hab; subst hbc
  simp only [CompatRow.acceptsStored, Bool.and_eq_true] at hs
  have hmem := hs.2
  refine ⟨?_, ?_, ?_⟩ <;>
    (rcases mem_compatibilityTable_cases hr with
       rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl <;>
       (revert hmem; cases a <;> decide))

/-- `exactDyadic` accumulation happens exactly in the round-once mode. -/
theorem exactDyadic_acc_iff_mode {mode : ArithmeticMode} {a b c : ScalarKind}
    (h : Compatible mode a b c .exactDyadic) : mode = .exactDyadicRoundOnce := by
  obtain ⟨r, hr, ha⟩ := (compatible_iff_exists_row mode a b c .exactDyadic).mp h
  have hs := CompatRow.accepts_imp_acceptsStored ha
  obtain ⟨_, _, hmode⟩ := CompatRow.acceptsStored_same hs
  simp only [CompatRow.accepts, Bool.and_eq_true] at ha
  have hacc := ha.2
  subst hmode
  rcases mem_compatibilityTable_cases hr with
    rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl <;> (revert hacc; decide)

/-! ### The displayed rows, read back as theorems

Each of the eight rows is checked against the relation, so the table cannot be
silently edited without breaking a named theorem. -/

theorem row0_modular_u32 :
    Compatible .modular .i32 .i32 .i32 .u32 := by decide
theorem row0_requires_u32 (acc : ScalarKind) :
    Compatible .modular .i32 .i32 .i32 acc ↔ acc = .u32 := by
  cases acc <;> decide
theorem row1_modular_u64 :
    Compatible .modular .i64 .i64 .i64 .u64 := by decide
theorem row1_requires_u64 (acc : ScalarKind) :
    Compatible .modular .u64 .u64 .u64 acc ↔ acc = .u64 := by
  cases acc <;> decide
theorem row2_checked_i64 :
    Compatible .checked .i32 .i32 .i32 .i64 := by decide
theorem row2_requires_i64 (acc : ScalarKind) :
    Compatible .checked .i16 .i16 .i16 acc ↔ acc = .i64 := by
  cases acc <;> decide
theorem row3_checked_u64 :
    Compatible .checked .u32 .u32 .u32 .u64 := by decide
theorem row4_strictFloat_binary16 (acc : ScalarKind) :
    Compatible .strictFloat .binary16 .binary16 .binary16 acc ↔
      (acc = .binary32 ∨ acc = .binary64) := by
  cases acc <;> decide
theorem row4_strictFloat_bfloat16 (acc : ScalarKind) :
    Compatible .strictFloat .bfloat16 .bfloat16 .bfloat16 acc ↔
      (acc = .binary32 ∨ acc = .binary64) := by
  cases acc <;> decide
theorem row5_strictFloat_binary32 (acc : ScalarKind) :
    Compatible .strictFloat .binary32 .binary32 .binary32 acc ↔
      (acc = .binary32 ∨ acc = .binary64) := by
  cases acc <;> decide
theorem row6_strictFloat_binary64 (acc : ScalarKind) :
    Compatible .strictFloat .binary64 .binary64 .binary64 acc ↔ acc = .binary64 := by
  cases acc <;> decide
theorem row7_exactDyadic (acc : ScalarKind) :
    Compatible .exactDyadicRoundOnce .binary32 .binary32 .binary32 acc ↔
      acc = .exactDyadic := by
  cases acc <;> decide

/-- No mixed-signedness or mixed-width tuple is compatible. -/
theorem mixed_kinds_incompatible (mode : ArithmeticMode) (acc : ScalarKind) :
    ¬ Compatible mode .i32 .u32 .i32 acc := by
  cases mode <;> cases acc <;> decide

/-- Integer stored kinds are never compatible with a floating mode. -/
theorem integer_kinds_not_floating (a acc : ScalarKind) (mode : ArithmeticMode)
    (ha : a.isInteger = true) (hm : mode.isFloating = true) :
    ¬ Compatible mode a a a acc := by
  intro h
  obtain ⟨r, hr, hrow⟩ := (compatible_iff_exists_row mode a a a acc).mp h
  have hs := CompatRow.accepts_imp_acceptsStored hrow
  obtain ⟨_, _, hmode⟩ := CompatRow.acceptsStored_same hs
  simp only [CompatRow.acceptsStored, Bool.and_eq_true] at hs
  have hmem := hs.2
  subst hmode
  rcases mem_compatibilityTable_cases hr with
    rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl <;>
    (revert hmem ha hm; cases a <;> decide)

/-! ### The compatible-row bitset (SPEC §8.3)

"The compatible-row bitset is the sum of `2^r` for rows whose mode and
stored-kind premise accepts the observed A/B/C triple while ignoring only the
accumulator field." -/

/-- SPEC §8.3's compatible-row bitset. -/
def compatibleRowBitset (mode : ArithmeticMode) (a b c : ScalarKind) : Nat :=
  (compatibilityTable.filter (fun r => r.acceptsStored mode a b c)).foldl
    (fun acc r => acc + 2 ^ r.index) 0

theorem compatibleRowBitset_modular_i32 :
    compatibleRowBitset .modular .i32 .i32 .i32 = 1 := by decide
theorem compatibleRowBitset_checked_i32 :
    compatibleRowBitset .checked .i32 .i32 .i32 = 4 := by decide
theorem compatibleRowBitset_strictFloat_binary32 :
    compatibleRowBitset .strictFloat .binary32 .binary32 .binary32 = 32 := by decide
theorem compatibleRowBitset_mixed :
    compatibleRowBitset .modular .i32 .u32 .i32 = 0 := by decide

/-- The bitset is zero exactly when no row's stored premise accepts the triple,
i.e. exactly when no accumulator can rescue the tuple. -/
theorem foldl_add_pow_eq_zero {l : List CompatRow} {s : Nat}
    (h : l.foldl (fun acc r => acc + 2 ^ r.index) s = 0) : l = [] ∧ s = 0 := by
  induction l generalizing s with
  | nil => exact ⟨rfl, h⟩
  | cons a t ih =>
    have h2 : s + 2 ^ a.index = 0 := (ih h).2
    have hp : 0 < 2 ^ a.index := Nat.two_pow_pos a.index
    generalize 2 ^ a.index = p at h2 hp
    exact absurd h2 (by omega)

/-- The bitset vanishes exactly when the filtered row list is empty. -/
theorem compatibleRowBitset_eq_zero_iff_filter
    (mode : ArithmeticMode) (a b c : ScalarKind) :
    compatibleRowBitset mode a b c = 0 ↔
      compatibilityTable.filter (fun r => r.acceptsStored mode a b c) = [] := by
  constructor
  · intro h
    exact (foldl_add_pow_eq_zero (l := compatibilityTable.filter
      (fun r => r.acceptsStored mode a b c)) h).1
  · intro h
    simp [compatibleRowBitset, h]

/-- A compatible tuple always has a nonzero compatible-row bitset. -/
theorem compatible_bitset_ne_zero {mode : ArithmeticMode} {a b c acc : ScalarKind}
    (h : Compatible mode a b c acc) : compatibleRowBitset mode a b c ≠ 0 := by
  obtain ⟨r, hr, ha⟩ := (compatible_iff_exists_row mode a b c acc).mp h
  intro hz
  have hfilter := (compatibleRowBitset_eq_zero_iff_filter mode a b c).mp hz
  have hmem : r ∈ compatibilityTable.filter (fun r => r.acceptsStored mode a b c) :=
    List.mem_filter.mpr ⟨hr, CompatRow.accepts_imp_acceptsStored ha⟩
  rw [hfilter] at hmem
  exact absurd hmem (by simp)

/-! ## Integer ranges and extension (SPEC §8.2) -/

namespace ScalarKind

/-- Least representable mathematical value of a stored integer kind. -/
def minValue (k : ScalarKind) : Int :=
  if k.isSignedInteger then -((2 ^ (k.bitWidth - 1) : Nat) : Int) else 0

/-- Greatest representable mathematical value of a stored integer kind. -/
def maxValue (k : ScalarKind) : Int :=
  if k.isSignedInteger then ((2 ^ (k.bitWidth - 1) : Nat) : Int) - 1
  else ((2 ^ k.bitWidth : Nat) : Int) - 1

/-- The declared interval of a stored integer kind; checked mode tests every
product, running sum, scale and conversion against it (SPEC §8.2). -/
def InRange (k : ScalarKind) (z : Int) : Prop := k.minValue ≤ z ∧ z ≤ k.maxValue

instance (k : ScalarKind) (z : Int) : Decidable (k.InRange z) :=
  inferInstanceAs (Decidable (_ ∧ _))

theorem minValue_le_maxValue (k : ScalarKind) (h : k.isInteger = true) :
    k.minValue ≤ k.maxValue := by
  cases k <;> simp only [isInteger, reduceCtorEq] at h <;> decide

theorem inRange_zero (k : ScalarKind) (h : k.isInteger = true) : k.InRange 0 := by
  cases k <;> simp only [isInteger, reduceCtorEq] at h <;> decide

end ScalarKind

/-- The two's-complement mathematical value of a `w`-bit pattern. -/
def toSigned (w v : Nat) : Int :=
  if v < 2 ^ (w - 1) then (v : Int) else (v : Int) - ((2 ^ w : Nat) : Int)

/-- The `w`-bit pattern of a mathematical integer. -/
def ofSigned (w : Nat) (z : Int) : Nat := (z % ((2 ^ w : Nat) : Int)).toNat

/-- Zero extension to `dstBits` (SPEC §8.2, modular mode). -/
def zeroExtend (dstBits v : Nat) : Nat := v % 2 ^ dstBits

/-- Sign extension to `dstBits` (SPEC §8.2, modular mode). -/
def signExtend (srcBits dstBits v : Nat) : Nat :=
  if v < 2 ^ (srcBits - 1) then v % 2 ^ dstBits
  else (v + (2 ^ dstBits - 2 ^ srcBits)) % 2 ^ dstBits

/-- SPEC §8.2: "signed source bits are sign-extended and unsigned source bits are
zero-extended to the accumulator width". -/
def ScalarKind.extendTo (k : ScalarKind) (dstBits v : Nat) : Nat :=
  if k.isSignedInteger then signExtend k.bitWidth dstBits v else zeroExtend dstBits v

theorem zeroExtend_lt {dstBits v : Nat} : zeroExtend dstBits v < 2 ^ dstBits :=
  Nat.mod_lt _ (Nat.two_pow_pos _)

theorem signExtend_lt {srcBits dstBits v : Nat} : signExtend srcBits dstBits v < 2 ^ dstBits := by
  unfold signExtend
  split <;> exact Nat.mod_lt _ (Nat.two_pow_pos _)

theorem extendTo_lt (k : ScalarKind) (dstBits v : Nat) :
    k.extendTo dstBits v < 2 ^ dstBits := by
  unfold ScalarKind.extendTo
  split
  · exact signExtend_lt
  · exact zeroExtend_lt

theorem zeroExtend_eq_self {srcBits dstBits v : Nat} (hle : srcBits ≤ dstBits)
    (hv : v < 2 ^ srcBits) : zeroExtend dstBits v = v := by
  have : (2 : Nat) ^ srcBits ≤ 2 ^ dstBits := Nat.pow_le_pow_right (by decide) hle
  exact Nat.mod_eq_of_lt (by omega)

theorem two_pow_pred (w : Nat) (h : 0 < w) : (2 : Nat) ^ w = 2 * 2 ^ (w - 1) := by
  obtain ⟨m, rfl⟩ : ∃ m, w = m + 1 := ⟨w - 1, by omega⟩
  simp [Nat.pow_succ, Nat.mul_comm]

/-- Sign extension preserves the mathematical value: this is the *only* claim
SPEC §8.2 makes about bit extension, and it is proved rather than assumed. -/
theorem signExtend_toSigned {srcBits dstBits v : Nat}
    (hs : 0 < srcBits) (hle : srcBits ≤ dstBits) (hv : v < 2 ^ srcBits) :
    toSigned dstBits (signExtend srcBits dstBits v) = toSigned srcBits v := by
  have hA : (2 : Nat) ^ srcBits = 2 * 2 ^ (srcBits - 1) := two_pow_pred srcBits hs
  have hB : (2 : Nat) ^ dstBits = 2 * 2 ^ (dstBits - 1) := two_pow_pred dstBits (by omega)
  have hSD : (2 : Nat) ^ srcBits ≤ 2 ^ dstBits := Nat.pow_le_pow_right (by decide) hle
  have hAB : (2 : Nat) ^ (srcBits - 1) ≤ 2 ^ (dstBits - 1) :=
    Nat.pow_le_pow_right (by decide) (by omega)
  by_cases h : v < 2 ^ (srcBits - 1)
  · have e1 : signExtend srcBits dstBits v = v := by
      simp only [signExtend, if_pos h]
      exact Nat.mod_eq_of_lt (by omega)
    rw [e1]
    simp only [toSigned]
    rw [if_pos h, if_pos (show v < 2 ^ (dstBits - 1) by omega)]
  · have hx : v + (2 ^ dstBits - 2 ^ srcBits) < 2 ^ dstBits := by omega
    have e1 : signExtend srcBits dstBits v = v + (2 ^ dstBits - 2 ^ srcBits) := by
      simp only [signExtend, if_neg h]
      exact Nat.mod_eq_of_lt hx
    rw [e1]
    simp only [toSigned]
    rw [if_neg h,
      if_neg (show ¬ (v + (2 ^ dstBits - 2 ^ srcBits) < 2 ^ (dstBits - 1)) by omega)]
    omega

/-- Zero extension preserves the mathematical value of an unsigned pattern. -/
theorem zeroExtend_value {srcBits dstBits v : Nat} (hle : srcBits ≤ dstBits)
    (hv : v < 2 ^ srcBits) : (zeroExtend dstBits v : Int) = (v : Int) := by
  rw [zeroExtend_eq_self hle hv]

/-! ## Modular arithmetic (SPEC §8.2, modular mode)

"Modular integer mode converts operands to the declared accumulator width and
performs every multiply, add, alpha scale, beta scale, and output conversion
modulo that width." -/

/-- Modular addition at width `w`. -/
def modAdd (w a b : Nat) : Nat := (a + b) % 2 ^ w
/-- Modular multiplication at width `w`. -/
def modMul (w a b : Nat) : Nat := (a * b) % 2 ^ w
/-- Output conversion: the low `w` bits. -/
def modTruncate (w a : Nat) : Nat := a % 2 ^ w

theorem modAdd_lt (w a b : Nat) : modAdd w a b < 2 ^ w := Nat.mod_lt _ (Nat.two_pow_pos _)
theorem modMul_lt (w a b : Nat) : modMul w a b < 2 ^ w := Nat.mod_lt _ (Nat.two_pow_pos _)
theorem modTruncate_lt (w a : Nat) : modTruncate w a < 2 ^ w := Nat.mod_lt _ (Nat.two_pow_pos _)

theorem modAdd_comm (w a b : Nat) : modAdd w a b = modAdd w b a := by
  simp [modAdd, Nat.add_comm]

theorem modMul_comm (w a b : Nat) : modMul w a b = modMul w b a := by
  simp [modMul, Nat.mul_comm]

theorem modAdd_assoc (w a b c : Nat) :
    modAdd w (modAdd w a b) c = modAdd w a (modAdd w b c) := by
  simp only [modAdd]
  rw [Nat.mod_add_mod, Nat.add_mod_mod, Nat.add_assoc]

/-- Modular mode is *not* claimed to distribute or reassociate in any way beyond
these two proved laws; SPEC §8.2 forbids implicit ring structure. -/
theorem modAdd_zero (w a : Nat) (h : a < 2 ^ w) : modAdd w a 0 = a := by
  simp [modAdd, Nat.mod_eq_of_lt h]

/-! ## Round to nearest, ties to even (SPEC §8.2)

"All floating modes use round-to-nearest, ties-to-even". -/

/-- Round `den * q + r` divided by `den` to the nearest integer, ties to even. -/
def rneCore (den q r : Nat) : Nat :=
  if 2 * r < den then q
  else if den < 2 * r then q + 1
  else if q % 2 = 0 then q else q + 1

/-- Round `num / den` to the nearest natural, ties to even. -/
def rneNat (num den : Nat) : Nat := rneCore den (num / den) (num % den)

/-- Round a rational `num / den` to the nearest integer, ties to even.  Rounding
is sign-symmetric, as IEEE round-to-nearest requires. -/
def rne (num : Int) (den : Nat) : Int :=
  if 0 ≤ num then (rneNat num.toNat den : Int) else -((rneNat (-num).toNat den : Nat) : Int)

theorem rneCore_le (den q r : Nat) : q ≤ rneCore den q r ∧ rneCore den q r ≤ q + 1 := by
  unfold rneCore
  split
  · omega
  · split
    · omega
    · split <;> omega

/-- The rounded value is within one half-unit of the exact quotient: both
`2·value ≤ (2·N+1)·den` and `2·N·den ≤ 2·value + den` hold. -/
theorem rneCore_spec (den q r : Nat) (hden : 0 < den) (hr : r < den) :
    2 * (den * q + r) ≤ (2 * rneCore den q r + 1) * den ∧
      2 * rneCore den q r * den ≤ 2 * (den * q + r) + den := by
  have key : ∀ a : Nat, (2 * a + 1) * den = 2 * (den * a) + den := by
    intro a
    rw [Nat.add_mul, Nat.one_mul, Nat.mul_assoc, Nat.mul_comm a den]
  have key2 : ∀ a : Nat, 2 * a * den = 2 * (den * a) := by
    intro a
    rw [Nat.mul_assoc, Nat.mul_comm a den]
  have hs : den * (q + 1) = den * q + den := Nat.mul_succ den q
  have k1 := key q
  have k2 := key (q + 1)
  have k3 := key2 q
  have k4 := key2 (q + 1)
  unfold rneCore
  split
  · omega
  · split
    · omega
    · split <;> omega

theorem rneNat_spec (num den : Nat) (hden : 0 < den) :
    2 * num ≤ (2 * rneNat num den + 1) * den ∧
      2 * rneNat num den * den ≤ 2 * num + den := by
  have hnum : den * (num / den) + num % den = num := Nat.div_add_mod num den
  have hr : num % den < den := Nat.mod_lt _ hden
  have h := rneCore_spec den (num / den) (num % den) hden hr
  rw [hnum] at h
  exact h

/-- Ties really go to even. -/
theorem rneCore_tie_even (den q r : Nat) (h : 2 * r = den) :
    rneCore den q r % 2 = 0 := by
  unfold rneCore
  rw [if_neg (by omega), if_neg (by omega)]
  split <;> omega

theorem rneNat_tie_even (num den : Nat) (h : 2 * (num % den) = den) :
    rneNat num den % 2 = 0 :=
  rneCore_tie_even den (num / den) (num % den) h

theorem rneCore_exact (den q : Nat) (hden : 0 < den) : rneCore den q 0 = q := by
  unfold rneCore
  rw [if_pos (by omega)]

theorem rneNat_of_dvd (num den : Nat) (hden : 0 < den) (h : num % den = 0) :
    rneNat num den = num / den := by
  simp only [rneNat, h]
  exact rneCore_exact den (num / den) hden

@[simp] theorem rneNat_zero (den : Nat) (hden : 0 < den) : rneNat 0 den = 0 := by
  simp [rneNat, rneCore, hden]

@[simp] theorem rne_zero (den : Nat) (hden : 0 < den) : rne 0 den = 0 := by
  simp [rne, rneNat_zero den hden]

/-- Rounding is sign-symmetric. -/
theorem rne_neg (num : Int) (den : Nat) (hden : 0 < den) :
    rne (-num) den = - rne num den := by
  rcases Int.lt_trichotomy num 0 with h | h | h
  · rw [rne, rne, if_pos (by omega), if_neg (by omega)]
    simp
  · subst h
    simp [rne_zero den hden]
  · rw [rne, rne, if_neg (by omega), if_pos (by omega)]
    simp

/-! ## The closed special-value table (SPEC §8.2)

"any quiet NaN operand yields the canonical quiet NaN; `0 × infinity` yields
that NaN; nonzero finite times infinity yields the xor-signed infinity; adding
opposite infinities yields the canonical NaN; adding equal-signed infinities or
an infinity and a finite value yields that infinity." -/

/-- The coarse class of a floating operand, which is all the special-value table
inspects. -/
inductive SpecialClass
  | nan
  | inf (sign : Bool)
  | zero (sign : Bool)
  | finite (sign : Bool)
  deriving DecidableEq, Repr, Inhabited

namespace SpecialClass

/-- Classify a bit pattern of format `f`. -/
def ofBits (f : FloatFormat) (b : Nat) : SpecialClass :=
  match f.decode b with
  | .nan _ _ _ => .nan
  | .infinity s => .inf s
  | .zero s => .zero s
  | .subnormal s _ => .finite s
  | .normal s _ _ => .finite s

/-- SPEC §8.2's product row of the special-value table.  `none` means "no
special case applies; use the mode's ordinary arithmetic". -/
def mul : SpecialClass → SpecialClass → Option SpecialClass
  | .nan, _ => some .nan
  | _, .nan => some .nan
  | .inf s, .inf t => some (.inf (xor s t))
  | .inf _, .zero _ => some .nan
  | .zero _, .inf _ => some .nan
  | .inf s, .finite t => some (.inf (xor s t))
  | .finite s, .inf t => some (.inf (xor s t))
  | _, _ => none

/-- SPEC §8.2's sum row of the special-value table. -/
def add : SpecialClass → SpecialClass → Option SpecialClass
  | .nan, _ => some .nan
  | _, .nan => some .nan
  | .inf s, .inf t => if s = t then some (.inf s) else some .nan
  | .inf s, _ => some (.inf s)
  | _, .inf t => some (.inf t)
  | _, _ => none

theorem mul_nan_left (y : SpecialClass) : mul .nan y = some .nan := by cases y <;> rfl
theorem mul_nan_right (x : SpecialClass) : mul x .nan = some .nan := by cases x <;> rfl
theorem add_nan_left (y : SpecialClass) : add .nan y = some .nan := by cases y <;> rfl
theorem add_nan_right (x : SpecialClass) : add x .nan = some .nan := by cases x <;> rfl

/-- `0 × infinity` is the canonical quiet NaN. -/
theorem mul_zero_inf (s t : Bool) : mul (.zero s) (.inf t) = some .nan := rfl
theorem mul_inf_zero (s t : Bool) : mul (.inf s) (.zero t) = some .nan := rfl

/-- Nonzero finite times infinity is the xor-signed infinity. -/
theorem mul_finite_inf (s t : Bool) :
    mul (.finite s) (.inf t) = some (.inf (xor s t)) := rfl
theorem mul_inf_finite (s t : Bool) :
    mul (.inf s) (.finite t) = some (.inf (xor s t)) := rfl
theorem mul_inf_inf (s t : Bool) :
    mul (.inf s) (.inf t) = some (.inf (xor s t)) := rfl

/-- Adding opposite infinities is the canonical NaN. -/
theorem add_opposite_inf (s t : Bool) (h : s ≠ t) : add (.inf s) (.inf t) = some .nan := by
  simp [add, h]

/-- Adding equal-signed infinities is that infinity. -/
theorem add_same_inf (s : Bool) : add (.inf s) (.inf s) = some (.inf s) := by
  simp [add]

/-- Adding an infinity and a finite value is that infinity. -/
theorem add_inf_finite (s t : Bool) : add (.inf s) (.finite t) = some (.inf s) := rfl
theorem add_inf_zero (s t : Bool) : add (.inf s) (.zero t) = some (.inf s) := rfl
theorem add_finite_inf (s t : Bool) : add (.finite s) (.inf t) = some (.inf t) := rfl
theorem add_zero_inf (s t : Bool) : add (.zero s) (.inf t) = some (.inf t) := rfl

/-- An operand that the special tables react to: NaN or infinity. -/
def isSpecial : SpecialClass → Bool
  | nan => true
  | inf _ => true
  | _ => false

/-- The product table applies exactly when an operand is NaN or infinite. -/
theorem mul_eq_none_iff (x y : SpecialClass) :
    mul x y = none ↔ (x.isSpecial = false ∧ y.isSpecial = false) := by
  cases x <;> cases y <;> simp [mul, isSpecial]

/-- The sum table applies exactly when an operand is NaN or infinite. -/
theorem add_eq_none_iff (x y : SpecialClass) :
    add x y = none ↔ (x.isSpecial = false ∧ y.isSpecial = false) := by
  cases x with
  | nan => cases y <;> simp [add, isSpecial]
  | inf s =>
      cases y with
      | nan => simp [add, isSpecial]
      | inf t => cases s <;> cases t <;> simp [add, isSpecial]
      | zero t => simp [add, isSpecial]
      | finite t => simp [add, isSpecial]
  | zero s => cases y <;> simp [add, isSpecial]
  | finite s => cases y <;> simp [add, isSpecial]

/-- The product table is commutative. -/
theorem mul_comm (x y : SpecialClass) : mul x y = mul y x := by
  cases x <;> cases y <;> simp [mul, Bool.xor_comm]

/-- The sum table is commutative. -/
theorem add_comm (x y : SpecialClass) : add x y = add y x := by
  cases x <;> cases y <;> try rfl
  case inf.inf s t =>
    by_cases h : s = t
    · subst h; rfl
    · show (if s = t then some (SpecialClass.inf s) else some SpecialClass.nan)
        = (if t = s then some (SpecialClass.inf t) else some SpecialClass.nan)
      rw [if_neg h, if_neg (Ne.symm h)]

end SpecialClass

/-! ### Exact-dyadic reduction (SPEC §8.2)

"Exact-dyadic reduction first applies this product table, yields canonical NaN
if both infinity signs occur among the scaled terms, yields the sole infinity
sign if exactly one occurs, and otherwise rounds the finite exact dyadic." -/

/-- Does the term list contain a NaN? -/
def anyNaN (terms : List SpecialClass) : Bool :=
  terms.any (fun t => t == .nan)

/-- Does the term list contain an infinity of the given sign? -/
def anyInf (terms : List SpecialClass) (s : Bool) : Bool :=
  terms.any (fun t => t == .inf s)

/-- SPEC §8.2's exact-dyadic reduction rule.  `none` means "no special value:
round the finite exact dyadic". -/
def dyadicReduce (terms : List SpecialClass) : Option SpecialClass :=
  if anyNaN terms then some .nan
  else if anyInf terms true && anyInf terms false then some .nan
  else if anyInf terms true then some (.inf true)
  else if anyInf terms false then some (.inf false)
  else none

theorem dyadicReduce_nan {terms : List SpecialClass} (h : anyNaN terms = true) :
    dyadicReduce terms = some .nan := by simp [dyadicReduce, h]

theorem dyadicReduce_both_inf {terms : List SpecialClass}
    (hn : anyNaN terms = false) (h1 : anyInf terms true = true)
    (h2 : anyInf terms false = true) : dyadicReduce terms = some .nan := by
  simp [dyadicReduce, hn, h1, h2]

theorem dyadicReduce_sole_inf {terms : List SpecialClass} {s : Bool}
    (hn : anyNaN terms = false) (h1 : anyInf terms s = true)
    (h2 : anyInf terms (!s) = false) : dyadicReduce terms = some (.inf s) := by
  cases s <;> simp_all [dyadicReduce]

theorem dyadicReduce_finite {terms : List SpecialClass}
    (hn : anyNaN terms = false) (h1 : anyInf terms true = false)
    (h2 : anyInf terms false = false) : dyadicReduce terms = none := by
  simp [dyadicReduce, hn, h1, h2]

theorem dyadicReduce_nil : dyadicReduce [] = none := by decide

/-- The reduction is a total function of the term list. -/
theorem dyadicReduce_total (terms : List SpecialClass) :
    ∃ r, dyadicReduce terms = r := ⟨_, rfl⟩

/-! ## Exact-zero sign rule (SPEC §8.2)

"an exact zero is encoded as `+0`, except a negative exact result that
underflows to zero is `-0`." -/

/-- Sign of the encoded zero for an exact result that rounds to zero:
`false` (i.e. `+0`) for an exactly zero result, `true` (i.e. `-0`) for a
negative result that underflows. -/
def exactZeroSign (exactIsZero : Bool) (exactIsNegative : Bool) : Bool :=
  if exactIsZero then false else exactIsNegative

theorem exactZeroSign_exact_zero (n : Bool) : exactZeroSign true n = false := rfl
theorem exactZeroSign_negative_underflow : exactZeroSign false true = true := rfl
theorem exactZeroSign_positive_underflow : exactZeroSign false false = false := rfl

end WasmGemmGnaf.Gemm
