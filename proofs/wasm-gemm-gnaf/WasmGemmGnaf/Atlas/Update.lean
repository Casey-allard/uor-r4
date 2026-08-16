import WasmGemmGnaf.Atlas.Delta
import WasmGemmGnaf.Foundation.Result
set_option autoImplicit false

/-!
# Atlas: the update operator (SPEC §12.5)

`Atlas.semanticApplyBody` is the total, structurally recursive mathematical
update of SPEC §12.5.  `Atlas.accumulate` is the budgeted executable refinement
of it, and `Atlas.BudgetedResult` carries the two conservation equations as
**proof fields**, so no construction site can claim a budget it did not respect:
`stepsConservation` and `bytesConservation` are discharged from `Nat.zero_add`
and `Nat.add_sub_cancel'` at every constructor, never assumed.

## Shape of the update

`semanticApplyBody b d` is a *homomorphism on the declaration base*: it appends
the derived content of the genuinely new declarations to the existing content,

    semanticObjects := b.semanticObjects ++ new.map objectEntryOf
    …

and does not touch anything else in the semantic half.  That is what makes
`Atlas.incremental_eq_full_rebuild` (in `Atlas/Rebuild.lean`) provable from
`List.map_append` rather than by definitional unfolding of a
recompute-everything function.

## Invalidation

SPEC §12.5 step 5 requires every affected optimization fact to be invalidated
before a new seal.  A delta that declares anything therefore clears the whole
optimizer half (candidate facts, cost surfaces, partitions, envelope,
certificate store).  This is the conservative direction: it can only *discard*
evidence, never retain stale evidence, and it is exactly what makes the applied
state coherent again.  A delta that declares nothing changes nothing at all,
which is `Atlas.update_empty_fixed_point`.
-/

namespace WasmGemmGnaf.Atlas

open WasmGemmGnaf.Foundation

/-! ## Scope -/

/-- The three scope identities of a state (SPEC §12.1). -/
structure Scope where
  objectiveId : ObjectiveId
  profileId : ProfileId
  problemId : ProblemId
  deriving DecidableEq

/-- The scope of a state body. -/
def StateBody.scope (b : StateBody) : Scope :=
  ⟨b.objectiveId, b.profileId, b.problemId⟩

/-! ## The derived semantic content of a declaration set -/

/-- The accumulated delta root of a declaration base: the identity of the single
canonical delta equivalent to everything applied so far.

SPEC §12.1 leaves the content of `DeltaRoot` abstract.  This choice is forced by
SPEC §12.5: rebuild equivalence has "no ambient input", so every field of the
rebuilt body — including this root — must be a function of the declaration base
alone.  It is taken over the *canonical* declaration list, so it is independent
of the order in which the deltas arrived. -/
def accumulatedDeltaRootOf (decls : CanonicalDeclarationSet) : DeltaRoot :=
  ⟨[DeltaId ⟨⟨Canon.norm declarationKey decls.declarations⟩⟩]⟩

/-- Declaration bases with the same declarations have the same accumulated
delta root. -/
theorem accumulatedDeltaRootOf_congr {d₁ d₂ : CanonicalDeclarationSet}
    (h : ∀ x, x ∈ d₁.declarations ↔ x ∈ d₂.declarations) :
    accumulatedDeltaRootOf d₁ = accumulatedDeltaRootOf d₂ := by
  simp only [accumulatedDeltaRootOf, Canon.norm_eq_of_mem_iff declarationKey_injective h]

/-- **The derived body**: the complete semantic half of a state, determined by
its scope and its declaration base, with an empty optimizer half.

