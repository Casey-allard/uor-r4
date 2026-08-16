import WasmGemmGnaf.Atlas.CostSurface
import WasmGemmGnaf.Atlas.Certificate
set_option autoImplicit false

/-!
# Atlas: the lower envelope (SPEC §12.4)

SPEC §12.4 requires the envelope to **distinguish** six outcomes:

1. attained minimum;
2. infeasible region;
3. nonattained infimum;
4. incomplete coverage;
5. unsupported profile;
6. invalidated or unsealed state.

`Atlas.EnvelopeStatus` (declared in `Atlas/State.lean`, because the state body
stores it) already has exactly six constructors.  This file discharges the
obligation that they really are six: `Atlas.EnvelopeStatus.pairwise_distinct`
plus the fifteen individual disequations, and `card_eq_six`.

The reason this matters is UOR-GNAF §13.3: "A valid profile without a minimum
may prove and return `unattained`; incomplete evidence returns
`optimization-incomplete`."  A representation that collapsed
`nonattainedInfimum` or `incompleteCoverage` into `attainedMinimum` would
report an optimum that was never proved.

`Atlas.EnvelopeAnswer` is the payload-bearing form: six constructors, each
carrying the evidence its status is about, with `status` mapping onto
`EnvelopeStatus` and `status_surjective` proving that every status is
reachable.  `Atlas.Envelope.evaluate` is the exact evaluator; it is **not** a
bookkeeping check:

* `evaluate_attainedMinimum_sound` shows that an `attainedMinimum` answer forces
  a recorded region whose recorded candidate carries exactly the recorded
  score, so the answer cannot be produced without the recomputation succeeding;
* every non-exact region is reported as `invalidatedOrUnsealed`, never as an
  optimum (`evaluate_not_exact`).

What `evaluate` does **not** establish is stated as a proved scope lemma,
`evaluate_scope_recorded_only`: the answer is a function of the recorded
envelope, candidate facts, cost surface and profile identity alone, so no
proposition quantified over byte strings follows from it.
-/

namespace WasmGemmGnaf.Atlas

open WasmGemmGnaf.Foundation

/-! ## A canonical identifier that no schema can produce

Used only to build the witnesses of the reachability theorems below.  Every
`CanonicalSchema` has a nonempty type tag, so `nullId` is provably not the
identity of anything. -/

/-- The empty canonical identifier. -/
def nullId : CanonicalObjectId :=
  { schemaVersion := 0
    domain := CanonicalDomainTag.generic
    typeTag := ByteArray.empty
    canonicalBodyBytes := ByteArray.empty }

theorem nullId_typeTag_size : nullId.typeTag.size = 0 := rfl

/-- No schema identity is `nullId`: schemas have nonempty type tags. -/
theorem nullId_ne_identity {α : Type} (schema : CanonicalSchema α) (a : α) :
    CanonicalObjectId.ofTyped (Identity schema a) ≠ nullId := by
  intro h
  have := congrArg CanonicalObjectId.typeTag h
  have hpos := schema.typeTagNonempty
  simp only [CanonicalObjectId.ofTyped, Identity] at this
  rw [this] at hpos
  exact absurd hpos (by simp [nullId_typeTag_size])

/-! ## The six statuses are six (SPEC §12.4) -/

namespace EnvelopeStatus

theorem attainedMinimum_ne_infeasibleRegion :
    attainedMinimum ≠ infeasibleRegion := by decide
theorem attainedMinimum_ne_nonattainedInfimum :
    attainedMinimum ≠ nonattainedInfimum := by decide
theorem attainedMinimum_ne_incompleteCoverage :
    attainedMinimum ≠ incompleteCoverage := by decide
theorem attainedMinimum_ne_unsupportedProfile :
    attainedMinimum ≠ unsupportedProfile := by decide
theorem attainedMinimum_ne_invalidatedOrUnsealed :
    attainedMinimum ≠ invalidatedOrUnsealed := by decide
theorem infeasibleRegion_ne_nonattainedInfimum :
    infeasibleRegion ≠ nonattainedInfimum := by decide
theorem infeasibleRegion_ne_incompleteCoverage :
    infeasibleRegion ≠ incompleteCoverage := by decide
