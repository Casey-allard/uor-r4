/-
  Wasm/CostedExplore.lean --- the costed all-branch explorer.

  Normative source: SPEC.md sections 7.4, 7.5 and 10.1.  This file closes item 3
  of the seam documented at the head of `Artifact/Release.lean`: SPEC §10.1's
  `Wasm.initialGemmInvocationCosted` and `Wasm.exploreAllCosted`, i.e. the two
  fields of `Universal.CostedMachine`.

  `Wasm/Fuel.lean` supplies `Wasm.exploreAll`, whose `complete` constructor
  carries a plain, unordered, possibly empty `List Wasm.ExecutionObservation`.
  `Universal.CostedTreeResult.complete` demands a
  `Foundation.NonemptyCanonicalFrontier (Universal.CostedExecutionObservation …)`
  — a *nonempty*, *canonically sorted* frontier over a proof-carrying type.
  Bridging the two needs three things, and all three are built here:

  * a `Foundation.CanonicalSchema` for the costed observation type, with a
    prefix-free injective encoder assembled from `Foundation/Bytes.lean`'s
    combinators (`costedObservationSchema_encode_injective`);
  * a canonical insertion sort over that order, proved to produce a
    `Foundation.CanonicalSorted` output that is a permutation of its input
    (`costedObservations_sorted`, `costedObservations_perm`);
  * a transport of `Wasm.covers_exploreTree` (hence of
    `Wasm.runFuel_sound` and `Wasm.runFuel_complete_with_bound`) through the
    sort, giving `Universal.CostedCoverage`.

  ## ANTI-VACUITY

  The decisive theorem is `Wasm.exploreAllCosted_complete_of_nonempty`: whenever
  the plain explorer returns `.complete` with a nonempty observation list, the
  costed explorer returns `.complete`.  Nothing here answers
  `.initializationFailure` unconditionally: `exploreAllCosted_initializationFailure_iff`
  proves that constructor is reached *exactly* when
  `Wasm.exploreTree (bound + 1) [] initial = []`, which happens only for a
  configuration every one of whose branches gets stuck while still `running`
  (`Wasm.successors` empty on a running configuration).  Similarly
  `initialGemmInvocationCosted` returns `.error` exactly when
  `Wasm.initialConfig` reports an `InstantiationFault`, and the fault is
  recoverable from the report (`initializationFailureReport_injective`).

  ## TWO DISCLOSED DEVIATIONS

  1. SPEC §10.1's display of `exploreAllCosted_erases` concludes

         Wasm.exploreAll bound initial =
           .complete (observations.map (·.observation)) coverage.erase

     with *list equality*.  That is unattainable together with
     `NonemptyCanonicalFrontier.ordered`: the frontier is canonically sorted by
     its schema encoding, while `Wasm.exploreTree` emits its observations in
     depth-first successor order, and the two orders differ in general.  What is
     proved here is the strongest true form: `Wasm.exploreAllCosted_erases`
     gives the *same* `.complete` constructor of `Wasm.exploreAll`, with the
     *same* observation multiset — `List.Perm` in place of `=`.  Coverage
     itself is unaffected: `CoversEveryMaximalFiniteBranch` and
     `Universal.CostedCoverage` are both membership statements, and membership
     is permutation invariant.

  2. `Universal.CostedTreeResult` offers exactly three constructors, and the
     empty-frontier case of a completed exploration fits none of them.  It is
     reported as `.initializationFailure .allocationFailed`; the fault tag is
     the least-wrong of the three available and is *not* a claim that
     instantiation failed.  `exploreAllCosted_initializationFailure_iff` pins
     down exactly when this arises, so the fallback is characterised rather
     than hidden.

  Every declaration in this file is a definition or a proved theorem.  There is
  no `sorry`, no `admit`, no project axiom, no `native_decide`, no `unsafe` and
  no `partial`.
-/
import WasmGemmGnaf.Universal.Competitor
import WasmGemmGnaf.Wasm.Costed

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf.Foundation

/-! ## Structural type tags for this layer -/

/-- Structural leaf type tag: a numeric structural index followed by a name.
The index makes tags provably distinct without kernel evaluation of string
literals. -/
def costedExploreTag (index : Nat) (name : String) : ByteArray :=
  TypeTag.leaf (Bytes.natBytes index ++ name.toUTF8.toList)

theorem costedExploreTag_size_pos (index : Nat) (name : String) :
    (costedExploreTag index name).size > 0 :=
  TypeTag.leaf_size_pos _

theorem costedExploreTag_ne {i j : Nat} (n m : String) (h : i ≠ j) :
    costedExploreTag i n ≠ costedExploreTag j m := by
  intro heq
  have h1 : Bytes.natBytes i ++ n.toUTF8.toList = Bytes.natBytes j ++ m.toUTF8.toList :=
    TypeTag.leaf_injective heq
  exact h (Bytes.natBytes_prefixFree _ _ _ _ h1).1

/-! ## Canonical encoding of execution observations (SPEC §6.2)

The encoders below are built exclusively from `Foundation/Bytes.lean`'s
combinators, so prefix-freeness — and hence injectivity — is compositional.
Injectivity of each *tuple* projection is proved by exhibiting a decoder and a
round-trip, which is robust against later constructor additions: a new
constructor without a tuple row breaks the round-trip. -/

/-- Structural index of a trap. -/
def trapIndex : Trap → Nat
  | .unreachable => 0
  | .outOfBounds => 1
  | .divideByZero => 2

/-- Structural index of a permitted `memory.grow` outcome. -/
def growOutcomeIndex : Num.GrowOutcome → Nat
  | .refused => 0
  | .grown previousPages => previousPages + 1

/-- The first-order tuple of a reduction event. -/
def eventTuple : Event → Nat × Nat × Nat × Nat × UInt32 × UInt32
  | .step => (0, 0, 0, 0, 0, 0)
  | .branch => (1, 0, 0, 0, 0, 0)
  | .growAttempt outcome => (2, growOutcomeIndex outcome, 0, 0, 0, 0)
  | .enterGemm => (3, 0, 0, 0, 0, 0)
  | .trapEvent trap => (4, 0, trapIndex trap, 0, 0, 0)
  | .throwEvent exceptionValue =>
      (5, 0, 0, exceptionValue.tag, exceptionValue.payload, 0)
  | .returnEvent value => (6, 0, 0, 0, 0, value)

/-- The decoder that witnesses injectivity of `eventTuple`. -/
def eventOfTuple : Nat × Nat × Nat × Nat × UInt32 × UInt32 → Option Event
  | (0, _, _, _, _, _) => some .step
  | (1, _, _, _, _, _) => some .branch
  | (2, 0, _, _, _, _) => some (.growAttempt .refused)
  | (2, n + 1, _, _, _, _) => some (.growAttempt (.grown n))
  | (3, _, _, _, _, _) => some .enterGemm
  | (4, _, 0, _, _, _) => some (.trapEvent .unreachable)
  | (4, _, 1, _, _, _) => some (.trapEvent .outOfBounds)
  | (4, _, 2, _, _, _) => some (.trapEvent .divideByZero)
  | (5, _, _, tag, payload, _) => some (.throwEvent ⟨tag, payload⟩)
  | (6, _, _, _, _, value) => some (.returnEvent value)
  | _ => none

theorem eventOfTuple_eventTuple (e : Event) : eventOfTuple (eventTuple e) = some e := by
  cases e with
  | growAttempt outcome => cases outcome <;> rfl
  | trapEvent trap => cases trap <;> rfl
  | step => rfl
  | branch => rfl
  | enterGemm => rfl
  | throwEvent exceptionValue => rfl
  | returnEvent value => rfl

theorem eventTuple_injective : Function.Injective eventTuple := by
  intro a b h
  have ha := eventOfTuple_eventTuple a
  rw [h, eventOfTuple_eventTuple b] at ha
  exact (Option.some.inj ha).symm

