import WasmGemmGnaf.Gemm.Scalar
set_option autoImplicit false

/-!
# Gemm: floating-point bit layouts (SPEC §8.2)

SPEC §8.2 pins the canonical quiet-NaN bit patterns of the released formats:

* `binary16`  → `0x7e00`
* `bfloat16`  → `0x7fc0`
* `binary32`  → `0x7fc00000`
* `binary64`  → `0x7ff8000000000000`

and makes a signaling NaN in `alpha`, `beta`, `A`, `B` or the entry `C` the sole
released trigger for status `5`.  This module gives the bit-level decode/encode
for the four released formats, the canonical NaN constants (proved equal to the
literals above, not assumed), and the signaling-NaN detection predicate.

Everything is over `Nat` bit patterns: SPEC §8.3 requires address and value
arithmetic to be done over mathematical integers before any machine conversion.
-/

namespace WasmGemmGnaf.Gemm

/-- A binary interchange format: one sign bit, `expBits` exponent bits and
`mantBits` trailing significand bits. -/
structure FloatFormat where
  expBits : Nat
  mantBits : Nat
  deriving DecidableEq, Repr, Inhabited

namespace FloatFormat

/-- Total width of the encoding in bits. -/
def totalBits (f : FloatFormat) : Nat := 1 + f.expBits + f.mantBits

/-- The all-ones biased exponent, i.e. the infinity/NaN exponent field. -/
def expMax (f : FloatFormat) : Nat := 2 ^ f.expBits - 1

/-- IEEE exponent bias. -/
def bias (f : FloatFormat) : Nat := 2 ^ (f.expBits - 1) - 1

/-- The quiet bit: the most significant trailing significand bit. -/
def quietBitMask (f : FloatFormat) : Nat := 2 ^ (f.mantBits - 1)

/-- The canonical quiet NaN of SPEC §8.2: all-ones exponent, quiet bit set,
every other significand bit clear, sign clear. -/
def canonicalQuietNaN (f : FloatFormat) : Nat :=
  f.expMax * 2 ^ f.mantBits + f.quietBitMask

/-! ## The four released formats -/

/-- IEEE 754 `binary16`. -/
def binary16 : FloatFormat := ⟨5, 10⟩
/-- `bfloat16`: 8 exponent bits, 7 trailing significand bits. -/
def bfloat16 : FloatFormat := ⟨8, 7⟩
/-- IEEE 754 `binary32`. -/
def binary32 : FloatFormat := ⟨8, 23⟩
/-- IEEE 754 `binary64`. -/
def binary64 : FloatFormat := ⟨11, 52⟩

@[simp] theorem binary16_totalBits : binary16.totalBits = 16 := rfl
@[simp] theorem bfloat16_totalBits : bfloat16.totalBits = 16 := rfl
@[simp] theorem binary32_totalBits : binary32.totalBits = 32 := rfl
@[simp] theorem binary64_totalBits : binary64.totalBits = 64 := rfl

/-! ### The pinned canonical quiet NaN constants (SPEC §8.2) -/

theorem canonicalQuietNaN_binary16 : binary16.canonicalQuietNaN = 0x7e00 := by decide
theorem canonicalQuietNaN_bfloat16 : bfloat16.canonicalQuietNaN = 0x7fc0 := by decide
theorem canonicalQuietNaN_binary32 : binary32.canonicalQuietNaN = 0x7fc00000 := by decide
theorem canonicalQuietNaN_binary64 :
    binary64.canonicalQuietNaN = 0x7ff8000000000000 := by decide

/-! ## Field extraction and assembly -/

/-- Assemble a bit pattern from sign, biased exponent and trailing significand. -/
def pack (f : FloatFormat) (s : Bool) (e m : Nat) : Nat :=
  m + 2 ^ f.mantBits * (e + 2 ^ f.expBits * (if s then 1 else 0))

/-- The sign bit. -/
def signOf (f : FloatFormat) (b : Nat) : Bool :=
  decide (b / 2 ^ (f.mantBits + f.expBits) % 2 = 1)

/-- The biased exponent field. -/
def expOf (f : FloatFormat) (b : Nat) : Nat :=
  b / 2 ^ f.mantBits % 2 ^ f.expBits

