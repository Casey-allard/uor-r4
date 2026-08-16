import WasmGemmGnaf.Atlas.Update
import WasmGemmGnaf.Atlas.Seal
set_option autoImplicit false

/-!
# Atlas: full rebuild and rebuild equivalence (SPEC §12.5)

`Atlas.semanticRebuildBody` derives the complete state body from a declaration
base and nothing else.  SPEC §12.5 is explicit that this is the point:
"`contents` is exactly `state.body.declarationBase`, so rebuild equivalence has
no ambient input."

## Two consequences of "no ambient input"

1. **The accumulated delta root must be derivable.**  It is: see
   `Atlas.accumulatedDeltaRootOf` in `Atlas/Update.lean`.

2. **The scope identities cannot be recovered from the declarations.**  A
   declaration base does not name its own profile, problem or objective, so
   `semanticRebuildBody` — whose only input is the declaration base — must
   choose them.  It uses `Atlas.Scope.unscoped`.

   Both forms of the rebuild theorem are therefore proved:

   * `Atlas.incremental_eq_full_rebuild_scoped` — the general statement, for a
     rebuild carried out in the state's own scope.  This is the one to use.
   * `Atlas.incremental_eq_full_rebuild` — SPEC §12.5's literal statement,
     which additionally requires the state to be in the unscoped scope, because
     otherwise the two sides genuinely disagree on the three scope identities
     and the equation is false.  Stating it without that hypothesis would be a
     false theorem, so the hypothesis is stated rather than hidden.

Both require `Atlas.Coherent state.body`: a state whose recorded semantic half
is not the one its declarations derive cannot equal any rebuild, and
`Atlas.rebuild_coherent` / `Atlas.semanticApplyBody_coherent` show the
hypothesis is preserved by exactly the operations that produce states.
-/

namespace WasmGemmGnaf.Atlas

open WasmGemmGnaf.Foundation

/-! ## The unscoped scope -/

/-- The scope a rebuild uses when the declaration base does not name one. -/
def Scope.unscoped : Scope := ⟨nullId, nullId, nullId⟩

/-! ## `Atlas.semanticRebuildBody` (SPEC §12.5) -/

/-- The full rebuild in a given scope. -/
def semanticRebuildBodyWith (scope : Scope) (declarations : CanonicalDeclarationSet) :
    StateBody :=
  derivedBody scope declarations

/-- **SPEC §12.5**, `Atlas.semanticRebuildBody`: a total structurally recursive
function of the declaration base alone. -/
def semanticRebuildBody (declarations : CanonicalDeclarationSet) : StateBody :=
  semanticRebuildBodyWith Scope.unscoped declarations

@[simp] theorem semanticRebuildBodyWith_declarationBase (s : Scope)
    (d : CanonicalDeclarationSet) :
    (semanticRebuildBodyWith s d).declarationBase = d := rfl

@[simp] theorem semanticRebuildBody_declarationBase (d : CanonicalDeclarationSet) :
    (semanticRebuildBody d).declarationBase = d := rfl

@[simp] theorem semanticRebuildBody_scope (d : CanonicalDeclarationSet) :
    (semanticRebuildBody d).scope = Scope.unscoped := rfl

/-- A rebuilt body is coherent. -/
theorem semanticRebuildBodyWith_coherent (s : Scope) (d : CanonicalDeclarationSet) :
    Coherent (semanticRebuildBodyWith s d) := derivedBody_coherent s d

theorem semanticRebuildBody_coherent (d : CanonicalDeclarationSet) :
    Coherent (semanticRebuildBody d) := derivedBody_coherent _ d

/-- A rebuilt body records no optimizer conclusion (SPEC §12.1: optimization
certificates are constructed afterward). -/
theorem semanticRebuildBody_optimizer_empty (d : CanonicalDeclarationSet) :
    (semanticRebuildBody d).candidateFacts = ⟨[]⟩ ∧
    (semanticRebuildBody d).costSurfaces = ⟨[]⟩ ∧
    (semanticRebuildBody d).searchPartitions = ⟨[], ⟨[]⟩⟩ ∧
    (semanticRebuildBody d).lowerEnvelope = ⟨[], ⟨[]⟩⟩ ∧
    (semanticRebuildBody d).certificates = ⟨[], ⟨[]⟩⟩ :=
  derivedBody_optimizer_empty _ d