/-- The canonical prefix-free encoding of a reduction event. -/
def eventBytes (e : Event) : List UInt8 :=
  Bytes.pairBytes Bytes.natBytes
    (Bytes.pairBytes Bytes.natBytes
      (Bytes.pairBytes Bytes.natBytes
        (Bytes.pairBytes Bytes.natBytes
          (Bytes.pairBytes Bytes.u32Bytes Bytes.u32Bytes)))) (eventTuple e)

theorem eventBytes_prefixFree : Bytes.PrefixFree eventBytes :=
  (Bytes.pairBytes_prefixFree Bytes.natBytes_prefixFree
    (Bytes.pairBytes_prefixFree Bytes.natBytes_prefixFree
      (Bytes.pairBytes_prefixFree Bytes.natBytes_prefixFree
        (Bytes.pairBytes_prefixFree Bytes.natBytes_prefixFree
          (Bytes.pairBytes_prefixFree Bytes.u32Bytes_prefixFree
            Bytes.u32Bytes_prefixFree))))).comp eventTuple_injective

theorem eventBytes_injective : Function.Injective eventBytes :=
  eventBytes_prefixFree.injective

/-- The first-order tuple of an execution observation: constructor index,
trace, entry-boundary store snapshot, final ABI-visible store, and the two
terminal-status payload words.  `ObservableEffects` carries no field in the
closed release profile (`Wasm.ObservableEffects.eq_none`), so it contributes
nothing to the tuple and nothing to the encoding. -/
def observationTuple :
    ExecutionObservation → Nat × List Event × Option (List UInt8) × List UInt8 × Nat × UInt32
  | .returned trace entry value final _ =>
      (0, trace, some entry.bytes, final.bytes, 0, value)
  | .trappedBeforeEntry trace trap final _ =>
      (1, trace, none, final.bytes, trapIndex trap, 0)
  | .trappedAfterEntry trace entry trap final _ =>
      (2, trace, some entry.bytes, final.bytes, trapIndex trap, 0)
  | .thrownBeforeEntry trace exceptionValue final _ =>
      (3, trace, none, final.bytes, exceptionValue.tag, exceptionValue.payload)
  | .thrownAfterEntry trace entry exceptionValue final _ =>
      (4, trace, some entry.bytes, final.bytes, exceptionValue.tag,
        exceptionValue.payload)

/-- The decoder that witnesses injectivity of `observationTuple`. -/
def observationOfTuple :
    Nat × List Event × Option (List UInt8) × List UInt8 × Nat × UInt32 →
      Option ExecutionObservation
  | (0, trace, some entry, final, _, value) =>
      some (.returned trace ⟨entry⟩ value ⟨final⟩ {})
  | (1, trace, none, final, 0, _) =>
      some (.trappedBeforeEntry trace .unreachable ⟨final⟩ {})
  | (1, trace, none, final, 1, _) =>
      some (.trappedBeforeEntry trace .outOfBounds ⟨final⟩ {})
  | (1, trace, none, final, 2, _) =>
      some (.trappedBeforeEntry trace .divideByZero ⟨final⟩ {})
  | (2, trace, some entry, final, 0, _) =>
      some (.trappedAfterEntry trace ⟨entry⟩ .unreachable ⟨final⟩ {})
  | (2, trace, some entry, final, 1, _) =>
      some (.trappedAfterEntry trace ⟨entry⟩ .outOfBounds ⟨final⟩ {})
  | (2, trace, some entry, final, 2, _) =>
      some (.trappedAfterEntry trace ⟨entry⟩ .divideByZero ⟨final⟩ {})
  | (3, trace, none, final, tag, payload) =>
      some (.thrownBeforeEntry trace ⟨tag, payload⟩ ⟨final⟩ {})
  | (4, trace, some entry, final, tag, payload) =>
      some (.thrownAfterEntry trace ⟨entry⟩ ⟨tag, payload⟩ ⟨final⟩ {})
  | _ => none

theorem observationOfTuple_observationTuple (o : ExecutionObservation) :
    observationOfTuple (observationTuple o) = some o := by
  cases o with
  | returned trace entry value final effects => rfl
  | trappedBeforeEntry trace trap final effects => cases trap <;> rfl
  | trappedAfterEntry trace entry trap final effects => cases trap <;> rfl
  | thrownBeforeEntry trace exceptionValue final effects => rfl
  | thrownAfterEntry trace entry exceptionValue final effects => rfl

theorem observationTuple_injective : Function.Injective observationTuple := by
  intro a b h
  have ha := observationOfTuple_observationTuple a
  rw [h, observationOfTuple_observationTuple b] at ha
  exact (Option.some.inj ha).symm

/-- The canonical prefix-free encoding of an execution observation: the whole
observation, with nothing projected away. -/
def observationBytes (o : ExecutionObservation) : List UInt8 :=
  Bytes.pairBytes Bytes.natBytes
    (Bytes.pairBytes (Bytes.listBytes eventBytes)
      (Bytes.pairBytes (Bytes.optionBytes Bytes.stringBytes)
        (Bytes.pairBytes Bytes.stringBytes
          (Bytes.pairBytes Bytes.natBytes Bytes.u32Bytes)))) (observationTuple o)

theorem observationBytes_prefixFree : Bytes.PrefixFree observationBytes :=
  (Bytes.pairBytes_prefixFree Bytes.natBytes_prefixFree
    (Bytes.pairBytes_prefixFree (Bytes.listBytes_prefixFree eventBytes_prefixFree)
      (Bytes.pairBytes_prefixFree
        (Bytes.optionBytes_prefixFree Bytes.stringBytes_prefixFree)
        (Bytes.pairBytes_prefixFree Bytes.stringBytes_prefixFree
          (Bytes.pairBytes_prefixFree Bytes.natBytes_prefixFree
            Bytes.u32Bytes_prefixFree))))).comp observationTuple_injective

theorem observationBytes_injective : Function.Injective observationBytes :=
  observationBytes_prefixFree.injective

/-- The frozen canonical schema of a plain execution observation. -/
def observationSchema : CanonicalSchema ExecutionObservation :=
  CanonicalSchema.ofPrefixFree 1 CanonicalDomainTag.generic
    (costedExploreTag 1 "Wasm.ExecutionObservation")
    (costedExploreTag_size_pos 1 _) observationBytes observationBytes_prefixFree

theorem observationSchema_encode_injective :
    Function.Injective observationSchema.encode :=
  observationSchema.encode_injective

/-! ## The costed observation type is determined by its observation

`Universal.CostedExecutionObservation` has four fields: the observation, a
proof that it is a real finite execution, the cost, and a proof that the cost is
*exactly* the trace cost.  The two proof fields are propositions, and the cost
field is pinned by the fourth, so the whole record is determined by its
observation.  That is what makes a schema on the observation alone injective on
the costed type — no data is discarded. -/

theorem costedExecutionObservation_eq_of_observation {P : Profile}
    {W : Universal.Semantics P} {initial : Config}
    {a b : Universal.CostedExecutionObservation W initial}
    (h : a.observation = b.observation) : a = b := by
  obtain ⟨oa, ea, ca, xa⟩ := a
  obtain ⟨ob, eb, cb, xb⟩ := b
  simp only at h
  subst h
  subst xa
  subst xb
  rfl

theorem costedExecutionObservation_observation_injective {P : Profile}
    (W : Universal.Semantics P) (initial : Config) :
    Function.Injective
      (fun c : Universal.CostedExecutionObservation W initial => c.observation) :=
  fun _ _ h => costedExecutionObservation_eq_of_observation h

/-- The canonical prefix-free encoding of a costed execution observation. -/
def costedObservationBytes {P : Profile} (W : Universal.Semantics P)
    (initial : Config) (c : Universal.CostedExecutionObservation W initial) :
    List UInt8 :=
  observationBytes c.observation