/-- The trailing significand field. -/
def mantOf (f : FloatFormat) (b : Nat) : Nat :=
  b % 2 ^ f.mantBits

@[simp] theorem mantOf_pack (f : FloatFormat) (s : Bool) (e m : Nat)
    (hm : m < 2 ^ f.mantBits) : f.mantOf (f.pack s e m) = m := by
  simp only [mantOf, pack, Nat.add_mul_mod_self_left]
  exact Nat.mod_eq_of_lt hm

@[simp] theorem expOf_pack (f : FloatFormat) (s : Bool) (e m : Nat)
    (he : e < 2 ^ f.expBits) (hm : m < 2 ^ f.mantBits) :
    f.expOf (f.pack s e m) = e := by
  have hM : (0 : Nat) < 2 ^ f.mantBits := Nat.two_pow_pos _
  simp only [expOf, pack]
  rw [Nat.add_mul_div_left _ _ hM, Nat.div_eq_of_lt hm, Nat.zero_add,
    Nat.add_mul_mod_self_left, Nat.mod_eq_of_lt he]

theorem div_pack (f : FloatFormat) (s : Bool) (e m : Nat)
    (he : e < 2 ^ f.expBits) (hm : m < 2 ^ f.mantBits) :
    f.pack s e m / 2 ^ (f.mantBits + f.expBits) = (if s then 1 else 0) := by
  have hM : (0 : Nat) < 2 ^ f.mantBits := Nat.two_pow_pos _
  have hE : (0 : Nat) < 2 ^ f.expBits := Nat.two_pow_pos _
  simp only [pack, Nat.pow_add, ← Nat.div_div_eq_div_mul]
  rw [Nat.add_mul_div_left _ _ hM, Nat.div_eq_of_lt hm, Nat.zero_add,
    Nat.add_mul_div_left _ _ hE, Nat.div_eq_of_lt he, Nat.zero_add]

@[simp] theorem signOf_pack (f : FloatFormat) (s : Bool) (e m : Nat)
    (he : e < 2 ^ f.expBits) (hm : m < 2 ^ f.mantBits) :
    f.signOf (f.pack s e m) = s := by
  simp only [signOf, div_pack f s e m he hm]
  cases s <;> simp

theorem pack_lt (f : FloatFormat) (s : Bool) {e m : Nat}
    (he : e < 2 ^ f.expBits) (hm : m < 2 ^ f.mantBits) :
    f.pack s e m < 2 ^ f.totalBits := by
  have hE : (0 : Nat) < 2 ^ f.expBits := Nat.two_pow_pos _
  have hpow : (2 : Nat) ^ f.totalBits = 2 ^ f.mantBits * (2 * 2 ^ f.expBits) := by
    simp only [totalBits, Nat.pow_add, Nat.pow_one]
    simp [Nat.mul_comm, Nat.mul_assoc]
  have h1 : e + 2 ^ f.expBits * (if s then 1 else 0) + 1 ≤ 2 * 2 ^ f.expBits := by
    cases s <;> simp <;> omega
  rw [hpow]
  calc f.pack s e m
      < 2 ^ f.mantBits * (e + 2 ^ f.expBits * (if s then 1 else 0) + 1) := by
        simp only [pack, Nat.mul_succ]; omega
    _ ≤ 2 ^ f.mantBits * (2 * 2 ^ f.expBits) := Nat.mul_le_mul_left _ h1

theorem expOf_lt (f : FloatFormat) (b : Nat) : f.expOf b < 2 ^ f.expBits :=
  Nat.mod_lt _ (Nat.two_pow_pos _)

theorem mantOf_lt (f : FloatFormat) (b : Nat) : f.mantOf b < 2 ^ f.mantBits :=
  Nat.mod_lt _ (Nat.two_pow_pos _)

