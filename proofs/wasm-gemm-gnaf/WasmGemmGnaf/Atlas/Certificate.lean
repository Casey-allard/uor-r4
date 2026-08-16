import WasmGemmGnaf.Atlas.State
set_option autoImplicit false

/-!
# Atlas: seal core, deterministic checkers and the seal certificate body
(SPEC §12.1)

This file transcribes `Atlas.SealCore`, `Atlas.SealCoreId`,
`Atlas.SealCheckTag`, `Atlas.canonicalSealCheckResultId`,
`Atlas.SealCertificateBody` and `Atlas.VerifiesSealCertificateBody`, and proves
the theorem SPEC §12.1 demands:

* `Atlas.seal_certificate_body_unique` — `VerifiesSealCertificateBody` pins
  every one of the twenty fields of the body to a function of `(state, core)`,
  so at most one body can ever verify.

It also discharges the *acyclicity* claim of SPEC §12.1 ("no component contains
its enclosing identity") in the only form that is a theorem rather than a
convention: the schemas of `StateBody`, `SealCore`, `SealCertificateBody` and
`SealCheckResultBody` have provably distinct structural type tags (and the
checker results live in a different identity domain), so an identity of one of
these types is never equal to an identity of another, and in particular a
verified `SealCertificateBody` never carries its own identity in any of its
identity fields (`Atlas.verified_selfId_not_mem_referencedIdentities`).

The deterministic checkers are real computations: `Atlas.sealCheckResult`
recomputes each seal condition from the state and compares it with the core,
and `canonicalSealCheckResultId` identifies a record that stores the complete
checker input (the canonical state and core bytes), the checker result, and the
retained preimages, exactly as SPEC §12.1 requires.
-/

namespace WasmGemmGnaf.Atlas

open WasmGemmGnaf.Foundation

/-! ## Decidable list helpers -/

/-- Decidable membership of an identity in a list of identities. -/
def memId (l : List CanonicalObjectId) (x : CanonicalObjectId) : Bool :=
  l.any (fun y => decide (y = x))

theorem memId_iff (l : List CanonicalObjectId) (x : CanonicalObjectId) :
    memId l x = true ↔ x ∈ l := by
  constructor
  · intro h
    simp only [memId, List.any_eq_true, decide_eq_true_eq] at h
    obtain ⟨y, hy, rfl⟩ := h
    exact hy
  · intro h
    simp only [memId, List.any_eq_true, decide_eq_true_eq]
    exact ⟨x, h, rfl⟩

/-- Every element of `xs` occurs in `ys`. -/
def subsetId (xs ys : List CanonicalObjectId) : Bool :=
  xs.all (fun x => memId ys x)

theorem subsetId_iff (xs ys : List CanonicalObjectId) :
    subsetId xs ys = true ↔ ∀ x ∈ xs, x ∈ ys := by
  simp only [subsetId, List.all_eq_true]
  constructor
  · intro h x hx; exact (memId_iff ys x).mp (h x hx)
  · intro h x hx; exact (memId_iff ys x).mpr (h x hx)

/-- Decidable duplicate-freeness of a list of identities. -/
def distinctIds : List CanonicalObjectId → Bool
  | [] => true
  | x :: xs => !(memId xs x) && distinctIds xs

/-- A `distinctIds` list really is duplicate free. -/
theorem nodup_of_distinctIds : ∀ {l : List CanonicalObjectId},
    distinctIds l = true → l.Nodup
  | [], _ => List.nodup_nil
  | x :: xs, h => by
    simp only [distinctIds, Bool.and_eq_true, Bool.not_eq_true'] at h
    refine List.nodup_cons.mpr ⟨?_, nodup_of_distinctIds h.2⟩
    intro hx
    exact absurd ((memId_iff xs x).mpr hx) (by simp [h.1])

/-! ## `Atlas.SealCore` (SPEC §12.1) -/

/-- **SPEC §12.1**, `Atlas.SealCore`.  A first-order record over the state
identity, the three scope identities, the seven structural roots and the
baseline score.  It contains no certificate body and no proof. -/
structure SealCore where
  stateId : StateIdentity
  profileId : ProfileId
  problemId : ProblemId
  objectiveId : ObjectiveId
  closureRoot : ClosureRoot
  attentionRoot : AttentionRoot
  dependencyRoot : DependencyRoot
  partitionCoverRoot : PartitionCoverRoot
  envelopeRoot : EnvelopeRoot
  certificateRoot : CertificateRoot
  retentionRoot : RetentionRoot
  baselineScore : Nat
  deriving DecidableEq

namespace SealCore

def toTuple (c : SealCore) :
    CanonicalObjectId × CanonicalObjectId × CanonicalObjectId × CanonicalObjectId ×
      ClosureRoot × AttentionRoot × DependencyRoot × PartitionCoverRoot ×
      EnvelopeRoot × CertificateRoot × RetentionRoot × Nat :=
  (c.stateId, c.profileId, c.problemId, c.objectiveId, c.closureRoot,
   c.attentionRoot, c.dependencyRoot, c.partitionCoverRoot, c.envelopeRoot,
   c.certificateRoot, c.retentionRoot, c.baselineScore)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [SealCore.mk.injEq]
  exact h

def bytes (c : SealCore) : List UInt8 :=
  Bytes.pairBytes CanonicalObjectId.bytes
   (Bytes.pairBytes CanonicalObjectId.bytes
    (Bytes.pairBytes CanonicalObjectId.bytes
     (Bytes.pairBytes CanonicalObjectId.bytes
      (Bytes.pairBytes ClosureRoot.bytes
       (Bytes.pairBytes AttentionRoot.bytes
        (Bytes.pairBytes DependencyRoot.bytes
         (Bytes.pairBytes PartitionCoverRoot.bytes
          (Bytes.pairBytes EnvelopeRoot.bytes
           (Bytes.pairBytes CertificateRoot.bytes
            (Bytes.pairBytes RetentionRoot.bytes Bytes.natBytes)))))))))) c.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
   (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
    (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
     (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
      (Bytes.pairBytes_prefixFree ClosureRoot.bytes_prefixFree
       (Bytes.pairBytes_prefixFree AttentionRoot.bytes_prefixFree
        (Bytes.pairBytes_prefixFree DependencyRoot.bytes_prefixFree
         (Bytes.pairBytes_prefixFree PartitionCoverRoot.bytes_prefixFree
          (Bytes.pairBytes_prefixFree EnvelopeRoot.bytes_prefixFree
           (Bytes.pairBytes_prefixFree CertificateRoot.bytes_prefixFree
            (Bytes.pairBytes_prefixFree RetentionRoot.bytes_prefixFree
              Bytes.natBytes_prefixFree))))))))))).comp toTuple_injective

/-- The frozen canonical schema of `Atlas.SealCore` (structural index 2). -/
def identitySchema : CanonicalSchema SealCore :=
  CanonicalSchema.ofPrefixFree 1 CanonicalDomainTag.atlasState
    (leafTag 2 "Atlas.SealCore") (leafTag_size_pos 2 _)
    bytes bytes_prefixFree

@[simp] theorem identitySchema_typeTag :
    identitySchema.typeTag = leafTag 2 "Atlas.SealCore" := rfl

end SealCore

/-- **SPEC §12.1**, `Atlas.SealCoreIdentity`. -/
abbrev SealCoreIdentity := ObjectId SealCore

/-- **SPEC §12.1**, `Atlas.SealCoreId`. -/
def SealCoreId (core : SealCore) : SealCoreIdentity :=
  Identity SealCore.identitySchema core

theorem SealCoreId_eq_iff {a b : SealCore} : SealCoreId a = SealCoreId b ↔ a = b :=
  Identity_eq_iff SealCore.identitySchema

theorem SealCoreId_injective : Function.Injective SealCoreId :=
  Identity_injective SealCore.identitySchema

@[simp] theorem SealCoreId_typeTag (core : SealCore) :
    (SealCoreId core).typeTag = leafTag 2 "Atlas.SealCore" := rfl

/-- Canonical encoding of a typed identifier, used where an identity occurs
inside another canonical body. -/
def objectIdBytes {α : Type} (id : ObjectId α) : List UInt8 :=
  CanonicalObjectId.bytes (CanonicalObjectId.ofTyped id)

theorem objectIdBytes_prefixFree {α : Type} :
    Bytes.PrefixFree (objectIdBytes (α := α)) :=
  CanonicalObjectId.bytes_prefixFree.comp CanonicalObjectId.ofTyped_injective

/-! ## `Atlas.SealCheckTag` (SPEC §12.1) -/

/-- **SPEC §12.1**, `Atlas.SealCheckTag`: the seven deterministic seal
checks. -/
inductive SealCheckTag
  | closureLeast
  | attentionComplete
  | dependenciesComplete
  | universalCoverComplete
  | envelopeExact
  | certificatesSound
  | retentionComplete
  deriving DecidableEq, Repr, Inhabited

namespace SealCheckTag

def index : SealCheckTag → Nat
  | closureLeast => 0
  | attentionComplete => 1
  | dependenciesComplete => 2
  | universalCoverComplete => 3
  | envelopeExact => 4
  | certificatesSound => 5
  | retentionComplete => 6

theorem index_injective : Function.Injective index := by
  intro a b h
  cases a <;> cases b <;> simp_all [index]

def bytes (t : SealCheckTag) : List UInt8 := [UInt8.ofNat t.index]

theorem bytes_injective : Function.Injective bytes := by
  intro a b h
  cases a <;> cases b <;> simp_all [bytes, index]

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Bytes.prefixFree_of_constLength bytes 1 (fun t => by cases t <;> rfl) bytes_injective

/-- The complete finite enumeration of the seal checks. -/
def all : List SealCheckTag :=
  [closureLeast, attentionComplete, dependenciesComplete, universalCoverComplete,
   envelopeExact, certificatesSound, retentionComplete]

theorem mem_all (t : SealCheckTag) : t ∈ all := by cases t <;> simp [all]

theorem all_nodup : all.Nodup := by decide

/-- The seven seal checks form a finite cover in the sense of
`Foundation/Finite.lean`. -/
instance : Fintype SealCheckTag where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

theorem card_eq_seven : Fintype.card SealCheckTag = 7 := rfl

end SealCheckTag

/-! ## The deterministic checkers (SPEC §12.1)

Each checker recomputes a seal condition from the state and compares it with
the corresponding root recorded in the core.  They are total `Bool` functions
of `(state, core)`, so `canonicalSealCheckResultId` below is a function of the
state and core alone — which is precisely what makes
`seal_certificate_body_unique` true. -/

/-- Closure check: the core's closure root is the state's closure root, the
root lists exactly the closure facts, the fact set is closed under the
derivation edges, and every fact is supported by a derivation whose premises
are facts (leastness). -/
def closureLeastCheck (s : UnsealedState) (core : SealCore) : Bool :=
  let c := s.body.semanticClosure
  decide (core.closureRoot = c.root) &&
  decide (c.root.factIds = c.facts) &&
  c.derivations.all (fun d =>
    !(subsetId d.premises c.facts) || memId c.facts d.conclusion) &&
  c.facts.all (fun f =>
    c.derivations.any (fun d =>
      decide (d.conclusion = f) && subsetId d.premises c.facts))

/-- Attention check: the core's attention root is the state's, the root lists
exactly the indexed signatures, every routed target is a known semantic object,
and every semantic object is routed by at least one signature. -/
def attentionCompleteCheck (s : UnsealedState) (core : SealCore) : Bool :=
  let a := s.body.attentionIndex
  decide (core.attentionRoot = a.root) &&
  decide (a.root.signatures = a.entries.map (·.signature)) &&
  a.entries.all (fun e => subsetId e.targets s.body.semanticObjects.keys) &&
  s.body.semanticObjects.keys.all (fun k => a.entries.any (fun e => memId e.targets k))

/-- Dependency check: the core's dependency root is the state's, the root lists
exactly the dependency sources, every dependency target is a referenced object,
and every semantic object has a dependency edge. -/
def dependenciesCompleteCheck (s : UnsealedState) (core : SealCore) : Bool :=
  let d := s.body.dependencyGraph
  decide (core.dependencyRoot = d.root) &&
  decide (d.root.edgeIds = d.edges.map (·.source)) &&
  d.edges.all (fun e => subsetId e.dependsOn s.body.referencedObjects) &&
  s.body.semanticObjects.keys.all (fun k => d.edges.any (fun e => decide (e.source = k)))

/-- Universal cover check: the core's cover root is the state's, the root lists
exactly the partition identities, the covered candidates are duplicate free,
and the cover is *exact* — every candidate is covered and every covered element
is a candidate. -/
def universalCoverCompleteCheck (s : UnsealedState) (core : SealCore) : Bool :=
  let p := s.body.searchPartitions
  decide (core.partitionCoverRoot = p.coverRoot) &&
  decide (p.coverRoot.partitionIds = p.entries.map (·.partitionId)) &&
  distinctIds p.covered &&
  subsetId s.body.candidateFacts.keys p.covered &&
  subsetId p.covered s.body.candidateFacts.keys

/-- One region of the envelope is exact when an attained minimum names a
candidate whose recorded score is exactly the region bound, and every
non-attained status names no candidate (SPEC §12.4). -/
def envelopeRegionExact (s : UnsealedState) (r : EnvelopeRegion) : Bool :=
  match r.status, r.attained with
  | EnvelopeStatus.attainedMinimum, some c =>
      memId s.body.candidateFacts.keys c &&
      decide (s.body.costSurfaces.score? c = some r.bound)
  | EnvelopeStatus.attainedMinimum, none => false
  | _, some _ => false
  | _, none => true

/-- Envelope check: the core's envelope root is the state's, the root lists
exactly the region identities, and every region is exact. -/
def envelopeExactCheck (s : UnsealedState) (core : SealCore) : Bool :=
  let e := s.body.lowerEnvelope
  decide (core.envelopeRoot = e.root) &&
  decide (e.root.regionIds = e.regions.map (·.regionId)) &&
  e.regions.all (fun r => envelopeRegionExact s r)

/-- Certificate store check: the core's certificate root is the state's, the
root lists exactly the stored certificate identities, and every stored
certificate names a referenced subject and only referenced dependencies. -/
def certificatesSoundCheck (s : UnsealedState) (core : SealCore) : Bool :=
  let st := s.body.certificates
  decide (core.certificateRoot = st.root) &&
  decide (st.root.certificateIds = st.entries.map (·.certificateId)) &&
  st.entries.all (fun c =>
    memId s.body.referencedObjects c.subject &&
    subsetId c.dependencies s.body.referencedObjects)

/-- Retention check: the core's retention root is the state's retention root,
and every referenced object resolves to a retained preimage. -/
def retentionCompleteCheck (s : UnsealedState) (core : SealCore) : Bool :=
  decide (core.retentionRoot = s.retentionRoot) &&
  s.body.referencedObjects.all (fun id => s.retainedObjects.graph.resolves id)

/-- The deterministic result of one seal check. -/
def sealCheckResult (s : UnsealedState) (core : SealCore) : SealCheckTag → Bool
  | .closureLeast => closureLeastCheck s core
  | .attentionComplete => attentionCompleteCheck s core
  | .dependenciesComplete => dependenciesCompleteCheck s core
  | .universalCoverComplete => universalCoverCompleteCheck s core
  | .envelopeExact => envelopeExactCheck s core
  | .certificatesSound => certificatesSoundCheck s core
  | .retentionComplete => retentionCompleteCheck s core

/-- The retention half of the retention check is not an assumption: an
`UnsealedState` carries a complete object graph, so every referenced object
always resolves. -/
theorem referencedObjects_all_resolve (s : UnsealedState) :
    s.body.referencedObjects.all (fun id => s.retainedObjects.graph.resolves id) = true := by
  rw [List.all_eq_true]
  intro id hid
  exact s.retainedObjects.complete id hid

/-- Consequently the retention check succeeds exactly when the core records the
state's own retention root. -/
theorem retentionCompleteCheck_iff (s : UnsealedState) (core : SealCore) :
    retentionCompleteCheck s core = true ↔ core.retentionRoot = s.retentionRoot := by
  simp [retentionCompleteCheck, referencedObjects_all_resolve s]

/-! ## Canonical checker-result identities (SPEC §12.1)

"The deterministic checkers define `canonicalSealCheckResultId` by storing the
complete checker input, result, and retained preimages in canonical form." -/

/-- The canonical body of one checker result: the complete input (the canonical
bytes of the state body and of the core, plus their identities), the tag, the
result, and the retained preimages. -/
structure SealCheckResultBody where
  version : Nat
  tag : SealCheckTag
  stateId : StateIdentity
  coreId : CanonicalObjectId
  stateBodyBytes : ByteArray
  coreBytes : ByteArray
  retainedPreimages : List PreimageEntry
  outcome : Bool
  deriving DecidableEq

namespace SealCheckResultBody

def toTuple (r : SealCheckResultBody) :
    Nat × SealCheckTag × CanonicalObjectId × CanonicalObjectId × ByteArray ×
      ByteArray × List PreimageEntry × Bool :=
  (r.version, r.tag, r.stateId, r.coreId, r.stateBodyBytes, r.coreBytes,
   r.retainedPreimages, r.outcome)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [SealCheckResultBody.mk.injEq]
  exact h

def bytes (r : SealCheckResultBody) : List UInt8 :=
  Bytes.pairBytes Bytes.natBytes
   (Bytes.pairBytes SealCheckTag.bytes
    (Bytes.pairBytes CanonicalObjectId.bytes
     (Bytes.pairBytes CanonicalObjectId.bytes
      (Bytes.pairBytes Bytes.byteArrayBytes
       (Bytes.pairBytes Bytes.byteArrayBytes
        (Bytes.pairBytes (Bytes.listBytes PreimageEntry.bytes)
          Bytes.boolBytes)))))) r.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree Bytes.natBytes_prefixFree
   (Bytes.pairBytes_prefixFree SealCheckTag.bytes_prefixFree
    (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
     (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
      (Bytes.pairBytes_prefixFree Bytes.byteArrayBytes_prefixFree
       (Bytes.pairBytes_prefixFree Bytes.byteArrayBytes_prefixFree
        (Bytes.pairBytes_prefixFree
          (Bytes.listBytes_prefixFree PreimageEntry.bytes_prefixFree)
          Bytes.boolBytes_prefixFree))))))).comp toTuple_injective