theorem costedObservationBytes_prefixFree {P : Profile}
    (W : Universal.Semantics P) (initial : Config) :
    Bytes.PrefixFree (costedObservationBytes W initial) :=
  observationBytes_prefixFree.comp
    (costedExecutionObservation_observation_injective W initial)

/-- **The canonical schema of a costed execution observation.** -/
def costedObservationSchema {P : Profile} (W : Universal.Semantics P)
    (initial : Config) :
    CanonicalSchema (Universal.CostedExecutionObservation W initial) :=
  CanonicalSchema.ofPrefixFree 1 CanonicalDomainTag.generic
    (costedExploreTag 2 "Universal.CostedExecutionObservation")
    (costedExploreTag_size_pos 2 _) (costedObservationBytes W initial)
    (costedObservationBytes_prefixFree W initial)

/-- **The schema's encoder is injective**, as SPEC §6.2 requires. -/
theorem costedObservationSchema_encode_injective {P : Profile}
    (W : Universal.Semantics P) (initial : Config) :
    Function.Injective (costedObservationSchema W initial).encode :=
  (costedObservationSchema W initial).encode_injective

@[simp] theorem costedObservationSchema_encode {P : Profile}
    (W : Universal.Semantics P) (initial : Config)
    (c : Universal.CostedExecutionObservation W initial) :
    (costedObservationSchema W initial).encode c =
      Bytes.pack (observationBytes c.observation) := rfl

/-! ## The canonical sort (SPEC §6.2)

An insertion sort under `Foundation.CanonicalBytesLE` on the schema encoding.
Totality and transitivity of the order are already proved in
`Foundation/Order.lean`; only the two list-level obligations are discharged
here. -/

section Sorting

variable {α : Type}

/-- Insert `a` into a canonically ordered list. -/
def canonicalInsert (schema : CanonicalSchema α) (a : α) : List α → List α
  | [] => [a]
  | b :: rest =>
      if CanonicalBytesLE (schema.encode a) (schema.encode b) then a :: b :: rest
      else b :: canonicalInsert schema a rest

/-- The canonical insertion sort. -/
def canonicalSort (schema : CanonicalSchema α) : List α → List α
  | [] => []
  | a :: rest => canonicalInsert schema a (canonicalSort schema rest)

theorem canonicalSorted_map_tail {schema : CanonicalSchema α} {b : α}
    {rest : List α} (h : CanonicalSorted ((b :: rest).map schema.encode)) :
    CanonicalSorted (rest.map schema.encode) := by
  cases rest with
  | nil => simp
  | cons c t =>
    simp only [List.map_cons] at h ⊢
    exact (canonicalSorted_cons_cons.mp h).2

theorem canonicalSorted_map_cons {schema : CanonicalSchema α} (b : α) :
    ∀ l : List α, CanonicalSorted (l.map schema.encode) →
      (∀ (x : α) (t : List α), l = x :: t →
        CanonicalBytesLE (schema.encode b) (schema.encode x)) →
      CanonicalSorted ((b :: l).map schema.encode)
  | [], _, _ => by simp
  | c :: t, hs, hh => by
    simp only [List.map_cons] at hs ⊢
    exact canonicalSorted_cons_cons.mpr ⟨hh c t rfl, hs⟩

theorem canonicalInsert_head (schema : CanonicalSchema α) (a : α) :
    ∀ (l : List α) (x : α) (t : List α),
      canonicalInsert schema a l = x :: t → x = a ∨ ∃ r, l = x :: r
  | [], x, t, h => by
    unfold canonicalInsert at h
    simp only [List.cons.injEq] at h
    exact Or.inl h.1.symm
  | b :: rest, x, t, h => by
    unfold canonicalInsert at h
    split at h
    · simp only [List.cons.injEq] at h
      exact Or.inl h.1.symm
    · simp only [List.cons.injEq] at h
      exact Or.inr ⟨rest, by rw [h.1]⟩

theorem canonicalInsert_sorted (schema : CanonicalSchema α) (a : α) :
    ∀ l : List α, CanonicalSorted (l.map schema.encode) →
      CanonicalSorted ((canonicalInsert schema a l).map schema.encode)
  | [], _ => by simp [canonicalInsert]
  | b :: rest, hs => by
    unfold canonicalInsert
    split
    · rename_i hle
      simp only [List.map_cons] at hs ⊢
      exact canonicalSorted_cons_cons.mpr ⟨hle, hs⟩
    · rename_i hgt
      have hba : CanonicalBytesLE (schema.encode b) (schema.encode a) :=
        (CanonicalBytesLE.total (schema.encode a) (schema.encode b)).resolve_left hgt
      refine canonicalSorted_map_cons b _
        (canonicalInsert_sorted schema a rest (canonicalSorted_map_tail hs)) ?_
      intro x t hxt
      rcases canonicalInsert_head schema a rest x t hxt with hxa | ⟨r, hr⟩
      · rw [hxa]; exact hba
      · rw [hr] at hs
        simp only [List.map_cons] at hs
        exact (canonicalSorted_cons_cons.mp hs).1

theorem canonicalInsert_perm (schema : CanonicalSchema α) (a : α) :
    ∀ l : List α, (canonicalInsert schema a l).Perm (a :: l)
  | [] => List.Perm.refl _
  | b :: rest => by
    unfold canonicalInsert
    split
    · exact List.Perm.refl _
    · exact List.Perm.trans (List.Perm.cons b (canonicalInsert_perm schema a rest))
        (List.Perm.swap a b rest)

/-- **The canonical sort is canonically ordered.** -/
theorem canonicalSort_sorted (schema : CanonicalSchema α) :
    ∀ l : List α, CanonicalSorted ((canonicalSort schema l).map schema.encode)
  | [] => by simp [canonicalSort]
  | a :: rest => by
    unfold canonicalSort
    exact canonicalInsert_sorted schema a _ (canonicalSort_sorted schema rest)

/-- **The canonical sort is a permutation of its input.** -/
theorem canonicalSort_perm (schema : CanonicalSchema α) :
    ∀ l : List α, (canonicalSort schema l).Perm l
  | [] => List.Perm.refl _
  | a :: rest => by
    unfold canonicalSort
    exact List.Perm.trans (canonicalInsert_perm schema a (canonicalSort schema rest))
      (List.Perm.cons a (canonicalSort_perm schema rest))

theorem mem_canonicalSort {schema : CanonicalSchema α} {l : List α} {a : α} :
    a ∈ canonicalSort schema l ↔ a ∈ l :=
  (canonicalSort_perm schema l).mem_iff

theorem length_canonicalSort (schema : CanonicalSchema α) (l : List α) :
    (canonicalSort schema l).length = l.length :=
  (canonicalSort_perm schema l).length_eq

/-- A nonempty canonically ordered list is a `NonemptyCanonicalFrontier`. -/
def frontierOfSortedCons (schema : CanonicalSchema α) (a : α) (rest : List α)
    (hs : CanonicalSorted ((a :: rest).map schema.encode)) :
    NonemptyCanonicalFrontier α where
  schema := schema
  head := a
  rest := rest
  ordered := hs

@[simp] theorem elements_frontierOfSortedCons (schema : CanonicalSchema α) (a : α)
    (rest : List α) (hs : CanonicalSorted ((a :: rest).map schema.encode)) :
    (frontierOfSortedCons schema a rest hs).elements = a :: rest := rfl

end Sorting

/-! ## Lifting plain observations to costed observations -/

/-- The costed observation of a real finite execution.  The cost is *exactly*
the trace cost, by construction: no freedom is left. -/
def costedOfObservation {P : Profile} (W : Universal.Semantics P)
    {initial : Config} (o : ExecutionObservation)
    (h : FiniteExecution initial o) :
    Universal.CostedExecutionObservation W initial where
  observation := o
  execution := h
  cost := Cost.traceCost (o.trace.map W.costEvent)
  costExact := rfl

@[simp] theorem costedOfObservation_observation {P : Profile}
    (W : Universal.Semantics P) {initial : Config} (o : ExecutionObservation)
    (h : FiniteExecution initial o) :
    (costedOfObservation W o h).observation = o := rfl

