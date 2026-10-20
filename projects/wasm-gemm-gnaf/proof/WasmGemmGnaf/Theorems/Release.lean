/-
  Theorems: the release theorem, at the strength it actually has.

  This module is an INDEX over `Artifact/Release.lean`.  It exists so that the
  *conditional* nature of the release result is visible at the top level rather
  than buried under a chain of abbreviations: every hypothesis is written out in
  the statement, and the scope the statement is about is collected into a single
  checkable conjunction.

  ## What is proved here

  | name | content |
  |---|---|
  | `Theorems.release_scope_identities` | every profile / problem / setting / objective identity equation of the release instance, in one conjunction |
  | `Theorems.release_decider_answers_admissible` | UV-003 at the release instance: the decider hypothesis of `Universal.exists_globalOptimal_of_nonempty` is discharged |
  | `Theorems.release_globalOptimal_of_nonempty` | **the release theorem**: nonemptiness of the admissible evaluated set implies `Universal.GlobalOptimal` at the release setting, decider and objective — full statement written out |
  | `Theorems.release_obligation_reduction` | the two forms of the remaining antecedent are interchangeable |
  | `Theorems.release_lower_bound_clause` | from `GlobalOptimal` at the release scope, the lower-bound clause at an *arbitrary* `competitorBytes : ByteArray` |
  | `Theorems.release_tie_break_clause` | the canonical-least half of the same projection |
  | `Theorems.release_competitor_universe_inhabited` | the competitor clause is not vacuous |
  | `Theorems.release_seam_nondegenerate` | **GO-008**: the constructed `Release.seam` charges every event, charges every validation, budgets every coordinate, and its machine completes with a nonempty frontier |
  | `Theorems.release_systemEvaluation_inhabited` | **GO-008's headline**: `Universal.SystemEvaluation` at the constructed setting is inhabited on a `ProfileValid` closed literal |
  | `Theorems.release_globalOptimal_of_nonempty_at_seam` | the release theorem with the seam no longer a parameter |
  | `Theorems.release_globalOptimal_of_witness_semantics` | exactly what is left of the antecedent on that literal — and it is a *false* hypothesis, not a discharge |

  ## THE NAME IS HONEST, AND HERE IS WHY IT HAS TO BE

  `Artifact.released_wasm_gemm_gnaf_global_optimal` is **not declared here and is
  not declared anywhere in this repository**.  SPEC §15 forbids that name from
  taking `Coverage`, `LowerBound`, `Correct`, `FaithfulWasm`, `CompilerCorrect`
  or `GlobalOptimal` as parameters, so it cannot be stated as a conditional; and
  the antecedent below is open.  `release_globalOptimal_of_nonempty` is named
  after its hypothesis on purpose.  A conditional theorem wearing the
  unconditional theorem's name would make every downstream citation false, and
  no amount of surrounding prose would repair it.

  ## EVERY HYPOTHESIS THAT REMAINS

  There are four, and all four are arguments of the theorems below — none is an
  axiom, a `sorry`, or a definition quietly chosen to make something true.
  Number 2 is **closed** as of GO-008: see §4 of this file and §5 of
  `Artifact/Release.lean`.  It is left in the list, marked, because the
  theorems of §2 and §3 still quantify over `(seam : Release.Seam)` and are
  therefore still stated at that generality.

  1. `[Foundation.Fintype (Gemm.RawInvocation Release.wasmProfile)]` — SPEC
     §8.4's `problem_input_fintype`.  Recorded OUTSTANDING under `O-3` in
     `Theorems/Status.lean` (`Gemm.raw_input_finite`).  Carried as an
     instance-implicit argument, exactly as `Universal/Competitor.lean` carries
     it.
  2. `(seam : Release.Seam)` — **CLOSED (GO-008)**, and still universally
     quantified in §2 and §3 because the results hold for *every* choice.
     `Release.seam` is a closed term: `Release.semantics` (`Release.costEvent`
     plus `Wasm.validationCost` and `Wasm.instantiatedStaticBytes`),
     `Release.machine` (`Wasm.releaseCostedMachine`, the real all-branch costed
     explorer), and `Release.limit` (five coordinates pinned by SPEC §8.2, the
     other eleven at the profile maximum the pinned costed-step budget
     implies).  `Theorems.release_seam_nondegenerate` proves it is not the fake
     seam — the cost model is not free, validation is not free, no coordinate is
     budgeted at zero, and the machine returns a completed **nonempty** frontier
     on a real module.  Two disclosures travel with it and are stated in §4:
     `Release.costEvent` is a *lower bound* on the SPEC §7.5 contribution law,
     and `Release.witnessModule` is not a GEMM implementation.
  3. `hne` — nonemptiness of the admissible, evaluated set.  **This is the one
     that matters, and it is still open.**  No byte sequence is proved
     `ProfileValid ∧ SemanticCorrect ∧ SemanticWithinResources` anywhere in this
     repository, and none is asserted to be.  It is an explicit hypothesis of
     every conclusion below.  Two of its four conjuncts are now theorems on one
     closed literal, `Release.witnessBytes`: `Release.witness_profileValid`
     (profile validity) and `Theorems.release_systemEvaluation_inhabited` (the
     `Universal.SystemEvaluation` inhabitant, which before GO-008 was not known
     to be satisfiable at *any* seam).  `Universal.SemanticWithinResources` is
     unproved for it and `Universal.SemanticCorrect` is **false** for it — the
     witness module's `gemm` returns the constant `0`.  `Artifact/Baseline.lean`
     proves profile validity for the compiled GEMM witness
     (`Artifact.baseline_profileValid`); its `SemanticCorrect` is
     `GNAF.compile_refines`, omitted under `BI-002`/`O-6`.  Two of four on a
     module that does not compute GEMM is not four of four, and `hne` stays a
     hypothesis here.
  4. The disclosed profile deviation: `Release.wasmProfile` is
     `Wasm.unitWitnessProfile`, not SPEC §7.2's release literal — the cost table
     is `Wasm.canonicalCostTableUnits`, whose eight units, canonical GC widths
     and audit rows are the canonical release values, but which is *not* built
     by `Wasm.buildCanonicalCostTable` from a vendored Core 3.0 conformance map,
     because neither that function nor that map exists here.  What CO-006
     reported — empty `ruleRows` and `initializationRows`, so that `rowFor?`
     returned `none` for every rule — is closed: the rows are now an exact,
     duplicate-free cover of `Wasm.RuleId` and `Wasm.InitEventId`, and
     `release_scope_identities` asserts that cover rather than asserting
     emptiness.  What remains open is only the *provenance* of the row
     contributions: they are proved equal to this repository's own contribution
     law (`Wasm.canonicalCostTable_charges_exactly`), not cross-checked against
     an external conformance map.

  `Release.decider` is **noncomputable** (`Classical.propDecidable` and
  `Classical.choice`, both Prop-level, neither on an executable path, neither
  producing a claimed executable witness).  It computes no artifact; the
  conclusion below is an existential and names no byte literal.  Identifying a
  committed release literal with the byte sequence it selects is a further,
  undischarged obligation (`Artifact.released_bytes_equal_selection`, `O-4`).

  Every declaration in this file is a proved theorem.  There is no `sorry`, no
  `admit`, no project axiom, no `native_decide`, no `unsafe`, no `partial`.
