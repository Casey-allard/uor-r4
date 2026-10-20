import WasmGemmGnaf.Atlas.Envelope
set_option autoImplicit false

/-!
# Atlas: canonical deltas and canonical form (SPEC §12.5)

This file provides three things the update operator needs.

## 1. The canonical form of a state body

`Atlas.canonicalize` presents every canonical list of a `StateBody` in the
canonical byte order of `Foundation/Order.lean`, with duplicates removed.  It is
a genuine normalisation, not an erasure:

* `Atlas.canonicalize_idempotent` — it is a normal form;
* `Atlas.mem_canonicalize_*` — it preserves the exact *content* of every
  component (membership in both directions);
* `Atlas.canonicalize_objectiveId` / `profileId` / `problemId` — it preserves
  the scope identities.

Consequently `canonicalize x = canonicalize y` is a statement about the content
of `x` and `y`, never a vacuous statement about a constant function; that is
proved as `Atlas.canonicalize_ne_of_declaration_ne`.

## 2. The delta grammar

**The delta grammar is deliberately restricted to be additive.**  A
`Atlas.Delta` is exactly a finite ordered list of canonical declaration byte
strings, and nothing else — there is no retraction, deletion or
"reinterpretation" constructor.

This restriction is required, not cosmetic.  SPEC §12.5 demands
`incremental_eq_full_rebuild`, whose right-hand side is a rebuild from
`state.declarationBase ∪ delta.declarations` and has *no other input*.  A delta
that could delete a declaration would not be recoverable from that union, and a
delta that could rewrite one would make `semanticApplyBody` fail to be a
homomorphism on the declaration base.  With the additive grammar the update is a
homomorphism and the rebuild theorem is a real theorem rather than a
definitional identity of a recompute-everything function.

## 3. The derived semantic content of a declaration

`Atlas.declId`, `Atlas.objectEntryOf`, … deterministically derive the semantic
half of the state from a declaration.  Each is injective, and each is applied by
`List.map`, which is what makes the update a homomorphism.
-/

namespace WasmGemmGnaf.Atlas

open WasmGemmGnaf.Foundation

/-! ## Canonical ordering and duplicate removal

A generic insertion normal form keyed by an injective byte encoding.  The two
facts that matter are `Canon.norm_eq_of_mem_iff` (the normal form depends only
on the *set* of elements) and `Canon.norm_idem`. -/

namespace Canon

variable {α : Type}

/-- Insert `x` into a key-sorted duplicate-free list. -/
def insert (key : α → List UInt8) (x : α) : List α → List α
  | [] => [x]
  | y :: ys =>
      if key y = key x then y :: ys
      else if UnsignedLexicographicLE (key x) (key y) then x :: y :: ys
      else y :: insert key x ys

/-- The canonical form of a list: sorted by key, duplicate free. -/
def norm (key : α → List UInt8) (l : List α) : List α := l.foldr (insert key) []

@[simp] theorem norm_nil (key : α → List UInt8) : norm key [] = [] := rfl

@[simp] theorem norm_cons (key : α → List UInt8) (x : α) (xs : List α) :
    norm key (x :: xs) = insert key x (norm key xs) := rfl

/-- Strictly increasing keys. -/
def SortedBy (key : α → List UInt8) : List α → Prop
  | [] => True
  | x :: xs =>
      (∀ y ∈ xs, UnsignedLexicographicLE (key x) (key y) ∧ key x ≠ key y) ∧
        SortedBy key xs

theorem mem_insert {key : α → List UInt8} (hkey : Function.Injective key) (x z : α) :
    ∀ l : List α, z ∈ insert key x l ↔ z = x ∨ z ∈ l
  | [] => by simp [insert]
  | y :: ys => by
    rw [insert]
    split
    · rename_i h
      have hyx : y = x := hkey h
      subst hyx
      constructor
      · intro hz; exact Or.inr hz
      · intro hz
        rcases hz with rfl | hz
        · exact List.mem_cons_self
        · exact hz
    · split
      · simp only [List.mem_cons]
      · rw [List.mem_cons, mem_insert hkey x z ys, List.mem_cons]
        constructor
        · intro hz
          rcases hz with h1 | h1 | h1
          · exact Or.inr (Or.inl h1)
          · exact Or.inl h1
          · exact Or.inr (Or.inr h1)
        · intro hz
          rcases hz with h1 | h1 | h1
          · exact Or.inr (Or.inl h1)
          · exact Or.inl h1
          · exact Or.inr (Or.inr h1)