SPEC §12.1: "Semantic facts are computed first; snapshot identity is fixed;
optimization certificates are constructed afterward."  A derived body therefore
records no candidate, cost, partition, envelope or certificate. -/
def derivedBody (scope : Scope) (decls : CanonicalDeclarationSet) : StateBody where
  declarationBase := decls
  accumulatedDeltaRoot := accumulatedDeltaRootOf decls
  semanticObjects := ⟨decls.declarations.map objectEntryOf⟩
  shapeEdges := ⟨decls.declarations.map hyperEdgeOf⟩
  semanticClosure :=
    ⟨decls.declarations.map declId,
     decls.declarations.map derivationOf,
     ⟨decls.declarations.map declId⟩⟩
  attentionIndex :=
    ⟨decls.declarations.map attentionEntryOf, ⟨decls.declarations⟩⟩
  dependencyGraph :=
    ⟨decls.declarations.map dependencyEdgeOf, ⟨decls.declarations.map declId⟩⟩
  candidateFacts := ⟨[]⟩
  costSurfaces := ⟨[]⟩
  searchPartitions := ⟨[], ⟨[]⟩⟩
  lowerEnvelope := ⟨[], ⟨[]⟩⟩
  certificates := ⟨[], ⟨[]⟩⟩
  objectiveId := scope.objectiveId
  profileId := scope.profileId
  problemId := scope.problemId

@[simp] theorem derivedBody_declarationBase (s : Scope) (d : CanonicalDeclarationSet) :
    (derivedBody s d).declarationBase = d := rfl

@[simp] theorem derivedBody_scope (s : Scope) (d : CanonicalDeclarationSet) :
    (derivedBody s d).scope = s := rfl

/-- The optimizer half of a derived body is empty: no optimizer conclusion is
ever a premise of the semantic half (SPEC §12.1). -/
theorem derivedBody_optimizer_empty (s : Scope) (d : CanonicalDeclarationSet) :
    (derivedBody s d).candidateFacts = ⟨[]⟩ ∧
    (derivedBody s d).costSurfaces = ⟨[]⟩ ∧
    (derivedBody s d).searchPartitions = ⟨[], ⟨[]⟩⟩ ∧
    (derivedBody s d).lowerEnvelope = ⟨[], ⟨[]⟩⟩ ∧
    (derivedBody s d).certificates = ⟨[], ⟨[]⟩⟩ :=
  ⟨rfl, rfl, rfl, rfl, rfl⟩

/-- A body is **coherent** when it is exactly the body derived from its own
scope and declaration base. -/
def Coherent (b : StateBody) : Prop := b = derivedBody b.scope b.declarationBase

theorem derivedBody_coherent (s : Scope) (d : CanonicalDeclarationSet) :
    Coherent (derivedBody s d) := rfl

/-! ## `Atlas.semanticApplyBody` (SPEC §12.5) -/

/-- The body obtained by appending the derived content of `new` to `b` and
invalidating the optimizer half.  Every semantic component is an append: this is
the homomorphism. -/
def appliedBody (b : StateBody) (new : List ByteArray) : StateBody where
  declarationBase := ⟨b.declarationBase.declarations ++ new⟩
  accumulatedDeltaRoot := accumulatedDeltaRootOf ⟨b.declarationBase.declarations ++ new⟩
  semanticObjects := ⟨b.semanticObjects.entries ++ new.map objectEntryOf⟩
  shapeEdges := ⟨b.shapeEdges.edges ++ new.map hyperEdgeOf⟩
  semanticClosure :=
    ⟨b.semanticClosure.facts ++ new.map declId,
     b.semanticClosure.derivations ++ new.map derivationOf,
     ⟨b.semanticClosure.root.factIds ++ new.map declId⟩⟩
  attentionIndex :=
    ⟨b.attentionIndex.entries ++ new.map attentionEntryOf,
     ⟨b.attentionIndex.root.signatures ++ new⟩⟩
  dependencyGraph :=
    ⟨b.dependencyGraph.edges ++ new.map dependencyEdgeOf,
     ⟨b.dependencyGraph.root.edgeIds ++ new.map declId⟩⟩
  candidateFacts := ⟨[]⟩
  costSurfaces := ⟨[]⟩
  searchPartitions := ⟨[], ⟨[]⟩⟩
  lowerEnvelope := ⟨[], ⟨[]⟩⟩
  certificates := ⟨[], ⟨[]⟩⟩
  objectiveId := b.objectiveId
  profileId := b.profileId
  problemId := b.problemId

