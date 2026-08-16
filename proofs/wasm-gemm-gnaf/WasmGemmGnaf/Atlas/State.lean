import WasmGemmGnaf.Foundation.SchemaRegistry
import WasmGemmGnaf.Foundation.Finite
set_option autoImplicit false

/-!
# Atlas: state (SPEC §12.1)

This file transcribes the proof-free `Atlas.StateBody` of SPEC §12.1 together
with every component type it names.  SPEC §12.1 leaves the component carriers
abstract; this file makes concrete first-order choices for all of them
(`CanonicalDeclarationSet`, `DeltaRoot`, `CanonicalObjectMap`,
`CanonicalHypergraph`, `SemanticClosureBody`, `AttentionIndexBody`,
`DependencyGraphBody`, `CandidateFactMap`, `CostSurfaceMap`, `PartitionMap`,
`LowerEnvelopeBody`, `CertificateStoreBody`, and the roots), so that everything
is genuinely encodable with a *proved* injective canonical encoder and genuinely
decidable.

Two structural obligations of SPEC §12.1 are discharged here:

* **Encodability.**  Every `*Body` type is a first-order record: no function
  fields, no proof fields.  Each one gets a prefix-free byte encoder built from
  `Foundation/Bytes.lean` and a `CanonicalSchema` whose injectivity proof is a
  field of the schema, so `Atlas.StateId` is injective
  (`Atlas.StateId_eq_iff`).

* **No optimizer conclusion is a premise of semantic closure.**
  `SemanticClosureBody` mentions only facts, derivation edges and its own root:
  it has no candidate, cost, partition, envelope or certificate field.  This is
  enforced structurally and witnessed by
  `Atlas.semanticClosure_invariant_under_optimizer` and friends: replacing the
  whole optimizer half of a `StateBody` leaves the semantic half, and therefore
  the semantic references, definitionally unchanged.
-/

namespace WasmGemmGnaf.Atlas

open WasmGemmGnaf.Foundation

/-! ## Structural type tags

Every schema in the Atlas layer takes its type tag from `leafTag index name`.
The index makes tags provably distinct without any kernel evaluation of string
literals; the name keeps them human readable. -/

/-- Structural leaf type tag: a numeric structural index followed by a name. -/
def leafTag (index : Nat) (name : String) : ByteArray :=
  TypeTag.leaf (Bytes.natBytes index ++ name.toUTF8.toList)

theorem leafTag_size_pos (index : Nat) (name : String) :
    (leafTag index name).size > 0 :=
  TypeTag.leaf_size_pos _

/-- Distinct structural indices give distinct type tags. -/
theorem leafTag_ne {i j : Nat} (n m : String) (h : i ≠ j) :
    leafTag i n ≠ leafTag j m := by
  intro heq
  have h1 : Bytes.natBytes i ++ n.toUTF8.toList = Bytes.natBytes j ++ m.toUTF8.toList :=
    TypeTag.leaf_injective heq
  exact h (Bytes.natBytes_prefixFree _ _ _ _ h1).1

/-! ## Shared encoder abbreviations -/

namespace Enc

/-- Canonical encoding of a list of erased identifiers. -/
def idList : List CanonicalObjectId → List UInt8 :=
  Bytes.listBytes CanonicalObjectId.bytes

theorem idList_prefixFree : Bytes.PrefixFree idList :=
  Bytes.listBytes_prefixFree CanonicalObjectId.bytes_prefixFree

/-- Canonical encoding of a list of byte strings. -/
def byteArrayList : List ByteArray → List UInt8 :=
  Bytes.listBytes Bytes.byteArrayBytes

theorem byteArrayList_prefixFree : Bytes.PrefixFree byteArrayList :=
  Bytes.listBytes_prefixFree Bytes.byteArrayBytes_prefixFree

/-- Canonical encoding of a list of natural numbers. -/
def natList : List Nat → List UInt8 :=
  Bytes.listBytes Bytes.natBytes

theorem natList_prefixFree : Bytes.PrefixFree natList :=
  Bytes.listBytes_prefixFree Bytes.natBytes_prefixFree

/-- Canonical encoding of an optional identifier. -/
def optionId : Option CanonicalObjectId → List UInt8 :=
  Bytes.optionBytes CanonicalObjectId.bytes

theorem optionId_prefixFree : Bytes.PrefixFree optionId :=
  Bytes.optionBytes_prefixFree CanonicalObjectId.bytes_prefixFree

end Enc

/-! ## Leaf records -/

/-- A retained preimage: the object identity together with the exact bytes it
was computed from (SPEC §12.1 retention). -/
structure PreimageEntry where
  objectId : CanonicalObjectId
  preimageBytes : ByteArray
  deriving DecidableEq

namespace PreimageEntry

def toTuple (e : PreimageEntry) : CanonicalObjectId × ByteArray :=
  (e.objectId, e.preimageBytes)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [PreimageEntry.mk.injEq]
  exact h

def bytes (e : PreimageEntry) : List UInt8 :=
  Bytes.pairBytes CanonicalObjectId.bytes Bytes.byteArrayBytes e.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
    Bytes.byteArrayBytes_prefixFree).comp toTuple_injective

end PreimageEntry

