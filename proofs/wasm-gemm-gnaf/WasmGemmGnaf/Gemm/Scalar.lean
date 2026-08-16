set_option autoImplicit false

/-!
# Gemm: stored scalar encodings (SPEC §8.1, §8.3)

The released ABI admits exactly thirteen scalar tags.  SPEC §8.3 fixes their
numbering:

`i8=0`, `u8=1`, `i16=2`, `u16=3`, `i32=4`, `u32=5`, `i64=6`, `u64=7`,
`binary16=8`, `bfloat16=9`, `binary32=10`, `binary64=11`, `exactDyadic=12`,

where `exactDyadic` is permitted only as an accumulator.

SPEC §8.1 additionally fixes the alignment story: "`Layout.alignment` is the
derived constant one for every stored kind".  It is therefore a *function of no
input* here, and `alignment_eq_one` records that no stronger promise can be
inferred.
-/

namespace WasmGemmGnaf.Gemm

/-- The closed set of released scalar encodings (SPEC §8.1). -/
inductive ScalarKind
  | i8 | u8 | i16 | u16 | i32 | u32 | i64 | u64
  | binary16 | bfloat16 | binary32 | binary64 | exactDyadic
  deriving DecidableEq, Repr, Inhabited

namespace ScalarKind

/-! ## Tag numbering (SPEC §8.3) -/

/-- The ABI tag byte of a scalar kind, exactly as printed in SPEC §8.3. -/
def tag : ScalarKind → Nat
  | i8 => 0
  | u8 => 1
  | i16 => 2
  | u16 => 3
  | i32 => 4
  | u32 => 5
  | i64 => 6
  | u64 => 7
  | binary16 => 8
  | bfloat16 => 9
  | binary32 => 10
  | binary64 => 11
  | exactDyadic => 12

/-- Inverse of `tag`; `none` for every unassigned tag byte. -/
def ofTag : Nat → Option ScalarKind
  | 0 => some i8
  | 1 => some u8
  | 2 => some i16
  | 3 => some u16
  | 4 => some i32
  | 5 => some u32
  | 6 => some i64
  | 7 => some u64
  | 8 => some binary16
  | 9 => some bfloat16
  | 10 => some binary32
  | 11 => some binary64
  | 12 => some exactDyadic
  | _ + 13 => none

/-- The complete finite enumeration, in tag order. -/
def all : List ScalarKind :=
  [i8, u8, i16, u16, i32, u32, i64, u64,
   binary16, bfloat16, binary32, binary64, exactDyadic]

theorem mem_all (k : ScalarKind) : k ∈ all := by cases k <;> decide

theorem all_nodup : all.Nodup := by decide

theorem all_length : all.length = 13 := rfl

theorem all_map_tag : all.map tag = List.range 13 := by decide

/-- Every tag byte is one of `0 … 12`. -/
theorem tag_lt (k : ScalarKind) : k.tag < 13 := by cases k <;> decide

theorem tag_injective : Function.Injective tag := by
  intro a b h
  cases a <;> cases b <;> simp_all [tag]

@[simp] theorem ofTag_tag (k : ScalarKind) : ofTag k.tag = some k := by
  cases k <;> rfl

theorem tag_ofTag {n : Nat} {k : ScalarKind} (h : ofTag n = some k) : k.tag = n := by
  match n with
  | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 =>
      simp only [ofTag, Option.some.injEq] at h; subst h; rfl
  | _ + 13 => exact absurd h (by simp [ofTag])

/-- The tag map and its inverse are mutually inverse on the assigned range. -/
theorem ofTag_eq_some_iff {n : Nat} {k : ScalarKind} :
    ofTag n = some k ↔ k.tag = n :=
  ⟨tag_ofTag, fun h => h ▸ ofTag_tag k⟩

theorem ofTag_eq_none_iff (n : Nat) : ofTag n = none ↔ 13 ≤ n := by
  match n with
  | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 => simp [ofTag]
  | m + 13 => simp [ofTag]

theorem ofTag_isSome_iff (n : Nat) : (ofTag n).isSome = true ↔ n < 13 := by
  match n with
  | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 => simp [ofTag]
  | m + 13 => simp [ofTag]

/-! ## Widths -/

/-- Stored width in bytes.  `exactDyadic` has no stored form, hence width `0`. -/
def byteWidth : ScalarKind → Nat
  | i8 | u8 => 1
  | i16 | u16 | binary16 | bfloat16 => 2
  | i32 | u32 | binary32 => 4
  | i64 | u64 | binary64 => 8
  | exactDyadic => 0

/-- Stored width in bits. -/
def bitWidth (k : ScalarKind) : Nat := 8 * k.byteWidth