theorem infeasibleRegion_ne_unsupportedProfile :
    infeasibleRegion ≠ unsupportedProfile := by decide
theorem infeasibleRegion_ne_invalidatedOrUnsealed :
    infeasibleRegion ≠ invalidatedOrUnsealed := by decide
theorem nonattainedInfimum_ne_incompleteCoverage :
    nonattainedInfimum ≠ incompleteCoverage := by decide
theorem nonattainedInfimum_ne_unsupportedProfile :
    nonattainedInfimum ≠ unsupportedProfile := by decide
theorem nonattainedInfimum_ne_invalidatedOrUnsealed :
    nonattainedInfimum ≠ invalidatedOrUnsealed := by decide
theorem incompleteCoverage_ne_unsupportedProfile :
    incompleteCoverage ≠ unsupportedProfile := by decide
theorem incompleteCoverage_ne_invalidatedOrUnsealed :
    incompleteCoverage ≠ invalidatedOrUnsealed := by decide
theorem unsupportedProfile_ne_invalidatedOrUnsealed :
    unsupportedProfile ≠ invalidatedOrUnsealed := by decide

/-- **SPEC §12.4**: the six statuses are pairwise distinct — no two of them
collapse.  Stated as duplicate-freeness of the complete enumeration, which is
equivalent to the fifteen disequations above and covers them uniformly. -/
theorem pairwise_distinct : all.Pairwise (· ≠ ·) := by decide

/-- There are exactly six statuses. -/
theorem card_eq_six : Fintype.card EnvelopeStatus = 6 := rfl

/-- Distinct statuses have distinct canonical encodings, so the distinction
survives serialisation into the state body and the seal. -/
theorem bytes_ne_of_ne {a b : EnvelopeStatus} (h : a ≠ b) : bytes a ≠ bytes b :=
  fun hb => h (bytes_injective hb)

end EnvelopeStatus

/-! ## The payload-bearing envelope answer -/

/-- The six envelope outcomes of SPEC §12.4, each carrying the evidence its
status is about.  These are six distinct constructors: an `attainedMinimum`
answer can never be confused with a `nonattainedInfimum` or an
`incompleteCoverage` one. -/
inductive EnvelopeAnswer
  /-- The region's minimum is attained by `candidate`, whose recorded score is
  `score`. -/
  | attainedMinimum (candidate : CanonicalObjectId) (score : Nat)
  /-- The region is infeasible; `searchBound` is the bound up to which the
  region was exhausted. -/
  | infeasibleRegion (region : CanonicalObjectId) (searchBound : Nat)
  /-- The region has an infimum that is approached but not attained
  (UOR-GNAF §13.3 `unattained`). -/
  | nonattainedInfimum (region : CanonicalObjectId) (infimum : Nat)
  /-- Coverage of the region is incomplete (UOR-GNAF §13.3
  `optimization-incomplete`); the outstanding identities are listed. -/
  | incompleteCoverage (region : CanonicalObjectId) (outstanding : List CanonicalObjectId)
  /-- The requested profile is not the profile the envelope was computed for. -/
  | unsupportedProfile (requested : ProfileId) (available : ProfileId)
  /-- The recorded region does not recompute, so the state is invalidated or
  unsealed with respect to it. -/
  | invalidatedOrUnsealed (stateId : StateIdentity)
  deriving DecidableEq

namespace EnvelopeAnswer

/-- The status an answer reports. -/
def status : EnvelopeAnswer → EnvelopeStatus
  | attainedMinimum _ _ => .attainedMinimum
  | infeasibleRegion _ _ => .infeasibleRegion
  | nonattainedInfimum _ _ => .nonattainedInfimum
  | incompleteCoverage _ _ => .incompleteCoverage
  | unsupportedProfile _ _ => .unsupportedProfile
  | invalidatedOrUnsealed _ => .invalidatedOrUnsealed

@[simp] theorem status_attainedMinimum (c : CanonicalObjectId) (s : Nat) :
    (attainedMinimum c s).status = .attainedMinimum := rfl
@[simp] theorem status_infeasibleRegion (r : CanonicalObjectId) (s : Nat) :
    (infeasibleRegion r s).status = .infeasibleRegion := rfl