/-- Every representable bit pattern is the assembly of its own fields. -/
theorem pack_fields (f : FloatFormat) {b : Nat} (hb : b < 2 ^ f.totalBits) :
    f.pack (f.signOf b) (f.expOf b) (f.mantOf b) = b := by
  have hM : (0 : Nat) < 2 ^ f.mantBits := Nat.two_pow_pos _
  have hE : (0 : Nat) < 2 ^ f.expBits := Nat.two_pow_pos _
  have hpow2 : (2 : Nat) ^ f.totalBits = 2 ^ (f.mantBits + f.expBits) * 2 := by
    simp only [totalBits, Nat.pow_add, Nat.pow_one]
    simp [Nat.mul_comm, Nat.mul_assoc]
  have hb' : b < 2 ^ (f.mantBits + f.expBits) * 2 := by rw [← hpow2]; exact hb
  have hq : b / 2 ^ (f.mantBits + f.expBits) < 2 := Nat.div_lt_of_lt_mul hb'
  have hdd : b / 2 ^ f.mantBits / 2 ^ f.expBits = b / 2 ^ (f.mantBits + f.expBits) := by
    rw [Nat.div_div_eq_div_mul, ← Nat.pow_add]
  have key : ∀ q : Nat, q < 2 → (if decide (q % 2 = 1) then 1 else 0) = q := by
    intro q hq2
    match q with
    | 0 => rfl
    | 1 => rfl
    | (_ + 2) => omega
  have hsign : (if f.signOf b then 1 else 0) = b / 2 ^ (f.mantBits + f.expBits) := by
    simp only [signOf]
    exact key _ hq
  simp only [pack, expOf, mantOf, hsign, ← hdd]
  rw [Nat.mod_add_div (b / 2 ^ f.mantBits) (2 ^ f.expBits),
    Nat.mod_add_div b (2 ^ f.mantBits)]

/-! ## Classification -/

/-- The decoded meaning of a bit pattern. -/
inductive FloatClass
  | zero (sign : Bool)
  | subnormal (sign : Bool) (mant : Nat)
  | normal (sign : Bool) (exp : Nat) (mant : Nat)
  | infinity (sign : Bool)
  | nan (sign : Bool) (quiet : Bool) (payload : Nat)
  deriving DecidableEq, Repr, Inhabited

namespace FloatClass

/-- Sign of a decoded value. -/
def sign : FloatClass → Bool
  | zero s | subnormal s _ | normal s _ _ | infinity s | nan s _ _ => s

/-- A value is finite when it is zero, subnormal or normal. -/
def isFinite : FloatClass → Bool
  | zero _ | subnormal _ _ | normal _ _ _ => true
  | infinity _ | nan _ _ _ => false

def isNaN : FloatClass → Bool
  | nan _ _ _ => true
  | _ => false

def isInfinity : FloatClass → Bool
  | infinity _ => true
  | _ => false

def isZero : FloatClass → Bool
  | zero _ => true
  | _ => false

theorem isFinite_iff_not_nan_not_inf (c : FloatClass) :
    c.isFinite = true ↔ (c.isNaN = false ∧ c.isInfinity = false) := by
  cases c <;> simp [isFinite, isNaN, isInfinity]

end FloatClass

/-- Decode a bit pattern of format `f`. -/
def decode (f : FloatFormat) (b : Nat) : FloatClass :=
  let s := f.signOf b
  let e := f.expOf b
  let m := f.mantOf b
  if e = 0 then
    if m = 0 then .zero s else .subnormal s m
  else if e = f.expMax then
    if m = 0 then .infinity s else .nan s (decide (f.quietBitMask ≤ m)) m
  else .normal s e m

/-- Encode a decoded value back to a bit pattern of format `f`. -/
def encode (f : FloatFormat) : FloatClass → Nat
  | .zero s => f.pack s 0 0
  | .subnormal s m => f.pack s 0 m
  | .normal s e m => f.pack s e m
  | .infinity s => f.pack s f.expMax 0
  | .nan s _ p => f.pack s f.expMax p

/-- The bit patterns a decoded value must satisfy to be in canonical range. -/
def FloatClass.Valid (f : FloatFormat) : FloatClass → Prop
  | .zero _ => True
  | .subnormal _ m => 0 < m ∧ m < 2 ^ f.mantBits
  | .normal _ e m => 0 < e ∧ e ≠ f.expMax ∧ e < 2 ^ f.expBits ∧ m < 2 ^ f.mantBits
  | .infinity _ => True
  | .nan _ q p => 0 < p ∧ p < 2 ^ f.mantBits ∧ q = decide (f.quietBitMask ≤ p)

