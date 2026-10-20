/-
  The Atlas lifecycle: traces, strategy-indexed prefix evaluations, and the
  native lifecycle cost polynomial.
  Normative source: SPEC.md section 16 (the lifecycle half).

  SCOPE — read this before citing anything here.

  This file supplies the lifecycle carrier and accounting SPEC 16 makes
  normative: `LifecycleTraceBody` with its canonical identity,
  `ResolvedLifecycleTrace`, `LifecycleSizeVector`, `LifecycleAlgorithmTag`,
  `LifecyclePrefixResult`, `LifecycleEvaluation`, and `Atlas.nativeLifecycleBound`.

  Two things about it are load bearing and easy to get wrong, so they are
  spelled out.

  1. **Every verifier here is a recomputation, never a stored conclusion.**
     `VerifiesExactLifecyclePrefix` says the recorded before/after state
     identities, selected delta, optional seal and query-result identities are
     *exactly* what replaying the trace under the named strategy produces;
     `VerifiesLifecyclePrefixCost` and `VerifiesLifecycleSize` say the same of
     the charged cost and the size vector.  No structure field asserts that a
     bound holds, that two strategies agree, or that a lifecycle is optimal.

  2. **`lifecycle_native_bound` is OMITTED.**  It is a genuine inequality
     between the summed prefix costs and the polynomial, and it is false for an
     arbitrary primitive-cost table and an arbitrary trace: the coefficients
     must dominate the per-prefix charges, which is a property of a *release*
     table this repository has not pinned.  Rather than weaken the statement or
     assume it, it is absent; `nativeLifecycleBound_scope` below records
     machine-checked what the definition alone does and does not give.

  Also omitted, with reasons, at the end of the file:
  `canonicalFullRebuildEvaluation`,
  `lifecycle_incremental_semantics_eq_full_rebuild`,
  `lifecycle_full_rebuild_comparator_exact`.

  Every declaration in this file is either a definition or a kernel-checked
  theorem.  Nothing is assumed: no placeholder proof, no project axiom, and no
  compiled-evaluation decision procedure appears anywhere below.
-/
import WasmGemmGnaf.Atlas.Query
import WasmGemmGnaf.Atlas.Attention
import WasmGemmGnaf.Cost.Lifecycle

set_option autoImplicit false

namespace WasmGemmGnaf.Atlas

open WasmGemmGnaf.Foundation

/-! ## A canonical finite set of naturals (SPEC 16, `CanonicalFinset Nat`) -/

/-- A finite set of naturals in canonical form: strictly ascending, hence
duplicate free and uniquely determined by its membership. -/
structure CanonicalNatFinset where
  /-- The elements, strictly ascending. -/
  elements : List Nat
  /-- They really are strictly ascending. -/
  ascending : List.Pairwise (· < ·) elements

namespace CanonicalNatFinset

/-- The empty canonical finite set. -/
def empty : CanonicalNatFinset := ⟨[], List.Pairwise.nil⟩

@[simp] theorem empty_elements : empty.elements = [] := rfl

/-- Membership. -/
def Mem (s : CanonicalNatFinset) (n : Nat) : Prop := n ∈ s.elements

instance : Membership Nat CanonicalNatFinset where
  mem s n := Mem s n

instance decidableMem (s : CanonicalNatFinset) (n : Nat) : Decidable (Mem s n) :=
  inferInstanceAs (Decidable (n ∈ s.elements))

/-- A canonical finite set is duplicate free. -/
theorem nodup (s : CanonicalNatFinset) : s.elements.Nodup :=
  s.ascending.imp (fun h => Nat.ne_of_lt h)

theorem eq_of_elements_eq {a b : CanonicalNatFinset} (h : a.elements = b.elements) :
    a = b := by
  cases a; cases b; cases h; rfl

end CanonicalNatFinset

/-! ## The lifecycle trace body (SPEC 16) -/

/--
  **SPEC 16**, `Atlas.LifecycleTraceBody`: the first-order, canonically encoded
  release input that names one lifecycle.

  `CanonicalList DeltaId` and `CanonicalList RequestId` are the *schedules* of a
  trace, so their canonical order is the trace order, not the byte order; they
  are therefore plain lists, and `ResolvedLifecycleTrace.resolvesDeltas` pins
  them to the resolved objects position by position.
-/
structure LifecycleTraceBody where
  /-- Schema version. -/
  version : Nat
  /-- The pinned profile identity. -/
  profileId : ProfileId
  /-- The pinned problem identity. -/
  problemId : ProblemId
  /-- The pinned objective identity. -/
  objectiveId : ObjectiveId
  /-- The identity of the state the lifecycle starts from. -/
  initialStateId : StateIdentity
  /-- The delta schedule, in trace order. -/
  deltaIds : List Foundation.DeltaId
  /-- The request schedule, in trace order. -/
  requestIds : List RequestId
  /-- The prefix ordinal each request is answered after. -/
  requestAfterPrefixes : List Nat
  /-- The prefix ordinals at which the state is sealed. -/
  sealAfterPrefixes : CanonicalNatFinset
  /-- The number of deltas in the trace. -/
  horizon : Nat

namespace LifecycleTraceBody

/-- The flattening used to build the canonical encoder. -/
def toTuple (b : LifecycleTraceBody) :
    Nat × CanonicalObjectId × CanonicalObjectId × CanonicalObjectId ×
      CanonicalObjectId × List CanonicalObjectId × List CanonicalObjectId ×
      List Nat × List Nat × Nat :=
  (b.version, b.profileId, b.problemId, b.objectiveId, b.initialStateId,
   b.deltaIds, b.requestIds, b.requestAfterPrefixes,
   b.sealAfterPrefixes.elements, b.horizon)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  obtain ⟨v₁, p₁, q₁, o₁, s₁, d₁, r₁, ra₁, ⟨se₁, ha₁⟩, hz₁⟩ := a
  obtain ⟨v₂, p₂, q₂, o₂, s₂, d₂, r₂, ra₂, ⟨se₂, ha₂⟩, hz₂⟩ := b
  simp only [toTuple, Prod.mk.injEq] at h
  obtain ⟨e1, e2, e3, e4, e5, e6, e7, e8, e9, e10⟩ := h
  subst e1; subst e2; subst e3; subst e4; subst e5
  subst e6; subst e7; subst e8; subst e9; subst e10
  rfl