theorem sortedBy_insert {key : α → List UInt8} (hkey : Function.Injective key) (x : α) :
    ∀ {l : List α}, SortedBy key l → SortedBy key (insert key x l)
  | [], _ => ⟨fun y hy => absurd hy (by simp), trivial⟩
  | y :: ys, h => by
    rw [insert]
    split
    · exact h
    · rename_i hne
      split
      · rename_i hle
        refine ⟨?_, h⟩
        intro z hz
        rcases List.mem_cons.mp hz with rfl | hz'
        · exact ⟨hle, fun hk => hne hk.symm⟩
        · have hyz := h.1 z hz'
          refine ⟨UnsignedLexicographicLE.trans _ _ _ hle hyz.1, ?_⟩
          intro hk
          have hxy : key x = key y :=
            UnsignedLexicographicLE.antisymm _ _ hle (hk ▸ hyz.1)
          exact hne hxy.symm
      · rename_i hnle
        have hyx : UnsignedLexicographicLE (key y) (key x) := by
          rcases UnsignedLexicographicLE.total (key x) (key y) with h' | h'
          · exact absurd h' hnle
          · exact h'
        refine ⟨?_, sortedBy_insert hkey x h.2⟩
        intro z hz
        rcases (mem_insert hkey x z ys).mp hz with rfl | hz'
        · exact ⟨hyx, hne⟩
        · exact h.1 z hz'

theorem mem_norm {key : α → List UInt8} (hkey : Function.Injective key) (z : α) :
    ∀ l : List α, z ∈ norm key l ↔ z ∈ l
  | [] => by simp
  | x :: xs => by
    rw [norm_cons, mem_insert hkey, mem_norm hkey z xs, List.mem_cons]

theorem sortedBy_norm {key : α → List UInt8} (hkey : Function.Injective key) :
    ∀ l : List α, SortedBy key (norm key l)
  | [] => trivial
  | x :: xs => by
    rw [norm_cons]
    exact sortedBy_insert hkey x (sortedBy_norm hkey xs)

/-- Two key-sorted duplicate-free lists with the same members are equal. -/
theorem eq_of_sortedBy {key : α → List UInt8} (hkey : Function.Injective key) :
    ∀ {l₁ l₂ : List α}, SortedBy key l₁ → SortedBy key l₂ →
      (∀ z, z ∈ l₁ ↔ z ∈ l₂) → l₁ = l₂
  | [], [], _, _, _ => rfl
  | [], y :: _, _, _, h => absurd ((h y).mpr List.mem_cons_self) (by simp)
  | x :: _, [], _, _, h => absurd ((h x).mp List.mem_cons_self) (by simp)
  | x :: xs, y :: ys, h1, h2, h => by
    have hxy : x = y := by
      rcases List.mem_cons.mp ((h x).mp List.mem_cons_self) with hx | hx
      · exact hx
      · rcases List.mem_cons.mp ((h y).mpr List.mem_cons_self) with hy | hy
        · exact hy.symm
        · have g1 := h1.1 y hy
          have g2 := h2.1 x hx
          exact absurd (UnsignedLexicographicLE.antisymm _ _ g1.1 g2.1) g1.2
    subst hxy
    have htail : ∀ z, z ∈ xs ↔ z ∈ ys := by
      intro z
      constructor
      · intro hz
        rcases List.mem_cons.mp ((h z).mp (List.mem_cons_of_mem _ hz)) with rfl | hz'
        · exact absurd rfl (h1.1 z hz).2
        · exact hz'
      · intro hz
        rcases List.mem_cons.mp ((h z).mpr (List.mem_cons_of_mem _ hz)) with rfl | hz'
        · exact absurd rfl (h2.1 z hz).2
        · exact hz'
    rw [eq_of_sortedBy hkey h1.2 h2.2 htail]

