import WasmGemmGnaf.Atlas.State
set_option autoImplicit false

/-!
# Atlas: attention (SPEC §12.2)

SPEC §12.2 requires `Atlas.attend : StateBody → RequestSignature →
CanonicalSet CanonicalObjectId` to be "an exact work-routing mechanism" over
canonical applicability signatures.

This file supplies it as a *real, total, decidable* function: `attend` is a
structurally recursive computation over the recorded attention index, it
terminates on every input, and its result is a `CanonicalSet` — a duplicate-free
list whose `Nodup` proof is produced by the construction (`dedupIds_nodup`), not
assumed.

## Proved

* `Atlas.attend_deterministic` — attention is a function of the recorded index
  and the request alone.
* `Atlas.attend_mem_iff` — exact characterisation of the routed set.
* `Atlas.attend_subset_indexTargets`, `Atlas.attend_subset_objects` — the result
  is a subset of the objects the state records (the latter under the state's own
  well-formedness condition, which is decidable and provably equivalent to the
  side condition already checked by `Atlas.attentionCompleteCheck`).
* `Atlas.attend_monotone` — widening the request can only add routed objects.

## Omitted, deliberately

`attention_no_optimum_relevant_false_negative` (SPEC §12.2) is **not** in this
repository.  Its conclusion quantifies over `Universal.SystemEvaluation`, which
does not exist in this repository, and its content is the undischarged universal
coverage obligation of SPEC §10.5 — the same obligation
`Atlas/CoverageScope.lean` proves the seal's cover check cannot supply.  Stating
it would require assuming exactly what it is supposed to prove.

## Anti-vacuity (UOR-GNAF §18: index presence proves nothing)

`Atlas.attend_blind_to_optimizer_state` proves that `attend` is determined by
the recorded attention index alone: two states whose indexes agree route
identically no matter how their candidate facts, cost surfaces, partitions,
envelope, or certificates differ.  Therefore "object `x` is in
`attend s request`" carries **no** information about optimality, feasibility, or
cost of `x`; it says only that some indexer wrote `x` under a matching
signature.
-/

namespace WasmGemmGnaf.Atlas

open WasmGemmGnaf.Foundation

/-! ## Canonical sets of identifiers -/

/-- Duplicate removal, keeping the last occurrence.  Structurally recursive,
hence total. -/
def dedupIds : List CanonicalObjectId → List CanonicalObjectId
  | [] => []
  | x :: t => if x ∈ dedupIds t then dedupIds t else x :: dedupIds t