/-- One entry of a canonical object map. -/
structure ObjectEntry where
  key : CanonicalObjectId
  valueBytes : ByteArray
  deriving DecidableEq

namespace ObjectEntry

def toTuple (e : ObjectEntry) : CanonicalObjectId × ByteArray := (e.key, e.valueBytes)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [ObjectEntry.mk.injEq]
  exact h

def bytes (e : ObjectEntry) : List UInt8 :=
  Bytes.pairBytes CanonicalObjectId.bytes Bytes.byteArrayBytes e.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
    Bytes.byteArrayBytes_prefixFree).comp toTuple_injective

end ObjectEntry

/-- A shape hyperedge: an identified edge over an ordered list of endpoints. -/
structure HyperEdge where
  edgeId : CanonicalObjectId
  endpoints : List CanonicalObjectId
  deriving DecidableEq

namespace HyperEdge

def toTuple (e : HyperEdge) : CanonicalObjectId × List CanonicalObjectId :=
  (e.edgeId, e.endpoints)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [HyperEdge.mk.injEq]
  exact h

def bytes (e : HyperEdge) : List UInt8 :=
  Bytes.pairBytes CanonicalObjectId.bytes Enc.idList e.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
    Enc.idList_prefixFree).comp toTuple_injective

end HyperEdge

/-- A semantic derivation edge: premises entail a conclusion.  Note that the
premises are plain semantic object identities; there is no candidate, cost or
envelope component anywhere in this type (SPEC §12.1). -/
structure DerivationEdge where
  edgeId : CanonicalObjectId
  premises : List CanonicalObjectId
  conclusion : CanonicalObjectId
  deriving DecidableEq

namespace DerivationEdge

def toTuple (e : DerivationEdge) :
    CanonicalObjectId × List CanonicalObjectId × CanonicalObjectId :=
  (e.edgeId, e.premises, e.conclusion)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [DerivationEdge.mk.injEq]
  exact h

def bytes (e : DerivationEdge) : List UInt8 :=
  Bytes.pairBytes CanonicalObjectId.bytes
    (Bytes.pairBytes Enc.idList CanonicalObjectId.bytes) e.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
    (Bytes.pairBytes_prefixFree Enc.idList_prefixFree
      CanonicalObjectId.bytes_prefixFree)).comp toTuple_injective

end DerivationEdge

/-! ## Component carriers of `Atlas.StateBody` -/

/-- The canonical declaration base: the exact, ordered declaration byte
strings the state was built from. -/
structure CanonicalDeclarationSet where
  declarations : List ByteArray
  deriving DecidableEq

namespace CanonicalDeclarationSet

def bytes (d : CanonicalDeclarationSet) : List UInt8 := Enc.byteArrayList d.declarations

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Enc.byteArrayList_prefixFree.comp (by
    intro a b h; cases a; cases b; simpa using h)

end CanonicalDeclarationSet

/-- The accumulated delta root: the exact list of applied delta identities. -/
structure DeltaRoot where
  appliedDeltaIds : List CanonicalObjectId
  deriving DecidableEq

namespace DeltaRoot

def bytes (r : DeltaRoot) : List UInt8 := Enc.idList r.appliedDeltaIds

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Enc.idList_prefixFree.comp (by intro a b h; cases a; cases b; simpa using h)

end DeltaRoot

/-- The canonical semantic object map. -/
structure CanonicalObjectMap where
  entries : List ObjectEntry
  deriving DecidableEq

namespace CanonicalObjectMap

def keys (m : CanonicalObjectMap) : List CanonicalObjectId := m.entries.map (·.key)

def bytes (m : CanonicalObjectMap) : List UInt8 :=
  Bytes.listBytes ObjectEntry.bytes m.entries

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.listBytes_prefixFree ObjectEntry.bytes_prefixFree).comp
    (by intro a b h; cases a; cases b; simpa using h)

end CanonicalObjectMap

/-- The canonical shape hypergraph. -/
structure CanonicalHypergraph where
  edges : List HyperEdge
  deriving DecidableEq

namespace CanonicalHypergraph

def bytes (g : CanonicalHypergraph) : List UInt8 :=
  Bytes.listBytes HyperEdge.bytes g.edges

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.listBytes_prefixFree HyperEdge.bytes_prefixFree).comp
    (by intro a b h; cases a; cases b; simpa using h)

end CanonicalHypergraph

/-- The closure root: the exact set of closure fact identities. -/
structure ClosureRoot where
  factIds : List CanonicalObjectId
  deriving DecidableEq

namespace ClosureRoot

def bytes (r : ClosureRoot) : List UInt8 := Enc.idList r.factIds

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Enc.idList_prefixFree.comp (by intro a b h; cases a; cases b; simpa using h)

end ClosureRoot

/-- The semantic closure.

**SPEC §12.1**: "Optimizer conclusions such as best, dominated, or selected
SHALL NOT be premises in semantic closure."  That prohibition is enforced by
this declaration: the type has a fact list, a derivation-edge list and its own
root, and no field of any candidate, cost, partition, envelope or certificate
type exists here or in `DerivationEdge`. -/
structure SemanticClosureBody where
  facts : List CanonicalObjectId
  derivations : List DerivationEdge
  root : ClosureRoot
  deriving DecidableEq