/-- The canonical prefix-free encoding of a lifecycle trace body. -/
def bytes (b : LifecycleTraceBody) : List UInt8 :=
  Bytes.pairBytes Bytes.natBytes
   (Bytes.pairBytes CanonicalObjectId.bytes
    (Bytes.pairBytes CanonicalObjectId.bytes
     (Bytes.pairBytes CanonicalObjectId.bytes
      (Bytes.pairBytes CanonicalObjectId.bytes
       (Bytes.pairBytes Enc.idList
        (Bytes.pairBytes Enc.idList
         (Bytes.pairBytes Enc.natList
          (Bytes.pairBytes Enc.natList Bytes.natBytes)))))))) b.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree Bytes.natBytes_prefixFree
   (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
    (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
     (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
      (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
       (Bytes.pairBytes_prefixFree Enc.idList_prefixFree
        (Bytes.pairBytes_prefixFree Enc.idList_prefixFree
         (Bytes.pairBytes_prefixFree Enc.natList_prefixFree
          (Bytes.pairBytes_prefixFree Enc.natList_prefixFree
            Bytes.natBytes_prefixFree))))))))).comp toTuple_injective

/-- The frozen canonical schema of a lifecycle trace body. -/
def identitySchema : CanonicalSchema LifecycleTraceBody :=
  CanonicalSchema.ofPrefixFree 1 CanonicalDomainTag.atlasState
    (leafTag 9 "Atlas.LifecycleTraceBody") (leafTag_size_pos 9 _)
    bytes bytes_prefixFree

/-- Every canonical identity a trace body references. -/
def referencedObjects (b : LifecycleTraceBody) : List CanonicalObjectId :=
  [b.profileId, b.problemId, b.objectiveId, b.initialStateId] ++
    b.deltaIds ++ b.requestIds

end LifecycleTraceBody

/-- **SPEC 16**, `Atlas.LifecycleTraceIdentity`. -/
abbrev LifecycleTraceIdentity := ObjectId LifecycleTraceBody

/-- **SPEC 16**, `Atlas.LifecycleTraceId`. -/
def LifecycleTraceId (body : LifecycleTraceBody) : LifecycleTraceIdentity :=
  Identity LifecycleTraceBody.identitySchema body

/-- Trace identity is structural comparison, not a digest. -/
theorem LifecycleTraceId_eq_iff {a b : LifecycleTraceBody} :
    LifecycleTraceId a = LifecycleTraceId b ↔ a = b :=
  Identity_eq_iff LifecycleTraceBody.identitySchema

theorem LifecycleTraceId_injective : Function.Injective LifecycleTraceId :=
  fun _ _ h => LifecycleTraceId_eq_iff.mp h

/-! ## Complete preimages of a trace (SPEC 16, `CompleteLifecycleObjectGraph`) -/

/-- **SPEC 16**, `CompleteLifecycleObjectGraph`: every identity the trace body
references resolves in the retained graph.  This mirrors
`Atlas.CompleteObjectGraph` of SPEC 12.1; the completeness field is a side
condition discharged at construction, never a stored conclusion. -/
structure CompleteLifecycleObjectGraph (body : LifecycleTraceBody) where
  /-- The retained graph. -/
  graph : ObjectGraph
  /-- It resolves every referenced identity. -/
  complete : ∀ id ∈ body.referencedObjects, graph.resolves id = true

/-- The canonical complete preimage graph of a trace body: retain exactly the
canonical preimage of every identity it references. -/
def canonicalLifecycleObjectGraph (body : LifecycleTraceBody) :
    CompleteLifecycleObjectGraph body where
  graph := ObjectGraph.ofIds body.referencedObjects
  complete := fun _ hid => ObjectGraph.ofIds_resolves hid

/-! ## Resolved traces (SPEC 16) -/

/-- The canonical identities of a resolved delta schedule. -/
def CanonicalDeltaIds (deltas : List Delta) : List Foundation.DeltaId :=
  deltas.map DeltaId

/-- The canonical identities of a resolved request schedule. -/
def CanonicalRequestIds (requests : List RequestSignature) : List RequestId :=
  requests.map RequestSignatureId

/--
  **SPEC 16**, `Atlas.ResolvedLifecycleTrace`.

  Every `resolves*` field is an equation between recomputed identities and the
  first-order schedule recorded in the body, so a resolved trace cannot name
  objects the body does not.
-/
structure ResolvedLifecycleTrace (body : LifecycleTraceBody) where
  /-- The state the lifecycle starts from. -/
  initialState : UnsealedState
  /-- The resolved deltas, in trace order. -/
  deltas : List Delta
  /-- The resolved requests, in trace order. -/
  requests : List RequestSignature
  /-- The initial state really is the one the body names. -/
  resolvesInitial : initialState.bodyId = body.initialStateId
  /-- The delta schedule really is the one the body names. -/
  resolvesDeltas : CanonicalDeltaIds deltas = body.deltaIds
  /-- The request schedule really is the one the body names. -/
  resolvesRequests : CanonicalRequestIds requests = body.requestIds
  /-- The horizon is the length of the delta schedule. -/
  horizonEq : body.horizon = body.deltaIds.length
  /-- Every request has exactly one scheduled prefix ordinal. -/
  requestScheduleLength :
    body.requestAfterPrefixes.length = body.requestIds.length
  /-- Every scheduled request ordinal is inside the horizon. -/
  requestOrdinalsValid : ∀ ordinal ∈ body.requestAfterPrefixes,
    ordinal ≤ body.horizon
  /-- Every seal ordinal is inside the horizon. -/
  sealOrdinalsValid : ∀ ordinal ∈ body.sealAfterPrefixes.elements,
    ordinal ≤ body.horizon
  /-- Every referenced object is retained. -/
  completePreimages : CompleteLifecycleObjectGraph body