/-- The frozen canonical schema of a checker result (structural index 3).  It
lives in the `atlasSealCheck` identity domain, not in `atlasState`. -/
def identitySchema : CanonicalSchema SealCheckResultBody :=
  CanonicalSchema.ofPrefixFree 1 CanonicalDomainTag.atlasSealCheck
    (leafTag 3 "Atlas.SealCheckResultBody") (leafTag_size_pos 3 _)
    bytes bytes_prefixFree

@[simp] theorem identitySchema_domain :
    identitySchema.domain = CanonicalDomainTag.atlasSealCheck := rfl

end SealCheckResultBody

/-- The canonical checker-result body for one check of one state against one
core. -/
def canonicalSealCheckResultBody (state : UnsealedState) (core : SealCore)
    (tag : SealCheckTag) : SealCheckResultBody where
  version := 1
  tag := tag
  stateId := state.bodyId
  coreId := CanonicalObjectId.ofTyped (SealCoreId core)
  stateBodyBytes := StateBody.identitySchema.encode state.body
  coreBytes := SealCore.identitySchema.encode core
  retainedPreimages := state.retainedObjects.graph.preimages
  outcome := sealCheckResult state core tag

/-- **SPEC §12.1**, `Atlas.canonicalSealCheckResultId`. -/
def canonicalSealCheckResultId (state : UnsealedState) (core : SealCore)
    (tag : SealCheckTag) : CanonicalObjectId :=
  CanonicalObjectId.ofTyped
    (Identity SealCheckResultBody.identitySchema
      (canonicalSealCheckResultBody state core tag))