@[simp] theorem status_nonattainedInfimum (r : CanonicalObjectId) (s : Nat) :
    (nonattainedInfimum r s).status = .nonattainedInfimum := rfl
@[simp] theorem status_incompleteCoverage (r : CanonicalObjectId)
    (o : List CanonicalObjectId) : (incompleteCoverage r o).status = .incompleteCoverage := rfl
@[simp] theorem status_unsupportedProfile (p q : ProfileId) :
    (unsupportedProfile p q).status = .unsupportedProfile := rfl
@[simp] theorem status_invalidatedOrUnsealed (i : StateIdentity) :
    (invalidatedOrUnsealed i).status = .invalidatedOrUnsealed := rfl

/-- Answers with different statuses are different answers. -/
theorem ne_of_status_ne {a b : EnvelopeAnswer} (h : a.status ≠ b.status) : a ≠ b :=
  fun hab => h (hab ▸ rfl)

/-- Every status is the status of some answer: none of the six is a dead
letter. -/
theorem status_surjective (s : EnvelopeStatus) : ∃ a : EnvelopeAnswer, a.status = s := by
  cases s with
  | attainedMinimum => exact ⟨attainedMinimum nullId 0, rfl⟩
  | infeasibleRegion => exact ⟨infeasibleRegion nullId 0, rfl⟩
  | nonattainedInfimum => exact ⟨nonattainedInfimum nullId 0, rfl⟩
  | incompleteCoverage => exact ⟨incompleteCoverage nullId [], rfl⟩
  | unsupportedProfile => exact ⟨unsupportedProfile nullId nullId, rfl⟩
  | invalidatedOrUnsealed => exact ⟨invalidatedOrUnsealed nullId, rfl⟩

/-- An attained minimum is never reported as anything else, and nothing else is
ever reported as an attained minimum. -/
theorem attainedMinimum_ne_of_other {a : EnvelopeAnswer}
    (h : a.status ≠ EnvelopeStatus.attainedMinimum) (c : CanonicalObjectId) (s : Nat) :
    a ≠ attainedMinimum c s :=
  ne_of_status_ne (by simpa using h)

end EnvelopeAnswer

/-! ## Exact region lookup and exactness -/

namespace LowerEnvelopeBody

/-- The recorded region with a given identity, if any. -/
def region? (e : LowerEnvelopeBody) (id : CanonicalObjectId) : Option EnvelopeRegion :=
  e.regions.find? (fun r => decide (r.regionId = id))

theorem region?_sound {e : LowerEnvelopeBody} {id : CanonicalObjectId}
    {r : EnvelopeRegion} (h : e.region? id = some r) :
    r ∈ e.regions ∧ r.regionId = id := by
  obtain ⟨hmem, hp⟩ := Find.sound _ e.regions r h
  exact ⟨hmem, by simpa using hp⟩

end LowerEnvelopeBody

/-- `Atlas.envelopeRegionExact` of `Atlas/Certificate.lean`, stated on the
state *body* so that it can be evaluated without a retention proof. -/
def regionExact (b : StateBody) (r : EnvelopeRegion) : Bool :=
  match r.status, r.attained with
  | EnvelopeStatus.attainedMinimum, some c =>
      memId b.candidateFacts.keys c &&
      decide (b.costSurfaces.score? c = some r.bound)
  | EnvelopeStatus.attainedMinimum, none => false
  | _, some _ => false
  | _, none => true

/-- The body-level exactness test is *the* seal-level test of
`Atlas/Certificate.lean`, not a weaker copy. -/
theorem regionExact_eq_envelopeRegionExact (s : UnsealedState) (r : EnvelopeRegion) :
    envelopeRegionExact s r = regionExact s.body r := rfl

/-- An exact `attainedMinimum` region names a recorded candidate carrying
exactly the recorded bound. -/
theorem regionExact_attained {b : StateBody} {r : EnvelopeRegion}
    (hstatus : r.status = EnvelopeStatus.attainedMinimum)
    (h : regionExact b r = true) :
    ∃ c, r.attained = some c ∧ c ∈ b.candidateFacts.keys ∧
      b.costSurfaces.score? c = some r.bound := by
  cases hatt : r.attained with
  | none => rw [regionExact, hstatus, hatt] at h; exact absurd h (by simp)
  | some c =>
    rw [regionExact, hstatus, hatt] at h
    simp only [Bool.and_eq_true, decide_eq_true_eq] at h
    exact ⟨c, rfl, (memId_iff _ _).mp h.1, h.2⟩