@[simp] theorem appliedBody_declarationBase (b : StateBody) (new : List ByteArray) :
    (appliedBody b new).declarationBase = ⟨b.declarationBase.declarations ++ new⟩ := rfl

@[simp] theorem appliedBody_scope (b : StateBody) (new : List ByteArray) :
    (appliedBody b new).scope = b.scope := rfl

/-- Appending nothing to an already-applied body changes nothing. -/
theorem appliedBody_nil_applied (b : StateBody) (new : List ByteArray) :
    appliedBody (appliedBody b new) [] = appliedBody b new := by
  simp [appliedBody]

/-- Applying to a derived body is deriving from the extended base: **the
homomorphism on the declaration base**. -/
theorem appliedBody_derived (s : Scope) (decls : CanonicalDeclarationSet)
    (new : List ByteArray) :
    appliedBody (derivedBody s decls) new =
      derivedBody s ⟨decls.declarations ++ new⟩ := by
  simp [appliedBody, derivedBody, List.map_append]

/-- **SPEC §12.5**, `Atlas.semanticApplyBody`.

A total function of finite canonical inputs.  It appends the derived content of
the genuinely new declarations (a homomorphism on the declaration base) and
invalidates the optimizer half whenever the delta declares anything. -/
def semanticApplyBody (b : StateBody) (d : Delta) : StateBody :=
  if d.declarations.declarations.isEmpty then b
  else appliedBody b (newDeclarations b.declarationBase d)

/-- A delta that declares nothing changes nothing. -/
theorem semanticApplyBody_trivial {b : StateBody} {d : Delta}
    (h : d.declarations.declarations.isEmpty = true) : semanticApplyBody b d = b := by
  simp [semanticApplyBody, h]

theorem semanticApplyBody_nontrivial {b : StateBody} {d : Delta}
    (h : ¬ d.declarations.declarations.isEmpty = true) :
    semanticApplyBody b d = appliedBody b (newDeclarations b.declarationBase d) := by
  rw [semanticApplyBody, if_neg h]

@[simp] theorem semanticApplyBody_empty (b : StateBody) :
    semanticApplyBody b Delta.empty = b := rfl

/-- A delta whose declaration list is empty is `Delta.empty`. -/
theorem declarations_eq_nil {d : Delta} (h : d.declarations.declarations.isEmpty = true) :
    d.declarations = ⟨[]⟩ := by
  cases hd : d.declarations with
  | mk l =>
    cases l with
    | nil => rfl
    | cons x xs => rw [hd] at h; exact absurd h (by simp)

/-- The declaration base of an update is the union of the base and the delta. -/
theorem semanticApplyBody_declarationBase (b : StateBody) (d : Delta) :
    (semanticApplyBody b d).declarationBase = b.declarationBase ∪ d.declarations := by
  by_cases h : d.declarations.declarations.isEmpty = true
  · rw [semanticApplyBody_trivial h, declarations_eq_nil h,
      CanonicalDeclarationSet.union_nil]
  · rw [semanticApplyBody_nontrivial h]
    rfl

/-- The scope survives every update. -/
@[simp] theorem semanticApplyBody_scope (b : StateBody) (d : Delta) :
    (semanticApplyBody b d).scope = b.scope := by
  rw [semanticApplyBody]; split <;> rfl

/-- **SPEC §12.5**, `semantic_update_idempotent`: applying the same delta twice
is applying it once. -/
theorem semantic_update_idempotent (b : StateBody) (d : Delta) :
    semanticApplyBody (semanticApplyBody b d) d = semanticApplyBody b d := by
  by_cases h : d.declarations.declarations.isEmpty = true
  · rw [semanticApplyBody_trivial h, semanticApplyBody_trivial h]
  · have hnew : newDeclarations (semanticApplyBody b d).declarationBase d = [] := by
      rw [semanticApplyBody_declarationBase, newDeclarations]
      exact CanonicalDeclarationSet.newIn_union_self _ _
    rw [semanticApplyBody_nontrivial h, hnew, semanticApplyBody_nontrivial h,
      appliedBody_nil_applied]

/-! ## The budget algebra (SPEC §12.5) -/

