import WasmGemmGnaf.Atlas.Certificate
set_option autoImplicit false

/-!
# Atlas: the seal (SPEC §12.1)

This file completes SPEC §12.1: the seven seal conditions
(`Atlas.VerifiesLeastClosure` … `Atlas.ResolvesEveryReferencedPreimage`), the
proof-carrying `Atlas.SealCertificate`, and `Atlas.SealedState` whose identity
is taken over `(core, certificate.body)` only.

Each seal condition is the *decidable* recomputation of that condition by the
deterministic checkers of `Atlas/Certificate.lean`; the propositions are
therefore checkable rather than postulated, and each one is unfolded into its
mathematical content by proved lemmas (closure is closed and supported, the
root partition cover is exact and duplicate free, the envelope regions are
exact, and so on).

The main results:

* `Atlas.SealCertificate.subsingleton` — for a given state and core there is at
  most one seal certificate.  This is `seal_certificate_body_unique` plus
  proof irrelevance of the seven verification fields.
* `Atlas.SealedState.sealId_eq_iff` — a seal identity determines the core and
  the certificate body exactly.
* `Atlas.SealedState.sealId_functional` — a state and a core determine the seal
  identity.
* `Atlas.sealId_typeTag_ne_*` — the seal identity is a *product* identity and is
  therefore never equal to the identity of any of its components (acyclicity).
-/

namespace WasmGemmGnaf.Atlas

open WasmGemmGnaf.Foundation

/-! ## The seven seal conditions (SPEC §12.1) -/

/-- **SPEC §12.1**, `VerifiesLeastClosure`: the core's closure root is the
state's, it lists exactly the closure facts, the fact set is closed under the
derivation edges, and every fact is supported by a derivation whose premises are
themselves facts. -/
def VerifiesLeastClosure (state : UnsealedState) (core : SealCore) : Prop :=
  closureLeastCheck state core = true

/-- **SPEC §12.1**, `VerifiesAttentionCoverage`. -/
def VerifiesAttentionCoverage (state : UnsealedState) (core : SealCore) : Prop :=
  attentionCompleteCheck state core = true

/-- **SPEC §12.1**, `VerifiesDependencyCoverage`. -/
def VerifiesDependencyCoverage (state : UnsealedState) (core : SealCore) : Prop :=
  dependenciesCompleteCheck state core = true

/-- **SPEC §12.1**, `VerifiesRootPartitionCover`. -/
def VerifiesRootPartitionCover (state : UnsealedState) (core : SealCore) : Prop :=
  universalCoverCompleteCheck state core = true

/-- **SPEC §12.1**, `VerifiesLowerEnvelope`. -/
def VerifiesLowerEnvelope (state : UnsealedState) (core : SealCore) : Prop :=
  envelopeExactCheck state core = true

/-- **SPEC §12.1**, `VerifiesCertificateStore`. -/
def VerifiesCertificateStore (state : UnsealedState) (core : SealCore) : Prop :=
  certificatesSoundCheck state core = true

/-- **SPEC §12.1**, `ResolvesEveryReferencedPreimage`. -/
def ResolvesEveryReferencedPreimage (state : UnsealedState) (core : SealCore) : Prop :=
  retentionCompleteCheck state core = true

instance (state : UnsealedState) (core : SealCore) :
    Decidable (VerifiesLeastClosure state core) := by
  unfold VerifiesLeastClosure; infer_instance

instance (state : UnsealedState) (core : SealCore) :
    Decidable (VerifiesAttentionCoverage state core) := by
  unfold VerifiesAttentionCoverage; infer_instance

instance (state : UnsealedState) (core : SealCore) :
    Decidable (VerifiesDependencyCoverage state core) := by
  unfold VerifiesDependencyCoverage; infer_instance

instance (state : UnsealedState) (core : SealCore) :
    Decidable (VerifiesRootPartitionCover state core) := by
  unfold VerifiesRootPartitionCover; infer_instance

instance (state : UnsealedState) (core : SealCore) :
    Decidable (VerifiesLowerEnvelope state core) := by
  unfold VerifiesLowerEnvelope; infer_instance

