/-
  Constructive enumeration of the raw-input carrier (SPEC §8.3, §8.4, §10.1).

  SPEC §8.4 requires `instance problem_input_fintype : Fintype
  problem.RawInvocation`, and SPEC §15 names it `Gemm.raw_input_finite`.  Until
  now every `Universal` and `Artifact` result carried it as the hypothesis
  `[Foundation.Fintype (Gemm.RawInvocation P)]`.  This file discharges it.

  The enumeration is astronomically long — about `256 ^ (2 ^ 32)` entries — and
  nothing here evaluates it.  It is a structural definition with two structural
  proofs: every lawful `(ptr, len, bytes)` triple occurs, and no triple occurs
  twice.  There is no `decide`, no `native_decide`, no `Classical.choice`, and
  no instance hypothesis: the instance is global and constructive.

  Layering note.  The file lives in the `Universal` layer (SPEC §5) because it
  is the input enumerator SPEC §10.1's `Universal.evaluate` ranges over, and
  because it must see `Universal/Sublevel.lean`'s `byteListsOfLength`.  It
  declares into the `Gemm` namespace, which SPEC §15 fixes for these names.
-/
import WasmGemmGnaf.Universal.Sublevel
import WasmGemmGnaf.Gemm.Reference

set_option autoImplicit false

namespace WasmGemmGnaf.Universal.Enumerate

/-! ## A dependent filter-map

`dfilterMap l q f` keeps exactly those elements of `l` satisfying the decidable
predicate `q` and applies `f`, which may use the proof.  This is the one step
that attaches a `Prop`-valued lawfulness field to a proof-free carrier, and it
is where duplicate-freedom has to be argued from injectivity of `f` rather than
from any decidability of equality on the result. -/

section DFilterMap

variable {α β : Type}

/-- Filter by a decidable predicate and map, with the proof available. -/
def dfilterMap (l : List α) (q : α → Prop) [DecidablePred q] (f : ∀ a, q a → β) :
    List β :=
  l.filterMap (fun a => if h : q a then some (f a h) else none)

theorem mem_dfilterMap {l : List α} {q : α → Prop} [DecidablePred q]
    {f : ∀ a, q a → β} {a : α} (ha : a ∈ l) (hq : q a) : f a hq ∈ dfilterMap l q f :=
  List.mem_filterMap.mpr ⟨a, ha, by rw [dif_pos hq]⟩

/-- Every entry of `dfilterMap` really comes from an entry of `l` satisfying
`q`: the construction adds nothing. -/
theorem exists_of_mem_dfilterMap {l : List α} {q : α → Prop} [DecidablePred q]
    {f : ∀ a, q a → β} {b : β} (hb : b ∈ dfilterMap l q f) :
    ∃ a, a ∈ l ∧ ∃ h : q a, f a h = b := by
  obtain ⟨a, ha, heq⟩ := List.mem_filterMap.mp hb
  refine ⟨a, ha, ?_⟩
  split at heq
  · next hq => exact ⟨hq, Option.some.inj heq⟩
  · nomatch heq

/-- Duplicate-freedom of `dfilterMap`, from duplicate-freedom of `l` and
injectivity of `f`.  No `DecidableEq β` is used: the argument is structural. -/
theorem nodup_dfilterMap {l : List α} {q : α → Prop} [DecidablePred q]
    {f : ∀ a, q a → β} (hl : l.Nodup)
    (hinj : ∀ (a₁ : α) (h₁ : q a₁) (a₂ : α) (h₂ : q a₂), f a₁ h₁ = f a₂ h₂ → a₁ = a₂) :
    (dfilterMap l q f).Nodup := by
  rw [dfilterMap, List.Nodup, List.pairwise_filterMap]
  refine hl.imp ?_
  intro a₁ a₂ hne b₁ hb₁ b₂ hb₂ hb
  refine hne ?_
  split at hb₁
  · next h₁ =>
    split at hb₂
    · next h₂ =>
      exact hinj a₁ h₁ a₂ h₂ (by rw [Option.some.inj hb₁, Option.some.inj hb₂, hb])
    · nomatch hb₂
  · nomatch hb₁

end DFilterMap

/-! ## Duplicate-freedom of the byte enumerations

`allBytes` and `byteListsOfLength` are already defined above with their
membership lemmas; only the `Nodup` half was missing.

`List.nodup_range` in the toolchain is proved through `List.range_eq_range'`
and depends on `Classical.choice`.  The whole point of this file is a
constructive instance, so the fact is reproved here by induction, using only
`range_succ`, `pairwise_append` and `mem_range`. -/