namespace SemanticClosureBody

def toTuple (c : SemanticClosureBody) :
    List CanonicalObjectId × List DerivationEdge × ClosureRoot :=
  (c.facts, c.derivations, c.root)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [SemanticClosureBody.mk.injEq]
  exact h

def bytes (c : SemanticClosureBody) : List UInt8 :=
  Bytes.pairBytes Enc.idList
    (Bytes.pairBytes (Bytes.listBytes DerivationEdge.bytes) ClosureRoot.bytes)
    c.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree Enc.idList_prefixFree
    (Bytes.pairBytes_prefixFree
      (Bytes.listBytes_prefixFree DerivationEdge.bytes_prefixFree)
      ClosureRoot.bytes_prefixFree)).comp toTuple_injective

end SemanticClosureBody

/-- The attention root: the exact list of indexed request signatures. -/
structure AttentionRoot where
  signatures : List ByteArray
  deriving DecidableEq

namespace AttentionRoot

def bytes (r : AttentionRoot) : List UInt8 := Enc.byteArrayList r.signatures

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Enc.byteArrayList_prefixFree.comp (by intro a b h; cases a; cases b; simpa using h)

end AttentionRoot

/-- One attention-index entry: a request signature and the objects it routes to. -/
structure AttentionEntry where
  signature : ByteArray
  targets : List CanonicalObjectId
  deriving DecidableEq

namespace AttentionEntry

def toTuple (e : AttentionEntry) : ByteArray × List CanonicalObjectId :=
  (e.signature, e.targets)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [AttentionEntry.mk.injEq]
  exact h

def bytes (e : AttentionEntry) : List UInt8 :=
  Bytes.pairBytes Bytes.byteArrayBytes Enc.idList e.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree Bytes.byteArrayBytes_prefixFree
    Enc.idList_prefixFree).comp toTuple_injective

end AttentionEntry

/-- The attention index (SPEC §12.2's routing table, in first-order form). -/
structure AttentionIndexBody where
  entries : List AttentionEntry
  root : AttentionRoot
  deriving DecidableEq

namespace AttentionIndexBody

def toTuple (a : AttentionIndexBody) : List AttentionEntry × AttentionRoot :=
  (a.entries, a.root)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [AttentionIndexBody.mk.injEq]
  exact h

def bytes (a : AttentionIndexBody) : List UInt8 :=
  Bytes.pairBytes (Bytes.listBytes AttentionEntry.bytes) AttentionRoot.bytes a.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree
    (Bytes.listBytes_prefixFree AttentionEntry.bytes_prefixFree)
    AttentionRoot.bytes_prefixFree).comp toTuple_injective

end AttentionIndexBody

/-- The dependency root: the exact list of dependency-edge sources. -/
structure DependencyRoot where
  edgeIds : List CanonicalObjectId
  deriving DecidableEq

namespace DependencyRoot

def bytes (r : DependencyRoot) : List UInt8 := Enc.idList r.edgeIds

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Enc.idList_prefixFree.comp (by intro a b h; cases a; cases b; simpa using h)

end DependencyRoot

/-- One dependency edge (SPEC §12.3): an object and the exact objects it
depends on. -/
structure DependencyEdge where
  source : CanonicalObjectId
  dependsOn : List CanonicalObjectId
  deriving DecidableEq

namespace DependencyEdge

def toTuple (e : DependencyEdge) : CanonicalObjectId × List CanonicalObjectId :=
  (e.source, e.dependsOn)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [DependencyEdge.mk.injEq]
  exact h

def bytes (e : DependencyEdge) : List UInt8 :=
  Bytes.pairBytes CanonicalObjectId.bytes Enc.idList e.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
    Enc.idList_prefixFree).comp toTuple_injective

end DependencyEdge

/-- The dependency graph (SPEC §12.3). -/
structure DependencyGraphBody where
  edges : List DependencyEdge
  root : DependencyRoot
  deriving DecidableEq

namespace DependencyGraphBody

def toTuple (d : DependencyGraphBody) : List DependencyEdge × DependencyRoot :=
  (d.edges, d.root)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [DependencyGraphBody.mk.injEq]
  exact h

def bytes (d : DependencyGraphBody) : List UInt8 :=
  Bytes.pairBytes (Bytes.listBytes DependencyEdge.bytes) DependencyRoot.bytes d.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree
    (Bytes.listBytes_prefixFree DependencyEdge.bytes_prefixFree)
    DependencyRoot.bytes_prefixFree).comp toTuple_injective

end DependencyGraphBody

/-- One candidate fact: a candidate identity and its canonical fact bytes. -/
structure CandidateFactEntry where
  candidateId : CanonicalObjectId
  factBytes : ByteArray
  deriving DecidableEq

namespace CandidateFactEntry

def toTuple (e : CandidateFactEntry) : CanonicalObjectId × ByteArray :=
  (e.candidateId, e.factBytes)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [CandidateFactEntry.mk.injEq]
  exact h

def bytes (e : CandidateFactEntry) : List UInt8 :=
  Bytes.pairBytes CanonicalObjectId.bytes Bytes.byteArrayBytes e.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
    Bytes.byteArrayBytes_prefixFree).comp toTuple_injective

