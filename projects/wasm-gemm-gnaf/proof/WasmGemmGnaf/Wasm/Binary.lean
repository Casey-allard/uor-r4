/-
  Wasm/Binary.lean --- the Core 3.0 binary format: LEB128, sections,
  `Wasm.encode` and `Wasm.decode`.

  Normative source: SPEC.md section 7.3.

  ## What is proved

  The file is organised around one structure, `Wasm.Codec`, which bundles an
  encoder, a decoder, and *both* halves of the inverse law:

  * `dec_enc` --- decoding an encoded value followed by arbitrary trailing
    bytes returns exactly that value and exactly those trailing bytes;
  * `dec_sound` --- whenever the decoder succeeds, the consumed prefix *is* the
    encoding of the value it returned.

  Every combinator (pairs, lists, options, tagged sums, length-prefixed
  sections, ...) carries both laws, so the module codec is correct by
  construction.  `dec_sound` is the theorem that rules out a wrong module: a
  malformed byte string can only produce `Except.error fault`, never a module
  that does not re-encode to those very bytes.

  ## Declared scope of the byte format

  The instruction subset is the one declared in `Wasm/Syntax.lean`.  Within it,
  the serialisation is the canonical one fixed by this file:

  * the module preamble is the pinned `\0asm` magic and version `1`;
  * sections carry a one-byte identifier and a ULEB128 byte-size prefix, and
    appear in the pinned order `1, 2, 3, 4, 5, 13, 6, 7, 8, 9, 11`;
    `encode` always emits all eleven sections, and `decode` accepts exactly
    that canonical sequence;
  * section 3 carries complete function definitions (type index, locals and
    body); the pinned format's split of functions across sections 3 and 10 is
    *not* reproduced;
  * ULEB128 and SLEB128 are *canonical*: a redundant continuation byte is
    rejected with `DecodeFault.nonCanonicalLeb128`, which is what makes decoding
    injective (a permissive decoder cannot be injective, and SPEC section 7.3
    asks for `encode_decode_roundtrip`);
  * names are raw byte strings, length-prefixed; UTF-8 well-formedness is a
    validation condition, not a syntactic one;
  * instruction opcodes come from `Wasm.Opcode`, a pinned dense enumeration
    with one code per `Instr` constructor, written as ULEB128; the numeric
    operator families (`iBinOp`, `vecBinOp`, ...) then carry their width and
    operator as further tags.

  Consequently this file does **not** claim byte-level identity with the
  vendored Core 3.0 opcode table, and does **not** claim `decode_sound` /
  `decode_complete` against the vendored `Wasm.DeclarativeBinaryRelation` of
  SPEC section 7.3 --- that relation is not defined in this layer, and claiming
  it would be a fake proof.  What *is* proved is the self-contained
  inverse-pair specification above, in both directions, for the whole declared
  subset:

  * `Wasm.leb128_roundtrip`, `Wasm.decULEB_sound`, `Wasm.decSLEB_encodeSLEB`,
    `Wasm.decSLEB_sound`;
  * `Wasm.decInstr_decExpr_enc`, `Wasm.decInstr_decExpr_sound`;
  * `Wasm.encode_decode_roundtrip`, `Wasm.decode_is_encode`,
    `Wasm.encode_injective`, `Wasm.decode_error_or_encode`.

  Every declaration in this file is proved.  Nothing is assumed.
-/
import WasmGemmGnaf.Wasm.Syntax

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf.Foundation

/-! ## Typed decoding faults

Every failure path of the decoder names the reason.  There is no untyped
`none`, and no failure path that could return a module. -/

/-- A typed decoding fault. -/
inductive DecodeFault where
  /-- The input ended in the middle of a form. -/
  | unexpectedEndOfInput
  /-- A LEB128 number carried a redundant continuation byte. -/
  | nonCanonicalLeb128
  /-- A fixed-width immediate was truncated. -/
  | truncatedImmediate (width : Nat)
  /-- An opcode number that no instruction of the declared subset uses. -/
  | unknownOpcode (code : Nat)
  /-- A one-byte discriminant that no form of the expected type uses. -/
  | unknownTag (code : Nat)
  /-- The section identifier at this position was not the expected one. -/
  | badSectionId (expected : Nat) (found : Nat)
  /-- A section's declared byte size did not match its contents. -/
  | badSectionSize (id : Nat)
  /-- The module preamble was not the pinned magic. -/
  | badMagic
  /-- The module preamble did not carry version 1. -/
  | badVersion
  /-- Bytes remained after a complete form was decoded. -/
  | trailingBytes (count : Nat)
  /-- The structural fuel was exhausted; the input is more deeply nested than
  it is long, which no encoded input can be. -/
  | fuelExhausted
  deriving DecidableEq, Repr, Inhabited

/-! ## Byte helpers -/