/-- **The canonical form depends only on the set of elements.**  This is what
makes the canonical form order and multiplicity insensitive, and therefore what
makes batched updates confluent. -/
theorem norm_eq_of_mem_iff {key : α → List UInt8} (hkey : Function.Injective key)
    {l₁ l₂ : List α} (h : ∀ z, z ∈ l₁ ↔ z ∈ l₂) : norm key l₁ = norm key l₂ :=
  eq_of_sortedBy hkey (sortedBy_norm hkey l₁) (sortedBy_norm hkey l₂)
    (fun z => by rw [mem_norm hkey z l₁, mem_norm hkey z l₂]; exact h z)

/-- The canonical form is a normal form. -/
theorem norm_idem {key : α → List UInt8} (hkey : Function.Injective key) (l : List α) :
    norm key (norm key l) = norm key l :=
  norm_eq_of_mem_iff hkey (fun z => mem_norm hkey z l)

end Canon

/-! ## The keys used by the canonical form -/

/-- The canonical key of a declaration byte string. -/
def declarationKey (d : ByteArray) : List UInt8 := d.toList

theorem declarationKey_injective : Function.Injective declarationKey :=
  Bytes.toList_injective

theorem idKey_injective : Function.Injective CanonicalObjectId.bytes :=
  CanonicalObjectId.bytes_injective

/-! ## The canonical form of a state body -/

/-- **The canonical form of a state body.**  Every canonical list is presented
in the canonical byte order without duplicates; nothing is added and nothing is
dropped. -/
def canonicalize (b : StateBody) : StateBody where
  declarationBase := ⟨Canon.norm declarationKey b.declarationBase.declarations⟩
  accumulatedDeltaRoot :=
    ⟨Canon.norm CanonicalObjectId.bytes b.accumulatedDeltaRoot.appliedDeltaIds⟩
  semanticObjects := ⟨Canon.norm ObjectEntry.bytes b.semanticObjects.entries⟩
  shapeEdges := ⟨Canon.norm HyperEdge.bytes b.shapeEdges.edges⟩
  semanticClosure :=
    ⟨Canon.norm CanonicalObjectId.bytes b.semanticClosure.facts,
     Canon.norm DerivationEdge.bytes b.semanticClosure.derivations,
     ⟨Canon.norm CanonicalObjectId.bytes b.semanticClosure.root.factIds⟩⟩
  attentionIndex :=
    ⟨Canon.norm AttentionEntry.bytes b.attentionIndex.entries,
     ⟨Canon.norm declarationKey b.attentionIndex.root.signatures⟩⟩
  dependencyGraph :=
    ⟨Canon.norm DependencyEdge.bytes b.dependencyGraph.edges,
     ⟨Canon.norm CanonicalObjectId.bytes b.dependencyGraph.root.edgeIds⟩⟩
  candidateFacts := ⟨Canon.norm CandidateFactEntry.bytes b.candidateFacts.entries⟩
  costSurfaces := ⟨Canon.norm CostSurfaceEntry.bytes b.costSurfaces.entries⟩
  searchPartitions :=
    ⟨Canon.norm PartitionEntry.bytes b.searchPartitions.entries,
     ⟨Canon.norm CanonicalObjectId.bytes b.searchPartitions.coverRoot.partitionIds⟩⟩
  lowerEnvelope :=
    ⟨Canon.norm EnvelopeRegion.bytes b.lowerEnvelope.regions,
     ⟨Canon.norm CanonicalObjectId.bytes b.lowerEnvelope.root.regionIds⟩⟩
  certificates :=
    ⟨Canon.norm CertificateEntry.bytes b.certificates.entries,
     ⟨Canon.norm CanonicalObjectId.bytes b.certificates.root.certificateIds⟩⟩
  objectiveId := b.objectiveId
  profileId := b.profileId
  problemId := b.problemId

/-! ### The canonical form preserves content -/

@[simp] theorem canonicalize_objectiveId (b : StateBody) :
    (canonicalize b).objectiveId = b.objectiveId := rfl
@[simp] theorem canonicalize_profileId (b : StateBody) :
    (canonicalize b).profileId = b.profileId := rfl