-/
import WasmGemmGnaf.Artifact.Release

set_option autoImplicit false

namespace WasmGemmGnaf.Theorems

open WasmGemmGnaf

/-! ## 1. The scope the release theorem is about

One conjunction, all `rfl`-or-near-`rfl`, so that "which profile, which problem,
which objective" is checkable in a single place instead of being reassembled
from a dozen files.  The four `ruleRows`/`initializationRows` conjuncts are the
machine-checked cost-profile statement SPEC §7.5 requires of
`buildCanonicalCostTable`'s argument — an exact, duplicate-free cover, with
every rule resolving to a row; they are part of the scope statement, not a
footnote to it. -/

/--
  **The release scope, stated in full.**

  Reading, in order: the profile body is the canonical Core 3.0 wasm32 body at
  the pinned revision commit over `Wasm.canonicalCostTableUnits`; the address
  model is 32-bit with the 65536-page limit; the cost table is the *units*
  table, and its audit rows are an exact duplicate-free cover of every pinned
  Core rule identifier and every harness initialization event; decoding costs
  one unit per byte plus one terminal unit; the problem is
  `Gemm.canonicalWGNGv1ProblemBody` at `workloadRepetitions = 1` with the
  `2 ^ 320` costed step budget; the setting forwards exactly those numbers, adds
  nothing to `Gemm.Reference.Accepts` and narrows nothing in the raw-invocation
  carrier; and the objective is `Cost.canonicalObjectiveBody` — weight one on
  every coordinate, unsigned byte-lexicographic tie order, score the plain
  coordinate sum.