theorem expMax_lt (f : FloatFormat) : f.expMax < 2 ^ f.expBits := by
  have := Nat.two_pow_pos f.expBits
  simp only [expMax]
  omega

theorem encode_lt (f : FloatFormat) {c : FloatClass} (h : c.Valid f) :
    f.encode c < 2 ^ f.totalBits := by
  have hE := Nat.two_pow_pos f.expBits
  have hM := Nat.two_pow_pos f.mantBits
  cases c with
  | zero s => exact pack_lt f s hE hM
  | subnormal s m => exact pack_lt f s hE h.2
  | normal s e m => exact pack_lt f s h.2.2.1 h.2.2.2
  | infinity s => exact pack_lt f s (expMax_lt f) hM
  | nan s q p => exact pack_lt f s (expMax_lt f) h.2.1

/-- Round trip: encoding a canonical decoded value and decoding it again is the
identity.  In particular this covers every finite value. -/
theorem decode_encode (f : FloatFormat) (hpos : 0 < f.expBits) {c : FloatClass}
    (h : c.Valid f) : f.decode (f.encode c) = c := by
  have hE := Nat.two_pow_pos f.expBits
  have hM := Nat.two_pow_pos f.mantBits
  have hmaxpos : 0 < f.expMax := by
    have : (2 : Nat) ^ 1 ≤ 2 ^ f.expBits := Nat.pow_le_pow_right (by decide) hpos
    simp only [expMax]
    omega
  cases c with
  | zero s =>
    simp only [encode, decode, mantOf_pack f s 0 0 hM, expOf_pack f s 0 0 hE hM,
      signOf_pack f s 0 0 hE hM]
    simp
  | subnormal s m =>
    obtain ⟨hp, hlt⟩ := h
    simp only [encode, decode, mantOf_pack f s 0 m hlt, expOf_pack f s 0 m hE hlt,
      signOf_pack f s 0 m hE hlt]
    simp [Nat.ne_of_gt hp]
  | normal s e m =>
    obtain ⟨hp, hne, he, hm⟩ := h
    simp only [encode, decode, mantOf_pack f s e m hm, expOf_pack f s e m he hm,
      signOf_pack f s e m he hm]
    simp [Nat.ne_of_gt hp, hne]
  | infinity s =>
    simp only [encode, decode, mantOf_pack f s f.expMax 0 hM,
      expOf_pack f s f.expMax 0 (expMax_lt f) hM,
      signOf_pack f s f.expMax 0 (expMax_lt f) hM]
    simp [Nat.ne_of_gt hmaxpos]
  | nan s q p =>
    obtain ⟨hp, hlt, hq⟩ := h
    simp only [encode, decode, mantOf_pack f s f.expMax p hlt,
      expOf_pack f s f.expMax p (expMax_lt f) hlt,
      signOf_pack f s f.expMax p (expMax_lt f) hlt]
    simp [Nat.ne_of_gt hmaxpos, Nat.ne_of_gt hp, hq]

/-- Round trip in the other direction: every representable bit pattern decodes
and re-encodes to itself. -/
theorem encode_decode (f : FloatFormat) {b : Nat} (hb : b < 2 ^ f.totalBits) :
    f.encode (f.decode b) = b := by
  have hpack := pack_fields f hb
  have key : f.encode (f.decode b)
      = f.pack (f.signOf b) (f.expOf b) (f.mantOf b) := by
    simp only [decode]
    split
    · next he =>
      split
      · next hm => simp only [encode]; rw [he, hm]
      · next _ => simp only [encode]; rw [he]
    · next _ =>
      split
      · next he2 =>
        split
        · next hm => simp only [encode]; rw [he2, hm]
        · next _ => simp only [encode]; rw [he2]
      · next _ => simp only [encode]
  rw [key, hpack]