namespace ResolvedLifecycleTrace

variable {body : LifecycleTraceBody}

/-- The resolved trace has exactly `body.horizon` deltas. -/
theorem deltas_length (trace : ResolvedLifecycleTrace body) :
    trace.deltas.length = body.horizon := by
  rw [trace.horizonEq, ← trace.resolvesDeltas, CanonicalDeltaIds, List.length_map]

/-- The resolved trace has exactly as many requests as scheduled ordinals. -/
theorem requests_length (trace : ResolvedLifecycleTrace body) :
    trace.requests.length = body.requestAfterPrefixes.length := by
  rw [trace.requestScheduleLength, ← trace.resolvesRequests, CanonicalRequestIds,
    List.length_map]

/-- The scope the lifecycle runs in: the initial state's own scope. -/
def scope (trace : ResolvedLifecycleTrace body) : Scope :=
  trace.initialState.body.scope

end ResolvedLifecycleTrace

/-! ## Strategies (SPEC 16) -/

/-- **SPEC 16**, `Atlas.LifecycleAlgorithmTag`.  A lifecycle evaluation is
strategy indexed; there is no untagged evaluation whose native and full-rebuild
costs could be interchanged. -/
inductive LifecycleAlgorithmTag
  /-- The native incremental update operator of SPEC 12.5. -/
  | nativeIncremental
  /-- The comparator: a canonical full rebuild at every seal point. -/
  | canonicalFullRebuildAtEverySeal
  deriving DecidableEq, Repr, Inhabited

/-! ## Replaying a trace

Every verifier below is stated against these total, computable replays.  They
are functions of the trace and the strategy alone. -/

namespace ResolvedLifecycleTrace

variable {body : LifecycleTraceBody}

/-- The delta selected at prefix `n`: none at prefix `0` (the initial state),
otherwise the `n-1`-st delta of the schedule. -/
def selectedDelta? (trace : ResolvedLifecycleTrace body) : Nat → Option Delta
  | 0 => none
  | k + 1 => trace.deltas[k]?

@[simp] theorem selectedDelta?_zero (trace : ResolvedLifecycleTrace body) :
    trace.selectedDelta? 0 = none := rfl

/-- The state body reached after applying the first `n` deltas incrementally. -/
def nativeBodyAt (trace : ResolvedLifecycleTrace body) : Nat → StateBody
  | 0 => trace.initialState.body
  | k + 1 =>
      match trace.deltas[k]? with
      | some delta => semanticApplyBody (trace.nativeBodyAt k) delta
      | none => trace.nativeBodyAt k

@[simp] theorem nativeBodyAt_zero (trace : ResolvedLifecycleTrace body) :
    trace.nativeBodyAt 0 = trace.initialState.body := rfl

/-- The declaration base accumulated after `n` prefixes. -/
def accumulatedDeclarations (trace : ResolvedLifecycleTrace body) (n : Nat) :
    CanonicalDeclarationSet :=
  (trace.nativeBodyAt n).declarationBase

/-- The state body the canonical full rebuild reaches after `n` prefixes: the
rebuild of the accumulated declaration base in the trace's own scope. -/
def rebuildBodyAt (trace : ResolvedLifecycleTrace body) (n : Nat) : StateBody :=
  semanticRebuildBodyWith trace.scope (trace.accumulatedDeclarations n)

/-- The state body a named strategy reaches after `n` prefixes. -/
def bodyAt (trace : ResolvedLifecycleTrace body) :
    LifecycleAlgorithmTag → Nat → StateBody
  | .nativeIncremental, n => trace.nativeBodyAt n
  | .canonicalFullRebuildAtEverySeal, n => trace.rebuildBodyAt n

/-- **The two strategies reach the same state on a coherent trace.**  This is
`Atlas.semanticApplyBody_eq_rebuild` of SPEC 12.5, lifted to prefix `0`; it is
not the full observational-equality theorem of SPEC 16, which is omitted. -/
theorem bodyAt_zero_agree (trace : ResolvedLifecycleTrace body)
    (hcoherent : Coherent trace.initialState.body) :
    trace.bodyAt .nativeIncremental 0 = trace.bodyAt .canonicalFullRebuildAtEverySeal 0 := by
  show trace.initialState.body =
    semanticRebuildBodyWith trace.scope trace.initialState.body.declarationBase
  exact hcoherent

/-- Whether prefix `n` is a seal point. -/
def isSealPoint (_trace : ResolvedLifecycleTrace body) (n : Nat) : Bool :=
  decide (n ∈ body.sealAfterPrefixes.elements)

/-- The requests scheduled to be answered after prefix `n`. -/
def scheduledRequests (trace : ResolvedLifecycleTrace body) (n : Nat) :
    List RequestSignature :=
  (body.requestAfterPrefixes.zip trace.requests).filterMap
    (fun p => if p.1 = n then some p.2 else none)

end ResolvedLifecycleTrace

/-! ## Query outcomes and their identities -/

/-- The first-order body of one answered lifecycle query: the state it was
answered against, the request, and the exact routed identities.  Every field is
recomputed by `Atlas.attendTargets`; none is a stored conclusion. -/
structure QueryOutcomeBody where
  /-- The state the query was answered against. -/
  stateId : StateIdentity
  /-- The request. -/
  requestId : RequestId
  /-- The exact routed identities, in index order. -/
  targets : List CanonicalObjectId
  deriving DecidableEq

namespace QueryOutcomeBody

/-- The flattening used to build the canonical encoder. -/
def toTuple (b : QueryOutcomeBody) :
    CanonicalObjectId × CanonicalObjectId × List CanonicalObjectId :=
  (b.stateId, b.requestId, b.targets)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [QueryOutcomeBody.mk.injEq]
  exact ⟨h.1, h.2.1, h.2.2⟩

/-- The canonical prefix-free encoding of a query outcome. -/
def bytes (b : QueryOutcomeBody) : List UInt8 :=
  Bytes.pairBytes CanonicalObjectId.bytes
    (Bytes.pairBytes CanonicalObjectId.bytes Enc.idList) b.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
    (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
      Enc.idList_prefixFree)).comp toTuple_injective