/-! ## Rebuild equivalence (SPEC §12.5) -/

/-- **The rebuild equation, unreduced.**  On a coherent state the incremental
update *is* the full rebuild in the state's own scope — not merely up to
canonical form. -/
theorem semanticApplyBody_eq_rebuild {b : StateBody} (hb : Coherent b) (d : Delta) :
    semanticApplyBody b d =
      semanticRebuildBodyWith b.scope (b.declarationBase ∪ d.declarations) :=
  semanticApplyBody_eq_derived hb d

/-- **SPEC §12.5**, `incremental_eq_full_rebuild`, general (scoped) form. -/
theorem incremental_eq_full_rebuild_scoped {budget : BuildBudget}
    {state : UnsealedState} {delta : Delta} {successor : UnsealedState}
    (hcoherent : Coherent state.body)
    (hupdate : (accumulate budget state delta).result = .complete successor) :
    canonicalize successor.body =
      canonicalize (semanticRebuildBodyWith state.body.scope
        (state.body.declarationBase ∪ delta.declarations)) := by
  rw [accumulate_complete_refines_semantic hupdate,
    semanticApplyBody_eq_rebuild hcoherent]

/-- **SPEC §12.5**, `incremental_eq_full_rebuild`, literal form.

The extra `hscope` hypothesis is not decoration: `semanticRebuildBody` has only
the declaration base as input and therefore cannot reproduce a scope the
declarations do not name.  Without it the statement is false. -/
theorem incremental_eq_full_rebuild {budget : BuildBudget}
    {state : UnsealedState} {delta : Delta} {successor : UnsealedState}
    (hcoherent : Coherent state.body)
    (hscope : state.body.scope = Scope.unscoped)
    (hupdate : (accumulate budget state delta).result = .complete successor) :
    canonicalize successor.body =
      canonicalize (semanticRebuildBody
        (state.body.declarationBase ∪ delta.declarations)) := by
  rw [incremental_eq_full_rebuild_scoped hcoherent hupdate, hscope]
  rfl

/-- The same equation before canonicalisation, which is strictly stronger. -/
theorem incremental_eq_full_rebuild_exact {budget : BuildBudget}
    {state : UnsealedState} {delta : Delta} {successor : UnsealedState}
    (hcoherent : Coherent state.body)
    (hupdate : (accumulate budget state delta).result = .complete successor) :
    successor.body =
      semanticRebuildBodyWith state.body.scope
        (state.body.declarationBase ∪ delta.declarations) := by
  rw [accumulate_complete_refines_semantic hupdate,
    semanticApplyBody_eq_rebuild hcoherent]

/-! ## The budgeted rebuild (SPEC §12.5) -/

/-- The exact number of derivation steps a rebuild performs: one canonical
identity, one object entry, one shape edge, one closure derivation, one
attention entry and one dependency edge per declaration, plus the accumulated
delta root. -/
def rebuildCost (declarations : CanonicalDeclarationSet) : BuildBudget where
  remainingSteps := 6 * declarations.declarations.length + 1
  remainingBytes :=
    declarations.declarations.foldr (fun x acc => x.size + acc) 0

/-- **SPEC §12.5**, `Atlas.rebuildSufficientBudget`: computed from the exact
input size, not postulated. -/
def rebuildSufficientBudget (declarations : CanonicalDeclarationSet) : BuildBudget :=
  rebuildCost declarations

theorem rebuildSufficientBudget_covers (declarations : CanonicalDeclarationSet) :
    BudgetCovers (rebuildSufficientBudget declarations) (rebuildCost declarations) :=
  BudgetCovers.refl _

/-- **SPEC §12.5**, `Atlas.rebuild`. -/
def rebuild (budget : BuildBudget) (declarations : CanonicalDeclarationSet) :
    BudgetedResult budget UnsealedState UpdateError :=
  if h : BudgetCovers budget (rebuildCost declarations) then
    completeResult budget (rebuildCost declarations) h
      (unsealedFromBody (semanticRebuildBody declarations))
  else
    exhaustedResult budget
      ⟨StateId (semanticRebuildBody declarations), budget.remainingSteps,
       (rebuildCost declarations).remainingSteps⟩

