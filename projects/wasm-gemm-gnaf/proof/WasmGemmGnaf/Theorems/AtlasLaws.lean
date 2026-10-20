/-
  Theorems: the UOR Atlas laws.

  This module is an INDEX.  Every declaration restates a proposition already
  proved in `WasmGemmGnaf/Atlas/` and closes it by `exact`-ing the original, so
  SPEC §15's required-name list is checkable against full statements in one
  place.

  ## SPEC §15 declarations discharged here

  | SPEC §15 name                     | discharged by                          |
  |-----------------------------------|----------------------------------------|
  | `Atlas.semantic_closure_least`    | `Theorems.semantic_closure_least`      |
  | `Atlas.incremental_eq_full_rebuild` | `Theorems.incremental_eq_full_rebuild` |

  `Atlas.incremental_eq_full_rebuild` is discharged with the extra hypothesis
  `state.body.scope = Scope.unscoped`.  That hypothesis is not decoration:
  `semanticRebuildBody` takes only the declaration base as input and cannot
  reproduce a scope the declarations do not name, so the literal SPEC statement
  is false without it.  The unrestricted content is discharged by
  `Theorems.incremental_eq_full_rebuild_scoped`, which is a *scoped* equation
  with no side condition, and strengthened past canonicalisation by
  `Theorems.incremental_eq_full_rebuild_exact`.

  ## Additional proved results indexed here (not on the §15 list)

  * `semantic_closure_eq_derivable` — the computed fixed point **is** the
    inductive derivation closure, not merely some fixed point above `A`.
  * `semantic_closure_merge` and `closure_merge_law` — UOR-GNAF Theorem 11.3,
    `Cl (A ∪ B) = Cl (Cl A ∪ Cl B)`, concretely and in the abstract form that
    assumes only extensivity, monotonicity and idempotence.
  * `update_empty_fixed_point` — the empty delta is a fixed point of the update
    operator, by `rfl`.
  * `seal_certificate_body_unique` — at most one seal certificate body verifies
    against a given state and core.
  * `universalCoverCompleteCheck_scope_blind` — a HARDENING result.  It proves
    the seal's cover check is a function of three recorded components only, and
    therefore cannot witness any proposition quantified over `ByteArray`.  This
    is the machine-checked statement of why `Atlas.seal_implies_universal_coverage`
    is *absent* from this repository rather than derived from the seal.

  ## SPEC §15 Atlas declarations that remain OUTSTANDING

  * `Atlas.attention_no_optimum_relevant_false_negative` — `Atlas/Attention.lean`
    proves determinism, monotonicity, index-determination and blindness to
    optimizer state, but no theorem says attention never misses an
    optimum-relevant object; that statement needs a notion of optimum, which
    does not exist here.  Blocking obligations: `O-3`, `O-5`.
  * `Atlas.seal_verifier_reconstructs_every_preimage` — `Atlas/Seal.lean` proves
    `resolvesEveryReferencedPreimage_iff` for *referenced* preimages; the §15
    name asserts more.  Blocking obligation: `O-3`.
  * `Atlas.seal_implies_universal_coverage` — deliberately absent and provably
    underivable from the recorded cover check; see
    `universalCoverCompleteCheck_scope_blind` below.  Blocking obligation:
    `O-5`.
  * `Atlas.lifecycle_prefix_conservation`, `Atlas.lifecycle_native_bound`,
    `Atlas.lifecycle_incremental_semantics_eq_full_rebuild`,
    `Atlas.lifecycle_full_rebuild_comparator_exact` — `Atlas/Lifecycle.lean`
    does not exist.  Blocking obligation: `O-6`.

  `Atlas.invalidation_complete` exists at its §15 name in
  `WasmGemmGnaf/Atlas/Dependency.lean` and is not re-indexed here.