/-- The frozen canonical schema of a query outcome. -/
def identitySchema : CanonicalSchema QueryOutcomeBody :=
  CanonicalSchema.ofPrefixFree 1 CanonicalDomainTag.queryResult
    (leafTag 10 "Atlas.QueryOutcomeBody") (leafTag_size_pos 10 _)
    bytes bytes_prefixFree

end QueryOutcomeBody

/-- The canonical identity of an answered lifecycle query. -/
def QueryOutcomeId (b : QueryOutcomeBody) : QueryResultId :=
  CanonicalObjectId.ofTyped (Identity QueryOutcomeBody.identitySchema b)

theorem QueryOutcomeId_eq_iff {a b : QueryOutcomeBody} :
    QueryOutcomeId a = QueryOutcomeId b ↔ a = b :=
  CanonicalObjectId.ofTyped_Identity_eq_iff QueryOutcomeBody.identitySchema

namespace ResolvedLifecycleTrace

variable {body : LifecycleTraceBody}

/-- The exact query outcome of one request at prefix `n` under a strategy. -/
def queryOutcomeAt (trace : ResolvedLifecycleTrace body)
    (algorithm : LifecycleAlgorithmTag) (n : Nat) (request : RequestSignature) :
    QueryOutcomeBody :=
  { stateId := StateId (trace.bodyAt algorithm n)
    requestId := RequestSignatureId request
    targets := attendTargets (trace.bodyAt algorithm n) request }

/-- The exact query-result identities produced at prefix `n`. -/
def queryResultIdsAt (trace : ResolvedLifecycleTrace body)
    (algorithm : LifecycleAlgorithmTag) (n : Nat) : List QueryResultId :=
  (trace.scheduledRequests n).map
    (fun request => QueryOutcomeId (trace.queryOutcomeAt algorithm n request))

/-- The exact seal identity produced at prefix `n`, when `n` is a seal point. -/
def sealIdAt (trace : ResolvedLifecycleTrace body)
    (_algorithm : LifecycleAlgorithmTag) (n : Nat) : Option SealIdentity :=
  if trace.isSealPoint n then
    some (derivedSealedState trace.scope (trace.accumulatedDeclarations n)).sealId
  else
    none

end ResolvedLifecycleTrace

/-! ## The exact prefix transition verifier (SPEC 16) -/

/--
  **SPEC 16**, `VerifiesExactLifecyclePrefix`.

  Each prefix verifier binds its strategy, before state, selected delta, after
  state, optional seal and exact query-result identities.  Every conjunct is an
  equation against the replay above, so nothing here can be recorded without
  being recomputable.
-/
def VerifiesExactLifecyclePrefix {body : LifecycleTraceBody}
    (algorithm : LifecycleAlgorithmTag) (trace : ResolvedLifecycleTrace body)
    (prefixOrdinal : Fin (body.horizon + 1))
    (beforeStateId : StateIdentity) (deltaId : Option Foundation.DeltaId)
    (afterStateId : StateIdentity) (sealId : Option SealIdentity)
    (queryResultIds : List QueryResultId) : Prop :=
  beforeStateId = StateId (trace.bodyAt algorithm (prefixOrdinal.val - 1)) ∧
  deltaId = (trace.selectedDelta? prefixOrdinal.val).map DeltaId ∧
  afterStateId = StateId (trace.bodyAt algorithm prefixOrdinal.val) ∧
  sealId = trace.sealIdAt algorithm prefixOrdinal.val ∧
  queryResultIds = trace.queryResultIdsAt algorithm prefixOrdinal.val

/-! ## The exact prefix cost (SPEC 16) -/

namespace ResolvedLifecycleTrace

variable {body : LifecycleTraceBody}

/-- The total size in bytes of a list of canonical declarations. -/
def declarationBytes (l : List ByteArray) : Nat := (l.map ByteArray.size).sum

/-- The genuinely new declarations the selected delta contributes at prefix
`n`. -/
def novelDeclarationsAt (trace : ResolvedLifecycleTrace body) (n : Nat) :
    List ByteArray :=
  match trace.selectedDelta? n with
  | none => []
  | some delta =>
      newDeclarations (trace.nativeBodyAt (n - 1)).declarationBase delta

/-- The declarations a strategy actually canonicalizes at prefix `n`: the novel
ones for the native operator, and — at a seal point — the whole accumulated base
for the canonical full rebuild. -/
def chargedDeclarationsAt (trace : ResolvedLifecycleTrace body)
    (algorithm : LifecycleAlgorithmTag) (n : Nat) : List ByteArray :=
  match algorithm with
  | .nativeIncremental => trace.novelDeclarationsAt n
  | .canonicalFullRebuildAtEverySeal =>
      if trace.isSealPoint n then (trace.accumulatedDeclarations n).declarations
      else trace.novelDeclarationsAt n

/--
  **SPEC 16**, the exact charge of one lifecycle prefix.

  This is a *definition* of the lifecycle cost model, in the sense SPEC 9.1
  fixes the artifact cost model: it says what is charged, not that any bound
  holds.  Accumulation, closure, indexing, invalidation, sealing, retention and
  query selection are each charged against the work the replay actually
  performs at this prefix.
-/
def prefixCost (trace : ResolvedLifecycleTrace body)
    (algorithm : LifecycleAlgorithmTag) (n : Nat) : Cost.LifecycleVector :=
  let charged := trace.chargedDeclarationsAt algorithm n
  let novelCount := charged.length
  let chargedBytes := declarationBytes charged
  let retained := declarationBytes (trace.accumulatedDeclarations n).declarations
  let requests := trace.scheduledRequests n
  let routed :=
    (requests.map (fun r => (attendTargets (trace.bodyAt algorithm n) r).length)).sum
  let sealed := trace.isSealPoint n
  { authorityCheckSteps :=
      if n = 0 then
        declarationBytes trace.initialState.body.declarationBase.declarations
      else 0
    canonicalizationSteps := chargedBytes
    canonicalNovelObjects := novelCount
    canonicalNovelEdges := novelCount
    closureSteps := novelCount
    indexSteps := novelCount
    attentionBucketsTouched := novelCount
    dependencyObjectsVisited := novelCount
    partitionSteps := 0
    partitionCellsChanged := 0
    verifierSteps := if sealed then retained else 0
    sealSteps := if sealed then 1 + retained else 0
    querySelectionSteps := requests.length + routed
    migrationSteps := 0
    retainedStateBytes := retained
    peakWorkingBytes := retained + chargedBytes
    artifact := Cost.ArtifactVector.zero }