@[simp] theorem canonicalize_problemId (b : StateBody) :
    (canonicalize b).problemId = b.problemId := rfl

theorem mem_canonicalize_declarations (b : StateBody) (x : ByteArray) :
    x ∈ (canonicalize b).declarationBase.declarations ↔
      x ∈ b.declarationBase.declarations :=
  Canon.mem_norm declarationKey_injective x _

theorem mem_canonicalize_objects (b : StateBody) (x : ObjectEntry) :
    x ∈ (canonicalize b).semanticObjects.entries ↔ x ∈ b.semanticObjects.entries :=
  Canon.mem_norm ObjectEntry.bytes_prefixFree.injective x _

theorem mem_canonicalize_facts (b : StateBody) (x : CanonicalObjectId) :
    x ∈ (canonicalize b).semanticClosure.facts ↔ x ∈ b.semanticClosure.facts :=
  Canon.mem_norm idKey_injective x _

theorem mem_canonicalize_candidates (b : StateBody) (x : CandidateFactEntry) :
    x ∈ (canonicalize b).candidateFacts.entries ↔ x ∈ b.candidateFacts.entries :=
  Canon.mem_norm CandidateFactEntry.bytes_prefixFree.injective x _

theorem mem_canonicalize_regions (b : StateBody) (x : EnvelopeRegion) :
    x ∈ (canonicalize b).lowerEnvelope.regions ↔ x ∈ b.lowerEnvelope.regions :=
  Canon.mem_norm EnvelopeRegion.bytes_prefixFree.injective x _

/-- The canonical form is a normal form. -/
theorem canonicalize_idempotent (b : StateBody) :
    canonicalize (canonicalize b) = canonicalize b := by
  simp only [canonicalize,
    Canon.norm_idem declarationKey_injective, Canon.norm_idem idKey_injective,
    Canon.norm_idem ObjectEntry.bytes_prefixFree.injective,
    Canon.norm_idem HyperEdge.bytes_prefixFree.injective,
    Canon.norm_idem DerivationEdge.bytes_prefixFree.injective,
    Canon.norm_idem AttentionEntry.bytes_prefixFree.injective,
    Canon.norm_idem DependencyEdge.bytes_prefixFree.injective,
    Canon.norm_idem CandidateFactEntry.bytes_prefixFree.injective,
    Canon.norm_idem CostSurfaceEntry.bytes_prefixFree.injective,
    Canon.norm_idem PartitionEntry.bytes_prefixFree.injective,
    Canon.norm_idem EnvelopeRegion.bytes_prefixFree.injective,
    Canon.norm_idem CertificateEntry.bytes_prefixFree.injective]

/-- **Anti-vacuity of `canonicalize`.**  It is not a constant function: bodies
whose declaration content differs stay different after canonicalisation, so
`canonicalize x = canonicalize y` is a real statement about `x` and `y`. -/
theorem canonicalize_ne_of_declaration_ne (b₁ b₂ : StateBody) (x : ByteArray)
    (h₁ : x ∈ b₁.declarationBase.declarations)
    (h₂ : x ∉ b₂.declarationBase.declarations) :
    canonicalize b₁ ≠ canonicalize b₂ := by
  intro h
  refine h₂ ((mem_canonicalize_declarations b₂ x).mp ?_)
  rw [← h]
  exact (mem_canonicalize_declarations b₁ x).mpr h₁

/-! ### Content equality

`SameContent` is the exact hypothesis under which two bodies have the same
canonical form: equal scope identities and equal *sets* of entries in every
component. -/