/-- Every decoded representable pattern is canonical. -/
theorem decode_valid (f : FloatFormat) (b : Nat) : (f.decode b).Valid f := by
  simp only [decode]
  split
  · next he =>
    split
    · trivial
    · next hm => exact ⟨Nat.pos_of_ne_zero hm, mantOf_lt f b⟩
  · next he =>
    split
    · next he2 =>
      split
      · trivial
      · next hm => exact ⟨Nat.pos_of_ne_zero hm, mantOf_lt f b, rfl⟩
    · next he2 => exact ⟨Nat.pos_of_ne_zero he, he2, expOf_lt f b, mantOf_lt f b⟩

/-! ## NaN detection (SPEC §8.2) -/

/-- `b` is any NaN of format `f`. -/
def isNaN (f : FloatFormat) (b : Nat) : Bool :=
  f.expOf b == f.expMax && !(f.mantOf b == 0)

/-- `b` is a **signaling** NaN: a NaN whose quiet bit is clear.  A signaling NaN
in `alpha`, `beta`, `A`, `B` or the entry `C` is the sole released trigger for
status `5` (SPEC §8.2). -/
def isSignalingNaN (f : FloatFormat) (b : Nat) : Bool :=
  f.isNaN b && decide (f.mantOf b < f.quietBitMask)

/-- `b` is a quiet NaN. -/
def isQuietNaN (f : FloatFormat) (b : Nat) : Bool :=
  f.isNaN b && decide (f.quietBitMask ≤ f.mantOf b)

theorem expMax_pos (f : FloatFormat) (h : 0 < f.expBits) : 0 < f.expMax := by
  have : (2 : Nat) ^ 1 ≤ 2 ^ f.expBits := Nat.pow_le_pow_right (by decide) h
  simp only [expMax]
  omega

theorem isNaN_iff_decode_nan (f : FloatFormat) (hpos : 0 < f.expBits) (b : Nat) :
    f.isNaN b = true ↔ (f.decode b).isNaN = true := by
  have hmax : 0 < f.expMax := expMax_pos f hpos
  simp only [isNaN, Bool.and_eq_true, beq_iff_eq, Bool.not_eq_true',
    beq_eq_false_iff_ne, ne_eq, decode]
  constructor
  · rintro ⟨he, hm⟩
    rw [if_neg (by omega), if_pos he, if_neg hm]
    rfl
  · intro h
    by_cases he : f.expOf b = 0
    · rw [if_pos he] at h
      split at h <;> simp [FloatClass.isNaN] at h
    · rw [if_neg he] at h
      by_cases he2 : f.expOf b = f.expMax
      · rw [if_pos he2] at h
        by_cases hm : f.mantOf b = 0
        · rw [if_pos hm] at h; simp [FloatClass.isNaN] at h
        · exact ⟨he2, hm⟩
      · rw [if_neg he2] at h; simp [FloatClass.isNaN] at h

theorem signaling_and_quiet_exclusive (f : FloatFormat) (b : Nat) :
    ¬ (f.isSignalingNaN b = true ∧ f.isQuietNaN b = true) := by
  simp only [isSignalingNaN, isQuietNaN, Bool.and_eq_true, decide_eq_true_eq]
  omega

theorem nan_is_signaling_or_quiet (f : FloatFormat) (b : Nat) (h : f.isNaN b = true) :
    f.isSignalingNaN b = true ∨ f.isQuietNaN b = true := by
  simp only [isSignalingNaN, isQuietNaN, Bool.and_eq_true, decide_eq_true_eq, h,
    true_and]
  omega

/-- The canonical quiet NaN of every released format really is a quiet NaN, and
is never signaling. -/
theorem canonicalQuietNaN_isQuietNaN_binary16 :
    binary16.isQuietNaN binary16.canonicalQuietNaN = true := by decide
theorem canonicalQuietNaN_isQuietNaN_bfloat16 :
    bfloat16.isQuietNaN bfloat16.canonicalQuietNaN = true := by decide
theorem canonicalQuietNaN_isQuietNaN_binary32 :
    binary32.isQuietNaN binary32.canonicalQuietNaN = true := by decide
theorem canonicalQuietNaN_isQuietNaN_binary64 :
    binary64.isQuietNaN binary64.canonicalQuietNaN = true := by decide

theorem canonicalQuietNaN_not_signaling_binary16 :
    binary16.isSignalingNaN binary16.canonicalQuietNaN = false := by decide