end CandidateFactEntry

/-- The candidate fact map. -/
structure CandidateFactMap where
  entries : List CandidateFactEntry
  deriving DecidableEq

namespace CandidateFactMap

def keys (m : CandidateFactMap) : List CanonicalObjectId := m.entries.map (·.candidateId)

def bytes (m : CandidateFactMap) : List UInt8 :=
  Bytes.listBytes CandidateFactEntry.bytes m.entries

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.listBytes_prefixFree CandidateFactEntry.bytes_prefixFree).comp
    (by intro a b h; cases a; cases b; simpa using h)

end CandidateFactMap

/-- One cost surface entry: the exact score and coordinate vector of a
candidate (SPEC §9.1 cost coordinates, stored first order). -/
structure CostSurfaceEntry where
  candidateId : CanonicalObjectId
  score : Nat
  coordinates : List Nat
  deriving DecidableEq

namespace CostSurfaceEntry

def toTuple (e : CostSurfaceEntry) : CanonicalObjectId × Nat × List Nat :=
  (e.candidateId, e.score, e.coordinates)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [CostSurfaceEntry.mk.injEq]
  exact h

def bytes (e : CostSurfaceEntry) : List UInt8 :=
  Bytes.pairBytes CanonicalObjectId.bytes
    (Bytes.pairBytes Bytes.natBytes Enc.natList) e.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
    (Bytes.pairBytes_prefixFree Bytes.natBytes_prefixFree
      Enc.natList_prefixFree)).comp toTuple_injective

end CostSurfaceEntry

/-- The cost surface map. -/
structure CostSurfaceMap where
  entries : List CostSurfaceEntry
  deriving DecidableEq

namespace CostSurfaceMap

def keys (m : CostSurfaceMap) : List CanonicalObjectId := m.entries.map (·.candidateId)

/-- The recorded score of a candidate, if any. -/
def score? (m : CostSurfaceMap) (id : CanonicalObjectId) : Option Nat :=
  (m.entries.find? (fun e => decide (e.candidateId = id))).map (·.score)

def bytes (m : CostSurfaceMap) : List UInt8 :=
  Bytes.listBytes CostSurfaceEntry.bytes m.entries

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.listBytes_prefixFree CostSurfaceEntry.bytes_prefixFree).comp
    (by intro a b h; cases a; cases b; simpa using h)

end CostSurfaceMap

/-- The partition cover root: the exact list of partition identities. -/
structure PartitionCoverRoot where
  partitionIds : List CanonicalObjectId
  deriving DecidableEq

namespace PartitionCoverRoot

def bytes (r : PartitionCoverRoot) : List UInt8 := Enc.idList r.partitionIds

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Enc.idList_prefixFree.comp (by intro a b h; cases a; cases b; simpa using h)

end PartitionCoverRoot

/-- One search partition: an identity and its exact member candidates. -/
structure PartitionEntry where
  partitionId : CanonicalObjectId
  members : List CanonicalObjectId
  deriving DecidableEq

namespace PartitionEntry

def toTuple (e : PartitionEntry) : CanonicalObjectId × List CanonicalObjectId :=
  (e.partitionId, e.members)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [PartitionEntry.mk.injEq]
  exact h

def bytes (e : PartitionEntry) : List UInt8 :=
  Bytes.pairBytes CanonicalObjectId.bytes Enc.idList e.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
    Enc.idList_prefixFree).comp toTuple_injective

end PartitionEntry

/-- The search partition map together with its cover root. -/
structure PartitionMap where
  entries : List PartitionEntry
  coverRoot : PartitionCoverRoot
  deriving DecidableEq

namespace PartitionMap

def toTuple (p : PartitionMap) : List PartitionEntry × PartitionCoverRoot :=
  (p.entries, p.coverRoot)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [PartitionMap.mk.injEq]
  exact h

/-- Every candidate covered by some partition, in partition order. -/
def covered (p : PartitionMap) : List CanonicalObjectId :=
  p.entries.flatMap (·.members)

def bytes (p : PartitionMap) : List UInt8 :=
  Bytes.pairBytes (Bytes.listBytes PartitionEntry.bytes) PartitionCoverRoot.bytes
    p.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree
    (Bytes.listBytes_prefixFree PartitionEntry.bytes_prefixFree)
    PartitionCoverRoot.bytes_prefixFree).comp toTuple_injective

end PartitionMap

/-- The six envelope statuses SPEC §12.4 requires the envelope to distinguish. -/
inductive EnvelopeStatus
  | attainedMinimum
  | infeasibleRegion
  | nonattainedInfimum
  | incompleteCoverage
  | unsupportedProfile
  | invalidatedOrUnsealed
  deriving DecidableEq, Repr, Inhabited

namespace EnvelopeStatus

/-- Structural index, and the canonical one-byte encoding. -/
def index : EnvelopeStatus → Nat
  | attainedMinimum => 0
  | infeasibleRegion => 1
  | nonattainedInfimum => 2
  | incompleteCoverage => 3
  | unsupportedProfile => 4
  | invalidatedOrUnsealed => 5