/-- **SPEC §12.5**, `Atlas.BuildBudget`. -/
structure BuildBudget where
  remainingSteps : Nat
  remainingBytes : Nat
  deriving DecidableEq

/-- One budget covers another when it has at least as much of each resource. -/
def BudgetCovers (available required : BuildBudget) : Prop :=
  required.remainingSteps ≤ available.remainingSteps ∧
    required.remainingBytes ≤ available.remainingBytes

instance (a r : BuildBudget) : Decidable (BudgetCovers a r) := by
  unfold BudgetCovers; infer_instance

theorem BudgetCovers.refl (a : BuildBudget) : BudgetCovers a a :=
  ⟨Nat.le_refl _, Nat.le_refl _⟩

/-- The budget left after spending `cost`. -/
def BuildBudget.spend (initial cost : BuildBudget) : BuildBudget :=
  ⟨initial.remainingSteps - cost.remainingSteps,
   initial.remainingBytes - cost.remainingBytes⟩

/-- The errors an update can reject with. -/
inductive UpdateError
  | declarationRejected (declaration : CanonicalObjectId)
  | incompatibleDeltas (left right : CanonicalObjectId)
  deriving DecidableEq

/-- **SPEC §12.5**, `BudgetedResult`.

`stepsConservation` and `bytesConservation` are proof fields: a budgeted result
cannot be built without proving that what was consumed plus what remains is
exactly what was available. -/
structure BudgetedResult (initial : BuildBudget) (α ε : Type) where
  result : CheckResult α ε
  consumedSteps : Nat
  consumedBytes : Nat
  remainder : BuildBudget
  stepsConservation : consumedSteps + remainder.remainingSteps = initial.remainingSteps
  bytesConservation : consumedBytes + remainder.remainingBytes = initial.remainingBytes

namespace BudgetedResult

variable {α ε : Type}

/-- Conservation, restated: a budgeted result never consumes more than it had. -/
theorem consumedSteps_le {initial : BuildBudget} (r : BudgetedResult initial α ε) :
    r.consumedSteps ≤ initial.remainingSteps :=
  r.stepsConservation ▸ Nat.le_add_right _ _

theorem consumedBytes_le {initial : BuildBudget} (r : BudgetedResult initial α ε) :
    r.consumedBytes ≤ initial.remainingBytes :=
  r.bytesConservation ▸ Nat.le_add_right _ _

/-- Conservation, restated: the remainder never exceeds the initial budget. -/
theorem remainder_le {initial : BuildBudget} (r : BudgetedResult initial α ε) :
    BudgetCovers initial r.remainder :=
  ⟨r.stepsConservation ▸ Nat.le_add_left _ _, r.bytesConservation ▸ Nat.le_add_left _ _⟩

end BudgetedResult

/-- **SPEC §12.5**, `Atlas.noChangeResult`: consume nothing, change nothing. -/
def noChangeResult {α ε : Type} (budget : BuildBudget) (value : α) :
    BudgetedResult budget α ε where
  result := .complete value
  consumedSteps := 0
  consumedBytes := 0
  remainder := budget
  stepsConservation := Nat.zero_add _
  bytesConservation := Nat.zero_add _

/-- A successful budgeted step: the conservation equations are discharged by
`Nat.add_sub_cancel'` from the covering proof. -/
def completeResult {α ε : Type} (budget cost : BuildBudget)
    (h : BudgetCovers budget cost) (value : α) : BudgetedResult budget α ε where
  result := .complete value
  consumedSteps := cost.remainingSteps
  consumedBytes := cost.remainingBytes
  remainder := budget.spend cost
  stepsConservation := Nat.add_sub_cancel' h.1
  bytesConservation := Nat.add_sub_cancel' h.2

/-- An exhausted budget: nothing is consumed and nothing changes. -/
def exhaustedResult {α ε : Type} (budget : BuildBudget) (report : ResourceReport) :
    BudgetedResult budget α ε where
  result := .resourceExhausted report
  consumedSteps := 0
  consumedBytes := 0
  remainder := budget
  stepsConservation := Nat.zero_add _
  bytesConservation := Nat.zero_add _

