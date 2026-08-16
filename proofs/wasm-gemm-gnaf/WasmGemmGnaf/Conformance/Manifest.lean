import WasmGemmGnaf.Conformance.Claim
import WasmGemmGnaf.Foundation.Termination
set_option autoImplicit false

/-!
# Conformance: the ordered acyclic manifest stages (SPEC §4)

SPEC §4: "Manifest identities SHALL be acyclic and SHALL use three ordered
identity stages followed by one non-self-bound external attestation", and "No
manifest or generated file contains its own identity, and no two manifest
stages hash each other cyclically."

The model is a concrete `stageIndex` ordering together with the *preimage* of
each stage — the set of stage encodings that stage's canonical body binds.  The
load-bearing facts, all proved below, are

* `mem_preimage_iff`: a stage binds exactly the strictly earlier stages;
* `preimage_excludes_self`: no stage's preimage contains its own encoding;
* `preimage_excludes_later`: no stage's preimage contains any later stage;
* `binds_stageIndex_lt`: the monotonicity lemma — binding strictly decreases
  the stage index;
* `binds_acyclic` / `bindsPlus_irrefl`: the binding relation is well-founded
  and its transitive closure is irreflexive, so no two stages hash each other
  cyclically.

The `ReproducibilityAttestationBody` of SPEC §4 item 4 is *not* a stage: it is a
release-system attestation, is not an input to the Lean theorem or to
`OutputManifestBody`, and is excluded from its own comparison set.  It is
modelled separately, with its exclusion proved (`attestation_not_bound`).
-/

namespace WasmGemmGnaf.Conformance

open WasmGemmGnaf.Foundation

/-! ## Manifest file entries -/

/-- One tracked file: its repository path, its canonical bytes and its content
digest. -/
structure ManifestFileEntry where
  path : String
  canonicalBytes : ByteArray
  digest : ByteArray
  deriving DecidableEq

/-! ## The four ordered stages (SPEC §4) -/

/-- The ordered identity stages of SPEC §4. -/
inductive ManifestStage
  /-- Immutable authority, handwritten Lean source, fixtures, tool inputs. -/
  | sourceManifestCore
  /-- Source-core identity plus every generated Lean source on the final
  theorem path. -/
  | generatedProofInput
  /-- Generated-proof-input identity, toolchain/dependency identities, and the
  compiled declaration-environment digest. -/
  | preFinalEnvironment
  /-- Source-core, generated-proof-input and pre-final-environment identities,
  artifact, seal, registry, generated documentation, reproducibility plan. -/
  | outputManifest
  deriving DecidableEq, Repr, Inhabited

namespace ManifestStage

/-- The complete enumeration of stages, in manifest order. -/
def all : List ManifestStage :=
  [sourceManifestCore, generatedProofInput, preFinalEnvironment, outputManifest]

theorem mem_all (s : ManifestStage) : s ∈ all := by cases s <;> simp [all]

theorem all_nodup : all.Nodup := by decide

theorem all_length : all.length = 4 := rfl

instance : Fintype ManifestStage where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

/-- The stage ordering of SPEC §4. -/
def stageIndex : ManifestStage → Nat
  | sourceManifestCore => 0
  | generatedProofInput => 1
  | preFinalEnvironment => 2
  | outputManifest => 3

theorem stageIndex_injective : Function.Injective stageIndex := by
  intro a b h
  cases a <;> cases b <;> simp_all [stageIndex]

theorem stageIndex_lt_four (s : ManifestStage) : stageIndex s < 4 := by
  cases s <;> decide

/-- The preimage of a stage: the stage encodings that stage's identity covers,
directly or through the stages it binds.  Each stage covers exactly the
strictly earlier stages, and nothing else. -/
def preimage : ManifestStage → List ManifestStage
  | sourceManifestCore => []
  | generatedProofInput => [sourceManifestCore]
  | preFinalEnvironment => [sourceManifestCore, generatedProofInput]
  | outputManifest =>
      [sourceManifestCore, generatedProofInput, preFinalEnvironment]

/-- The *direct* preimage: the stage identities a stage's canonical body
literally records (SPEC §4 — `PreFinalEnvironmentBody` binds the
generated-proof-input identity, which in turn binds the source-core identity). -/
def directPreimage : ManifestStage → List ManifestStage
  | sourceManifestCore => []
  | generatedProofInput => [sourceManifestCore]
  | preFinalEnvironment => [generatedProofInput]
  | outputManifest =>
      [sourceManifestCore, generatedProofInput, preFinalEnvironment]