theorem index_injective : Function.Injective index := by
  intro a b h
  cases a <;> cases b <;> simp_all [index]

def bytes (s : EnvelopeStatus) : List UInt8 := [UInt8.ofNat s.index]

theorem bytes_injective : Function.Injective bytes := by
  intro a b h
  cases a <;> cases b <;> simp_all [bytes, index]

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Bytes.prefixFree_of_constLength bytes 1 (fun s => by cases s <;> rfl) bytes_injective

/-- The complete finite enumeration of envelope statuses. -/
def all : List EnvelopeStatus :=
  [attainedMinimum, infeasibleRegion, nonattainedInfimum, incompleteCoverage,
   unsupportedProfile, invalidatedOrUnsealed]

theorem mem_all (s : EnvelopeStatus) : s ∈ all := by cases s <;> simp [all]

theorem all_nodup : all.Nodup := by decide

instance : Fintype EnvelopeStatus where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

end EnvelopeStatus

/-- The envelope root: the exact list of region identities. -/
structure EnvelopeRoot where
  regionIds : List CanonicalObjectId
  deriving DecidableEq

namespace EnvelopeRoot

def bytes (r : EnvelopeRoot) : List UInt8 := Enc.idList r.regionIds

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Enc.idList_prefixFree.comp (by intro a b h; cases a; cases b; simpa using h)

end EnvelopeRoot

/-- One envelope region (SPEC §12.4). -/
structure EnvelopeRegion where
  regionId : CanonicalObjectId
  status : EnvelopeStatus
  attained : Option CanonicalObjectId
  bound : Nat
  deriving DecidableEq

namespace EnvelopeRegion

def toTuple (r : EnvelopeRegion) :
    CanonicalObjectId × EnvelopeStatus × Option CanonicalObjectId × Nat :=
  (r.regionId, r.status, r.attained, r.bound)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [EnvelopeRegion.mk.injEq]
  exact h

def bytes (r : EnvelopeRegion) : List UInt8 :=
  Bytes.pairBytes CanonicalObjectId.bytes
    (Bytes.pairBytes EnvelopeStatus.bytes
      (Bytes.pairBytes Enc.optionId Bytes.natBytes)) r.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
    (Bytes.pairBytes_prefixFree EnvelopeStatus.bytes_prefixFree
      (Bytes.pairBytes_prefixFree Enc.optionId_prefixFree
        Bytes.natBytes_prefixFree))).comp toTuple_injective

end EnvelopeRegion

/-- The lower envelope (SPEC §12.4). -/
structure LowerEnvelopeBody where
  regions : List EnvelopeRegion
  root : EnvelopeRoot
  deriving DecidableEq

namespace LowerEnvelopeBody

def toTuple (e : LowerEnvelopeBody) : List EnvelopeRegion × EnvelopeRoot :=
  (e.regions, e.root)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [LowerEnvelopeBody.mk.injEq]
  exact h

def bytes (e : LowerEnvelopeBody) : List UInt8 :=
  Bytes.pairBytes (Bytes.listBytes EnvelopeRegion.bytes) EnvelopeRoot.bytes e.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree
    (Bytes.listBytes_prefixFree EnvelopeRegion.bytes_prefixFree)
    EnvelopeRoot.bytes_prefixFree).comp toTuple_injective

end LowerEnvelopeBody

/-- The certificate store root: the exact list of stored certificate
identities. -/
structure CertificateRoot where
  certificateIds : List CanonicalObjectId
  deriving DecidableEq

namespace CertificateRoot

def bytes (r : CertificateRoot) : List UInt8 := Enc.idList r.certificateIds

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Enc.idList_prefixFree.comp (by intro a b h; cases a; cases b; simpa using h)

end CertificateRoot

/-- One stored certificate: its identity, its subject and its exact
dependencies (SPEC §12.3). -/
structure CertificateEntry where
  certificateId : CanonicalObjectId
  subject : CanonicalObjectId
  dependencies : List CanonicalObjectId
  deriving DecidableEq

namespace CertificateEntry

def toTuple (e : CertificateEntry) :
    CanonicalObjectId × CanonicalObjectId × List CanonicalObjectId :=
  (e.certificateId, e.subject, e.dependencies)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [CertificateEntry.mk.injEq]
  exact h

def bytes (e : CertificateEntry) : List UInt8 :=
  Bytes.pairBytes CanonicalObjectId.bytes
    (Bytes.pairBytes CanonicalObjectId.bytes Enc.idList) e.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
    (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
      Enc.idList_prefixFree)).comp toTuple_injective

end CertificateEntry

/-- The certificate store. -/
structure CertificateStoreBody where
  entries : List CertificateEntry
  root : CertificateRoot
  deriving DecidableEq

namespace CertificateStoreBody

def toTuple (s : CertificateStoreBody) : List CertificateEntry × CertificateRoot :=
  (s.entries, s.root)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [CertificateStoreBody.mk.injEq]
  exact h

def bytes (s : CertificateStoreBody) : List UInt8 :=
  Bytes.pairBytes (Bytes.listBytes CertificateEntry.bytes) CertificateRoot.bytes
    s.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree
    (Bytes.listBytes_prefixFree CertificateEntry.bytes_prefixFree)
    CertificateRoot.bytes_prefixFree).comp toTuple_injective