/-! ## Retention: an unsealed state from a body -/

namespace ObjectGraph

/-- The retained graph that keeps exactly the canonical preimage of each listed
identity.  The preimage of a canonical identity is its own canonical body
byte string, so this graph is faithful, not a placeholder. -/
def ofIds (ids : List CanonicalObjectId) : ObjectGraph :=
  ⟨ids.map (fun id => ⟨id, id.canonicalBodyBytes⟩)⟩

/-- Every retained preimage really is the preimage of its identity. -/
theorem ofIds_faithful (ids : List CanonicalObjectId) :
    ∀ e ∈ (ofIds ids).preimages, e.preimageBytes = e.objectId.canonicalBodyBytes := by
  intro e he
  simp only [ofIds, List.mem_map] at he
  obtain ⟨id, _, rfl⟩ := he
  rfl

theorem ofIds_resolves {ids : List CanonicalObjectId} {id : CanonicalObjectId}
    (h : id ∈ ids) : (ofIds ids).resolves id = true := by
  rw [resolves_iff]
  exact ⟨⟨id, id.canonicalBodyBytes⟩, List.mem_map_of_mem h, rfl⟩

end ObjectGraph

/-- The unsealed state of a body, retaining exactly the canonical preimage of
every identity the body references. -/
def unsealedFromBody (b : StateBody) : UnsealedState where
  body := b
  bodyId := StateId b
  bodyIdEq := rfl
  retainedObjects := ⟨ObjectGraph.ofIds b.referencedObjects,
    fun _ hid => ObjectGraph.ofIds_resolves hid⟩

@[simp] theorem unsealedFromBody_body (b : StateBody) : (unsealedFromBody b).body = b := rfl

theorem unsealedFromBody_congr {b₁ b₂ : StateBody} (h : b₁ = b₂) :
    unsealedFromBody b₁ = unsealedFromBody b₂ := by rw [h]

/-! ## `Atlas.accumulate` (SPEC §12.5) -/

/-- The exact cost of applying a delta: one recognition step per incoming
declaration against the current base, plus one derivation step per genuinely new
declaration, and the exact bytes of the newly stored declarations. -/
def applyCost (b : StateBody) (d : Delta) : BuildBudget where
  remainingSteps :=
    (b.declarationBase.declarations.length + 1) * d.declarations.declarations.length +
      (newDeclarations b.declarationBase d).length
  remainingBytes :=
    (newDeclarations b.declarationBase d).foldr (fun x acc => x.size + acc) 0

@[simp] theorem applyCost_empty (b : StateBody) :
    applyCost b Delta.empty = ⟨0, 0⟩ := rfl

/-- **SPEC §12.5**, `Atlas.replayRecognitionCost`: the cost of replaying a delta
that has already been applied. -/
def replayRecognitionCost (s : UnsealedState) (d : Delta) : BuildBudget :=
  applyCost s.body d

/-- The state-level update: the trivial delta leaves the state, including its
retained graph, exactly as it was. -/
def applyState (s : UnsealedState) (d : Delta) : UnsealedState :=
  if d.declarations.declarations.isEmpty then s
  else unsealedFromBody (semanticApplyBody s.body d)

theorem applyState_body (s : UnsealedState) (d : Delta) :
    (applyState s d).body = semanticApplyBody s.body d := by
  rw [applyState, semanticApplyBody]
  split <;> rfl

@[simp] theorem applyState_empty (s : UnsealedState) : applyState s Delta.empty = s := rfl

/-- **SPEC §12.5**, `Atlas.accumulate`. -/
def accumulate (budget : BuildBudget) (state : UnsealedState) (delta : Delta) :
    BudgetedResult budget UnsealedState UpdateError :=
  if h : BudgetCovers budget (applyCost state.body delta) then
    completeResult budget (applyCost state.body delta) h (applyState state delta)
  else
    exhaustedResult budget
      ⟨state.bodyId, budget.remainingSteps,
       (applyCost state.body delta).remainingSteps⟩