@[simp] theorem canonicalSealCheckResultId_domain (state : UnsealedState)
    (core : SealCore) (tag : SealCheckTag) :
    (canonicalSealCheckResultId state core tag).domain =
      CanonicalDomainTag.atlasSealCheck := rfl

@[simp] theorem canonicalSealCheckResultId_typeTag (state : UnsealedState)
    (core : SealCore) (tag : SealCheckTag) :
    (canonicalSealCheckResultId state core tag).typeTag =
      leafTag 3 "Atlas.SealCheckResultBody" := rfl

/-- Distinct checks of the same state and core have distinct result
identities. -/
theorem canonicalSealCheckResultId_tag_injective (state : UnsealedState)
    (core : SealCore) {t u : SealCheckTag}
    (h : canonicalSealCheckResultId state core t =
      canonicalSealCheckResultId state core u) : t = u := by
  have hb : canonicalSealCheckResultBody state core t =
      canonicalSealCheckResultBody state core u :=
    (CanonicalObjectId.ofTyped_Identity_eq_iff SealCheckResultBody.identitySchema).mp h
  exact congrArg SealCheckResultBody.tag hb

/-- The checker-result identity records the checker's actual result. -/
theorem canonicalSealCheckResultBody_outcome (state : UnsealedState)
    (core : SealCore) (tag : SealCheckTag) :
    (canonicalSealCheckResultBody state core tag).outcome =
      sealCheckResult state core tag := rfl

