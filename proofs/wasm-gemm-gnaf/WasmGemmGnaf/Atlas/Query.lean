import WasmGemmGnaf.Atlas.Rebuild
set_option autoImplicit false

/-!
# Atlas: query (SPEC §12.5, UOR-GNAF §10.9)

SPEC §12.5: "Query and artifact emission SHALL use only a `SealedState`, so
stale or partially merged evidence is never promoted."  `Atlas.query` therefore
takes a `SealedState` and nothing else; there is no overload taking an
`UnsealedState`, and no way to construct a `SealedState` without the seven
proved seal conditions of `Atlas/Seal.lean`.

## The answer algebra

`Atlas.QueryResult` carries the branches UOR-GNAF §10.9 requires an answer
algebra to have, and in particular the two that §13.3 singles out:

* `unattained` — "A valid profile without a minimum may prove and return
  `unattained`";
* `optimizationIncomplete` — "incomplete evidence returns
  `optimization-incomplete`";

together with `workloadIncomplete` (§10.9's `WorkloadIncomplete`), which is the
answer whenever the sealed envelope has no cell for the requested region.  A
query that cannot answer is *required* to say so rather than to fall back on an
optimum.

## What `.optimal` means, and what it does not

`Atlas.sealed_query_sound` proves that an `.optimal` answer carries a
certificate satisfying `Atlas.QueryCertificate.Verifies`, every conjunct of
which is recomputable from the sealed state: the state identity, the three scope
identities, the envelope root, a recorded envelope cell whose recorded attained
candidate is the selected one, the candidate's recorded score, and — the
strongest conjunct — `Atlas.IsAttainedMinimum` over the recorded partition
cover.

`Atlas.query_optimal_scope_recorded_only` proves the matching negative: the
verification is a function of the recorded components alone.  An `.optimal`
answer is therefore a statement about the sealed record, **not** a statement
about every byte string; the universal claim would have to come from the
coverage layer, and `Atlas.universalCoverCompleteCheck_scope_blind` in
`Atlas/CoverageScope.lean` proves the seal's cover check cannot supply it.
-/

namespace WasmGemmGnaf.Atlas

open WasmGemmGnaf.Foundation

/-! ## Requests -/

/-- The answer shape a request asks for (UOR-GNAF §10.9). -/
inductive AnswerShape
  | scalar
  | paretoFrontier
  deriving DecidableEq, Inhabited

/-- A canonical, first-order query request. -/
structure QueryRequest where
  profileId : ProfileId
  problemId : ProblemId
  objectiveId : ObjectiveId
  regionId : CanonicalObjectId
  shape : AnswerShape
  deriving DecidableEq

/-! ## Query certificates -/

/-- The verifiable syntactic components of a query answer.  Every field is data
the verifier recomputes from the sealed state. -/
structure QueryCertificate where
  stateId : StateIdentity
  profileId : ProfileId
  problemId : ProblemId
  objectiveId : ObjectiveId
  regionId : CanonicalObjectId
  candidate : CanonicalObjectId
  score : Nat
  envelopeRoot : EnvelopeRoot
  deriving DecidableEq

/-- **The verification relation.**  Each conjunct is a recomputation against the
sealed state; none of them is a stored conclusion. -/
def QueryCertificate.Verifies (c : QueryCertificate) (selected : CanonicalObjectId)
    (request : QueryRequest) (sealed : SealedState) : Prop :=
  c.candidate = selected ∧
  c.stateId = sealed.state.bodyId ∧
  c.stateId = sealed.core.stateId ∧
  c.profileId = sealed.core.profileId ∧
  c.profileId = request.profileId ∧
  c.problemId = sealed.core.problemId ∧
  c.problemId = request.problemId ∧
  c.objectiveId = sealed.core.objectiveId ∧
  c.objectiveId = request.objectiveId ∧
  c.regionId = request.regionId ∧
  c.envelopeRoot = sealed.core.envelopeRoot ∧
  (∃ r, sealed.state.body.lowerEnvelope.region? c.regionId = some r ∧
    r.status = EnvelopeStatus.attainedMinimum ∧
    r.attained = some selected ∧ r.bound = c.score) ∧
  selected ∈ sealed.state.body.candidateFacts.keys ∧
  sealed.state.body.costSurfaces.score? selected = some c.score ∧
  IsAttainedMinimum sealed.state.body.costSurfaces
    sealed.state.body.searchPartitions.covered selected c.score

/-- The decidable recomputation of the parts of `Verifies` that are not fixed by
construction. -/
def certificateCheck (sealed : SealedState) (selected : CanonicalObjectId)
    (score : Nat) : Bool :=
  memId sealed.state.body.candidateFacts.keys selected &&
  memId sealed.state.body.searchPartitions.covered selected &&
  decide (sealed.state.body.costSurfaces.score? selected = some score) &&
  decide (sealed.state.body.costSurfaces.minScore?
    sealed.state.body.searchPartitions.covered = some score)

/-- The checker really does establish the attainment conjuncts. -/
theorem certificateCheck_sound {sealed : SealedState} {selected : CanonicalObjectId}
    {score : Nat} (h : certificateCheck sealed selected score = true) :
    selected ∈ sealed.state.body.candidateFacts.keys ∧
    sealed.state.body.costSurfaces.score? selected = some score ∧
    IsAttainedMinimum sealed.state.body.costSurfaces
      sealed.state.body.searchPartitions.covered selected score := by
  simp only [certificateCheck, Bool.and_eq_true, decide_eq_true_eq] at h
  obtain ⟨⟨⟨hcand, hcov⟩, hscore⟩, hmin⟩ := h
  exact ⟨(memId_iff _ _).mp hcand, hscore,
    attained_of_minScore? ((memId_iff _ _).mp hcov) hscore hmin⟩

/-! ## The answer algebra (UOR-GNAF §10.9, §13.3) -/

/-- The closed, payload-bearing result of a query on a sealed state. -/
inductive QueryResult
  /-- A scalar optimum, with the certificate that verifies it. -/
  | optimal (selected : CanonicalObjectId) (certificate : QueryCertificate)
  /-- The complete Pareto frontier of the recorded cover. -/
  | paretoFrontier (frontier : List CanonicalObjectId)
  /-- The region is infeasible. -/
  | infeasible (region : CanonicalObjectId) (searchBound : Nat)
  /-- **UOR-GNAF §13.3 `unattained`**: an infimum that is not attained. -/
  | unattained (region : CanonicalObjectId) (infimum : Nat)
  /-- **UOR-GNAF §13.3 `optimization-incomplete`**: the recorded cell claims an
  optimum but the recomputation does not support it. -/
  | optimizationIncomplete (region : CanonicalObjectId)
      (outstanding : List CanonicalObjectId)
  /-- **UOR-GNAF §10.9 `WorkloadIncomplete`**: the sealed evidence does not
  cover the requested region. -/
  | workloadIncomplete (region : CanonicalObjectId)
      (outstanding : List CanonicalObjectId)
  /-- The request names a profile the seal was not built for. -/
  | unsupportedProfile (requested available : ProfileId)
  /-- The request names a problem or objective the seal was not built for. -/
  | scopeMismatch (requested available : CanonicalObjectId)
  /-- The recorded cell does not recompute against the sealed state. -/
  | invalidatedOrUnsealed (stateId : StateIdentity)
  deriving DecidableEq

namespace QueryResult

/-- Structural index, used to prove the branches do not collapse. -/
def index : QueryResult → Nat
  | optimal _ _ => 0
  | paretoFrontier _ => 1
  | infeasible _ _ => 2
  | unattained _ _ => 3
  | optimizationIncomplete _ _ => 4
  | workloadIncomplete _ _ => 5
  | unsupportedProfile _ _ => 6
  | scopeMismatch _ _ => 7
  | invalidatedOrUnsealed _ => 8

theorem ne_of_index_ne {a b : QueryResult} (h : a.index ≠ b.index) : a ≠ b :=
  fun hab => h (hab ▸ rfl)

/-- An optimum is never any of the incomplete, unattained or infeasible
answers. -/
theorem optimal_ne_unattained (s : CanonicalObjectId) (c : QueryCertificate)
    (r : CanonicalObjectId) (i : Nat) : optimal s c ≠ unattained r i :=
  ne_of_index_ne (by simp [index])

theorem optimal_ne_workloadIncomplete (s : CanonicalObjectId) (c : QueryCertificate)
    (r : CanonicalObjectId) (o : List CanonicalObjectId) :
    optimal s c ≠ workloadIncomplete r o :=
  ne_of_index_ne (by simp [index])

theorem optimal_ne_optimizationIncomplete (s : CanonicalObjectId) (c : QueryCertificate)
    (r : CanonicalObjectId) (o : List CanonicalObjectId) :
    optimal s c ≠ optimizationIncomplete r o :=
  ne_of_index_ne (by simp [index])

theorem unattained_ne_optimizationIncomplete (r : CanonicalObjectId) (i : Nat)
    (r' : CanonicalObjectId) (o : List CanonicalObjectId) :
    unattained r i ≠ optimizationIncomplete r' o :=
  ne_of_index_ne (by simp [index])

theorem workloadIncomplete_ne_optimizationIncomplete (r : CanonicalObjectId)
    (o : List CanonicalObjectId) (r' : CanonicalObjectId)
    (o' : List CanonicalObjectId) :
    workloadIncomplete r o ≠ optimizationIncomplete r' o' :=
  ne_of_index_ne (by simp [index])

end QueryResult

/-! ## `Atlas.query` (SPEC §12.5) -/

/-- The certificate a scalar optimum would carry. -/
def queryCertificateOf (sealed : SealedState) (request : QueryRequest)
    (candidate : CanonicalObjectId) (score : Nat) : QueryCertificate where
  stateId := sealed.state.bodyId
  profileId := request.profileId
  problemId := request.problemId
  objectiveId := request.objectiveId
  regionId := request.regionId
  candidate := candidate
  score := score
  envelopeRoot := sealed.core.envelopeRoot

/-- The scalar branch: the envelope answer, refined by the recomputation of the
attainment conjuncts. -/
def scalarAnswer (sealed : SealedState) (request : QueryRequest) : QueryResult :=
  match Envelope.evaluate sealed.state.body request.profileId request.regionId with
  | .attainedMinimum c s =>
      if certificateCheck sealed c s then
        .optimal c (queryCertificateOf sealed request c s)
      else .optimizationIncomplete request.regionId [c]
  | .infeasibleRegion r b => .infeasible r b
  | .nonattainedInfimum r i => .unattained r i
  | .incompleteCoverage r out => .workloadIncomplete r out
  | .unsupportedProfile p q => .unsupportedProfile p q
  | .invalidatedOrUnsealed sid => .invalidatedOrUnsealed sid

/-- **SPEC §12.5**, `Atlas.query`.  It takes a `SealedState`; there is no
unsealed entry point. -/
def query (sealed : SealedState) (request : QueryRequest) : QueryResult :=
  if request.profileId ≠ sealed.core.profileId then
    .unsupportedProfile request.profileId sealed.core.profileId
  else if request.problemId ≠ sealed.core.problemId then
    .scopeMismatch request.problemId sealed.core.problemId
  else if request.objectiveId ≠ sealed.core.objectiveId then
    .scopeMismatch request.objectiveId sealed.core.objectiveId
  else
    match request.shape with
    | .paretoFrontier =>
        .paretoFrontier (sealed.state.body.costSurfaces.frontier
          sealed.state.body.searchPartitions.covered)
    | .scalar => scalarAnswer sealed request

/-! ### Defining equations -/

theorem query_of_profile_ne {sealed : SealedState} {request : QueryRequest}
    (h : request.profileId ≠ sealed.core.profileId) :
    query sealed request =
      .unsupportedProfile request.profileId sealed.core.profileId := by
  simp [query, h]

theorem query_scalar {sealed : SealedState} {request : QueryRequest}
    (hp : request.profileId = sealed.core.profileId)
    (hq : request.problemId = sealed.core.problemId)
    (ho : request.objectiveId = sealed.core.objectiveId)
    (hs : request.shape = AnswerShape.scalar) :
    query sealed request = scalarAnswer sealed request := by
  simp [query, hp, hq, ho, hs]

theorem query_pareto {sealed : SealedState} {request : QueryRequest}
    (hp : request.profileId = sealed.core.profileId)
    (hq : request.problemId = sealed.core.problemId)
    (ho : request.objectiveId = sealed.core.objectiveId)
    (hs : request.shape = AnswerShape.paretoFrontier) :
    query sealed request =
      .paretoFrontier (sealed.state.body.costSurfaces.frontier
        sealed.state.body.searchPartitions.covered) := by
  simp [query, hp, hq, ho, hs]

/-! ## Soundness -/

/-- **SPEC §12.5**, `sealed_query_sound`: an `.optimal` answer carries a
certificate that verifies against the sealed state. -/
theorem sealed_query_sound {sealed : SealedState} {request : QueryRequest}
    {selected : CanonicalObjectId} {certificate : QueryCertificate}
    (hquery : query sealed request = .optimal selected certificate) :
    certificate.Verifies selected request sealed := by
  rw [query] at hquery
  split at hquery
  · exact absurd hquery (by simp)
  · rename_i hprofile
    split at hquery
    · exact absurd hquery (by simp)
    · rename_i hproblem
      split at hquery
      · exact absurd hquery (by simp)
      · rename_i hobjective
        split at hquery
        · exact absurd hquery (by simp)
        · rw [scalarAnswer] at hquery
          split at hquery
          · rename_i c s henv
            split at hquery
            · rename_i hcheck
              obtain ⟨hmem, hscore, hmin⟩ := certificateCheck_sound hcheck
              obtain ⟨hcsel, hcert⟩ :
                  c = selected ∧ queryCertificateOf sealed request c s = certificate := by
                simpa using hquery
              subst hcsel
              subst hcert
              obtain ⟨_, r, hr, hstatus, hatt, hbound, _, _⟩ :=
                Envelope.evaluate_attainedMinimum_sound henv
              exact ⟨rfl, rfl,
                sealed.state.bodyIdEq.trans (SealedState.core_stateId sealed).symm,
                (by simpa using hprofile), rfl,
                (by simpa using hproblem), rfl,
                (by simpa using hobjective), rfl, rfl, rfl,
                ⟨r, hr, hstatus, hatt, hbound⟩, hmem, hscore, hmin⟩
            · exact absurd hquery (by simp)
          · exact absurd hquery (by simp)
          · exact absurd hquery (by simp)
          · exact absurd hquery (by simp)
          · exact absurd hquery (by simp)
          · exact absurd hquery (by simp)

/-- An `.optimal` answer is in particular an exact attained minimum over the
recorded partition cover. -/
theorem sealed_query_optimal_is_attained_minimum {sealed : SealedState}
    {request : QueryRequest} {selected : CanonicalObjectId}
    {certificate : QueryCertificate}
    (hquery : query sealed request = .optimal selected certificate) :
    IsAttainedMinimum sealed.state.body.costSurfaces
      sealed.state.body.searchPartitions.covered selected certificate.score :=
  (sealed_query_sound hquery).2.2.2.2.2.2.2.2.2.2.2.2.2.2

/-- A `.paretoFrontier` answer is the *complete* nondominated set of the
recorded cover: membership is an exact characterisation, in both directions. -/
theorem sealed_query_pareto_complete {sealed : SealedState} {request : QueryRequest}
    {frontier : List CanonicalObjectId}
    (hquery : query sealed request = .paretoFrontier frontier)
    (c : CanonicalObjectId) :
    c ∈ frontier ↔
      c ∈ sealed.state.body.searchPartitions.covered ∧
      ∀ d ∈ sealed.state.body.searchPartitions.covered,
        ¬ Dominates (sealed.state.body.costSurfaces.coordinates d)
          (sealed.state.body.costSurfaces.coordinates c) := by
  have hfr : sealed.state.body.costSurfaces.frontier
      sealed.state.body.searchPartitions.covered = frontier := by
    rw [query] at hquery
    split at hquery
    · exact absurd hquery (by simp)
    · split at hquery
      · exact absurd hquery (by simp)
      · split at hquery
        · exact absurd hquery (by simp)
        · split at hquery
          · simpa using hquery
          · exfalso
            rw [scalarAnswer] at hquery
            split at hquery
            · split at hquery <;> exact absurd hquery (by simp)
            · exact absurd hquery (by simp)
            · exact absurd hquery (by simp)
            · exact absurd hquery (by simp)
            · exact absurd hquery (by simp)
            · exact absurd hquery (by simp)
  rw [← hfr]
  exact CostSurfaceMap.mem_frontier_iff _ _ c

/-! ## Incompleteness is reachable and is reported

A query that cannot answer must say so.  The two lemmas below are the exact
routes to `workloadIncomplete`, and `query_workloadIncomplete_reachable`
exhibits a genuine sealed state on which the answer really is
`workloadIncomplete` — so the branch is not decoration. -/

theorem query_workloadIncomplete_of_missing_region {sealed : SealedState}
    {request : QueryRequest}
    (hp : request.profileId = sealed.core.profileId)
    (hq : request.problemId = sealed.core.problemId)
    (ho : request.objectiveId = sealed.core.objectiveId)
    (hs : request.shape = AnswerShape.scalar)
    (hbody : request.profileId = sealed.state.body.profileId)
    (hregion : sealed.state.body.lowerEnvelope.region? request.regionId = none) :
    query sealed request = .workloadIncomplete request.regionId [request.regionId] := by
  rw [query_scalar hp hq ho hs, scalarAnswer,
    Envelope.evaluate_of_region_none hbody hregion]

/-- The sealed state produced by a rebuild has an empty envelope, so every
scalar query against it is answered `workloadIncomplete`.  This is the witness
that the branch is reachable. -/
theorem query_workloadIncomplete_reachable :
    query (derivedSealedState Scope.unscoped ⟨[]⟩)
        ⟨nullId, nullId, nullId, nullId, AnswerShape.scalar⟩ =
      .workloadIncomplete nullId [nullId] := by
  refine query_workloadIncomplete_of_missing_region rfl rfl rfl rfl rfl ?_
  rfl

/-! ## Scope: what an `.optimal` answer does not establish

`QueryCertificate.Verifies` mentions exactly the state identity, the core's
scope identities and envelope root, and four recorded components of the state
body.  Two sealed states agreeing on those satisfy it for the same certificates
— *whatever* byte strings exist, decode, validate or compute GEMM.  No
proposition quantified over `ByteArray` therefore follows from
`sealed_query_sound` alone. -/
theorem query_optimal_scope_recorded_only (s₁ s₂ : SealedState)
    (c : QueryCertificate) (selected : CanonicalObjectId) (request : QueryRequest)
    (hid : s₁.state.bodyId = s₂.state.bodyId)
    (hcore : s₁.core = s₂.core)
    (henv : s₁.state.body.lowerEnvelope = s₂.state.body.lowerEnvelope)
    (hcand : s₁.state.body.candidateFacts = s₂.state.body.candidateFacts)
    (hcost : s₁.state.body.costSurfaces = s₂.state.body.costSurfaces)
    (hpart : s₁.state.body.searchPartitions = s₂.state.body.searchPartitions) :
    c.Verifies selected request s₁ ↔ c.Verifies selected request s₂ := by
  simp only [QueryCertificate.Verifies, hid, hcore, henv, hcand, hcost, hpart]

end WasmGemmGnaf.Atlas