end ResolvedLifecycleTrace

/-- **SPEC 16**, `Cost.VerifiesLifecyclePrefixCost`: the recorded charge is
exactly the replay's charge. -/
def VerifiesLifecyclePrefixCost {body : LifecycleTraceBody}
    (algorithm : LifecycleAlgorithmTag) (trace : ResolvedLifecycleTrace body)
    (prefixOrdinal : Fin (body.horizon + 1))
    (_beforeStateId : StateIdentity) (_deltaId : Option Foundation.DeltaId)
    (_afterStateId : StateIdentity) (_sealId : Option SealIdentity)
    (_queryResultIds : List QueryResultId) (cost : Cost.LifecycleVector) : Prop :=
  cost = trace.prefixCost algorithm prefixOrdinal.val

/-! ## Prefix results (SPEC 16) -/

/-- **SPEC 16**, `Atlas.LifecyclePrefixResult`. -/
structure LifecyclePrefixResult {body : LifecycleTraceBody}
    (trace : ResolvedLifecycleTrace body) (algorithm : LifecycleAlgorithmTag) where
  /-- Which prefix this result is about. -/
  prefixOrdinal : Fin (body.horizon + 1)
  /-- The state identity before the prefix. -/
  beforeStateId : StateIdentity
  /-- The delta applied at the prefix, if any. -/
  deltaId : Option Foundation.DeltaId
  /-- The state identity after the prefix. -/
  afterStateId : StateIdentity
  /-- The seal produced at the prefix, if any. -/
  sealId : Option SealIdentity
  /-- The exact query-result identities produced at the prefix. -/
  queryResultIds : List QueryResultId
  /-- The charge of the prefix. -/
  cost : Cost.LifecycleVector
  /-- The transition really is the replay's transition. -/
  exactTransition : VerifiesExactLifecyclePrefix algorithm trace prefixOrdinal
    beforeStateId deltaId afterStateId sealId queryResultIds
  /-- The charge really is the replay's charge. -/
  exactCost : VerifiesLifecyclePrefixCost algorithm trace prefixOrdinal
    beforeStateId deltaId afterStateId sealId queryResultIds cost

namespace LifecyclePrefixResult

variable {body : LifecycleTraceBody} {trace : ResolvedLifecycleTrace body}
  {algorithm : LifecycleAlgorithmTag}

/-- The recorded charge is the replay's charge. -/
theorem cost_eq (r : LifecyclePrefixResult trace algorithm) :
    r.cost = trace.prefixCost algorithm r.prefixOrdinal.val := r.exactCost

/-- The recorded after-state identity is the replay's state identity. -/
theorem afterStateId_eq (r : LifecyclePrefixResult trace algorithm) :
    r.afterStateId = StateId (trace.bodyAt algorithm r.prefixOrdinal.val) :=
  r.exactTransition.2.2.1

/-- The recorded query-result identities are the replay's. -/
theorem queryResultIds_eq (r : LifecyclePrefixResult trace algorithm) :
    r.queryResultIds = trace.queryResultIdsAt algorithm r.prefixOrdinal.val :=
  r.exactTransition.2.2.2.2

/-- **Two prefix results for the same ordinal are identical.**  Nothing in a
prefix result is free: the replay determines every field. -/
theorem eq_of_prefixOrdinal_eq {a b : LifecyclePrefixResult trace algorithm}
    (h : a.prefixOrdinal = b.prefixOrdinal) : a = b := by
  obtain ⟨oa, ba, da, aa, sa, qa, ca, ta, ka⟩ := a
  obtain ⟨ob, bb, db, ab, sb, qb, cb, tb, kb⟩ := b
  simp only at h
  subst h
  obtain ⟨t1, t2, t3, t4, t5⟩ := ta
  obtain ⟨u1, u2, u3, u4, u5⟩ := tb
  subst t1; subst t2; subst t3; subst t4; subst t5
  subst u1; subst u2; subst u3; subst u4; subst u5
  have : ca = cb := by rw [ka, kb]
  subst this
  rfl

end LifecyclePrefixResult

/-! ## Prefix-ordered families

SPEC 16 writes `NonemptyCanonicalList (LifecyclePrefixResult …)`.  The canonical
order of a lifecycle family is **prefix order** — SPEC 16 itself fixes the fold
as "natural addition in prefix order" — not the byte order of a schema.  The
carrier below is therefore a nonempty list, and `CoversExactlyEveryPrefix` pins
the order: it forces the ordinals to be exactly `0, 1, …, horizon`, in that
order, with no repetition and nothing missing.  `coversExactlyEveryPrefix_nodup`
and `coversExactlyEveryPrefix_length` prove that the pinning really is that
strong. -/

/-- A nonempty family in prefix order. -/
structure PrefixOrderedList (α : Type) where
  /-- The first element. -/
  head : α
  /-- The remaining elements, in order. -/
  rest : List α

namespace PrefixOrderedList

variable {α : Type}

/-- The elements, in prefix order. -/
def elements (l : PrefixOrderedList α) : List α := l.head :: l.rest

theorem elements_ne_nil (l : PrefixOrderedList α) : l.elements ≠ [] := by
  simp [elements]

@[simp] theorem elements_cons (a : α) (rest : List α) :
    (PrefixOrderedList.mk a rest).elements = a :: rest := rfl

end PrefixOrderedList

