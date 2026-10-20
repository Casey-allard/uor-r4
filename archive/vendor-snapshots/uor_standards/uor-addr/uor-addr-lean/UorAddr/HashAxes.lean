/-!
# Per-σ-axis κ-label widths + the CBOR depth bound.

UOR-ADDR's κ-label is `<algorithm>:<lowercase-hex(digest)>`; its byte width
is `len(prefix) + 1 (':') + 2 * digestBytes`. The four 32-byte axes vary
only in prefix length; sha512 (foundation 0.5.2 generalized the resolver
tower over the fingerprint-width const generic) is a 64-byte digest.

Mirrors `crate::hash` (`AddrHash::LABEL_BYTES`), the per-axis
`AddressLabel{Sha256,Blake3,Sha3_256,Keccak256,Sha512}` output shapes
(`SITE_COUNT` = 71 / 71 / 73 / 74 / 135), the `AddrBounds` /
`AddrBounds64` site ceilings (74 / 135), and
`crate::cbor::shapes::bounds::MAX_CBOR_DEPTH`.

No mathlib — every identity is `rfl` / `decide`.
-/
namespace UorAddr.HashAxes

/-- κ-label byte width: `prefixLen + 1 (':') + 2 * digestBytes`. -/
def labelWidth (prefixLen digestBytes : Nat) : Nat :=
  prefixLen + 1 + 2 * digestBytes

/-- The four `Hasher<32>` axes emit a 32-byte digest. -/
def digestBytes : Nat := 32
/-- The sha512 axis emits a 64-byte digest (`Hasher<64>`). -/
def digestBytes64 : Nat := 64

/-- `len("sha256")`. -/
def sha256PrefixLen : Nat := 6
/-- `len("blake3")`. -/
def blake3PrefixLen : Nat := 6
/-- `len("sha3-256")`. -/
def sha3_256PrefixLen : Nat := 8
/-- `len("keccak256")`. -/
def keccak256PrefixLen : Nat := 9
/-- `len("sha512")`. -/
def sha512PrefixLen : Nat := 6

theorem sha256_label_width : labelWidth sha256PrefixLen digestBytes = 71 := rfl
theorem blake3_label_width : labelWidth blake3PrefixLen digestBytes = 71 := rfl
theorem sha3_256_label_width : labelWidth sha3_256PrefixLen digestBytes = 73 := rfl
theorem keccak256_label_width : labelWidth keccak256PrefixLen digestBytes = 74 := rfl
theorem sha512_label_width : labelWidth sha512PrefixLen digestBytes64 = 135 := rfl

/-- The `AddrBounds` site ceiling (`NERVE_SITES_MAX`) for the 32-byte axes. -/
def nerveSitesMax : Nat := 74
/-- The `AddrBounds64` site ceiling for the sha512 axis. -/
def nerveSitesMax64 : Nat := 135

/-- Every 32-byte axis's κ-label fits the `AddrBounds` ceiling (keccak256,
the widest at 74, is exactly at the bound); sha512 fits `AddrBounds64`. -/
theorem every_axis_fits_site_ceiling :
    labelWidth sha256PrefixLen digestBytes ≤ nerveSitesMax ∧
    labelWidth blake3PrefixLen digestBytes ≤ nerveSitesMax ∧
    labelWidth sha3_256PrefixLen digestBytes ≤ nerveSitesMax ∧
    labelWidth keccak256PrefixLen digestBytes ≤ nerveSitesMax ∧
    labelWidth sha512PrefixLen digestBytes64 ≤ nerveSitesMax64 := by
  decide

-- ── CBOR realization depth bound (RFC 8949 §4.2 recursive canonicalizer) ──

/-- Mirrors `crate::cbor::shapes::bounds::MAX_CBOR_DEPTH` — the
native-stack-safety guard on the recursive CBOR canonicalizer. -/
def maxCborDepth : Nat := 1024

/-- A CBOR data item of nesting depth `d` is admissible iff `d ≤ maxCborDepth`. -/
def cborDepthAdmissible (d : Nat) : Bool := d ≤ maxCborDepth

theorem cbor_depth_bound_is_strict (d : Nat) :
    cborDepthAdmissible d = true ↔ d ≤ maxCborDepth := by
  unfold cborDepthAdmissible
  exact decide_eq_true_iff

/-- Exactly-at-bound depth is admissible. -/
theorem cbor_at_bound_admissible : cborDepthAdmissible maxCborDepth = true := by
  decide

/-- Over-bound depth is inadmissible. -/
theorem cbor_over_bound_inadmissible :
    cborDepthAdmissible (maxCborDepth + 1) = false := by
  decide

end UorAddr.HashAxes