-/
theorem release_scope_identities (seam : Release.Seam) :
    -- the profile (SPEC §7.2), including the disclosed deviation
    Release.wasmProfile.body =
        Wasm.canonicalCore3Wasm32ProfileBody Wasm.core3RevisionCommit
          Wasm.canonicalCostTableUnits ∧
    Release.wasmProfile.body.addressBits = 32 ∧
    Release.wasmProfile.body.maxPages = 65536 ∧
    Release.wasmProfile.costTableBody = Wasm.canonicalCostTableUnits ∧
    Release.wasmProfile.costTableBody.ruleRows.map Wasm.CostRuleRow.ruleId =
        Wasm.RuleId.all.map Wasm.RuleId.name ∧
    (Release.wasmProfile.costTableBody.ruleRows.map
      Wasm.CostRuleRow.ruleId).Nodup ∧
    (∀ r : Wasm.RuleId,
      (Release.wasmProfile.costTableBody.rowFor? r.name).isSome) ∧
    Release.wasmProfile.costTableBody.initializationRows.map
        Wasm.CostRuleRow.ruleId =
      Wasm.InitEventId.all.map Wasm.InitEventId.name ∧
    (∀ bytes : ByteArray,
      Release.wasmProfile.costTableBody.decodeCost bytes = bytes.size + 1) ∧
    -- the problem (SPEC §8.3)
    Release.gemmProblem.body =
        Gemm.canonicalWGNGv1ProblemBody Release.wasmProfile.body
          (workloadRepetitions := 1) ∧
    Release.gemmProblem.workloadRepetitions = 1 ∧
    Release.gemmProblem.maxSteps = 2 ^ 320 ∧
    -- the setting (SPEC §10.1)
    (Release.setting seam).problem = Release.problem seam ∧
    (Release.setting seam).problem.maxSteps = 2 ^ 320 ∧
    (Release.setting seam).problem.workloadRepetitions = 1 ∧
    (Release.setting seam).problem.RawInvocation =
        Gemm.RawInvocation Release.wasmProfile ∧
    (∀ (raw : Gemm.RawInvocation Release.wasmProfile)
        (observation : Wasm.ExecutionObservation),
      (Release.setting seam).problem.Accepts raw observation ↔
        Gemm.Reference.Accepts Release.gemmProblem raw observation) ∧
    -- the objective (SPEC §9.3)
    (Release.costObjective seam).body = Cost.canonicalObjectiveBody ∧
    (∀ co : Cost.ArtifactCoordinate,
      (Release.costObjective seam).body.weight co = 1) ∧
    (Release.costObjective seam).body.tieOrder = .unsignedByteLexicographic ∧
    (∀ c : Cost.CompleteSystemCost,
      (Release.costObjective seam).score c = Cost.CanonicalObjective.score c) :=
  ⟨Release.wasmProfile_body,
   Release.wasmProfile_addressBits,
   Release.wasmProfile_maxPages,
   Release.wasmProfile_costTableBody,
   Release.wasmProfile_ruleRows_exact_cover.1,
   Release.wasmProfile_ruleRows_nodup,
   Release.wasmProfile_covers_every_rule,
   Release.wasmProfile_ruleRows_exact_cover.2,
   Release.wasmProfile_decodeCost,
   Release.gemmProblem_canonical,
   Release.gemmProblem_workloadRepetitions,
   Release.gemmProblem_maxSteps,
   Release.setting_problem seam,
   Release.setting_maxSteps seam,
   Release.setting_workloadRepetitions seam,
   Release.setting_rawInvocation seam,
   Release.setting_accepts seam,
   Release.costObjective_body seam,
   Release.costObjective_weights_one seam,
   Release.costObjective_tieOrder seam,
   Release.costObjective_score seam⟩