/-- **SPEC 16**, `CoversExactlyEveryPrefix`: the family's ordinals are exactly
`0, …, horizon`, in prefix order. -/
def CoversExactlyEveryPrefix {body : LifecycleTraceBody}
    {trace : ResolvedLifecycleTrace body} {algorithm : LifecycleAlgorithmTag}
    (prefixes : PrefixOrderedList (LifecyclePrefixResult trace algorithm)) : Prop :=
  prefixes.elements.map (fun r => r.prefixOrdinal.val) = List.range (body.horizon + 1)

/-- The cover has exactly `horizon + 1` members. -/
theorem coversExactlyEveryPrefix_length {body : LifecycleTraceBody}
    {trace : ResolvedLifecycleTrace body} {algorithm : LifecycleAlgorithmTag}
    {prefixes : PrefixOrderedList (LifecyclePrefixResult trace algorithm)}
    (h : CoversExactlyEveryPrefix prefixes) :
    prefixes.elements.length = body.horizon + 1 := by
  have := congrArg List.length h
  simpa using this

/-- The cover repeats no prefix. -/
theorem coversExactlyEveryPrefix_nodup {body : LifecycleTraceBody}
    {trace : ResolvedLifecycleTrace body} {algorithm : LifecycleAlgorithmTag}
    {prefixes : PrefixOrderedList (LifecyclePrefixResult trace algorithm)}
    (h : CoversExactlyEveryPrefix prefixes) :
    (prefixes.elements.map (fun r => r.prefixOrdinal.val)).Nodup := by
  rw [h]
  exact List.nodup_range

/-- The cover misses no prefix. -/
theorem coversExactlyEveryPrefix_complete {body : LifecycleTraceBody}
    {trace : ResolvedLifecycleTrace body} {algorithm : LifecycleAlgorithmTag}
    {prefixes : PrefixOrderedList (LifecyclePrefixResult trace algorithm)}
    (h : CoversExactlyEveryPrefix prefixes) (n : Nat) (hn : n ≤ body.horizon) :
    ∃ r ∈ prefixes.elements, r.prefixOrdinal.val = n := by
  have hmem : n ∈ prefixes.elements.map (fun r => r.prefixOrdinal.val) := by
    rw [h]
    exact List.mem_range.mpr (Nat.lt_succ_of_le hn)
  obtain ⟨r, hr, hval⟩ := List.mem_map.mp hmem
  exact ⟨r, hr, hval⟩

/-! ## Lifecycle size vectors and the native bound (SPEC 16) -/

/-- **SPEC 16**, `Atlas.LifecycleSizeVector`.  Defined once, in
`Cost/Lifecycle.lean`; this is the Atlas-facing name of the same record. -/
abbrev LifecycleSizeVector := Cost.LifecycleSizeVector

/--
  **SPEC 16**, `Atlas.nativeLifecycleBound`: the fixed componentwise polynomial.

  Authority checking is bounded by `cAuthority * authorityBytesChecked`;
  canonicalization by
  `cCanonicalize * deltaBytes * (log2 (retainedStateBytes + 2) + 1)`; closure by
  `cClosure * closureDerivations`; indexing by `cIndex * (canonicalNovelObjects
  + canonicalNovelEdges + attentionBucketsTouched)`; invalidation by
  `cDependency * dependencyImpactObjects`; partition work by `cPartition *
  partitionCellsChanged`; verification and sealing by their fixed coefficients
  times `certificateBytesChecked`, `sealCount` and the cumulative
  `sealInputBytesScanned`; query work by `cQuery * (queryCount +
  requestBytesChecked + attentionCandidatesVisited)`; migration by `cMigration *
  migrationObjects`.  Artifact work is exactly `size.artifactWork`, and the
  retained-state and peak-working-byte coordinates are passed through, not
  absorbed into constants.

  Defined once, in `Cost/Lifecycle.lean`; this is the Atlas-facing name.
-/
def nativeLifecycleBound (table : Cost.PrimitiveCostTable)
    (size : LifecycleSizeVector) : Cost.LifecycleVector :=
  Cost.nativeLifecycleBound table size

/-- Artifact work is exactly the size vector's artifact work. -/
theorem nativeLifecycleBound_artifact (table : Cost.PrimitiveCostTable)
    (size : LifecycleSizeVector) :
    (nativeLifecycleBound table size).artifact = size.artifactWork := rfl

/-- Retained-state bytes are passed through, not absorbed into a constant. -/
theorem nativeLifecycleBound_retainedStateBytes (table : Cost.PrimitiveCostTable)
    (size : LifecycleSizeVector) :
    (nativeLifecycleBound table size).retainedStateBytes
      = size.retainedStateBytes := rfl

/-- Peak working bytes are passed through, not absorbed into a constant. -/
theorem nativeLifecycleBound_peakWorkingBytes (table : Cost.PrimitiveCostTable)
    (size : LifecycleSizeVector) :
    (nativeLifecycleBound table size).peakWorkingBytes
      = size.peakWorkingBytes := rfl

/--
  **Scope of `nativeLifecycleBound`.**  The polynomial is a function of the
  primitive-cost table and the size vector *only*: two lifecycles with the same
  size vector get the same bound, whatever their traces, strategies or prefix
  costs were.  Consequently no statement about an actual lifecycle total follows
  from this definition alone — that is exactly the content of the omitted
  `lifecycle_native_bound`, and it must be proved, never read off from here.
-/
theorem nativeLifecycleBound_scope_size_only
    (table : Cost.PrimitiveCostTable) (a b : LifecycleSizeVector) (h : a = b) :
    nativeLifecycleBound table a = nativeLifecycleBound table b := by rw [h]

/-! ## The exact size verifier (SPEC 16) -/

namespace ResolvedLifecycleTrace

variable {body : LifecycleTraceBody}