theorem uint8_ofNat_toNat (b : UInt8) : UInt8.ofNat b.toNat = b := by
  apply UInt8.toNat_inj.mp
  have h : b.toNat < 256 := by simpa using UInt8.toNat_lt b
  simp [UInt8.toNat_ofNat', Nat.mod_eq_of_lt h]

theorem uint8_toNat_ofNat (n : Nat) : (UInt8.ofNat n).toNat = n % 256 := by
  simp [UInt8.toNat_ofNat']

theorem uint8_toNat_lt (b : UInt8) : b.toNat < 256 := by
  simpa using UInt8.toNat_lt b

/-! ## Unsigned LEB128

The encoder is `Foundation.Bytes.natBytes`: little-endian base 128 with the
continuation bit set on every byte but the last.  That is exactly canonical
unsigned LEB128, and `Foundation/Bytes.lean` already proves it prefix-free. -/

/-- Canonical unsigned LEB128 encoding. -/
def encodeULEB (n : Nat) : List UInt8 := Bytes.natBytes n

theorem encodeULEB_lt {n : Nat} (h : n < 128) : encodeULEB n = [UInt8.ofNat n] :=
  Bytes.natBytes_lt h

theorem encodeULEB_ge {n : Nat} (h : ¬ n < 128) :
    encodeULEB n = UInt8.ofNat (n % 128 + 128) :: encodeULEB (n / 128) :=
  Bytes.natBytes_ge h

theorem encodeULEB_ne_nil (n : Nat) : encodeULEB n ≠ [] := by
  by_cases h : n < 128
  · rw [encodeULEB_lt h]; simp
  · rw [encodeULEB_ge h]; simp

/-- Canonical unsigned LEB128 decoding.  A continuation byte whose tail decodes
to zero is redundant and is rejected. -/
def decULEB : List UInt8 → Except DecodeFault (Nat × List UInt8)
  | [] => .error .unexpectedEndOfInput
  | b :: rest =>
    if b.toNat < 128 then .ok (b.toNat, rest)
    else
      match decULEB rest with
      | .error e => .error e
      | .ok (m, r) =>
        if m = 0 then .error .nonCanonicalLeb128
        else .ok (b.toNat - 128 + 128 * m, r)

theorem decULEB_encodeULEB (n : Nat) (r : List UInt8) :
    decULEB (encodeULEB n ++ r) = .ok (n, r) := by
  induction n using Nat.strongRecOn with
  | _ n ih =>
    by_cases h : n < 128
    · rw [encodeULEB_lt h]
      simp only [List.singleton_append, decULEB]
      rw [uint8_toNat_ofNat, Nat.mod_eq_of_lt (by omega)]
      simp [h]
    · rw [encodeULEB_ge h]
      simp only [List.cons_append, decULEB]
      rw [uint8_toNat_ofNat, Nat.mod_eq_of_lt (by have := Nat.mod_lt n (y := 128); omega)]
      have hb : ¬ (n % 128 + 128 < 128) := by omega
      simp only [hb, if_false]
      rw [ih (n / 128) (by omega)]
      have hpos : ¬ (n / 128 = 0) := by
        have := Nat.div_add_mod n 128
        have := Nat.mod_lt n (y := 128) (by omega)
        omega
      simp only [hpos, if_false]
      have : n % 128 + 128 - 128 + 128 * (n / 128) = n := by
        have := Nat.div_add_mod n 128
        omega
      rw [this]

theorem decULEB_sound :
    ∀ (s : List UInt8) (n : Nat) (r : List UInt8),
      decULEB s = .ok (n, r) → s = encodeULEB n ++ r := by
  intro s
  induction s with
  | nil => intro n r h; simp [decULEB] at h
  | cons b rest ih =>
    intro n r h
    simp only [decULEB] at h
    by_cases hb : b.toNat < 128
    · simp only [hb, if_true, Except.ok.injEq, Prod.mk.injEq] at h
      obtain ⟨rfl, rfl⟩ := h
      rw [encodeULEB_lt hb, uint8_ofNat_toNat]
      rfl
    · simp only [hb, if_false] at h
      revert h
      cases hd : decULEB rest with
      | error e => intro h; simp at h
      | ok p =>
        obtain ⟨m, r'⟩ := p
        intro h
        simp only at h
        by_cases hm : m = 0
        · simp [hm] at h
        · simp only [hm, if_false, Except.ok.injEq, Prod.mk.injEq] at h
          obtain ⟨rfl, rfl⟩ := h
          have hrest := ih m r' hd
          have hbn : 128 ≤ b.toNat := by omega
          have hblt : b.toNat < 256 := uint8_toNat_lt b
          have hm1 : 1 ≤ m := by omega
          have hne : ¬ (b.toNat - 128 + 128 * m < 128) := by omega
          rw [encodeULEB_ge hne]
          have h1 : (b.toNat - 128 + 128 * m) % 128 = b.toNat - 128 := by
            omega
          have h2 : (b.toNat - 128 + 128 * m) / 128 = m := by
            omega
          rw [h1, h2]
          have : b.toNat - 128 + 128 = b.toNat := by omega
          rw [this, uint8_ofNat_toNat]
          simp [hrest]

/-- The unsigned LEB128 round trip in the form of SPEC section 7.3: decoding an
encoded number returns the number together with the number of bytes it
occupied. -/
def decodeULEB (bs : List UInt8) : Option (Nat × Nat) :=
  match decULEB bs with
  | .ok (n, r) => some (n, bs.length - r.length)
  | .error _ => none

/-- **LEB128 round trip.** -/
theorem leb128_roundtrip (n : Nat) :
    decodeULEB (encodeULEB n) = some (n, (encodeULEB n).length) := by
  have h : decULEB (encodeULEB n) = .ok (n, []) := by
    have := decULEB_encodeULEB n []
    simpa using this
  simp [decodeULEB, h]

/-! ## Signed LEB128

Canonical signed LEB128.  Nonnegative and negative values recurse on a natural
number, so both encoders are structurally terminating; the negative branch
encodes `i = -1 - m` by complementing the seven-bit groups of `m`, which is
exactly the two's-complement bit pattern the format prescribes. -/

/-- Signed LEB128 groups of a nonnegative value. -/
def slebNonneg (n : Nat) : List UInt8 :=
  if n < 64 then [UInt8.ofNat n]
  else UInt8.ofNat (n % 128 + 128) :: slebNonneg (n / 128)
termination_by n
decreasing_by omega

/-- Signed LEB128 groups of the negative value `-1 - m`. -/
def slebNeg (m : Nat) : List UInt8 :=
  if m < 64 then [UInt8.ofNat (127 - m)]
  else UInt8.ofNat (127 - m % 128 + 128) :: slebNeg (m / 128)
termination_by m
decreasing_by omega

theorem slebNonneg_lt {n : Nat} (h : n < 64) : slebNonneg n = [UInt8.ofNat n] := by
  rw [slebNonneg]; simp [h]

theorem slebNonneg_ge {n : Nat} (h : ¬ n < 64) :
    slebNonneg n = UInt8.ofNat (n % 128 + 128) :: slebNonneg (n / 128) := by
  rw [slebNonneg]; simp [h]

theorem slebNeg_lt {m : Nat} (h : m < 64) : slebNeg m = [UInt8.ofNat (127 - m)] := by
  rw [slebNeg]; simp [h]

theorem slebNeg_ge {m : Nat} (h : ¬ m < 64) :
    slebNeg m = UInt8.ofNat (127 - m % 128 + 128) :: slebNeg (m / 128) := by
  rw [slebNeg]; simp [h]

/-- Canonical signed LEB128 encoding. -/
def encodeSLEB (i : Int) : List UInt8 :=
  if 0 ≤ i then slebNonneg i.toNat else slebNeg (i.natAbs - 1)

/-- Canonical signed LEB128 decoding. -/
def decSLEB : List UInt8 → Except DecodeFault (Int × List UInt8)
  | [] => .error .unexpectedEndOfInput
  | b :: rest =>
    if b.toNat < 128 then
      .ok (if b.toNat < 64 then (b.toNat : Int) else (b.toNat : Int) - 128, rest)
    else
      match decSLEB rest with
      | .error e => .error e
      | .ok (v, r) =>
        if (v = 0 ∧ b.toNat - 128 < 64) ∨ (v = -1 ∧ 64 ≤ b.toNat - 128) then
          .error .nonCanonicalLeb128
        else .ok (((b.toNat - 128 : Nat) : Int) + 128 * v, r)

theorem decSLEB_slebNonneg (n : Nat) (r : List UInt8) :
    decSLEB (slebNonneg n ++ r) = .ok ((n : Int), r) := by
  induction n using Nat.strongRecOn with
  | _ n ih =>
    by_cases h : n < 64
    · rw [slebNonneg_lt h]
      simp only [List.singleton_append, decSLEB]
      rw [uint8_toNat_ofNat, Nat.mod_eq_of_lt (by omega)]
      simp [show n < 128 by omega, h]
    · rw [slebNonneg_ge h]
      simp only [List.cons_append, decSLEB]
      rw [uint8_toNat_ofNat, Nat.mod_eq_of_lt (by have := Nat.mod_lt n (y := 128); omega)]
      have hb : ¬ (n % 128 + 128 < 128) := by omega
      simp only [hb, if_false]
      rw [ih (n / 128) (by omega)]
      have hdiv : n % 128 + 128 * (n / 128) = n := by
        have := Nat.div_add_mod n 128; omega
      have hcond : ¬ ((((n / 128 : Nat) : Int) = 0 ∧ n % 128 + 128 - 128 < 64) ∨
          (((n / 128 : Nat) : Int) = -1 ∧ 64 ≤ n % 128 + 128 - 128)) := by
        rintro (⟨h1, h2⟩ | ⟨h1, _⟩)
        · have : n / 128 = 0 := by omega
          omega
        · omega
      simp only [hcond, if_false, Except.ok.injEq, Prod.mk.injEq, and_true]
      omega

theorem decSLEB_slebNeg (m : Nat) (r : List UInt8) :
    decSLEB (slebNeg m ++ r) = .ok ((-1 - (m : Int)), r) := by
  induction m using Nat.strongRecOn with
  | _ m ih =>
    by_cases h : m < 64
    · rw [slebNeg_lt h]
      simp only [List.singleton_append, decSLEB]
      rw [uint8_toNat_ofNat, Nat.mod_eq_of_lt (by omega)]
      have h1 : 127 - m < 128 := by omega
      have h2 : ¬ (127 - m < 64) := by omega
      simp only [h1, if_true, h2, if_false, Except.ok.injEq, Prod.mk.injEq, and_true]
      omega
    · rw [slebNeg_ge h]
      simp only [List.cons_append, decSLEB]
      rw [uint8_toNat_ofNat,
        Nat.mod_eq_of_lt (by have := Nat.mod_lt m (y := 128); omega)]
      have hb : ¬ (127 - m % 128 + 128 < 128) := by omega
      simp only [hb, if_false]
      rw [ih (m / 128) (by omega)]
      have hdiv : m % 128 + 128 * (m / 128) = m := by
        have := Nat.div_add_mod m 128; omega
      have hcond : ¬ ((-1 - ((m / 128 : Nat) : Int) = 0 ∧
            127 - m % 128 + 128 - 128 < 64) ∨
          (-1 - ((m / 128 : Nat) : Int) = -1 ∧ 64 ≤ 127 - m % 128 + 128 - 128)) := by
        rintro (⟨h1, _⟩ | ⟨h1, h2⟩)
        · omega
        · have : m / 128 = 0 := by omega
          omega
      simp only [hcond, if_false, Except.ok.injEq, Prod.mk.injEq, and_true]
      omega

theorem decSLEB_encodeSLEB (i : Int) (r : List UInt8) :
    decSLEB (encodeSLEB i ++ r) = .ok (i, r) := by
  unfold encodeSLEB
  by_cases h : 0 ≤ i
  · simp only [h, if_true]
    have hi : ((i.toNat : Nat) : Int) = i := by omega
    rw [decSLEB_slebNonneg, hi]
  · simp only [h, if_false]
    have hlt : i < 0 := by omega
    have hi : (-1 - ((i.natAbs - 1 : Nat) : Int)) = i := by omega
    rw [decSLEB_slebNeg, hi]

theorem decSLEB_sound :
    ∀ (s : List UInt8) (v : Int) (r : List UInt8),
      decSLEB s = .ok (v, r) → s = encodeSLEB v ++ r := by
  intro s
  induction s with
  | nil => intro v r h; simp [decSLEB] at h
  | cons b rest ih =>
    intro v r h
    have hblt : b.toNat < 256 := uint8_toNat_lt b
    simp only [decSLEB] at h
    by_cases hb : b.toNat < 128
    · simp only [hb, if_true, Except.ok.injEq, Prod.mk.injEq] at h
      obtain ⟨hv, rfl⟩ := h
      by_cases h64 : b.toNat < 64
      · simp only [h64, if_true] at hv
        subst hv
        have hnn : (0 : Int) ≤ (b.toNat : Int) := by omega
        unfold encodeSLEB
        simp only [hnn, if_true]
        have : (b.toNat : Int).toNat = b.toNat := by omega
        rw [this, slebNonneg_lt h64, uint8_ofNat_toNat]
        rfl
      · simp only [h64, if_false] at hv
        subst hv
        have hneg : ¬ (0 : Int) ≤ (b.toNat : Int) - 128 := by omega
        unfold encodeSLEB
        simp only [hneg, if_false]
        have hna : ((b.toNat : Int) - 128).natAbs - 1 = 127 - b.toNat := by omega
        rw [hna, slebNeg_lt (by omega)]
        have : 127 - (127 - b.toNat) = b.toNat := by omega
        rw [this, uint8_ofNat_toNat]
        rfl
    · simp only [hb, if_false] at h
      revert h
      cases hd : decSLEB rest with
      | error e => intro h; simp at h
      | ok p =>
        obtain ⟨w, r'⟩ := p
        intro h
        simp only at h
        by_cases hc : (w = 0 ∧ b.toNat - 128 < 64) ∨ (w = -1 ∧ 64 ≤ b.toNat - 128)
        · simp [hc] at h
        · simp only [hc, if_false, Except.ok.injEq, Prod.mk.injEq] at h
          obtain ⟨hv, rfl⟩ := h
          have hrest := ih w r' hd
          have hlow : b.toNat - 128 < 128 := by omega
          have hb128 : 128 ≤ b.toNat := by omega
          have hc1 : ¬ (w = 0 ∧ b.toNat - 128 < 64) := fun x => hc (Or.inl x)
          have hc2 : ¬ (w = -1 ∧ 64 ≤ b.toNat - 128) := fun x => hc (Or.inr x)
          have hkey1 : w = 0 → 64 ≤ b.toNat - 128 := by
            intro hz
            rcases Nat.lt_or_ge (b.toNat - 128) 64 with hlt | hge
            · exact absurd ⟨hz, hlt⟩ hc1
            · exact hge
          have hkey2 : w = -1 → b.toNat - 128 < 64 := by
            intro hz
            rcases Nat.lt_or_ge (b.toNat - 128) 64 with hlt | hge
            · exact hlt
            · exact absurd ⟨hz, hge⟩ hc2
          subst hv
          by_cases hw : 0 ≤ w
          · -- nonnegative tail
            have hnn : (0 : Int) ≤ ((b.toNat - 128 : Nat) : Int) + 128 * w := by omega
            unfold encodeSLEB
            simp only [hnn, if_true]
            have hwenc : encodeSLEB w = slebNonneg w.toNat := by
              unfold encodeSLEB; simp [hw]
            have hge : ¬ (((b.toNat - 128 : Nat) : Int) + 128 * w).toNat < 64 := by
              by_cases hw0 : w = 0
              · have h64 : 64 ≤ b.toNat - 128 := hkey1 hw0
                omega
              · omega
            rw [slebNonneg_ge hge]
            have hmod : (((b.toNat - 128 : Nat) : Int) + 128 * w).toNat % 128
                = b.toNat - 128 := by omega
            have hdiv : (((b.toNat - 128 : Nat) : Int) + 128 * w).toNat / 128
                = w.toNat := by omega
            rw [hmod, hdiv, show b.toNat - 128 + 128 = b.toNat by omega,
              uint8_ofNat_toNat, ← hwenc, hrest]
            rfl
          · -- negative tail
            have hwlt : w < 0 := by omega
            have hneg : ¬ (0 : Int) ≤ ((b.toNat - 128 : Nat) : Int) + 128 * w := by omega
            unfold encodeSLEB
            simp only [hneg, if_false]
            have hwenc : encodeSLEB w = slebNeg (w.natAbs - 1) := by
              unfold encodeSLEB; simp [hw]
            have hge : ¬ (((b.toNat - 128 : Nat) : Int) + 128 * w).natAbs - 1 < 64 := by
              by_cases hw1 : w = -1
              · have h64 : b.toNat - 128 < 64 := hkey2 hw1
                omega
              · omega
            rw [slebNeg_ge hge]
            have hdiv : ((((b.toNat - 128 : Nat) : Int) + 128 * w).natAbs - 1) / 128
                = w.natAbs - 1 := by omega
            rw [show ((((b.toNat - 128 : Nat) : Int) + 128 * w).natAbs - 1) % 128
                = 127 - (b.toNat - 128) by omega, hdiv,
              show 127 - (127 - (b.toNat - 128)) + 128 = b.toNat by omega,
              uint8_ofNat_toNat, ← hwenc, hrest]
            rfl

/-! ## Fixed-width little-endian immediates

`f32.const` and `f64.const` carry raw four- and eight-byte little-endian bit
patterns.  Both directions of the inverse are proved. -/

/-- The `k` low-order little-endian bytes of `n`. -/
def bytesLE : Nat → Nat → List UInt8
  | 0, _ => []
  | k + 1, n => UInt8.ofNat (n % 256) :: bytesLE k (n / 256)

/-- The value of a little-endian byte string. -/
def natLE : List UInt8 → Nat
  | [] => 0
  | b :: bs => b.toNat + 256 * natLE bs

@[simp] theorem length_bytesLE (k n : Nat) : (bytesLE k n).length = k := by
  induction k generalizing n with
  | zero => rfl
  | succ k ih => simp [bytesLE, ih]

theorem natLE_bytesLE_of_lt : ∀ (k n : Nat), n < 256 ^ k → natLE (bytesLE k n) = n := by
  intro k
  induction k with
  | zero => intro n h; simp [bytesLE, natLE]; omega
  | succ k ih =>
    intro n h
    have hpow : (256 : Nat) ^ (k + 1) = 256 ^ k * 256 := by rw [Nat.pow_succ]
    rw [hpow] at h
    have hlt : n / 256 < 256 ^ k := by omega
    simp only [bytesLE, natLE, ih (n / 256) hlt, uint8_toNat_ofNat]
    omega

theorem natLE_lt (bs : List UInt8) : natLE bs < 256 ^ bs.length := by
  induction bs with
  | nil => simp [natLE]
  | cons b bs ih =>
    have hb : b.toNat < 256 := uint8_toNat_lt b
    simp only [natLE, List.length_cons, Nat.pow_succ]
    have : 256 * natLE bs + 256 ≤ 256 * 256 ^ bs.length := by
      have : natLE bs + 1 ≤ 256 ^ bs.length := ih
      omega
    omega

theorem bytesLE_natLE : ∀ bs : List UInt8, bytesLE bs.length (natLE bs) = bs
  | [] => rfl
  | b :: bs => by
    have hb : b.toNat < 256 := uint8_toNat_lt b
    show UInt8.ofNat ((b.toNat + 256 * natLE bs) % 256)
        :: bytesLE bs.length ((b.toNat + 256 * natLE bs) / 256) = b :: bs
    rw [show (b.toNat + 256 * natLE bs) % 256 = b.toNat by omega,
      show (b.toNat + 256 * natLE bs) / 256 = natLE bs by omega,
      uint8_ofNat_toNat, bytesLE_natLE bs]

/-! ## Codecs

A `Codec α` is an encoder together with a decoder and *both* halves of the
inverse law.  `fuel` is a structural budget: it never affects which byte
strings are accepted as long as it is at least the value's `cost`, and `cost`
is bounded by the encoding length (see `Instr.cost_le_length`), so passing the
input length as fuel is always enough. -/

/-- A bidirectionally verified binary codec. -/
structure Codec (α : Type) where
  /-- The encoder. -/
  enc : α → List UInt8
  /-- The decoder: consumes a prefix of the input and returns the remainder. -/
  dec : Nat → List UInt8 → Except DecodeFault (α × List UInt8)
  /-- The structural fuel a value needs in order to be decodable. -/
  cost : α → Nat
  /-- Decoding an encoding returns the value and the untouched remainder. -/
  dec_enc : ∀ (a : α) (fuel : Nat) (r : List UInt8),
    cost a ≤ fuel → dec fuel (enc a ++ r) = .ok (a, r)
  /-- Whenever the decoder succeeds, the bytes it consumed are exactly the
  encoding of the value it produced. -/
  dec_sound : ∀ (fuel : Nat) (s : List UInt8) (a : α) (r : List UInt8),
    dec fuel s = .ok (a, r) → s = enc a ++ r
  /-- The fuel a value needs never exceeds the length of its encoding, so
  passing the input length as fuel is always enough. -/
  cost_le : ∀ a : α, cost a ≤ (enc a).length

namespace Codec

variable {α β : Type}

/-- Encoders of a codec are injective. -/
theorem enc_injective (c : Codec α) : Function.Injective c.enc := by
  intro a b h
  have h1 := c.dec_enc a (c.cost a + c.cost b) [] (by omega)
  have h2 := c.dec_enc b (c.cost a + c.cost b) [] (by omega)
  rw [h] at h1
  rw [h1] at h2
  simpa using h2

/-- Encoders of a codec are prefix-free in the sense of SPEC section 6.2. -/
theorem enc_prefixFree (c : Codec α) : Bytes.PrefixFree c.enc := by
  intro a b r s h
  have e1 := c.dec_enc a (c.cost a + c.cost b) r (by omega)
  have e2 := c.dec_enc b (c.cost a + c.cost b) s (by omega)
  rw [h, e2] at e1
  simp only [Except.ok.injEq, Prod.mk.injEq] at e1
  exact ⟨e1.1.symm, e1.2.symm⟩

/-- Decoding is deterministic: two successful decodes of the same input agree,
whatever fuel each was given. -/
theorem dec_deterministic (c : Codec α) {fuel fuel' : Nat} {s : List UInt8}
    {a b : α} {r r' : List UInt8}
    (ha : c.dec fuel s = .ok (a, r)) (hb : c.dec fuel' s = .ok (b, r')) :
    a = b ∧ r = r' := by
  have h1 := c.dec_sound _ _ _ _ ha
  have h2 := c.dec_sound _ _ _ _ hb
  have heq : c.enc a ++ r = c.enc b ++ r' := by rw [← h1, ← h2]
  exact c.enc_prefixFree _ _ _ _ heq

end Codec

/-! ## Generic list utilities used by the codec combinators -/

theorem takeDrop_of_length {α : Type} (l r : List α) (k : Nat) (h : l.length = k) :
    (l ++ r).take k = l ∧ (l ++ r).drop k = r := by
  subst h; simp

theorem nodup_map_unique {α β : Type} :
    ∀ (l : List α) (f : α → β), (l.map f).Nodup →
      ∀ a ∈ l, ∀ b ∈ l, f a = f b → a = b := by
  intro l
  induction l with
  | nil => intro f _ a ha; exact absurd ha (by simp)
  | cons x xs ih =>
    intro f hnd a ha b hb hfab
    simp only [List.map_cons, List.nodup_cons] at hnd
    obtain ⟨hx, hxs⟩ := hnd
    simp only [List.mem_cons] at ha hb
    rcases ha with rfl | ha
    · rcases hb with rfl | hb
      · rfl
      · exact absurd (hfab ▸ List.mem_map_of_mem hb) hx
    · rcases hb with rfl | hb
      · exact absurd (hfab.symm ▸ List.mem_map_of_mem ha) hx
      · exact ih f hxs a ha b hb hfab

theorem find?_eq_some_of_unique {α : Type} (p : α → Bool) :
    ∀ (l : List α) (a : α), a ∈ l → p a = true →
      (∀ b ∈ l, p b = true → b = a) → l.find? p = some a := by
  intro l
  induction l with
  | nil => intro a ha; exact absurd ha (by simp)
  | cons x xs ih =>
    intro a ha hp huniq
    rw [List.find?_cons]
    by_cases hx : p x = true
    · simp only [hx, if_true]
      rw [huniq x (by simp) hx]
    · simp only [hx, if_false]
      have hne : a ≠ x := by intro h; exact hx (h ▸ hp)
      have ha' : a ∈ xs := by
        simp only [List.mem_cons] at ha
        rcases ha with rfl | ha
        · exact absurd rfl hne
        · exact ha
      exact ih a ha' hp (fun b hb => huniq b (by simp [hb]))

/-! ## Prefix stripping -/

/-- Strip a literal byte prefix. -/
def stripPrefix : List UInt8 → List UInt8 → Option (List UInt8)
  | [], s => some s
  | _ :: _, [] => none
  | b :: bs, x :: xs => if b = x then stripPrefix bs xs else none

theorem stripPrefix_append : ∀ (p r : List UInt8), stripPrefix p (p ++ r) = some r
  | [], r => rfl
  | b :: bs, r => by
    show (if b = b then stripPrefix bs (bs ++ r) else none) = some r
    simp [stripPrefix_append bs r]

theorem stripPrefix_sound :
    ∀ (p s r : List UInt8), stripPrefix p s = some r → s = p ++ r := by
  intro p
  induction p with
  | nil => intro s r h; simpa [stripPrefix] using h
  | cons b bs ih =>
    intro s r h
    cases s with
    | nil => simp [stripPrefix] at h
    | cons x xs =>
      simp only [stripPrefix] at h
      by_cases hbx : b = x
      · subst hbx
        simp only [if_pos rfl] at h
        rw [ih xs r h]; rfl
      · simp [hbx] at h

/-! ## Codec combinators -/

namespace Codec

variable {α β : Type}

/-- Transport a codec along a bijection. -/
def iso (c : Codec β) (f : α → β) (g : β → α)
    (hgf : ∀ a, g (f a) = a) (hfg : ∀ b, f (g b) = b) : Codec α where
  enc a := c.enc (f a)
  dec fuel s :=
    match c.dec fuel s with
    | .error e => .error e
    | .ok (b, r) => .ok (g b, r)
  cost a := c.cost (f a)
  dec_enc := by
    intro a fuel r h
    simp only [c.dec_enc (f a) fuel r h, hgf]
  dec_sound := by
    intro fuel s a r h
    revert h
    cases hd : c.dec fuel s with
    | error e => intro h; simp at h
    | ok p =>
      obtain ⟨b, r'⟩ := p
      intro h
      simp only [Except.ok.injEq, Prod.mk.injEq] at h
      obtain ⟨rfl, rfl⟩ := h
      rw [hfg]
      exact c.dec_sound _ _ _ _ hd
  cost_le a := c.cost_le (f a)

/-- Prefix a codec with a literal byte string. -/
def prefixed (p : List UInt8) (fault : DecodeFault) (c : Codec α) : Codec α where
  enc a := p ++ c.enc a
  dec fuel s :=
    match stripPrefix p s with
    | none => .error fault
    | some t => c.dec fuel t
  cost := c.cost
  dec_enc := by
    intro a fuel r h
    simp only [List.append_assoc, stripPrefix_append, c.dec_enc a fuel r h]
  dec_sound := by
    intro fuel s a r h
    revert h
    cases hp : stripPrefix p s with
    | none => intro h; simp at h
    | some t =>
      intro h
      rw [stripPrefix_sound p s t hp, c.dec_sound _ _ _ _ h, List.append_assoc]
  cost_le a := by
    have := c.cost_le a
    simp only [List.length_append]
    omega

/-- Concatenated codec of a pair. -/
def pair (c : Codec α) (d : Codec β) : Codec (α × β) where
  enc x := c.enc x.1 ++ d.enc x.2
  dec fuel s :=
    match c.dec fuel s with
    | .error e => .error e
    | .ok (a, r) =>
      match d.dec fuel r with
      | .error e => .error e
      | .ok (b, r') => .ok ((a, b), r')
  cost x := c.cost x.1 + d.cost x.2
  dec_enc := by
    intro x fuel r h
    obtain ⟨a, b⟩ := x
    simp only [List.append_assoc]
    simp only [c.dec_enc a fuel _ (by simp only [] at h ⊢; omega),
      d.dec_enc b fuel r (by simp only [] at h ⊢; omega)]
  dec_sound := by
    intro fuel s x r h
    revert h
    cases hc : c.dec fuel s with
    | error e => intro h; simp at h
    | ok p =>
      obtain ⟨a, t⟩ := p
      cases hd : d.dec fuel t with
      | error e => intro h; simp [hd] at h
      | ok q =>
        obtain ⟨b, t'⟩ := q
        intro h
        simp only [hd, Except.ok.injEq, Prod.mk.injEq] at h
        obtain ⟨rfl, rfl⟩ := h
        rw [c.dec_sound _ _ _ _ hc, d.dec_sound _ _ _ _ hd, List.append_assoc]
  cost_le x := by
    have h1 := c.cost_le x.1
    have h2 := d.cost_le x.2
    simp only [List.length_append]
    omega

/-- Tagged codec of a sum. -/
def sumCodec (c : Codec α) (d : Codec β) : Codec (α ⊕ β) where
  enc x := Bytes.sumBytes c.enc d.enc x
  dec fuel s :=
    match s with
    | [] => .error .unexpectedEndOfInput
    | b :: r =>
      if b = 0 then
        match c.dec fuel r with
        | .error e => .error e
        | .ok (a, r') => .ok (.inl a, r')
      else if b = 1 then
        match d.dec fuel r with
        | .error e => .error e
        | .ok (a, r') => .ok (.inr a, r')
      else .error (.unknownTag b.toNat)
  cost x := match x with | .inl a => c.cost a | .inr b => d.cost b
  dec_enc := by
    intro x fuel r h
    cases x with
    | inl a => simp [Bytes.sumBytes, c.dec_enc a fuel r h]
    | inr b => simp [Bytes.sumBytes, d.dec_enc b fuel r h]
  dec_sound := by
    intro fuel s x r h
    cases s with
    | nil => simp [Bytes.sumBytes] at h
    | cons b t =>
      simp only at h
      by_cases hb0 : b = 0
      · subst hb0
        simp only [if_pos rfl] at h
        revert h
        cases hc : c.dec fuel t with
        | error e => intro h; simp at h
        | ok q =>
          obtain ⟨a, r'⟩ := q
          intro h
          simp only [Except.ok.injEq, Prod.mk.injEq] at h
          obtain ⟨rfl, rfl⟩ := h
          simp [Bytes.sumBytes, c.dec_sound _ _ _ _ hc]
      · simp only [hb0, if_false] at h
        by_cases hb1 : b = 1
        · subst hb1
          simp only [if_pos rfl] at h
          revert h
          cases hd : d.dec fuel t with
          | error e => intro h; simp at h
          | ok q =>
            obtain ⟨a, r'⟩ := q
            intro h
            simp only [Except.ok.injEq, Prod.mk.injEq] at h
            obtain ⟨rfl, rfl⟩ := h
            simp [Bytes.sumBytes, d.dec_sound _ _ _ _ hd]
        · simp [hb1] at h
  cost_le x := by
    cases x with
    | inl a =>
      have := c.cost_le a
      simp only [Bytes.sumBytes, List.length_cons]
      omega
    | inr b =>
      have := d.cost_le b
      simp only [Bytes.sumBytes, List.length_cons]
      omega

/-- The codec of the unit type: no bytes at all. -/
def unitCodec : Codec Unit where
  enc _ := []
  dec _ s := .ok ((), s)
  cost _ := 0
  dec_enc := by intro a _ r _; cases a; rfl
  dec_sound := by
    intro _ s a r h
    simp only [Except.ok.injEq, Prod.mk.injEq] at h
    obtain ⟨_, rfl⟩ := h
    rfl
  cost_le _ := Nat.zero_le _

/-- One-byte boolean codec. -/
def boolCodec : Codec Bool where
  enc := Bytes.boolBytes
  dec _ s :=
    match s with
    | [] => .error .unexpectedEndOfInput
    | b :: r =>
      if b = 0 then .ok (false, r)
      else if b = 1 then .ok (true, r)
      else .error (.unknownTag b.toNat)
  cost _ := 0
  dec_enc := by intro a _ r _; cases a <;> simp [Bytes.boolBytes]
  dec_sound := by
    intro _ s a r h
    cases s with
    | nil => simp at h
    | cons b t =>
      simp only at h
      by_cases hb0 : b = 0
      · subst hb0
        simp only [if_pos rfl, Except.ok.injEq, Prod.mk.injEq] at h
        obtain ⟨rfl, rfl⟩ := h
        rfl
      · simp only [hb0, if_false] at h
        by_cases hb1 : b = 1
        · subst hb1
          simp only [if_pos rfl, Except.ok.injEq, Prod.mk.injEq] at h
          obtain ⟨rfl, rfl⟩ := h
          rfl
        · simp [hb1] at h
  cost_le _ := Nat.zero_le _

/-! ### Numbers, booleans and byte strings -/

/-- Canonical unsigned LEB128 codec. -/
def natCodec : Codec Nat where
  enc := encodeULEB
  dec _ s := decULEB s
  cost _ := 0
  dec_enc := by intro a _ r _; exact decULEB_encodeULEB a r
  dec_sound := by intro _ s a r h; exact decULEB_sound s a r h
  cost_le _ := Nat.zero_le _

/-- Canonical signed LEB128 codec. -/
def intCodec : Codec Int where
  enc := encodeSLEB
  dec _ s := decSLEB s
  cost _ := 0
  dec_enc := by intro a _ r _; exact decSLEB_encodeSLEB a r
  dec_sound := by intro _ s a r h; exact decSLEB_sound s a r h
  cost_le _ := Nat.zero_le _

/-- Length-prefixed raw byte string codec. -/
def bytesCodec : Codec (List UInt8) where
  enc l := encodeULEB l.length ++ l
  dec _ s :=
    match decULEB s with
    | .error e => .error e
    | .ok (n, t) =>
      if n ≤ t.length then .ok (t.take n, t.drop n)
      else .error .unexpectedEndOfInput
  cost _ := 0
  dec_enc := by
    intro a _ r _
    rw [List.append_assoc, decULEB_encodeULEB]
    have := takeDrop_of_length a r a.length rfl
    simp [this.1, this.2]
  dec_sound := by
    intro _ s a r h
    revert h
    cases hd : decULEB s with
    | error e => intro h; simp at h
    | ok p =>
      obtain ⟨n, t⟩ := p
      intro h
      simp only at h
      by_cases hn : n ≤ t.length
      · simp only [hn, if_true, Except.ok.injEq, Prod.mk.injEq] at h
        obtain ⟨rfl, rfl⟩ := h
        rw [decULEB_sound s n t hd, List.length_take, Nat.min_eq_left hn,
          List.append_assoc, List.take_append_drop]
      · simp [hn] at h
  cost_le _ := Nat.zero_le _

/-- Four-byte little-endian `UInt32` codec. -/
def uint32Codec : Codec UInt32 where
  enc x := bytesLE 4 x.toNat
  dec _ s :=
    if 4 ≤ s.length then .ok (UInt32.ofNat (natLE (s.take 4)), s.drop 4)
    else .error (.truncatedImmediate 4)
  cost _ := 0
  dec_enc := by
    intro a _ r _
    have hlen : (bytesLE 4 a.toNat).length = 4 := length_bytesLE 4 a.toNat
    have hd := takeDrop_of_length (bytesLE 4 a.toNat) r 4 hlen
    have hge : 4 ≤ (bytesLE 4 a.toNat ++ r).length := by simp [hlen]
    simp only [hge, if_true, hd.1, hd.2]
    have hlt : a.toNat < 256 ^ 4 := by
      have h0 := UInt32.toNat_lt a; simp at h0 ⊢; omega
    rw [natLE_bytesLE_of_lt 4 a.toNat hlt]
    simp
  dec_sound := by
    intro _ s a r h
    by_cases hs : 4 ≤ s.length
    · simp only [hs, if_true, Except.ok.injEq, Prod.mk.injEq] at h
      obtain ⟨rfl, rfl⟩ := h
      have hlen : (s.take 4).length = 4 := by simp; omega
      have hlt : natLE (s.take 4) < 2 ^ 32 := by
        have h0 := natLE_lt (s.take 4); rw [hlen] at h0; simpa using h0
      have htn : (UInt32.ofNat (natLE (s.take 4))).toNat = natLE (s.take 4) := by
        rw [UInt32.toNat_ofNat']
        exact Nat.mod_eq_of_lt hlt
      have hb : bytesLE 4 (natLE (s.take 4)) = s.take 4 := by
        have h1 := bytesLE_natLE (s.take 4)
        rwa [hlen] at h1
      rw [htn, hb, List.take_append_drop]
    · simp [hs] at h
  cost_le _ := Nat.zero_le _

/-- Eight-byte little-endian `UInt64` codec. -/
def uint64Codec : Codec UInt64 where
  enc x := bytesLE 8 x.toNat
  dec _ s :=
    if 8 ≤ s.length then .ok (UInt64.ofNat (natLE (s.take 8)), s.drop 8)
    else .error (.truncatedImmediate 8)
  cost _ := 0
  dec_enc := by
    intro a _ r _
    have hlen : (bytesLE 8 a.toNat).length = 8 := length_bytesLE 8 a.toNat
    have hd := takeDrop_of_length (bytesLE 8 a.toNat) r 8 hlen
    have hge : 8 ≤ (bytesLE 8 a.toNat ++ r).length := by simp [hlen]
    simp only [hge, if_true, hd.1, hd.2]
    have hlt : a.toNat < 256 ^ 8 := by
      have h0 := UInt64.toNat_lt a; simp at h0 ⊢; omega
    rw [natLE_bytesLE_of_lt 8 a.toNat hlt]
    simp
  dec_sound := by
    intro _ s a r h
    by_cases hs : 8 ≤ s.length
    · simp only [hs, if_true, Except.ok.injEq, Prod.mk.injEq] at h
      obtain ⟨rfl, rfl⟩ := h
      have hlen : (s.take 8).length = 8 := by simp; omega
      have hlt : natLE (s.take 8) < 2 ^ 64 := by
        have h0 := natLE_lt (s.take 8); rw [hlen] at h0; simpa using h0
      have htn : (UInt64.ofNat (natLE (s.take 8))).toNat = natLE (s.take 8) := by
        rw [UInt64.toNat_ofNat']
        exact Nat.mod_eq_of_lt hlt
      have hb : bytesLE 8 (natLE (s.take 8)) = s.take 8 := by
        have h1 := bytesLE_natLE (s.take 8)
        rwa [hlen] at h1
      rw [htn, hb, List.take_append_drop]
    · simp [hs] at h
  cost_le _ := Nat.zero_le _

/-! ### Options and lists -/

/-- Tagged optional codec. -/
def optionCodec (c : Codec α) : Codec (Option α) where
  enc x := Bytes.optionBytes c.enc x
  dec fuel s :=
    match s with
    | [] => .error .unexpectedEndOfInput
    | b :: r =>
      if b = 0 then .ok (none, r)
      else if b = 1 then
        match c.dec fuel r with
        | .error e => .error e
        | .ok (a, r') => .ok (some a, r')
      else .error (.unknownTag b.toNat)
  cost x := match x with | none => 0 | some a => c.cost a
  dec_enc := by
    intro a fuel r h
    cases a with
    | none => rfl
    | some a => simp [Bytes.optionBytes, c.dec_enc a fuel r h]
  dec_sound := by
    intro fuel s a r h
    cases s with
    | nil => simp [Bytes.optionBytes] at h
    | cons b t =>
      simp only at h
      by_cases hb0 : b = 0
      · subst hb0
        simp only [if_pos rfl, Except.ok.injEq, Prod.mk.injEq] at h
        obtain ⟨rfl, rfl⟩ := h
        rfl
      · simp only [hb0, if_false] at h
        by_cases hb1 : b = 1
        · subst hb1
          simp only [if_pos rfl] at h
          revert h
          cases hc : c.dec fuel t with
          | error e => intro h; simp at h
          | ok p =>
            obtain ⟨x, r'⟩ := p
            intro h
            simp only [Except.ok.injEq, Prod.mk.injEq] at h
            obtain ⟨rfl, rfl⟩ := h
            simp [Bytes.optionBytes, c.dec_sound _ _ _ _ hc]
        · simp [hb1] at h
  cost_le a := by cases a with
    | none => exact Nat.zero_le _
    | some a =>
      have := c.cost_le a
      simp only [Bytes.optionBytes, List.length_cons]
      omega

/-- Decode exactly `n` consecutive elements. -/
def decNTimes (c : Codec α) (fuel : Nat) :
    Nat → List UInt8 → Except DecodeFault (List α × List UInt8)
  | 0, s => .ok ([], s)
  | n + 1, s =>
    match c.dec fuel s with
    | .error e => .error e
    | .ok (a, r) =>
      match decNTimes c fuel n r with
      | .error e => .error e
      | .ok (as, r') => .ok (a :: as, r')

/-- Total structural fuel of a list of values. -/
def costList (c : Codec α) : List α → Nat
  | [] => 0
  | a :: as => c.cost a + costList c as

theorem decNTimes_concat (c : Codec α) (fuel : Nat) :
    ∀ (l : List α) (r : List UInt8), costList c l ≤ fuel →
      decNTimes c fuel l.length (Bytes.concatBytes c.enc l ++ r) = .ok (l, r) := by
  intro l
  induction l with
  | nil => intro r _; rfl
  | cons a as ih =>
    intro r h
    simp only [costList] at h
    show decNTimes c fuel (as.length + 1) _ = _
    simp only [Bytes.concatBytes, List.append_assoc, decNTimes,
      c.dec_enc a fuel _ (by omega), ih r (by omega)]

theorem decNTimes_sound (c : Codec α) (fuel : Nat) :
    ∀ (n : Nat) (s : List UInt8) (l : List α) (r : List UInt8),
      decNTimes c fuel n s = .ok (l, r) →
        s = Bytes.concatBytes c.enc l ++ r ∧ l.length = n := by
  intro n
  induction n with
  | zero =>
    intro s l r h
    simp only [decNTimes, Except.ok.injEq, Prod.mk.injEq] at h
    obtain ⟨rfl, rfl⟩ := h
    exact ⟨rfl, rfl⟩
  | succ n ih =>
    intro s l r h
    simp only [decNTimes] at h
    revert h
    cases hc : c.dec fuel s with
    | error e => intro h; simp at h
    | ok p =>
      obtain ⟨a, t⟩ := p
      cases hn : decNTimes c fuel n t with
      | error e => intro h; simp [hn] at h
      | ok q =>
        obtain ⟨as, t'⟩ := q
        intro h
        simp only [hn, Except.ok.injEq, Prod.mk.injEq] at h
        obtain ⟨rfl, rfl⟩ := h
        obtain ⟨ht, hlen⟩ := ih t as t' hn
        refine ⟨?_, by simp [hlen]⟩
        rw [c.dec_sound _ _ _ _ hc, ht]
        simp [Bytes.concatBytes, List.append_assoc]

theorem costList_le_concat (c : Codec α) :
    ∀ l : List α, costList c l ≤ (Bytes.concatBytes c.enc l).length := by
  intro l
  induction l with
  | nil => exact Nat.zero_le _
  | cons a as ih =>
    have h := c.cost_le a
    simp only [costList, Bytes.concatBytes, List.length_append]
    omega

/-- Count-prefixed list codec. -/
def listCodec (c : Codec α) : Codec (List α) where
  enc l := Bytes.listBytes c.enc l
  dec fuel s :=
    match decULEB s with
    | .error e => .error e
    | .ok (n, t) => decNTimes c fuel n t
  cost := costList c
  dec_enc := by
    intro l fuel r h
    have h1 : Bytes.listBytes c.enc l
        = encodeULEB l.length ++ Bytes.concatBytes c.enc l := rfl
    rw [h1, List.append_assoc]
    simp only [decULEB_encodeULEB, decNTimes_concat c fuel l r h]
  dec_sound := by
    intro fuel s l r h
    revert h
    cases hd : decULEB s with
    | error e => intro h; simp at h
    | ok p =>
      obtain ⟨n, t⟩ := p
      intro h
      obtain ⟨ht, hlen⟩ := decNTimes_sound c fuel n t l r h
      rw [decULEB_sound s n t hd, ht, ← hlen]
      simp [Bytes.listBytes, encodeULEB, List.append_assoc]
  cost_le l := by
    have h := costList_le_concat c l
    simp only [Bytes.listBytes, List.length_append]
    omega

/-! ### Tagged finite types -/

/-- A finite type together with an injective numeric tag. -/
structure Tagged (α : Type) where
  /-- The numeric tag. -/
  code : α → Nat
  /-- Every value of the type. -/
  all : List α
  /-- The list is complete. -/
  complete : ∀ a, a ∈ all
  /-- Distinct values carry distinct tags. -/
  codes_nodup : (all.map code).Nodup

/-- The codec of a tagged finite type: the ULEB128 tag, and nothing else. -/
def tagged [DecidableEq α] (t : Tagged α) : Codec α where
  enc a := encodeULEB (t.code a)
  dec _ s :=
    match decULEB s with
    | .error e => .error e
    | .ok (n, r) =>
      match t.all.find? (fun a => decide (t.code a = n)) with
      | some a => .ok (a, r)
      | none => .error (.unknownOpcode n)
  cost _ := 0
  dec_enc := by
    intro a _ r _
    rw [decULEB_encodeULEB]
    have hfind : t.all.find? (fun b => decide (t.code b = t.code a)) = some a := by
      refine find?_eq_some_of_unique _ t.all a (t.complete a) (by simp) ?_
      intro b hb hbp
      exact nodup_map_unique t.all t.code t.codes_nodup b hb a (t.complete a)
        (of_decide_eq_true hbp)
    simp only [hfind]
  dec_sound := by
    intro _ s a r h
    revert h
    cases hd : decULEB s with
    | error e => intro h; simp at h
    | ok p =>
      obtain ⟨n, t'⟩ := p
      cases hf : t.all.find? (fun b => decide (t.code b = n)) with
      | none => intro h; simp [hf] at h
      | some b =>
        intro h
        have hcode : t.code b = n := by
          have := List.find?_some hf
          simpa using this
        simp only [hf, Except.ok.injEq, Prod.mk.injEq] at h
        obtain ⟨rfl, rfl⟩ := h
        rw [decULEB_sound s n t' hd, hcode]
  cost_le _ := Nat.zero_le _

/-! ### Sized (length-prefixed) codecs and sections -/

/-- Length-prefix a codec, and require its contents to consume the whole
declared extent. -/
def sized (c : Codec α) : Codec α where
  enc a := encodeULEB (c.enc a).length ++ c.enc a
  dec fuel s :=
    match decULEB s with
    | .error e => .error e
    | .ok (n, t) =>
      if n ≤ t.length then
        match c.dec fuel (t.take n) with
        | .error e => .error e
        | .ok (a, rem) =>
          if rem = [] then .ok (a, t.drop n) else .error (.badSectionSize n)
      else .error (.badSectionSize n)
  cost := c.cost
  dec_enc := by
    intro a fuel r h
    rw [List.append_assoc, decULEB_encodeULEB]
    have hd := takeDrop_of_length (c.enc a) r (c.enc a).length rfl
    have hle : (c.enc a).length ≤ (c.enc a ++ r).length := by simp
    simp only [hle, if_true, hd.1, hd.2]
    have : c.dec fuel (c.enc a) = .ok (a, []) := by
      have := c.dec_enc a fuel [] h
      simpa using this
    simp [this]
  dec_sound := by
    intro fuel s a r h
    revert h
    cases hd : decULEB s with
    | error e => intro h; simp at h
    | ok p =>
      obtain ⟨n, t⟩ := p
      intro h
      simp only at h
      by_cases hn : n ≤ t.length
      · simp only [hn, if_true] at h
        revert h
        cases hc : c.dec fuel (t.take n) with
        | error e => intro h; simp at h
        | ok q =>
          obtain ⟨b, rem⟩ := q
          intro h
          by_cases hrem : rem = []
          · subst hrem
            simp only [if_pos rfl, Except.ok.injEq, Prod.mk.injEq] at h
            obtain ⟨rfl, rfl⟩ := h
            have hct : t.take n = c.enc a := by
              have := c.dec_sound _ _ _ _ hc
              simpa using this
            have hlen : (c.enc a).length = n := by
              rw [← hct]; simp; omega
            rw [decULEB_sound s n t hd, hlen, List.append_assoc, ← hct,
              List.take_append_drop]
          · simp [hrem] at h
      · simp [hn] at h
  cost_le a := by
    have := c.cost_le a
    simp only [List.length_append]
    omega

/-- A section: the pinned one-byte identifier, then the length-prefixed
contents. -/
def sectionCodec (id : UInt8) (c : Codec α) : Codec α :=
  prefixed [id] (.badSectionId id.toNat 0) (sized c)

end Codec

/-! ## Codecs of the finite operator and type-tag enumerations

Each enumeration is given a dense numeric tag, a complete duplicate-free list
of its inhabitants, and the resulting `Codec` via `Codec.tagged`.  The tag
numbering is the pinned one of this file; see the header for the declared scope
of the byte format. -/

namespace Enum
open Codec

/-- Numeric tags of `IntWidth`. -/
def intWidthTagged : Codec.Tagged IntWidth where
  code
    | .i32 => 0
    | .i64 => 1
  all := [.i32, .i64]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `IntWidth`. -/
def intWidthC : Codec IntWidth := Codec.tagged intWidthTagged

@[simp] theorem intWidthC_dec_enc (a : IntWidth) (fuel : Nat) (r : List UInt8) :
    intWidthC.dec fuel (intWidthC.enc a ++ r) = .ok (a, r) :=
  intWidthC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `FloatWidth`. -/
def floatWidthTagged : Codec.Tagged FloatWidth where
  code
    | .f32 => 0
    | .f64 => 1
  all := [.f32, .f64]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `FloatWidth`. -/
def floatWidthC : Codec FloatWidth := Codec.tagged floatWidthTagged

@[simp] theorem floatWidthC_dec_enc (a : FloatWidth) (fuel : Nat) (r : List UInt8) :
    floatWidthC.dec fuel (floatWidthC.enc a ++ r) = .ok (a, r) :=
  floatWidthC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `SignExt`. -/
def signExtTagged : Codec.Tagged SignExt where
  code
    | .signed => 0
    | .unsigned => 1
  all := [.signed, .unsigned]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `SignExt`. -/
def signExtC : Codec SignExt := Codec.tagged signExtTagged

@[simp] theorem signExtC_dec_enc (a : SignExt) (fuel : Nat) (r : List UInt8) :
    signExtC.dec fuel (signExtC.enc a ++ r) = .ok (a, r) :=
  signExtC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `MemWidth`. -/
def memWidthTagged : Codec.Tagged MemWidth where
  code
    | .w8 => 0
    | .w16 => 1
    | .w32 => 2
  all := [.w8, .w16, .w32]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `MemWidth`. -/
def memWidthC : Codec MemWidth := Codec.tagged memWidthTagged

@[simp] theorem memWidthC_dec_enc (a : MemWidth) (fuel : Nat) (r : List UInt8) :
    memWidthC.dec fuel (memWidthC.enc a ++ r) = .ok (a, r) :=
  memWidthC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `CatchKind`. -/
def catchKindTagged : Codec.Tagged CatchKind where
  code
    | .catch => 0
    | .catchRef => 1
    | .catchAll => 2
    | .catchAllRef => 3
  all := [.catch, .catchRef, .catchAll, .catchAllRef]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `CatchKind`. -/
def catchKindC : Codec CatchKind := Codec.tagged catchKindTagged

@[simp] theorem catchKindC_dec_enc (a : CatchKind) (fuel : Nat) (r : List UInt8) :
    catchKindC.dec fuel (catchKindC.enc a ++ r) = .ok (a, r) :=
  catchKindC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `IUnOp`. -/
def iUnOpTagged : Codec.Tagged IUnOp where
  code
    | .clz => 0
    | .ctz => 1
    | .popcnt => 2
    | .extend8S => 3
    | .extend16S => 4
    | .extend32S => 5
  all := [.clz, .ctz, .popcnt, .extend8S, .extend16S, .extend32S]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `IUnOp`. -/
def iUnOpC : Codec IUnOp := Codec.tagged iUnOpTagged

@[simp] theorem iUnOpC_dec_enc (a : IUnOp) (fuel : Nat) (r : List UInt8) :
    iUnOpC.dec fuel (iUnOpC.enc a ++ r) = .ok (a, r) :=
  iUnOpC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `IBinOp`. -/
def iBinOpTagged : Codec.Tagged IBinOp where
  code
    | .add => 0
    | .sub => 1
    | .mul => 2
    | .divS => 3
    | .divU => 4
    | .remS => 5
    | .remU => 6
    | .and => 7
    | .or => 8
    | .xor => 9
    | .shl => 10
    | .shrS => 11
    | .shrU => 12
    | .rotl => 13
    | .rotr => 14
  all := [.add, .sub, .mul, .divS, .divU, .remS, .remU, .and, .or, .xor, .shl, .shrS, .shrU, .rotl, .rotr]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `IBinOp`. -/
def iBinOpC : Codec IBinOp := Codec.tagged iBinOpTagged

@[simp] theorem iBinOpC_dec_enc (a : IBinOp) (fuel : Nat) (r : List UInt8) :
    iBinOpC.dec fuel (iBinOpC.enc a ++ r) = .ok (a, r) :=
  iBinOpC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `ITestOp`. -/
def iTestOpTagged : Codec.Tagged ITestOp where
  code
    | .eqz => 0
  all := [.eqz]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `ITestOp`. -/
def iTestOpC : Codec ITestOp := Codec.tagged iTestOpTagged

@[simp] theorem iTestOpC_dec_enc (a : ITestOp) (fuel : Nat) (r : List UInt8) :
    iTestOpC.dec fuel (iTestOpC.enc a ++ r) = .ok (a, r) :=
  iTestOpC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `IRelOp`. -/
def iRelOpTagged : Codec.Tagged IRelOp where
  code
    | .eq => 0
    | .ne => 1
    | .ltS => 2
    | .ltU => 3
    | .gtS => 4
    | .gtU => 5
    | .leS => 6
    | .leU => 7
    | .geS => 8
    | .geU => 9
  all := [.eq, .ne, .ltS, .ltU, .gtS, .gtU, .leS, .leU, .geS, .geU]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `IRelOp`. -/
def iRelOpC : Codec IRelOp := Codec.tagged iRelOpTagged

@[simp] theorem iRelOpC_dec_enc (a : IRelOp) (fuel : Nat) (r : List UInt8) :
    iRelOpC.dec fuel (iRelOpC.enc a ++ r) = .ok (a, r) :=
  iRelOpC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `FUnOp`. -/
def fUnOpTagged : Codec.Tagged FUnOp where
  code
    | .abs => 0
    | .neg => 1
    | .ceil => 2
    | .floor => 3
    | .trunc => 4
    | .nearest => 5
    | .sqrt => 6
  all := [.abs, .neg, .ceil, .floor, .trunc, .nearest, .sqrt]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `FUnOp`. -/
def fUnOpC : Codec FUnOp := Codec.tagged fUnOpTagged

@[simp] theorem fUnOpC_dec_enc (a : FUnOp) (fuel : Nat) (r : List UInt8) :
    fUnOpC.dec fuel (fUnOpC.enc a ++ r) = .ok (a, r) :=
  fUnOpC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `FBinOp`. -/
def fBinOpTagged : Codec.Tagged FBinOp where
  code
    | .add => 0
    | .sub => 1
    | .mul => 2
    | .div => 3
    | .min => 4
    | .max => 5
    | .copysign => 6
  all := [.add, .sub, .mul, .div, .min, .max, .copysign]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `FBinOp`. -/
def fBinOpC : Codec FBinOp := Codec.tagged fBinOpTagged

@[simp] theorem fBinOpC_dec_enc (a : FBinOp) (fuel : Nat) (r : List UInt8) :
    fBinOpC.dec fuel (fBinOpC.enc a ++ r) = .ok (a, r) :=
  fBinOpC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `FRelOp`. -/
def fRelOpTagged : Codec.Tagged FRelOp where
  code
    | .eq => 0
    | .ne => 1
    | .lt => 2
    | .gt => 3
    | .le => 4
    | .ge => 5
  all := [.eq, .ne, .lt, .gt, .le, .ge]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `FRelOp`. -/
def fRelOpC : Codec FRelOp := Codec.tagged fRelOpTagged

@[simp] theorem fRelOpC_dec_enc (a : FRelOp) (fuel : Nat) (r : List UInt8) :
    fRelOpC.dec fuel (fRelOpC.enc a ++ r) = .ok (a, r) :=
  fRelOpC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `CvtOp`. -/
def cvtOpTagged : Codec.Tagged CvtOp where
  code
    | .i32WrapI64 => 0
    | .i32TruncF32S => 1
    | .i32TruncF32U => 2
    | .i32TruncF64S => 3
    | .i32TruncF64U => 4
    | .i64ExtendI32S => 5
    | .i64ExtendI32U => 6
    | .i64TruncF32S => 7
    | .i64TruncF32U => 8
    | .i64TruncF64S => 9
    | .i64TruncF64U => 10
    | .f32ConvertI32S => 11
    | .f32ConvertI32U => 12
    | .f32ConvertI64S => 13
    | .f32ConvertI64U => 14
    | .f32DemoteF64 => 15
    | .f64ConvertI32S => 16
    | .f64ConvertI32U => 17
    | .f64ConvertI64S => 18
    | .f64ConvertI64U => 19
    | .f64PromoteF32 => 20
    | .i32ReinterpretF32 => 21
    | .i64ReinterpretF64 => 22
    | .f32ReinterpretI32 => 23
    | .f64ReinterpretI64 => 24
    | .i32TruncSatF32S => 25
    | .i32TruncSatF32U => 26
    | .i32TruncSatF64S => 27
    | .i32TruncSatF64U => 28
    | .i64TruncSatF32S => 29
    | .i64TruncSatF32U => 30
    | .i64TruncSatF64S => 31
    | .i64TruncSatF64U => 32
  all := [.i32WrapI64, .i32TruncF32S, .i32TruncF32U, .i32TruncF64S, .i32TruncF64U, .i64ExtendI32S, .i64ExtendI32U, .i64TruncF32S, .i64TruncF32U, .i64TruncF64S, .i64TruncF64U, .f32ConvertI32S, .f32ConvertI32U, .f32ConvertI64S, .f32ConvertI64U, .f32DemoteF64, .f64ConvertI32S, .f64ConvertI32U, .f64ConvertI64S, .f64ConvertI64U, .f64PromoteF32, .i32ReinterpretF32, .i64ReinterpretF64, .f32ReinterpretI32, .f64ReinterpretI64, .i32TruncSatF32S, .i32TruncSatF32U, .i32TruncSatF64S, .i32TruncSatF64U, .i64TruncSatF32S, .i64TruncSatF32U, .i64TruncSatF64S, .i64TruncSatF64U]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `CvtOp`. -/
def cvtOpC : Codec CvtOp := Codec.tagged cvtOpTagged

@[simp] theorem cvtOpC_dec_enc (a : CvtOp) (fuel : Nat) (r : List UInt8) :
    cvtOpC.dec fuel (cvtOpC.enc a ++ r) = .ok (a, r) :=
  cvtOpC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `VecShape`. -/
def vecShapeTagged : Codec.Tagged VecShape where
  code
    | .i8x16 => 0
    | .i16x8 => 1
    | .i32x4 => 2
    | .i64x2 => 3
    | .f32x4 => 4
    | .f64x2 => 5
  all := [.i8x16, .i16x8, .i32x4, .i64x2, .f32x4, .f64x2]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `VecShape`. -/
def vecShapeC : Codec VecShape := Codec.tagged vecShapeTagged

@[simp] theorem vecShapeC_dec_enc (a : VecShape) (fuel : Nat) (r : List UInt8) :
    vecShapeC.dec fuel (vecShapeC.enc a ++ r) = .ok (a, r) :=
  vecShapeC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `VecUnOp`. -/
def vecUnOpTagged : Codec.Tagged VecUnOp where
  code
    | .not => 0
    | .neg => 1
    | .abs => 2
    | .sqrt => 3
    | .ceil => 4
    | .floor => 5
    | .trunc => 6
    | .nearest => 7
    | .popcnt => 8
    | .anyTrue => 9
    | .allTrue => 10
    | .bitmask => 11
  all := [.not, .neg, .abs, .sqrt, .ceil, .floor, .trunc, .nearest, .popcnt, .anyTrue, .allTrue, .bitmask]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `VecUnOp`. -/
def vecUnOpC : Codec VecUnOp := Codec.tagged vecUnOpTagged

@[simp] theorem vecUnOpC_dec_enc (a : VecUnOp) (fuel : Nat) (r : List UInt8) :
    vecUnOpC.dec fuel (vecUnOpC.enc a ++ r) = .ok (a, r) :=
  vecUnOpC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `VecBinOp`. -/
def vecBinOpTagged : Codec.Tagged VecBinOp where
  code
    | .and => 0
    | .andnot => 1
    | .or => 2
    | .xor => 3
    | .add => 4
    | .sub => 5
    | .mul => 6
    | .div => 7
    | .min => 8
    | .max => 9
    | .pmin => 10
    | .pmax => 11
    | .addSatS => 12
    | .addSatU => 13
    | .subSatS => 14
    | .subSatU => 15
    | .avgrU => 16
    | .shl => 17
    | .shrS => 18
    | .shrU => 19
  all := [.and, .andnot, .or, .xor, .add, .sub, .mul, .div, .min, .max, .pmin, .pmax, .addSatS, .addSatU, .subSatS, .subSatU, .avgrU, .shl, .shrS, .shrU]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `VecBinOp`. -/
def vecBinOpC : Codec VecBinOp := Codec.tagged vecBinOpTagged

@[simp] theorem vecBinOpC_dec_enc (a : VecBinOp) (fuel : Nat) (r : List UInt8) :
    vecBinOpC.dec fuel (vecBinOpC.enc a ++ r) = .ok (a, r) :=
  vecBinOpC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `VecRelOp`. -/
def vecRelOpTagged : Codec.Tagged VecRelOp where
  code
    | .eq => 0
    | .ne => 1
    | .ltS => 2
    | .ltU => 3
    | .gtS => 4
    | .gtU => 5
    | .leS => 6
    | .leU => 7
    | .geS => 8
    | .geU => 9
    | .lt => 10
    | .gt => 11
    | .le => 12
    | .ge => 13
  all := [.eq, .ne, .ltS, .ltU, .gtS, .gtU, .leS, .leU, .geS, .geU, .lt, .gt, .le, .ge]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `VecRelOp`. -/
def vecRelOpC : Codec VecRelOp := Codec.tagged vecRelOpTagged

@[simp] theorem vecRelOpC_dec_enc (a : VecRelOp) (fuel : Nat) (r : List UInt8) :
    vecRelOpC.dec fuel (vecRelOpC.enc a ++ r) = .ok (a, r) :=
  vecRelOpC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `NumType`. -/
def numTypeTagged : Codec.Tagged NumType where
  code
    | .i32 => 0
    | .i64 => 1
    | .f32 => 2
    | .f64 => 3
  all := [.i32, .i64, .f32, .f64]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `NumType`. -/
def numTypeC : Codec NumType := Codec.tagged numTypeTagged

@[simp] theorem numTypeC_dec_enc (a : NumType) (fuel : Nat) (r : List UInt8) :
    numTypeC.dec fuel (numTypeC.enc a ++ r) = .ok (a, r) :=
  numTypeC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `VecType`. -/
def vecTypeTagged : Codec.Tagged VecType where
  code
    | .v128 => 0
  all := [.v128]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `VecType`. -/
def vecTypeC : Codec VecType := Codec.tagged vecTypeTagged

@[simp] theorem vecTypeC_dec_enc (a : VecType) (fuel : Nat) (r : List UInt8) :
    vecTypeC.dec fuel (vecTypeC.enc a ++ r) = .ok (a, r) :=
  vecTypeC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `PackedType`. -/
def packedTypeTagged : Codec.Tagged PackedType where
  code
    | .i8 => 0
    | .i16 => 1
  all := [.i8, .i16]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `PackedType`. -/
def packedTypeC : Codec PackedType := Codec.tagged packedTypeTagged

@[simp] theorem packedTypeC_dec_enc (a : PackedType) (fuel : Nat) (r : List UInt8) :
    packedTypeC.dec fuel (packedTypeC.enc a ++ r) = .ok (a, r) :=
  packedTypeC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `AddressType`. -/
def addressTypeTagged : Codec.Tagged AddressType where
  code
    | .i32 => 0
    | .i64 => 1
  all := [.i32, .i64]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `AddressType`. -/
def addressTypeC : Codec AddressType := Codec.tagged addressTypeTagged

@[simp] theorem addressTypeC_dec_enc (a : AddressType) (fuel : Nat) (r : List UInt8) :
    addressTypeC.dec fuel (addressTypeC.enc a ++ r) = .ok (a, r) :=
  addressTypeC.dec_enc a fuel r (Nat.zero_le _)

/-- Numeric tags of `AbsHeapType`. -/
def absHeapTypeTagged : Codec.Tagged AbsHeapType where
  code
    | .any => 0
    | .eq => 1
    | .i31 => 2
    | .struct => 3
    | .array => 4
    | .none => 5
    | .func => 6
    | .nofunc => 7
    | .extern => 8
    | .noextern => 9
    | .exn => 10
    | .noexn => 11
  all := [.any, .eq, .i31, .struct, .array, .none, .func, .nofunc, .extern, .noextern, .exn, .noexn]
  complete := by intro a; cases a <;> decide
  codes_nodup := by decide

/-- Codec of `AbsHeapType`. -/
def absHeapTypeC : Codec AbsHeapType := Codec.tagged absHeapTypeTagged

@[simp] theorem absHeapTypeC_dec_enc (a : AbsHeapType) (fuel : Nat) (r : List UInt8) :
    absHeapTypeC.dec fuel (absHeapTypeC.enc a ++ r) = .ok (a, r) :=
  absHeapTypeC.dec_enc a fuel r (Nat.zero_le _)

end Enum

/-! ## Codecs of the Core type grammar and of instruction immediates

Composite types are transported along explicit bijections onto sums and
products of already-verified codecs, so both halves of the inverse law come
for free from the combinators. -/

namespace Bin
open Codec

theorem costList_zero {α : Type} (c : Codec α) (hc : ∀ a, c.cost a = 0) :
    ∀ l : List α, costList c l = 0
  | [] => rfl
  | a :: as => by
      show c.cost a + costList c as = 0
      rw [hc a, costList_zero c hc as]

theorem dec_enc_zero {α : Type} (c : Codec α) (a : α) (fuel : Nat) (r : List UInt8)
    (h : c.cost a = 0) : c.dec fuel (c.enc a ++ r) = .ok (a, r) :=
  c.dec_enc a fuel r (by rw [h]; exact Nat.zero_le _)

/-! ### Primitive immediates -/

/-- Unsigned index/count immediates. -/
abbrev natC : Codec Nat := Codec.natCodec
/-- Signed constant immediates. -/
abbrev intC : Codec Int := Codec.intCodec
/-- Boolean flags. -/
abbrev boolC : Codec Bool := Codec.boolCodec
/-- Raw 32-bit patterns. -/
abbrev u32C : Codec UInt32 := Codec.uint32Codec
/-- Raw 64-bit patterns. -/
abbrev u64C : Codec UInt64 := Codec.uint64Codec
/-- Length-prefixed byte strings (names and data payloads). -/
abbrev bytesC : Codec (List UInt8) := Codec.bytesCodec
/-- Vectors of indices. -/
abbrev natsC : Codec (List Nat) := Codec.listCodec natC

@[simp] theorem natC_cost (a : Nat) : natC.cost a = 0 := rfl
@[simp] theorem intC_cost (a : Int) : intC.cost a = 0 := rfl
@[simp] theorem boolC_cost (a : Bool) : boolC.cost a = 0 := rfl
@[simp] theorem u32C_cost (a : UInt32) : u32C.cost a = 0 := rfl
@[simp] theorem u64C_cost (a : UInt64) : u64C.cost a = 0 := rfl
@[simp] theorem bytesC_cost (a : List UInt8) : bytesC.cost a = 0 := rfl
@[simp] theorem natsC_cost (a : List Nat) : natsC.cost a = 0 :=
  costList_zero natC (fun _ => rfl) a

@[simp] theorem natC_dec_enc (a : Nat) (fuel : Nat) (r : List UInt8) :
    natC.dec fuel (natC.enc a ++ r) = .ok (a, r) := dec_enc_zero _ a fuel r rfl
@[simp] theorem intC_dec_enc (a : Int) (fuel : Nat) (r : List UInt8) :
    intC.dec fuel (intC.enc a ++ r) = .ok (a, r) := dec_enc_zero _ a fuel r rfl
@[simp] theorem boolC_dec_enc (a : Bool) (fuel : Nat) (r : List UInt8) :
    boolC.dec fuel (boolC.enc a ++ r) = .ok (a, r) := dec_enc_zero _ a fuel r rfl
@[simp] theorem u32C_dec_enc (a : UInt32) (fuel : Nat) (r : List UInt8) :
    u32C.dec fuel (u32C.enc a ++ r) = .ok (a, r) := dec_enc_zero _ a fuel r rfl
@[simp] theorem u64C_dec_enc (a : UInt64) (fuel : Nat) (r : List UInt8) :
    u64C.dec fuel (u64C.enc a ++ r) = .ok (a, r) := dec_enc_zero _ a fuel r rfl
@[simp] theorem bytesC_dec_enc (a : List UInt8) (fuel : Nat) (r : List UInt8) :
    bytesC.dec fuel (bytesC.enc a ++ r) = .ok (a, r) := dec_enc_zero _ a fuel r rfl
@[simp] theorem natsC_dec_enc (a : List Nat) (fuel : Nat) (r : List UInt8) :
    natsC.dec fuel (natsC.enc a ++ r) = .ok (a, r) :=
  dec_enc_zero _ a fuel r (natsC_cost a)

/-! ### Heap, reference and value types -/

/-- Inverse of `HeapType.toSum`. -/
def ofSumHeapType : AbsHeapType ⊕ Nat → HeapType
  | .inl t => .abs t
  | .inr i => .concrete i

/-- Codec of `HeapType`. -/
def heapTypeC : Codec HeapType :=
  Codec.iso (Codec.sumCodec Enum.absHeapTypeC natC) HeapType.toSum ofSumHeapType
    (fun a => by cases a <;> rfl)
    (fun b => match b with | .inl _ => rfl | .inr _ => rfl)

@[simp] theorem heapTypeC_cost (t : HeapType) : heapTypeC.cost t = 0 := by
  cases t <;> rfl

@[simp] theorem heapTypeC_dec_enc (a : HeapType) (fuel : Nat) (r : List UInt8) :
    heapTypeC.dec fuel (heapTypeC.enc a ++ r) = .ok (a, r) :=
  dec_enc_zero _ a fuel r (heapTypeC_cost a)

/-- Codec of `RefType`. -/
def refTypeC : Codec RefType :=
  Codec.iso (Codec.pair boolC heapTypeC)
    (fun t => (t.nullable, t.heapType)) (fun p => ⟨p.1, p.2⟩)
    (fun _ => rfl) (fun _ => rfl)

@[simp] theorem refTypeC_cost (t : RefType) : refTypeC.cost t = 0 := by
  show boolC.cost t.nullable + heapTypeC.cost t.heapType = 0
  simp

@[simp] theorem refTypeC_dec_enc (a : RefType) (fuel : Nat) (r : List UInt8) :
    refTypeC.dec fuel (refTypeC.enc a ++ r) = .ok (a, r) :=
  dec_enc_zero _ a fuel r (refTypeC_cost a)

/-- Inverse of `ValType.toSum`. -/
def ofSumValType : NumType ⊕ (VecType ⊕ RefType) → ValType
  | .inl t => .num t
  | .inr (.inl t) => .vec t
  | .inr (.inr t) => .ref t

/-- Codec of `ValType`. -/
def valTypeC : Codec ValType :=
  Codec.iso (Codec.sumCodec Enum.numTypeC (Codec.sumCodec Enum.vecTypeC refTypeC))
    ValType.toSum ofSumValType
    (fun a => by cases a <;> rfl)
    (fun b => match b with
      | .inl _ => rfl | .inr (.inl _) => rfl | .inr (.inr _) => rfl)

@[simp] theorem valTypeC_cost (t : ValType) : valTypeC.cost t = 0 := by
  cases t with
  | num t => rfl
  | vec t => rfl
  | ref t => show refTypeC.cost t = 0; simp

@[simp] theorem valTypeC_dec_enc (a : ValType) (fuel : Nat) (r : List UInt8) :
    valTypeC.dec fuel (valTypeC.enc a ++ r) = .ok (a, r) :=
  dec_enc_zero _ a fuel r (valTypeC_cost a)

/-- Codec of a result type. -/
abbrev valTypesC : Codec (List ValType) := Codec.listCodec valTypeC

@[simp] theorem valTypesC_cost (l : List ValType) : valTypesC.cost l = 0 :=
  costList_zero valTypeC valTypeC_cost l

@[simp] theorem valTypesC_dec_enc (a : List ValType) (fuel : Nat) (r : List UInt8) :
    valTypesC.dec fuel (valTypesC.enc a ++ r) = .ok (a, r) :=
  dec_enc_zero _ a fuel r (valTypesC_cost a)

/-! ### Storage, field, structure, array, function and tag types -/

/-- Inverse of `StorageType.toSum`. -/
def ofSumStorageType : ValType ⊕ PackedType → StorageType
  | .inl t => .val t
  | .inr t => .packed t

/-- Codec of `StorageType`. -/
def storageTypeC : Codec StorageType :=
  Codec.iso (Codec.sumCodec valTypeC Enum.packedTypeC) StorageType.toSum ofSumStorageType
    (fun a => by cases a <;> rfl)
    (fun b => match b with | .inl _ => rfl | .inr _ => rfl)

@[simp] theorem storageTypeC_cost (t : StorageType) : storageTypeC.cost t = 0 := by
  cases t with
  | val t => show valTypeC.cost t = 0; simp
  | packed t => rfl

/-- Codec of `FieldType`. -/
def fieldTypeC : Codec FieldType :=
  Codec.iso (Codec.pair boolC storageTypeC)
    (fun t => (t.mutable, t.storage)) (fun p => ⟨p.1, p.2⟩)
    (fun _ => rfl) (fun _ => rfl)

@[simp] theorem fieldTypeC_cost (t : FieldType) : fieldTypeC.cost t = 0 := by
  show boolC.cost t.mutable + storageTypeC.cost t.storage = 0
  simp

/-- Codec of `StructType`. -/
def structTypeC : Codec StructType :=
  Codec.iso (Codec.listCodec fieldTypeC) (fun t => t.fields) (fun l => ⟨l⟩)
    (fun _ => rfl) (fun _ => rfl)

@[simp] theorem structTypeC_cost (t : StructType) : structTypeC.cost t = 0 :=
  costList_zero fieldTypeC fieldTypeC_cost t.fields

/-- Codec of `ArrayType`. -/
def arrayTypeC : Codec ArrayType :=
  Codec.iso fieldTypeC (fun t => t.element) (fun f => ⟨f⟩)
    (fun _ => rfl) (fun _ => rfl)

@[simp] theorem arrayTypeC_cost (t : ArrayType) : arrayTypeC.cost t = 0 :=
  fieldTypeC_cost t.element

/-- Codec of `FuncType`. -/
def funcTypeC : Codec FuncType :=
  Codec.iso (Codec.pair valTypesC valTypesC)
    (fun t => (t.params, t.results)) (fun p => ⟨p.1, p.2⟩)
    (fun _ => rfl) (fun _ => rfl)

@[simp] theorem funcTypeC_cost (t : FuncType) : funcTypeC.cost t = 0 := by
  show valTypesC.cost t.params + valTypesC.cost t.results = 0
  simp

@[simp] theorem funcTypeC_dec_enc (a : FuncType) (fuel : Nat) (r : List UInt8) :
    funcTypeC.dec fuel (funcTypeC.enc a ++ r) = .ok (a, r) :=
  dec_enc_zero _ a fuel r (funcTypeC_cost a)

/-- Codec of `TagType`. -/
def tagTypeC : Codec TagType :=
  Codec.iso funcTypeC (fun t => t.funcType) (fun f => ⟨f⟩)
    (fun _ => rfl) (fun _ => rfl)

@[simp] theorem tagTypeC_cost (t : TagType) : tagTypeC.cost t = 0 :=
  funcTypeC_cost t.funcType

@[simp] theorem tagTypeC_dec_enc (a : TagType) (fuel : Nat) (r : List UInt8) :
    tagTypeC.dec fuel (tagTypeC.enc a ++ r) = .ok (a, r) :=
  dec_enc_zero _ a fuel r (tagTypeC_cost a)

/-- Inverse of `CompType.toSum`. -/
def ofSumCompType : FuncType ⊕ (StructType ⊕ ArrayType) → CompType
  | .inl t => .func t
  | .inr (.inl t) => .struct t
  | .inr (.inr t) => .array t

/-- Codec of `CompType`. -/
def compTypeC : Codec CompType :=
  Codec.iso (Codec.sumCodec funcTypeC (Codec.sumCodec structTypeC arrayTypeC))
    CompType.toSum ofSumCompType
    (fun a => by cases a <;> rfl)
    (fun b => match b with
      | .inl _ => rfl | .inr (.inl _) => rfl | .inr (.inr _) => rfl)

@[simp] theorem compTypeC_cost (t : CompType) : compTypeC.cost t = 0 := by
  cases t with
  | func t => show funcTypeC.cost t = 0; simp
  | «struct» t => show structTypeC.cost t = 0; simp
  | array t => show arrayTypeC.cost t = 0; simp

/-- Codec of `SubType`. -/
def subTypeC : Codec SubType :=
  Codec.iso (Codec.pair boolC (Codec.pair natsC compTypeC))
    (fun t => (t.final, t.supertypes, t.body)) (fun p => ⟨p.1, p.2.1, p.2.2⟩)
    (fun _ => rfl) (fun _ => rfl)

@[simp] theorem subTypeC_cost (t : SubType) : subTypeC.cost t = 0 := by
  show boolC.cost t.final + (natsC.cost t.supertypes + compTypeC.cost t.body) = 0
  simp

/-- Codec of `RecType`. -/
def recTypeC : Codec RecType :=
  Codec.iso (Codec.listCodec subTypeC) (fun t => t.types) (fun l => ⟨l⟩)
    (fun _ => rfl) (fun _ => rfl)

@[simp] theorem recTypeC_cost (t : RecType) : recTypeC.cost t = 0 :=
  costList_zero subTypeC subTypeC_cost t.types

/-! ### Limits, memory, table and global types -/

/-- Codec of `Limits`. -/
def limitsC : Codec Limits :=
  Codec.iso (Codec.pair natC (Codec.optionCodec natC))
    (fun l => (l.min, l.max)) (fun p => ⟨p.1, p.2⟩)
    (fun _ => rfl) (fun _ => rfl)

@[simp] theorem limitsC_cost (l : Limits) : limitsC.cost l = 0 := by
  show natC.cost l.min + (Codec.optionCodec natC).cost l.max = 0
  cases l.max <;> rfl

/-- Codec of `MemType`. -/
def memTypeC : Codec MemType :=
  Codec.iso (Codec.pair Enum.addressTypeC limitsC)
    (fun t => (t.addressType, t.limits)) (fun p => ⟨p.1, p.2⟩)
    (fun _ => rfl) (fun _ => rfl)

@[simp] theorem memTypeC_cost (t : MemType) : memTypeC.cost t = 0 := by
  show Enum.addressTypeC.cost t.addressType + limitsC.cost t.limits = 0
  rw [limitsC_cost]
  rfl

@[simp] theorem memTypeC_dec_enc (a : MemType) (fuel : Nat) (r : List UInt8) :
    memTypeC.dec fuel (memTypeC.enc a ++ r) = .ok (a, r) :=
  dec_enc_zero _ a fuel r (memTypeC_cost a)

/-- Codec of `TableType`. -/
def tableTypeC : Codec TableType :=
  Codec.iso (Codec.pair Enum.addressTypeC (Codec.pair limitsC refTypeC))
    (fun t => (t.addressType, t.limits, t.element)) (fun p => ⟨p.1, p.2.1, p.2.2⟩)
    (fun _ => rfl) (fun _ => rfl)

@[simp] theorem tableTypeC_cost (t : TableType) : tableTypeC.cost t = 0 := by
  show Enum.addressTypeC.cost t.addressType
      + (limitsC.cost t.limits + refTypeC.cost t.element) = 0
  rw [limitsC_cost, refTypeC_cost]
  rfl

@[simp] theorem tableTypeC_dec_enc (a : TableType) (fuel : Nat) (r : List UInt8) :
    tableTypeC.dec fuel (tableTypeC.enc a ++ r) = .ok (a, r) :=
  dec_enc_zero _ a fuel r (tableTypeC_cost a)

/-- Codec of `GlobalType`. -/
def globalTypeC : Codec GlobalType :=
  Codec.iso (Codec.pair boolC valTypeC)
    (fun t => (t.mutable, t.valType)) (fun p => ⟨p.1, p.2⟩)
    (fun _ => rfl) (fun _ => rfl)

@[simp] theorem globalTypeC_cost (t : GlobalType) : globalTypeC.cost t = 0 := by
  show boolC.cost t.mutable + valTypeC.cost t.valType = 0
  simp

@[simp] theorem globalTypeC_dec_enc (a : GlobalType) (fuel : Nat) (r : List UInt8) :
    globalTypeC.dec fuel (globalTypeC.enc a ++ r) = .ok (a, r) :=
  dec_enc_zero _ a fuel r (globalTypeC_cost a)

/-! ### Instruction immediates -/

/-- Inverse of the sum view of `BlockType`. -/
def ofSumBlockType : Unit ⊕ (ValType ⊕ Nat) → BlockType
  | .inl _ => .empty
  | .inr (.inl t) => .value t
  | .inr (.inr i) => .typeIndex i

/-- Sum view of `BlockType`. -/
def toSumBlockType : BlockType → Unit ⊕ (ValType ⊕ Nat)
  | .empty => .inl ()
  | .value t => .inr (.inl t)
  | .typeIndex i => .inr (.inr i)

/-- Codec of `BlockType`. -/
def blockTypeC : Codec BlockType :=
  Codec.iso (Codec.sumCodec Codec.unitCodec (Codec.sumCodec valTypeC natC))
    toSumBlockType ofSumBlockType
    (fun a => by cases a <;> rfl)
    (fun b => match b with
      | .inl () => rfl | .inr (.inl _) => rfl | .inr (.inr _) => rfl)

@[simp] theorem blockTypeC_cost (t : BlockType) : blockTypeC.cost t = 0 := by
  cases t with
  | empty => rfl
  | value t => show valTypeC.cost t = 0; simp
  | typeIndex i => rfl

@[simp] theorem blockTypeC_dec_enc (a : BlockType) (fuel : Nat) (r : List UInt8) :
    blockTypeC.dec fuel (blockTypeC.enc a ++ r) = .ok (a, r) :=
  dec_enc_zero _ a fuel r (blockTypeC_cost a)

/-- Codec of `MemArg`. -/
def memArgC : Codec MemArg :=
  Codec.iso (Codec.pair natC (Codec.pair natC natC))
    (fun m => (m.memory, m.align, m.offset)) (fun p => ⟨p.1, p.2.1, p.2.2⟩)
    (fun _ => rfl) (fun _ => rfl)

@[simp] theorem memArgC_cost (m : MemArg) : memArgC.cost m = 0 := rfl

@[simp] theorem memArgC_dec_enc (a : MemArg) (fuel : Nat) (r : List UInt8) :
    memArgC.dec fuel (memArgC.enc a ++ r) = .ok (a, r) :=
  dec_enc_zero _ a fuel r rfl

/-- Codec of a `try_table` catch clause. -/
def catchC : Codec Catch :=
  Codec.iso (Codec.pair Enum.catchKindC (Codec.pair natC natC))
    (fun c => (c.kind, c.tag, c.label)) (fun p => ⟨p.1, p.2.1, p.2.2⟩)
    (fun _ => rfl) (fun _ => rfl)

@[simp] theorem catchC_cost (c : Catch) : catchC.cost c = 0 := rfl

/-- Codec of a list of catch clauses. -/
abbrev catchesC : Codec (List Catch) := Codec.listCodec catchC

@[simp] theorem catchesC_cost (l : List Catch) : catchesC.cost l = 0 :=
  costList_zero catchC catchC_cost l

@[simp] theorem catchesC_dec_enc (a : List Catch) (fuel : Nat) (r : List UInt8) :
    catchesC.dec fuel (catchesC.enc a ++ r) = .ok (a, r) :=
  dec_enc_zero _ a fuel r (catchesC_cost a)

/-- Optional sign-extension immediate. -/
abbrev optSignExtC : Codec (Option SignExt) := Codec.optionCodec Enum.signExtC
/-- Optional narrowed-width immediate. -/
abbrev optMemWidthC : Codec (Option MemWidth) := Codec.optionCodec Enum.memWidthC
/-- Optional narrowed-width-and-sign immediate of a partial load. -/
abbrev optNarrowC : Codec (Option (MemWidth × SignExt)) :=
  Codec.optionCodec (Codec.pair Enum.memWidthC Enum.signExtC)
/-- Optional type annotation of `select`. -/
abbrev optValTypesC : Codec (Option (List ValType)) := Codec.optionCodec valTypesC

@[simp] theorem optSignExtC_cost (a : Option SignExt) : optSignExtC.cost a = 0 := by
  cases a <;> rfl
@[simp] theorem optMemWidthC_cost (a : Option MemWidth) : optMemWidthC.cost a = 0 := by
  cases a <;> rfl
@[simp] theorem optNarrowC_cost (a : Option (MemWidth × SignExt)) :
    optNarrowC.cost a = 0 := by cases a <;> rfl
@[simp] theorem optValTypesC_cost (a : Option (List ValType)) :
    optValTypesC.cost a = 0 := by cases a with
  | none => rfl
  | some l => show valTypesC.cost l = 0; simp

@[simp] theorem optSignExtC_dec_enc (a : Option SignExt) (fuel : Nat) (r : List UInt8) :
    optSignExtC.dec fuel (optSignExtC.enc a ++ r) = .ok (a, r) :=
  dec_enc_zero _ a fuel r (optSignExtC_cost a)
@[simp] theorem optMemWidthC_dec_enc (a : Option MemWidth) (fuel : Nat) (r : List UInt8) :
    optMemWidthC.dec fuel (optMemWidthC.enc a ++ r) = .ok (a, r) :=
  dec_enc_zero _ a fuel r (optMemWidthC_cost a)
@[simp] theorem optNarrowC_dec_enc (a : Option (MemWidth × SignExt)) (fuel : Nat)
    (r : List UInt8) : optNarrowC.dec fuel (optNarrowC.enc a ++ r) = .ok (a, r) :=
  dec_enc_zero _ a fuel r (optNarrowC_cost a)
@[simp] theorem optValTypesC_dec_enc (a : Option (List ValType)) (fuel : Nat)
    (r : List UInt8) : optValTypesC.dec fuel (optValTypesC.enc a ++ r) = .ok (a, r) :=
  dec_enc_zero _ a fuel r (optValTypesC_cost a)

end Bin

/-! ## Instruction opcodes

The opcode numbering is a pinned dense enumeration internal to this file, one
code per constructor of `Instr`.  Byte-level identity with the vendored Core
3.0 opcode table is *not* claimed here; see the file header. -/

open Codec Bin Enum

/-- One opcode per instruction form of the declared subset. -/
inductive Opcode where
  | unreachable
  | nop
  | br
  | brIf
  | brTable
  | ret
  | call
  | callIndirect
  | returnCall
  | returnCallIndirect
  | throw
  | throwRef
  | drop
  | select
  | localGet
  | localSet
  | localTee
  | globalGet
  | globalSet
  | refNull
  | refIsNull
  | refFunc
  | refEq
  | refAsNonNull
  | refTest
  | refCast
  | refI31
  | i31GetS
  | i31GetU
  | anyConvertExtern
  | externConvertAny
  | structNew
  | structNewDefault
  | structGet
  | structSet
  | arrayNew
  | arrayNewDefault
  | arrayGet
  | arraySet
  | arrayLen
  | tableGet
  | tableSet
  | tableSize
  | tableGrow
  | tableFill
  | tableCopy
  | tableInit
  | elemDrop
  | load
  | store
  | vecLoad
  | vecStore
  | memorySize
  | memoryGrow
  | memoryFill
  | memoryCopy
  | memoryInit
  | dataDrop
  | i32Const
  | i64Const
  | f32Const
  | f64Const
  | iUnOp
  | iBinOp
  | iTestOp
  | iRelOp
  | fUnOp
  | fBinOp
  | fRelOp
  | cvtOp
  | vecConst
  | vecUnOp
  | vecBinOp
  | vecRelOp
  | vecBitselect
  | vecSplat
  | vecExtractLane
  | vecReplaceLane
  | vecShuffle
  | block
  | loop
  | ifThenElse
  | tryTable
  deriving DecidableEq, Repr, Inhabited

namespace Opcode

/-- The pinned dense opcode number. -/
def code : Opcode → Nat
  | .unreachable => 0
  | .nop => 1
  | .br => 2
  | .brIf => 3
  | .brTable => 4
  | .ret => 5
  | .call => 6
  | .callIndirect => 7
  | .returnCall => 8
  | .returnCallIndirect => 9
  | .throw => 10
  | .throwRef => 11
  | .drop => 12
  | .select => 13
  | .localGet => 14
  | .localSet => 15
  | .localTee => 16
  | .globalGet => 17
  | .globalSet => 18
  | .refNull => 19
  | .refIsNull => 20
  | .refFunc => 21
  | .refEq => 22
  | .refAsNonNull => 23
  | .refTest => 24
  | .refCast => 25
  | .refI31 => 26
  | .i31GetS => 27
  | .i31GetU => 28
  | .anyConvertExtern => 29
  | .externConvertAny => 30
  | .structNew => 31
  | .structNewDefault => 32
  | .structGet => 33
  | .structSet => 34
  | .arrayNew => 35
  | .arrayNewDefault => 36
  | .arrayGet => 37
  | .arraySet => 38
  | .arrayLen => 39
  | .tableGet => 40
  | .tableSet => 41
  | .tableSize => 42
  | .tableGrow => 43
  | .tableFill => 44
  | .tableCopy => 45
  | .tableInit => 46
  | .elemDrop => 47
  | .load => 48
  | .store => 49
  | .vecLoad => 50
  | .vecStore => 51
  | .memorySize => 52
  | .memoryGrow => 53
  | .memoryFill => 54
  | .memoryCopy => 55
  | .memoryInit => 56
  | .dataDrop => 57
  | .i32Const => 58
  | .i64Const => 59
  | .f32Const => 60
  | .f64Const => 61
  | .iUnOp => 62
  | .iBinOp => 63
  | .iTestOp => 64
  | .iRelOp => 65
  | .fUnOp => 66
  | .fBinOp => 67
  | .fRelOp => 68
  | .cvtOp => 69
  | .vecConst => 70
  | .vecUnOp => 71
  | .vecBinOp => 72
  | .vecRelOp => 73
  | .vecBitselect => 74
  | .vecSplat => 75
  | .vecExtractLane => 76
  | .vecReplaceLane => 77
  | .vecShuffle => 78
  | .block => 79
  | .loop => 80
  | .ifThenElse => 81
  | .tryTable => 82

/-- Every opcode. -/
def all : List Opcode :=
  [.unreachable, .nop, .br, .brIf, .brTable, .ret, .call, .callIndirect, .returnCall, .returnCallIndirect, .throw, .throwRef, .drop, .select, .localGet, .localSet, .localTee, .globalGet, .globalSet, .refNull, .refIsNull, .refFunc, .refEq, .refAsNonNull, .refTest, .refCast, .refI31, .i31GetS, .i31GetU, .anyConvertExtern, .externConvertAny, .structNew, .structNewDefault, .structGet, .structSet, .arrayNew, .arrayNewDefault, .arrayGet, .arraySet, .arrayLen, .tableGet, .tableSet, .tableSize, .tableGrow, .tableFill, .tableCopy, .tableInit, .elemDrop, .load, .store, .vecLoad, .vecStore, .memorySize, .memoryGrow, .memoryFill, .memoryCopy, .memoryInit, .dataDrop, .i32Const, .i64Const, .f32Const, .f64Const, .iUnOp, .iBinOp, .iTestOp, .iRelOp, .fUnOp, .fBinOp, .fRelOp, .cvtOp, .vecConst, .vecUnOp, .vecBinOp, .vecRelOp, .vecBitselect, .vecSplat, .vecExtractLane, .vecReplaceLane, .vecShuffle, .block, .loop, .ifThenElse, .tryTable]

theorem mem_all (o : Opcode) : o ∈ all := by cases o <;> decide

theorem codes_nodup : (all.map code).Nodup := by decide

end Opcode

/-- The tagged finite type of opcodes. -/
def opcodeTagged : Codec.Tagged Opcode where
  code := Opcode.code
  all := Opcode.all
  complete := Opcode.mem_all
  codes_nodup := Opcode.codes_nodup

/-- Codec of opcodes. -/
def opcodeC : Codec Opcode := Codec.tagged opcodeTagged

@[simp] theorem opcodeC_dec_enc (o : Opcode) (fuel : Nat) (r : List UInt8) :
    opcodeC.dec fuel (opcodeC.enc o ++ r) = .ok (o, r) :=
  opcodeC.dec_enc o fuel r (Nat.zero_le _)

theorem opcodeC_enc_length_pos (o : Opcode) : 0 < (opcodeC.enc o).length := by
  have h : opcodeC.enc o = encodeULEB (Opcode.code o) := rfl
  rw [h]
  cases hc : encodeULEB (Opcode.code o) with
  | nil => exact absurd hc (encodeULEB_ne_nil _)
  | cons a l => simp

/-! ## Instruction encoding -/

mutual

/-- Canonical encoding of an instruction. -/
def Instr.enc : Instr → List UInt8
  | .unreachable => opcodeC.enc .unreachable
  | .nop => opcodeC.enc .nop
  | .br a0 => opcodeC.enc .br ++ natC.enc a0
  | .brIf a0 => opcodeC.enc .brIf ++ natC.enc a0
  | .brTable a0 a1 => opcodeC.enc .brTable ++ (natsC.enc a0 ++ natC.enc a1)
  | .ret => opcodeC.enc .ret
  | .call a0 => opcodeC.enc .call ++ natC.enc a0
  | .callIndirect a0 a1 => opcodeC.enc .callIndirect ++ (natC.enc a0 ++ natC.enc a1)
  | .returnCall a0 => opcodeC.enc .returnCall ++ natC.enc a0
  | .returnCallIndirect a0 a1 => opcodeC.enc .returnCallIndirect ++ (natC.enc a0 ++ natC.enc a1)
  | .throw a0 => opcodeC.enc .throw ++ natC.enc a0
  | .throwRef => opcodeC.enc .throwRef
  | .drop => opcodeC.enc .drop
  | .select a0 => opcodeC.enc .select ++ optValTypesC.enc a0
  | .localGet a0 => opcodeC.enc .localGet ++ natC.enc a0
  | .localSet a0 => opcodeC.enc .localSet ++ natC.enc a0
  | .localTee a0 => opcodeC.enc .localTee ++ natC.enc a0
  | .globalGet a0 => opcodeC.enc .globalGet ++ natC.enc a0
  | .globalSet a0 => opcodeC.enc .globalSet ++ natC.enc a0
  | .refNull a0 => opcodeC.enc .refNull ++ heapTypeC.enc a0
  | .refIsNull => opcodeC.enc .refIsNull
  | .refFunc a0 => opcodeC.enc .refFunc ++ natC.enc a0
  | .refEq => opcodeC.enc .refEq
  | .refAsNonNull => opcodeC.enc .refAsNonNull
  | .refTest a0 a1 => opcodeC.enc .refTest ++ (boolC.enc a0 ++ heapTypeC.enc a1)
  | .refCast a0 a1 => opcodeC.enc .refCast ++ (boolC.enc a0 ++ heapTypeC.enc a1)
  | .refI31 => opcodeC.enc .refI31
  | .i31GetS => opcodeC.enc .i31GetS
  | .i31GetU => opcodeC.enc .i31GetU
  | .anyConvertExtern => opcodeC.enc .anyConvertExtern
  | .externConvertAny => opcodeC.enc .externConvertAny
  | .structNew a0 => opcodeC.enc .structNew ++ natC.enc a0
  | .structNewDefault a0 => opcodeC.enc .structNewDefault ++ natC.enc a0
  | .structGet a0 a1 a2 => opcodeC.enc .structGet ++ (natC.enc a0 ++ (natC.enc a1 ++ optSignExtC.enc a2))
  | .structSet a0 a1 => opcodeC.enc .structSet ++ (natC.enc a0 ++ natC.enc a1)
  | .arrayNew a0 => opcodeC.enc .arrayNew ++ natC.enc a0
  | .arrayNewDefault a0 => opcodeC.enc .arrayNewDefault ++ natC.enc a0
  | .arrayGet a0 a1 => opcodeC.enc .arrayGet ++ (natC.enc a0 ++ optSignExtC.enc a1)
  | .arraySet a0 => opcodeC.enc .arraySet ++ natC.enc a0
  | .arrayLen => opcodeC.enc .arrayLen
  | .tableGet a0 => opcodeC.enc .tableGet ++ natC.enc a0
  | .tableSet a0 => opcodeC.enc .tableSet ++ natC.enc a0
  | .tableSize a0 => opcodeC.enc .tableSize ++ natC.enc a0
  | .tableGrow a0 => opcodeC.enc .tableGrow ++ natC.enc a0
  | .tableFill a0 => opcodeC.enc .tableFill ++ natC.enc a0
  | .tableCopy a0 a1 => opcodeC.enc .tableCopy ++ (natC.enc a0 ++ natC.enc a1)
  | .tableInit a0 a1 => opcodeC.enc .tableInit ++ (natC.enc a0 ++ natC.enc a1)
  | .elemDrop a0 => opcodeC.enc .elemDrop ++ natC.enc a0
  | .load a0 a1 a2 => opcodeC.enc .load ++ (numTypeC.enc a0 ++ (optNarrowC.enc a1 ++ memArgC.enc a2))
  | .store a0 a1 a2 => opcodeC.enc .store ++ (numTypeC.enc a0 ++ (optMemWidthC.enc a1 ++ memArgC.enc a2))
  | .vecLoad a0 => opcodeC.enc .vecLoad ++ memArgC.enc a0
  | .vecStore a0 => opcodeC.enc .vecStore ++ memArgC.enc a0
  | .memorySize a0 => opcodeC.enc .memorySize ++ natC.enc a0
  | .memoryGrow a0 => opcodeC.enc .memoryGrow ++ natC.enc a0
  | .memoryFill a0 => opcodeC.enc .memoryFill ++ natC.enc a0
  | .memoryCopy a0 a1 => opcodeC.enc .memoryCopy ++ (natC.enc a0 ++ natC.enc a1)
  | .memoryInit a0 a1 => opcodeC.enc .memoryInit ++ (natC.enc a0 ++ natC.enc a1)
  | .dataDrop a0 => opcodeC.enc .dataDrop ++ natC.enc a0
  | .i32Const a0 => opcodeC.enc .i32Const ++ intC.enc a0
  | .i64Const a0 => opcodeC.enc .i64Const ++ intC.enc a0
  | .f32Const a0 => opcodeC.enc .f32Const ++ u32C.enc a0
  | .f64Const a0 => opcodeC.enc .f64Const ++ u64C.enc a0
  | .iUnOp a0 a1 => opcodeC.enc .iUnOp ++ (intWidthC.enc a0 ++ iUnOpC.enc a1)
  | .iBinOp a0 a1 => opcodeC.enc .iBinOp ++ (intWidthC.enc a0 ++ iBinOpC.enc a1)
  | .iTestOp a0 a1 => opcodeC.enc .iTestOp ++ (intWidthC.enc a0 ++ iTestOpC.enc a1)
  | .iRelOp a0 a1 => opcodeC.enc .iRelOp ++ (intWidthC.enc a0 ++ iRelOpC.enc a1)
  | .fUnOp a0 a1 => opcodeC.enc .fUnOp ++ (floatWidthC.enc a0 ++ fUnOpC.enc a1)
  | .fBinOp a0 a1 => opcodeC.enc .fBinOp ++ (floatWidthC.enc a0 ++ fBinOpC.enc a1)
  | .fRelOp a0 a1 => opcodeC.enc .fRelOp ++ (floatWidthC.enc a0 ++ fRelOpC.enc a1)
  | .cvtOp a0 => opcodeC.enc .cvtOp ++ cvtOpC.enc a0
  | .vecConst a0 a1 => opcodeC.enc .vecConst ++ (u64C.enc a0 ++ u64C.enc a1)
  | .vecUnOp a0 a1 => opcodeC.enc .vecUnOp ++ (vecShapeC.enc a0 ++ vecUnOpC.enc a1)
  | .vecBinOp a0 a1 => opcodeC.enc .vecBinOp ++ (vecShapeC.enc a0 ++ vecBinOpC.enc a1)
  | .vecRelOp a0 a1 => opcodeC.enc .vecRelOp ++ (vecShapeC.enc a0 ++ vecRelOpC.enc a1)
  | .vecBitselect => opcodeC.enc .vecBitselect
  | .vecSplat a0 => opcodeC.enc .vecSplat ++ vecShapeC.enc a0
  | .vecExtractLane a0 a1 a2 => opcodeC.enc .vecExtractLane ++ (vecShapeC.enc a0 ++ (natC.enc a1 ++ optSignExtC.enc a2))
  | .vecReplaceLane a0 a1 => opcodeC.enc .vecReplaceLane ++ (vecShapeC.enc a0 ++ natC.enc a1)
  | .vecShuffle a0 => opcodeC.enc .vecShuffle ++ natsC.enc a0
  | .block bt body => opcodeC.enc .block ++ (blockTypeC.enc bt ++ Expr.enc body)
  | .loop bt body => opcodeC.enc .loop ++ (blockTypeC.enc bt ++ Expr.enc body)
  | .ifThenElse bt b1 b2 =>
      opcodeC.enc .ifThenElse ++ (blockTypeC.enc bt ++ (Expr.enc b1 ++ Expr.enc b2))
  | .tryTable bt cs body =>
      opcodeC.enc .tryTable ++ (blockTypeC.enc bt ++ (catchesC.enc cs ++ Expr.enc body))

/-- Canonical encoding of an instruction sequence. -/
def Expr.enc : Expr → List UInt8
  | .nil => [0]
  | .cons i e => 1 :: (Instr.enc i ++ Expr.enc e)

end

/-- The structural fuel an instruction needs: its own encoding length. -/
def Instr.cost (i : Instr) : Nat := (Instr.enc i).length

/-- The structural fuel an instruction sequence needs. -/
def Expr.cost (e : Expr) : Nat := (Expr.enc e).length

theorem Instr.enc_length_pos (i : Instr) : 0 < (Instr.enc i).length := by
  cases i <;> simp only [Instr.enc, List.length_append] <;>
    first
      | exact opcodeC_enc_length_pos _
      | exact Nat.lt_of_lt_of_le (opcodeC_enc_length_pos _) (Nat.le_add_right _ _)

theorem Instr.one_le_cost (i : Instr) : 1 ≤ Instr.cost i := Instr.enc_length_pos i

theorem Expr.one_le_cost (e : Expr) : 1 ≤ Expr.cost e := by
  cases e <;> simp [Expr.cost, Expr.enc]

/-! ## Instruction decoding -/

/-- Nullary instruction form. -/
def dec0 (x : Instr) (s : List UInt8) : Except DecodeFault (Instr × List UInt8) := .ok (x, s)

/-- Unary instruction form. -/
def dec1 {α : Type} (c : Codec α) (k : α → Instr) (s : List UInt8) :
    Except DecodeFault (Instr × List UInt8) :=
  match c.dec 0 s with
  | .error e => .error e
  | .ok (a, r) => .ok (k a, r)

/-- Binary instruction form. -/
def dec2 {α β : Type} (c : Codec α) (d : Codec β) (k : α → β → Instr) (s : List UInt8) :
    Except DecodeFault (Instr × List UInt8) :=
  match c.dec 0 s with
  | .error e => .error e
  | .ok (a, r) =>
    match d.dec 0 r with
    | .error e => .error e
    | .ok (b, r') => .ok (k a b, r')

/-- Ternary instruction form. -/
def dec3 {α β γ : Type} (c : Codec α) (d : Codec β) (g : Codec γ)
    (k : α → β → γ → Instr) (s : List UInt8) : Except DecodeFault (Instr × List UInt8) :=
  match c.dec 0 s with
  | .error e => .error e
  | .ok (a, r) =>
    match d.dec 0 r with
    | .error e => .error e
    | .ok (b, r') =>
      match g.dec 0 r' with
      | .error e => .error e
      | .ok (x, r'') => .ok (k a b x, r'')

theorem dec0_sound {x i : Instr} {s r : List UInt8} (h : dec0 x s = .ok (i, r)) :
    i = x ∧ s = r := by
  simp only [dec0, Except.ok.injEq, Prod.mk.injEq] at h
  exact ⟨h.1.symm, h.2⟩

theorem dec1_sound {α : Type} {c : Codec α} {k : α → Instr} {s : List UInt8}
    {i : Instr} {r : List UInt8} (h : dec1 c k s = .ok (i, r)) :
    ∃ a, i = k a ∧ s = c.enc a ++ r := by
  revert h
  cases hc : c.dec 0 s with
  | error e => intro h; simp [dec1, hc] at h
  | ok q =>
    obtain ⟨a, t⟩ := q
    intro h
    simp only [dec1, hc, Except.ok.injEq, Prod.mk.injEq] at h
    obtain ⟨h1, h2⟩ := h
    subst h1; subst h2
    exact ⟨a, rfl, c.dec_sound _ _ _ _ hc⟩

theorem dec2_sound {α β : Type} {c : Codec α} {d : Codec β} {k : α → β → Instr}
    {s : List UInt8} {i : Instr} {r : List UInt8} (h : dec2 c d k s = .ok (i, r)) :
    ∃ a b, i = k a b ∧ s = c.enc a ++ (d.enc b ++ r) := by
  revert h
  cases hc : c.dec 0 s with
  | error e => intro h; simp [dec2, hc] at h
  | ok q =>
    obtain ⟨a, t⟩ := q
    cases hd : d.dec 0 t with
    | error e => intro h; simp [dec2, hc, hd] at h
    | ok q2 =>
      obtain ⟨b, t2⟩ := q2
      intro h
      simp only [dec2, hc, hd, Except.ok.injEq, Prod.mk.injEq] at h
      obtain ⟨h1, h2⟩ := h
      subst h1; subst h2
      refine ⟨a, b, rfl, ?_⟩
      rw [c.dec_sound _ _ _ _ hc, d.dec_sound _ _ _ _ hd]

theorem dec3_sound {α β γ : Type} {c : Codec α} {d : Codec β} {g : Codec γ}
    {k : α → β → γ → Instr} {s : List UInt8} {i : Instr} {r : List UInt8}
    (h : dec3 c d g k s = .ok (i, r)) :
    ∃ a b x, i = k a b x ∧ s = c.enc a ++ (d.enc b ++ (g.enc x ++ r)) := by
  revert h
  cases hc : c.dec 0 s with
  | error e => intro h; simp [dec3, hc] at h
  | ok q =>
    obtain ⟨a, t⟩ := q
    cases hd : d.dec 0 t with
    | error e => intro h; simp [dec3, hc, hd] at h
    | ok q2 =>
      obtain ⟨b, t2⟩ := q2
      cases hg : g.dec 0 t2 with
      | error e => intro h; simp [dec3, hc, hd, hg] at h
      | ok q3 =>
        obtain ⟨x, t3⟩ := q3
        intro h
        simp only [dec3, hc, hd, hg, Except.ok.injEq, Prod.mk.injEq] at h
        obtain ⟨h1, h2⟩ := h
        subst h1; subst h2
        refine ⟨a, b, x, rfl, ?_⟩
        rw [c.dec_sound _ _ _ _ hc, d.dec_sound _ _ _ _ hd, g.dec_sound _ _ _ _ hg]

mutual

/-- Decode one instruction. -/
def decInstr : Nat → List UInt8 → Except DecodeFault (Instr × List UInt8)
  | 0, _ => .error .fuelExhausted
  | f + 1, s =>
    match opcodeC.dec 0 s with
    | .error e => .error e
    | .ok (op, t) =>
      match op with
      | .unreachable => dec0 .unreachable t
      | .nop => dec0 .nop t
      | .br => dec1 natC (fun a => .br a) t
      | .brIf => dec1 natC (fun a => .brIf a) t
      | .brTable => dec2 natsC natC (fun a b => .brTable a b) t
      | .ret => dec0 .ret t
      | .call => dec1 natC (fun a => .call a) t
      | .callIndirect => dec2 natC natC (fun a b => .callIndirect a b) t
      | .returnCall => dec1 natC (fun a => .returnCall a) t
      | .returnCallIndirect => dec2 natC natC (fun a b => .returnCallIndirect a b) t
      | .throw => dec1 natC (fun a => .throw a) t
      | .throwRef => dec0 .throwRef t
      | .drop => dec0 .drop t
      | .select => dec1 optValTypesC (fun a => .select a) t
      | .localGet => dec1 natC (fun a => .localGet a) t
      | .localSet => dec1 natC (fun a => .localSet a) t
      | .localTee => dec1 natC (fun a => .localTee a) t
      | .globalGet => dec1 natC (fun a => .globalGet a) t
      | .globalSet => dec1 natC (fun a => .globalSet a) t
      | .refNull => dec1 heapTypeC (fun a => .refNull a) t
      | .refIsNull => dec0 .refIsNull t
      | .refFunc => dec1 natC (fun a => .refFunc a) t
      | .refEq => dec0 .refEq t
      | .refAsNonNull => dec0 .refAsNonNull t
      | .refTest => dec2 boolC heapTypeC (fun a b => .refTest a b) t
      | .refCast => dec2 boolC heapTypeC (fun a b => .refCast a b) t
      | .refI31 => dec0 .refI31 t
      | .i31GetS => dec0 .i31GetS t
      | .i31GetU => dec0 .i31GetU t
      | .anyConvertExtern => dec0 .anyConvertExtern t
      | .externConvertAny => dec0 .externConvertAny t
      | .structNew => dec1 natC (fun a => .structNew a) t
      | .structNewDefault => dec1 natC (fun a => .structNewDefault a) t
      | .structGet => dec3 natC natC optSignExtC (fun a b x => .structGet a b x) t
      | .structSet => dec2 natC natC (fun a b => .structSet a b) t
      | .arrayNew => dec1 natC (fun a => .arrayNew a) t
      | .arrayNewDefault => dec1 natC (fun a => .arrayNewDefault a) t
      | .arrayGet => dec2 natC optSignExtC (fun a b => .arrayGet a b) t
      | .arraySet => dec1 natC (fun a => .arraySet a) t
      | .arrayLen => dec0 .arrayLen t
      | .tableGet => dec1 natC (fun a => .tableGet a) t
      | .tableSet => dec1 natC (fun a => .tableSet a) t
      | .tableSize => dec1 natC (fun a => .tableSize a) t
      | .tableGrow => dec1 natC (fun a => .tableGrow a) t
      | .tableFill => dec1 natC (fun a => .tableFill a) t
      | .tableCopy => dec2 natC natC (fun a b => .tableCopy a b) t
      | .tableInit => dec2 natC natC (fun a b => .tableInit a b) t
      | .elemDrop => dec1 natC (fun a => .elemDrop a) t
      | .load => dec3 numTypeC optNarrowC memArgC (fun a b x => .load a b x) t
      | .store => dec3 numTypeC optMemWidthC memArgC (fun a b x => .store a b x) t
      | .vecLoad => dec1 memArgC (fun a => .vecLoad a) t
      | .vecStore => dec1 memArgC (fun a => .vecStore a) t
      | .memorySize => dec1 natC (fun a => .memorySize a) t
      | .memoryGrow => dec1 natC (fun a => .memoryGrow a) t
      | .memoryFill => dec1 natC (fun a => .memoryFill a) t
      | .memoryCopy => dec2 natC natC (fun a b => .memoryCopy a b) t
      | .memoryInit => dec2 natC natC (fun a b => .memoryInit a b) t
      | .dataDrop => dec1 natC (fun a => .dataDrop a) t
      | .i32Const => dec1 intC (fun a => .i32Const a) t
      | .i64Const => dec1 intC (fun a => .i64Const a) t
      | .f32Const => dec1 u32C (fun a => .f32Const a) t
      | .f64Const => dec1 u64C (fun a => .f64Const a) t
      | .iUnOp => dec2 intWidthC iUnOpC (fun a b => .iUnOp a b) t
      | .iBinOp => dec2 intWidthC iBinOpC (fun a b => .iBinOp a b) t
      | .iTestOp => dec2 intWidthC iTestOpC (fun a b => .iTestOp a b) t
      | .iRelOp => dec2 intWidthC iRelOpC (fun a b => .iRelOp a b) t
      | .fUnOp => dec2 floatWidthC fUnOpC (fun a b => .fUnOp a b) t
      | .fBinOp => dec2 floatWidthC fBinOpC (fun a b => .fBinOp a b) t
      | .fRelOp => dec2 floatWidthC fRelOpC (fun a b => .fRelOp a b) t
      | .cvtOp => dec1 cvtOpC (fun a => .cvtOp a) t
      | .vecConst => dec2 u64C u64C (fun a b => .vecConst a b) t
      | .vecUnOp => dec2 vecShapeC vecUnOpC (fun a b => .vecUnOp a b) t
      | .vecBinOp => dec2 vecShapeC vecBinOpC (fun a b => .vecBinOp a b) t
      | .vecRelOp => dec2 vecShapeC vecRelOpC (fun a b => .vecRelOp a b) t
      | .vecBitselect => dec0 .vecBitselect t
      | .vecSplat => dec1 vecShapeC (fun a => .vecSplat a) t
      | .vecExtractLane => dec3 vecShapeC natC optSignExtC (fun a b x => .vecExtractLane a b x) t
      | .vecReplaceLane => dec2 vecShapeC natC (fun a b => .vecReplaceLane a b) t
      | .vecShuffle => dec1 natsC (fun a => .vecShuffle a) t
      | .block =>
        match blockTypeC.dec 0 t with
        | .error e => .error e
        | .ok (bt, t1) =>
          match decExpr f t1 with
          | .error e => .error e
          | .ok (body, t2) => .ok (.block bt body, t2)
      | .loop =>
        match blockTypeC.dec 0 t with
        | .error e => .error e
        | .ok (bt, t1) =>
          match decExpr f t1 with
          | .error e => .error e
          | .ok (body, t2) => .ok (.loop bt body, t2)
      | .ifThenElse =>
        match blockTypeC.dec 0 t with
        | .error e => .error e
        | .ok (bt, t1) =>
          match decExpr f t1 with
          | .error e => .error e
          | .ok (b1, t2) =>
            match decExpr f t2 with
            | .error e => .error e
            | .ok (b2, t3) => .ok (.ifThenElse bt b1 b2, t3)
      | .tryTable =>
        match blockTypeC.dec 0 t with
        | .error e => .error e
        | .ok (bt, t1) =>
          match catchesC.dec 0 t1 with
          | .error e => .error e
          | .ok (cs, t2) =>
            match decExpr f t2 with
            | .error e => .error e
            | .ok (body, t3) => .ok (.tryTable bt cs body, t3)

/-- Decode an instruction sequence. -/
def decExpr : Nat → List UInt8 → Except DecodeFault (Expr × List UInt8)
  | 0, _ => .error .fuelExhausted
  | f + 1, s =>
    match s with
    | [] => .error .unexpectedEndOfInput
    | b :: t =>
      if b = 0 then .ok (.nil, t)
      else if b = 1 then
        match decInstr f t with
        | .error e => .error e
        | .ok (i, t1) =>
          match decExpr f t1 with
          | .error e => .error e
          | .ok (e2, t2) => .ok (.cons i e2, t2)
      else .error (.unknownTag b.toNat)

end

/-! ## Correctness of the instruction codec -/

theorem decBlock_sound {k : BlockType → Expr → Instr} {f : Nat} {t : List UInt8}
    {i : Instr} {r : List UInt8}
    (ihE : ∀ (s : List UInt8) (e : Expr) (r : List UInt8),
      decExpr f s = .ok (e, r) → s = Expr.enc e ++ r)
    (h : (match blockTypeC.dec 0 t with
          | Except.error e => Except.error e
          | Except.ok (bt, t1) =>
            match decExpr f t1 with
            | Except.error e => Except.error e
            | Except.ok (body, t2) => Except.ok (k bt body, t2)) = Except.ok (i, r)) :
    ∃ bt body, i = k bt body ∧ t = blockTypeC.enc bt ++ (Expr.enc body ++ r) := by
  revert h
  cases hb : blockTypeC.dec 0 t with
  | error e => intro h; exact absurd h (by simp)
  | ok qb =>
    obtain ⟨bt, t1⟩ := qb
    intro h
    replace h : (match decExpr f t1 with
        | Except.error e => Except.error e
        | Except.ok (body, t2) => Except.ok (k bt body, t2)) = Except.ok (i, r) := h
    revert h
    cases he : decExpr f t1 with
    | error e => intro h; exact absurd h (by simp)
    | ok qe =>
      obtain ⟨body, t2⟩ := qe
      intro h
      replace h : Except.ok (k bt body, t2) = (Except.ok (i, r) :
        Except DecodeFault (Instr × List UInt8)) := h
      simp only [Except.ok.injEq, Prod.mk.injEq] at h
      obtain ⟨rfl, rfl⟩ := h
      refine ⟨bt, body, rfl, ?_⟩
      rw [blockTypeC.dec_sound _ _ _ _ hb, ihE _ _ _ he]

theorem decIf_sound {f : Nat} {t : List UInt8} {i : Instr} {r : List UInt8}
    (ihE : ∀ (s : List UInt8) (e : Expr) (r : List UInt8),
      decExpr f s = .ok (e, r) → s = Expr.enc e ++ r)
    (h : (match blockTypeC.dec 0 t with
          | Except.error e => Except.error e
          | Except.ok (bt, t1) =>
            match decExpr f t1 with
            | Except.error e => Except.error e
            | Except.ok (b1, t2) =>
              match decExpr f t2 with
              | Except.error e => Except.error e
              | Except.ok (b2, t3) =>
                Except.ok (Instr.ifThenElse bt b1 b2, t3)) = Except.ok (i, r)) :
    ∃ bt b1 b2, i = .ifThenElse bt b1 b2 ∧
      t = blockTypeC.enc bt ++ (Expr.enc b1 ++ (Expr.enc b2 ++ r)) := by
  revert h
  cases hb : blockTypeC.dec 0 t with
  | error e => intro h; exact absurd h (by simp)
  | ok qb =>
    obtain ⟨bt, t1⟩ := qb
    intro h
    replace h : (match decExpr f t1 with
        | Except.error e => Except.error e
        | Except.ok (b1, t2) =>
          match decExpr f t2 with
          | Except.error e => Except.error e
          | Except.ok (b2, t3) =>
            Except.ok (Instr.ifThenElse bt b1 b2, t3)) = Except.ok (i, r) := h
    revert h
    cases he : decExpr f t1 with
    | error e => intro h; exact absurd h (by simp)
    | ok qe =>
      obtain ⟨b1, t2⟩ := qe
      intro h
      replace h : (match decExpr f t2 with
          | Except.error e => Except.error e
          | Except.ok (b2, t3) =>
            Except.ok (Instr.ifThenElse bt b1 b2, t3)) = Except.ok (i, r) := h
      revert h
      cases he2 : decExpr f t2 with
      | error e => intro h; exact absurd h (by simp)
      | ok qe2 =>
        obtain ⟨b2, t3⟩ := qe2
        intro h
        replace h : Except.ok (Instr.ifThenElse bt b1 b2, t3) = (Except.ok (i, r) :
          Except DecodeFault (Instr × List UInt8)) := h
        simp only [Except.ok.injEq, Prod.mk.injEq] at h
        obtain ⟨rfl, rfl⟩ := h
        refine ⟨bt, b1, b2, rfl, ?_⟩
        rw [blockTypeC.dec_sound _ _ _ _ hb, ihE _ _ _ he, ihE _ _ _ he2]

theorem decTry_sound {f : Nat} {t : List UInt8} {i : Instr} {r : List UInt8}
    (ihE : ∀ (s : List UInt8) (e : Expr) (r : List UInt8),
      decExpr f s = .ok (e, r) → s = Expr.enc e ++ r)
    (h : (match blockTypeC.dec 0 t with
          | Except.error e => Except.error e
          | Except.ok (bt, t1) =>
            match catchesC.dec 0 t1 with
            | Except.error e => Except.error e
            | Except.ok (cs, t2) =>
              match decExpr f t2 with
              | Except.error e => Except.error e
              | Except.ok (body, t3) =>
                Except.ok (Instr.tryTable bt cs body, t3)) = Except.ok (i, r)) :
    ∃ bt cs body, i = .tryTable bt cs body ∧
      t = blockTypeC.enc bt ++ (catchesC.enc cs ++ (Expr.enc body ++ r)) := by
  revert h
  cases hb : blockTypeC.dec 0 t with
  | error e => intro h; exact absurd h (by simp)
  | ok qb =>
    obtain ⟨bt, t1⟩ := qb
    intro h
    replace h : (match catchesC.dec 0 t1 with
        | Except.error e => Except.error e
        | Except.ok (cs, t2) =>
          match decExpr f t2 with
          | Except.error e => Except.error e
          | Except.ok (body, t3) =>
            Except.ok (Instr.tryTable bt cs body, t3)) = Except.ok (i, r) := h
    revert h
    cases hc : catchesC.dec 0 t1 with
    | error e => intro h; exact absurd h (by simp)
    | ok qc =>
      obtain ⟨cs, t2⟩ := qc
      intro h
      replace h : (match decExpr f t2 with
          | Except.error e => Except.error e
          | Except.ok (body, t3) =>
            Except.ok (Instr.tryTable bt cs body, t3)) = Except.ok (i, r) := h
      revert h
      cases he : decExpr f t2 with
      | error e => intro h; exact absurd h (by simp)
      | ok qe =>
        obtain ⟨body, t3⟩ := qe
        intro h
        replace h : Except.ok (Instr.tryTable bt cs body, t3) = (Except.ok (i, r) :
          Except DecodeFault (Instr × List UInt8)) := h
        simp only [Except.ok.injEq, Prod.mk.injEq] at h
        obtain ⟨rfl, rfl⟩ := h
        refine ⟨bt, cs, body, rfl, ?_⟩
        rw [blockTypeC.dec_sound _ _ _ _ hb, catchesC.dec_sound _ _ _ _ hc,
          ihE _ _ _ he]

set_option maxHeartbeats 4000000 in
/-- Decoding an encoded instruction (or instruction sequence) returns exactly
that instruction and exactly the untouched trailing bytes. -/
theorem decInstr_decExpr_enc : ∀ fuel : Nat,
    (∀ (i : Instr) (r : List UInt8), Instr.cost i ≤ fuel →
        decInstr fuel (Instr.enc i ++ r) = .ok (i, r)) ∧
    (∀ (e : Expr) (r : List UInt8), Expr.cost e ≤ fuel →
        decExpr fuel (Expr.enc e ++ r) = .ok (e, r)) := by
  intro fuel
  induction fuel with
  | zero =>
    refine ⟨?_, ?_⟩
    · intro i r h; exact absurd h (by have := Instr.one_le_cost i; omega)
    · intro e r h; exact absurd h (by have := Expr.one_le_cost e; omega)
  | succ f ih =>
    obtain ⟨ihI, ihE⟩ := ih
    refine ⟨?_, ?_⟩
    · intro i r h
      cases i
      case block bt body =>
        have hb : Expr.cost body ≤ f := by
          have h1 := opcodeC_enc_length_pos Opcode.block
          simp only [Instr.cost, Instr.enc, List.length_append] at h
          simp only [Expr.cost]
          omega
        simp [decInstr, Instr.enc, List.append_assoc, ihE body r hb]
      case loop bt body =>
        have hb : Expr.cost body ≤ f := by
          have h1 := opcodeC_enc_length_pos Opcode.loop
          simp only [Instr.cost, Instr.enc, List.length_append] at h
          simp only [Expr.cost]
          omega
        simp [decInstr, Instr.enc, List.append_assoc, ihE body r hb]
      case ifThenElse bt b1 b2 =>
        have hcs : Expr.cost b1 ≤ f ∧ Expr.cost b2 ≤ f := by
          have h1 := opcodeC_enc_length_pos Opcode.ifThenElse
          simp only [Instr.cost, Instr.enc, List.length_append] at h
          simp only [Expr.cost]
          omega
        simp [decInstr, Instr.enc, List.append_assoc,
          ihE b1 (Expr.enc b2 ++ r) hcs.1, ihE b2 r hcs.2]
      case tryTable bt cs body =>
        have hb : Expr.cost body ≤ f := by
          have h1 := opcodeC_enc_length_pos Opcode.tryTable
          simp only [Instr.cost, Instr.enc, List.length_append] at h
          simp only [Expr.cost]
          omega
        simp [decInstr, Instr.enc, List.append_assoc, ihE body r hb]
      all_goals
        simp [decInstr, Instr.enc, dec0, dec1, dec2, dec3, List.append_assoc]
    · intro e r h
      cases e
      case nil => simp [decExpr, Expr.enc]
      case cons i e' =>
        have hcs : Instr.cost i ≤ f ∧ Expr.cost e' ≤ f := by
          simp only [Expr.cost, Expr.enc, List.length_cons, List.length_append] at h
          simp only [Instr.cost, Expr.cost]
          omega
        simp [decExpr, Expr.enc, List.append_assoc,
          ihI i (Expr.enc e' ++ r) hcs.1, ihE e' r hcs.2]

set_option maxHeartbeats 4000000 in
/-- Whenever the instruction decoder succeeds, the bytes it consumed are
exactly the encoding of the instruction it returned. -/
theorem decInstr_decExpr_sound : ∀ fuel : Nat,
    (∀ (s : List UInt8) (i : Instr) (r : List UInt8),
        decInstr fuel s = .ok (i, r) → s = Instr.enc i ++ r) ∧
    (∀ (s : List UInt8) (e : Expr) (r : List UInt8),
        decExpr fuel s = .ok (e, r) → s = Expr.enc e ++ r) := by
  intro fuel
  induction fuel with
  | zero =>
    refine ⟨?_, ?_⟩
    · intro s i r h; simp [decInstr] at h
    · intro s e r h; simp [decExpr] at h
  | succ f ih =>
    obtain ⟨ihI, ihE⟩ := ih
    refine ⟨?_, ?_⟩
    · intro s i r h
      rw [decInstr] at h
      cases hop : opcodeC.dec 0 s with
      | error e => rw [hop] at h; simp at h
      | ok q =>
        obtain ⟨op, t⟩ := q
        rw [hop] at h
        have hs : s = opcodeC.enc op ++ t := opcodeC.dec_sound _ _ _ _ hop
        subst hs
        cases op
        case block =>
          obtain ⟨bt, body, rfl, rfl⟩ := decBlock_sound ihE h
          simp [Instr.enc, List.append_assoc]
        case loop =>
          obtain ⟨bt, body, rfl, rfl⟩ := decBlock_sound ihE h
          simp [Instr.enc, List.append_assoc]
        case ifThenElse =>
          obtain ⟨bt, b1, b2, rfl, rfl⟩ := decIf_sound ihE h
          simp [Instr.enc, List.append_assoc]
        case tryTable =>
          obtain ⟨bt, cs, body, rfl, rfl⟩ := decTry_sound ihE h
          simp [Instr.enc, List.append_assoc]
        all_goals
          first
          | (obtain ⟨rfl, rfl⟩ := dec0_sound h; simp [Instr.enc])
          | (obtain ⟨a, rfl, rfl⟩ := dec1_sound h
             simp [Instr.enc, List.append_assoc])
          | (obtain ⟨a, b, rfl, rfl⟩ := dec2_sound h
             simp [Instr.enc, List.append_assoc])
          | (obtain ⟨a, b, x, rfl, rfl⟩ := dec3_sound h
             simp [Instr.enc, List.append_assoc])
    · intro s e r h
      cases s with
      | nil => rw [decExpr] at h; exact absurd h (by simp)
      | cons b t =>
        rw [decExpr] at h
        by_cases hb0 : b = 0
        · subst hb0
          simp only [if_pos rfl, Except.ok.injEq, Prod.mk.injEq] at h
          obtain ⟨rfl, rfl⟩ := h
          rfl
        · simp only [hb0, if_false] at h
          by_cases hb1 : b = 1
          · subst hb1
            simp only [if_pos rfl] at h
            revert h
            cases hi : decInstr f t with
            | error e => intro h; simp at h
            | ok q =>
              obtain ⟨i, t1⟩ := q
              cases he : decExpr f t1 with
              | error e => intro h; simp [he] at h
              | ok q2 =>
                obtain ⟨e2, t2⟩ := q2
                intro h
                simp only [he, Except.ok.injEq, Prod.mk.injEq] at h
                obtain ⟨rfl, rfl⟩ := h
                show (1 : UInt8) :: t = 1 :: (Instr.enc i ++ Expr.enc e2 ++ r)
                rw [ihI _ _ _ hi, ihE _ _ _ he, List.append_assoc]
          · simp [hb1] at h

/-- The verified codec of a single instruction. -/
def instrC : Codec Instr where
  enc := Instr.enc
  dec := decInstr
  cost := Instr.cost
  dec_enc a fuel r h := (decInstr_decExpr_enc fuel).1 a r h
  dec_sound fuel s a r h := (decInstr_decExpr_sound fuel).1 s a r h
  cost_le _ := Nat.le_refl _

/-- The verified codec of an instruction sequence. -/
def exprC : Codec Expr where
  enc := Expr.enc
  dec := decExpr
  cost := Expr.cost
  dec_enc a fuel r h := (decInstr_decExpr_enc fuel).2 a r h
  dec_sound fuel s a r h := (decInstr_decExpr_sound fuel).2 s a r h
  cost_le _ := Nat.le_refl _

/-! ## Module component codecs -/

/-- Sum view of an import description. -/
def toSumImportDesc :
    ImportDesc → Nat ⊕ (TableType ⊕ (MemType ⊕ (GlobalType ⊕ TagType)))
  | .func t => .inl t
  | .table t => .inr (.inl t)
  | .mem t => .inr (.inr (.inl t))
  | .global t => .inr (.inr (.inr (.inl t)))
  | .tag t => .inr (.inr (.inr (.inr t)))

/-- Inverse of `toSumImportDesc`. -/
def ofSumImportDesc :
    Nat ⊕ (TableType ⊕ (MemType ⊕ (GlobalType ⊕ TagType))) → ImportDesc
  | .inl t => .func t
  | .inr (.inl t) => .table t
  | .inr (.inr (.inl t)) => .mem t
  | .inr (.inr (.inr (.inl t))) => .global t
  | .inr (.inr (.inr (.inr t))) => .tag t

/-- Codec of an import description. -/
def importDescC : Codec ImportDesc :=
  Codec.iso (Codec.sumCodec Bin.natC (Codec.sumCodec Bin.tableTypeC
      (Codec.sumCodec Bin.memTypeC (Codec.sumCodec Bin.globalTypeC Bin.tagTypeC))))
    toSumImportDesc ofSumImportDesc
    (fun a => by cases a <;> rfl)
    (fun b => match b with
      | .inl _ => rfl | .inr (.inl _) => rfl | .inr (.inr (.inl _)) => rfl
      | .inr (.inr (.inr (.inl _))) => rfl | .inr (.inr (.inr (.inr _))) => rfl)

/-- Codec of an import. -/
def importC : Codec Import :=
  Codec.iso (Codec.pair Bin.bytesC (Codec.pair Bin.bytesC importDescC))
    (fun i => (i.module, i.name, i.desc)) (fun p => ⟨p.1, p.2.1, p.2.2⟩)
    (fun _ => rfl) (fun _ => rfl)

/-- Sum view of an export description. -/
def toSumExportDesc : ExportDesc → Nat ⊕ (Nat ⊕ (Nat ⊕ (Nat ⊕ Nat)))
  | .func i => .inl i
  | .table i => .inr (.inl i)
  | .mem i => .inr (.inr (.inl i))
  | .global i => .inr (.inr (.inr (.inl i)))
  | .tag i => .inr (.inr (.inr (.inr i)))

/-- Inverse of `toSumExportDesc`. -/
def ofSumExportDesc : Nat ⊕ (Nat ⊕ (Nat ⊕ (Nat ⊕ Nat))) → ExportDesc
  | .inl i => .func i
  | .inr (.inl i) => .table i
  | .inr (.inr (.inl i)) => .mem i
  | .inr (.inr (.inr (.inl i))) => .global i
  | .inr (.inr (.inr (.inr i))) => .tag i

/-- Codec of an export description. -/
def exportDescC : Codec ExportDesc :=
  Codec.iso (Codec.sumCodec Bin.natC (Codec.sumCodec Bin.natC
      (Codec.sumCodec Bin.natC (Codec.sumCodec Bin.natC Bin.natC))))
    toSumExportDesc ofSumExportDesc
    (fun a => by cases a <;> rfl)
    (fun b => match b with
      | .inl _ => rfl | .inr (.inl _) => rfl | .inr (.inr (.inl _)) => rfl
      | .inr (.inr (.inr (.inl _))) => rfl | .inr (.inr (.inr (.inr _))) => rfl)

/-- Codec of an export. -/
def exportC : Codec Export :=
  Codec.iso (Codec.pair Bin.bytesC exportDescC)
    (fun e => (e.name, e.desc)) (fun p => ⟨p.1, p.2⟩)
    (fun _ => rfl) (fun _ => rfl)

/-- Codec of a defined function. -/
def funcC : Codec Func :=
  Codec.iso (Codec.pair Bin.natC (Codec.pair Bin.valTypesC exprC))
    (fun f => (f.type, f.locals, f.body)) (fun p => ⟨p.1, p.2.1, p.2.2⟩)
    (fun _ => rfl) (fun _ => rfl)

/-- Codec of a defined table. -/
def tableC : Codec Table :=
  Codec.iso (Codec.pair Bin.tableTypeC exprC)
    (fun t => (t.type, t.init)) (fun p => ⟨p.1, p.2⟩)
    (fun _ => rfl) (fun _ => rfl)

/-- Codec of a defined memory. -/
def memC : Codec Mem :=
  Codec.iso Bin.memTypeC (fun m => m.type) (fun t => ⟨t⟩)
    (fun _ => rfl) (fun _ => rfl)

/-- Codec of a defined global. -/
def globalC : Codec Global :=
  Codec.iso (Codec.pair Bin.globalTypeC exprC)
    (fun g => (g.type, g.init)) (fun p => ⟨p.1, p.2⟩)
    (fun _ => rfl) (fun _ => rfl)

/-- Sum view of an element-segment mode. -/
def toSumElemMode : ElemMode → Unit ⊕ ((Nat × Expr) ⊕ Unit)
  | .passive => .inl ()
  | .active t o => .inr (.inl (t, o))
  | .declarative => .inr (.inr ())

/-- Inverse of `toSumElemMode`. -/
def ofSumElemMode : Unit ⊕ ((Nat × Expr) ⊕ Unit) → ElemMode
  | .inl _ => .passive
  | .inr (.inl (t, o)) => .active t o
  | .inr (.inr _) => .declarative

/-- Codec of an element-segment mode. -/
def elemModeC : Codec ElemMode :=
  Codec.iso (Codec.sumCodec Codec.unitCodec
      (Codec.sumCodec (Codec.pair Bin.natC exprC) Codec.unitCodec))
    toSumElemMode ofSumElemMode
    (fun a => by cases a <;> rfl)
    (fun b => match b with
      | .inl () => rfl | .inr (.inl (_, _)) => rfl | .inr (.inr ()) => rfl)

/-- Codec of an element segment. -/
def elemC : Codec Elem :=
  Codec.iso (Codec.pair Bin.refTypeC (Codec.pair (Codec.listCodec exprC) elemModeC))
    (fun e => (e.type, e.init, e.mode)) (fun p => ⟨p.1, p.2.1, p.2.2⟩)
    (fun _ => rfl) (fun _ => rfl)

/-- Sum view of a data-segment mode. -/
def toSumDataMode : DataMode → Unit ⊕ (Nat × Expr)
  | .passive => .inl ()
  | .active m o => .inr (m, o)

/-- Inverse of `toSumDataMode`. -/
def ofSumDataMode : Unit ⊕ (Nat × Expr) → DataMode
  | .inl _ => .passive
  | .inr (m, o) => .active m o

/-- Codec of a data-segment mode. -/
def dataModeC : Codec DataMode :=
  Codec.iso (Codec.sumCodec Codec.unitCodec (Codec.pair Bin.natC exprC))
    toSumDataMode ofSumDataMode
    (fun a => by cases a <;> rfl)
    (fun b => match b with | .inl () => rfl | .inr (_, _) => rfl)

/-- Codec of a data segment. -/
def dataC : Codec Data :=
  Codec.iso (Codec.pair Bin.bytesC dataModeC)
    (fun d => (d.init, d.mode)) (fun p => ⟨p.1, p.2⟩)
    (fun _ => rfl) (fun _ => rfl)

/-! ## Sections and the module codec

`encode` always emits all eleven sections, in the pinned order, each with its
identifier byte and ULEB128 byte-size prefix; `decode` accepts exactly that
canonical sequence. -/

/-- The nested product the module record is transported onto. -/
abbrev ModuleTuple :=
  List RecType × List Import × List Func × List Table × List Mem × List TagType ×
    List Global × List Export × Option Nat × List Elem × List Data

/-- The section sequence of a module, in the pinned order. -/
def moduleTupleC : Codec ModuleTuple :=
  Codec.pair (Codec.sectionCodec 1 (Codec.listCodec Bin.recTypeC))
    (Codec.pair (Codec.sectionCodec 2 (Codec.listCodec importC))
      (Codec.pair (Codec.sectionCodec 3 (Codec.listCodec funcC))
        (Codec.pair (Codec.sectionCodec 4 (Codec.listCodec tableC))
          (Codec.pair (Codec.sectionCodec 5 (Codec.listCodec memC))
            (Codec.pair (Codec.sectionCodec 13 (Codec.listCodec Bin.tagTypeC))
              (Codec.pair (Codec.sectionCodec 6 (Codec.listCodec globalC))
                (Codec.pair (Codec.sectionCodec 7 (Codec.listCodec exportC))
                  (Codec.pair (Codec.sectionCodec 8 (Codec.optionCodec Bin.natC))
                    (Codec.pair (Codec.sectionCodec 9 (Codec.listCodec elemC))
                      (Codec.sectionCodec 11 (Codec.listCodec dataC)))))))))))

/-- The module preamble: the pinned `\0asm` magic and version 1. -/
def magicBytes : List UInt8 := [0, 0x61, 0x73, 0x6D, 1, 0, 0, 0]

/-- The verified codec of a whole module. -/
def moduleC : Codec Module :=
  Codec.prefixed magicBytes .badMagic
    (Codec.iso moduleTupleC
      (fun m => (m.types, m.imports, m.funcs, m.tables, m.mems, m.tags,
        m.globals, m.exports, m.start, m.elems, m.datas))
      (fun p => ⟨p.1, p.2.1, p.2.2.1, p.2.2.2.1, p.2.2.2.2.1, p.2.2.2.2.2.1,
        p.2.2.2.2.2.2.1, p.2.2.2.2.2.2.2.1, p.2.2.2.2.2.2.2.2.1,
        p.2.2.2.2.2.2.2.2.2.1, p.2.2.2.2.2.2.2.2.2.2⟩)
      (fun _ => rfl) (fun _ => rfl))

/-! ## `Wasm.encode` and `Wasm.decode` -/

/-- The canonical byte-list encoding of a module. -/
def encodeList (m : Module) : List UInt8 := moduleC.enc m

/-- The canonical binary encoding of a module. -/
def encode (m : Module) : ByteArray := Bytes.pack (encodeList m)

/-- The decoder.  Every failure path returns a typed `DecodeFault`. -/
def decode (b : ByteArray) : Except DecodeFault Module :=
  match moduleC.dec b.size b.toList with
  | .error e => .error e
  | .ok (m, r) => if r = [] then .ok m else .error (.trailingBytes r.length)

@[simp] theorem toList_encode (m : Module) : (encode m).toList = encodeList m :=
  Bytes.toList_pack _

@[simp] theorem size_encode (m : Module) : (encode m).size = (encodeList m).length :=
  Bytes.size_pack _

/-- **Round trip.**  Decoding an encoded module returns exactly that module. -/
theorem encode_decode_roundtrip (m : Module) : decode (encode m) = .ok m := by
  have hcost : moduleC.cost m ≤ (encodeList m).length := moduleC.cost_le m
  have hdec : moduleC.dec (encodeList m).length (encodeList m) = .ok (m, []) := by
    have h := moduleC.dec_enc m (encodeList m).length [] hcost
    simpa [encodeList] using h
  simp only [decode, size_encode, toList_encode, hdec]
  simp

/-- **Soundness.**  A byte string that decodes at all is the encoding of the
module it decodes to; a malformed byte string yields a typed `DecodeFault` and never
a wrong module.

This is deliberately *not* named `decode_sound`: the theorem of that name in
SPEC section 7.3 is stated against the vendored `DeclarativeBinaryRelation`,
which this layer does not define and therefore does not claim. -/
theorem decode_is_encode {b : ByteArray} {m : Module} (h : decode b = .ok m) :
    b = encode m := by
  revert h
  unfold decode
  cases hd : moduleC.dec b.size b.toList with
  | error e => intro h; exact absurd h (by simp)
  | ok q =>
    obtain ⟨m', r⟩ := q
    intro h
    by_cases hr : r = []
    · subst hr
      simp only [reduceIte, Except.ok.injEq] at h
      subst h
      have hs := moduleC.dec_sound _ _ _ _ hd
      apply Bytes.toList_injective
      simpa [encode, encodeList] using hs
    · simp [hr] at h

/-- Encoding is injective. -/
theorem encodeList_injective : Function.Injective encodeList :=
  moduleC.enc_injective

/-- Encoding is injective. -/
theorem encode_injective : Function.Injective encode := by
  intro a b h
  exact encodeList_injective (by simpa using congrArg ByteArray.toList h)

/-- Decoding is total in the strong sense required by SPEC section 7.3: every
byte string either produces a typed fault, or produces the unique module whose
encoding it is. -/
theorem decode_error_or_encode (b : ByteArray) :
    (∃ f : DecodeFault, decode b = .error f) ∨
    (∃ m : Module, decode b = .ok m ∧ b = encode m) := by
  cases h : decode b with
  | error e => exact Or.inl ⟨e, rfl⟩
  | ok m => exact Or.inr ⟨m, rfl, decode_is_encode h⟩

/-- Two byte strings that decode to the same module are equal. -/
theorem decode_injective {b c : ByteArray} {m : Module}
    (hb : decode b = .ok m) (hc : decode c = .ok m) : b = c := by
  rw [decode_is_encode hb, decode_is_encode hc]

/-- The preamble is the pinned magic and version. -/
theorem magicBytes_eq : magicBytes = [0, 0x61, 0x73, 0x6D, 1, 0, 0, 0] := rfl

/-- Every encoded module starts with the pinned preamble. -/
theorem encodeList_prefix (m : Module) :
    ∃ t : List UInt8, encodeList m = magicBytes ++ t :=
  ⟨_, rfl⟩

/-- The empty module round trips. -/
theorem decode_encode_empty : decode (encode Module.empty) = .ok Module.empty :=
  encode_decode_roundtrip Module.empty

/-! ## Further consequences -/

/-- The unsigned LEB128 encoder is exactly the canonical self-delimiting
natural-number encoder of `Foundation/Bytes.lean`. -/
theorem encodeULEB_eq_natBytes (n : Nat) : encodeULEB n = Bytes.natBytes n := rfl

/-- Instruction encoding is injective. -/
theorem Instr.enc_injective : Function.Injective Instr.enc := instrC.enc_injective

/-- Instruction-sequence encoding is injective. -/
theorem Expr.enc_injective : Function.Injective Expr.enc := exprC.enc_injective

/-- Instruction encoding is prefix-free. -/
theorem Instr.enc_prefixFree : Bytes.PrefixFree Instr.enc := instrC.enc_prefixFree

/-- Module encoding is prefix-free. -/
theorem encodeList_prefixFree : Bytes.PrefixFree encodeList := moduleC.enc_prefixFree

/-- A byte string that is not the encoding of any module cannot decode: the
decoder returns a typed fault.  This is the contrapositive of `decode_is_encode`
and is the precise sense in which a malformed input never yields a wrong
module. -/
theorem decode_error_of_not_encode (b : ByteArray) (h : ∀ m : Module, b ≠ encode m) :
    ∃ f : DecodeFault, decode b = .error f := by
  rcases decode_error_or_encode b with hf | ⟨m, _, hm⟩
  · exact hf
  · exact absurd hm (h m)

/-- A byte string that does not start with the pinned preamble is rejected with
`DecodeFault.badMagic`. -/
theorem decode_badMagic {b : ByteArray} (h : stripPrefix magicBytes b.toList = none) :
    decode b = .error .badMagic := by
  simp only [decode, moduleC, Codec.prefixed, h]

/-- The empty input is rejected with `DecodeFault.badMagic`. -/
theorem decode_empty : decode ByteArray.empty = .error .badMagic := by
  refine decode_badMagic ?_
  have h : ByteArray.empty.toList = [] := by
    simp [Bytes.byteArray_toList]
  rw [h]
  rfl

end WasmGemmGnaf.Wasm