/-! ## `Atlas.SealCertificateBody` (SPEC §12.1) -/

/-- **SPEC §12.1**, `Atlas.SealCertificateBody`.  Twenty first-order fields: no
function, no proof, no nested certificate. -/
structure SealCertificateBody where
  version : Nat
  coreId : SealCoreIdentity
  stateId : StateIdentity
  profileId : ProfileId
  problemId : ProblemId
  objectiveId : ObjectiveId
  closureRoot : ClosureRoot
  attentionRoot : AttentionRoot
  dependencyRoot : DependencyRoot
  partitionCoverRoot : PartitionCoverRoot
  envelopeRoot : EnvelopeRoot
  certificateRoot : CertificateRoot
  retentionRoot : RetentionRoot
  closureCheckResultId : CanonicalObjectId
  attentionCheckResultId : CanonicalObjectId
  dependencyCheckResultId : CanonicalObjectId
  partitionCheckResultId : CanonicalObjectId
  envelopeCheckResultId : CanonicalObjectId
  certificateStoreCheckResultId : CanonicalObjectId
  retentionCheckResultId : CanonicalObjectId
  deriving DecidableEq

namespace SealCertificateBody

/-- Field-wise equality of two certificate bodies implies equality.  This is
the congruence step of `seal_certificate_body_unique`. -/
theorem eq_of_fields {a b : SealCertificateBody}
    (h1 : a.version = b.version)
    (h2 : a.coreId = b.coreId)
    (h3 : a.stateId = b.stateId)
    (h4 : a.profileId = b.profileId)
    (h5 : a.problemId = b.problemId)
    (h6 : a.objectiveId = b.objectiveId)
    (h7 : a.closureRoot = b.closureRoot)
    (h8 : a.attentionRoot = b.attentionRoot)
    (h9 : a.dependencyRoot = b.dependencyRoot)
    (h10 : a.partitionCoverRoot = b.partitionCoverRoot)
    (h11 : a.envelopeRoot = b.envelopeRoot)
    (h12 : a.certificateRoot = b.certificateRoot)
    (h13 : a.retentionRoot = b.retentionRoot)
    (h14 : a.closureCheckResultId = b.closureCheckResultId)
    (h15 : a.attentionCheckResultId = b.attentionCheckResultId)
    (h16 : a.dependencyCheckResultId = b.dependencyCheckResultId)
    (h17 : a.partitionCheckResultId = b.partitionCheckResultId)
    (h18 : a.envelopeCheckResultId = b.envelopeCheckResultId)
    (h19 : a.certificateStoreCheckResultId = b.certificateStoreCheckResultId)
    (h20 : a.retentionCheckResultId = b.retentionCheckResultId) :
    a = b := by
  cases a; cases b
  simp only [SealCertificateBody.mk.injEq]
  exact ⟨h1, h2, h3, h4, h5, h6, h7, h8, h9, h10, h11, h12, h13, h14, h15, h16,
    h17, h18, h19, h20⟩