/-- **SPEC §12.5**, `rebuild_succeeds_at_computed_bound`. -/
theorem rebuild_succeeds_at_computed_bound (declarations : CanonicalDeclarationSet) :
    (rebuild (rebuildSufficientBudget declarations) declarations).result =
      .complete (unsealedFromBody (semanticRebuildBody declarations)) := by
  rw [rebuild, dif_pos (rebuildSufficientBudget_covers declarations)]
  rfl

/-- A rebuild that fits in its budget always produces the rebuilt body. -/
theorem rebuild_complete_iff (budget : BuildBudget)
    (declarations : CanonicalDeclarationSet) :
    (rebuild budget declarations).result =
      .complete (unsealedFromBody (semanticRebuildBody declarations)) ↔
      BudgetCovers budget (rebuildCost declarations) := by
  constructor
  · intro h
    by_cases hc : BudgetCovers budget (rebuildCost declarations)
    · exact hc
    · rw [rebuild, dif_neg hc] at h
      exact absurd h (by simp [exhaustedResult])
  · intro hc
    rw [rebuild, dif_pos hc]
    rfl

/-- The rebuilt state is coherent, so it is a legitimate starting point for the
next incremental update. -/
theorem rebuild_coherent (declarations : CanonicalDeclarationSet) :
    Coherent (unsealedFromBody (semanticRebuildBody declarations)).body :=
  semanticRebuildBody_coherent declarations

/-! ## A rebuilt state passes every structural seal check (SPEC §12.1)

The seven deterministic checkers of `Atlas/Certificate.lean` are *not* assumed
here: each one is evaluated on the rebuilt state against the core the rebuild
determines, and proved to return `true`.  This is what makes the query layer
non-vacuous — there really are sealed states to query.

The rebuilt state has an **empty optimizer half**, so the cover, envelope and
certificate-store checks pass over empty records.  That is a fact about the
rebuild, not a claim of optimality: see
`Atlas.universalCoverCompleteCheck_scope_blind` in `Atlas/CoverageScope.lean`
for exactly what an empty cover does *not* establish. -/

/-- The unsealed state a rebuild produces. -/
def derivedState (scope : Scope) (decls : CanonicalDeclarationSet) : UnsealedState :=
  unsealedFromBody (semanticRebuildBodyWith scope decls)

@[simp] theorem derivedState_body (scope : Scope) (decls : CanonicalDeclarationSet) :
    (derivedState scope decls).body = semanticRebuildBodyWith scope decls := rfl

/-- The seal core the rebuilt state determines: every root is recomputed from
the rebuilt body. -/
def derivedCore (scope : Scope) (decls : CanonicalDeclarationSet) : SealCore where
  stateId := StateId (semanticRebuildBodyWith scope decls)
  profileId := scope.profileId
  problemId := scope.problemId
  objectiveId := scope.objectiveId
  closureRoot := ⟨decls.declarations.map declId⟩
  attentionRoot := ⟨decls.declarations⟩
  dependencyRoot := ⟨decls.declarations.map declId⟩
  partitionCoverRoot := ⟨[]⟩
  envelopeRoot := ⟨[]⟩
  certificateRoot := ⟨[]⟩
  retentionRoot := (derivedState scope decls).retentionRoot
  baselineScore := 0

theorem derivedCore_binds (scope : Scope) (decls : CanonicalDeclarationSet) :
    (derivedCore scope decls).stateId = (derivedState scope decls).bodyId ∧
    (derivedCore scope decls).profileId = (derivedState scope decls).body.profileId ∧
    (derivedCore scope decls).problemId = (derivedState scope decls).body.problemId ∧
    (derivedCore scope decls).objectiveId = (derivedState scope decls).body.objectiveId :=
  ⟨rfl, rfl, rfl, rfl⟩

