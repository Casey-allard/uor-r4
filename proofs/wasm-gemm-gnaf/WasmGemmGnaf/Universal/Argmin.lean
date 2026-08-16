/-
  Phase D: attainment of `GlobalOptimal`.
  Normative sources: SPEC.md section 13 (Phase D), section 1 (`GlobalOptimal`),
  section 9.3 (properness), section 10.1 (the competitor universe) and section
  10.3 (the sublevel construction).

  WHAT THIS FILE DOES.

  It proves that `GlobalOptimal` is *attainable*: if the admissible set is
  nonempty — i.e. if there exists even one byte sequence that is profile valid,
  semantically correct, and within resources, and that the decider evaluates —
  then some byte sequence satisfies `GlobalOptimal` outright.  The construction
  is entirely classical; nothing is computed.

  The chain is:

    1. `systemEvaluation_subsingleton` — `SystemEvaluation S bytes` has at most
       one inhabitant.  Every field is pinned: `module` by `decodeEq` (decoding
       is a function), `perInput` pointwise by `initialEq`, `treeComplete` and
       `resourceExact`, and `cost` by `costExact` through
       `Cost.exact_unique`.  THIS IS LOAD-BEARING.  `GlobalOptimal`'s lower
       bound clause is `∀ competitorEval, SystemEvaluationRel … ∧ score ≤ score`
       — a CONJUNCTION under a universal quantifier, so it asserts that the
       decider's answer is *every* inhabitant of the evaluation type.  That is
       satisfiable only because the type is a subsingleton.
    2. `correct_of_admissible` / `feasible_of_admissible` — the *extensional*
       predicates of SPEC §10.1 imply the *evaluator-side* predicates `Correct`
       and `Feasible` for any evaluation of the same bytes.  This works because
       `Wasm.isTerminalObservation` makes every recorded finite execution a
       maximal execution, so the extensional quantifiers reach every recorded
       branch.
    3. `exists_least_admissible_score` — the achievable scores form a nonempty
       set of naturals, hence have a least element.
    4. `minimizer_size_le_score` — a minimizer's byte size is bounded by its
       score, via properness (`Cost.ProperObjective.coordinate_le_score`) and
       the `ExactAggregateCost` equation `cost.static.moduleBytes = bytes.size`.
       So all minimizers live inside the finite list
       `Enumerate.boundedByteArrays m`.
    5. `exists_canonical_least_minimizer` — a nonempty sub-collection of a
       finite list has a least element under the total order
       `Foundation.CanonicalBytesLE`.
    6. `exists_globalOptimal_of_nonempty` — that element satisfies every
       conjunct of `GlobalOptimal`.

  WHAT THIS FILE DOES NOT DO.

  It does NOT prove `Artifact.released_wasm_gemm_gnaf_global_optimal`, and it
  does not bring that theorem any closer than the two remaining obligations it
  isolates:

    (a) NONEMPTINESS — exhibiting one concrete byte sequence together with
        proofs of `ProfileValid`, `SemanticCorrect` and `SemanticWithinResources`
        and an inhabitant of `SystemEvaluation`.  Nothing here supplies one;
        the theorem below is an implication with exactly that antecedent.
    (b) IDENTIFICATION — proving that the committed release literal *is* the
        byte sequence this construction selects.  The selection is classical and
        non-constructive: `exists_globalOptimal_of_nonempty` produces an
        existential, not a computable choice, so it names no literal.

  The value of the file is the reduction: the optimality half of the release
  theorem is discharged, and what remains is an exhibition problem.

  ONE HYPOTHESIS IS ADDED, AND IT IS NOT DERIVABLE.  `GlobalOptimal` requires
  that the decider's answer on an admissible competitor *is* that competitor's
  evaluation.  A `Decider` is an arbitrary total function of the bytes; nothing
  in `Universal/Competitor.lean` forces it to return `.complete` when an
  evaluation exists.  `DeciderAnswersAdmissible` below is the weakest statement
  that closes the gap — it only asks that the decider return *some* completed
  evaluation on admissible bytes that have one — and
  `deciderAnswersAdmissible_rel` upgrades it to the required "for every
  inhabitant" form using `systemEvaluation_subsingleton`.

  Every declaration in this file is either a definition or a proved theorem.
  Nothing is assumed, nothing is `sorry`, and no conclusion is stored in a
  structure field.