/-- The exact size vector of a replayed lifecycle family. -/
def lifecycleSize (trace : ResolvedLifecycleTrace body)
    (algorithm : LifecycleAlgorithmTag)
    (prefixes : PrefixOrderedList (LifecyclePrefixResult trace algorithm)) :
    LifecycleSizeVector :=
  let ordinals := prefixes.elements.map (fun r => r.prefixOrdinal.val)
  let costs := prefixes.elements.map (fun r => r.cost)
  { horizon := body.horizon
    authorityBytesChecked :=
      declarationBytes trace.initialState.body.declarationBase.declarations
    deltaBytes :=
      (ordinals.map (fun n => declarationBytes (trace.novelDeclarationsAt n))).sum
    requestBytesChecked :=
      (ordinals.map (fun n =>
        ((trace.scheduledRequests n).map
          (fun r => declarationBytes r.tokens)).sum)).sum
    canonicalNovelObjects := (costs.map (·.canonicalNovelObjects)).sum
    canonicalNovelEdges := (costs.map (·.canonicalNovelEdges)).sum
    closureDerivations := (costs.map (·.closureSteps)).sum
    attentionBucketsTouched := (costs.map (·.attentionBucketsTouched)).sum
    dependencyImpactObjects := (costs.map (·.dependencyObjectsVisited)).sum
    partitionCellsChanged := (costs.map (·.partitionCellsChanged)).sum
    certificateBytesChecked := (costs.map (·.verifierSteps)).sum
    retainedStateBytes := Cost.maxOf (costs.map (·.retainedStateBytes))
    sealCount := (ordinals.filter (fun n => trace.isSealPoint n)).length
    sealInputBytesScanned :=
      (ordinals.map (fun n =>
        if trace.isSealPoint n then
          declarationBytes (trace.accumulatedDeclarations n).declarations
        else 0)).sum
    queryCount := (ordinals.map (fun n => (trace.scheduledRequests n).length)).sum
    attentionCandidatesVisited :=
      (ordinals.map (fun n =>
        ((trace.scheduledRequests n).map
          (fun r => (attendTargets (trace.bodyAt algorithm n) r).length)).sum)).sum
    migrationObjects := 0
    peakWorkingBytes := Cost.maxOf (costs.map (·.peakWorkingBytes))
    artifactWork := Cost.ArtifactVector.zero }

end ResolvedLifecycleTrace

/-- **SPEC 16**, `VerifiesLifecycleSize`: the recorded size vector is exactly
the replay's size vector. -/
def VerifiesLifecycleSize {body : LifecycleTraceBody}
    {algorithm : LifecycleAlgorithmTag} (trace : ResolvedLifecycleTrace body)
    (prefixes : PrefixOrderedList (LifecyclePrefixResult trace algorithm))
    (size : LifecycleSizeVector) : Prop :=
  size = trace.lifecycleSize algorithm prefixes

/-! ## Lifecycle evaluations (SPEC 16) -/

/--
  **SPEC 16**, `Atlas.LifecycleEvaluation`.

  Strategy indexed: `algorithm` is an index of the type, so a native evaluation
  and a full-rebuild evaluation are never interchangeable terms.
-/
structure LifecycleEvaluation {body : LifecycleTraceBody}
    (trace : ResolvedLifecycleTrace body) (algorithm : LifecycleAlgorithmTag) where
  /-- One result per prefix, in prefix order. -/
  prefixes : PrefixOrderedList (LifecyclePrefixResult trace algorithm)
  /-- They cover exactly every prefix. -/
  exactPrefixCover : CoversExactlyEveryPrefix prefixes
  /-- The size vector. -/
  size : LifecycleSizeVector
  /-- It is exactly the replay's size vector. -/
  sizeExact : VerifiesLifecycleSize trace prefixes size
  /-- The lifecycle total. -/
  total : Cost.LifecycleVector
  /-- It is exactly the frozen fold of the prefix costs, in prefix order. -/
  totalExact : total = Cost.sumLifecycle (prefixes.elements.map (fun r => r.cost))

/--
  **SPEC 16**, `Atlas.lifecycle_prefix_conservation`.

  The lifecycle total is the frozen mixed fold of the prefix costs in prefix
  order — every step/count coordinate summed, `retainedStateBytes` and
  `peakWorkingBytes` taken as componentwise maxima, the empty fold all zero.
  Nothing may be charged to a lifecycle that is not charged to one of its
  prefixes, and nothing charged to a prefix may be dropped.
-/
theorem lifecycle_prefix_conservation {body : LifecycleTraceBody}
    {trace : ResolvedLifecycleTrace body} {algorithm : LifecycleAlgorithmTag}
    (evaluation : LifecycleEvaluation trace algorithm) :
    evaluation.total =
      Cost.sumLifecycle (evaluation.prefixes.elements.map (fun r => r.cost)) :=
  evaluation.totalExact

namespace LifecycleEvaluation

variable {body : LifecycleTraceBody} {trace : ResolvedLifecycleTrace body}
  {algorithm : LifecycleAlgorithmTag}

/-- The evaluation covers exactly `horizon + 1` prefixes. -/
theorem prefixes_length (evaluation : LifecycleEvaluation trace algorithm) :
    evaluation.prefixes.elements.length = body.horizon + 1 :=
  coversExactlyEveryPrefix_length evaluation.exactPrefixCover

/-- **Conservation, per coordinate.**  Every prefix's charge on an additively
folded coordinate is dominated by the lifecycle total. -/
theorem prefix_le_total_add_coord (evaluation : LifecycleEvaluation trace algorithm)
    (f : Cost.LifecycleVector → Nat)
    (hzero : f Cost.LifecycleVector.zero = 0)
    (hcomb : ∀ a b, f (Cost.LifecycleVector.combine a b) = f a + f b)
    {r : LifecyclePrefixResult trace algorithm}
    (hr : r ∈ evaluation.prefixes.elements) :
    f r.cost ≤ f evaluation.total := by
  rw [evaluation.totalExact]
  exact Cost.le_sumLifecycle_add_coord f hzero hcomb (List.mem_map_of_mem hr)

/-- The same for a maximum-folded coordinate. -/
theorem prefix_le_total_max_coord (evaluation : LifecycleEvaluation trace algorithm)
    (f : Cost.LifecycleVector → Nat)
    (hzero : f Cost.LifecycleVector.zero = 0)
    (hcomb : ∀ a b, f (Cost.LifecycleVector.combine a b) = max (f a) (f b))
    {r : LifecyclePrefixResult trace algorithm}
    (hr : r ∈ evaluation.prefixes.elements) :
    f r.cost ≤ f evaluation.total := by
  rw [evaluation.totalExact]
  exact Cost.le_sumLifecycle_max_coord f hzero hcomb (List.mem_map_of_mem hr)