/-- Two state bodies with the same scope and the same content in every
component. -/
structure SameContent (b₁ b₂ : StateBody) : Prop where
  declarations : ∀ x, x ∈ b₁.declarationBase.declarations ↔
    x ∈ b₂.declarationBase.declarations
  deltaIds : ∀ x, x ∈ b₁.accumulatedDeltaRoot.appliedDeltaIds ↔
    x ∈ b₂.accumulatedDeltaRoot.appliedDeltaIds
  objects : ∀ x, x ∈ b₁.semanticObjects.entries ↔ x ∈ b₂.semanticObjects.entries
  shapeEdges : ∀ x, x ∈ b₁.shapeEdges.edges ↔ x ∈ b₂.shapeEdges.edges
  facts : ∀ x, x ∈ b₁.semanticClosure.facts ↔ x ∈ b₂.semanticClosure.facts
  derivations : ∀ x, x ∈ b₁.semanticClosure.derivations ↔
    x ∈ b₂.semanticClosure.derivations
  closureRoot : ∀ x, x ∈ b₁.semanticClosure.root.factIds ↔
    x ∈ b₂.semanticClosure.root.factIds
  attentionEntries : ∀ x, x ∈ b₁.attentionIndex.entries ↔ x ∈ b₂.attentionIndex.entries
  attentionRoot : ∀ x, x ∈ b₁.attentionIndex.root.signatures ↔
    x ∈ b₂.attentionIndex.root.signatures
  dependencyEdges : ∀ x, x ∈ b₁.dependencyGraph.edges ↔ x ∈ b₂.dependencyGraph.edges
  dependencyRoot : ∀ x, x ∈ b₁.dependencyGraph.root.edgeIds ↔
    x ∈ b₂.dependencyGraph.root.edgeIds
  candidates : ∀ x, x ∈ b₁.candidateFacts.entries ↔ x ∈ b₂.candidateFacts.entries
  costs : ∀ x, x ∈ b₁.costSurfaces.entries ↔ x ∈ b₂.costSurfaces.entries
  partitions : ∀ x, x ∈ b₁.searchPartitions.entries ↔ x ∈ b₂.searchPartitions.entries
  partitionRoot : ∀ x, x ∈ b₁.searchPartitions.coverRoot.partitionIds ↔
    x ∈ b₂.searchPartitions.coverRoot.partitionIds
  regions : ∀ x, x ∈ b₁.lowerEnvelope.regions ↔ x ∈ b₂.lowerEnvelope.regions
  envelopeRoot : ∀ x, x ∈ b₁.lowerEnvelope.root.regionIds ↔
    x ∈ b₂.lowerEnvelope.root.regionIds
  certificateEntries : ∀ x, x ∈ b₁.certificates.entries ↔ x ∈ b₂.certificates.entries
  certificateRoot : ∀ x, x ∈ b₁.certificates.root.certificateIds ↔
    x ∈ b₂.certificates.root.certificateIds
  objectiveId : b₁.objectiveId = b₂.objectiveId
  profileId : b₁.profileId = b₂.profileId
  problemId : b₁.problemId = b₂.problemId

/-- Bodies with the same content have the same canonical form. -/
theorem canonicalize_eq_of_sameContent {b₁ b₂ : StateBody} (h : SameContent b₁ b₂) :
    canonicalize b₁ = canonicalize b₂ := by
  simp only [canonicalize,
    Canon.norm_eq_of_mem_iff declarationKey_injective h.declarations,
    Canon.norm_eq_of_mem_iff idKey_injective h.deltaIds,
    Canon.norm_eq_of_mem_iff ObjectEntry.bytes_prefixFree.injective h.objects,
    Canon.norm_eq_of_mem_iff HyperEdge.bytes_prefixFree.injective h.shapeEdges,
    Canon.norm_eq_of_mem_iff idKey_injective h.facts,
    Canon.norm_eq_of_mem_iff DerivationEdge.bytes_prefixFree.injective h.derivations,
    Canon.norm_eq_of_mem_iff idKey_injective h.closureRoot,
    Canon.norm_eq_of_mem_iff AttentionEntry.bytes_prefixFree.injective h.attentionEntries,
    Canon.norm_eq_of_mem_iff declarationKey_injective h.attentionRoot,
    Canon.norm_eq_of_mem_iff DependencyEdge.bytes_prefixFree.injective h.dependencyEdges,
    Canon.norm_eq_of_mem_iff idKey_injective h.dependencyRoot,
    Canon.norm_eq_of_mem_iff CandidateFactEntry.bytes_prefixFree.injective h.candidates,
    Canon.norm_eq_of_mem_iff CostSurfaceEntry.bytes_prefixFree.injective h.costs,
    Canon.norm_eq_of_mem_iff PartitionEntry.bytes_prefixFree.injective h.partitions,
    Canon.norm_eq_of_mem_iff idKey_injective h.partitionRoot,
    Canon.norm_eq_of_mem_iff EnvelopeRegion.bytes_prefixFree.injective h.regions,
    Canon.norm_eq_of_mem_iff idKey_injective h.envelopeRoot,
    Canon.norm_eq_of_mem_iff CertificateEntry.bytes_prefixFree.injective h.certificateEntries,
    Canon.norm_eq_of_mem_iff idKey_injective h.certificateRoot,
    h.objectiveId, h.profileId, h.problemId]