-/
import WasmGemmGnaf.Universal.Sublevel
import WasmGemmGnaf.Universal.LowerBound

set_option autoImplicit false

namespace WasmGemmGnaf.Universal

/-! ## Generic order and arithmetic lemmas

These are stated for the plain carriers so that no part of the argument below
depends on the WebAssembly or GEMM layers. -/

section Generic

/-- Well-ordering of the naturals, in the form used below: a satisfiable
predicate on `Nat` has a least witness.  Proved by strong induction; no
decidability of `p` is needed and no choice principle is used to *produce* the
witness — the recursion does. -/
theorem exists_least_nat (p : Nat → Prop) :
    ∀ n : Nat, p n → ∃ m : Nat, p m ∧ ∀ k : Nat, p k → m ≤ k := by
  intro n
  induction n using Nat.strongRecOn with
  | _ n ih =>
    intro hn
    by_cases hlt : ∃ j, j < n ∧ p j
    · obtain ⟨j, hj, hpj⟩ := hlt
      exact ih j hj hpj
    · refine ⟨n, hn, ?_⟩
      intro k hk
      exact Nat.le_of_not_lt (fun hkn => hlt ⟨k, hkn, hk⟩)

/-- A nonempty sub-collection of a finite list of byte arrays has a
canonically least member.  `Foundation.CanonicalBytesLE` is total, reflexive and
transitive, which is exactly what the induction needs. -/
theorem exists_canonical_least_in_list :
    ∀ (l : List ByteArray) (p : ByteArray → Prop), (∃ x, x ∈ l ∧ p x) →
      ∃ x, x ∈ l ∧ p x ∧ ∀ y ∈ l, p y → Foundation.CanonicalBytesLE x y := by
  intro l
  induction l with
  | nil => intro p hex; obtain ⟨x, hx, _⟩ := hex; simp at hx
  | cons a t ih =>
    intro p hex
    by_cases hsub : ∃ x, x ∈ t ∧ p x
    · obtain ⟨z, hz, hpz, hmin⟩ := ih p hsub
      by_cases hpa : p a
      · rcases Foundation.CanonicalBytesLE.total a z with hle | hle
        · refine ⟨a, by simp, hpa, ?_⟩
          intro y hy hpy
          rcases List.mem_cons.mp hy with rfl | hyt
          · exact Foundation.CanonicalBytesLE.refl _
          · exact Foundation.CanonicalBytesLE.trans hle (hmin y hyt hpy)
        · refine ⟨z, by simp [hz], hpz, ?_⟩
          intro y hy hpy
          rcases List.mem_cons.mp hy with rfl | hyt
          · exact hle
          · exact hmin y hyt hpy
      · refine ⟨z, by simp [hz], hpz, ?_⟩
        intro y hy hpy
        rcases List.mem_cons.mp hy with rfl | hyt
        · exact absurd hpy hpa
        · exact hmin y hyt hpy
    · obtain ⟨x, hx, hpx⟩ := hex
      rcases List.mem_cons.mp hx with rfl | hxt
      · refine ⟨x, by simp, hpx, ?_⟩
        intro y hy hpy
        rcases List.mem_cons.mp hy with rfl | hyt
        · exact Foundation.CanonicalBytesLE.refl _
        · exact absurd ⟨y, hyt, hpy⟩ hsub
      · exact absurd ⟨x, hxt, hpx⟩ hsub