theorem derivedState_keys (scope : Scope) (decls : CanonicalDeclarationSet) :
    (derivedState scope decls).body.semanticObjects.keys =
      decls.declarations.map declId := by
  simp [derivedState, unsealedFromBody, semanticRebuildBodyWith, derivedBody,
    CanonicalObjectMap.keys, List.map_map, objectEntryOf]

theorem derivedState_closureLeastCheck (scope : Scope) (decls : CanonicalDeclarationSet) :
    closureLeastCheck (derivedState scope decls) (derivedCore scope decls) = true := by
  have hfacts : (derivedState scope decls).body.semanticClosure.facts =
      decls.declarations.map declId := rfl
  have hderiv : (derivedState scope decls).body.semanticClosure.derivations =
      decls.declarations.map derivationOf := rfl
  simp only [closureLeastCheck, Bool.and_eq_true, decide_eq_true_eq]
  refine ⟨⟨⟨rfl, rfl⟩, ?_⟩, ?_⟩
  · rw [hderiv, hfacts, List.all_eq_true]
    intro d hd
    obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hd
    have hsub : subsetId (derivationOf x).premises (decls.declarations.map declId) = true := by
      simp [derivationOf, subsetId]
    rw [hsub]
    simp only [Bool.not_true, Bool.false_or]
    exact (memId_iff _ _).mpr (List.mem_map_of_mem hx)
  · rw [hfacts, hderiv, List.all_eq_true]
    intro f hf
    obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hf
    rw [List.any_eq_true]
    exact ⟨derivationOf x, List.mem_map_of_mem hx, by simp [derivationOf, subsetId]⟩