instance (state : UnsealedState) (core : SealCore) :
    Decidable (VerifiesCertificateStore state core) := by
  unfold VerifiesCertificateStore; infer_instance

instance (state : UnsealedState) (core : SealCore) :
    Decidable (ResolvesEveryReferencedPreimage state core) := by
  unfold ResolvesEveryReferencedPreimage; infer_instance

/-! ### The conditions are exactly the deterministic checker results -/

theorem verifiesLeastClosure_iff (state : UnsealedState) (core : SealCore) :
    VerifiesLeastClosure state core ↔
      sealCheckResult state core .closureLeast = true := Iff.rfl

theorem verifiesAttentionCoverage_iff (state : UnsealedState) (core : SealCore) :
    VerifiesAttentionCoverage state core ↔
      sealCheckResult state core .attentionComplete = true := Iff.rfl

theorem verifiesDependencyCoverage_iff (state : UnsealedState) (core : SealCore) :
    VerifiesDependencyCoverage state core ↔
      sealCheckResult state core .dependenciesComplete = true := Iff.rfl

theorem verifiesRootPartitionCover_iff (state : UnsealedState) (core : SealCore) :
    VerifiesRootPartitionCover state core ↔
      sealCheckResult state core .universalCoverComplete = true := Iff.rfl

theorem verifiesLowerEnvelope_iff (state : UnsealedState) (core : SealCore) :
    VerifiesLowerEnvelope state core ↔
      sealCheckResult state core .envelopeExact = true := Iff.rfl

theorem verifiesCertificateStore_iff (state : UnsealedState) (core : SealCore) :
    VerifiesCertificateStore state core ↔
      sealCheckResult state core .certificatesSound = true := Iff.rfl

theorem resolvesEveryReferencedPreimage_iff (state : UnsealedState) (core : SealCore) :
    ResolvesEveryReferencedPreimage state core ↔
      sealCheckResult state core .retentionComplete = true := Iff.rfl

/-! ### Mathematical content of the conditions -/

namespace VerifiesLeastClosure

variable {state : UnsealedState} {core : SealCore}

theorem rootEq (h : VerifiesLeastClosure state core) :
    core.closureRoot = state.body.semanticClosure.root := by
  simp only [VerifiesLeastClosure, closureLeastCheck, Bool.and_eq_true,
    decide_eq_true_eq] at h
  exact h.1.1.1

theorem root_lists_facts (h : VerifiesLeastClosure state core) :
    state.body.semanticClosure.root.factIds = state.body.semanticClosure.facts := by
  simp only [VerifiesLeastClosure, closureLeastCheck, Bool.and_eq_true,
    decide_eq_true_eq] at h
  exact h.1.1.2