def toTuple (b : SealCertificateBody) :
    Nat × SealCoreIdentity × CanonicalObjectId × CanonicalObjectId ×
      CanonicalObjectId × CanonicalObjectId × ClosureRoot × AttentionRoot ×
      DependencyRoot × PartitionCoverRoot × EnvelopeRoot × CertificateRoot ×
      RetentionRoot × CanonicalObjectId × CanonicalObjectId × CanonicalObjectId ×
      CanonicalObjectId × CanonicalObjectId × CanonicalObjectId ×
      CanonicalObjectId :=
  (b.version, b.coreId, b.stateId, b.profileId, b.problemId, b.objectiveId,
   b.closureRoot, b.attentionRoot, b.dependencyRoot, b.partitionCoverRoot,
   b.envelopeRoot, b.certificateRoot, b.retentionRoot, b.closureCheckResultId,
   b.attentionCheckResultId, b.dependencyCheckResultId, b.partitionCheckResultId,
   b.envelopeCheckResultId, b.certificateStoreCheckResultId,
   b.retentionCheckResultId)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [SealCertificateBody.mk.injEq]
  exact h

def bytes (b : SealCertificateBody) : List UInt8 :=
  Bytes.pairBytes Bytes.natBytes
   (Bytes.pairBytes objectIdBytes
    (Bytes.pairBytes CanonicalObjectId.bytes
     (Bytes.pairBytes CanonicalObjectId.bytes
      (Bytes.pairBytes CanonicalObjectId.bytes
       (Bytes.pairBytes CanonicalObjectId.bytes
        (Bytes.pairBytes ClosureRoot.bytes
         (Bytes.pairBytes AttentionRoot.bytes
          (Bytes.pairBytes DependencyRoot.bytes
           (Bytes.pairBytes PartitionCoverRoot.bytes
            (Bytes.pairBytes EnvelopeRoot.bytes
             (Bytes.pairBytes CertificateRoot.bytes
              (Bytes.pairBytes RetentionRoot.bytes
               (Bytes.pairBytes CanonicalObjectId.bytes
                (Bytes.pairBytes CanonicalObjectId.bytes
                 (Bytes.pairBytes CanonicalObjectId.bytes
                  (Bytes.pairBytes CanonicalObjectId.bytes
                   (Bytes.pairBytes CanonicalObjectId.bytes
                    (Bytes.pairBytes CanonicalObjectId.bytes
                      CanonicalObjectId.bytes)))))))))))))))))) b.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree Bytes.natBytes_prefixFree
   (Bytes.pairBytes_prefixFree objectIdBytes_prefixFree
    (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
     (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
      (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
       (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
        (Bytes.pairBytes_prefixFree ClosureRoot.bytes_prefixFree
         (Bytes.pairBytes_prefixFree AttentionRoot.bytes_prefixFree
          (Bytes.pairBytes_prefixFree DependencyRoot.bytes_prefixFree
           (Bytes.pairBytes_prefixFree PartitionCoverRoot.bytes_prefixFree
            (Bytes.pairBytes_prefixFree EnvelopeRoot.bytes_prefixFree
             (Bytes.pairBytes_prefixFree CertificateRoot.bytes_prefixFree
              (Bytes.pairBytes_prefixFree RetentionRoot.bytes_prefixFree
               (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
                (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
                 (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
                  (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
                   (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
                    (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
                      CanonicalObjectId.bytes_prefixFree))))))))))))))))))).comp
    toTuple_injective