theorem derivedState_attentionCompleteCheck (scope : Scope)
    (decls : CanonicalDeclarationSet) :
    attentionCompleteCheck (derivedState scope decls) (derivedCore scope decls) = true := by
  have hentries : (derivedState scope decls).body.attentionIndex.entries =
      decls.declarations.map attentionEntryOf := rfl
  have hkeys := derivedState_keys scope decls
  simp only [attentionCompleteCheck, Bool.and_eq_true, decide_eq_true_eq]
  refine ⟨⟨⟨rfl, ?_⟩, ?_⟩, ?_⟩
  · rw [hentries, List.map_map]
    simp only [Function.comp_def, attentionEntryOf]
    exact (List.map_id' decls.declarations).symm
  · rw [hentries, List.all_eq_true]
    intro e he
    obtain ⟨x, hx, rfl⟩ := List.mem_map.mp he
    rw [subsetId_iff, hkeys]
    intro y hy
    have hyx : y = declId x := by simpa [attentionEntryOf] using hy
    rw [hyx]
    exact List.mem_map_of_mem hx
  · rw [hkeys, List.all_eq_true]
    intro k hk
    obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hk
    rw [hentries, List.any_eq_true]
    exact ⟨attentionEntryOf x, List.mem_map_of_mem hx, by simp [attentionEntryOf, memId]⟩

theorem derivedState_dependenciesCompleteCheck (scope : Scope)
    (decls : CanonicalDeclarationSet) :
    dependenciesCompleteCheck (derivedState scope decls) (derivedCore scope decls) = true := by
  have hedges : (derivedState scope decls).body.dependencyGraph.edges =
      decls.declarations.map dependencyEdgeOf := rfl
  have hkeys := derivedState_keys scope decls
  simp only [dependenciesCompleteCheck, Bool.and_eq_true, decide_eq_true_eq]
  refine ⟨⟨⟨rfl, ?_⟩, ?_⟩, ?_⟩
  · rw [hedges, List.map_map]
    simp only [Function.comp_def, dependencyEdgeOf]
    rfl
  · rw [hedges, List.all_eq_true]
    intro e he
    obtain ⟨x, hx, rfl⟩ := List.mem_map.mp he
    simp [dependencyEdgeOf, subsetId]
  · rw [hkeys, List.all_eq_true]
    intro k hk
    obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hk
    rw [hedges, List.any_eq_true]
    exact ⟨dependencyEdgeOf x, List.mem_map_of_mem hx, by simp [dependencyEdgeOf]⟩

theorem derivedState_universalCoverCompleteCheck (scope : Scope)
    (decls : CanonicalDeclarationSet) :
    universalCoverCompleteCheck (derivedState scope decls) (derivedCore scope decls) = true := by
  simp [universalCoverCompleteCheck, derivedState_body, derivedCore,
    semanticRebuildBodyWith, derivedBody, PartitionMap.covered, distinctIds,
    subsetId, CandidateFactMap.keys]

theorem derivedState_envelopeExactCheck (scope : Scope)
    (decls : CanonicalDeclarationSet) :
    envelopeExactCheck (derivedState scope decls) (derivedCore scope decls) = true := by
  simp [envelopeExactCheck, derivedState_body, derivedCore, semanticRebuildBodyWith,
    derivedBody]

theorem derivedState_certificatesSoundCheck (scope : Scope)
    (decls : CanonicalDeclarationSet) :
    certificatesSoundCheck (derivedState scope decls) (derivedCore scope decls) = true := by
  simp [certificatesSoundCheck, derivedState_body, derivedCore, semanticRebuildBodyWith,
    derivedBody]

theorem derivedState_retentionCompleteCheck (scope : Scope)
    (decls : CanonicalDeclarationSet) :
    retentionCompleteCheck (derivedState scope decls) (derivedCore scope decls) = true := by
  rw [retentionCompleteCheck_iff]
  rfl

/-- **Every structural seal check passes on a rebuilt state.** -/
theorem derivedState_sealCheckResult (scope : Scope) (decls : CanonicalDeclarationSet)
    (tag : SealCheckTag) :
    sealCheckResult (derivedState scope decls) (derivedCore scope decls) tag = true := by
  cases tag with
  | closureLeast => exact derivedState_closureLeastCheck scope decls
  | attentionComplete => exact derivedState_attentionCompleteCheck scope decls
  | dependenciesComplete => exact derivedState_dependenciesCompleteCheck scope decls
  | universalCoverComplete => exact derivedState_universalCoverCompleteCheck scope decls
  | envelopeExact => exact derivedState_envelopeExactCheck scope decls
  | certificatesSound => exact derivedState_certificatesSoundCheck scope decls
  | retentionComplete => exact derivedState_retentionCompleteCheck scope decls

/-- The seal certificate of a rebuilt state: every field is discharged by a
proved checker result, none is assumed. -/
def derivedSealCertificate (scope : Scope) (decls : CanonicalDeclarationSet) :
    SealCertificate (derivedState scope decls) (derivedCore scope decls) where
  body := canonicalSealCertificateBody (derivedState scope decls) (derivedCore scope decls)
  bodyValid := verifies_canonicalSealCertificateBody _ _ rfl
  coreBindsState := derivedCore_binds scope decls
  closureLeast := derivedState_closureLeastCheck scope decls
  attentionComplete := derivedState_attentionCompleteCheck scope decls
  dependenciesComplete := derivedState_dependenciesCompleteCheck scope decls
  universalCoverComplete := derivedState_universalCoverCompleteCheck scope decls
  envelopeExact := derivedState_envelopeExactCheck scope decls
  certificatesSound := derivedState_certificatesSoundCheck scope decls
  retentionComplete := derivedState_retentionCompleteCheck scope decls

/-- **A rebuilt state really can be sealed.**  This is the witness that makes
the query layer of `Atlas/Query.lean` non-vacuous. -/
def derivedSealedState (scope : Scope) (decls : CanonicalDeclarationSet) : SealedState where
  state := derivedState scope decls
  core := derivedCore scope decls
  certificate := derivedSealCertificate scope decls
  sealId := SealId (derivedCore scope decls) (derivedSealCertificate scope decls).body
  sealIdEq := rfl

@[simp] theorem derivedSealedState_body (scope : Scope) (decls : CanonicalDeclarationSet) :
    (derivedSealedState scope decls).state.body = semanticRebuildBodyWith scope decls := rfl

@[simp] theorem derivedSealedState_core (scope : Scope) (decls : CanonicalDeclarationSet) :
    (derivedSealedState scope decls).core = derivedCore scope decls := rfl

end WasmGemmGnaf.Atlas
