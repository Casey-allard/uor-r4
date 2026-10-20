/-
  Exact schema binding for the scope-critical definitions.

  SPEC §1 and `authority/global-optimality-WGG-GO-1.json` require the conformance
  gate to compare the *compiled, unfolded* declaration bodies against the frozen
  authority schema, and say plainly that recording the implementation's own
  proposition without that comparison is insufficient.

  A name check cannot do that: a weaker or conditional proposition under the right
  name passes it. What follows spells each scope-critical definition out in full
  and closes the equivalence by `Iff.rfl`, so the binding is *definitional*. If
  anyone narrows a quantifier, drops a conjunct, adds a hypothesis, or swaps the
  competitor domain for a scoped predicate, these stop elaborating.

  This is the machine-checked half of the authority gate rule. `Tools/required.py`
  supplies the inventory half.
-/
import WasmGemmGnaf.Universal.Sublevel

set_option autoImplicit false

namespace WasmGemmGnaf.Conformance

open Universal

variable {P : Wasm.Profile} [Foundation.Fintype (Gemm.RawInvocation P)]

/--
**`GlobalOptimal` matches the frozen `WGG-GO-1` schema.**

Every conjunct of SPEC §1 is written out here rather than referenced. The
competitor quantifier is `∀ competitorBytes : ByteArray` in *both* clauses, with
no scope predicate; the lower-bound clause is the conjunction SPEC §1 gives, not
an implication; and the tie-break clause carries `CanonicalBytesLE`.

`Iff.rfl` is the whole proof, which is the point: it holds only while the
definition is *definitionally* this proposition.
-/
theorem globalOptimal_matches_authority_schema
    (S : Setting P) (D : Decider S)
    (O : Cost.ProperObjective P S.problem) (releasedBytes : ByteArray) :
    GlobalOptimal S D O releasedBytes ↔
      (ProfileValid P releasedBytes ∧
       SemanticCorrect S releasedBytes ∧
       SemanticWithinResources S releasedBytes ∧
       ∃ releasedEval : SystemEvaluation S releasedBytes,
         SystemEvaluationRel S D releasedBytes releasedEval ∧
         Correct releasedEval ∧
         Feasible releasedEval ∧
         (∀ competitorBytes : ByteArray,
            ProfileValid P competitorBytes →
            SemanticCorrect S competitorBytes →
            SemanticWithinResources S competitorBytes →
            ∀ competitorEval : SystemEvaluation S competitorBytes,
              SystemEvaluationRel S D competitorBytes competitorEval ∧
              O.score releasedEval.cost ≤ O.score competitorEval.cost) ∧
         (∀ competitorBytes : ByteArray,
            ProfileValid P competitorBytes →
            SemanticCorrect S competitorBytes →
            SemanticWithinResources S competitorBytes →
            ∀ competitorEval : SystemEvaluation S competitorBytes,
              SystemEvaluationRel S D competitorBytes competitorEval →
              O.score releasedEval.cost = O.score competitorEval.cost →
              Foundation.CanonicalBytesLE releasedBytes competitorBytes)) :=
  Iff.rfl

/-- **`ProfileValid` matches the frozen schema.**  SPEC §17.2 requires the gate to
reject an artifact-specific `ProfileValid`; this pins it to the four extensional
conditions of SPEC §10.1 and nothing else. -/
theorem profileValid_matches_authority_schema
    (profile : Wasm.Profile) (bytes : ByteArray) :
    ProfileValid profile bytes ↔
      (∃ module : Wasm.Module,
        Wasm.decode bytes = .ok module ∧
        Universal.validateUnder profile module = true ∧
        module.imports = [] ∧
        Universal.HasExactGemmExports profile module) :=
  Iff.rfl

/-- **The competitor domain is unrestricted.**  Extracted so the property the
audit cares about — that the quantifier ranges over every finite byte sequence —
is checkable on its own, without unfolding the whole proposition. -/
theorem globalOptimal_competitor_domain_unrestricted
    (S : Setting P) (D : Decider S)
    (O : Cost.ProperObjective P S.problem) {releasedBytes : ByteArray}
    (h : GlobalOptimal S D O releasedBytes) :
    ∀ competitorBytes : ByteArray,
      ProfileValid P competitorBytes →
      SemanticCorrect S competitorBytes →
      SemanticWithinResources S competitorBytes →
      ∀ competitorEval : SystemEvaluation S competitorBytes,
        SystemEvaluationRel S D competitorBytes competitorEval ∧
        O.score (Classical.choose h.2.2.2).cost ≤ O.score competitorEval.cost := by
  have hs := Classical.choose_spec h.2.2.2
  exact hs.2.2.2.1

end WasmGemmGnaf.Conformance