/-- `exactDyadic` is the only kind that is not a stored encoding. -/
def isStored : ScalarKind → Bool
  | exactDyadic => false
  | _ => true

theorem byteWidth_pos_iff (k : ScalarKind) : 0 < k.byteWidth ↔ k.isStored = true := by
  cases k <;> decide

theorem byteWidth_le_eight (k : ScalarKind) : k.byteWidth ≤ 8 := by cases k <;> decide

theorem bitWidth_le_sixtyFour (k : ScalarKind) : k.bitWidth ≤ 64 := by cases k <;> decide

/-! ## Families -/

def isInteger : ScalarKind → Bool
  | i8 | u8 | i16 | u16 | i32 | u32 | i64 | u64 => true
  | _ => false

def isSignedInteger : ScalarKind → Bool
  | i8 | i16 | i32 | i64 => true
  | _ => false

def isUnsignedInteger : ScalarKind → Bool
  | u8 | u16 | u32 | u64 => true
  | _ => false

def isFloat : ScalarKind → Bool
  | binary16 | bfloat16 | binary32 | binary64 => true
  | _ => false

def isExactDyadic : ScalarKind → Bool
  | exactDyadic => true
  | _ => false

/-- The four families partition the thirteen kinds. -/
theorem family_partition (k : ScalarKind) :
    (k.isInteger = true ∧ k.isFloat = false ∧ k.isExactDyadic = false) ∨
    (k.isInteger = false ∧ k.isFloat = true ∧ k.isExactDyadic = false) ∨
    (k.isInteger = false ∧ k.isFloat = false ∧ k.isExactDyadic = true) := by
  cases k <;> decide

theorem isInteger_iff_signed_or_unsigned (k : ScalarKind) :
    k.isInteger = true ↔ (k.isSignedInteger = true ∨ k.isUnsignedInteger = true) := by
  cases k <;> decide

theorem signed_not_unsigned (k : ScalarKind) :
    ¬ (k.isSignedInteger = true ∧ k.isUnsignedInteger = true) := by
  cases k <;> decide

theorem isStored_iff_not_exactDyadic (k : ScalarKind) :
    k.isStored = true ↔ k ≠ exactDyadic := by
  cases k <;> simp [isStored]

/-! ## Alignment (SPEC §8.1)

"The released ABI has no alignment field and imposes byte alignment only:
`Layout.alignment` is the derived constant one for every stored kind." -/

/-- The derived alignment of every stored kind: the constant `1`. -/
def alignment (_k : ScalarKind) : Nat := 1

@[simp] theorem alignment_eq_one (k : ScalarKind) : k.alignment = 1 := rfl

/-- No address or width can raise the promised alignment above one. -/
theorem alignment_constant (j k : ScalarKind) : j.alignment = k.alignment := rfl

/-! ## Raw stored values -/

/-- The set of representable stored bit patterns of a kind. -/
def valueBound (k : ScalarKind) : Nat := 2 ^ k.bitWidth

theorem valueBound_pos (k : ScalarKind) : 0 < k.valueBound :=
  Nat.two_pow_pos _

end ScalarKind

/-- A raw stored scalar value: the little-endian bit pattern of `k`, together
with the proof that it fits the stored width.  For `exactDyadic` the only value
is `0`, since it is never stored. -/
structure ScalarValue (k : ScalarKind) where
  bits : Nat
  bits_lt : bits < k.valueBound

namespace ScalarValue

variable {k : ScalarKind}

theorem ext {a b : ScalarValue k} (h : a.bits = b.bits) : a = b := by
  cases a; cases b; cases h; rfl

theorem ext_iff {a b : ScalarValue k} : a = b ↔ a.bits = b.bits :=
  ⟨fun h => h ▸ rfl, ext⟩

instance : DecidableEq (ScalarValue k) := fun a b =>
  if h : a.bits = b.bits then isTrue (ext h) else isFalse (fun hc => h (hc ▸ rfl))

/-- The all-zero pattern. -/
def zero (k : ScalarKind) : ScalarValue k :=
  ⟨0, k.valueBound_pos⟩

instance : Inhabited (ScalarValue k) := ⟨zero k⟩

/-- Truncate an arbitrary natural to a stored value of `k`. -/
def ofNat (k : ScalarKind) (n : Nat) : ScalarValue k :=
  ⟨n % k.valueBound, Nat.mod_lt _ k.valueBound_pos⟩

@[simp] theorem ofNat_bits (k : ScalarKind) (n : Nat) :
    (ofNat k n).bits = n % k.valueBound := rfl

theorem ofNat_bits_of_lt {k : ScalarKind} {n : Nat} (h : n < k.valueBound) :
    (ofNat k n).bits = n := by
  simp [ofNat, Nat.mod_eq_of_lt h]

end ScalarValue

end WasmGemmGnaf.Gemm