/-- An exact region that is *not* an attained minimum names no candidate: a
nonattained infimum, an infeasible region or an incomplete cell can never smuggle
in a "selected" candidate. -/
theorem regionExact_not_attained {b : StateBody} {r : EnvelopeRegion}
    (hstatus : r.status ≠ EnvelopeStatus.attainedMinimum)
    (h : regionExact b r = true) : r.attained = none := by
  cases hatt : r.attained with
  | none => rfl
  | some c =>
    rw [regionExact, hatt] at h
    cases hs : r.status <;> rw [hs] at h <;> simp at h
    exact absurd hs hstatus

/-! ## The exact envelope evaluator -/

namespace Envelope

/-- The answer determined by an exact region. -/
def answerOfRegion (b : StateBody) (r : EnvelopeRegion) : EnvelopeAnswer :=
  match r.status, r.attained with
  | EnvelopeStatus.attainedMinimum, some c => .attainedMinimum c r.bound
  | EnvelopeStatus.attainedMinimum, none => .invalidatedOrUnsealed (StateId b)
  | EnvelopeStatus.infeasibleRegion, _ => .infeasibleRegion r.regionId r.bound
  | EnvelopeStatus.nonattainedInfimum, _ => .nonattainedInfimum r.regionId r.bound
  | EnvelopeStatus.incompleteCoverage, _ =>
      .incompleteCoverage r.regionId b.candidateFacts.keys
  | EnvelopeStatus.unsupportedProfile, _ => .unsupportedProfile b.profileId b.profileId
  | EnvelopeStatus.invalidatedOrUnsealed, _ => .invalidatedOrUnsealed (StateId b)

/-- **SPEC §12.4**, the envelope evaluation.

The profile is checked first, then the region is looked up, then the region is
*recomputed*.  A region that does not recompute is reported as
`invalidatedOrUnsealed`; it is never promoted to an optimum. -/
def evaluate (b : StateBody) (profile : ProfileId) (regionId : CanonicalObjectId) :
    EnvelopeAnswer :=
  if profile = b.profileId then
    match b.lowerEnvelope.region? regionId with
    | none => .incompleteCoverage regionId [regionId]
    | some r =>
        if regionExact b r then answerOfRegion b r
        else .invalidatedOrUnsealed (StateId b)
  else .unsupportedProfile profile b.profileId

/-! ### Defining equations -/

theorem evaluate_of_profile_ne {b : StateBody} {profile : ProfileId}
    (regionId : CanonicalObjectId) (h : profile ≠ b.profileId) :
    evaluate b profile regionId = .unsupportedProfile profile b.profileId := by
  simp [evaluate, h]

theorem evaluate_of_region_none {b : StateBody} {profile : ProfileId}
    {regionId : CanonicalObjectId} (hp : profile = b.profileId)
    (h : b.lowerEnvelope.region? regionId = none) :
    evaluate b profile regionId = .incompleteCoverage regionId [regionId] := by
  simp [evaluate, hp, h]

theorem evaluate_of_region_exact {b : StateBody} {profile : ProfileId}
    {regionId : CanonicalObjectId} {r : EnvelopeRegion} (hp : profile = b.profileId)
    (h : b.lowerEnvelope.region? regionId = some r) (hex : regionExact b r = true) :
    evaluate b profile regionId = answerOfRegion b r := by
  simp [evaluate, hp, h, hex]

/-- A region that fails to recompute is reported as invalidated, never as an
optimum. -/
theorem evaluate_not_exact {b : StateBody} {profile : ProfileId}
    {regionId : CanonicalObjectId} {r : EnvelopeRegion} (hp : profile = b.profileId)
    (h : b.lowerEnvelope.region? regionId = some r) (hex : regionExact b r = false) :
    evaluate b profile regionId = .invalidatedOrUnsealed (StateId b) := by
  simp [evaluate, hp, h, hex]

/-! ### Soundness -/

/-- **The load-bearing soundness statement of the envelope.**