/-- The closure is **closed**: a derivation all of whose premises are facts has
its conclusion among the facts. -/
theorem closed (h : VerifiesLeastClosure state core) :
    ∀ d ∈ state.body.semanticClosure.derivations,
      (∀ p ∈ d.premises, p ∈ state.body.semanticClosure.facts) →
        d.conclusion ∈ state.body.semanticClosure.facts := by
  simp only [VerifiesLeastClosure, closureLeastCheck, Bool.and_eq_true,
    decide_eq_true_eq] at h
  intro d hd hpre
  have h' := (List.all_eq_true.mp h.1.2) d hd
  rw [Bool.or_eq_true, Bool.not_eq_true'] at h'
  rcases h' with hfalse | hmem
  · exact absurd ((subsetId_iff _ _).mpr hpre) (by simp [hfalse])
  · exact (memId_iff _ _).mp hmem

/-- The closure is **least** in the supported sense: every fact is the
conclusion of a derivation whose premises are themselves facts, so no fact is
free-standing. -/
theorem supported (h : VerifiesLeastClosure state core) :
    ∀ f ∈ state.body.semanticClosure.facts,
      ∃ d ∈ state.body.semanticClosure.derivations,
        d.conclusion = f ∧
          ∀ p ∈ d.premises, p ∈ state.body.semanticClosure.facts := by
  simp only [VerifiesLeastClosure, closureLeastCheck, Bool.and_eq_true,
    decide_eq_true_eq] at h
  intro f hf
  have h' := (List.all_eq_true.mp h.2) f hf
  simp only [List.any_eq_true, Bool.and_eq_true, decide_eq_true_eq] at h'
  obtain ⟨d, hd, hconc, hsub⟩ := h'
  exact ⟨d, hd, hconc, (subsetId_iff _ _).mp hsub⟩

end VerifiesLeastClosure

namespace VerifiesAttentionCoverage

variable {state : UnsealedState} {core : SealCore}

theorem rootEq (h : VerifiesAttentionCoverage state core) :
    core.attentionRoot = state.body.attentionIndex.root := by
  simp only [VerifiesAttentionCoverage, attentionCompleteCheck, Bool.and_eq_true,
    decide_eq_true_eq] at h
  exact h.1.1.1

/-- Attention routes only known semantic objects. -/
theorem targets_known (h : VerifiesAttentionCoverage state core) :
    ∀ e ∈ state.body.attentionIndex.entries,
      ∀ t ∈ e.targets, t ∈ state.body.semanticObjects.keys := by
  simp only [VerifiesAttentionCoverage, attentionCompleteCheck, Bool.and_eq_true,
    decide_eq_true_eq] at h
  intro e he
  exact (subsetId_iff _ _).mp ((List.all_eq_true.mp h.1.2) e he)

/-- Attention has no coverage hole: every semantic object is routed by some
signature. -/
theorem covers_objects (h : VerifiesAttentionCoverage state core) :
    ∀ k ∈ state.body.semanticObjects.keys,
      ∃ e ∈ state.body.attentionIndex.entries, k ∈ e.targets := by
  simp only [VerifiesAttentionCoverage, attentionCompleteCheck, Bool.and_eq_true,
    decide_eq_true_eq] at h
  intro k hk
  have h' := (List.all_eq_true.mp h.2) k hk
  simp only [List.any_eq_true] at h'
  obtain ⟨e, he, hmem⟩ := h'
  exact ⟨e, he, (memId_iff _ _).mp hmem⟩

end VerifiesAttentionCoverage

namespace VerifiesDependencyCoverage

variable {state : UnsealedState} {core : SealCore}

theorem rootEq (h : VerifiesDependencyCoverage state core) :
    core.dependencyRoot = state.body.dependencyGraph.root := by
  simp only [VerifiesDependencyCoverage, dependenciesCompleteCheck,
    Bool.and_eq_true, decide_eq_true_eq] at h
  exact h.1.1.1

/-- Every dependency names a referenced object: the impact cone is closed. -/
theorem dependencies_referenced (h : VerifiesDependencyCoverage state core) :
    ∀ e ∈ state.body.dependencyGraph.edges,
      ∀ d ∈ e.dependsOn, d ∈ state.body.referencedObjects := by
  simp only [VerifiesDependencyCoverage, dependenciesCompleteCheck,
    Bool.and_eq_true, decide_eq_true_eq] at h
  intro e he
  exact (subsetId_iff _ _).mp ((List.all_eq_true.mp h.1.2) e he)

/-- Every semantic object has a dependency edge. -/
theorem every_object_has_edge (h : VerifiesDependencyCoverage state core) :
    ∀ k ∈ state.body.semanticObjects.keys,
      ∃ e ∈ state.body.dependencyGraph.edges, e.source = k := by
  simp only [VerifiesDependencyCoverage, dependenciesCompleteCheck,
    Bool.and_eq_true, decide_eq_true_eq] at h
  intro k hk
  have h' := (List.all_eq_true.mp h.2) k hk
  simp only [List.any_eq_true, decide_eq_true_eq] at h'
  exact h'

end VerifiesDependencyCoverage

namespace VerifiesRootPartitionCover

variable {state : UnsealedState} {core : SealCore}

theorem rootEq (h : VerifiesRootPartitionCover state core) :
    core.partitionCoverRoot = state.body.searchPartitions.coverRoot := by
  simp only [VerifiesRootPartitionCover, universalCoverCompleteCheck,
    Bool.and_eq_true, decide_eq_true_eq] at h
  exact h.1.1.1.1

/-- The partitions are genuinely disjoint: no candidate is covered twice. -/
theorem covered_nodup (h : VerifiesRootPartitionCover state core) :
    state.body.searchPartitions.covered.Nodup := by
  simp only [VerifiesRootPartitionCover, universalCoverCompleteCheck,
    Bool.and_eq_true, decide_eq_true_eq] at h
  exact nodup_of_distinctIds h.1.1.2

/-- Universality: every candidate lies in some partition. -/
theorem covers_candidates (h : VerifiesRootPartitionCover state core) :
    ∀ c ∈ state.body.candidateFacts.keys,
      c ∈ state.body.searchPartitions.covered := by
  simp only [VerifiesRootPartitionCover, universalCoverCompleteCheck,
    Bool.and_eq_true, decide_eq_true_eq] at h
  exact (subsetId_iff _ _).mp h.1.2

/-- Exactness: nothing outside the candidate set is covered. -/
theorem covered_are_candidates (h : VerifiesRootPartitionCover state core) :
    ∀ c ∈ state.body.searchPartitions.covered,
      c ∈ state.body.candidateFacts.keys := by
  simp only [VerifiesRootPartitionCover, universalCoverCompleteCheck,
    Bool.and_eq_true, decide_eq_true_eq] at h
  exact (subsetId_iff _ _).mp h.2

end VerifiesRootPartitionCover

namespace VerifiesLowerEnvelope

variable {state : UnsealedState} {core : SealCore}

theorem rootEq (h : VerifiesLowerEnvelope state core) :
    core.envelopeRoot = state.body.lowerEnvelope.root := by
  simp only [VerifiesLowerEnvelope, envelopeExactCheck, Bool.and_eq_true,
    decide_eq_true_eq] at h
  exact h.1.1

theorem regions_exact (h : VerifiesLowerEnvelope state core) :
    ∀ r ∈ state.body.lowerEnvelope.regions, envelopeRegionExact state r = true := by
  simp only [VerifiesLowerEnvelope, envelopeExactCheck, Bool.and_eq_true,
    decide_eq_true_eq] at h
  exact List.all_eq_true.mp h.2

/-- An attained minimum names a real candidate whose recorded score is exactly
the region bound (SPEC §12.4: "attained minimum"). -/
theorem attained_score (h : VerifiesLowerEnvelope state core)
    {r : EnvelopeRegion} (hr : r ∈ state.body.lowerEnvelope.regions)
    {c : CanonicalObjectId} (hstatus : r.status = EnvelopeStatus.attainedMinimum)
    (hattained : r.attained = some c) :
    c ∈ state.body.candidateFacts.keys ∧
      state.body.costSurfaces.score? c = some r.bound := by
  have hex := regions_exact h r hr
  rw [envelopeRegionExact, hstatus, hattained] at hex
  simp only [Bool.and_eq_true, decide_eq_true_eq] at hex
  exact ⟨(memId_iff _ _).mp hex.1, hex.2⟩

/-- A region that is not an attained minimum names no candidate: infeasible,
nonattained, incompletely covered, unsupported and invalidated regions never
claim an attained artifact (SPEC §12.4). -/
theorem not_attained_none (h : VerifiesLowerEnvelope state core)
    {r : EnvelopeRegion} (hr : r ∈ state.body.lowerEnvelope.regions)
    (hstatus : r.status ≠ EnvelopeStatus.attainedMinimum) :
    r.attained = none := by
  have hex := regions_exact h r hr
  cases hattained : r.attained with
  | none => rfl
  | some c =>
    rw [envelopeRegionExact, hattained] at hex
    cases hs : r.status with
    | attainedMinimum => exact absurd hs hstatus
    | infeasibleRegion => rw [hs] at hex; simp at hex
    | nonattainedInfimum => rw [hs] at hex; simp at hex
    | incompleteCoverage => rw [hs] at hex; simp at hex
    | unsupportedProfile => rw [hs] at hex; simp at hex
    | invalidatedOrUnsealed => rw [hs] at hex; simp at hex

end VerifiesLowerEnvelope

namespace VerifiesCertificateStore

variable {state : UnsealedState} {core : SealCore}

theorem rootEq (h : VerifiesCertificateStore state core) :
    core.certificateRoot = state.body.certificates.root := by
  simp only [VerifiesCertificateStore, certificatesSoundCheck, Bool.and_eq_true,
    decide_eq_true_eq] at h
  exact h.1.1

/-- Every stored certificate is about a referenced object and depends only on
referenced objects (SPEC §12.3: exact dependency lists). -/
theorem entries_referenced (h : VerifiesCertificateStore state core) :
    ∀ c ∈ state.body.certificates.entries,
      c.subject ∈ state.body.referencedObjects ∧
        ∀ d ∈ c.dependencies, d ∈ state.body.referencedObjects := by
  simp only [VerifiesCertificateStore, certificatesSoundCheck, Bool.and_eq_true,
    decide_eq_true_eq] at h
  intro c hc
  have h' := (List.all_eq_true.mp h.2) c hc
  rw [Bool.and_eq_true] at h'
  exact ⟨(memId_iff _ _).mp h'.1, (subsetId_iff _ _).mp h'.2⟩

end VerifiesCertificateStore

namespace ResolvesEveryReferencedPreimage

variable {state : UnsealedState} {core : SealCore}

theorem rootEq (h : ResolvesEveryReferencedPreimage state core) :
    core.retentionRoot = state.retentionRoot :=
  (retentionCompleteCheck_iff state core).mp h

/-- Every referenced object resolves to a retained preimage.  The hypothesis is
recorded for API symmetry; the conclusion already follows from the state's own
complete object graph. -/
theorem resolves (_h : ResolvesEveryReferencedPreimage state core) :
    ∀ id ∈ state.body.referencedObjects,
      state.retainedObjects.graph.resolves id = true :=
  fun id hid => state.retainedObjects.complete id hid

end ResolvesEveryReferencedPreimage

/-- Retention is not an extra assumption: it holds exactly when the core
records the state's own retention root, because an `UnsealedState` already
carries a complete object graph. -/
theorem resolvesEveryReferencedPreimage_of_rootEq (state : UnsealedState)
    (core : SealCore) (h : core.retentionRoot = state.retentionRoot) :
    ResolvesEveryReferencedPreimage state core :=
  (retentionCompleteCheck_iff state core).mpr h

/-! ## `Atlas.SealCertificate` (SPEC §12.1) -/

/-- **SPEC §12.1**, `Atlas.SealCertificate`.  The body is data; every other
field is a proposition that has been checked. -/
structure SealCertificate (state : UnsealedState) (core : SealCore) where
  body : SealCertificateBody
  bodyValid : VerifiesSealCertificateBody state core body
  coreBindsState : core.stateId = state.bodyId ∧
    core.profileId = state.body.profileId ∧
    core.problemId = state.body.problemId ∧
    core.objectiveId = state.body.objectiveId
  closureLeast : VerifiesLeastClosure state core
  attentionComplete : VerifiesAttentionCoverage state core
  dependenciesComplete : VerifiesDependencyCoverage state core
  universalCoverComplete : VerifiesRootPartitionCover state core
  envelopeExact : VerifiesLowerEnvelope state core
  certificatesSound : VerifiesCertificateStore state core
  retentionComplete : ResolvesEveryReferencedPreimage state core

namespace SealCertificate

variable {state : UnsealedState} {core : SealCore}

/-- Certificates with equal bodies are equal: all the remaining fields are
propositions. -/
theorem eq_of_body_eq (c d : SealCertificate state core) (h : c.body = d.body) :
    c = d := by
  cases c; cases d
  cases h
  rfl

/-- **The seal certificate of a state and core is unique.**  This is
`seal_certificate_body_unique` (SPEC §12.1) together with proof irrelevance of
the checked propositions. -/
theorem eq_all (c d : SealCertificate state core) : c = d :=
  eq_of_body_eq c d
    (seal_certificate_body_unique state core c.body d.body c.bodyValid d.bodyValid)

instance : Subsingleton (SealCertificate state core) := ⟨eq_all⟩

/-- The certified body is the canonical body of the state and core. -/
theorem body_eq_canonical (c : SealCertificate state core) :
    c.body = canonicalSealCertificateBody state core :=
  eq_canonicalSealCertificateBody c.bodyValid

/-- A certified core records the state's identity. -/
theorem core_stateId (c : SealCertificate state core) :
    core.stateId = StateId state.body := by
  rw [c.coreBindsState.1, state.bodyIdEq]

/-- Every one of the seven deterministic seal checks succeeds on a certified
state and core. -/
theorem sealCheckResult_true (c : SealCertificate state core) :
    ∀ tag : SealCheckTag, sealCheckResult state core tag = true := by
  intro tag
  cases tag with
  | closureLeast => exact c.closureLeast
  | attentionComplete => exact c.attentionComplete
  | dependenciesComplete => exact c.dependenciesComplete
  | universalCoverComplete => exact c.universalCoverComplete
  | envelopeExact => exact c.envelopeExact
  | certificatesSound => exact c.certificatesSound
  | retentionComplete => exact c.retentionComplete

/-- Hence every stored checker-result record of a certified seal records a
successful outcome. -/
theorem checkResultBody_outcome_true (c : SealCertificate state core)
    (tag : SealCheckTag) :
    (canonicalSealCheckResultBody state core tag).outcome = true :=
  sealCheckResult_true c tag

end SealCertificate

/-! ## `Atlas.SealedState` (SPEC §12.1) -/

/-- **SPEC §12.1**, `Atlas.SealIdentity`: the seal identifies the pair
`(core, certificate.body)` and nothing else. -/
abbrev SealIdentity := ObjectId (SealCore × SealCertificateBody)

/-- The frozen canonical schema of a seal: the structural product of the core
schema and the certificate body schema (`Foundation/SchemaRegistry.lean`). -/
def SealedState.identitySchema : CanonicalSchema (SealCore × SealCertificateBody) :=
  CanonicalSchema.product 1 CanonicalDomainTag.atlasState
    SealCore.identitySchema SealCertificateBody.identitySchema

/-- The seal identity of a core and a certificate body. -/
def SealId (core : SealCore) (body : SealCertificateBody) : SealIdentity :=
  Identity SealedState.identitySchema (core, body)

/-- **SPEC §12.1**, `Atlas.SealedState`. -/
structure SealedState where
  state : UnsealedState
  core : SealCore
  certificate : SealCertificate state core
  sealId : SealIdentity
  sealIdEq : sealId = Identity SealedState.identitySchema (core, certificate.body)

namespace SealedState

/-- A seal identity determines the core and the certificate body exactly: this
is `Identity_eq_iff` for the product schema. -/
theorem sealId_eq_iff {c₁ c₂ : SealCore} {b₁ b₂ : SealCertificateBody} :
    SealId c₁ b₁ = SealId c₂ b₂ ↔ (c₁, b₁) = (c₂, b₂) :=
  Identity_eq_iff SealedState.identitySchema

theorem sealId_injective : Function.Injective (fun p : SealCore × SealCertificateBody =>
    Identity SealedState.identitySchema p) :=
  Identity_injective SealedState.identitySchema

/-- Equal seal identities force equal cores. -/
theorem core_eq_of_sealId_eq {c₁ c₂ : SealCore} {b₁ b₂ : SealCertificateBody}
    (h : SealId c₁ b₁ = SealId c₂ b₂) : c₁ = c₂ :=
  congrArg Prod.fst (sealId_eq_iff.mp h)

/-- Equal seal identities force equal certificate bodies. -/
theorem body_eq_of_sealId_eq {c₁ c₂ : SealCore} {b₁ b₂ : SealCertificateBody}
    (h : SealId c₁ b₁ = SealId c₂ b₂) : b₁ = b₂ :=
  congrArg Prod.snd (sealId_eq_iff.mp h)

/-- The seal identity of a sealed state is the identity of its own core and
certificate body. -/
theorem sealId_eq (s : SealedState) : s.sealId = SealId s.core s.certificate.body :=
  s.sealIdEq

/-- **The seal identity is a function of the state and the core.**  Nothing in
the certificate can influence it, because the certificate body is itself
determined by the state and the core. -/
theorem sealId_functional (s t : SealedState) (hstate : s.state = t.state)
    (hcore : s.core = t.core) : s.sealId = t.sealId := by
  obtain ⟨st₁, c₁, cert₁, id₁, hid₁⟩ := s
  obtain ⟨st₂, c₂, cert₂, id₂, hid₂⟩ := t
  cases hstate
  cases hcore
  have hb : cert₁.body = cert₂.body :=
    seal_certificate_body_unique st₁ c₁ cert₁.body cert₂.body cert₁.bodyValid
      cert₂.bodyValid
  show id₁ = id₂
  rw [hid₁, hid₂, hb]

/-- Two sealed states over the same state and core are equal up to their seal
identity — in fact their certificates are literally the same. -/
theorem certificate_heq (s t : SealedState) (hstate : s.state = t.state)
    (hcore : s.core = t.core) : s.certificate.body = t.certificate.body := by
  obtain ⟨st₁, c₁, cert₁, _, _⟩ := s
  obtain ⟨st₂, c₂, cert₂, _, _⟩ := t
  cases hstate
  cases hcore
  exact seal_certificate_body_unique st₁ c₁ cert₁.body cert₂.body cert₁.bodyValid
    cert₂.bodyValid

/-- A sealed state's core binds its state's identity. -/
theorem core_stateId (s : SealedState) : s.core.stateId = StateId s.state.body :=
  s.certificate.core_stateId

/-- A sealed state satisfies all seven deterministic checks. -/
theorem sealCheckResult_true (s : SealedState) :
    ∀ tag : SealCheckTag, sealCheckResult s.state s.core tag = true :=
  s.certificate.sealCheckResult_true

end SealedState

/-! ## Acyclicity of the seal identity (SPEC §12.1)

The seal identity is a *product* identity over `(SealCore, SealCertificateBody)`
while every component identity is a *leaf* identity, so the seal identity is
never one of the identities it seals. -/

theorem sealId_typeTag :
    ∀ (core : SealCore) (body : SealCertificateBody),
      (SealId core body).typeTag =
        TypeTag.product SealCore.identitySchema.typeTag
          SealCertificateBody.identitySchema.typeTag :=
  fun _ _ => rfl

theorem sealId_typeTag_ne_stateBody (core : SealCore) (body : SealCertificateBody)
    (b : StateBody) : (SealId core body).typeTag ≠ (StateId b).typeTag := by
  intro h
  exact TypeTag.leaf_ne_product _ _ _ h.symm

theorem sealId_typeTag_ne_sealCore (core₁ : SealCore) (body : SealCertificateBody)
    (core₂ : SealCore) : (SealId core₁ body).typeTag ≠ (SealCoreId core₂).typeTag := by
  intro h
  exact TypeTag.leaf_ne_product _ _ _ h.symm

theorem sealId_typeTag_ne_sealCertificateBody (core : SealCore)
    (body₁ body₂ : SealCertificateBody) :
    (SealId core body₁).typeTag ≠ (SealCertificateBodyId body₂).typeTag := by
  intro h
  exact TypeTag.leaf_ne_product _ _ _ h.symm

/-- **Acyclicity**: the seal identity is never equal to the identity of the
certificate body it seals, nor to the identity of its core, nor to the identity
of the state body. -/
theorem sealId_ne_component_identities (core : SealCore) (body : SealCertificateBody)
    (b : StateBody) :
    CanonicalObjectId.ofTyped (SealId core body) ≠ SealCertificateBodyId body ∧
    CanonicalObjectId.ofTyped (SealId core body) ≠
      CanonicalObjectId.ofTyped (SealCoreId core) ∧
    CanonicalObjectId.ofTyped (SealId core body) ≠ StateId b := by
  refine ⟨?_, ?_, ?_⟩
  · intro h
    exact sealId_typeTag_ne_sealCertificateBody core body body
      (congrArg CanonicalObjectId.typeTag h)
  · intro h
    exact sealId_typeTag_ne_sealCore core body core
      (congrArg CanonicalObjectId.typeTag h)
  · intro h
    exact sealId_typeTag_ne_stateBody core body b
      (congrArg CanonicalObjectId.typeTag h)

end WasmGemmGnaf.Atlas