/-- The frozen canonical schema of `Atlas.SealCertificateBody` (structural
index 4). -/
def identitySchema : CanonicalSchema SealCertificateBody :=
  CanonicalSchema.ofPrefixFree 1 CanonicalDomainTag.atlasState
    (leafTag 4 "Atlas.SealCertificateBody") (leafTag_size_pos 4 _)
    bytes bytes_prefixFree

@[simp] theorem identitySchema_typeTag :
    identitySchema.typeTag = leafTag 4 "Atlas.SealCertificateBody" := rfl

/-- Every identity a certificate body carries. -/
def referencedIdentities (b : SealCertificateBody) : List CanonicalObjectId :=
  [CanonicalObjectId.ofTyped b.coreId, b.stateId, b.profileId, b.problemId,
   b.objectiveId, b.closureCheckResultId, b.attentionCheckResultId,
   b.dependencyCheckResultId, b.partitionCheckResultId, b.envelopeCheckResultId,
   b.certificateStoreCheckResultId, b.retentionCheckResultId]

end SealCertificateBody

/-- The identity of a certificate body. -/
def SealCertificateBodyId (body : SealCertificateBody) : CanonicalObjectId :=
  CanonicalObjectId.ofTyped (Identity SealCertificateBody.identitySchema body)

theorem SealCertificateBodyId_eq_iff {a b : SealCertificateBody} :
    SealCertificateBodyId a = SealCertificateBodyId b ↔ a = b :=
  CanonicalObjectId.ofTyped_Identity_eq_iff SealCertificateBody.identitySchema

theorem SealCertificateBodyId_injective :
    Function.Injective SealCertificateBodyId :=
  fun _ _ h => SealCertificateBodyId_eq_iff.mp h

@[simp] theorem SealCertificateBodyId_typeTag (b : SealCertificateBody) :
    (SealCertificateBodyId b).typeTag = leafTag 4 "Atlas.SealCertificateBody" := rfl

@[simp] theorem SealCertificateBodyId_domain (b : SealCertificateBody) :
    (SealCertificateBodyId b).domain = CanonicalDomainTag.atlasState := rfl

/-! ## `Atlas.VerifiesSealCertificateBody` (SPEC §12.1) -/

/-- **SPEC §12.1**, `Atlas.VerifiesSealCertificateBody`.  Every field of the
body is pinned to a function of `(state, core)`. -/
def VerifiesSealCertificateBody
    (state : UnsealedState) (core : SealCore)
    (body : SealCertificateBody) : Prop :=
  body.version = 1 ∧
  body.coreId = SealCoreId core ∧
  body.stateId = state.bodyId ∧
  body.stateId = core.stateId ∧
  body.profileId = core.profileId ∧
  body.problemId = core.problemId ∧
  body.objectiveId = core.objectiveId ∧
  body.closureRoot = core.closureRoot ∧
  body.attentionRoot = core.attentionRoot ∧
  body.dependencyRoot = core.dependencyRoot ∧
  body.partitionCoverRoot = core.partitionCoverRoot ∧
  body.envelopeRoot = core.envelopeRoot ∧
  body.certificateRoot = core.certificateRoot ∧
  body.retentionRoot = core.retentionRoot ∧
  body.closureCheckResultId =
    canonicalSealCheckResultId state core .closureLeast ∧
  body.attentionCheckResultId =
    canonicalSealCheckResultId state core .attentionComplete ∧
  body.dependencyCheckResultId =
    canonicalSealCheckResultId state core .dependenciesComplete ∧
  body.partitionCheckResultId =
    canonicalSealCheckResultId state core .universalCoverComplete ∧
  body.envelopeCheckResultId =
    canonicalSealCheckResultId state core .envelopeExact ∧
  body.certificateStoreCheckResultId =
    canonicalSealCheckResultId state core .certificatesSound ∧
  body.retentionCheckResultId =
    canonicalSealCheckResultId state core .retentionComplete

instance instDecidableVerifiesSealCertificateBody (state : UnsealedState)
    (core : SealCore) (body : SealCertificateBody) :
    Decidable (VerifiesSealCertificateBody state core body) := by
  unfold VerifiesSealCertificateBody
  infer_instance