/-! ## 2. The release theorem

Everything below needs SPEC §8.4's `problem_input_fintype`, which this
repository does not discharge (`O-3`), and the `Release.Seam` data, which it
does not have.  Both are arguments. -/

section Release

variable [Foundation.Fintype (Gemm.RawInvocation Release.wasmProfile)]

/--
  **UV-003 at the release instance.**

  `Universal.exists_globalOptimal_of_nonempty` has two hypotheses.  This is the
  one that *is* discharged: the release decider returns a completed evaluation
  on every admissible byte sequence that has one.  It is not derivable for an
  arbitrary `Universal.Decider` — nothing forbids a decider from answering
  `.profileFailure` on bytes that do have an evaluation — and it is what makes
  `release_globalOptimal_of_nonempty` rest on nonemptiness *alone*.
-/
theorem release_decider_answers_admissible (seam : Release.Seam) :
    ∀ b : ByteArray,
      Universal.Admissible (Release.setting seam) b →
      Universal.SystemEvaluation (Release.setting seam) b →
      ∃ e : Universal.SystemEvaluation (Release.setting seam) b,
        Universal.SystemEvaluationRel (Release.setting seam)
          (Release.decider seam) b e :=
  Release.deciderAnswersAdmissible seam

/--
  **The release theorem, at the strength it actually has.**

  *If* one byte sequence is profile valid, semantically correct, within
  resources and has a system evaluation, *then* some byte sequence satisfies
  `Universal.GlobalOptimal` at the release setting, the release decider and the
  release objective: all three extensional conjuncts, the evaluation
  existential with `Correct` and `Feasible`, the universal lower-bound clause
  over **all** of `ByteArray`, and the canonical tie-break clause.

  The conclusion is written out rather than abbreviated as
  `Universal.GlobalOptimal …` so that the competitor quantifier is visible: it
  is `∀ competitorBytes : ByteArray`, with no scope predicate, no registry, no
  attention index and no "known algorithms" restriction.

  **What this is not.**  It is not
  `Artifact.released_wasm_gemm_gnaf_global_optimal`.  The antecedent `hne` is
  open — this repository proves no concrete byte sequence admissible — the
  `Release.Seam` argument is data it does not have, the `Foundation.Fintype`
  instance is `O-3`, and the conclusion is an existential produced by classical
  reasoning that names no byte literal.  Cite it with its hypotheses or not at
  all.
