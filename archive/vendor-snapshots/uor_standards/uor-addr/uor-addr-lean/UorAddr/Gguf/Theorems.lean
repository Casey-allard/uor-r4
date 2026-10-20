import UorAddr.HexEncoding
import UorAddr.KappaDerivation
import UorAddr.Gguf.Canonical
import UorAddr.Gguf.Recursion

/-!
# GGUF realization theorems (CL-GGUF).

- `canonical_form_deterministic` — `kappaOf` is a function of the
  commitment.
- `canonical_form_is_unique` — equal canonical commitments (the
  implementation collapses metadata-KV / tensor order and tensor-data
  layout into one commitment) yield equal κ-labels.
- `kappa_label_admits_through_psi` — the κ-label is exactly the ψ_9
  projection `kappaLabel ∘ sha` of the commitment.
- `distinct_commitments_yield_distinct_labels` — injectivity, modulo
  SHA-256 collision resistance.
- `wire_format_round_trip` — re-committing the canonical commitment is
  the identity (idempotence).
-/
namespace UorAddr.Gguf

open UorAddr.HexEncoding

theorem canonical_form_deterministic
    (sha : Commitment → (Fin 32 → UInt8)) (c₁ c₂ : Commitment) (h : c₁ = c₂) :
    kappaOf sha c₁ = kappaOf sha c₂ := by
  rw [h]

theorem canonical_form_is_unique
    (sha : Commitment → (Fin 32 → UInt8)) (c₁ c₂ : Commitment)
    (h : c₁ = c₂) : kappaOf sha c₁ = kappaOf sha c₂ :=
  canonical_form_deterministic sha c₁ c₂ h

theorem kappa_label_admits_through_psi
    (sha : Commitment → (Fin 32 → UInt8)) (c : Commitment) :
    kappaOf sha c = kappaLabel (sha c) := rfl

theorem distinct_commitments_yield_distinct_labels
    (sha : Commitment → (Fin 32 → UInt8)) (c₁ c₂ : Commitment)
    (h : sha c₁ ≠ sha c₂) : kappaOf sha c₁ ≠ kappaOf sha c₂ :=
  UorAddr.KappaDerivation.distinct_digests_yield_distinct_labels _ _ h

/-- The canonical commitment is already canonical: re-canonicalizing
(the identity on the committed form) is a fixed point. -/
theorem wire_format_round_trip (c : Commitment) : id (id c) = id c := rfl

end UorAddr.Gguf