-/
import WasmGemmGnaf.Atlas.SemanticClosure
import WasmGemmGnaf.Atlas.Update
import WasmGemmGnaf.Atlas.Rebuild
import WasmGemmGnaf.Atlas.Certificate
import WasmGemmGnaf.Atlas.CoverageScope

set_option autoImplicit false

namespace WasmGemmGnaf.Theorems

open WasmGemmGnaf.Atlas

/-! ## SPEC §12.1: the semantic closure is least -/

/-- **SPEC §15, `Atlas.semantic_closure_least`.**  The semantic closure is the
*least* set closed under the rule set and containing `A`: it is closed, it
contains `A`, and it is contained in every predicate with those two properties.

The third component quantifies over an arbitrary `Prop`-valued predicate, so no
decidability side condition weakens leastness.  This is what SPEC §12.1 demands
in place of a mere fixed point. -/
theorem semantic_closure_least (R : SemanticRuleSet) (A : SemanticFacts) :
    Closure.ClosedFacts R (semanticClosure R A) ∧
    Closure.FactSub A (semanticClosure R A) ∧
    (∀ S : SemanticJudgment → Prop, Closure.ClosedUnder R S →
      (∀ x, A x = true → S x) → ∀ x, semanticClosure R A x = true → S x) :=
  Atlas.semantic_closure_least R A

/-- The closure coincides with inductive derivability: the computed fixed point
**is** the derivation closure, not merely some fixed point above `A`. -/
theorem semantic_closure_eq_derivable (R : SemanticRuleSet) (A : SemanticFacts)
    (x : SemanticJudgment) :
    semanticClosure R A x = true ↔ Closure.Derivable R A x :=
  Atlas.semantic_closure_eq_derivable R A x

/-! ## UOR-GNAF Theorem 11.3: the merge law -/

/-- **UOR-GNAF Theorem 11.3, the merge law**: `Cl (A ∪ B) = Cl (Cl A ∪ Cl B)`.
Closing two fact sets separately and then closing their union loses nothing, so
an incremental build may close each contribution independently. -/
theorem semantic_closure_merge (R : SemanticRuleSet) (A B : SemanticFacts) :
    semanticClosure R (Closure.factUnion A B) =
      semanticClosure R
        (Closure.factUnion (semanticClosure R A) (semanticClosure R B)) :=
  Atlas.semantic_closure_merge R A B

