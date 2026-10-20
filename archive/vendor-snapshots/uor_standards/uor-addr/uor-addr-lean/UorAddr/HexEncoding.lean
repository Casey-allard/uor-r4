import UOR.Enforcement

/-!
# Hex Encoding — `[0, 16) → ASCII '0'..'9' | 'a'..'f'`

The κ-derivation maps a 32-byte SHA-256 digest to the 64-hex suffix of
the κ-label by lower-nibble + upper-nibble extraction followed by
`hexLower : UInt8 → UInt8` lookup. This file establishes that
`hexLower` is **injective on `[0, 16)`**: distinct nibbles map to
distinct ASCII bytes. Lifted byte-pair-wise, the result is that
distinct digests yield distinct κ-labels.

Pinned conformance IDs:
- `CL-H01` — `hex_lower_injective`
- `CL-H02` — `hex_byte_pair_roundtrip`

Division of labor:
- These Lean theorems pin the universal statement.
- The Rust implementation
  (`crates/uor-addr/src/resolvers.rs` `HEX_LOWER`) is the runtime
  encoder; `tests::conformance::hex_lower_table_matches_lean_spec`
  pins the byte-for-byte agreement with this Lean definition.
-/

namespace UorAddr.HexEncoding

/-- The 16-byte lowercase-hex lookup table mirrors the Rust constant
`HEX_LOWER` in `crates/uor-addr/src/resolvers.rs`. -/
def hexLower : UInt8 → UInt8
  | 0  => 0x30  -- '0'
  | 1  => 0x31  -- '1'
  | 2  => 0x32  -- '2'
  | 3  => 0x33  -- '3'
  | 4  => 0x34  -- '4'
  | 5  => 0x35  -- '5'
  | 6  => 0x36  -- '6'
  | 7  => 0x37  -- '7'
  | 8  => 0x38  -- '8'
  | 9  => 0x39  -- '9'
  | 10 => 0x61  -- 'a'
  | 11 => 0x62  -- 'b'
  | 12 => 0x63  -- 'c'
  | 13 => 0x64  -- 'd'
  | 14 => 0x65  -- 'e'
  | 15 => 0x66  -- 'f'
  | _  => 0x00  -- out-of-domain; never reached on a 4-bit input

/-- Decoder: inverse of `hexLower` on its valid output range. -/
def decodeNibble : UInt8 → UInt8
  | 0x30 => 0  | 0x31 => 1  | 0x32 => 2  | 0x33 => 3
  | 0x34 => 4  | 0x35 => 5  | 0x36 => 6  | 0x37 => 7
  | 0x38 => 8  | 0x39 => 9
  | 0x61 => 10 | 0x62 => 11 | 0x63 => 12 | 0x64 => 13
  | 0x65 => 14 | 0x66 => 15
  | _    => 16  -- out-of-domain sentinel

/-- CL-H01 — `hexLower` is injective on `[0, 16)`. Exhaustive
verification across the 256-pair lookup table — `decide` evaluates
`16 × 16 = 256` cases at compile time. -/
theorem hex_lower_injective :
    ∀ (a b : UInt8), a < 16 → b < 16 → hexLower a = hexLower b → a = b := by
  decide

/-- CL-H02 — `decodeNibble` round-trips `hexLower` on `[0, 16)`. -/
theorem hex_byte_pair_roundtrip :
    ∀ (n : UInt8), n < 16 → decodeNibble (hexLower n) = n := by
  decide

/-- The output of `hexLower` on `[0, 16)` lies in the ASCII set
`{'0'..'9', 'a'..'f'}`. -/
theorem hex_lower_in_lowercase_hex_alphabet :
    ∀ (n : UInt8), n < 16 →
      (0x30 ≤ hexLower n ∧ hexLower n ≤ 0x39) ∨
      (0x61 ≤ hexLower n ∧ hexLower n ≤ 0x66) := by
  decide

/-- For each byte `b : UInt8`, encoding its two nibbles yields a pair
in the lowercase-hex alphabet. -/
def encodeByte (b : UInt8) : UInt8 × UInt8 :=
  (hexLower (b >>> 4), hexLower (b &&& 0x0F))

/-- The high nibble of any UInt8 is `< 16`. -/
theorem high_nibble_lt_sixteen (b : UInt8) : b >>> 4 < 16 := by decide

/-- The low nibble of any UInt8 is `< 16`. -/
theorem low_nibble_lt_sixteen (b : UInt8) : b &&& 0x0F < 16 := by decide

/-- The two-byte encoding `encodeByte` is **injective on all of
`UInt8`** — exactly what lifts a distinct-digest claim to a
distinct-κ-label claim, byte position by byte position. -/
theorem encode_byte_injective :
    ∀ (a b : UInt8), encodeByte a = encodeByte b → a = b := by
  decide

end UorAddr.HexEncoding