/-- Composing a fixed prefix with a componentwise maximum is the componentwise
maximum of the two compositions: `sequentialCompose` adds thirteen coordinates
and maximises three, and both operations distribute over `max`. -/
theorem value_sequentialCompose_componentwiseMax
    (init a b : Cost.DynamicVector) (dc : Cost.DynamicCoordinate) :
    dc.value (Cost.sequentialCompose init (Cost.ComponentwiseMax a b)) =
      max (dc.value (Cost.sequentialCompose init a))
        (dc.value (Cost.sequentialCompose init b)) := by
  cases dc <;>
    simp only [Cost.DynamicCoordinate.value, Cost.sequentialCompose,
      Cost.ComponentwiseMax] <;> omega

/-- If a fixed prefix composed with *each* recorded branch stays inside a limit,
so does the prefix composed with the componentwise maximum over the branches.
The empty-list case needs the prefix itself to be inside the limit. -/
theorem sequentialCompose_maxOverCosts_le {init limit : Cost.DynamicVector}
    (hinit : Cost.DynamicVector.ComponentwiseLE init limit) :
    ∀ l : List Cost.DynamicVector,
      (∀ v ∈ l, Cost.DynamicVector.ComponentwiseLE
        (Cost.sequentialCompose init v) limit) →
      Cost.DynamicVector.ComponentwiseLE
        (Cost.sequentialCompose init (Cost.maxOverCosts l)) limit := by
  intro l
  induction l with
  | nil =>
    intro _
    rw [Cost.maxOverCosts_nil, Cost.sequentialCompose_zero_right]
    exact hinit
  | cons v t ih =>
    intro h dc
    have hv := h v (by simp) dc
    have ht := ih (fun w hw => h w (by simp [hw])) dc
    rw [Cost.maxOverCosts_cons, value_sequentialCompose_componentwiseMax]
    exact Nat.max_le.mpr ⟨hv, ht⟩

end Generic

/-! ## Uniqueness of the evaluation (SPEC §10.1)

`SystemEvaluation S bytes` records the *exact* result of running the decider's
finite pipeline on `bytes`.  Every field is pinned by an equation to a function
value, so the type has at most one inhabitant.  This is what makes the
conjunctive lower-bound clause of `GlobalOptimal` satisfiable at all. -/

section Uniqueness

variable {P : Wasm.Profile}

/-- An `InputEvaluation` is pinned by its own fields: `initialization` by
`initialEq` (costed initialization is a function), `initial` by
`initialConfigEq`, `observations` by `treeComplete` (the explorer is a
function), and `resourceVector` by `resourceExact`. -/
theorem inputEvaluation_subsingleton {S : Setting P} {module : Wasm.Module}
    {raw : Gemm.RawInvocation P} (a b : InputEvaluation S module raw) : a = b := by
  obtain ⟨ai, aini, aceq, aeq, aobs, atc, arv, arex⟩ := a
  obtain ⟨bi, bini, bceq, beq, bobs, btc, brv, brex⟩ := b
  have hini : aini = bini := by
    have h := aeq.symm.trans beq
    exact Except.ok.inj h
  subst hini
  subst aceq
  subst bceq
  obtain ⟨ac, hac⟩ := atc
  obtain ⟨bc, hbc⟩ := btc
  have hobs : aobs = bobs := by
    have h := hac.symm.trans hbc
    injection h
  subst hobs
  have hrv : arv = brv := arex.trans brex.symm
  subst hrv
  rfl

instance instSubsingletonInputEvaluation {S : Setting P} {module : Wasm.Module}
    {raw : Gemm.RawInvocation P} : Subsingleton (InputEvaluation S module raw) :=
  ⟨inputEvaluation_subsingleton⟩

variable [Foundation.Fintype (Gemm.RawInvocation P)]