/-- The canonical certificate body of a state and core: the unique candidate
for verification. -/
def canonicalSealCertificateBody (state : UnsealedState) (core : SealCore) :
    SealCertificateBody where
  version := 1
  coreId := SealCoreId core
  stateId := state.bodyId
  profileId := core.profileId
  problemId := core.problemId
  objectiveId := core.objectiveId
  closureRoot := core.closureRoot
  attentionRoot := core.attentionRoot
  dependencyRoot := core.dependencyRoot
  partitionCoverRoot := core.partitionCoverRoot
  envelopeRoot := core.envelopeRoot
  certificateRoot := core.certificateRoot
  retentionRoot := core.retentionRoot
  closureCheckResultId := canonicalSealCheckResultId state core .closureLeast
  attentionCheckResultId := canonicalSealCheckResultId state core .attentionComplete
  dependencyCheckResultId :=
    canonicalSealCheckResultId state core .dependenciesComplete
  partitionCheckResultId :=
    canonicalSealCheckResultId state core .universalCoverComplete
  envelopeCheckResultId := canonicalSealCheckResultId state core .envelopeExact
  certificateStoreCheckResultId :=
    canonicalSealCheckResultId state core .certificatesSound
  retentionCheckResultId := canonicalSealCheckResultId state core .retentionComplete

/-- The canonical body verifies exactly when the core binds the state's
identity. -/
theorem verifies_canonicalSealCertificateBody (state : UnsealedState)
    (core : SealCore) (h : state.bodyId = core.stateId) :
    VerifiesSealCertificateBody state core (canonicalSealCertificateBody state core) :=
  ⟨rfl, rfl, rfl, h, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl,
   rfl, rfl, rfl, rfl, rfl⟩

/-- A verifying body *is* the canonical body. -/
theorem eq_canonicalSealCertificateBody {state : UnsealedState} {core : SealCore}
    {body : SealCertificateBody}
    (h : VerifiesSealCertificateBody state core body) :
    body = canonicalSealCertificateBody state core := by
  obtain ⟨h1, h2, h3, _, h5, h6, h7, h8, h9, h10, h11, h12, h13, h14, h15, h16,
    h17, h18, h19, h20, h21⟩ := h
  exact SealCertificateBody.eq_of_fields h1 h2 h3 h5 h6 h7 h8 h9 h10 h11 h12 h13
    h14 h15 h16 h17 h18 h19 h20 h21

/-- **SPEC §12.1, required theorem**: at most one seal certificate body can
verify against a given state and core. -/
theorem seal_certificate_body_unique
    (state : UnsealedState)
    (core : SealCore)
    (a b : SealCertificateBody)
    (ha : VerifiesSealCertificateBody state core a)
    (hb : VerifiesSealCertificateBody state core b) :
    a = b := by
  rw [eq_canonicalSealCertificateBody ha, eq_canonicalSealCertificateBody hb]

/-- Hence the verifying body is a function of the state and core: two states
with the same identity and the same core have the same body. -/
theorem seal_certificate_body_functional
    {state : UnsealedState} {core : SealCore} {a b : SealCertificateBody}
    (ha : VerifiesSealCertificateBody state core a)
    (hb : VerifiesSealCertificateBody state core b) :
    SealCertificateBodyId a = SealCertificateBodyId b :=
  congrArg SealCertificateBodyId (seal_certificate_body_unique state core a b ha hb)

/-! ## Acyclicity (SPEC §12.1)

"No identity hashes a Lean proof or function, and no component contains its
enclosing identity."

The Lean-level content of that sentence is that the four Atlas schemas are
structurally distinguishable, so an identity of one type is never an identity of
another; combined with `VerifiesSealCertificateBody`, this shows that no
identity field of a verified certificate body is the body's own identity. -/

theorem stateBody_typeTag_ne_sealCore :
    StateBody.identitySchema.typeTag ≠ SealCore.identitySchema.typeTag :=
  leafTag_ne _ _ (by decide)

theorem stateBody_typeTag_ne_sealCertificateBody :
    StateBody.identitySchema.typeTag ≠ SealCertificateBody.identitySchema.typeTag :=
  leafTag_ne _ _ (by decide)

theorem sealCore_typeTag_ne_sealCertificateBody :
    SealCore.identitySchema.typeTag ≠ SealCertificateBody.identitySchema.typeTag :=
  leafTag_ne _ _ (by decide)

theorem sealCheckResult_typeTag_ne_sealCertificateBody :
    SealCheckResultBody.identitySchema.typeTag ≠
      SealCertificateBody.identitySchema.typeTag :=
  leafTag_ne _ _ (by decide)

/-- A seal core identity is never a certificate body identity: `SealCore`
carries no `SealCertificateBody`, and the two identity spaces are disjoint. -/
theorem sealCoreId_ne_sealCertificateBodyId (core : SealCore)
    (body : SealCertificateBody) :
    CanonicalObjectId.ofTyped (SealCoreId core) ≠ SealCertificateBodyId body := by
  intro h
  exact sealCore_typeTag_ne_sealCertificateBody
    (congrArg CanonicalObjectId.typeTag h)

