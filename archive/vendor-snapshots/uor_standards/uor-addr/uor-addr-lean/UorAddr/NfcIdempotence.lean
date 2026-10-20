import UOR.Enforcement

/-!
# NFC Idempotence (Axiomatised, Spec-Pinned)

The Unicode Standard, UAX #15 §1.1, defines NFC as a normalisation
form. By specification it is **idempotent**:

  `∀ s : String, nfc (nfc s) = nfc s`

Proving this property from first principles requires the full Unicode
decomposition / composition tables — out of scope for this V&V layer.
We axiomatise the spec-level property and pin the implementation (the
`unicode-normalization` Rust crate consumed by
`crates/uor-addr/src/ops/canonicalize.rs`) at runtime via
[CP-N01](../../CONFORMANCE.md#cp--probabilistic-class--empirical-scaling)
over 100 000 randomly generated Unicode strings.

Pinned conformance IDs:
- `CL-N01` — `nfc_is_idempotent`
-/

namespace UorAddr.NfcIdempotence

/-- Abstract NFC operator. The concrete implementation lives in the
host-boundary transform `ops::canonicalize::jcs_nfc` and is verified
against this spec by `CP-N01`. -/
opaque nfc : String → String

/-- CL-N01 — NFC is idempotent (Unicode UAX #15 §1.1). -/
axiom nfc_is_idempotent : ∀ s : String, nfc (nfc s) = nfc s

/-- Iterated application converges in one step: any `n ≥ 1` rounds of
NFC equals one round. -/
theorem nfc_fixed_point (s : String) (n : Nat) (hn : n ≥ 1) :
    Nat.recOn n s (fun _ acc => nfc acc) = nfc s := by
  induction n with
  | zero => omega
  | succ m ih =>
    cases m with
    | zero => rfl
    | succ m' =>
      simp only [Nat.recOn]
      rw [ih (by omega)]
      exact nfc_is_idempotent s

end UorAddr.NfcIdempotence