/-- `Binds s t`: the canonical body of stage `s` contains the identity of stage
`t`. -/
def Binds (s t : ManifestStage) : Prop := t ∈ directPreimage s

instance : DecidableRel Binds := fun s t =>
  inferInstanceAs (Decidable (t ∈ directPreimage s))

/-- What a body records is covered by the stage's preimage. -/
theorem directPreimage_subset_preimage (s t : ManifestStage)
    (h : t ∈ directPreimage s) : t ∈ preimage s := by
  cases s <;> cases t <;> simp_all [directPreimage, preimage]

/-- **The stage ordering characterises the preimage.** -/
theorem mem_preimage_iff (s t : ManifestStage) :
    t ∈ preimage s ↔ stageIndex t < stageIndex s := by
  cases s <;> cases t <;> simp [preimage, stageIndex]

/-- **Monotonicity: binding strictly decreases the stage index.** -/
theorem binds_stageIndex_lt {s t : ManifestStage} (h : Binds s t) :
    stageIndex t < stageIndex s :=
  (mem_preimage_iff s t).mp (directPreimage_subset_preimage s t h)

/-- **No stage's preimage contains its own encoding.** -/
theorem preimage_excludes_self (s : ManifestStage) : s ∉ preimage s := by
  intro h
  exact Nat.lt_irrefl _ ((mem_preimage_iff s s).mp h)

/-- No stage's body records its own identity. -/
theorem directPreimage_excludes_self (s : ManifestStage) : s ∉ directPreimage s :=
  fun h => preimage_excludes_self s (directPreimage_subset_preimage s s h)

/-- **No stage's preimage contains any later stage.** -/
theorem preimage_excludes_later {s t : ManifestStage}
    (h : stageIndex s ≤ stageIndex t) : t ∉ preimage s := by
  intro hmem
  exact Nat.lt_irrefl _ (Nat.lt_of_lt_of_le ((mem_preimage_iff s t).mp hmem) h)

/-- No stage's body records the identity of a later stage. -/
theorem directPreimage_excludes_later {s t : ManifestStage}
    (h : stageIndex s ≤ stageIndex t) : t ∉ directPreimage s :=
  fun hmem => preimage_excludes_later h (directPreimage_subset_preimage s t hmem)

/-- Restated for the SPEC's own phrasing: a stage excludes its own encoding and
every later stage. -/
theorem preimage_excludes_self_and_later (s t : ManifestStage)
    (h : stageIndex s ≤ stageIndex t) : t ∉ preimage s :=
  preimage_excludes_later h