-/
theorem release_globalOptimal_of_nonempty (seam : Release.Seam)
    (hne : ∃ b : ByteArray,
      Universal.ProfileValid Release.wasmProfile b ∧
      Universal.SemanticCorrect (Release.setting seam) b ∧
      Universal.SemanticWithinResources (Release.setting seam) b ∧
      Nonempty (Universal.SystemEvaluation (Release.setting seam) b)) :
    ∃ releasedBytes : ByteArray,
      Universal.ProfileValid Release.wasmProfile releasedBytes ∧
      Universal.SemanticCorrect (Release.setting seam) releasedBytes ∧
      Universal.SemanticWithinResources (Release.setting seam) releasedBytes ∧
      ∃ releasedEval :
          Universal.SystemEvaluation (Release.setting seam) releasedBytes,
        Universal.SystemEvaluationRel (Release.setting seam)
            (Release.decider seam) releasedBytes releasedEval ∧
        Universal.Correct releasedEval ∧
        Universal.Feasible releasedEval ∧
        (∀ competitorBytes : ByteArray,
          Universal.ProfileValid Release.wasmProfile competitorBytes →
          Universal.SemanticCorrect (Release.setting seam) competitorBytes →
          Universal.SemanticWithinResources (Release.setting seam)
            competitorBytes →
          ∀ competitorEval :
              Universal.SystemEvaluation (Release.setting seam) competitorBytes,
            Universal.SystemEvaluationRel (Release.setting seam)
                (Release.decider seam) competitorBytes competitorEval ∧
            (Release.costObjective seam).score releasedEval.cost ≤
              (Release.costObjective seam).score competitorEval.cost) ∧
        (∀ competitorBytes : ByteArray,
          Universal.ProfileValid Release.wasmProfile competitorBytes →
          Universal.SemanticCorrect (Release.setting seam) competitorBytes →
          Universal.SemanticWithinResources (Release.setting seam)
            competitorBytes →
          ∀ competitorEval :
              Universal.SystemEvaluation (Release.setting seam) competitorBytes,
            Universal.SystemEvaluationRel (Release.setting seam)
                (Release.decider seam) competitorBytes competitorEval →
            (Release.costObjective seam).score releasedEval.cost =
                (Release.costObjective seam).score competitorEval.cost →
            Foundation.CanonicalBytesLE releasedBytes competitorBytes) := by
  obtain ⟨b, hprofile, hcorrect, hresources, hev⟩ := hne
  exact Release.exists_globalOptimal_of_admissible_evaluation seam
    ⟨b, ⟨hprofile, hcorrect, hresources⟩, hev⟩

/--
  The same conclusion from the packaged antecedent, so that the theorem above
  and `Release.exists_globalOptimal_of_nonempty` are visibly the same result.
-/
theorem release_globalOptimal_of_admissible (seam : Release.Seam)
    (hne : ∃ b : ByteArray, Universal.Admissible (Release.setting seam) b ∧
      Nonempty (Universal.SystemEvaluation (Release.setting seam) b)) :
    ∃ releasedBytes : ByteArray,
      Universal.GlobalOptimal (Release.setting seam) (Release.decider seam)
        (Release.costObjective seam) releasedBytes :=
  Release.exists_globalOptimal_of_admissible_evaluation seam hne

/--
  **The remaining obligation, stated as an equivalence.**

  Because the release decider relates every byte sequence to every evaluation it
  has, the "decider agrees" form of the antecedent and the "has an evaluation"
  form are interchangeable.  This is the precise content of the claim that the
  release theorem now rests on nonemptiness alone: there is nothing left in the
  antecedent except exhibiting one admissible, evaluated byte sequence — which
  this repository does **not** do.
-/
theorem release_obligation_reduction (seam : Release.Seam) :
    (∃ b : ByteArray, Universal.Admissible (Release.setting seam) b ∧
        ∃ e : Universal.SystemEvaluation (Release.setting seam) b,
          Universal.SystemEvaluationRel (Release.setting seam)
            (Release.decider seam) b e) ↔
      (∃ b : ByteArray, Universal.Admissible (Release.setting seam) b ∧
        Nonempty (Universal.SystemEvaluation (Release.setting seam) b)) :=
  Release.nonemptiness_iff_admissible_evaluation seam

/-! ## 3. The competitor quantifier really ranges over all of `ByteArray`

`GlobalOptimal` is only worth proving if its universal clause is universal.  The
two projections below take an arbitrary `competitorBytes : ByteArray` — bound at
the top of the statement, with no scope predicate anywhere — and produce the
lower-bound and tie-break clauses at it.  If the definition had narrowed the
quantifier to a registry, a sublevel, a plan family or an attention index, these
would not typecheck. -/