/-- Lift a list of observations, each proved to be a real finite execution. -/
def costedList {P : Profile} (W : Universal.Semantics P) (initial : Config) :
    (obs : List ExecutionObservation) →
      (∀ o ∈ obs, FiniteExecution initial o) →
      List (Universal.CostedExecutionObservation W initial)
  | [], _ => []
  | o :: rest, h =>
      costedOfObservation W o (h o List.mem_cons_self) ::
        costedList W initial rest (fun x hx => h x (List.mem_cons_of_mem o hx))

/-- Lifting discards nothing: the observations come back exactly. -/
theorem costedList_map_observation {P : Profile} (W : Universal.Semantics P)
    (initial : Config) :
    ∀ (obs : List ExecutionObservation) (h : ∀ o ∈ obs, FiniteExecution initial o),
      (costedList W initial obs h).map (·.observation) = obs
  | [], _ => rfl
  | o :: rest, h => by
    simp only [costedList, List.map_cons, costedOfObservation_observation,
      List.cons.injEq, true_and]
    exact costedList_map_observation W initial rest _

/-- **The canonically ordered costed observations of a list of observations.** -/
def costedObservations {P : Profile} (W : Universal.Semantics P)
    (initial : Config) (obs : List ExecutionObservation)
    (h : ∀ o ∈ obs, FiniteExecution initial o) :
    List (Universal.CostedExecutionObservation W initial) :=
  canonicalSort (costedObservationSchema W initial) (costedList W initial obs h)

/-- **The output is canonically ordered.** -/
theorem costedObservations_sorted {P : Profile} (W : Universal.Semantics P)
    (initial : Config) (obs : List ExecutionObservation)
    (h : ∀ o ∈ obs, FiniteExecution initial o) :
    CanonicalSorted
      ((costedObservations W initial obs h).map
        (costedObservationSchema W initial).encode) :=
  canonicalSort_sorted _ _

/-- **The output is a permutation of the input.** -/
theorem costedObservations_perm {P : Profile} (W : Universal.Semantics P)
    (initial : Config) (obs : List ExecutionObservation)
    (h : ∀ o ∈ obs, FiniteExecution initial o) :
    (costedObservations W initial obs h).Perm (costedList W initial obs h) :=
  canonicalSort_perm _ _

/-- The observations of the sorted costed list are a permutation of the input
observations: sorting reorders, and does not add, drop or alter. -/
theorem costedObservations_map_observation_perm {P : Profile}
    (W : Universal.Semantics P) (initial : Config)
    (obs : List ExecutionObservation)
    (h : ∀ o ∈ obs, FiniteExecution initial o) :
    ((costedObservations W initial obs h).map (·.observation)).Perm obs := by
  have hp := List.Perm.map (fun c : Universal.CostedExecutionObservation W initial =>
    c.observation) (costedObservations_perm W initial obs h)
  rwa [costedList_map_observation W initial obs h] at hp

theorem costedObservations_length {P : Profile} (W : Universal.Semantics P)
    (initial : Config) (obs : List ExecutionObservation)
    (h : ∀ o ∈ obs, FiniteExecution initial o) :
    (costedObservations W initial obs h).length = obs.length := by
  have hp := costedObservations_map_observation_perm W initial obs h
  have := hp.length_eq
  simpa using this

theorem costedObservations_ne_nil {P : Profile} (W : Universal.Semantics P)
    (initial : Config) (obs : List ExecutionObservation)
    (h : ∀ o ∈ obs, FiniteExecution initial o) (hne : obs ≠ []) :
    costedObservations W initial obs h ≠ [] := by
  intro hnil
  have hlen := costedObservations_length W initial obs h
  rw [hnil] at hlen
  exact hne (List.eq_nil_of_length_eq_zero hlen.symm)

/-! ## Costed initialization (SPEC §7.5, §10.1) -/

/-- The raw invocation a profile-level invocation installs. -/
def rawOfInvocation {P : Profile} (invocation : Invocation P) : RawInvocation where
  ptr := invocation.body.ptr.toNat
  bytes := invocation.body.bytes.toList

/-- The canonical site of a costed-initialization failure. -/
def initializationFaultSite : Foundation.CanonicalObjectId where
  schemaVersion := 1
  domain := CanonicalDomainTag.generic
  typeTag := costedExploreTag 3 "Wasm.initialGemmInvocationCosted"
  canonicalBodyBytes := Bytes.pack []

/-- The report code of each instantiation fault.  The three codes are distinct,
so the fault is recoverable from the report. -/
def initializationFaultCode : InstantiationFault → Nat
  | .invalidModule => 1
  | .allocationFailed => 2
  | .missingGemmExport => 3

theorem initializationFaultCode_injective :
    Function.Injective initializationFaultCode := by
  intro a b h
  cases a <;> cases b <;> first | rfl | exact absurd h (by decide)

/-- The located failure report of an instantiation fault. -/
def initializationFailureReport (fault : InstantiationFault) :
    Foundation.FailureReport where
  site := initializationFaultSite
  code := initializationFaultCode fault
  context := []

/-- The report names the fault: nothing is lost by reporting. -/
theorem initializationFailureReport_injective :
    Function.Injective initializationFailureReport := by
  intro a b h
  exact initializationFaultCode_injective (congrArg Foundation.FailureReport.code h)

/-- The initialization events `Wasm.initialConfig` actually performs on a
module that initializes: validation, memory allocation, global allocation,
export resolution, harness-frame construction and start-function selection.
The three fault rows of `Wasm.InitEventId` are charged nowhere, because a
faulting initialization produces no `InitializationObservation`. -/
def performedInitializationEvents : List InitEventId :=
  [ .validateModule, .allocateMemory, .allocateGlobals, .resolveGemmExport
  , .buildHarnessFrame, .selectStartFunction ]

/-- The sequential composition of the pinned initialization rows. -/
def initializationEventCost : List InitEventId → Cost.DynamicVector
  | [] => Cost.DynamicVector.zero
  | e :: rest => Cost.sequentialCompose e.row.contribution (initializationEventCost rest)

/-- The charge of materialising the module's static data: exactly
`Wasm.instantiatedStaticBytes`. -/
def staticInitializationCost (P : Profile) (m : Module) : Cost.DynamicVector :=
  { Cost.DynamicVector.zero with bytesWritten := instantiatedStaticBytes P m }

/-- **SPEC §7.5.**  The charge of decoding-independent initialization: the
pinned initialization rows composed with the exact static-byte charge. -/
def initializationCost (P : Profile) (m : Module) : Cost.DynamicVector :=
  Cost.sequentialCompose (initializationEventCost performedInitializationEvents)
    (staticInitializationCost P m)

/-- Every performed initialization event is charged one instantiation step and
one rule step, so the six rows contribute exactly six of each. -/
theorem initializationCost_instantiationSteps (P : Profile) (m : Module) :
    (initializationCost P m).instantiationSteps = 6 := rfl

theorem initializationCost_wasmRuleSteps (P : Profile) (m : Module) :
    (initializationCost P m).wasmRuleSteps = 6 := rfl

theorem initializationCost_dispatchSteps (P : Profile) (m : Module) :
    (initializationCost P m).dispatchSteps = 0 := rfl

/-- **The static bytes are charged exactly.**  The one extra written byte is
the `core3/init/allocate-globals` row of the pinned table. -/
theorem initializationCost_bytesWritten (P : Profile) (m : Module) :
    (initializationCost P m).bytesWritten = 1 + instantiatedStaticBytes P m := rfl

theorem instantiatedStaticBytes_le_initializationCost (P : Profile) (m : Module) :
    instantiatedStaticBytes P m ≤ (initializationCost P m).bytesWritten := by
  rw [initializationCost_bytesWritten]
  omega