end CertificateStoreBody

/-- The retention root: the exact list of retained object identities. -/
structure RetentionRoot where
  retainedIds : List CanonicalObjectId
  deriving DecidableEq

namespace RetentionRoot

def bytes (r : RetentionRoot) : List UInt8 := Enc.idList r.retainedIds

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Enc.idList_prefixFree.comp (by intro a b h; cases a; cases b; simpa using h)

end RetentionRoot

/-! ## `Atlas.StateBody` (SPEC §12.1) -/

/-- **SPEC §12.1**, `Atlas.StateBody`.  A first-order record: no function
fields, no proof fields, hence canonically encodable. -/
structure StateBody where
  declarationBase : CanonicalDeclarationSet
  accumulatedDeltaRoot : DeltaRoot
  semanticObjects : CanonicalObjectMap
  shapeEdges : CanonicalHypergraph
  semanticClosure : SemanticClosureBody
  attentionIndex : AttentionIndexBody
  dependencyGraph : DependencyGraphBody
  candidateFacts : CandidateFactMap
  costSurfaces : CostSurfaceMap
  searchPartitions : PartitionMap
  lowerEnvelope : LowerEnvelopeBody
  certificates : CertificateStoreBody
  objectiveId : ObjectiveId
  profileId : ProfileId
  problemId : ProblemId
  deriving DecidableEq

namespace StateBody

/-- The flattening used to build the canonical encoder. -/
def toTuple (b : StateBody) :
    CanonicalDeclarationSet × DeltaRoot × CanonicalObjectMap × CanonicalHypergraph ×
      SemanticClosureBody × AttentionIndexBody × DependencyGraphBody ×
      CandidateFactMap × CostSurfaceMap × PartitionMap × LowerEnvelopeBody ×
      CertificateStoreBody × CanonicalObjectId × CanonicalObjectId ×
      CanonicalObjectId :=
  (b.declarationBase, b.accumulatedDeltaRoot, b.semanticObjects, b.shapeEdges,
   b.semanticClosure, b.attentionIndex, b.dependencyGraph, b.candidateFacts,
   b.costSurfaces, b.searchPartitions, b.lowerEnvelope, b.certificates,
   b.objectiveId, b.profileId, b.problemId)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [StateBody.mk.injEq]
  exact h

/-- The canonical prefix-free encoding of a state body. -/
def bytes (b : StateBody) : List UInt8 :=
  Bytes.pairBytes CanonicalDeclarationSet.bytes
   (Bytes.pairBytes DeltaRoot.bytes
    (Bytes.pairBytes CanonicalObjectMap.bytes
     (Bytes.pairBytes CanonicalHypergraph.bytes
      (Bytes.pairBytes SemanticClosureBody.bytes
       (Bytes.pairBytes AttentionIndexBody.bytes
        (Bytes.pairBytes DependencyGraphBody.bytes
         (Bytes.pairBytes CandidateFactMap.bytes
          (Bytes.pairBytes CostSurfaceMap.bytes
           (Bytes.pairBytes PartitionMap.bytes
            (Bytes.pairBytes LowerEnvelopeBody.bytes
             (Bytes.pairBytes CertificateStoreBody.bytes
              (Bytes.pairBytes CanonicalObjectId.bytes
               (Bytes.pairBytes CanonicalObjectId.bytes
                CanonicalObjectId.bytes))))))))))))) b.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree CanonicalDeclarationSet.bytes_prefixFree
   (Bytes.pairBytes_prefixFree DeltaRoot.bytes_prefixFree
    (Bytes.pairBytes_prefixFree CanonicalObjectMap.bytes_prefixFree
     (Bytes.pairBytes_prefixFree CanonicalHypergraph.bytes_prefixFree
      (Bytes.pairBytes_prefixFree SemanticClosureBody.bytes_prefixFree
       (Bytes.pairBytes_prefixFree AttentionIndexBody.bytes_prefixFree
        (Bytes.pairBytes_prefixFree DependencyGraphBody.bytes_prefixFree
         (Bytes.pairBytes_prefixFree CandidateFactMap.bytes_prefixFree
          (Bytes.pairBytes_prefixFree CostSurfaceMap.bytes_prefixFree
           (Bytes.pairBytes_prefixFree PartitionMap.bytes_prefixFree
            (Bytes.pairBytes_prefixFree LowerEnvelopeBody.bytes_prefixFree
             (Bytes.pairBytes_prefixFree CertificateStoreBody.bytes_prefixFree
              (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
               (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
                CanonicalObjectId.bytes_prefixFree)))))))))))))).comp toTuple_injective

/-- The frozen canonical schema of `Atlas.StateBody` (structural index 1). -/
def identitySchema : CanonicalSchema StateBody :=
  CanonicalSchema.ofPrefixFree 1 CanonicalDomainTag.atlasState
    (leafTag 1 "Atlas.StateBody") (leafTag_size_pos 1 _)
    bytes bytes_prefixFree

@[simp] theorem identitySchema_typeTag :
    identitySchema.typeTag = leafTag 1 "Atlas.StateBody" := rfl