/-- **UOR-GNAF Theorem 11.3, abstract form.**  The merge law needs *only*
extensivity, monotonicity and idempotence — no property of the particular rule
set, and no decidability of the operand. -/
theorem closure_merge_law {J : Type}
    (Cl' : Closure.FactSet J → Closure.FactSet J)
    (ext : ∀ A, Closure.FactSub A (Cl' A))
    (mono : ∀ A B, Closure.FactSub A B → Closure.FactSub (Cl' A) (Cl' B))
    (idem : ∀ A, Cl' (Cl' A) = Cl' A)
    (A B : Closure.FactSet J) :
    Cl' (Closure.factUnion A B) =
      Cl' (Closure.factUnion (Cl' A) (Cl' B)) :=
  Closure.closure_merge_law Cl' ext mono idem A B

/-! ## SPEC §12.5: incremental update equals full rebuild -/

/-- **SPEC §15, `Atlas.incremental_eq_full_rebuild`**, literal form.

On a coherent, unscoped state, a completed incremental update produces exactly
the canonical form of the full rebuild from the accumulated declaration base.

The `hscope` hypothesis is required for truth, not for convenience:
`semanticRebuildBody` receives only the declaration base and therefore cannot
reproduce a scope the declarations do not name. -/
theorem incremental_eq_full_rebuild {budget : BuildBudget}
    {state : UnsealedState} {delta : Delta} {successor : UnsealedState}
    (hcoherent : Coherent state.body)
    (hscope : state.body.scope = Scope.unscoped)
    (hupdate : (accumulate budget state delta).result = .complete successor) :
    canonicalize successor.body =
      canonicalize (semanticRebuildBody
        (state.body.declarationBase ∪ delta.declarations)) :=
  Atlas.incremental_eq_full_rebuild hcoherent hscope hupdate

/-- The same equation with no scope hypothesis, rebuilding in the state's own
scope.  This is the unrestricted content SPEC §12.5 intends. -/
theorem incremental_eq_full_rebuild_scoped {budget : BuildBudget}
    {state : UnsealedState} {delta : Delta} {successor : UnsealedState}
    (hcoherent : Coherent state.body)
    (hupdate : (accumulate budget state delta).result = .complete successor) :
    canonicalize successor.body =
      canonicalize (semanticRebuildBodyWith state.body.scope
        (state.body.declarationBase ∪ delta.declarations)) :=
  Atlas.incremental_eq_full_rebuild_scoped hcoherent hupdate

/-- The equation before canonicalisation, which is strictly stronger: the
incremental successor *is* the rebuild, not merely canonically equal to it. -/
theorem incremental_eq_full_rebuild_exact {budget : BuildBudget}
    {state : UnsealedState} {delta : Delta} {successor : UnsealedState}
    (hcoherent : Coherent state.body)
    (hupdate : (accumulate budget state delta).result = .complete successor) :
    successor.body =
      semanticRebuildBodyWith state.body.scope
        (state.body.declarationBase ∪ delta.declarations) :=
  Atlas.incremental_eq_full_rebuild_exact hcoherent hupdate

/-- **SPEC §12.5, `update_empty_fixed_point`.**  The empty delta changes
nothing: `accumulate` on it is definitionally the no-change result. -/
theorem update_empty_fixed_point (budget : BuildBudget) (state : UnsealedState) :
    accumulate budget state Delta.empty = noChangeResult budget state :=
  Atlas.update_empty_fixed_point budget state

/-! ## SPEC §12.1: seal certificate uniqueness -/

/-- **SPEC §12.1, required theorem.**  At most one seal certificate body can
verify against a given state and core, so a seal cannot be satisfied by two
different recorded bodies. -/
theorem seal_certificate_body_unique
    (state : UnsealedState) (core : SealCore) (a b : SealCertificateBody)
    (ha : VerifiesSealCertificateBody state core a)
    (hb : VerifiesSealCertificateBody state core b) : a = b :=
  Atlas.seal_certificate_body_unique state core a b ha hb

/-! ## Hardening: the cover check is blind to the byte universe -/

/--
**Scope blindness of the seal's cover check.**

`universalCoverCompleteCheck` is a function of exactly three recorded
components: the state's `searchPartitions`, the identities in its
`candidateFacts`, and the core's `partitionCoverRoot`.  Two states agreeing on
those three agree on the check — *whatever* byte strings exist, decode,
validate, or compute GEMM.

The consequence is the reason this lemma is indexed among the release theorems:
no proposition quantified over `ByteArray` can be derived from
`universalCoverCompleteCheck … = true` alone.  The check verifies that the
recorded cover is internally consistent with the recorded candidate facts; it
does not verify that those partitions denote every profile-valid byte string,
and it is satisfiable by a cover that records nothing at all.

`Atlas.seal_implies_universal_coverage` (SPEC §15) is therefore **absent** from
this repository rather than derived from the seal.  Deriving it from this check
would be unsound. -/
theorem universalCoverCompleteCheck_scope_blind
    (s₁ s₂ : UnsealedState) (core₁ core₂ : SealCore)
    (hp : s₁.body.searchPartitions = s₂.body.searchPartitions)
    (hc : s₁.body.candidateFacts.keys = s₂.body.candidateFacts.keys)
    (hr : core₁.partitionCoverRoot = core₂.partitionCoverRoot) :
    universalCoverCompleteCheck s₁ core₁ = universalCoverCompleteCheck s₂ core₂ :=
  Atlas.universalCoverCompleteCheck_scope_blind s₁ s₂ core₁ core₂ hp hc hr

end WasmGemmGnaf.Theorems