/-! ## Declarations and their derived semantic content -/

/-- The frozen canonical schema of a declaration (structural index 5). -/
def declarationSchema : CanonicalSchema ByteArray :=
  CanonicalSchema.ofPrefixFree 1 CanonicalDomainTag.delta
    (leafTag 5 "Atlas.Declaration") (leafTag_size_pos 5 _)
    Bytes.byteArrayBytes Bytes.byteArrayBytes_prefixFree

/-- The canonical identity of a declaration. -/
def declId (d : ByteArray) : CanonicalObjectId :=
  CanonicalObjectId.ofTyped (Identity declarationSchema d)

theorem declId_eq_iff {a b : ByteArray} : declId a = declId b ↔ a = b :=
  CanonicalObjectId.ofTyped_Identity_eq_iff declarationSchema

theorem declId_injective : Function.Injective declId :=
  fun _ _ h => declId_eq_iff.mp h

/-- The semantic object a declaration denotes. -/
def objectEntryOf (d : ByteArray) : ObjectEntry := ⟨declId d, d⟩

/-- The shape edge a declaration contributes. -/
def hyperEdgeOf (d : ByteArray) : HyperEdge := ⟨declId d, [declId d]⟩

/-- The premise-free derivation that admits a declaration as a closure fact. -/
def derivationOf (d : ByteArray) : DerivationEdge := ⟨declId d, [], declId d⟩

/-- The attention bucket a declaration occupies: its own bytes are its
applicability signature. -/
def attentionEntryOf (d : ByteArray) : AttentionEntry := ⟨d, [declId d]⟩

/-- The (empty) dependency edge of a declaration. -/
def dependencyEdgeOf (d : ByteArray) : DependencyEdge := ⟨declId d, []⟩

theorem objectEntryOf_injective : Function.Injective objectEntryOf := by
  intro a b h
  simpa [objectEntryOf] using congrArg ObjectEntry.valueBytes h

theorem hyperEdgeOf_injective : Function.Injective hyperEdgeOf := by
  intro a b h
  exact declId_injective (by simpa [hyperEdgeOf] using congrArg HyperEdge.edgeId h)

theorem derivationOf_injective : Function.Injective derivationOf := by
  intro a b h
  exact declId_injective (by simpa [derivationOf] using congrArg DerivationEdge.edgeId h)

theorem attentionEntryOf_injective : Function.Injective attentionEntryOf := by
  intro a b h
  simpa [attentionEntryOf] using congrArg AttentionEntry.signature h

theorem dependencyEdgeOf_injective : Function.Injective dependencyEdgeOf := by
  intro a b h
  exact declId_injective (by simpa [dependencyEdgeOf] using congrArg DependencyEdge.source h)

/-! ## Declaration sets -/

namespace CanonicalDeclarationSet

/-- Decidable membership of a declaration. -/
def memDecl (l : List ByteArray) (x : ByteArray) : Bool :=
  l.any (fun y => decide (y = x))

theorem memDecl_iff (l : List ByteArray) (x : ByteArray) :
    memDecl l x = true ↔ x ∈ l := by
  constructor
  · intro h
    simp only [memDecl, List.any_eq_true, decide_eq_true_eq] at h
    obtain ⟨y, hy, rfl⟩ := h
    exact hy
  · intro h
    simp only [memDecl, List.any_eq_true, decide_eq_true_eq]
    exact ⟨x, h, rfl⟩

/-- The declarations of `extra` that are genuinely new relative to `base`
(SPEC §12.5 step 1: "accumulate genuinely new objects and edges"). -/
def newIn (base extra : CanonicalDeclarationSet) : List ByteArray :=
  extra.declarations.filter (fun x => !(memDecl base.declarations x))