@[simp] theorem identitySchema_domain :
    identitySchema.domain = CanonicalDomainTag.atlasState := rfl

/-- Every object identity referenced anywhere in the body. -/
def referencedObjects (b : StateBody) : List CanonicalObjectId :=
  [b.objectiveId, b.profileId, b.problemId] ++
  b.accumulatedDeltaRoot.appliedDeltaIds ++
  b.semanticObjects.keys ++
  b.shapeEdges.edges.flatMap (fun e => e.edgeId :: e.endpoints) ++
  b.semanticClosure.facts ++
  b.semanticClosure.derivations.flatMap
    (fun d => d.edgeId :: d.conclusion :: d.premises) ++
  b.attentionIndex.entries.flatMap (·.targets) ++
  b.dependencyGraph.edges.flatMap (fun e => e.source :: e.dependsOn) ++
  b.candidateFacts.keys ++
  b.costSurfaces.keys ++
  b.searchPartitions.entries.flatMap (fun p => p.partitionId :: p.members) ++
  b.lowerEnvelope.regions.flatMap (fun r => r.regionId :: r.attained.toList) ++
  b.certificates.entries.flatMap
    (fun c => c.certificateId :: c.subject :: c.dependencies)

/-- The *semantic* references: exactly those identities that occur in the
semantic half of the state (declarations, objects, shape edges, closure).  No
candidate, cost, partition, envelope or certificate identity occurs here. -/
def semanticReferences (b : StateBody) : List CanonicalObjectId :=
  [b.objectiveId, b.profileId, b.problemId] ++
  b.accumulatedDeltaRoot.appliedDeltaIds ++
  b.semanticObjects.keys ++
  b.shapeEdges.edges.flatMap (fun e => e.edgeId :: e.endpoints) ++
  b.semanticClosure.facts ++
  b.semanticClosure.derivations.flatMap
    (fun d => d.edgeId :: d.conclusion :: d.premises)

end StateBody

/-! ## Identity of a state (SPEC §12.1) -/

/-- **SPEC §12.1**, `Atlas.StateId`. -/
def StateId (body : StateBody) : StateIdentity :=
  CanonicalObjectId.ofTyped (Identity StateBody.identitySchema body)

/-- Identity comparison of state bodies *is* structural comparison: this is the
`Identity_eq_iff` of `Foundation/Identity.lean` transported through erasure. -/
theorem StateId_eq_iff {a b : StateBody} : StateId a = StateId b ↔ a = b :=
  CanonicalObjectId.ofTyped_Identity_eq_iff StateBody.identitySchema

theorem StateId_injective : Function.Injective StateId :=
  fun _ _ h => StateId_eq_iff.mp h

@[simp] theorem StateId_typeTag (b : StateBody) :
    (StateId b).typeTag = leafTag 1 "Atlas.StateBody" := rfl

@[simp] theorem StateId_domain (b : StateBody) :
    (StateId b).domain = CanonicalDomainTag.atlasState := rfl

/-! ## Retention (SPEC §12.1, `CompleteObjectGraph`) -/

/-- The retained object graph: the exact preimages kept for a state. -/
structure ObjectGraph where
  preimages : List PreimageEntry
  deriving DecidableEq

namespace ObjectGraph

/-- Decidable resolution of one identity against the retained preimages. -/
def resolves (g : ObjectGraph) (id : CanonicalObjectId) : Bool :=
  g.preimages.any (fun e => decide (e.objectId = id))

/-- The retained identities, in retention order. -/
def retainedIds (g : ObjectGraph) : List CanonicalObjectId :=
  g.preimages.map (·.objectId)

/-- The retention root of a retained graph. -/
def retentionRoot (g : ObjectGraph) : RetentionRoot := ⟨g.retainedIds⟩

theorem resolves_iff (g : ObjectGraph) (id : CanonicalObjectId) :
    g.resolves id = true ↔ ∃ e ∈ g.preimages, e.objectId = id := by
  simp [resolves]

/-- A resolved identity is a retained identity. -/
theorem mem_retainedIds_of_resolves {g : ObjectGraph} {id : CanonicalObjectId}
    (h : g.resolves id = true) : id ∈ g.retainedIds := by
  obtain ⟨e, he, heq⟩ := (resolves_iff g id).mp h
  exact heq ▸ List.mem_map_of_mem he

def bytes (g : ObjectGraph) : List UInt8 :=
  Bytes.listBytes PreimageEntry.bytes g.preimages

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.listBytes_prefixFree PreimageEntry.bytes_prefixFree).comp
    (by intro a b h; cases a; cases b; simpa using h)

end ObjectGraph

/-- **SPEC §12.1**, `CompleteObjectGraph body`: a retained graph that resolves
every identity the body references.  The completeness condition is a decidable
side condition, not a stored conclusion. -/
structure CompleteObjectGraph (body : StateBody) where
  graph : ObjectGraph
  complete : ∀ id ∈ body.referencedObjects, graph.resolves id = true

/-- **SPEC §12.1**, `Atlas.UnsealedState`. -/
structure UnsealedState where
  body : StateBody
  bodyId : StateIdentity
  bodyIdEq : bodyId = StateId body
  retainedObjects : CompleteObjectGraph body

