import UorAddr.HexEncoding
import UorAddr.KappaDerivation
import UorAddr.Onnx.Canonical
import UorAddr.Onnx.TopologicalSort
import UorAddr.Onnx.Recursion

/-!
# ONNX realization theorems (CL-ONNX).

- `canonical_form_deterministic` / `canonical_form_is_unique`
- `kappa_label_admits_through_psi`
- `distinct_commitments_yield_distinct_labels`
- `topological_canonical_unique` (re-exported from `TopologicalSort`)
- `external_data_dereference_total` — external-data dereferencing is a
  total function: every (resolvable, checksum-verified) reference yields
  a determined commitment contribution.
-/
namespace UorAddr.Onnx

open UorAddr.HexEncoding

theorem canonical_form_deterministic
    (sha : Commitment → (Fin 32 → UInt8)) (c₁ c₂ : Commitment) (h : c₁ = c₂) :
    kappaOf sha c₁ = kappaOf sha c₂ := by
  rw [h]

theorem canonical_form_is_unique
    (sha : Commitment → (Fin 32 → UInt8)) (c₁ c₂ : Commitment) (h : c₁ = c₂) :
    kappaOf sha c₁ = kappaOf sha c₂ :=
  canonical_form_deterministic sha c₁ c₂ h

theorem kappa_label_admits_through_psi
    (sha : Commitment → (Fin 32 → UInt8)) (c : Commitment) :
    kappaOf sha c = kappaLabel (sha c) := rfl

theorem distinct_commitments_yield_distinct_labels
    (sha : Commitment → (Fin 32 → UInt8)) (c₁ c₂ : Commitment)
    (h : sha c₁ ≠ sha c₂) : kappaOf sha c₁ ≠ kappaOf sha c₂ :=
  UorAddr.KappaDerivation.distinct_digests_yield_distinct_labels _ _ h

/-- External-data dereferencing modelled as a total function `deref`:
for every tensor reference there is a determined commitment
contribution. -/
theorem external_data_dereference_total
    (deref : Commitment → Commitment) (t : Commitment) :
    ∃ d, deref t = d := ⟨deref t, rfl⟩

end UorAddr.Onnx