/--
  **Evaluation uniqueness.**  `SystemEvaluation S bytes` is a subsingleton.

  `module` is pinned by `decodeEq` because `Wasm.decode` is a function;
  `perInput` is pinned pointwise by `inputEvaluation_subsingleton` and
  `funext`; `observationsComplete` and `costExact` are proofs, hence
  proof-irrelevant; and `cost` is pinned by `Cost.exact_unique` applied to the
  two `ExactAggregateCost` witnesses, which agree on every argument once
  `perInput` does.

  This lemma is load-bearing for `GlobalOptimal`: its lower-bound clause
  asserts `SystemEvaluationRel S D competitorBytes competitorEval` for *every*
  `competitorEval`, which no decider could satisfy if two distinct evaluations
  of the same bytes existed.
-/
theorem systemEvaluation_subsingleton {S : Setting P} {bytes : ByteArray}
    (a b : SystemEvaluation S bytes) : a = b := by
  obtain ⟨am, adec, aper, aoc, acost, aex⟩ := a
  obtain ⟨bm, bdec, bper, boc, bcost, bex⟩ := b
  have hm : am = bm := by
    have h := adec.symm.trans bdec
    exact Except.ok.inj h
  subst hm
  have hper : aper = bper :=
    funext (fun raw => inputEvaluation_subsingleton (aper raw) (bper raw))
  subst hper
  have hcost : acost = bcost := Cost.exact_unique aex bex
  subst hcost
  rfl

instance instSubsingletonSystemEvaluation {S : Setting P} {bytes : ByteArray} :
    Subsingleton (SystemEvaluation S bytes) :=
  ⟨systemEvaluation_subsingleton⟩

