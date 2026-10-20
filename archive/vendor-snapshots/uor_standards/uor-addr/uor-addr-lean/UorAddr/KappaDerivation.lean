import UOR.Enforcement
import UorAddr.HexEncoding
import UorAddr.AddressShape

/-!
# κ-Derivation — The Address-from-Digest Identity

ψ_9 emits the 71-byte κ-label as `"sha256:" ‖ hex_lower(digest)`. This
file pins the universal statement: **the κ-label is a function of the
digest, and the function is injective.**

Pinned conformance IDs:
- `CL-K01` — `kappa_determined_by_digest`
- `CL-K02` — `distinct_digests_yield_distinct_labels`

Together with `CL-A01` (algebraic-closure) and `CD-D02` (byte-identity
against the 12 reference fixtures), this gives the full
content-addressing soundness statement: every well-formed JSON input
maps to **one** κ-label, and the map is injective on canonical-form
byte sequences (modulo SHA-256 collision resistance).
-/

namespace UorAddr.KappaDerivation

open UorAddr.HexEncoding
open UorAddr.AddressShape

/-- CL-K01 — equal digests yield equal κ-labels. Function congruence. -/
theorem kappa_determined_by_digest
    (d₁ d₂ : Fin 32 → UInt8) (h : d₁ = d₂) :
    kappaLabel d₁ = kappaLabel d₂ := by
  rw [h]

/-- CL-K01' — pointwise version: equal digests agree at every label
position. -/
theorem kappa_determined_pointwise
    (d₁ d₂ : Fin 32 → UInt8) (h : d₁ = d₂) (i : Fin 71) :
    kappaLabel d₁ i = kappaLabel d₂ i := by
  rw [h]

/-- Aux: at any hex-suffix position `i = 7 + 2k`, the κ-label byte is
`hexLower (digest k >>> 4)`. -/
theorem kappa_high_nibble_at
    (digest : Fin 32 → UInt8) (k : Fin 32) :
    let i : Fin 71 := ⟨7 + 2 * k.val, by omega⟩
    kappaLabel digest i = hexLower (digest k >>> 4) := by
  unfold kappaLabel
  have h : ¬ (7 + 2 * k.val) < 7 := by omega
  rw [dif_neg h]
  have hmod : (7 + 2 * k.val - 7) % 2 = 0 := by omega
  have hdiv : (7 + 2 * k.val - 7) / 2 = k.val := by omega
  simp only [hmod, if_true]
  congr 1
  · simp only [hdiv]
    rfl

/-- Aux: at any hex-suffix position `i = 7 + 2k + 1`, the κ-label byte
is `hexLower (digest k &&& 0x0F)`. -/
theorem kappa_low_nibble_at
    (digest : Fin 32 → UInt8) (k : Fin 32) :
    let i : Fin 71 := ⟨7 + 2 * k.val + 1, by omega⟩
    kappaLabel digest i = hexLower (digest k &&& 0x0F) := by
  unfold kappaLabel
  have h : ¬ (7 + 2 * k.val + 1) < 7 := by omega
  rw [dif_neg h]
  have hmod : (7 + 2 * k.val + 1 - 7) % 2 = 1 := by omega
  have hdiv : (7 + 2 * k.val + 1 - 7) / 2 = k.val := by omega
  have hne : ¬ ((7 + 2 * k.val + 1 - 7) % 2 = 0) := by omega
  simp only [hne, if_false]
  congr 1
  · simp only [hdiv]
    rfl

/-- Aux: the byte-encoding `encodeByte` agrees with the high/low
nibble bytes extracted at positions `(7 + 2k, 7 + 2k + 1)`. -/
theorem kappa_encodes_byte
    (digest : Fin 32 → UInt8) (k : Fin 32) :
    let iHi : Fin 71 := ⟨7 + 2 * k.val, by omega⟩
    let iLo : Fin 71 := ⟨7 + 2 * k.val + 1, by omega⟩
    (kappaLabel digest iHi, kappaLabel digest iLo) = encodeByte (digest k) := by
  rw [kappa_high_nibble_at digest k, kappa_low_nibble_at digest k]
  rfl

/-- CL-K02 — distinct digests yield distinct κ-labels.

Strategy: by contradiction. If `kappaLabel d₁ = kappaLabel d₂` then at
every byte index `k`, the encoded byte pair agrees; by
`encode_byte_injective` (`CL-H02`) we get `d₁ k = d₂ k`, hence
`d₁ = d₂` by function extensionality. -/
theorem distinct_digests_yield_distinct_labels
    (d₁ d₂ : Fin 32 → UInt8) (h : d₁ ≠ d₂) :
    kappaLabel d₁ ≠ kappaLabel d₂ := by
  intro hk
  apply h
  funext k
  have h₁ := kappa_encodes_byte d₁ k
  have h₂ := kappa_encodes_byte d₂ k
  have heq : encodeByte (d₁ k) = encodeByte (d₂ k) := by
    rw [← h₁, ← h₂, hk]
  exact UorAddr.HexEncoding.encode_byte_injective _ _ heq

end UorAddr.KappaDerivation