An `attainedMinimum` answer is only ever produced from a recorded region of the
recorded profile whose recorded candidate is a recorded candidate fact with
exactly the recorded score.  Every component of the answer is recomputable from
the state body. -/
theorem evaluate_attainedMinimum_sound {b : StateBody} {profile : ProfileId}
    {regionId : CanonicalObjectId} {c : CanonicalObjectId} {s : Nat}
    (h : evaluate b profile regionId = .attainedMinimum c s) :
    profile = b.profileId ∧
    ∃ r, b.lowerEnvelope.region? regionId = some r ∧
      r.status = EnvelopeStatus.attainedMinimum ∧
      r.attained = some c ∧ r.bound = s ∧
      c ∈ b.candidateFacts.keys ∧
      b.costSurfaces.score? c = some s := by
  rw [evaluate] at h
  split at h
  · rename_i hp
    split at h
    · exact absurd h (by simp)
    · rename_i r hr
      split at h
      · rename_i hex
        refine ⟨hp, r, hr, ?_⟩
        rw [answerOfRegion] at h
        split at h
        · rename_i c' hstatus hatt
          have hcb : c' = c ∧ r.bound = s := by simpa using h
          have hc : c' = c := hcb.1
          have hb : r.bound = s := hcb.2
          subst hc
          obtain ⟨c'', hatt'', hmem, hscore⟩ := regionExact_attained hstatus hex
          rw [hatt] at hatt''
          have : c'' = c' := by simpa using hatt''.symm
          subst this
          exact ⟨hstatus, hatt, hb, hmem, hb ▸ hscore⟩
        · exact absurd h (by simp)
        · exact absurd h (by simp)
        · exact absurd h (by simp)
        · exact absurd h (by simp)
        · exact absurd h (by simp)
        · exact absurd h (by simp)
      · exact absurd h (by simp)
  · exact absurd h (by simp)

/-- The status reported for an exact region is exactly the region's recorded
status: the evaluator neither upgrades nor downgrades a recorded cell. -/
theorem evaluate_status_of_exact {b : StateBody} {profile : ProfileId}
    {regionId : CanonicalObjectId} {r : EnvelopeRegion} (hp : profile = b.profileId)
    (h : b.lowerEnvelope.region? regionId = some r) (hex : regionExact b r = true) :
    (evaluate b profile regionId).status = r.status := by
  rw [evaluate_of_region_exact hp h hex, answerOfRegion]
  split
  · rename_i hstatus _; simp [hstatus]
  · rename_i hstatus hatt
    rw [regionExact, hstatus, hatt] at hex
    exact absurd hex (by simp)
  · rename_i hstatus; simp [hstatus]
  · rename_i hstatus; simp [hstatus]
  · rename_i hstatus; simp [hstatus]
  · rename_i hstatus; simp [hstatus]
  · rename_i hstatus; simp [hstatus]

/-! ### Scope: what the evaluator does not establish

`evaluate` reads exactly four components of the state body.  Two bodies that
agree on them give the same answer *whatever* byte strings exist, decode,
validate or compute GEMM.  Consequently no proposition quantified over
`ByteArray` follows from an `attainedMinimum` answer: it is a statement about
the recorded surface, and the universal claim must come from
`Universal/`-level coverage, not from here. -/
theorem evaluate_scope_recorded_only (b₁ b₂ : StateBody) (profile regionId : CanonicalObjectId)
    (hp : b₁.profileId = b₂.profileId)
    (henv : b₁.lowerEnvelope = b₂.lowerEnvelope)
    (hcand : b₁.candidateFacts = b₂.candidateFacts)
    (hcost : b₁.costSurfaces = b₂.costSurfaces)
    (hid : StateId b₁ = StateId b₂) :
    evaluate b₁ profile regionId = evaluate b₂ profile regionId := by
  simp only [evaluate, regionExact, answerOfRegion, hp, henv, hcand, hcost, hid]

end Envelope

/-! ## A blank state body, and reachability of all six statuses

The witnesses below prove that the evaluator is **not** vacuous: every one of
the six statuses of SPEC §12.4 is actually produced by `evaluate` on a concrete
state body.  A classifier that could only ever answer `incompleteCoverage`
would be useless, and a classifier that could only ever answer
`attainedMinimum` would be dangerous. -/