theorem nodup_range : ∀ n : Nat, (List.range n).Nodup
  | 0 => by rw [List.range_zero]; exact List.Pairwise.nil
  | n + 1 => by
      rw [List.range_succ, List.Nodup, List.pairwise_append]
      refine ⟨nodup_range n, List.pairwise_singleton _ _, ?_⟩
      intro a ha b hb
      have hlt : a < n := List.mem_range.mp ha
      have hbn : b = n := List.mem_singleton.mp hb
      omega

theorem allBytes_nodup : allBytes.Nodup := by
  rw [allBytes, List.Nodup, List.pairwise_map]
  refine List.Pairwise.imp_of_mem ?_ (nodup_range 256)
  intro a b ha hb hab heq
  refine hab ?_
  have h := congrArg UInt8.toNat heq
  rwa [UInt8.toNat_ofNat_of_lt' (List.mem_range.mp ha),
    UInt8.toNat_ofNat_of_lt' (List.mem_range.mp hb)] at h

theorem byteListsOfLength_nodup : ∀ n : Nat, (byteListsOfLength n).Nodup
  | 0 => by rw [byteListsOfLength_zero]; exact List.pairwise_singleton _ _
  | n + 1 => by
      rw [byteListsOfLength_succ, List.Nodup, List.pairwise_flatMap]
      refine ⟨?_, ?_⟩
      · intro b _
        rw [List.pairwise_map]
        refine (byteListsOfLength_nodup n).imp ?_
        intro x y hxy hc
        exact hxy (congrArg List.tail hc)
      · refine allBytes_nodup.imp ?_
        intro b₁ b₂ hne x hx y hy hxy
        obtain ⟨l₁, _, rfl⟩ := List.mem_map.mp hx
        obtain ⟨l₂, _, rfl⟩ := List.mem_map.mp hy
        exact hne (Option.some.inj (congrArg List.head? hxy))

end WasmGemmGnaf.Universal.Enumerate

namespace WasmGemmGnaf.Gemm

/-! ## Every 32-bit word -/

/-- Every `UInt32`. -/
def allUInt32 : List UInt32 := (List.range UInt32.size).map UInt32.ofNat

theorem mem_allUInt32 (x : UInt32) : x ∈ allUInt32 :=
  List.mem_map.mpr ⟨x.toNat, List.mem_range.mpr x.toNat_lt_size, UInt32.ofNat_toNat⟩

theorem allUInt32_nodup : allUInt32.Nodup := by
  rw [allUInt32, List.Nodup, List.pairwise_map]
  refine List.Pairwise.imp_of_mem ?_ (Universal.Enumerate.nodup_range UInt32.size)
  intro a b ha hb hab heq
  refine hab ?_
  have h := congrArg UInt32.toNat heq
  rwa [UInt32.toNat_ofNat_of_lt' (List.mem_range.mp ha),
    UInt32.toNat_ofNat_of_lt' (List.mem_range.mp hb)] at h

/-! ## Every byte array of a fixed size -/

/-- Every byte array of exactly size `n`, listed once each. -/
def byteArraysOfSize (n : Nat) : List ByteArray :=
  (Universal.Enumerate.byteListsOfLength n).map Foundation.Bytes.pack

/-- The enumeration is complete: every byte array of size `n` occurs. -/
theorem mem_byteArraysOfSize {n : Nat} {b : ByteArray} (h : b.size = n) :
    b ∈ byteArraysOfSize n := by
  refine List.mem_map.mpr ⟨b.toList, ?_, Foundation.Bytes.pack_toList b⟩
  have hlen : b.toList.length = n := by
    rw [Foundation.Bytes.length_toList, h]
  rw [← hlen]
  exact Universal.Enumerate.mem_byteListsOfLength b.toList

/-- The enumeration is exact: nothing of another size occurs. -/
theorem size_of_mem_byteArraysOfSize {n : Nat} {b : ByteArray}
    (h : b ∈ byteArraysOfSize n) : b.size = n := by
  obtain ⟨l, hl, hb⟩ := List.mem_map.mp h
  subst hb
  rw [Foundation.Bytes.size_pack,
    Universal.Enumerate.length_of_mem_byteListsOfLength n l hl]

theorem mem_byteArraysOfSize_iff {n : Nat} (b : ByteArray) :
    b ∈ byteArraysOfSize n ↔ b.size = n :=
  ⟨size_of_mem_byteArraysOfSize, mem_byteArraysOfSize⟩

theorem byteArraysOfSize_nodup (n : Nat) : (byteArraysOfSize n).Nodup := by
  rw [byteArraysOfSize, List.Nodup, List.pairwise_map]
  refine (Universal.Enumerate.byteListsOfLength_nodup n).imp ?_
  intro x y hxy hc
  exact hxy (Foundation.Bytes.pack_injective hc)

/-! ## The proof-free carrier

`RawInvocationBody` triples with `bytes.size = len.toNat`.  The remaining two
lawfulness clauses depend only on `(ptr, len)` and are imposed in the next
section; they cannot be imposed here without losing the clean product shape the
duplicate-freedom argument needs. -/

/-- All bodies with the given `ptr` and `len`. -/
def bodiesAt (P : Wasm.Profile) (ptr len : UInt32) : List (RawInvocationBody P) :=
  (byteArraysOfSize len.toNat).map (fun b => ⟨ptr, len, b⟩)

/-- All bodies with the given `ptr`. -/
def bodiesWithPtr (P : Wasm.Profile) (ptr : UInt32) : List (RawInvocationBody P) :=
  allUInt32.flatMap (bodiesAt P ptr)

/-- Every body whose byte count matches its declared length. -/
def rawInvocationBodies (P : Wasm.Profile) : List (RawInvocationBody P) :=
  allUInt32.flatMap (bodiesWithPtr P)

variable {P : Wasm.Profile}

theorem ptr_len_of_mem_bodiesAt {ptr len : UInt32} {body : RawInvocationBody P}
    (h : body ∈ bodiesAt P ptr len) : body.ptr = ptr ∧ body.len = len := by
  obtain ⟨b, _, hb⟩ := List.mem_map.mp h
  subst hb
  exact ⟨rfl, rfl⟩

theorem ptr_of_mem_bodiesWithPtr {ptr : UInt32} {body : RawInvocationBody P}
    (h : body ∈ bodiesWithPtr P ptr) : body.ptr = ptr := by
  obtain ⟨_, _, hb⟩ := List.mem_flatMap.mp h
  exact (ptr_len_of_mem_bodiesAt hb).1

theorem mem_bodiesAt {body : RawInvocationBody P}
    (h : body.bytes.size = body.len.toNat) : body ∈ bodiesAt P body.ptr body.len :=
  List.mem_map.mpr ⟨body.bytes, mem_byteArraysOfSize h, rfl⟩

theorem bodiesAt_nodup (P : Wasm.Profile) (ptr len : UInt32) :
    (bodiesAt P ptr len).Nodup := by
  rw [bodiesAt, List.Nodup, List.pairwise_map]
  refine (byteArraysOfSize_nodup len.toNat).imp ?_
  intro x y hxy hc
  exact hxy (congrArg RawInvocationBody.bytes hc)

theorem bodiesWithPtr_nodup (P : Wasm.Profile) (ptr : UInt32) :
    (bodiesWithPtr P ptr).Nodup := by
  rw [bodiesWithPtr, List.Nodup, List.pairwise_flatMap]
  refine ⟨fun len _ => bodiesAt_nodup P ptr len, ?_⟩
  refine allUInt32_nodup.imp ?_
  intro len₁ len₂ hne x hx y hy hxy
  refine hne ?_
  rw [← (ptr_len_of_mem_bodiesAt hx).2, ← (ptr_len_of_mem_bodiesAt hy).2, hxy]

/-- **Completeness of the proof-free carrier.** -/
theorem mem_rawInvocationBodies (body : RawInvocationBody P)
    (h : body.bytes.size = body.len.toNat) : body ∈ rawInvocationBodies P :=
  List.mem_flatMap.mpr ⟨body.ptr, mem_allUInt32 _,
    List.mem_flatMap.mpr ⟨body.len, mem_allUInt32 _, mem_bodiesAt h⟩⟩

theorem rawInvocationBodies_nodup (P : Wasm.Profile) :
    (rawInvocationBodies P).Nodup := by
  rw [rawInvocationBodies, List.Nodup, List.pairwise_flatMap]
  refine ⟨fun ptr _ => bodiesWithPtr_nodup P ptr, ?_⟩
  refine allUInt32_nodup.imp ?_
  intro ptr₁ ptr₂ hne x hx y hy hxy
  refine hne ?_
  rw [← ptr_of_mem_bodiesWithPtr hx, ← ptr_of_mem_bodiesWithPtr hy, hxy]

/-! ## The lawful carrier (SPEC §8.3)

`RawInvocationLawful P body` is exactly

* `body.bytes.size = body.len.toNat`,
* `body.ptr.toNat + body.len.toNat ≤ 2 ^ P.addressBits`,
* `pagesFor (body.ptr.toNat + body.len.toNat) ≤ P.maxPages`.

The first clause is what `rawInvocationBodies` already guarantees; the other
two are decidable conditions on `(ptr, len)` that `dfilterMap` imposes while
attaching the proof. -/

/-- Every lawful `(ptr, len, bytes)` triple, listed once each. -/
def rawInvocations (P : Wasm.Profile) : List (RawInvocation P) :=
  Universal.Enumerate.dfilterMap (rawInvocationBodies P) (RawInvocationLawful P)
    (fun body h => ⟨body, h⟩)

/-- **Completeness.**  Every lawful raw invocation occurs in the enumeration.
`RawInvocationLawful` is a `Prop`, so the proof field is irrelevant to identity
and the entry produced here *is* the given invocation. -/
theorem mem_rawInvocations (raw : RawInvocation P) : raw ∈ rawInvocations P :=
  Universal.Enumerate.mem_dfilterMap
    (mem_rawInvocationBodies raw.body raw.lawful.1) raw.lawful

/-- Every entry really is a lawful raw invocation whose body is enumerated: the
list is exactly the lawful carrier, not a superset. -/
theorem body_mem_of_mem_rawInvocations {raw : RawInvocation P}
    (h : raw ∈ rawInvocations P) : raw.body ∈ rawInvocationBodies P := by
  obtain ⟨body, hbody, _, hf⟩ := Universal.Enumerate.exists_of_mem_dfilterMap h
  rw [← hf]
  exact hbody

/-- **Duplicate-freedom.**  Two raw invocations are equal exactly when their
bodies are, so this is duplicate-freedom of `rawInvocationBodies` transported
along `RawInvocation.body`.  No decidability of equality on `RawInvocation` is
used, and none is available: the lawfulness field is a `Prop`. -/
theorem rawInvocations_nodup (P : Wasm.Profile) : (rawInvocations P).Nodup :=
  Universal.Enumerate.nodup_dfilterMap (rawInvocationBodies_nodup P)
    (fun _ _ _ _ heq => congrArg RawInvocation.body heq)

/--
  **SPEC §15 / SPEC §8.4 `problem_input_fintype`.**  The raw-input carrier is
  finite, constructively.

  This is a global instance with no instance hypothesis and no appeal to
  choice: `rawInvocations P` is a closed computable list, `mem_rawInvocations`
  its coverage proof and `rawInvocations_nodup` its duplicate-freedom proof.
  The list is not meant to be evaluated — it has about `256 ^ (2 ^ 32)` entries
  — and nothing in the development evaluates it.
-/
instance raw_input_finite (P : Wasm.Profile) :
    Foundation.Fintype (RawInvocation P) where
  elemsList := rawInvocations P
  complete := mem_rawInvocations
  nodupList := rawInvocations_nodup P

/-- The instance's enumeration covers the carrier.  Stated through
`Foundation.Fintype.elems` rather than by unfolding it: the enumeration is
`List.range UInt32.size`-shaped, so no proof here may force it to reduce. -/
theorem mem_elems_rawInvocation (P : Wasm.Profile) (raw : RawInvocation P) :
    raw ∈ Foundation.Fintype.elems (RawInvocation P) :=
  Foundation.Fintype.mem_elems raw

theorem elems_rawInvocation_nodup (P : Wasm.Profile) :
    (Foundation.Fintype.elems (RawInvocation P)).Nodup :=
  Foundation.Fintype.elems_nodup _

/-! ## The valid-input carrier (SPEC §8.3) -/

/-- The valid-input carrier: a raw invocation the classifier accepts.
`refValid_iff_classify` proves the Boolean is exactly `classify`'s `valid`
verdict, so this subtype is the set of inputs SPEC §8.3 calls valid. -/
abbrev ValidRawInvocation (P : Wasm.Profile) : Type :=
  { raw : RawInvocation P // refValid raw.body = true }

/-- The classifier's own verdict on a member of the valid-input carrier. -/
theorem validRawInvocation_classify (v : ValidRawInvocation P) :
    ∃ inv, classify P v.val.body = .valid inv :=
  refValid_classify v.property

/-- Every valid input, listed once each. -/
def validRawInvocations (P : Wasm.Profile) : List (ValidRawInvocation P) :=
  Universal.Enumerate.dfilterMap (rawInvocations P)
    (fun raw => refValid raw.body = true) (fun raw h => ⟨raw, h⟩)

theorem mem_validRawInvocations (v : ValidRawInvocation P) :
    v ∈ validRawInvocations P :=
  Universal.Enumerate.mem_dfilterMap (mem_rawInvocations v.val) v.property

theorem validRawInvocations_nodup (P : Wasm.Profile) :
    (validRawInvocations P).Nodup :=
  Universal.Enumerate.nodup_dfilterMap (rawInvocations_nodup P)
    (fun _ _ _ _ heq => congrArg Subtype.val heq)

/--
  **SPEC §15**, `Gemm.valid_input_finite`.  The valid-input carrier — the raw
  invocations the total classifier of SPEC §8.3 accepts — is finite, with no
  instance hypothesis and no `Fintype` argument: the enumeration, its coverage
  proof and its duplicate-freedom proof are all supplied here.

  Axiom note.  Unlike `raw_input_finite`, whose closure is `[propext,
  Quot.sound]`, this instance's closure also contains `Classical.choice.`  The
  filter is the classifier's own verdict, and `Gemm.classify` — hence
  `Gemm.refValid` — already depends on `Classical.choice` in committed code: the
  `Decidable (View.LayoutOk …)` instance embeds `View.checkInterval_iff`, which
  goes through `Shape.isEmpty_eq_false_iff`, proved by `omega` on an `Iff` goal.
  `omega` discharges `Iff`, `And` and `Or` goals classically.  No enumeration of
  the *valid* carrier can be choice-free while `Gemm.classify` is not; nothing
  is added here.
-/
instance valid_input_finite (P : Wasm.Profile) :
    Foundation.Fintype (ValidRawInvocation P) where
  elemsList := validRawInvocations P
  complete := mem_validRawInvocations
  nodupList := validRawInvocations_nodup P

/-! ### The valid-input enumeration through the total classifier

`validRawInvocations` above carries the lawfulness proof in its element type,
which is what the `Fintype` instance needs.  The audit asked in addition for the
plain list of *raw invocations the classifier accepts*, characterised by
`Gemm.classify` itself rather than by the Boolean `refValid`, in both
directions.  That list is the image of `validRawInvocations` under the subtype
projection, and `refValid_iff_classify` is what turns the Boolean filter into a
statement about the classifier's verdict. -/

/-- **The valid-input enumeration (SPEC §8.3).**  `rawInvocations P` restricted
to exactly the entries on which the total classifier returns `.valid`. -/
def validInvocations (P : Wasm.Profile) : List (RawInvocation P) :=
  (validRawInvocations P).map Subtype.val

/-- **Exactness of the valid-input enumeration, both directions.**  A raw
invocation occurs in `validInvocations P` if and only if the total classifier
`Gemm.classify` of SPEC §8.3 returns `.valid` on it.  Left to right the list is
sound (nothing the classifier rejects is listed); right to left it is complete
(no accepted input is missed). -/
theorem mem_validInvocations (raw : RawInvocation P) :
    raw ∈ validInvocations P ↔ ∃ inv, classify P raw.body = .valid inv := by
  constructor
  · intro h
    obtain ⟨v, _, hv⟩ := List.mem_map.mp h
    subst hv
    exact refValid_classify v.property
  · intro h
    obtain ⟨inv, hinv⟩ := h
    exact List.mem_map.mpr
      ⟨⟨raw, refValid_of_classify hinv⟩,
        mem_validRawInvocations ⟨raw, refValid_of_classify hinv⟩, rfl⟩

/-- **Duplicate-freedom of the valid-input enumeration.**  The subtype
projection is injective because the lawfulness field is a `Prop`, so this is
duplicate-freedom of `validRawInvocations` transported along it. -/
theorem validInvocations_nodup (P : Wasm.Profile) : (validInvocations P).Nodup := by
  rw [validInvocations, List.Nodup, List.pairwise_map]
  exact (validRawInvocations_nodup P).imp (fun h hc => h (Subtype.ext hc))

/-- Every listed valid input is a raw input: the valid enumeration is a
sublist-image of the raw one, not a separate universe. -/
theorem mem_rawInvocations_of_mem_validInvocations {raw : RawInvocation P}
    (_h : raw ∈ validInvocations P) : raw ∈ rawInvocations P :=
  mem_rawInvocations raw

/-- The `valid_input_finite` enumeration covers the valid-input carrier.  Stated
through `Foundation.Fintype.elems`, never by unfolding it: the enumeration is
`List.range UInt32.size`-shaped and no proof here may force it to reduce. -/
theorem mem_elems_validRawInvocation (P : Wasm.Profile) (v : ValidRawInvocation P) :
    v ∈ Foundation.Fintype.elems (ValidRawInvocation P) :=
  Foundation.Fintype.mem_elems v

theorem elems_validRawInvocation_nodup (P : Wasm.Profile) :
    (Foundation.Fintype.elems (ValidRawInvocation P)).Nodup :=
  Foundation.Fintype.elems_nodup _

end WasmGemmGnaf.Gemm

namespace WasmGemmGnaf.Universal

/-! ## The input enumerator (SPEC §10.1)

SPEC §10.1's `Universal.evaluate` ranges over `∀ raw : G.RawInvocation`.  The
enumerator it ranges over is the one below, and it is complete. -/

/-- The input enumerator SPEC §10.1 ranges over. -/
def inputEnumerator (P : Wasm.Profile) : List (Gemm.RawInvocation P) :=
  Gemm.rawInvocations P

theorem mem_inputEnumerator {P : Wasm.Profile} (raw : Gemm.RawInvocation P) :
    raw ∈ inputEnumerator P :=
  Gemm.mem_rawInvocations raw

/-- **SPEC §10.1's executable input enumerator.**  `Universal.evaluate P G bytes`
quantifies over `∀ raw : G.RawInvocation`; this is the list that quantifier
ranges over.  `Gemm.Problem.RawInvocation` is by definition `Gemm.RawInvocation
P`, so the problem contributes nothing to the carrier and the enumerator is a
plain total function of the profile — no choice, nothing noncomputable, no
`Fintype` hypothesis. -/
def enumerateInputs (P : Wasm.Profile) (_G : Gemm.Problem P) :
    List (Gemm.RawInvocation P) :=
  Gemm.rawInvocations P

theorem enumerateInputs_eq_inputEnumerator (P : Wasm.Profile) (G : Gemm.Problem P) :
    enumerateInputs P G = inputEnumerator P := rfl

/--
  **SPEC §15**, `Universal.input_enumerator_complete`.  The input enumerator
  misses no input: every raw invocation of the problem occurs in it.

  This is unconditional — no `Fintype` hypothesis, no sublevel bound, no
  scope predicate.  Together with `enumerateInputs_nodup` it says the
  enumerator is exactly the carrier.
-/
theorem input_enumerator_complete {P : Wasm.Profile} (G : Gemm.Problem P)
    (raw : Gemm.RawInvocation P) : raw ∈ enumerateInputs P G :=
  Gemm.mem_rawInvocations raw

theorem enumerateInputs_nodup (P : Wasm.Profile) (G : Gemm.Problem P) :
    (enumerateInputs P G).Nodup :=
  Gemm.rawInvocations_nodup P

theorem inputEnumerator_nodup (P : Wasm.Profile) : (inputEnumerator P).Nodup :=
  Gemm.rawInvocations_nodup P

/-! ### The valid-input enumerator

The same enumerator restricted to the inputs the total classifier of SPEC §8.3
accepts, with its exactness carried over from `Gemm.mem_validInvocations`. -/

/-- The valid-input enumerator: `enumerateInputs` keeping exactly the classifier's
`.valid` results. -/
def enumerateValidInputs (P : Wasm.Profile) (_G : Gemm.Problem P) :
    List (Gemm.RawInvocation P) :=
  Gemm.validInvocations P

theorem mem_enumerateValidInputs {P : Wasm.Profile} (G : Gemm.Problem P)
    (raw : Gemm.RawInvocation P) :
    raw ∈ enumerateValidInputs P G ↔ ∃ inv, Gemm.classify P raw.body = .valid inv :=
  Gemm.mem_validInvocations raw

theorem enumerateValidInputs_nodup (P : Wasm.Profile) (G : Gemm.Problem P) :
    (enumerateValidInputs P G).Nodup :=
  Gemm.validInvocations_nodup P

/-- The `Fintype` enumeration of the carrier is complete for the same reason:
`Gemm.raw_input_finite` is built from `inputEnumerator` itself. -/
theorem fintype_input_enumerator_complete {P : Wasm.Profile}
    (raw : Gemm.RawInvocation P) :
    raw ∈ Foundation.Fintype.elems (Gemm.RawInvocation P) :=
  Foundation.Fintype.mem_elems raw

end WasmGemmGnaf.Universal