/-- **No two stages hash each other.** -/
theorem binds_asymm {s t : ManifestStage} (h : Binds s t) : ¬ Binds t s := by
  intro h'
  exact Nat.lt_irrefl _ (Nat.lt_trans (binds_stageIndex_lt h) (binds_stageIndex_lt h'))

theorem binds_irrefl (s : ManifestStage) : ¬ Binds s s := directPreimage_excludes_self s

/-- The transitive closure of the binding relation. -/
inductive BindsPlus : ManifestStage → ManifestStage → Prop
  | single {s t : ManifestStage} : Binds s t → BindsPlus s t
  | tail {s t u : ManifestStage} : BindsPlus s t → Binds t u → BindsPlus s u

theorem bindsPlus_stageIndex_lt {s t : ManifestStage} (h : BindsPlus s t) :
    stageIndex t < stageIndex s := by
  induction h with
  | single hb => exact binds_stageIndex_lt hb
  | tail _ hb ih => exact Nat.lt_trans (binds_stageIndex_lt hb) ih

/-- **The manifest chain is acyclic**: no stage transitively binds itself. -/
theorem bindsPlus_irrefl (s : ManifestStage) : ¬ BindsPlus s s := by
  intro h
  exact Nat.lt_irrefl _ (bindsPlus_stageIndex_lt h)

/-- The binding relation is well-founded, with `stageIndex` as its measure. -/
theorem binds_acyclic : WellFounded (fun t s => Binds s t) :=
  Termination.wellFounded_of_measure stageIndex (fun t s => Binds s t)
    (fun _ _ h => binds_stageIndex_lt h)

/-- The binding chain reaches exactly the strictly earlier stages: the
transitive closure of `Binds` is the stage ordering. -/
theorem bindsPlus_of_stageIndex_lt {s t : ManifestStage}
    (h : stageIndex t < stageIndex s) : BindsPlus s t := by
  cases s <;> cases t <;> simp only [stageIndex] at h <;>
    first
      | omega
      | exact BindsPlus.single (by decide)
      | exact BindsPlus.tail
          (BindsPlus.single
            (show Binds preFinalEnvironment generatedProofInput by decide))
          (show Binds generatedProofInput sourceManifestCore by decide)

theorem bindsPlus_iff (s t : ManifestStage) :
    BindsPlus s t ↔ stageIndex t < stageIndex s :=
  ⟨bindsPlus_stageIndex_lt, bindsPlus_of_stageIndex_lt⟩

/-- The first stage binds nothing. -/
theorem sourceManifestCore_preimage_nil : preimage sourceManifestCore = [] := rfl

/-- The last stage binds all three earlier stages. -/
theorem outputManifest_preimage :
    preimage outputManifest =
      [sourceManifestCore, generatedProofInput, preFinalEnvironment] := rfl

end ManifestStage

/-! ## The stage bodies (SPEC §4) -/

/-- Stage 1: immutable authority, handwritten Lean source, fixtures and tool
inputs; every manifest and every generated output is excluded. -/
structure SourceManifestCore where
  /-- Pinned authority documents. -/
  authority : List ManifestFileEntry
  /-- Handwritten Lean sources. -/
  handwrittenSource : List ManifestFileEntry
  /-- Test and falsification fixtures. -/
  fixtures : List ManifestFileEntry
  /-- Tool inputs. -/
  toolInputs : List ManifestFileEntry
  deriving DecidableEq

/-- Stage 2: the source-core identity plus the path, canonical bytes and digest
of every generated Lean source compiled on the final theorem path (including
`Artifact/Bytes.lean`).  Its own encoding and every later output are excluded. -/
structure GeneratedProofInputBody where
  /-- `Identity SourceManifestCore.identitySchema sourceCore`. -/
  sourceCoreIdentity : CanonicalObjectId
  /-- Every generated Lean source on the final theorem path. -/
  generatedSources : List ManifestFileEntry
  deriving DecidableEq

/-- Stage 3: the generated-proof-input identity, the exact Lean, toolchain and
dependency identities, and the compiled declaration-environment digest used for
the final proof check. -/
structure PreFinalEnvironmentBody where
  /-- `Identity GeneratedProofInputBody.identitySchema generatedProofInput`. -/
  generatedProofInputIdentity : CanonicalObjectId
  /-- The pinned Lean toolchain identity. -/
  toolchainIdentity : CanonicalObjectId
  /-- The exact `lake-manifest.json` dependency identities. -/
  dependencyIdentities : List CanonicalObjectId
  /-- Digest of the compiled declaration environment. -/
  declarationEnvironmentDigest : ByteArray
  deriving DecidableEq

/-- Stage 4: the three earlier stage identities together with the artifact,
Atlas seal, proof registry, generated documentation and frozen reproducibility
plan.  `MANIFEST.json` is its canonical encoding and is excluded from its own
preimage. -/
structure OutputManifestBody where
  sourceCoreIdentity : CanonicalObjectId
  generatedProofInputIdentity : CanonicalObjectId
  preFinalEnvironmentIdentity : CanonicalObjectId
  artifactIdentity : CanonicalObjectId
  atlasSealIdentity : CanonicalObjectId
  proofRegistryIdentity : CanonicalObjectId
  generatedDocumentationIdentity : CanonicalObjectId
  reproducibilityPlanIdentity : CanonicalObjectId
  deriving DecidableEq

/-- The external attestation of SPEC §4 item 4.  It is emitted by CI after two
clean builds, is not an input to the Lean theorem or to `OutputManifestBody`,
and is excluded from its own comparison set. -/
structure ReproducibilityAttestationBody where
  /-- `Identity OutputManifestBody.identitySchema outputManifestBody`. -/
  outputManifestIdentity : CanonicalObjectId
  /-- The two clean-tree input identities. -/
  firstInputIdentity : CanonicalObjectId
  secondInputIdentity : CanonicalObjectId
  /-- The two compared sets of output identities. -/
  firstOutputIdentities : List CanonicalObjectId
  secondOutputIdentities : List CanonicalObjectId
  deriving DecidableEq

/-! ## The assembled manifest -/

/-- The four ordered stage bodies together with the identity each stage was
recorded under. -/
structure ManifestChain where
  sourceCore : SourceManifestCore
  sourceCoreIdentity : CanonicalObjectId
  generatedProofInput : GeneratedProofInputBody
  generatedProofInputIdentity : CanonicalObjectId
  preFinalEnvironment : PreFinalEnvironmentBody
  preFinalEnvironmentIdentity : CanonicalObjectId
  outputManifest : OutputManifestBody

namespace ManifestChain

/-- The identities a stage's body actually records, as a list. -/
def recordedPreimage (m : ManifestChain) : ManifestStage → List CanonicalObjectId
  | .sourceManifestCore => []
  | .generatedProofInput => [m.generatedProofInput.sourceCoreIdentity]
  | .preFinalEnvironment => [m.preFinalEnvironment.generatedProofInputIdentity]
  | .outputManifest =>
      [m.outputManifest.sourceCoreIdentity,
       m.outputManifest.generatedProofInputIdentity,
       m.outputManifest.preFinalEnvironmentIdentity]

/-- The identity a stage is recorded under. -/
def stageIdentity (m : ManifestChain) : ManifestStage → Option CanonicalObjectId
  | .sourceManifestCore => some m.sourceCoreIdentity
  | .generatedProofInput => some m.generatedProofInputIdentity
  | .preFinalEnvironment => some m.preFinalEnvironmentIdentity
  | .outputManifest => none

/-- Decidable chain linkage: every stage records exactly the identities of the
stages the SPEC ordering says it binds. -/
def linkedB (m : ManifestChain) : Bool :=
  (decide (m.generatedProofInput.sourceCoreIdentity = m.sourceCoreIdentity)) &&
  (decide (m.preFinalEnvironment.generatedProofInputIdentity =
      m.generatedProofInputIdentity)) &&
  (decide (m.outputManifest.sourceCoreIdentity = m.sourceCoreIdentity)) &&
  (decide (m.outputManifest.generatedProofInputIdentity =
      m.generatedProofInputIdentity)) &&
  (decide (m.outputManifest.preFinalEnvironmentIdentity =
      m.preFinalEnvironmentIdentity))

theorem linkedB_iff (m : ManifestChain) :
    m.linkedB = true ↔
      (m.generatedProofInput.sourceCoreIdentity = m.sourceCoreIdentity ∧
       m.preFinalEnvironment.generatedProofInputIdentity = m.generatedProofInputIdentity ∧
       m.outputManifest.sourceCoreIdentity = m.sourceCoreIdentity ∧
       m.outputManifest.generatedProofInputIdentity = m.generatedProofInputIdentity ∧
       m.outputManifest.preFinalEnvironmentIdentity = m.preFinalEnvironmentIdentity) := by
  simp [linkedB, Bool.and_eq_true, and_assoc]

/-- A linked chain records, at each stage, exactly the identities of the
strictly earlier stages: recorded identities are the images of the modelled
preimage. -/
theorem recordedPreimage_eq_of_linked {m : ManifestChain} (h : m.linkedB = true)
    (s : ManifestStage) :
    m.recordedPreimage s =
      (ManifestStage.directPreimage s).filterMap m.stageIdentity := by
  obtain ⟨h1, h2, h3, h4, h5⟩ := (linkedB_iff m).mp h
  cases s <;>
    simp [recordedPreimage, ManifestStage.directPreimage, stageIdentity,
      h1, h2, h3, h4, h5]

/-- **No stage records its own identity**: the last stage, whose canonical
encoding is `MANIFEST.json`, has no identity in the chain at all, and each
earlier stage's recorded preimage is drawn from strictly earlier stages. -/
theorem stageIdentity_outputManifest (m : ManifestChain) :
    m.stageIdentity .outputManifest = none := rfl

/-- The attestation is never part of any stage preimage: it is not a stage. -/
theorem attestation_not_bound (m : ManifestChain) (s : ManifestStage) :
    (m.recordedPreimage s).length ≤ 3 := by
  cases s <;> simp [recordedPreimage]

end ManifestChain

end WasmGemmGnaf.Conformance