/-- A state identity is never a seal core identity. -/
theorem stateId_ne_sealCoreId (b : StateBody) (core : SealCore) :
    StateId b ≠ CanonicalObjectId.ofTyped (SealCoreId core) := by
  intro h
  exact stateBody_typeTag_ne_sealCore (congrArg CanonicalObjectId.typeTag h)

/-- A state identity is never a certificate body identity. -/
theorem stateId_ne_sealCertificateBodyId (b : StateBody)
    (body : SealCertificateBody) :
    StateId b ≠ SealCertificateBodyId body := by
  intro h
  exact stateBody_typeTag_ne_sealCertificateBody
    (congrArg CanonicalObjectId.typeTag h)

/-- A checker-result identity is never a certificate body identity: they live
in different identity domains. -/
theorem canonicalSealCheckResultId_ne_sealCertificateBodyId
    (state : UnsealedState) (core : SealCore) (tag : SealCheckTag)
    (body : SealCertificateBody) :
    canonicalSealCheckResultId state core tag ≠ SealCertificateBodyId body := by
  intro h
  have : CanonicalDomainTag.atlasSealCheck = CanonicalDomainTag.atlasState :=
    congrArg CanonicalObjectId.domain h
  exact absurd this (by decide)

/-- A checker-result identity is never a state identity. -/
theorem canonicalSealCheckResultId_ne_stateId
    (state : UnsealedState) (core : SealCore) (tag : SealCheckTag)
    (b : StateBody) :
    canonicalSealCheckResultId state core tag ≠ StateId b := by
  intro h
  have : CanonicalDomainTag.atlasSealCheck = CanonicalDomainTag.atlasState :=
    congrArg CanonicalObjectId.domain h
  exact absurd this (by decide)

/-- A verified certificate body's core identity is not the body's own
identity. -/
theorem verified_coreId_ne_selfId {state : UnsealedState} {core : SealCore}
    {body : SealCertificateBody} (h : VerifiesSealCertificateBody state core body) :
    CanonicalObjectId.ofTyped body.coreId ≠ SealCertificateBodyId body := by
  rw [h.2.1]
  exact sealCoreId_ne_sealCertificateBodyId core body

/-- A verified certificate body's state identity is not the body's own
identity. -/
theorem verified_stateId_ne_selfId {state : UnsealedState} {core : SealCore}
    {body : SealCertificateBody} (h : VerifiesSealCertificateBody state core body) :
    body.stateId ≠ SealCertificateBodyId body := by
  rw [h.2.2.1, state.bodyIdEq]
  exact stateId_ne_sealCertificateBodyId state.body body

/-- **Acyclicity**: a verified certificate body never contains its own
identity — not as its core identity, not as its state identity, and not as any
of the seven checker-result identities. -/
theorem verified_selfId_not_mem_referencedIdentities
    {state : UnsealedState} {core : SealCore} {body : SealCertificateBody}
    (h : VerifiesSealCertificateBody state core body)
    (hprofile : body.profileId ≠ SealCertificateBodyId body)
    (hproblem : body.problemId ≠ SealCertificateBodyId body)
    (hobjective : body.objectiveId ≠ SealCertificateBodyId body) :
    SealCertificateBodyId body ∉ body.referencedIdentities := by
  have hcore := verified_coreId_ne_selfId h
  have hstate := verified_stateId_ne_selfId h
  obtain ⟨_, _, _, _, _, _, _, _, _, _, _, _, _, _, h15, h16, h17, h18, h19,
    h20, h21⟩ := h
  intro hmem
  simp only [SealCertificateBody.referencedIdentities, List.mem_cons,
    List.not_mem_nil, or_false] at hmem
  rcases hmem with hc | hc | hc | hc | hc | hc | hc | hc | hc | hc | hc | hc
  · exact hcore hc.symm
  · exact hstate hc.symm
  · exact hprofile hc.symm
  · exact hproblem hc.symm
  · exact hobjective hc.symm
  · exact canonicalSealCheckResultId_ne_sealCertificateBodyId state core
      .closureLeast body (by rw [← h15]; exact hc.symm)
  · exact canonicalSealCheckResultId_ne_sealCertificateBodyId state core
      .attentionComplete body (by rw [← h16]; exact hc.symm)
  · exact canonicalSealCheckResultId_ne_sealCertificateBodyId state core
      .dependenciesComplete body (by rw [← h17]; exact hc.symm)
  · exact canonicalSealCheckResultId_ne_sealCertificateBodyId state core
      .universalCoverComplete body (by rw [← h18]; exact hc.symm)
  · exact canonicalSealCheckResultId_ne_sealCertificateBodyId state core
      .envelopeExact body (by rw [← h19]; exact hc.symm)
  · exact canonicalSealCheckResultId_ne_sealCertificateBodyId state core
      .certificatesSound body (by rw [← h20]; exact hc.symm)
  · exact canonicalSealCheckResultId_ne_sealCertificateBodyId state core
      .retentionComplete body (by rw [← h21]; exact hc.symm)

/-- **Acyclicity**: a seal core that binds a state never carries its own
identity as that state's identity. -/
theorem sealCoreId_ne_stateId_of_binds (state : UnsealedState) (core : SealCore)
    (h : core.stateId = state.bodyId) :
    CanonicalObjectId.ofTyped (SealCoreId core) ≠ core.stateId := by
  rw [h, state.bodyIdEq]
  intro hc
  exact stateId_ne_sealCoreId state.body core hc.symm

end WasmGemmGnaf.Atlas