/-- **SPEC §12.5**, `update_empty_fixed_point`. -/
theorem update_empty_fixed_point (budget : BuildBudget) (state : UnsealedState) :
    accumulate budget state Delta.empty = noChangeResult budget state := rfl

/-- `accumulate` either completes or reports an exhausted budget; it never
rejects, and never returns `unresolved`, `unsupported` or `internalFailure`.
This is a *scope* statement: the update operator recognises everything the
restricted additive delta grammar can express, so `UpdateError` is unreachable
from `accumulate`. -/
theorem accumulate_result_dichotomy (budget : BuildBudget) (state : UnsealedState)
    (delta : Delta) :
    (accumulate budget state delta).result = .complete (applyState state delta) ∨
    ∃ report, (accumulate budget state delta).result = .resourceExhausted report := by
  rw [accumulate]
  split
  · exact Or.inl rfl
  · exact Or.inr ⟨_, rfl⟩

theorem accumulate_never_rejects (budget : BuildBudget) (state : UnsealedState)
    (delta : Delta) (e : UpdateError) :
    (accumulate budget state delta).result ≠ .rejected e := by
  intro hc
  rcases accumulate_result_dichotomy budget state delta with h | ⟨report, h⟩ <;>
    rw [h] at hc <;> simp at hc

/-- `accumulate` succeeds exactly when the budget covers the exact cost. -/
theorem accumulate_complete_iff (budget : BuildBudget) (state : UnsealedState)
    (delta : Delta) :
    (accumulate budget state delta).result = .complete (applyState state delta) ↔
      BudgetCovers budget (applyCost state.body delta) := by
  constructor
  · intro h
    by_cases hc : BudgetCovers budget (applyCost state.body delta)
    · exact hc
    · rw [accumulate, dif_neg hc] at h
      exact absurd h (by simp [exhaustedResult])
  · intro hc
    rw [accumulate, dif_pos hc]
    rfl

/-- **SPEC §12.5**, `accumulate_complete_refines_semantic`: the budgeted
executable update refines the mathematical one. -/
theorem accumulate_complete_refines_semantic {budget : BuildBudget}
    {state : UnsealedState} {delta : Delta} {successor : UnsealedState}
    (hupdate : (accumulate budget state delta).result = .complete successor) :
    successor.body = semanticApplyBody state.body delta := by
  rw [accumulate] at hupdate
  split at hupdate
  · have : applyState state delta = successor := by
      simpa [completeResult] using hupdate
    rw [← this, applyState_body]
  · exact absurd hupdate (by simp [exhaustedResult])

/-- A completed update returns exactly `applyState`. -/
theorem accumulate_complete_eq {budget : BuildBudget} {state : UnsealedState}
    {delta : Delta} {successor : UnsealedState}
    (hupdate : (accumulate budget state delta).result = .complete successor) :
    successor = applyState state delta := by
  rw [accumulate] at hupdate
  split at hupdate
  · have heq : applyState state delta = successor := by
      simpa [completeResult] using hupdate
    exact heq.symm
  · exact absurd hupdate (by simp [exhaustedResult])

/-- **SPEC §12.5**, `update_idempotent`: replaying a delta on the successor it
produced, with a budget that covers the replay cost, reproduces the successor
exactly. -/
theorem update_idempotent {budget : BuildBudget} {state : UnsealedState}
    {delta : Delta} {successor : UnsealedState}
    (hcomplete : (accumulate budget state delta).result = .complete successor)
    (hreplay : BudgetCovers (accumulate budget state delta).remainder
      (replayRecognitionCost successor delta)) :
    (accumulate (accumulate budget state delta).remainder successor delta).result =
      .complete successor := by
  have hcov : BudgetCovers (accumulate budget state delta).remainder
      (applyCost successor.body delta) := hreplay
  rw [(accumulate_complete_iff (accumulate budget state delta).remainder successor
    delta).mpr hcov]
  refine congrArg CheckResult.complete ?_
  have hsucc : successor = applyState state delta := accumulate_complete_eq hcomplete
  by_cases h : delta.declarations.declarations.isEmpty = true
  · rw [applyState, if_pos h]
  · have hbody : successor.body = semanticApplyBody state.body delta := by
      rw [hsucc, applyState_body]
    have hsucc' : successor = unsealedFromBody (semanticApplyBody state.body delta) := by
      rw [hsucc, applyState, if_neg h]
    rw [applyState, if_neg h, hbody, semantic_update_idempotent]
    exact hsucc'.symm