theorem canonicalQuietNaN_not_signaling_bfloat16 :
    bfloat16.isSignalingNaN bfloat16.canonicalQuietNaN = false := by decide
theorem canonicalQuietNaN_not_signaling_binary32 :
    binary32.isSignalingNaN binary32.canonicalQuietNaN = false := by decide
theorem canonicalQuietNaN_not_signaling_binary64 :
    binary64.isSignalingNaN binary64.canonicalQuietNaN = false := by decide

/-- The canonical quiet NaN is representable in its format. -/
theorem canonicalQuietNaN_lt_binary16 :
    binary16.canonicalQuietNaN < 2 ^ binary16.totalBits := by decide
theorem canonicalQuietNaN_lt_bfloat16 :
    bfloat16.canonicalQuietNaN < 2 ^ bfloat16.totalBits := by decide
theorem canonicalQuietNaN_lt_binary32 :
    binary32.canonicalQuietNaN < 2 ^ binary32.totalBits := by decide
theorem canonicalQuietNaN_lt_binary64 :
    binary64.canonicalQuietNaN < 2 ^ binary64.totalBits := by decide

/-- The canonical NaN has a clear sign bit. -/
theorem canonicalQuietNaN_signOf_binary32 :
    binary32.signOf binary32.canonicalQuietNaN = false := by decide

end FloatFormat

/-! ## Attaching formats to scalar kinds -/

namespace ScalarKind

/-- The interchange format of a floating stored kind. -/
def floatFormat? : ScalarKind → Option FloatFormat
  | binary16 => some FloatFormat.binary16
  | bfloat16 => some FloatFormat.bfloat16
  | binary32 => some FloatFormat.binary32
  | binary64 => some FloatFormat.binary64
  | _ => none

theorem floatFormat?_isSome_iff (k : ScalarKind) :
    (k.floatFormat?).isSome = true ↔ k.isFloat = true := by
  cases k <;> decide

/-- The format width agrees with the stored width of the kind. -/
theorem floatFormat?_totalBits (k : ScalarKind) (f : FloatFormat)
    (h : k.floatFormat? = some f) : f.totalBits = k.bitWidth := by
  cases k <;> simp only [floatFormat?, Option.some.injEq, reduceCtorEq] at h <;>
    subst h <;> rfl

/-- The canonical quiet NaN pattern of a floating stored kind (SPEC §8.2). -/
def canonicalQuietNaN? (k : ScalarKind) : Option Nat :=
  (k.floatFormat?).map FloatFormat.canonicalQuietNaN

theorem canonicalQuietNaN?_binary16 : binary16.canonicalQuietNaN? = some 0x7e00 := by decide
theorem canonicalQuietNaN?_bfloat16 : bfloat16.canonicalQuietNaN? = some 0x7fc0 := by decide
theorem canonicalQuietNaN?_binary32 :
    binary32.canonicalQuietNaN? = some 0x7fc00000 := by decide
theorem canonicalQuietNaN?_binary64 :
    binary64.canonicalQuietNaN? = some 0x7ff8000000000000 := by decide

/-- Signaling-NaN detection at the level of stored kinds: non-floating kinds
never signal. -/
def isSignalingNaNBits (k : ScalarKind) (bits : Nat) : Bool :=
  match k.floatFormat? with
  | some f => f.isSignalingNaN bits
  | none => false

theorem isSignalingNaNBits_of_integer (k : ScalarKind) (bits : Nat)
    (h : k.isInteger = true) : k.isSignalingNaNBits bits = false := by
  cases k <;> simp_all [isSignalingNaNBits, floatFormat?, isInteger]

theorem isSignalingNaNBits_canonicalQuietNaN (k : ScalarKind) (f : FloatFormat)
    (h : k.floatFormat? = some f) :
    k.isSignalingNaNBits f.canonicalQuietNaN = false := by
  cases k <;> simp only [floatFormat?, Option.some.injEq, reduceCtorEq] at h <;>
    subst h <;> simp only [isSignalingNaNBits, floatFormat?] <;> decide

end ScalarKind

end WasmGemmGnaf.Gemm