theorem mem_dedupIds (l : List CanonicalObjectId) (x : CanonicalObjectId) :
    x ∈ dedupIds l ↔ x ∈ l := by
  induction l with
  | nil => simp [dedupIds]
  | cons y t ih =>
    rw [dedupIds]
    by_cases h : y ∈ dedupIds t
    · rw [if_pos h]
      constructor
      · intro hx; exact List.mem_cons_of_mem _ (ih.mp hx)
      · intro hx
        rcases List.mem_cons.mp hx with rfl | hx'
        · exact h
        · exact ih.mpr hx'
    · rw [if_neg h]
      constructor
      · intro hx
        rcases List.mem_cons.mp hx with rfl | hx'
        · exact List.mem_cons_self
        · exact List.mem_cons_of_mem _ (ih.mp hx')
      · intro hx
        rcases List.mem_cons.mp hx with rfl | hx'
        · exact List.mem_cons_self
        · exact List.mem_cons_of_mem _ (ih.mpr hx')

theorem dedupIds_nodup (l : List CanonicalObjectId) : (dedupIds l).Nodup := by
  induction l with
  | nil => simp [dedupIds]
  | cons y t ih =>
    rw [dedupIds]
    by_cases h : y ∈ dedupIds t
    · rw [if_pos h]; exact ih
    · rw [if_neg h]; exact List.nodup_cons.mpr ⟨h, ih⟩

/-- A canonical set: a duplicate-free list in recorded order.  The `Nodup` field
is a proof obligation of every construction, so no duplicate can be smuggled
into a routing result. -/
structure CanonicalSet (α : Type) where
  elems : List α
  nodup : elems.Nodup

namespace CanonicalSet

/-- Membership in a canonical set. -/
def Mem {α : Type} (s : CanonicalSet α) (x : α) : Prop := x ∈ s.elems

instance {α : Type} : Membership α (CanonicalSet α) where
  mem s x := Mem s x

/-- Inclusion of canonical sets. -/
def Subset {α : Type} (s t : CanonicalSet α) : Prop := ∀ x, Mem s x → Mem t x

theorem Subset.refl {α : Type} (s : CanonicalSet α) : Subset s s := fun _ h => h

theorem Subset.trans {α : Type} {s t u : CanonicalSet α}
    (h₁ : Subset s t) (h₂ : Subset t u) : Subset s u := fun x h => h₂ x (h₁ x h)

/-- Two canonical sets with the same element list are equal. -/
theorem eq_of_elems_eq {α : Type} {s t : CanonicalSet α} (h : s.elems = t.elems) :
    s = t := by
  cases s; cases t
  simp only [CanonicalSet.mk.injEq]
  exact h

/-- The canonical set of a list of identifiers. -/
def ofIds (l : List CanonicalObjectId) : CanonicalSet CanonicalObjectId :=
  ⟨dedupIds l, dedupIds_nodup l⟩

@[simp] theorem mem_ofIds (l : List CanonicalObjectId) (x : CanonicalObjectId) :
    Mem (ofIds l) x ↔ x ∈ l := mem_dedupIds l x

/-- Membership in a canonical set of identifiers is decidable. -/
instance decidableMemIds (s : CanonicalSet CanonicalObjectId) (x : CanonicalObjectId) :
    Decidable (Mem s x) := List.instDecidableMemOfLawfulBEq x s.elems

end CanonicalSet

/-! ## Request signatures (SPEC §12.2) -/

/-- A canonical applicability signature request: the exact set of applicability
tokens the request demands.  Every shape and edge is indexed under such tokens
(`AttentionEntry.signature`). -/
structure RequestSignature where
  tokens : List ByteArray
  deriving DecidableEq

namespace RequestSignature

/-- The request matches an indexed signature when it demands that token. -/
def applies (request : RequestSignature) (signature : ByteArray) : Bool :=
  request.tokens.any (fun t => decide (t = signature))

theorem applies_iff (request : RequestSignature) (signature : ByteArray) :
    request.applies signature = true ↔ signature ∈ request.tokens := by
  simp only [applies, List.any_eq_true, decide_eq_true_eq]
  exact ⟨fun ⟨t, ht, htv⟩ => htv ▸ ht, fun h => ⟨signature, h, rfl⟩⟩

/-- Widening the demanded token set only ever matches more signatures. -/
theorem applies_mono {r₁ r₂ : RequestSignature}
    (h : ∀ t ∈ r₁.tokens, t ∈ r₂.tokens) (signature : ByteArray)
    (h₁ : r₁.applies signature = true) : r₂.applies signature = true :=
  (applies_iff r₂ signature).mpr (h signature ((applies_iff r₁ signature).mp h₁))

def bytes (r : RequestSignature) : List UInt8 := Enc.byteArrayList r.tokens

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Enc.byteArrayList_prefixFree.comp (by intro a b h; cases a; cases b; simpa using h)

/-- The frozen canonical schema of a request signature (structural index 6). -/
def identitySchema : CanonicalSchema RequestSignature :=
  CanonicalSchema.ofPrefixFree 1 CanonicalDomainTag.request
    (leafTag 6 "Atlas.RequestSignature") (leafTag_size_pos 6 _)
    bytes bytes_prefixFree

end RequestSignature

/-- The canonical identity of a request signature. -/
def RequestSignatureId (r : RequestSignature) : RequestId :=
  CanonicalObjectId.ofTyped (Identity RequestSignature.identitySchema r)

theorem RequestSignatureId_eq_iff {a b : RequestSignature} :
    RequestSignatureId a = RequestSignatureId b ↔ a = b :=
  CanonicalObjectId.ofTyped_Identity_eq_iff RequestSignature.identitySchema

/-! ## `Atlas.attend` (SPEC §12.2) -/

/-- The routed identities, in index order, before canonicalisation. -/
def attendTargets (s : StateBody) (request : RequestSignature) :
    List CanonicalObjectId :=
  (s.attentionIndex.entries.filter (fun e => request.applies e.signature)).flatMap
    (·.targets)

theorem mem_attendTargets_iff (s : StateBody) (request : RequestSignature)
    (x : CanonicalObjectId) :
    x ∈ attendTargets s request ↔
      ∃ e ∈ s.attentionIndex.entries,
        request.applies e.signature = true ∧ x ∈ e.targets := by
  simp only [attendTargets, List.mem_flatMap, List.mem_filter]
  constructor
  · intro ⟨e, ⟨he, happ⟩, hx⟩; exact ⟨e, he, happ, hx⟩
  · intro ⟨e, he, happ, hx⟩; exact ⟨e, ⟨he, happ⟩, hx⟩

/-- **SPEC §12.2**, `Atlas.attend`: exact work routing from a state and a
canonical applicability request to the canonical set of objects that must be
considered.  Total and decidable by construction. -/
def attend (s : StateBody) (request : RequestSignature) :
    CanonicalSet CanonicalObjectId :=
  CanonicalSet.ofIds (attendTargets s request)

/-- Exact characterisation of the routed set. -/
theorem attend_mem_iff (s : StateBody) (request : RequestSignature)
    (x : CanonicalObjectId) :
    CanonicalSet.Mem (attend s request) x ↔
      ∃ e ∈ s.attentionIndex.entries,
        request.applies e.signature = true ∧ x ∈ e.targets := by
  rw [attend, CanonicalSet.mem_ofIds, mem_attendTargets_iff]

/-- The result is duplicate free. -/
theorem attend_nodup (s : StateBody) (request : RequestSignature) :
    (attend s request).elems.Nodup := (attend s request).nodup

/-- Deciding whether an object is routed. -/
def attendContains (s : StateBody) (request : RequestSignature)
    (x : CanonicalObjectId) : Bool :=
  decide (CanonicalSet.Mem (attend s request) x)

theorem attendContains_iff (s : StateBody) (request : RequestSignature)
    (x : CanonicalObjectId) :
    attendContains s request x = true ↔ CanonicalSet.Mem (attend s request) x := by
  simp [attendContains]

/-! ### Determinism -/

/-- **Determinism.**  Attention is a function of the recorded attention index
and the request: equal indexes and equal requests give the identical canonical
set — the same list, in the same order. -/
theorem attend_deterministic (s₁ s₂ : StateBody) (r₁ r₂ : RequestSignature)
    (hs : s₁.attentionIndex = s₂.attentionIndex) (hr : r₁ = r₂) :
    attend s₁ r₁ = attend s₂ r₂ := by
  subst hr
  simp only [attend, attendTargets, hs]

/-- The same fact for one fixed state: `attend` is single valued. -/
theorem attend_functional (s : StateBody) (r : RequestSignature)
    (t u : CanonicalSet CanonicalObjectId)
    (ht : t = attend s r) (hu : u = attend s r) : t = u := by rw [ht, hu]

/-! ### Subset of the state's objects -/

/-- Every routed target of every index entry is a recorded semantic object.
This is precisely the side condition `Atlas.attentionCompleteCheck` verifies. -/
def AttentionTargetsKnown (s : StateBody) : Prop :=
  ∀ e ∈ s.attentionIndex.entries, ∀ x ∈ e.targets, x ∈ s.semanticObjects.keys

instance (s : StateBody) : Decidable (AttentionTargetsKnown s) := by
  unfold AttentionTargetsKnown; infer_instance

/-- Unconditionally, attention routes only objects the index already names. -/
theorem attend_subset_indexTargets (s : StateBody) (request : RequestSignature)
    (x : CanonicalObjectId) (h : CanonicalSet.Mem (attend s request) x) :
    x ∈ s.attentionIndex.entries.flatMap (·.targets) := by
  obtain ⟨e, he, _, hx⟩ := (attend_mem_iff s request x).mp h
  exact List.mem_flatMap.mpr ⟨e, he, hx⟩

/-- **The result is a subset of the state's objects.** -/
theorem attend_subset_objects (s : StateBody) (hs : AttentionTargetsKnown s)
    (request : RequestSignature) (x : CanonicalObjectId)
    (h : CanonicalSet.Mem (attend s request) x) : x ∈ s.semanticObjects.keys := by
  obtain ⟨e, he, _, hx⟩ := (attend_mem_iff s request x).mp h
  exact hs e he x hx

/-! ### Monotonicity in the request -/

/-- **Monotonicity.**  A request that demands more applicability tokens routes
at least as much work: attention never *loses* an object when the request is
widened. -/
theorem attend_monotone (s : StateBody) {r₁ r₂ : RequestSignature}
    (h : ∀ t ∈ r₁.tokens, t ∈ r₂.tokens) :
    CanonicalSet.Subset (attend s r₁) (attend s r₂) := by
  intro x hx
  obtain ⟨e, he, happ, hxe⟩ := (attend_mem_iff s r₁ x).mp hx
  exact (attend_mem_iff s r₂ x).mpr
    ⟨e, he, RequestSignature.applies_mono h e.signature happ, hxe⟩

/-- The empty request routes nothing. -/
theorem attend_empty_request (s : StateBody) (x : CanonicalObjectId) :
    ¬ CanonicalSet.Mem (attend s ⟨[]⟩) x := by
  intro hx
  obtain ⟨e, _, happ, _⟩ := (attend_mem_iff s ⟨[]⟩ x).mp hx
  simp [RequestSignature.applies] at happ

/-! ## Anti-vacuity: attention presence is not evidence

The pattern of `Atlas/CoverageScope.lean`: make the gap a machine-checked fact
rather than a remark. -/

/--
**Scope blindness of attention.**

`attend` is a function of exactly two things: the recorded `attentionIndex` and
the request.  Two states that agree on the index route identically — whatever
their candidate facts, cost surfaces, search partitions, lower envelope or
certificate store say.

Consequence, and the reason this lemma is here: no proposition about optimality,
feasibility or cost can be derived from `CanonicalSet.Mem (attend s r) x`.  The
membership records that an indexer wrote `x` under a matching signature, which
is exactly the situation UOR-GNAF §18 lists as an explicit non-claim:

  *that registration, discovery, index presence, or provenance alone proves
  eligibility, correctness, trust, feasibility, cost, or optimality.*

SPEC §12.2 says the same in one line: "An attention score alone is never an
optimality proof."
-/
theorem attend_blind_to_optimizer_state (b : StateBody)
    (cf : CandidateFactMap) (cs : CostSurfaceMap) (pm : PartitionMap)
    (le : LowerEnvelopeBody) (st : CertificateStoreBody) (r : RequestSignature) :
    attend (b.withOptimizerComponents cf cs pm le st) r = attend b r := rfl

/-- The general form: any two states with the same attention index are
indistinguishable to attention. -/
theorem attend_determined_by_index (s₁ s₂ : StateBody) (r : RequestSignature)
    (h : s₁.attentionIndex = s₂.attentionIndex) : attend s₁ r = attend s₂ r :=
  attend_deterministic s₁ s₂ r r h rfl

/-- And, spelled out for a single object: routing `x` says nothing that survives
replacing the whole optimizer half of the state. -/
theorem attend_mem_invariant_under_optimizer (b : StateBody)
    (cf : CandidateFactMap) (cs : CostSurfaceMap) (pm : PartitionMap)
    (le : LowerEnvelopeBody) (st : CertificateStoreBody) (r : RequestSignature)
    (x : CanonicalObjectId) :
    CanonicalSet.Mem (attend (b.withOptimizerComponents cf cs pm le st) r) x ↔
      CanonicalSet.Mem (attend b r) x := by
  rw [attend_blind_to_optimizer_state]

end WasmGemmGnaf.Atlas