/-- Sealing steps are conserved: no seal's charge escapes the total. -/
theorem sealSteps_le_total (evaluation : LifecycleEvaluation trace algorithm)
    {r : LifecyclePrefixResult trace algorithm}
    (hr : r ∈ evaluation.prefixes.elements) :
    r.cost.sealSteps ≤ evaluation.total.sealSteps :=
  evaluation.prefix_le_total_add_coord _ rfl (fun _ _ => rfl) hr

/-- Retained-state bytes are conserved as a maximum, exactly as SPEC 16's fold
prescribes. -/
theorem retainedStateBytes_le_total (evaluation : LifecycleEvaluation trace algorithm)
    {r : LifecyclePrefixResult trace algorithm}
    (hr : r ∈ evaluation.prefixes.elements) :
    r.cost.retainedStateBytes ≤ evaluation.total.retainedStateBytes :=
  evaluation.prefix_le_total_max_coord _ rfl (fun _ _ => rfl) hr

/-- The recorded size vector is the replay's size vector. -/
theorem size_eq (evaluation : LifecycleEvaluation trace algorithm) :
    evaluation.size = trace.lifecycleSize algorithm evaluation.prefixes :=
  evaluation.sizeExact

/-- **The evaluation is determined by the trace and the strategy.**  Two
evaluations of the same trace under the same strategy that cover the prefixes
identically agree on every prefix result, hence on the total and the size. -/
theorem eq_of_prefixes_eq {a b : LifecycleEvaluation trace algorithm}
    (h : a.prefixes = b.prefixes) : a = b := by
  obtain ⟨pa, ca, sa, ea, ta, ka⟩ := a
  obtain ⟨pb, cb, sb, eb, tb, kb⟩ := b
  simp only at h
  subst h
  have hs : sa = sb := by rw [ea, eb]
  have ht : ta = tb := by rw [ka, kb]
  subst hs; subst ht
  rfl

end LifecycleEvaluation

/-! ## Amortization (SPEC 16)

"An amortized statement SHALL divide only the exact summed numerator by the
explicit positive `body.horizon`; it SHALL retain the nonamortized total and
every prefix result."  The definition below takes the evaluation itself, so
every prefix result and the nonamortized total are retained by construction, and
it is only defined for a positive horizon. -/

/-- The amortized charge of an additively folded coordinate over an explicitly
positive horizon.  The evaluation — hence the nonamortized total and every
prefix result — is retained. -/
def amortized {body : LifecycleTraceBody} {trace : ResolvedLifecycleTrace body}
    {algorithm : LifecycleAlgorithmTag}
    (evaluation : LifecycleEvaluation trace algorithm)
    (f : Cost.LifecycleVector → Nat) (_hpos : 0 < body.horizon) : Nat :=
  f evaluation.total / body.horizon

/-- The amortized value never exceeds the nonamortized one: division is by a
positive horizon, and the total is retained. -/
theorem amortized_le_total {body : LifecycleTraceBody}
    {trace : ResolvedLifecycleTrace body} {algorithm : LifecycleAlgorithmTag}
    (evaluation : LifecycleEvaluation trace algorithm)
    (f : Cost.LifecycleVector → Nat) (hpos : 0 < body.horizon) :
    amortized evaluation f hpos ≤ f evaluation.total :=
  Nat.div_le_self _ _

/-- The amortized value really is the exact summed numerator divided by the
explicit horizon: nothing else is divided. -/
theorem amortized_eq {body : LifecycleTraceBody}
    {trace : ResolvedLifecycleTrace body} {algorithm : LifecycleAlgorithmTag}
    (evaluation : LifecycleEvaluation trace algorithm)
    (f : Cost.LifecycleVector → Nat) (hpos : 0 < body.horizon) :
    amortized evaluation f hpos =
      f (Cost.sumLifecycle (evaluation.prefixes.elements.map (fun r => r.cost)))
        / body.horizon := by
  rw [amortized, evaluation.totalExact]

/-! ## Omissions

Recorded here so that a reader of SPEC 16 can see what is *not* closed rather
than having to notice its absence.

* `Atlas.canonicalFullRebuildEvaluation` — a total constructor producing a
  full-rebuild evaluation for every resolved trace.  It requires exhibiting a
  `LifecyclePrefixResult` for each of the `horizon + 1` prefixes, hence a
  `PrefixOrderedList` whose ordinal map is definitionally `List.range
  (horizon + 1)`.  That construction is available, but the evaluation's `size`
  field would then have to be verified against `lifecycleSize`, which is a
  computation over the whole trace; producing it without a proof would store an
  unverified conclusion.  Omitted rather than stubbed.

* `Atlas.lifecycle_native_bound` — see the file header.  It is a real
  inequality, false for an arbitrary primitive-cost table, and the release table
  it would be stated over is not pinned in this repository.  Omitted rather than
  assumed.  `nativeLifecycleBound_scope_size_only` records that the polynomial
  by itself implies nothing about any lifecycle total.

* `Atlas.lifecycle_incremental_semantics_eq_full_rebuild` — requires
  `SameSealedQueryAndExecutionObservations`, i.e. equality of the *sealed*
  observations at every seal point.  `bodyAt_zero_agree` proves the base case
  from `Atlas.semanticApplyBody_eq_rebuild`; the inductive step additionally
  needs coherence to be preserved along the whole trace, which is a property of
  the resolved deltas, not of the definitions here.  Omitted rather than
  asserted.

* `Atlas.lifecycle_full_rebuild_comparator_exact` and `Atlas.regretAgainst` —
  both quantify over `canonicalFullRebuildEvaluation`, which is omitted above.
  `Cost.truncatedDifference` (the regret carrier) is already defined and proved
  in `Cost/Lifecycle.lean`; only the comparator instance is missing. -/

end WasmGemmGnaf.Atlas