/--
  **The universal lower-bound clause, extracted at an arbitrary competitor.**

  Given a proof of `GlobalOptimal` at the release scope and *any* byte sequence
  whatever that is profile valid, semantically correct and within resources: the
  released evaluation exists, is the release decider's own answer, is `Correct`
  and `Feasible`, and its score is `≤` the score of every evaluation of that
  competitor — which the decider also answers.

  `competitorBytes` is an ordinary universally quantified `ByteArray`.  That is
  the whole point of this declaration: it documents, in a form the kernel
  checks, that the release instance's competitor universe is every finite byte
  sequence.
-/
theorem release_lower_bound_clause (seam : Release.Seam)
    {releasedBytes : ByteArray}
    (h : Universal.GlobalOptimal (Release.setting seam) (Release.decider seam)
      (Release.costObjective seam) releasedBytes)
    (competitorBytes : ByteArray)
    (hprofile : Universal.ProfileValid Release.wasmProfile competitorBytes)
    (hcorrect : Universal.SemanticCorrect (Release.setting seam) competitorBytes)
    (hresources : Universal.SemanticWithinResources (Release.setting seam)
      competitorBytes) :
    ∃ releasedEval :
        Universal.SystemEvaluation (Release.setting seam) releasedBytes,
      Universal.SystemEvaluationRel (Release.setting seam)
          (Release.decider seam) releasedBytes releasedEval ∧
      Universal.Correct releasedEval ∧
      Universal.Feasible releasedEval ∧
      ∀ competitorEval :
          Universal.SystemEvaluation (Release.setting seam) competitorBytes,
        Universal.SystemEvaluationRel (Release.setting seam)
            (Release.decider seam) competitorBytes competitorEval ∧
        (Release.costObjective seam).score releasedEval.cost ≤
          (Release.costObjective seam).score competitorEval.cost :=
  Universal.globalOptimal_lower_bound_at h competitorBytes hprofile hcorrect
    hresources

/--
  **The tie-break clause, extracted at an arbitrary competitor.**  Same
  quantifier, canonical-least half: an equal-scoring competitor is never
  canonically smaller than the released bytes.
-/
theorem release_tie_break_clause (seam : Release.Seam)
    {releasedBytes : ByteArray}
    (h : Universal.GlobalOptimal (Release.setting seam) (Release.decider seam)
      (Release.costObjective seam) releasedBytes)
    (competitorBytes : ByteArray)
    (hprofile : Universal.ProfileValid Release.wasmProfile competitorBytes)
    (hcorrect : Universal.SemanticCorrect (Release.setting seam) competitorBytes)
    (hresources : Universal.SemanticWithinResources (Release.setting seam)
      competitorBytes) :
    ∃ releasedEval :
        Universal.SystemEvaluation (Release.setting seam) releasedBytes,
      Universal.SystemEvaluationRel (Release.setting seam)
          (Release.decider seam) releasedBytes releasedEval ∧
      ∀ competitorEval :
          Universal.SystemEvaluation (Release.setting seam) competitorBytes,
        Universal.SystemEvaluationRel (Release.setting seam)
            (Release.decider seam) competitorBytes competitorEval →
        (Release.costObjective seam).score releasedEval.cost =
            (Release.costObjective seam).score competitorEval.cost →
        Foundation.CanonicalBytesLE releasedBytes competitorBytes :=
  Universal.globalOptimal_canonical_least_at h competitorBytes hprofile hcorrect
    hresources

/--
  **Non-vacuity.**  The competitor premises are satisfied by the released bytes
  themselves, so the universal clause above is not true merely because its
  premises are unsatisfiable.
-/
theorem release_competitor_universe_inhabited (seam : Release.Seam)
    {releasedBytes : ByteArray}
    (h : Universal.GlobalOptimal (Release.setting seam) (Release.decider seam)
      (Release.costObjective seam) releasedBytes) :
    ∃ competitorBytes : ByteArray,
      Universal.ProfileValid Release.wasmProfile competitorBytes ∧
      Universal.SemanticCorrect (Release.setting seam) competitorBytes ∧
      Universal.SemanticWithinResources (Release.setting seam) competitorBytes :=
  Universal.globalOptimal_competitor_universe_inhabited h

end Release

/-! ## 4. The seam is constructed (GO-008)