/-! ## Batched updates and confluence (SPEC §12.5) -/

/-- **SPEC §12.5**, `Atlas.applyBatch`: deltas are merged through the pure
single-threaded update function. -/
def applyBatch (b : StateBody) (ds : List Delta) : StateBody :=
  ds.foldl semanticApplyBody b

@[simp] theorem applyBatch_nil (b : StateBody) : applyBatch b [] = b := rfl

@[simp] theorem applyBatch_cons (b : StateBody) (d : Delta) (ds : List Delta) :
    applyBatch b (d :: ds) = applyBatch (semanticApplyBody b d) ds := rfl

/-! ### Membership plumbing for the confluence proof -/

theorem mem_map_congr {α β : Type} (f : α → β) {l₁ l₂ : List α}
    (h : ∀ x, x ∈ l₁ ↔ x ∈ l₂) (y : β) : y ∈ l₁.map f ↔ y ∈ l₂.map f := by
  simp only [List.mem_map]
  constructor
  · intro hy
    obtain ⟨a, ha, he⟩ := hy
    exact ⟨a, (h a).mp ha, he⟩
  · intro hy
    obtain ⟨a, ha, he⟩ := hy
    exact ⟨a, (h a).mpr ha, he⟩

theorem mem_swap_append {α : Type} {u P Q P' Q' : List α}
    (hp : ∀ y, y ∈ P' ↔ y ∈ P) (hq : ∀ y, y ∈ Q' ↔ y ∈ Q) (x : α) :
    x ∈ (u ++ P) ++ Q' ↔ x ∈ (u ++ Q) ++ P' := by
  simp only [List.mem_append]
  constructor
  · intro h
    rcases h with (h | h) | h
    · exact Or.inl (Or.inl h)
    · exact Or.inr ((hp x).mpr h)
    · exact Or.inl (Or.inr ((hq x).mp h))
  · intro h
    rcases h with (h | h) | h
    · exact Or.inl (Or.inl h)
    · exact Or.inr ((hq x).mpr h)
    · exact Or.inl (Or.inr ((hp x).mp h))