/-- The decider's answer, when it is a completed evaluation, is *the*
evaluation: any inhabitant of the evaluation type is related to the bytes. -/
theorem systemEvaluationRel_of_exists {S : Setting P} {D : Decider S}
    {bytes : ByteArray} (h : ∃ e : SystemEvaluation S bytes,
      SystemEvaluationRel S D bytes e) (e' : SystemEvaluation S bytes) :
    SystemEvaluationRel S D bytes e' := by
  obtain ⟨e, hrel⟩ := h
  exact systemEvaluation_subsingleton e e' ▸ hrel

end Uniqueness

/-! ## Admissibility (SPEC §10.1)

The three extensional predicates of SPEC §10.1, packaged.  These are exactly
the premises of both competitor clauses of `GlobalOptimal`, and exactly its
first three conjuncts. -/

section Admissible

variable {P : Wasm.Profile}

/-- **SPEC §10.1.**  A byte sequence is admissible when it is profile valid,
semantically correct, and within resources.  No evaluation, certificate,
artifact or selector appears. -/
def Admissible (S : Setting P) (bytes : ByteArray) : Prop :=
  ProfileValid P bytes ∧ SemanticCorrect S bytes ∧ SemanticWithinResources S bytes

theorem admissible_profileValid {S : Setting P} {bytes : ByteArray}
    (h : Admissible S bytes) : ProfileValid P bytes := h.1

theorem admissible_semanticCorrect {S : Setting P} {bytes : ByteArray}
    (h : Admissible S bytes) : SemanticCorrect S bytes := h.2.1

theorem admissible_semanticWithinResources {S : Setting P} {bytes : ByteArray}
    (h : Admissible S bytes) : SemanticWithinResources S bytes := h.2.2

variable [Foundation.Fintype (Gemm.RawInvocation P)]

/-- Every input evaluation of a profile-valid byte sequence really does start
the costed invocation that SPEC §10.1's extensional predicates quantify over.
This is the bridge from the evaluator-side record to the extensional side. -/
theorem startsCostedInvocation_of_profileValid {S : Setting P} {bytes : ByteArray}
    (hprofile : ProfileValid P bytes) (e : SystemEvaluation S bytes)
    (raw : Gemm.RawInvocation P) :
    StartsCostedInvocation S bytes raw (e.perInput raw).initialization
      (e.perInput raw).initial := by
  obtain ⟨m, hdec, hval, _, _⟩ := hprofile
  have hm : m = e.module := profileValid_module_unique hdec e.decodeEq
  subst hm
  exact ⟨e.module, hdec, hval, (e.perInput raw).initialEq,
    (e.perInput raw).initialConfigEq⟩

/--
  **Reflection, correctness half.**  An admissible byte sequence's evaluation is
  `Correct`.

  The step that makes this work is `Wasm.isTerminalObservation`: *every*
  execution observation is maximal, so a recorded costed observation — which
  carries a `Wasm.FiniteExecution` proof — is a `Wasm.MaximalExecution`, and the
  extensional quantifier of `SemanticCorrectAt` therefore reaches it.
-/
theorem correct_of_admissible {S : Setting P} {bytes : ByteArray}
    (hadm : Admissible S bytes) (e : SystemEvaluation S bytes) : Correct e := by
  intro raw
  refine ⟨Foundation.NonemptyCanonicalFrontier.elements_ne_nil _, ?_⟩
  intro o _
  have hstart := startsCostedInvocation_of_profileValid hadm.1 e raw
  obtain ⟨obs, hobs, hacc⟩ :=
    (hadm.2.1 raw).2 (e.perInput raw).initialization (e.perInput raw).initial
      (Wasm.MaximalExecution.finite o.observation o.execution
        (Wasm.isTerminalObservation _)) ⟨hstart, trivial⟩
  have heq : o.observation = obs := hobs
  exact heq ▸ hacc

/--
  **Reflection, feasibility half.**  An admissible byte sequence's evaluation is
  `Feasible`.

  Each recorded branch is charged inside the problem's limit by
  `SemanticWithinResourcesAt` (again through `Wasm.isTerminalObservation`), the
  branch cost matched by the extensional side is *equal* to the recorded one
  because both are pinned to the trace cost by `costExact`, and the
  componentwise maximum over the branches then stays inside the limit by
  `sequentialCompose_maxOverCosts_le`.
-/
theorem feasible_of_admissible {S : Setting P} {bytes : ByteArray}
    (hadm : Admissible S bytes) (e : SystemEvaluation S bytes) : Feasible e := by
  intro raw
  have hstart := startsCostedInvocation_of_profileValid hadm.1 e raw
  have hbranch : ∀ o ∈ (e.perInput raw).observations.elements,
      Cost.DynamicVector.ComponentwiseLE
        (Cost.sequentialCompose (e.perInput raw).initialization.cost o.cost)
        S.problem.limit := by
    intro o _
    obtain ⟨costed, hcosted, hle⟩ :=
      (hadm.2.2 raw).2 (e.perInput raw).initialization (e.perInput raw).initial
        (Wasm.MaximalExecution.finite o.observation o.execution
          (Wasm.isTerminalObservation _)) ⟨hstart, trivial⟩
    have hobs : o.observation = costed.observation := hcosted
    have hcost : o.cost = costed.cost := by
      rw [o.costExact, costed.costExact, hobs]
    rw [hcost]
    exact hle
  have hhead : (e.perInput raw).observations.head ∈
      (e.perInput raw).observations.elements := by
    simp [Foundation.NonemptyCanonicalFrontier.elements]
  have hinit : Cost.DynamicVector.ComponentwiseLE
      (e.perInput raw).initialization.cost S.problem.limit :=
    Cost.DynamicVector.componentwiseLE_trans
      (Cost.sequentialCompose_le_left _ _) (hbranch _ hhead)
  rw [(e.perInput raw).resourceExact]
  refine sequentialCompose_maxOverCosts_le hinit _ ?_
  intro v hv
  obtain ⟨o, ho, hvo⟩ := List.mem_map.mp hv
  exact hvo ▸ hbranch o ho

/-- Both evaluator-side predicates at once. -/
theorem correctAndFeasible_of_admissible {S : Setting P} {bytes : ByteArray}
    (hadm : Admissible S bytes) (e : SystemEvaluation S bytes) :
    CorrectAndFeasible e :=
  ⟨correct_of_admissible hadm e, feasible_of_admissible hadm e⟩

end Admissible

/-! ## The argmin construction -/

section Argmin

variable {P : Wasm.Profile} [Foundation.Fintype (Gemm.RawInvocation P)]
  {S : Setting P} {D : Decider S} {O : Cost.ProperObjective P S.problem}

/-- `bytes` is admissible, the decider evaluates it, and the resulting score is
`n`. -/
def AchievesScore (S : Setting P) (D : Decider S)
    (O : Cost.ProperObjective P S.problem) (bytes : ByteArray) (n : Nat) : Prop :=
  Admissible S bytes ∧
  ∃ e : SystemEvaluation S bytes,
    SystemEvaluationRel S D bytes e ∧ O.score e.cost = n

/-- `n` is the score of some admissible, evaluated byte sequence. -/
def AchievableScore (S : Setting P) (D : Decider S)
    (O : Cost.ProperObjective P S.problem) (n : Nat) : Prop :=
  ∃ bytes : ByteArray, AchievesScore S D O bytes n

/--
  **Least achievable score.**  If one admissible byte sequence is evaluated by
  the decider, the set of achievable scores is a nonempty set of naturals and
  therefore has a least element.
-/
theorem exists_least_admissible_score
    (hne : ∃ b : ByteArray, Admissible S b ∧
      ∃ e : SystemEvaluation S b, SystemEvaluationRel S D b e) :
    ∃ m : Nat, AchievableScore S D O m ∧
      ∀ n : Nat, AchievableScore S D O n → m ≤ n := by
  obtain ⟨b, hadm, e, hrel⟩ := hne
  exact exists_least_nat (AchievableScore S D O) (O.score e.cost)
    ⟨b, hadm, e, hrel, rfl⟩

/--
  **Minimizers are finite.**  A byte sequence whose evaluated score is `m` has
  size at most `m`: properness bounds the `staticModuleBytes` coordinate by the
  score, and `Cost.ExactAggregateCost` pins that coordinate to `bytes.size`.
  Every minimizer therefore lies in the finite list
  `Enumerate.boundedByteArrays m`.
-/
theorem minimizer_size_le_score {bytes : ByteArray} {m : Nat}
    (h : AchievesScore S D O bytes m) : bytes.size ≤ m := by
  obtain ⟨_, e, _, hscore⟩ := h
  exact sublevel_bytes_size_le O e (Nat.le_of_eq hscore)

/-- Every minimizer is enumerated by the finite sublevel list. -/
theorem minimizer_mem_boundedByteArrays {bytes : ByteArray} {m : Nat}
    (h : AchievesScore S D O bytes m) :
    bytes ∈ Enumerate.boundedByteArrays m :=
  Enumerate.mem_boundedByteArrays (minimizer_size_le_score h)

/--
  **Canonical least minimizer.**  From nonemptiness of the admissible evaluated
  set: there is a least achievable score `m`, and among the byte sequences
  achieving it there is a canonically least one.

  Finiteness comes from `minimizer_mem_boundedByteArrays`; the selection comes
  from totality, reflexivity and transitivity of `Foundation.CanonicalBytesLE`.
-/
theorem exists_canonical_least_minimizer
    (hne : ∃ b : ByteArray, Admissible S b ∧
      ∃ e : SystemEvaluation S b, SystemEvaluationRel S D b e) :
    ∃ (m : Nat) (b : ByteArray),
      AchievesScore S D O b m ∧
      (∀ n : Nat, AchievableScore S D O n → m ≤ n) ∧
      (∀ c : ByteArray, AchievesScore S D O c m →
        Foundation.CanonicalBytesLE b c) := by
  obtain ⟨m, ⟨b₀, hb₀⟩, hleast⟩ := exists_least_admissible_score (O := O) hne
  obtain ⟨b, _, hb, hmin⟩ :=
    exists_canonical_least_in_list (Enumerate.boundedByteArrays m)
      (fun c => AchievesScore S D O c m)
      ⟨b₀, minimizer_mem_boundedByteArrays hb₀, hb₀⟩
  refine ⟨m, b, hb, hleast, ?_⟩
  intro c hc
  exact hmin c (minimizer_mem_boundedByteArrays hc) hc

/-! ### The decider hypothesis

`GlobalOptimal` asserts that the decider's answer on an admissible competitor
*is* that competitor's evaluation.  A `Decider` is an arbitrary total function
of the bytes, so this cannot be derived: nothing forbids a decider that returns
`.profileFailure` on bytes that do have an evaluation.  The weakest statement
that closes the gap only asks the decider to return *some* completed evaluation;
`systemEvaluation_subsingleton` then upgrades it to the "for every inhabitant"
form the predicate demands. -/

/-- The decider returns a completed evaluation on every admissible byte
sequence that has one. -/
def DeciderAnswersAdmissible (S : Setting P) (D : Decider S) : Prop :=
  ∀ b : ByteArray, Admissible S b → SystemEvaluation S b →
    ∃ e : SystemEvaluation S b, SystemEvaluationRel S D b e

/-- With uniqueness, "returns *some* evaluation" is "returns *the* evaluation",
for every inhabitant of the evaluation type.  This is exactly the shape
`GlobalOptimal`'s conjunctive lower-bound clause requires. -/
theorem deciderAnswersAdmissible_rel (hdec : DeciderAnswersAdmissible S D)
    {b : ByteArray} (hadm : Admissible S b) (e : SystemEvaluation S b) :
    SystemEvaluationRel S D b e :=
  systemEvaluationRel_of_exists (hdec b hadm e) e

/--
  **SPEC §13, Phase D — `GlobalOptimal` is attainable.**

  If the admissible set is nonempty (one byte sequence that is profile valid,
  semantically correct, within resources, and evaluated by the decider), and the
  decider answers on admissible bytes that have an evaluation, then some byte
  sequence satisfies `GlobalOptimal` in full: all three extensional conjuncts,
  the evaluation existential with `Correct` and `Feasible`, the universal
  lower-bound clause over **all** of `ByteArray`, and the canonical tie-break
  clause.

  This does NOT prove `Artifact.released_wasm_gemm_gnaf_global_optimal`.  That
  theorem needs, in addition, (a) a nonempty witness — a concrete module with
  proofs of the three extensional predicates — and (b) an identification proof
  that the committed release literal is the byte sequence selected here.  The
  selection below is an existential produced by classical reasoning; it names no
  literal.
-/
theorem exists_globalOptimal_of_nonempty
    (hdec : DeciderAnswersAdmissible S D)
    (hne : ∃ b : ByteArray, Admissible S b ∧
      ∃ e : SystemEvaluation S b, SystemEvaluationRel S D b e) :
    ∃ bytes : ByteArray, GlobalOptimal S D O bytes := by
  obtain ⟨m, b, ⟨hadm, e, hrel, hscore⟩, hleast, hmin⟩ :=
    exists_canonical_least_minimizer (O := O) hne
  refine ⟨b, hadm.1, hadm.2.1, hadm.2.2, e, hrel,
    correct_of_admissible hadm e, feasible_of_admissible hadm e, ?_, ?_⟩
  · intro cb hp hc hw ce
    have hcadm : Admissible S cb := ⟨hp, hc, hw⟩
    have hcrel : SystemEvaluationRel S D cb ce :=
      deciderAnswersAdmissible_rel hdec hcadm ce
    refine ⟨hcrel, ?_⟩
    have hach : AchievableScore S D O (O.score ce.cost) :=
      ⟨cb, hcadm, ce, hcrel, rfl⟩
    rw [hscore]
    exact hleast _ hach
  · intro cb hp hc hw ce hcrel heq
    refine hmin cb ⟨⟨hp, hc, hw⟩, ce, hcrel, ?_⟩
    rw [← heq]
    exact hscore

end Argmin

end WasmGemmGnaf.Universal