Everything in §2 and §3 takes `(seam : Release.Seam)` as an argument.  Until
`Release.seam` existed, nothing inhabited that type, so every one of those
statements was an implication whose *type-level* antecedent might have been
unsatisfiable — and the nonemptiness antecedent `hne` was worse than open, it
was not even known to be satisfiable at any seam.

`Artifact/Release.lean` §5 now builds one:

* `Release.semantics` — `Release.costEvent` (the pinned per-rule row that each
  plain event determines on its own), `Wasm.validationCost`,
  `Wasm.instantiatedStaticBytes`;
* `Release.machine` — `Wasm.releaseCostedMachine`, the real all-branch costed
  explorer of `Wasm/CostedExplore.lean`;
* `Release.limit` — five coordinates pinned verbatim by
  `Gemm.releaseResourceContract`, eleven supplied at the profile maximum the
  pinned costed-step budget implies (`Release.limit_pinned`,
  `Release.limit_unpinned` say which are which).

The statements below are about that constructed seam.  **Two disclosures travel
with them.**  `Release.costEvent` is a *lower bound* on SPEC §7.5's contribution
law, not the law: a plain `Wasm.Event` does not carry the transferred byte
count, the grow delta or the installed byte count, so
`Release.costEvent_le_eventContribution` is an inequality and not an equation.
And `Release.witnessModule` — the module on which the machine is proved to
complete — is the smallest module `GNAF.moduleOf` emits, whose `gemm` returns
the constant `0`; it is **not** `Artifact.baselineModule` and it computes no
product.  See `Artifact/Release.lean` §5.9 for why the compiled GEMM witness
cannot be discharged here. -/

/--
  **Non-degeneracy of the constructed seam.**

  Five facts, in one conjunction: some event's charge is not the zero vector
  (and is exactly the pinned dispatch row); *every* event is charged at least
  one rule step; every module costs at least one validation step; every one of
  the sixteen resource coordinates has a positive budget; and on
  `Release.witnessModule` the costed machine both initializes and returns a
  completed, nonempty, canonically ordered frontier — at *every* raw invocation.

  The last conjunct is the one that matters: a `Seam` whose machine answered
  `.initializationFailure` unconditionally would typecheck and would leave
  `Universal.SystemEvaluation` uninhabited, which is exactly the degenerate
  inhabitant this theorem rules out.
-/
theorem release_seam_nondegenerate :
    (∃ e : Wasm.Event,
        (Release.seam.semantics.costEvent e).charge ≠ Cost.DynamicVector.zero ∧
        (Release.seam.semantics.costEvent e).charge =
          Wasm.dispatchCharge Release.wasmProfile.costTableBody) ∧
    (∀ e : Wasm.Event,
      0 < (Release.seam.semantics.costEvent e).charge.wasmRuleSteps) ∧
    (∀ m : Wasm.Module, 0 < Release.seam.semantics.validationSteps m) ∧
    (∀ dc : Cost.DynamicCoordinate, 0 < dc.value Release.seam.limit) ∧
    (∀ raw : Gemm.RawInvocation Release.wasmProfile,
      Release.seam.machine.initialGemmInvocationCosted Release.witnessModule
          (Universal.toWasmInvocation raw) =
        .ok { initial := Release.witnessInitial raw
              cost := Wasm.initializationCost Release.wasmProfile
                        Release.witnessModule } ∧
      ∃ (frontier : Foundation.NonemptyCanonicalFrontier
            (Universal.CostedExecutionObservation Release.semantics
              (Release.witnessInitial raw)))
        (coverage : Universal.CostedCoverage Release.semantics
            (Release.setting Release.seam).problem.maxSteps
            (Release.witnessInitial raw) frontier),
        Release.seam.machine.exploreAllCosted
            (Release.setting Release.seam).problem.maxSteps
            (Release.witnessInitial raw) = .complete frontier coverage) :=
  ⟨Release.seam_costEvent_not_zero, Release.seam_costEvent_charge_pos,
   Release.seam_validationSteps_pos, Release.limit_pos,
   Release.seam_machine_completes⟩