/-- **SPEC §10.1**, `Wasm.initialGemmInvocationCosted`.  It returns `.ok`
exactly when `Wasm.initialConfig` builds a configuration, and `.error` exactly
on a real `Wasm.InstantiationFault`. -/
def initialGemmInvocationCosted {P : Profile} (m : Module)
    (invocation : Invocation P) :
    Except Foundation.FailureReport (Universal.InitializationObservation P) :=
  match initialConfig m (rawOfInvocation invocation) with
  | .error fault => .error (initializationFailureReport fault)
  | .ok initial => .ok { initial := initial, cost := initializationCost P m }

/-- **Non-vacuity.**  A module that genuinely initializes is accepted, and the
returned configuration is exactly `Wasm.initialConfig`'s. -/
theorem initialGemmInvocationCosted_ok {P : Profile} {m : Module}
    {invocation : Invocation P} {initial : Config}
    (h : initialConfig m (rawOfInvocation invocation) = .ok initial) :
    initialGemmInvocationCosted m invocation =
      .ok { initial := initial, cost := initializationCost P m } := by
  unfold initialGemmInvocationCosted
  rw [h]

theorem initialGemmInvocationCosted_error {P : Profile} {m : Module}
    {invocation : Invocation P} {fault : InstantiationFault}
    (h : initialConfig m (rawOfInvocation invocation) = .error fault) :
    initialGemmInvocationCosted m invocation =
      .error (initializationFailureReport fault) := by
  unfold initialGemmInvocationCosted
  rw [h]

/-- Failure is reported *only* on a real initialization fault. -/
theorem initialGemmInvocationCosted_error_iff {P : Profile} {m : Module}
    {invocation : Invocation P} {report : Foundation.FailureReport} :
    initialGemmInvocationCosted m invocation = .error report ↔
      ∃ fault : InstantiationFault,
        initialConfig m (rawOfInvocation invocation) = .error fault ∧
          report = initializationFailureReport fault := by
  unfold initialGemmInvocationCosted
  cases h : initialConfig m (rawOfInvocation invocation) with
  | error fault => simp [eq_comm]
  | ok initial => simp

/-- Success is reported *only* on a real initial configuration, and the
observation starts exactly there. -/
theorem initialGemmInvocationCosted_ok_iff {P : Profile} {m : Module}
    {invocation : Invocation P} {initial : Config} {cost : Cost.DynamicVector} :
    initialGemmInvocationCosted m invocation = .ok { initial := initial, cost := cost } ↔
      initialConfig m (rawOfInvocation invocation) = .ok initial ∧
        cost = initializationCost P m := by
  constructor
  · intro hh
    unfold initialGemmInvocationCosted at hh
    split at hh
    · exact absurd hh (by simp)
    · rename_i c hc
      simp only [Except.ok.injEq,
        Universal.InitializationObservation.mk.injEq] at hh
      exact ⟨by rw [hc, hh.1], hh.2.symm⟩
  · rintro ⟨h1, h2⟩
    rw [h2]
    exact initialGemmInvocationCosted_ok h1

/-- An invalid module is rejected before any reduction, and the report names
the reason. -/
theorem initialGemmInvocationCosted_invalid {P : Profile} {m : Module}
    (invocation : Invocation P) (h : validate m = false) :
    initialGemmInvocationCosted m invocation =
      .error (initializationFailureReport .invalidModule) :=
  initialGemmInvocationCosted_error (initialConfig_invalid h)

/-! ## Replaying a prefix

`Wasm.exploreAll`'s `nonterminalPrefix` constructor carries only the trace;
`Universal.NonterminalPrefix` additionally demands the configuration the prefix
reaches.  `runPrefix` recovers it deterministically from the successor
enumeration, which `Wasm.successors_keys_nodup` proves is keyed by its event. -/

/-- Replay a trace against the successor enumeration. -/
def runPrefix : Config → List Event → Option Config
  | c, [] => some c
  | c, e :: rest =>
      match (successors c).find? (fun p => decide (p.1 = e)) with
      | none => none
      | some p => runPrefix p.2 rest