namespace StateBody

/-- The empty state body over a given scope: no declarations, no objects, no
closure, no candidates, no certificates. -/
def blank (objectiveId profileId problemId : CanonicalObjectId) : StateBody where
  declarationBase := ⟨[]⟩
  accumulatedDeltaRoot := ⟨[]⟩
  semanticObjects := ⟨[]⟩
  shapeEdges := ⟨[]⟩
  semanticClosure := ⟨[], [], ⟨[]⟩⟩
  attentionIndex := ⟨[], ⟨[]⟩⟩
  dependencyGraph := ⟨[], ⟨[]⟩⟩
  candidateFacts := ⟨[]⟩
  costSurfaces := ⟨[]⟩
  searchPartitions := ⟨[], ⟨[]⟩⟩
  lowerEnvelope := ⟨[], ⟨[]⟩⟩
  certificates := ⟨[], ⟨[]⟩⟩
  objectiveId := objectiveId
  profileId := profileId
  problemId := problemId

@[simp] theorem blank_profileId (o p q : CanonicalObjectId) :
    (blank o p q).profileId = p := rfl

@[simp] theorem blank_declarationBase (o p q : CanonicalObjectId) :
    (blank o p q).declarationBase = ⟨[]⟩ := rfl

end StateBody

namespace Envelope

/-- A blank body carrying exactly one recorded region. -/
def withRegion (r : EnvelopeRegion) : StateBody :=
  { StateBody.blank nullId nullId nullId with
    lowerEnvelope := ⟨[r], ⟨[r.regionId]⟩⟩ }

/-- A blank body carrying one candidate with a recorded score, and one region
that records it as the attained minimum. -/
def attainedWitness : StateBody :=
  { StateBody.blank nullId nullId nullId with
    candidateFacts := ⟨[⟨nullId, ByteArray.empty⟩]⟩
    costSurfaces := ⟨[⟨nullId, 7, [7]⟩]⟩
    lowerEnvelope := ⟨[⟨nullId, EnvelopeStatus.attainedMinimum, some nullId, 7⟩], ⟨[nullId]⟩⟩ }

theorem evaluate_attainedWitness :
    evaluate attainedWitness nullId nullId = EnvelopeAnswer.attainedMinimum nullId 7 := by
  rfl

theorem evaluate_withRegion_status (s : EnvelopeStatus)
    (h : s ≠ EnvelopeStatus.attainedMinimum) :
    (evaluate (withRegion ⟨nullId, s, none, 0⟩) nullId nullId).status = s := by
  have hex : regionExact (withRegion ⟨nullId, s, none, 0⟩) ⟨nullId, s, none, 0⟩ = true := by
    cases s <;> first | rfl | exact absurd rfl h
  have hr : (withRegion ⟨nullId, s, none, 0⟩).lowerEnvelope.region? nullId =
      some ⟨nullId, s, none, 0⟩ := by
    simp [withRegion, LowerEnvelopeBody.region?]
  exact evaluate_status_of_exact rfl hr hex

/-- **Non-vacuity of the six-way distinction.**  Every status of SPEC §12.4 is
actually produced by the evaluator on a concrete state body. -/
theorem status_reachable (s : EnvelopeStatus) :
    ∃ (b : StateBody) (p r : CanonicalObjectId), (evaluate b p r).status = s := by
  cases s with
  | attainedMinimum =>
    exact ⟨attainedWitness, nullId, nullId, by rw [evaluate_attainedWitness]; rfl⟩
  | infeasibleRegion =>
    exact ⟨_, nullId, nullId, evaluate_withRegion_status _ (by decide)⟩
  | nonattainedInfimum =>
    exact ⟨_, nullId, nullId, evaluate_withRegion_status _ (by decide)⟩
  | incompleteCoverage =>
    exact ⟨_, nullId, nullId, evaluate_withRegion_status _ (by decide)⟩
  | unsupportedProfile =>
    exact ⟨_, nullId, nullId, evaluate_withRegion_status _ (by decide)⟩
  | invalidatedOrUnsealed =>
    exact ⟨_, nullId, nullId, evaluate_withRegion_status _ (by decide)⟩

end Envelope

end WasmGemmGnaf.Atlas