/-- **SPEC §12.5**, `update_batch_confluent`: compatible deltas commute after
canonicalisation. -/
theorem update_batch_confluent (b : StateBody) {left right : Delta}
    (hcompatible : Compatible left right) :
    canonicalize (applyBatch b [left, right]) =
      canonicalize (applyBatch b [right, left]) := by
  by_cases hl : left.declarations.declarations.isEmpty = true
  · simp only [applyBatch_cons, applyBatch_nil, semanticApplyBody_trivial hl]
  by_cases hr : right.declarations.declarations.isEmpty = true
  · simp only [applyBatch_cons, applyBatch_nil, semanticApplyBody_trivial hr]
  -- Both deltas declare something: the two orders append the same two blocks.
  have hR₁ : ∀ x,
      x ∈ newDeclarations
        (appliedBody b (newDeclarations b.declarationBase left)).declarationBase right ↔
      x ∈ newDeclarations b.declarationBase right :=
    fun x => newDeclarations_union_of_compatible hcompatible x
  have hL₁ : ∀ x,
      x ∈ newDeclarations
        (appliedBody b (newDeclarations b.declarationBase right)).declarationBase left ↔
      x ∈ newDeclarations b.declarationBase left :=
    fun x => newDeclarations_union_of_compatible hcompatible.symm x
  simp only [applyBatch_cons, applyBatch_nil,
    semanticApplyBody_nontrivial hl, semanticApplyBody_nontrivial hr]
  refine canonicalize_eq_of_sameContent ?_
  have hroot :
      accumulatedDeltaRootOf
        ⟨(b.declarationBase.declarations ++ newDeclarations b.declarationBase left) ++
          newDeclarations
            (appliedBody b (newDeclarations b.declarationBase left)).declarationBase right⟩ =
      accumulatedDeltaRootOf
        ⟨(b.declarationBase.declarations ++ newDeclarations b.declarationBase right) ++
          newDeclarations
            (appliedBody b (newDeclarations b.declarationBase right)).declarationBase left⟩ :=
    accumulatedDeltaRootOf_congr (fun y => mem_swap_append hL₁ hR₁ y)
  exact
    { declarations := fun x => mem_swap_append hL₁ hR₁ x
      deltaIds := fun x => by rw [show
        (appliedBody (appliedBody b (newDeclarations b.declarationBase left))
          (newDeclarations
            (appliedBody b (newDeclarations b.declarationBase left)).declarationBase
            right)).accumulatedDeltaRoot =
        (appliedBody (appliedBody b (newDeclarations b.declarationBase right))
          (newDeclarations
            (appliedBody b (newDeclarations b.declarationBase right)).declarationBase
            left)).accumulatedDeltaRoot from hroot]
      objects := fun x =>
        mem_swap_append (mem_map_congr objectEntryOf hL₁) (mem_map_congr objectEntryOf hR₁) x
      shapeEdges := fun x =>
        mem_swap_append (mem_map_congr hyperEdgeOf hL₁) (mem_map_congr hyperEdgeOf hR₁) x
      facts := fun x =>
        mem_swap_append (mem_map_congr declId hL₁) (mem_map_congr declId hR₁) x
      derivations := fun x =>
        mem_swap_append (mem_map_congr derivationOf hL₁) (mem_map_congr derivationOf hR₁) x
      closureRoot := fun x =>
        mem_swap_append (mem_map_congr declId hL₁) (mem_map_congr declId hR₁) x
      attentionEntries := fun x =>
        mem_swap_append (mem_map_congr attentionEntryOf hL₁)
          (mem_map_congr attentionEntryOf hR₁) x
      attentionRoot := fun x => mem_swap_append hL₁ hR₁ x
      dependencyEdges := fun x =>
        mem_swap_append (mem_map_congr dependencyEdgeOf hL₁)
          (mem_map_congr dependencyEdgeOf hR₁) x
      dependencyRoot := fun x =>
        mem_swap_append (mem_map_congr declId hL₁) (mem_map_congr declId hR₁) x
      candidates := fun _ => Iff.rfl
      costs := fun _ => Iff.rfl
      partitions := fun _ => Iff.rfl
      partitionRoot := fun _ => Iff.rfl
      regions := fun _ => Iff.rfl
      envelopeRoot := fun _ => Iff.rfl
      certificateEntries := fun _ => Iff.rfl
      certificateRoot := fun _ => Iff.rfl
      objectiveId := rfl
      profileId := rfl
      problemId := rfl }

/-! ## Coherence is preserved -/

/-- **The homomorphism.**  On a coherent body, applying a delta is exactly
deriving from the extended declaration base.  This is the fact
`Atlas/Rebuild.lean` turns into `incremental_eq_full_rebuild`. -/
theorem semanticApplyBody_eq_derived {b : StateBody} (hb : Coherent b) (d : Delta) :
    semanticApplyBody b d = derivedBody b.scope (b.declarationBase ∪ d.declarations) := by
  by_cases h : d.declarations.declarations.isEmpty = true
  · rw [semanticApplyBody_trivial h, declarations_eq_nil h,
      CanonicalDeclarationSet.union_nil]
    exact hb
  · have hstep :=
      appliedBody_derived b.scope b.declarationBase (newDeclarations b.declarationBase d)
    rw [← hb] at hstep
    rw [semanticApplyBody_nontrivial h]
    exact hstep

/-- Coherence survives every update. -/
theorem semanticApplyBody_coherent {b : StateBody} (hb : Coherent b) (d : Delta) :
    Coherent (semanticApplyBody b d) := by
  rw [semanticApplyBody_eq_derived hb d]
  exact derivedBody_coherent _ _

end WasmGemmGnaf.Atlas