/-- Replay is sound: a successful replay is a real reduction sequence. -/
theorem runPrefix_sound : ∀ (tr : List Event) (c c' : Config),
    runPrefix c tr = some c' → Reduces c tr c' := by
  intro tr
  induction tr with
  | nil =>
    intro c c' h
    unfold runPrefix at h
    have hc : c = c' := Option.some.inj h
    subst hc
    exact .refl c
  | cons e rest ih =>
    intro c c' h
    unfold runPrefix at h
    cases hf : (successors c).find? (fun p => decide (p.1 = e)) with
    | none => rw [hf] at h; exact absurd h (by simp)
    | some p =>
      rw [hf] at h
      have hmem : p ∈ successors c := List.mem_of_find?_eq_some hf
      have hp1 : p.1 = e := by
        have := List.find?_some hf
        simpa using this
      have hstep : Step c e p.2 := by
        refine (mem_successors_iff_step c e p.2).mp ?_
        rw [← hp1]
        exact hmem
      exact .cons hstep (ih p.2 c' h)

/-- Replay is complete: every reduction sequence replays, to its own endpoint.
This is where `Wasm.successors_keys_nodup` is used. -/
theorem runPrefix_eq_some_of_reduces {c c' : Config} {tr : List Event}
    (h : Reduces c tr c') : runPrefix c tr = some c' := by
  induction h with
  | refl c => rfl
  | @cons c e c₁ tr' c'' hstep _ ih =>
    have hmem : (e, c₁) ∈ successors c := (mem_successors_iff_step c e c₁).mpr hstep
    unfold runPrefix
    cases hf : (successors c).find? (fun p => decide (p.1 = e)) with
    | none =>
      rw [List.find?_eq_none] at hf
      exact absurd (hf (e, c₁) hmem) (by simp)
    | some p =>
      have hp : p ∈ successors c := List.mem_of_find?_eq_some hf
      have hp1 : p.1 = e := by
        have := List.find?_some hf
        simpa using this
      have hp2 : p.2 = c₁ := by
        refine eq_of_key_nodup (successors_keys_nodup c) ?_ hmem
        rw [← hp1]
        exact hp
      show runPrefix p.2 tr' = some c''
      rw [hp2]
      exact ih

/-- The overrunning prefix as `Universal.NonterminalPrefix` demands it: the
trace, the configuration it reaches, and the reduction proof.  The `none`
branch is proved unreachable, not defaulted. -/
def nonterminalPrefixOf (initial : Config) (n : Nat) (t : List Event)
    (hlen : t.length = n) (hex : ∃ final : Config, Reduces initial t final) :
    Universal.NonterminalPrefix initial n :=
  match hf : runPrefix initial t with
  | some final =>
      { witness := { events := t, final := final, lengthEq := hlen }
        valid := runPrefix_sound t initial final hf }
  | none =>
      False.elim (by
        obtain ⟨f, hred⟩ := hex
        rw [runPrefix_eq_some_of_reduces hred] at hf
        simp at hf)

/-- The recovered prefix carries exactly the trace it was given. -/
theorem nonterminalPrefixOf_events (initial : Config) (n : Nat) (t : List Event)
    (hlen : t.length = n) (hex : ∃ final : Config, Reduces initial t final) :
    (nonterminalPrefixOf initial n t hlen hex).witness.events = t := by
  unfold nonterminalPrefixOf
  split
  · rfl
  · rename_i hf
    exfalso
    obtain ⟨f, hred⟩ := hex
    rw [runPrefix_eq_some_of_reduces hred] at hf
    simp at hf

/-! ## The costed all-branch explorer (SPEC §7.4, §10.1) -/

/-- The costed coverage obligation, transported from `Wasm.covers_exploreTree`
— hence from `Wasm.runFuel_sound` and `Wasm.runFuel_complete_with_bound` —
through any permutation of the observation list. -/
theorem costedCoverage_of_perm {P : Profile} (W : Universal.Semantics P)
    (bound : Nat) (initial : Config)
    (f : NonemptyCanonicalFrontier (Universal.CostedExecutionObservation W initial))
    (hperm : (f.elements.map (·.observation)).Perm (exploreTree (bound + 1) [] initial)) :
    Universal.CostedCoverage W bound initial f := by
  refine ⟨fun o _ => o.execution, ?_⟩
  intro o hrun hlen
  have hmem : o ∈ exploreTree (bound + 1) [] initial :=
    (covers_exploreTree bound initial).2 o hrun hlen
  have hmem' : o ∈ f.elements.map (·.observation) := hperm.mem_iff.mpr hmem
  obtain ⟨c, hc, hco⟩ := List.mem_map.mp hmem'
  exact ⟨c, hc, hco⟩

/-- Build the completed costed result from a canonically ordered costed list.
The empty case is the disclosed deviation: `Universal.CostedTreeResult` has no
"completed with no observation" constructor. -/
def completeFrom {P : Profile} (W : Universal.Semantics P) (bound : Nat)
    (initial : Config) :
    (l : List (Universal.CostedExecutionObservation W initial)) →
    CanonicalSorted (l.map (costedObservationSchema W initial).encode) →
    (l.map (·.observation)).Perm (exploreTree (bound + 1) [] initial) →
    Universal.CostedTreeResult W bound initial
  | [], _, _ => .initializationFailure .allocationFailed
  | c :: rest, hs, hperm =>
      .complete (frontierOfSortedCons (costedObservationSchema W initial) c rest hs)
        (costedCoverage_of_perm W bound initial _ hperm)

theorem completeFrom_ne_nil {P : Profile} (W : Universal.Semantics P)
    (bound : Nat) (initial : Config) :
    ∀ (l : List (Universal.CostedExecutionObservation W initial))
      (hs : CanonicalSorted (l.map (costedObservationSchema W initial).encode))
      (hperm : (l.map (·.observation)).Perm (exploreTree (bound + 1) [] initial)),
      l ≠ [] →
      ∃ frontier coverage,
        completeFrom W bound initial l hs hperm = .complete frontier coverage
  | [], _, _, hne => absurd rfl hne
  | _ :: _, _, _, _ => ⟨_, _, rfl⟩

theorem completeFrom_nil {P : Profile} (W : Universal.Semantics P) (bound : Nat)
    (initial : Config) :
    ∀ (l : List (Universal.CostedExecutionObservation W initial))
      (hs : CanonicalSorted (l.map (costedObservationSchema W initial).encode))
      (hperm : (l.map (·.observation)).Perm (exploreTree (bound + 1) [] initial)),
      l = [] →
      completeFrom W bound initial l hs hperm = .initializationFailure .allocationFailed
  | [], _, _, _ => rfl
  | _ :: _, _, _, h => absurd h (by simp)

/-- The elements of a frontier returned by `completeFrom` are exactly the list
it was given. -/
def costedFrontierElements {P : Profile} {W : Universal.Semantics P} {bound : Nat}
    {initial : Config} :
    Universal.CostedTreeResult W bound initial →
      List (Universal.CostedExecutionObservation W initial)
  | .complete f _ => f.elements
  | .nonterminalPrefix _ => []
  | .initializationFailure _ => []

/-- The overrunning trace recorded by a costed result. -/
def costedResultOverrunEvents {P : Profile} {W : Universal.Semantics P} {bound : Nat}
    {initial : Config} :
    Universal.CostedTreeResult W bound initial → List Event
  | .complete _ _ => []
  | .nonterminalPrefix overrun => overrun.witness.events
  | .initializationFailure _ => []

theorem completeFrom_elements {P : Profile} (W : Universal.Semantics P)
    (bound : Nat) (initial : Config) :
    ∀ (l : List (Universal.CostedExecutionObservation W initial))
      (hs : CanonicalSorted (l.map (costedObservationSchema W initial).encode))
      (hperm : (l.map (·.observation)).Perm (exploreTree (bound + 1) [] initial))
      (frontier : NonemptyCanonicalFrontier
        (Universal.CostedExecutionObservation W initial))
      (coverage : Universal.CostedCoverage W bound initial frontier),
      completeFrom W bound initial l hs hperm = .complete frontier coverage →
      frontier.elements = l
  | [], _, _, _, _, h => by exact absurd h (by simp [completeFrom])
  | c :: rest, hs, hperm, frontier, coverage, h => by
    have := congrArg costedFrontierElements h
    simpa [completeFrom, costedFrontierElements] using this.symm

/-- The completed branch of the costed explorer. -/
def completeAll {P : Profile} (W : Universal.Semantics P) (bound : Nat)
    (initial : Config) : Universal.CostedTreeResult W bound initial :=
  completeFrom W bound initial
    (costedObservations W initial (exploreTree (bound + 1) [] initial)
      (covers_exploreTree bound initial).1)
    (costedObservations_sorted W initial _ _)
    (costedObservations_map_observation_perm W initial _ _)

/-- The overrunning branch of the costed explorer. -/
def nonterminalAll {P : Profile} (W : Universal.Semantics P) (bound : Nat)
    (initial : Config) (t : List Event)
    (hmem : t ∈ prefixes (bound + 1) [] initial) :
    Universal.CostedTreeResult W bound initial :=
  .nonterminalPrefix
    (nonterminalPrefixOf initial (bound + 1) t
      (by simpa using prefixes_length (bound + 1) [] initial t hmem)
      (by
        obtain ⟨suffix, final, ht, hred, _⟩ :=
          prefixes_sound (bound + 1) [] initial t hmem
        refine ⟨final, ?_⟩
        rw [ht]
        simpa using hred))

/-- **SPEC §10.1**, `Wasm.exploreAllCosted`.  It matches on exactly the same
scrutinee as `Wasm.exploreAll`, so the two results correspond constructor for
constructor. -/
def exploreAllCosted {P : Profile} (W : Universal.Semantics P) (bound : Nat)
    (initial : Config) : Universal.CostedTreeResult W bound initial :=
  match hp : (prefixes (bound + 1) [] initial).head? with
  | some t => nonterminalAll W bound initial t (mem_of_head? hp)
  | none => completeAll W bound initial

theorem exploreAllCosted_eq_completeAll {P : Profile} (W : Universal.Semantics P)
    (bound : Nat) (initial : Config)
    (h : (prefixes (bound + 1) [] initial).head? = none) :
    exploreAllCosted W bound initial = completeAll W bound initial := by
  unfold exploreAllCosted
  split
  · rename_i t hp
    rw [h] at hp
    exact absurd hp (by simp)
  · rfl

theorem exploreAllCosted_eq_nonterminalAll {P : Profile} (W : Universal.Semantics P)
    (bound : Nat) (initial : Config) (t : List Event)
    (h : (prefixes (bound + 1) [] initial).head? = some t) :
    exploreAllCosted W bound initial =
      nonterminalAll W bound initial t (mem_of_head? h) := by
  unfold exploreAllCosted
  split
  · rename_i t' hp
    rw [h] at hp
    have : t' = t := (Option.some.inj hp).symm
    subst this
    rfl
  · rename_i hp
    rw [h] at hp
    exact absurd hp (by simp)

/-! ### The plain explorer's shape -/

theorem head?_prefixes_eq_none_of_complete {bound : Nat} {initial : Config}
    {obs : List ExecutionObservation}
    {cov : CoversEveryMaximalFiniteBranch bound initial obs}
    (h : exploreAll bound initial = .complete obs cov) :
    (prefixes (bound + 1) [] initial).head? = none := by
  unfold exploreAll at h
  split at h
  · exact absurd h (by simp)
  · assumption

theorem exploreAll_complete_observations {bound : Nat} {initial : Config}
    {obs : List ExecutionObservation}
    {cov : CoversEveryMaximalFiniteBranch bound initial obs}
    (h : exploreAll bound initial = .complete obs cov) :
    obs = exploreTree (bound + 1) [] initial := by
  have h1 : (exploreAll bound initial).observations = obs := by rw [h]; rfl
  rw [observations_exploreAll] at h1
  exact h1.symm

theorem exploreAll_eq_complete_of_head?_none {bound : Nat} {initial : Config}
    (h : (prefixes (bound + 1) [] initial).head? = none) :
    exploreAll bound initial =
      .complete (exploreTree (bound + 1) [] initial) (covers_exploreTree bound initial) := by
  unfold exploreAll
  split
  · rename_i t hp
    rw [h] at hp
    exact absurd hp (by simp)
  · rfl

/-! ### Erasure (SPEC §7.5) -/

/-- The observations recorded by a costed result. -/
def costedResultObservations {P : Profile} {W : Universal.Semantics P} {bound : Nat}
    {initial : Config} (r : Universal.CostedTreeResult W bound initial) :
    List ExecutionObservation :=
  (costedFrontierElements r).map (·.observation)

/--
  **SPEC §7.5 / §10.1**, `Wasm.exploreAllCosted_erases`, in its strongest true
  form.

  When the costed explorer completes, the plain explorer completes too, with the
  *same* `.complete` constructor and the *same* observations up to order.  The
  SPEC display asks for list equality; that is incompatible with
  `NonemptyCanonicalFrontier.ordered`, which forces the canonical byte order,
  while `Wasm.exploreTree` emits depth-first successor order.  Coverage is a
  membership statement, so nothing is lost.
-/
theorem exploreAllCosted_erases {P : Profile} {W : Universal.Semantics P}
    {bound : Nat} {initial : Config}
    {frontier : NonemptyCanonicalFrontier (Universal.CostedExecutionObservation W initial)}
    {coverage : Universal.CostedCoverage W bound initial frontier}
    (h : exploreAllCosted W bound initial = .complete frontier coverage) :
    exploreAll bound initial =
        .complete (exploreTree (bound + 1) [] initial) (covers_exploreTree bound initial) ∧
      (frontier.elements.map (·.observation)).Perm (exploreTree (bound + 1) [] initial) := by
  have hnone : (prefixes (bound + 1) [] initial).head? = none := by
    cases hp : (prefixes (bound + 1) [] initial).head? with
    | none => rfl
    | some t =>
      rw [exploreAllCosted_eq_nonterminalAll W bound initial t hp] at h
      exact absurd h (by simp [nonterminalAll])
  refine ⟨exploreAll_eq_complete_of_head?_none hnone, ?_⟩
  rw [exploreAllCosted_eq_completeAll W bound initial hnone] at h
  unfold completeAll at h
  have helem :=
    completeFrom_elements W bound initial
      (costedObservations W initial (exploreTree (bound + 1) [] initial)
        (covers_exploreTree bound initial).1)
      (costedObservations_sorted W initial _ _)
      (costedObservations_map_observation_perm W initial _ _) frontier coverage h
  rw [helem]
  exact costedObservations_map_observation_perm W initial _ _

/-- The erased observations of a completed costed exploration are exactly, up
to order, the plain explorer's observations. -/
theorem exploreAllCosted_erases_observations {P : Profile} {W : Universal.Semantics P}
    {bound : Nat} {initial : Config}
    {frontier : NonemptyCanonicalFrontier (Universal.CostedExecutionObservation W initial)}
    {coverage : Universal.CostedCoverage W bound initial frontier}
    (h : exploreAllCosted W bound initial = .complete frontier coverage) :
    (costedResultObservations (exploreAllCosted W bound initial)).Perm
      (exploreAll bound initial).observations := by
  obtain ⟨hplain, hperm⟩ := exploreAllCosted_erases h
  rw [h, observations_exploreAll]
  exact hperm

/-- When the costed explorer reports an overrun, so does the plain explorer,
with the same trace. -/
theorem exploreAllCosted_erases_nonterminal {P : Profile} {W : Universal.Semantics P}
    {bound : Nat} {initial : Config}
    {overrun : Universal.NonterminalPrefix initial (bound + 1)}
    (h : exploreAllCosted W bound initial = .nonterminalPrefix overrun) :
    ∃ (t : TraceOfLength (bound + 1)),
      exploreAll bound initial =
          .nonterminalPrefix (exploreTree (bound + 1) [] initial)
            (covers_exploreTree bound initial) t ∧
        t.events = overrun.witness.events := by
  cases hp : (prefixes (bound + 1) [] initial).head? with
  | none =>
    rw [exploreAllCosted_eq_completeAll W bound initial hp] at h
    unfold completeAll completeFrom at h
    split at h
    · exact absurd h (by simp)
    · exact absurd h (by simp)
  | some t =>
    refine ⟨⟨t, by simpa using prefixes_length (bound + 1) [] initial t (mem_of_head? hp)⟩,
      ?_, ?_⟩
    · unfold exploreAll
      split
      · rename_i t' hp'
        rw [hp] at hp'
        have : t' = t := (Option.some.inj hp').symm
        subst this
        rfl
      · rename_i hp'
        rw [hp] at hp'
        exact absurd hp' (by simp)
    · rw [exploreAllCosted_eq_nonterminalAll W bound initial t hp] at h
      unfold nonterminalAll at h
      have hev := congrArg costedResultOverrunEvents h
      simpa [costedResultOverrunEvents, nonterminalPrefixOf_events] using hev

/-- **The `.initializationFailure` fallback is characterised.**  It is reached
exactly when the plain explorer completes with *no* observation at all, i.e.
when every branch of `initial` becomes stuck while still `running`. -/
theorem exploreAllCosted_initializationFailure_iff {P : Profile}
    (W : Universal.Semantics P) (bound : Nat) (initial : Config) :
    (∃ fault, exploreAllCosted W bound initial = .initializationFailure fault) ↔
      ((prefixes (bound + 1) [] initial).head? = none ∧
        exploreTree (bound + 1) [] initial = []) := by
  constructor
  · rintro ⟨fault, h⟩
    have hnone : (prefixes (bound + 1) [] initial).head? = none := by
      cases hp : (prefixes (bound + 1) [] initial).head? with
      | none => rfl
      | some t =>
        rw [exploreAllCosted_eq_nonterminalAll W bound initial t hp] at h
        exact absurd h (by simp [nonterminalAll])
    refine ⟨hnone, ?_⟩
    rw [exploreAllCosted_eq_completeAll W bound initial hnone] at h
    unfold completeAll at h
    cases hlist : exploreTree (bound + 1) [] initial with
    | nil => rfl
    | cons o rest =>
      exfalso
      obtain ⟨frontier, coverage, hc⟩ :=
        completeFrom_ne_nil W bound initial
          (costedObservations W initial (exploreTree (bound + 1) [] initial)
            (covers_exploreTree bound initial).1)
          (costedObservations_sorted W initial _ _)
          (costedObservations_map_observation_perm W initial _ _)
          (costedObservations_ne_nil W initial _ _ (by rw [hlist]; simp))
      exact absurd (h.symm.trans hc) (by simp)
  · rintro ⟨hnone, hempty⟩
    refine ⟨.allocationFailed, ?_⟩
    rw [exploreAllCosted_eq_completeAll W bound initial hnone]
    unfold completeAll
    have hnil : costedObservations W initial (exploreTree (bound + 1) [] initial)
        (covers_exploreTree bound initial).1 = [] := by
      apply List.eq_nil_of_length_eq_zero
      rw [costedObservations_length W initial (exploreTree (bound + 1) [] initial)
        (covers_exploreTree bound initial).1, hempty]
      rfl
    exact completeFrom_nil W bound initial _ _ _ hnil

/-! ### Soundness and completeness -/

/--
  **SPEC §7.4.**  Soundness: every observation the costed explorer returns is a
  real costed execution of `initial` — a real finite execution, charged exactly
  its trace cost, and present in the plain explorer's tree.
-/
theorem exploreAllCosted_sound {P : Profile} {W : Universal.Semantics P}
    {bound : Nat} {initial : Config}
    {frontier : NonemptyCanonicalFrontier (Universal.CostedExecutionObservation W initial)}
    {coverage : Universal.CostedCoverage W bound initial frontier}
    (h : exploreAllCosted W bound initial = .complete frontier coverage)
    (c : Universal.CostedExecutionObservation W initial)
    (hc : c ∈ frontier.elements) :
    FiniteExecution initial c.observation ∧
      c.cost = Cost.traceCost (c.observation.trace.map W.costEvent) ∧
      TreeContains (exploreAll bound initial) c.observation := by
  refine ⟨c.execution, c.costExact, ?_⟩
  obtain ⟨_, hperm⟩ := exploreAllCosted_erases h
  unfold TreeContains
  rw [observations_exploreAll]
  exact hperm.mem_iff.mp (List.mem_map_of_mem hc)

/--
  **SPEC §7.4.**  Completeness relative to the explicit bound: every relational
  branch of length at most the bound appears in the costed frontier.  This is
  `Wasm.runFuel_complete_with_bound`, transported.
-/
theorem exploreAllCosted_complete {P : Profile} {W : Universal.Semantics P}
    {bound : Nat} {initial : Config}
    {frontier : NonemptyCanonicalFrontier (Universal.CostedExecutionObservation W initial)}
    {coverage : Universal.CostedCoverage W bound initial frontier}
    (h : exploreAllCosted W bound initial = .complete frontier coverage)
    {o : ExecutionObservation} (hrun : FiniteExecution initial o)
    (hlen : o.trace.length ≤ bound) :
    ∃ c ∈ frontier.elements, c.observation = o := by
  obtain ⟨hplain, hperm⟩ := exploreAllCosted_erases h
  have hmem : TreeContains (exploreAll bound initial) o :=
    runFuel_complete_with_bound hrun hlen
  unfold TreeContains at hmem
  rw [observations_exploreAll] at hmem
  obtain ⟨c, hc, hco⟩ := List.mem_map.mp (hperm.mem_iff.mpr hmem)
  exact ⟨c, hc, hco⟩

/-- Every branch the plain explorer found is in the costed frontier. -/
theorem exploreAllCosted_complete_of_treeContains {P : Profile}
    {W : Universal.Semantics P} {bound : Nat} {initial : Config}
    {frontier : NonemptyCanonicalFrontier (Universal.CostedExecutionObservation W initial)}
    {coverage : Universal.CostedCoverage W bound initial frontier}
    (h : exploreAllCosted W bound initial = .complete frontier coverage)
    {o : ExecutionObservation} (hmem : TreeContains (exploreAll bound initial) o) :
    ∃ c ∈ frontier.elements, c.observation = o := by
  obtain ⟨hplain, hperm⟩ := exploreAllCosted_erases h
  unfold TreeContains at hmem
  rw [observations_exploreAll] at hmem
  obtain ⟨c, hc, hco⟩ := List.mem_map.mp (hperm.mem_iff.mpr hmem)
  exact ⟨c, hc, hco⟩

/--
  **THE DECISIVE THEOREM.**

  Whenever the plain explorer returns `.complete` with a nonempty observation
  list, the costed explorer returns `.complete`.  Without this the machine could
  never be shown to complete, and every downstream `Universal.SystemEvaluation`
  would stay uninhabited.
-/
theorem exploreAllCosted_complete_of_nonempty {P : Profile}
    (W : Universal.Semantics P) {bound : Nat} {initial : Config}
    {obs : List ExecutionObservation}
    {cov : CoversEveryMaximalFiniteBranch bound initial obs}
    (h : exploreAll bound initial = .complete obs cov) (hne : obs ≠ []) :
    ∃ (frontier : NonemptyCanonicalFrontier
          (Universal.CostedExecutionObservation W initial))
      (coverage : Universal.CostedCoverage W bound initial frontier),
      exploreAllCosted W bound initial = .complete frontier coverage := by
  have hnone : (prefixes (bound + 1) [] initial).head? = none :=
    head?_prefixes_eq_none_of_complete h
  have hobs : obs = exploreTree (bound + 1) [] initial :=
    exploreAll_complete_observations h
  have hne' : exploreTree (bound + 1) [] initial ≠ [] := by rwa [hobs] at hne
  obtain ⟨frontier, coverage, hc⟩ :=
    completeFrom_ne_nil W bound initial
      (costedObservations W initial (exploreTree (bound + 1) [] initial)
        (covers_exploreTree bound initial).1)
      (costedObservations_sorted W initial _ _)
      (costedObservations_map_observation_perm W initial _ _)
      (costedObservations_ne_nil W initial _ _ hne')
  refine ⟨frontier, coverage, ?_⟩
  rw [exploreAllCosted_eq_completeAll W bound initial hnone]
  unfold completeAll
  exact hc

/-- The same statement in the form the next phase consumes: a nonempty tree
gives a completed costed exploration whose frontier covers it. -/
theorem exploreAllCosted_complete_of_exploreTree_ne_nil {P : Profile}
    (W : Universal.Semantics P) (bound : Nat) (initial : Config)
    (hnone : (prefixes (bound + 1) [] initial).head? = none)
    (hne : exploreTree (bound + 1) [] initial ≠ []) :
    ∃ (frontier : NonemptyCanonicalFrontier
          (Universal.CostedExecutionObservation W initial))
      (coverage : Universal.CostedCoverage W bound initial frontier),
      exploreAllCosted W bound initial = .complete frontier coverage :=
  exploreAllCosted_complete_of_nonempty W (exploreAll_eq_complete_of_head?_none hnone)
    (by rwa [← exploreAll_complete_observations
      (exploreAll_eq_complete_of_head?_none hnone)] at hne)

/-! ## The released costed machine (SPEC §10.1) -/

/-- **SPEC §10.1.**  The costed machine: costed initialization and the bounded
all-branch costed explorer, assembled from the two definitions above. -/
def releaseCostedMachine {P : Profile} (W : Universal.Semantics P) :
    Universal.CostedMachine W where
  initialGemmInvocationCosted := initialGemmInvocationCosted
  exploreAllCosted := exploreAllCosted W

@[simp] theorem releaseCostedMachine_initial {P : Profile}
    (W : Universal.Semantics P) (m : Module) (invocation : Invocation P) :
    (releaseCostedMachine W).initialGemmInvocationCosted m invocation =
      initialGemmInvocationCosted m invocation := rfl

@[simp] theorem releaseCostedMachine_explore {P : Profile}
    (W : Universal.Semantics P) (bound : Nat) (initial : Config) :
    (releaseCostedMachine W).exploreAllCosted bound initial =
      exploreAllCosted W bound initial := rfl

/-- **Non-vacuity of the machine.**  On a module that genuinely initializes and
an initial configuration all of whose branches terminate inside the bound with
at least one observation, the released machine both starts and completes. -/
theorem releaseCostedMachine_completes {P : Profile} (W : Universal.Semantics P)
    {m : Module} {invocation : Invocation P} {initial : Config} {bound : Nat}
    (hinit : initialConfig m (rawOfInvocation invocation) = .ok initial)
    {obs : List ExecutionObservation}
    {cov : CoversEveryMaximalFiniteBranch bound initial obs}
    (hplain : exploreAll bound initial = .complete obs cov) (hne : obs ≠ []) :
    (releaseCostedMachine W).initialGemmInvocationCosted m invocation =
        .ok { initial := initial, cost := initializationCost P m } ∧
      ∃ (frontier : NonemptyCanonicalFrontier
            (Universal.CostedExecutionObservation W initial))
        (coverage : Universal.CostedCoverage W bound initial frontier),
        (releaseCostedMachine W).exploreAllCosted bound initial =
          .complete frontier coverage :=
  ⟨initialGemmInvocationCosted_ok hinit,
    exploreAllCosted_complete_of_nonempty W hplain hne⟩

end WasmGemmGnaf.Wasm