section AtSeam

variable [Foundation.Fintype (Gemm.RawInvocation Release.wasmProfile)]

/--
  **GO-008's headline, at top level.**

  `Universal.SystemEvaluation` at the *constructed* release setting is
  inhabited, on a closed byte literal that is also `Universal.ProfileValid`.

  This is what defeats vacuity.  It does **not** say the literal is
  `Universal.Admissible`: `Universal.SemanticCorrect` demands
  `Gemm.Reference.Accepts` at every raw invocation, which is false for a module
  whose `gemm` returns `0`, and is not claimed.  Two of the four conjuncts of
  the release antecedent are now theorems on one literal; two are not.
-/
theorem release_systemEvaluation_inhabited :
    ∃ bytes : ByteArray,
      Universal.ProfileValid Release.wasmProfile bytes ∧
      Nonempty (Universal.SystemEvaluation (Release.setting Release.seam) bytes) :=
  ⟨Release.witnessBytes, Release.witness_profileValid,
    ⟨Release.witnessSystemEvaluation⟩⟩

/-- **UV-003 at the constructed seam.** -/
theorem release_decider_answers_admissible_at_seam :
    ∀ b : ByteArray,
      Universal.Admissible (Release.setting Release.seam) b →
      Universal.SystemEvaluation (Release.setting Release.seam) b →
      ∃ e : Universal.SystemEvaluation (Release.setting Release.seam) b,
        Universal.SystemEvaluationRel (Release.setting Release.seam)
          (Release.decider Release.seam) b e :=
  Release.deciderAnswersAdmissible Release.seam

/--
  **The release theorem at the constructed seam.**

  Same content as `release_globalOptimal_of_nonempty`, with the seam no longer
  a parameter: the setting, the decider and the objective are closed terms of
  this repository.  The antecedent `hne` is still open, and is still the only
  thing left.
-/
theorem release_globalOptimal_of_nonempty_at_seam
    (hne : ∃ b : ByteArray,
      Universal.ProfileValid Release.wasmProfile b ∧
      Universal.SemanticCorrect (Release.setting Release.seam) b ∧
      Universal.SemanticWithinResources (Release.setting Release.seam) b ∧
      Nonempty (Universal.SystemEvaluation (Release.setting Release.seam) b)) :
    ∃ releasedBytes : ByteArray,
      Universal.GlobalOptimal (Release.setting Release.seam)
        (Release.decider Release.seam) (Release.costObjective Release.seam)
        releasedBytes := by
  obtain ⟨b, hprofile, hcorrect, hresources, hev⟩ := hne
  exact Release.exists_globalOptimal_of_admissible_evaluation Release.seam
    ⟨b, ⟨hprofile, hcorrect, hresources⟩, hev⟩

/--
  **Exactly what is left, on a closed literal, at the constructed seam.**

  Read the hypotheses first.  `hcorrect` and `hresources` are conjuncts (b) and
  (c) of the release antecedent at `Release.witnessBytes`; this repository
  supplies neither, and `hcorrect` is in fact *false* for the witness module,
  which returns the constant `0`.  What the theorem records is that conjuncts
  (a) and (d) — profile validity and the system evaluation — are no longer
  hypotheses of anything.

  Citing it as evidence that a globally optimal artifact exists would be citing
  a false hypothesis as a fact.  **GO-006 is open.**
-/
theorem release_globalOptimal_of_witness_semantics
    (hcorrect : Universal.SemanticCorrect (Release.setting Release.seam)
      Release.witnessBytes)
    (hresources : Universal.SemanticWithinResources (Release.setting Release.seam)
      Release.witnessBytes) :
    ∃ releasedBytes : ByteArray,
      Universal.GlobalOptimal (Release.setting Release.seam)
        (Release.decider Release.seam) (Release.costObjective Release.seam)
        releasedBytes :=
  Release.globalOptimal_of_witness_semantics hcorrect hresources

end AtSeam

end WasmGemmGnaf.Theorems