namespace UnsealedState

/-- The retention root of the state's retained graph. -/
def retentionRoot (s : UnsealedState) : RetentionRoot :=
  s.retainedObjects.graph.retentionRoot

/-- Every referenced object of an unsealed state is resolvable: this is the
retention obligation of SPEC §12.1, discharged from the state's own field
rather than assumed. -/
theorem resolves_referenced (s : UnsealedState) :
    ∀ id ∈ s.body.referencedObjects, s.retainedObjects.graph.resolves id = true :=
  s.retainedObjects.complete

/-- Every referenced object is retained. -/
theorem referenced_mem_retained (s : UnsealedState) :
    ∀ id ∈ s.body.referencedObjects, id ∈ s.retainedObjects.graph.retainedIds :=
  fun id hid => ObjectGraph.mem_retainedIds_of_resolves (s.retainedObjects.complete id hid)

/-- The stored identity of an unsealed state is the canonical identity of its
body. -/
theorem bodyId_eq (s : UnsealedState) : s.bodyId = StateId s.body := s.bodyIdEq

/-- Equal stored identities force equal bodies. -/
theorem body_eq_of_bodyId_eq {s t : UnsealedState} (h : s.bodyId = t.bodyId) :
    s.body = t.body :=
  StateId_eq_iff.mp (by rw [← s.bodyIdEq, ← t.bodyIdEq]; exact h)

@[simp] theorem bodyId_typeTag (s : UnsealedState) :
    s.bodyId.typeTag = leafTag 1 "Atlas.StateBody" := by
  rw [s.bodyIdEq]; rfl

@[simp] theorem bodyId_domain (s : UnsealedState) :
    s.bodyId.domain = CanonicalDomainTag.atlasState := by
  rw [s.bodyIdEq]; rfl

end UnsealedState

/-! ## Semantic closure carries no optimizer conclusion (SPEC §12.1)

SPEC §12.1: "Optimizer conclusions such as best, dominated, or selected SHALL
NOT be premises in semantic closure.  Semantic facts are computed first;
snapshot identity is fixed; optimization certificates are constructed
afterward."

`SemanticClosureBody` has no field of any optimizer type, so replacing the
entire optimizer half of a state body cannot change the semantic half.  The
following theorems state exactly that. -/

/-- Replace the whole optimizer half of a state body. -/
def StateBody.withOptimizerComponents (b : StateBody)
    (candidateFacts : CandidateFactMap) (costSurfaces : CostSurfaceMap)
    (searchPartitions : PartitionMap) (lowerEnvelope : LowerEnvelopeBody)
    (certificates : CertificateStoreBody) : StateBody :=
  { b with
    candidateFacts := candidateFacts
    costSurfaces := costSurfaces
    searchPartitions := searchPartitions
    lowerEnvelope := lowerEnvelope
    certificates := certificates }

/-- **SPEC §12.1**: the semantic closure is invariant under every replacement
of the optimizer components. -/
theorem semanticClosure_invariant_under_optimizer (b : StateBody)
    (cf : CandidateFactMap) (cs : CostSurfaceMap) (pm : PartitionMap)
    (le : LowerEnvelopeBody) (st : CertificateStoreBody) :
    (b.withOptimizerComponents cf cs pm le st).semanticClosure = b.semanticClosure :=
  rfl

/-- The declaration base, delta root, semantic objects and shape edges are
likewise untouched by the optimizer components. -/
theorem semanticInputs_invariant_under_optimizer (b : StateBody)
    (cf : CandidateFactMap) (cs : CostSurfaceMap) (pm : PartitionMap)
    (le : LowerEnvelopeBody) (st : CertificateStoreBody) :
    (b.withOptimizerComponents cf cs pm le st).declarationBase = b.declarationBase ∧
    (b.withOptimizerComponents cf cs pm le st).accumulatedDeltaRoot =
      b.accumulatedDeltaRoot ∧
    (b.withOptimizerComponents cf cs pm le st).semanticObjects = b.semanticObjects ∧
    (b.withOptimizerComponents cf cs pm le st).shapeEdges = b.shapeEdges :=
  ⟨rfl, rfl, rfl, rfl⟩

/-- Hence the semantic references — the premises available to closure — are
invariant under every optimizer replacement. -/
theorem semanticReferences_invariant_under_optimizer (b : StateBody)
    (cf : CandidateFactMap) (cs : CostSurfaceMap) (pm : PartitionMap)
    (le : LowerEnvelopeBody) (st : CertificateStoreBody) :
    (b.withOptimizerComponents cf cs pm le st).semanticReferences =
      b.semanticReferences :=
  rfl

/-- The semantic references are a sublist-in-membership of all references: the
semantic half never invents an identity of its own. -/
theorem semanticReferences_subset (b : StateBody) :
    ∀ id ∈ b.semanticReferences, id ∈ b.referencedObjects := by
  intro id h
  simp only [StateBody.semanticReferences, List.mem_append] at h
  simp only [StateBody.referencedObjects, List.mem_append]
  exact Or.inl (Or.inl (Or.inl (Or.inl (Or.inl (Or.inl (Or.inl h))))))

end WasmGemmGnaf.Atlas
