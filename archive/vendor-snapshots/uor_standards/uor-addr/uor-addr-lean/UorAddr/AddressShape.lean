import UOR.Enforcement
import UorAddr.HexEncoding

/-!
# Address Shape — Wire-Format Width and ASCII Composition

The κ-label `AddressLabel` is exactly 71 ASCII bytes:

  `b"sha256:"` (7 bytes) ‖ `hex_lower(digest)` (64 bytes)

This file models the κ-label as a `Fin 71 → UInt8` function. Width is
structural (carried by the type). Per-position claims reduce to
decidable case-splits.

Pinned conformance IDs:
- `CL-W01` — `address_label_width_is_seventy_one`
- `CL-W02` — `address_prefix_is_sha256_colon`
- `CL-W03` — `address_hex_digits_are_lowercase`
-/

namespace UorAddr.AddressShape

open UorAddr.HexEncoding

/-- The wire-format κ-label width (bytes). -/
def kappaLabelWidth : Nat := 71

/-- The 7-byte ASCII prefix `"sha256:"`. -/
def addressPrefix : Fin 7 → UInt8
  | ⟨0, _⟩ => 0x73  -- 's'
  | ⟨1, _⟩ => 0x68  -- 'h'
  | ⟨2, _⟩ => 0x61  -- 'a'
  | ⟨3, _⟩ => 0x32  -- '2'
  | ⟨4, _⟩ => 0x35  -- '5'
  | ⟨5, _⟩ => 0x36  -- '6'
  | ⟨6, _⟩ => 0x3a  -- ':'

/-- The κ-label as a fixed-width 71-byte function. Bytes 0..7 are the
ASCII prefix; bytes 7..71 are the lowercase-hex rendering of the
32-byte digest (high nibble of digest byte `k` at position `7 + 2k`,
low nibble at `7 + 2k + 1`). Spec mirror of the Rust loop in
`crates/uor-addr/src/resolvers.rs::AddressKInvariantResolver::resolve`. -/
def kappaLabel (digest : Fin 32 → UInt8) : Fin 71 → UInt8 := fun i =>
  if h : (i : Nat) < 7 then
    addressPrefix ⟨i.val, h⟩
  else
    let j := (i : Nat) - 7  -- 0..63
    let byteIdx : Fin 32 := ⟨j / 2, by omega⟩
    let nibble : UInt8 :=
      if j % 2 = 0 then digest byteIdx >>> 4
      else digest byteIdx &&& 0x0F
    hexLower nibble

/-- CL-W01 — the wire-format κ-label width is 71. Structurally
trivial: the κ-label is `Fin 71 → UInt8`, so its domain has 71
elements by typing. The named constant matches the Rust constant
`ADDRESS_LABEL_BYTES` in `crates/uor-addr/src/model.rs`. -/
theorem address_label_width_is_seventy_one :
    kappaLabelWidth = 71 := rfl

/-- CL-W02 — the first 7 bytes of any κ-label are the ASCII literal
`"sha256:"`. -/
theorem address_prefix_is_sha256_colon
    (digest : Fin 32 → UInt8) (i : Fin 71) (hi : (i : Nat) < 7) :
    kappaLabel digest i = addressPrefix ⟨i.val, hi⟩ := by
  unfold kappaLabel
  rw [dif_pos hi]

/-- Aux: at any position `i ∈ [7, 71)` the value `kappaLabel digest i`
is `hexLower n` for some `n < 16`. -/
theorem kappa_suffix_is_hex_of_nibble
    (digest : Fin 32 → UInt8) (i : Fin 71) (hi : 7 ≤ (i : Nat)) :
    ∃ n : UInt8, n < 16 ∧ kappaLabel digest i = hexLower n := by
  unfold kappaLabel
  have h : ¬ (i : Nat) < 7 := by omega
  rw [dif_neg h]
  by_cases hp : ((i : Nat) - 7) % 2 = 0
  · refine ⟨digest ⟨((i : Nat) - 7) / 2, by omega⟩ >>> 4, ?_, ?_⟩
    · exact UorAddr.HexEncoding.high_nibble_lt_sixteen _
    · simp only [hp, if_true]
  · refine ⟨digest ⟨((i : Nat) - 7) / 2, by omega⟩ &&& 0x0F, ?_, ?_⟩
    · exact UorAddr.HexEncoding.low_nibble_lt_sixteen _
    · simp only [hp, if_false]

/-- CL-W03 — every byte in the 64-hex suffix is in
`{'0'..'9', 'a'..'f'}`. -/
theorem address_hex_digits_are_lowercase
    (digest : Fin 32 → UInt8) (i : Fin 71) (hi : 7 ≤ (i : Nat)) :
    (0x30 ≤ kappaLabel digest i ∧ kappaLabel digest i ≤ 0x39) ∨
    (0x61 ≤ kappaLabel digest i ∧ kappaLabel digest i ≤ 0x66) := by
  obtain ⟨n, hn, hk⟩ := kappa_suffix_is_hex_of_nibble digest i hi
  rw [hk]
  exact UorAddr.HexEncoding.hex_lower_in_lowercase_hex_alphabet n hn

end UorAddr.AddressShape