/-- The union of two declaration sets: the base followed by the genuinely new
declarations of the extension. -/
def union (base extra : CanonicalDeclarationSet) : CanonicalDeclarationSet :=
  ⟨base.declarations ++ newIn base extra⟩

instance : Union CanonicalDeclarationSet := ⟨union⟩

@[simp] theorem union_def (base extra : CanonicalDeclarationSet) :
    base ∪ extra = ⟨base.declarations ++ newIn base extra⟩ := rfl

theorem mem_newIn {base extra : CanonicalDeclarationSet} {x : ByteArray} :
    x ∈ newIn base extra ↔ x ∈ extra.declarations ∧ x ∉ base.declarations := by
  simp only [newIn, List.mem_filter, Bool.not_eq_true']
  constructor
  · intro h
    refine ⟨h.1, fun hc => ?_⟩
    rw [(memDecl_iff base.declarations x).mpr hc] at h
    exact absurd h.2 (by simp)
  · intro h
    refine ⟨h.1, ?_⟩
    cases hm : memDecl base.declarations x with
    | false => rfl
    | true => exact absurd ((memDecl_iff base.declarations x).mp hm) h.2

theorem mem_union {base extra : CanonicalDeclarationSet} {x : ByteArray} :
    x ∈ (base ∪ extra).declarations ↔
      x ∈ base.declarations ∨ x ∈ extra.declarations := by
  rw [union_def]
  simp only [List.mem_append, mem_newIn]
  constructor
  · intro h
    rcases h with h | h
    · exact Or.inl h
    · exact Or.inr h.1
  · intro h
    rcases h with h | h
    · exact Or.inl h
    · by_cases hb : x ∈ base.declarations
      · exact Or.inl hb
      · exact Or.inr ⟨h, hb⟩

@[simp] theorem newIn_nil (base : CanonicalDeclarationSet) :
    newIn base ⟨[]⟩ = [] := rfl

@[simp] theorem union_nil (base : CanonicalDeclarationSet) :
    base ∪ (⟨[]⟩ : CanonicalDeclarationSet) = base := by
  rw [union_def, newIn_nil, List.append_nil]

/-- Reapplying the same extension adds nothing: the union is idempotent in its
second argument. -/
theorem newIn_union_self (base extra : CanonicalDeclarationSet) :
    newIn (base ∪ extra) extra = [] := by
  rw [newIn]
  apply List.filter_eq_nil_iff.mpr
  intro x hx
  simp only [Bool.not_eq_true', Bool.not_eq_false]
  exact (memDecl_iff _ x).mpr (mem_union.mpr (Or.inr hx))

end CanonicalDeclarationSet

/-! ## The delta (SPEC §12.5)

**Restricted, additive grammar** — see the file header for why. -/

/-- **SPEC §12.5**, `Atlas.Delta`: a first-order canonical delta.  It is exactly
a finite ordered list of canonical declaration byte strings. -/
structure Delta where
  declarations : CanonicalDeclarationSet
  deriving DecidableEq

namespace Delta

/-- **SPEC §12.5**, `Atlas.Delta.empty`. -/
def empty : Delta := ⟨⟨[]⟩⟩

@[simp] theorem empty_declarations : empty.declarations = ⟨[]⟩ := rfl

/-- A delta is trivial when it declares nothing. -/
def isTrivial (d : Delta) : Bool := d.declarations.declarations.isEmpty

@[simp] theorem isTrivial_empty : empty.isTrivial = true := rfl

theorem isTrivial_iff (d : Delta) :
    d.isTrivial = true ↔ d.declarations.declarations = [] := by
  cases d with
  | mk decls => cases decls with
    | mk l => cases l <;> simp [isTrivial]

/-- The canonical prefix-free encoding of a delta. -/
def bytes (d : Delta) : List UInt8 :=
  CanonicalDeclarationSet.bytes d.declarations

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  CanonicalDeclarationSet.bytes_prefixFree.comp
    (by intro a b h; cases a; cases b; simpa using h)

/-- The frozen canonical schema of a delta (structural index 6). -/
def identitySchema : CanonicalSchema Delta :=
  CanonicalSchema.ofPrefixFree 1 CanonicalDomainTag.delta
    (leafTag 6 "Atlas.Delta") (leafTag_size_pos 6 _)
    bytes bytes_prefixFree

end Delta

/-- **SPEC §12.5**, the canonical identity of a delta. -/
def DeltaId (d : Delta) : CanonicalObjectId :=
  CanonicalObjectId.ofTyped (Identity Delta.identitySchema d)

theorem DeltaId_eq_iff {a b : Delta} : DeltaId a = DeltaId b ↔ a = b :=
  CanonicalObjectId.ofTyped_Identity_eq_iff Delta.identitySchema

theorem DeltaId_injective : Function.Injective DeltaId :=
  fun _ _ h => DeltaId_eq_iff.mp h

/-- The genuinely new declarations a delta contributes to a state body. -/
def newDeclarations (base : CanonicalDeclarationSet) (d : Delta) : List ByteArray :=
  CanonicalDeclarationSet.newIn base d.declarations

@[simp] theorem newDeclarations_empty (base : CanonicalDeclarationSet) :
    newDeclarations base Delta.empty = [] := rfl

theorem mem_newDeclarations {base : CanonicalDeclarationSet} {d : Delta} {x : ByteArray} :
    x ∈ newDeclarations base d ↔
      x ∈ d.declarations.declarations ∧ x ∉ base.declarations :=
  CanonicalDeclarationSet.mem_newIn

theorem union_eq_append_newDeclarations (base : CanonicalDeclarationSet) (d : Delta) :
    base ∪ d.declarations = ⟨base.declarations ++ newDeclarations base d⟩ := rfl

/-! ## Compatibility of deltas (SPEC §12.5, `Atlas.Compatible`) -/

/-- **SPEC §12.5**, `Atlas.Compatible`: two deltas are compatible when they
declare disjoint sets of declarations, so neither can shadow the other.  This is
exactly the condition under which the *genuinely new* content of one delta is
independent of whether the other has already been applied. -/
def Compatible (left right : Delta) : Prop :=
  ∀ x ∈ left.declarations.declarations, x ∉ right.declarations.declarations

theorem Compatible.symm {left right : Delta} (h : Compatible left right) :
    Compatible right left := by
  intro x hx hc
  exact h x hc hx

/-- The decidable form of `Compatible`. -/
def compatibleCheck (left right : Delta) : Bool :=
  left.declarations.declarations.all
    (fun x => !(CanonicalDeclarationSet.memDecl right.declarations.declarations x))

theorem compatibleCheck_iff (left right : Delta) :
    compatibleCheck left right = true ↔ Compatible left right := by
  simp only [compatibleCheck, List.all_eq_true, Bool.not_eq_true']
  constructor
  · intro h x hx hc
    have hx' := h x hx
    rw [(CanonicalDeclarationSet.memDecl_iff _ x).mpr hc] at hx'
    exact absurd hx' (by simp)
  · intro h x hx
    cases hm : CanonicalDeclarationSet.memDecl right.declarations.declarations x with
    | false => rfl
    | true => exact absurd ((CanonicalDeclarationSet.memDecl_iff _ x).mp hm) (h x hx)

instance (left right : Delta) : Decidable (Compatible left right) :=
  decidable_of_iff _ (compatibleCheck_iff left right)

/-- Under compatibility, applying `left` first does not change which
declarations of `right` are new. -/
theorem newDeclarations_union_of_compatible {base : CanonicalDeclarationSet}
    {left right : Delta} (h : Compatible left right) :
    ∀ x, x ∈ newDeclarations (base ∪ left.declarations) right ↔
      x ∈ newDeclarations base right := by
  intro x
  rw [mem_newDeclarations, mem_newDeclarations]
  constructor
  · intro hx
    exact ⟨hx.1, fun hc => hx.2 (CanonicalDeclarationSet.mem_union.mpr (Or.inl hc))⟩
  · intro hx
    refine ⟨hx.1, fun hc => ?_⟩
    rcases CanonicalDeclarationSet.mem_union.mp hc with hc' | hc'
    · exact hx.2 hc'
    · exact h x hc' hx.1

end WasmGemmGnaf.Atlas
