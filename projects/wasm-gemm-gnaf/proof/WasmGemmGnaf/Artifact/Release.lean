/-
  Artifact/Release.lean --- the concrete release instantiation, and UV-003.

  Normative sources: SPEC.md sections 1 (`GlobalOptimal`), 7.2 (the release
  profile), 8.3 (the release problem), 9.3 (the release objective), 10.1 (the
  competitor universe and `Universal.evaluate`) and 13 (Phase D).

  ## WHAT THIS FILE PROVES

  It discharges obligation **UV-003**: the decider hypothesis
  `Universal.DeciderAnswersAdmissible`, which `Universal/Argmin.lean` isolated
  and could not derive.  With it discharged,
  `Release.exists_globalOptimal_of_nonempty` reduces the release theorem to a
  single remaining obligation — nonemptiness of the admissible, evaluated set.

  It also proves that the release relation `Universal.SystemEvaluationRel`,
  instantiated at this decider, satisfies exactly the three properties SPEC §1
  demands of it: **sound**, **functional**, and **complete for every profile
  valid, semantically correct, resource bounded module that has an evaluation**.

  ## THE PRICE, STATED PLAINLY

  `Release.decider` is **noncomputable**.  It is defined by
  `Classical.propDecidable` on `Nonempty (Universal.SystemEvaluation …)` and by
  `Classical.choice` on the inhabitant.  Consequently:

  * It is **not** SPEC §10.1's "implemented finite decoder, validator, input
    enumerator and all-branch explorer".  It names no procedure, enumerates
    nothing, explores nothing, and cannot be run.
  * It **computes no artifact**.  `exists_globalOptimal_of_nonempty` yields an
    existential; `Release.decider` contributes nothing towards exhibiting a
    concrete winning byte sequence, and nothing here may be read as an emitted
    release.
  * What it does satisfy is the *relational* contract — soundness,
    functionality, completeness — and that relational contract is precisely
    what `Universal.GlobalOptimal` consumes.  `GlobalOptimal` never asks the
    decider to be executable; it asks that the decider's answer on an
    admissible competitor *be* that competitor's evaluation.

  That distinction is the difference between the theorem being provable and an
  artifact being emittable.  This file achieves the first and does not touch
  the second.

  `Classical.choice` is used, deliberately and only, in
  `Release.evaluateClassically` (and hence in `Release.decider`, which is a
  one-field wrapper around it).  Neither is on an executable path, and neither
  is claimed to produce an executable witness.  Every other declaration in this
  file is a definition or a proved theorem; there is no `sorry`, no `admit`, no
  project axiom, no `native_decide`, no `unsafe` and no `partial`.

  ## THE SEAM IS CONSTRUCTED (GO-008)

  §5 below builds a `Release.Seam` and proves it is not degenerate.  Items 2, 3
  and 4 of what used to be this file's hypothesis list are now definitions:

  * `Release.semantics` — `Release.costEvent` together with
    `Wasm.validationCost` and `Wasm.instantiatedStaticBytes`, assembled by
    `Release.semanticsFor`.
  * `Release.machine` — `Wasm.releaseCostedMachine` of `Wasm/CostedExplore.lean`:
    `Wasm.initialGemmInvocationCosted` and the bounded all-branch costed
    explorer `Wasm.exploreAllCosted`.
  * `Release.limit` — five coordinates pinned verbatim from
    `Gemm.releaseResourceContract`, eleven supplied at the profile maximum the
    pinned costed-step budget implies.  `Release.limit_pinned` and
    `Release.limit_unpinned` say which are which.
  * `Release.seam` — the three of them, as a closed term.

  Non-degeneracy is proved, not asserted:

  * `Release.seam_costEvent_not_zero` and `Release.seam_costEvent_charge_pos` —
    the cost model is not free.
  * `Release.seam_validationSteps_pos` — validation is not free.
  * `Release.seam_machine_completes` — on `Release.witnessModule` the machine
    initializes and returns a **completed, nonempty, canonically ordered**
    costed frontier at every raw invocation.
  * `Release.systemEvaluation_inhabited` — `Universal.SystemEvaluation` at
    `Release.setting Release.seam` is inhabited, on the closed literal
    `Release.witnessBytes`, which is also `Universal.ProfileValid`.

  ## THREE DISCLOSURES THAT TRAVEL WITH THE CONSTRUCTION

  1. **`Release.costEvent` is a lower bound, not SPEC §7.5's contribution law.**
     `Universal.Semantics.costEvent` maps a *plain* `Wasm.Event`, and a plain
     event does not carry the transferred byte count, the `memory.grow` delta or
     the installed byte count — `Wasm.Event.step` alone is the erasure of
     fourteen distinct rules.  `Release.costEvent` charges the pinned rule-step
     row for every event and the pinned dispatch row for the events whose
     dispatch is visible.  `Release.costEvent_le_eventContribution` proves the
     charge is componentwise **≤** `Wasm.eventContribution`, and
     `Release.costEvent_charge_wasmRuleSteps` proves it is never zero;
     `Release.costEvent_branch_charge` and `Release.costEvent_trap_charge` prove
     it is *exact* on the branch and trap rules.  It is not exact on arithmetic,
     memory access, `memory.grow`, `enterGemm`, `throw` or `return`.

  2. **`Release.witnessModule` is not a GEMM implementation, and is not
     `Artifact.baselineModule`.**  It is the smallest module `GNAF.moduleOf`
     emits — same shape, same pinned ABI type, same two exports, same
     single-page memory — whose body is `i32.const 0`.  Every branch of it
     terminates in at most three reduction steps, which is what makes the
     explorer dischargeable.  Discharging the machine on the compiled GEMM
     witness would require a termination proof for `GNAF.bodyCode`'s loops
     inside a `2 ^ 320` step budget, which this repository does not have and
     which no `decide` can supply at that fuel.  Everything proved about
     `Release.witnessModule` is about `Release.witnessModule`.

  3. **`Universal.Admissible` is still open.**  `Release.witness_profileValid`
     proves conjunct (a) and `Release.systemEvaluation_inhabited` proves
     conjunct (d), both on a closed literal.  `Universal.SemanticCorrect` —
     conjunct (b) — demands `Gemm.Reference.Accepts` at *every* raw invocation
     and is **false** for a module whose `gemm` returns the constant `0`; it is
     not claimed here, and `Release.globalOptimal_of_witness_semantics` takes it
     as a hypothesis exactly because it is one.  `GO-006` is open.

  ## WHAT REMAINS A HYPOTHESIS, AND WHY

  One thing, and it is not faked.

  `[Foundation.Fintype (Gemm.RawInvocation Release.wasmProfile)]` — SPEC §8.4's
  `problem_input_fintype`.  `Universal/Competitor.lean` already takes it as a
  hypothesis; `Theorems/GemmTotal.lean` records it as **omitted** under `O-3`
  (the carrier is mathematically finite, but no duplicate-free covering `List`
  with its `Nodup` and coverage proofs exists here).  It is carried here as an
  instance-implicit argument, exactly as upstream, and everything in §3 and
  §5.12 — including `Release.systemEvaluation_inhabited` — carries it.

  ## A DISCLOSED DEVIATION IN THE PROFILE

  SPEC §7.2 fixes `Release.wasmProfile` as
  `Wasm.Profile.checked (Wasm.canonicalCore3Wasm32ProfileBody … Release.wasmCostTableBody)`,
  and SPEC §7.5 builds `Release.wasmCostTableBody` with
  `Wasm.buildCanonicalCostTable` from the vendored Core 3.0 conformance map.
  `Wasm.buildCanonicalCostTable`, `Release.core3RuleConformanceMap`,
  `Release.canonicalRuleContribution` and
  `Release.canonicalInitializationContribution` **do not exist** in this
  repository, and `vendor/wasm-spec` contains no conformance data.

  `Release.wasmProfile` below is therefore `Wasm.unitWitnessProfile`: the same
  `Wasm.canonicalCore3Wasm32ProfileBody` at the same pinned
  `Wasm.core3RevisionCommit`, with `Wasm.canonicalCostTableUnits` in place of
  the release cost table.  Every projection SPEC §7.2 names — address bits,
  page limit, invocation-byte ceiling, feature table, closed import policy,
  required exports, resource limits, nondeterminism policy, the three semantics
  identities, and the eight cost *units* — is the canonical release value.

  `ruleRows` and `initializationRows` are **no longer empty** (that was CO-006):
  `Wasm.canonicalCostTableUnits` now carries one row per `Wasm.RuleId` and one
  per `Wasm.InitEventId`, proved to be a duplicate-free exact cover
  (`Wasm.canonicalCostTable_exact_cover`,
  `Wasm.canonicalCostTable_ruleRows_nodup`,
  `Wasm.canonicalCostTable_covers_every_rule`) and proved to charge exactly what
  the contribution law charges (`Wasm.canonicalCostTable_charges_exactly`).

  What still differs from SPEC §7.2 is the *provenance* of those rows: they are
  written here and checked against this repository's own `eventContribution`,
  not derived by `Wasm.buildCanonicalCostTable` from a vendored Core 3.0
  conformance map, which does not exist here.  This is disclosed, not hidden:
  the profile below is **not** SPEC §7.2's release literal, and no theorem in
  this file should be cited as if it were.  When the conformance-map layer
  lands, exactly one definition — `Release.wasmProfile` — changes, and nothing
  else in this file does.
-/
import WasmGemmGnaf.Universal.Argmin
import WasmGemmGnaf.Cost.CanonicalObjective
import WasmGemmGnaf.Wasm.Costed
import WasmGemmGnaf.Wasm.CostedExplore
import WasmGemmGnaf.Gemm.Problem
import WasmGemmGnaf.Gemm.Reference
import WasmGemmGnaf.GNAF.CompileCorrect

set_option autoImplicit false

namespace WasmGemmGnaf.Release

open WasmGemmGnaf

/-! ## 1. The release scope objects

`Release.gemmProblemBodyFor` / `Release.gemmProblemFor` already live in
`Gemm/Problem.lean`, in this namespace, as the profile-indexed family SPEC §8.3
pins.  Nothing about them is redefined here; the profile is supplied and they
are instantiated. -/

/-! ### 1.1 The profile (SPEC §7.2) -/

/-- SPEC §7.2's profile body, at the pinned Core 3.0 revision commit.

**Disclosed deviation**: the cost table is `Wasm.canonicalCostTableUnits`, not
SPEC §7.5's `Release.wasmCostTableBody`, which cannot be built here.  See the
file header. -/
def wasmProfileBody : Wasm.ProfileBody :=
  Wasm.canonicalCore3Wasm32ProfileBody Wasm.core3RevisionCommit
    Wasm.canonicalCostTableUnits

/-- The checked release profile: the canonical Core 3.0 wasm32 body together
with its lawfulness proof.  It is `Wasm.unitWitnessProfile`, reused rather than
rebuilt. -/
def wasmProfile : Wasm.Profile := Wasm.unitWitnessProfile

/-- **SPEC §7.2**, `Release.wasmProfile_body`. -/
theorem wasmProfile_body : wasmProfile.body = wasmProfileBody := rfl

/-- The cost table actually carried by the profile above.  Stated so the
deviation is checkable, not merely described. -/
theorem wasmProfile_costTableBody :
    wasmProfile.costTableBody = Wasm.canonicalCostTableUnits := rfl

/-- **SPEC §7.5, the audit rows.**  The profile's cost table carries an exact,
duplicate-free cover of every pinned Core rule identifier and of every harness
initialization event identifier.  This was CO-006: the rows used to be empty,
so `rowFor?` returned `none` for every rule and every per-rule charge collapsed
to the uniform `ruleStepUnit`. -/
theorem wasmProfile_ruleRows_exact_cover :
    wasmProfile.costTableBody.ruleRows.map Wasm.CostRuleRow.ruleId =
        Wasm.RuleId.all.map Wasm.RuleId.name ∧
      wasmProfile.costTableBody.initializationRows.map Wasm.CostRuleRow.ruleId =
        Wasm.InitEventId.all.map Wasm.InitEventId.name :=
  ⟨Wasm.canonicalCostTable_exact_cover_ids,
   Wasm.canonicalCostTable_init_exact_cover_ids⟩

/-- Every pinned Core rule identifier resolves to a row on the release
profile. -/
theorem wasmProfile_covers_every_rule (r : Wasm.RuleId) :
    (wasmProfile.costTableBody.rowFor? r.name).isSome :=
  Wasm.canonicalCostTable_covers_every_rule r

/-- The rule identifiers of the release profile are duplicate free, so no rule
is charged two different costs. -/
theorem wasmProfile_ruleRows_nodup :
    (wasmProfile.costTableBody.ruleRows.map Wasm.CostRuleRow.ruleId).Nodup :=
  Wasm.canonicalCostTable_ruleRows_nodup

/-- SPEC §7.2's address model, on the release profile. -/
theorem wasmProfile_addressBits : wasmProfile.body.addressBits = 32 := rfl

/-- SPEC §7.2's page limit, on the release profile. -/
theorem wasmProfile_maxPages : wasmProfile.body.maxPages = 65536 := rfl

/-- SPEC §7.5's decode cost, on the release profile: one unit per byte plus one
terminal unit. -/
theorem wasmProfile_decodeCost (bytes : ByteArray) :
    wasmProfile.costTableBody.decodeCost bytes = bytes.size + 1 :=
  Wasm.unitWitnessProfile_decodeCost bytes

/-! ### 1.2 The problem (SPEC §8.3) -/

/-- **SPEC §8.3**, `Release.gemmProblemBody`: the canonical WGNG-v1 body over
the release profile, with `workloadRepetitions = 1`. -/
def gemmProblemBody : Gemm.ProblemBody wasmProfile.body :=
  gemmProblemBodyFor wasmProfile

/-- **SPEC §8.3**, `Release.gemmProblem`. -/
def gemmProblem : Gemm.Problem wasmProfile := gemmProblemFor wasmProfile

/-- **SPEC §8.3**, `Release.gemmProblem_body`. -/
theorem gemmProblem_body : gemmProblem.body = gemmProblemBody := rfl

/-- The released body is literally `Gemm.canonicalWGNGv1ProblemBody` at
`workloadRepetitions = 1`. -/
theorem gemmProblem_canonical :
    gemmProblem.body =
      Gemm.canonicalWGNGv1ProblemBody wasmProfile.body (workloadRepetitions := 1) :=
  rfl

/-- SPEC §9.2: the workload is charged exactly once. -/
theorem gemmProblem_workloadRepetitions :
    gemmProblem.workloadRepetitions = 1 := rfl

/-- SPEC §8.2's costed step budget, on the released problem. -/
theorem gemmProblem_maxSteps : gemmProblem.maxSteps = 2 ^ 320 := rfl

/-! ### 1.3 The objective (SPEC §9.3)

The objective is `Cost.canonicalObjective`, whose body assigns weight one to
every constructor of `Cost.ArtifactCoordinate` and whose tie order is
`.unsignedByteLexicographic`.  It is indexed by the *universal* problem of the
setting, so it is defined after §2 below. -/

/-! ## 2. The release setting (SPEC §10.1)

`Universal.Setting` has three fields.  One of them — the problem — is fully
determined by §1 and `Gemm.Reference.Accepts`.  The other two are the seams
`Universal/Competitor.lean` documents at its head.  They are carried as
*parameters* through §2–§4, together with the per-invocation resource limit
vector, so that every result below holds for **every** costed semantics, costed
machine and limit; §5 then supplies one of each and proves the supplied triple
non-degenerate, so the parametrised results are not vacuous. -/

/-- **SPEC §7.5's two static cost counters, supplied.**  Given an event charge,
this assembles the whole `Universal.Semantics` at the release profile from
`Wasm.validationCost` and `Wasm.instantiatedStaticBytes`.  §5.1 supplies the
event charge (`Release.costEvent`) and §5.2 instantiates this at it. -/
def semanticsFor (costEvent : Wasm.Event → Cost.Event) :
    Universal.Semantics wasmProfile where
  costEvent := costEvent
  validationSteps := Wasm.validationCost wasmProfile.costTableBody
  staticDataBytes := Wasm.instantiatedStaticBytes wasmProfile

/-- The validation counter of `Release.semanticsFor` is SPEC §7.5's
`Wasm.validationCost`, and it is never zero. -/
theorem semanticsFor_validationSteps (costEvent : Wasm.Event → Cost.Event)
    (m : Wasm.Module) :
    (semanticsFor costEvent).validationSteps m =
        Wasm.validationCost wasmProfile.costTableBody m ∧
      0 < (semanticsFor costEvent).validationSteps m :=
  ⟨rfl, Wasm.validationCost_pos wasmProfile m⟩

/-- The static-data counter of `Release.semanticsFor` is SPEC §7.5's
`Wasm.instantiatedStaticBytes`. -/
theorem semanticsFor_staticDataBytes (costEvent : Wasm.Event → Cost.Event)
    (m : Wasm.Module) :
    (semanticsFor costEvent).staticDataBytes m =
      Wasm.instantiatedStaticBytes wasmProfile m := rfl

/--
  The three pieces of the release setting that `Universal/Competitor.lean`
  leaves as seams.

  This is data only; it carries no proposition, and in particular no
  correctness, coverage, feasibility or optimality claim.  §2–§4 quantify over
  it so that the release results hold for every inhabitant; §5 constructs
  `Release.seam` and proves it non-degenerate, which is what stops those results
  from being implications with an unsatisfiable antecedent.
-/
structure Seam where
  /-- SPEC §7.5's costed semantics: the event charge and the two static
  counters.  Supplied by `Release.semantics`. -/
  semantics : Universal.Semantics wasmProfile
  /-- SPEC §10.1's costed initialization and costed all-branch explorer.
  Supplied by `Release.machine` from `Wasm/CostedExplore.lean`. -/
  machine : Universal.CostedMachine semantics
  /-- SPEC §8.2's per-invocation resource limit, as a sixteen-coordinate
  dynamic vector.  The release resource contract pins six of the sixteen. -/
  limit : Cost.DynamicVector

section Parametric

variable (seam : Seam)

/-- **SPEC §10.1**, the release `Universal.Problem`: the released step budget,
the released workload multiplicity, and `Gemm.Reference.Accepts` at the
released GEMM problem.  The reference relation is a field of the *problem*, as
SPEC §8.4 requires, and is never artifact-specific. -/
def problem : Universal.Problem wasmProfile where
  maxSteps := gemmProblem.maxSteps
  limit := seam.limit
  workloadRepetitions := gemmProblem.workloadRepetitions
  Accepts := Gemm.Reference.Accepts gemmProblem

/-- **SPEC §10.1**, the release `Universal.Setting`. -/
def setting : Universal.Setting wasmProfile where
  semantics := seam.semantics
  machine := seam.machine
  problem := problem seam

@[simp] theorem setting_problem : (setting seam).problem = problem seam := rfl

/-- The setting's step budget is the released one. -/
theorem setting_maxSteps : (setting seam).problem.maxSteps = 2 ^ 320 := rfl

/-- The setting's workload multiplicity is the released one. -/
theorem setting_workloadRepetitions :
    (setting seam).problem.workloadRepetitions = 1 := rfl

/-- The setting's acceptance relation is exactly `Gemm.Reference.Accepts` at the
released problem: the setting adds nothing to it and removes nothing from it. -/
theorem setting_accepts (raw : Gemm.RawInvocation wasmProfile)
    (observation : Wasm.ExecutionObservation) :
    (setting seam).problem.Accepts raw observation ↔
      Gemm.Reference.Accepts gemmProblem raw observation :=
  Iff.rfl

/-- The setting's raw-invocation carrier is the fixed one: no narrowing of the
input domain. -/
theorem setting_rawInvocation :
    (setting seam).problem.RawInvocation = Gemm.RawInvocation wasmProfile := rfl

/-! ### The objective, instantiated -/

/-- **SPEC §9.3**, `Release.costObjective`: the canonical proper objective at
the release profile and the release setting's problem. -/
def costObjective : Cost.ProperObjective wasmProfile (setting seam).problem :=
  Cost.canonicalObjective wasmProfile (setting seam).problem

/-- **SPEC §9.3.**  The objective's body is the canonical release body. -/
theorem costObjective_body :
    (costObjective seam).body = Cost.canonicalObjectiveBody := rfl

/-- **SPEC §9.3**, `Release.costObjective_weights_one`: weight one on every
constructor of `Cost.ArtifactCoordinate`. -/
theorem costObjective_weights_one (co : Cost.ArtifactCoordinate) :
    (costObjective seam).body.weight co = 1 :=
  Cost.canonicalObjectiveBody_weight co

/-- **SPEC §9.3.**  The tie order is the unsigned byte-lexicographic order. -/
theorem costObjective_tieOrder :
    (costObjective seam).body.tieOrder = .unsignedByteLexicographic := rfl

/-- The canonical score is the plain coordinate sum: no coordinate is
discounted and none is free. -/
theorem costObjective_score (c : Cost.CompleteSystemCost) :
    (costObjective seam).score c = Cost.CanonicalObjective.score c :=
  Cost.canonicalObjective_score wasmProfile (setting seam).problem c

/-! ## 3. The release decider, and UV-003

Everything below needs SPEC §8.4's `problem_input_fintype`, which this
repository does not discharge (`O-3`).  It is carried as an instance-implicit
hypothesis, exactly as `Universal/Competitor.lean` carries it. -/

section Decider

variable [Foundation.Fintype (Gemm.RawInvocation wasmProfile)]

/-- The canonical site named by the decider's failure report.  It is a plain
identifier: the failure branch carries a located, payload-bearing report as
SPEC §6.1 requires, and asserts nothing. -/
def noEvaluationSite : Foundation.CanonicalObjectId where
  schemaVersion := 1
  domain := .artifact
  typeTag := Foundation.Bytes.pack [0x52, 0x45, 0x4c, 0x45, 0x41, 0x53, 0x45]
  canonicalBodyBytes := Foundation.Bytes.pack []

/-- The report the classical decider returns when the bytes have no system
evaluation at all.  `Universal.EvaluationResult` offers no "no evaluation
exists" constructor, so the located failure constructor is used; the theorems
below constrain only the `.complete` branch, and nothing here claims the bytes
failed profile validation for any particular reason. -/
def noEvaluationReport : Foundation.FailureReport where
  site := noEvaluationSite
  code := 1
  context := []

/--
  The classical evaluator.

  On `bytes` that have a `Universal.SystemEvaluation`, it returns `.complete`
  of one — necessarily *the* one, since `Universal.systemEvaluation_subsingleton`
  makes the type a subsingleton.  Otherwise it returns a located failure.

  **This uses `Classical.propDecidable` and `Classical.choice`, and is
  therefore noncomputable.**  It is not on any executable path, it is not an
  implementation of SPEC §10.1's `Universal.evaluate`, and it must not be read
  as one: it decodes nothing, validates nothing, enumerates no input and
  explores no branch.
-/
noncomputable def evaluateClassically (bytes : ByteArray) :
    Universal.EvaluationResult (setting seam) bytes :=
  @dite _ (Nonempty (Universal.SystemEvaluation (setting seam) bytes))
    (Classical.propDecidable _)
    (fun h => .complete (Classical.choice h))
    (fun _ => .profileFailure noEvaluationReport)

/-- **SPEC §10.1**, `Release.decider`: the release `Universal.Decider`, a
one-field wrapper around `Release.evaluateClassically`.  Noncomputable, for the
reason stated there. -/
noncomputable def decider : Universal.Decider (setting seam) :=
  ⟨evaluateClassically seam⟩

@[simp] theorem decider_evaluate (bytes : ByteArray) :
    (decider seam).evaluate bytes = evaluateClassically seam bytes := rfl

/--
  The evaluator returns `.complete e` for **every** inhabitant `e` of the
  evaluation type.  The `dif_pos` step supplies *some* inhabitant; uniqueness
  (`Universal.systemEvaluation_subsingleton`) upgrades it to the one asked for.
-/
theorem evaluateClassically_eq_complete {bytes : ByteArray}
    (e : Universal.SystemEvaluation (setting seam) bytes) :
    evaluateClassically seam bytes = .complete e := by
  have h : Nonempty (Universal.SystemEvaluation (setting seam) bytes) := ⟨e⟩
  show (@dite _ _ (Classical.propDecidable _) _ _) = _
  rw [dif_pos h]
  exact congrArg _ (Universal.systemEvaluation_subsingleton _ e)

/-- Consequently the release relation holds of every evaluation of every byte
sequence. -/
theorem systemEvaluationRel_of_evaluation {bytes : ByteArray}
    (e : Universal.SystemEvaluation (setting seam) bytes) :
    Universal.SystemEvaluationRel (setting seam) (decider seam) bytes e :=
  evaluateClassically_eq_complete seam e

/-! ### The three properties SPEC §1 requires of `SystemEvaluationRel` -/

/--
  **Soundness.**  The relation holds only of a genuine evaluation of exactly
  those bytes.

  Soundness here is structural and total: `Universal.SystemEvaluationRel` can
  relate `bytes` only to an inhabitant of `Universal.SystemEvaluation
  (setting seam) bytes`, and every such inhabitant carries, as *proved* fields,
  the decode equation for exactly those bytes, the maximal-execution coverage
  obligation at every raw invocation, and the exact aggregate cost equation of
  SPEC §9.1.  There is no `.complete` answer that escapes them.
-/
theorem systemEvaluationRel_sound {bytes : ByteArray}
    {e : Universal.SystemEvaluation (setting seam) bytes}
    (h : Universal.SystemEvaluationRel (setting seam) (decider seam) bytes e) :
    (decider seam).evaluate bytes = .complete e ∧
    Wasm.decode bytes = .ok e.module ∧
    (∀ raw : Gemm.RawInvocation wasmProfile,
      (e.perInput raw).CoversEveryMaximalExecution) ∧
    Cost.ExactAggregateCost bytes (Wasm.decode bytes = .ok e.module)
      (wasmProfile.body.costTableBody.decodeCost bytes)
      ((setting seam).semantics.validationSteps e.module)
      ((setting seam).semantics.staticDataBytes e.module)
      ((setting seam).problem.workloadRepetitions)
      (fun raw => (e.perInput raw).resourceVector) e.cost :=
  ⟨h, e.decodeEq, e.observationsComplete, e.costExact⟩

/--
  **Functionality.**  Two evaluations related to the same bytes are equal.

  Immediate from `Universal.systemEvaluation_subsingleton`: the relation is
  functional because its codomain is, which is stronger than functionality of
  the relation alone and is what `GlobalOptimal`'s conjunctive lower-bound
  clause actually needs.
-/
theorem systemEvaluationRel_functional {bytes : ByteArray}
    {a b : Universal.SystemEvaluation (setting seam) bytes}
    (_ha : Universal.SystemEvaluationRel (setting seam) (decider seam) bytes a)
    (_hb : Universal.SystemEvaluationRel (setting seam) (decider seam) bytes b) :
    a = b :=
  Universal.systemEvaluation_subsingleton a b

/--
  **Completeness.**  Every profile valid, semantically correct, resource
  bounded byte sequence that *has* an evaluation is related to it.

  The three extensional premises are stated because SPEC §1 states them; the
  proof does not consume them, because this decider is complete on strictly
  more than the admissible set.  That direction is safe: a decider that answers
  on more bytes than required still answers on all of them.
-/
theorem systemEvaluationRel_complete {bytes : ByteArray}
    (_hprofile : Universal.ProfileValid wasmProfile bytes)
    (_hcorrect : Universal.SemanticCorrect (setting seam) bytes)
    (_hresources : Universal.SemanticWithinResources (setting seam) bytes)
    (e : Universal.SystemEvaluation (setting seam) bytes) :
    Universal.SystemEvaluationRel (setting seam) (decider seam) bytes e :=
  systemEvaluationRel_of_evaluation seam e

/--
  **UV-003 discharged.**  `Universal.DeciderAnswersAdmissible` holds for the
  release decider.

  `Universal/Argmin.lean` isolated this as the one hypothesis of
  `Universal.exists_globalOptimal_of_nonempty` that is *not* derivable for an
  arbitrary `Universal.Decider` — nothing forbids a decider from answering
  `.profileFailure` on bytes that do have an evaluation.  The classical decider
  cannot: by construction it answers `.complete` whenever the evaluation type
  is inhabited, admissible or not.
-/
theorem deciderAnswersAdmissible :
    Universal.DeciderAnswersAdmissible (setting seam) (decider seam) :=
  fun _b _hadm e => ⟨e, systemEvaluationRel_of_evaluation seam e⟩

/-- The "for every inhabitant" form `GlobalOptimal`'s lower-bound clause
demands, at the release decider. -/
theorem deciderAnswersAdmissible_rel {b : ByteArray}
    (hadm : Universal.Admissible (setting seam) b)
    (e : Universal.SystemEvaluation (setting seam) b) :
    Universal.SystemEvaluationRel (setting seam) (decider seam) b e :=
  Universal.deciderAnswersAdmissible_rel (deciderAnswersAdmissible seam) hadm e

/-! ## 4. Attainability, instantiated -/

/--
  **SPEC §13, Phase D, at the release instantiation.**

  If the admissible, evaluated set is nonempty, some byte sequence satisfies
  `Universal.GlobalOptimal` at the release setting, the release decider and the
  release objective — with the competitor quantifier ranging over **all** of
  `ByteArray`.

  The decider hypothesis is discharged by `Release.deciderAnswersAdmissible`
  (UV-003).  Exactly one obligation is left: the antecedent.

  This does **not** prove `Artifact.released_wasm_gemm_gnaf_global_optimal`.
  The conclusion is an existential produced by classical reasoning; it names no
  literal, and identifying the committed release bytes with the byte sequence
  selected here is a separate, undischarged obligation.
-/
theorem exists_globalOptimal_of_nonempty
    (hne : ∃ b : ByteArray, Universal.Admissible (setting seam) b ∧
      ∃ e : Universal.SystemEvaluation (setting seam) b,
        Universal.SystemEvaluationRel (setting seam) (decider seam) b e) :
    ∃ bytes : ByteArray,
      Universal.GlobalOptimal (setting seam) (decider seam) (costObjective seam)
        bytes :=
  Universal.exists_globalOptimal_of_nonempty (deciderAnswersAdmissible seam) hne

/--
  The same statement with the relation clause removed.

  Because the release decider relates every byte sequence to every evaluation
  it has, the antecedent of `Release.exists_globalOptimal_of_nonempty` collapses
  to: *one* byte sequence that is profile valid, semantically correct, within
  resources, and has a system evaluation.  Nothing here exhibits one; this is
  the sharpest reduction available, not a discharge.
-/
theorem exists_globalOptimal_of_admissible_evaluation
    (hne : ∃ b : ByteArray, Universal.Admissible (setting seam) b ∧
      Nonempty (Universal.SystemEvaluation (setting seam) b)) :
    ∃ bytes : ByteArray,
      Universal.GlobalOptimal (setting seam) (decider seam) (costObjective seam)
        bytes := by
  obtain ⟨b, hadm, ⟨e⟩⟩ := hne
  exact exists_globalOptimal_of_nonempty seam
    ⟨b, hadm, e, systemEvaluationRel_of_evaluation seam e⟩

/--
  The reduction, stated as an equivalence of obligations: the two antecedents
  above are interchangeable.  This is what "the release theorem now rests on
  nonemptiness alone" means precisely.
-/
theorem nonemptiness_iff_admissible_evaluation :
    (∃ b : ByteArray, Universal.Admissible (setting seam) b ∧
        ∃ e : Universal.SystemEvaluation (setting seam) b,
          Universal.SystemEvaluationRel (setting seam) (decider seam) b e) ↔
      (∃ b : ByteArray, Universal.Admissible (setting seam) b ∧
        Nonempty (Universal.SystemEvaluation (setting seam) b)) := by
  constructor
  · rintro ⟨b, hadm, e, _⟩
    exact ⟨b, hadm, ⟨e⟩⟩
  · rintro ⟨b, hadm, ⟨e⟩⟩
    exact ⟨b, hadm, e, systemEvaluationRel_of_evaluation seam e⟩

end Decider

end Parametric

/-! ## 5. The seam, constructed (GO-008)

Everything above is universally quantified over `(seam : Seam)`, so every
release theorem is an implication whose antecedent may be unsatisfiable.  This
section removes that possibility by *building* a `Seam` and proving it is not
degenerate.

### 5.1 The event charge (SPEC §7.5)

`Universal.Semantics.costEvent` maps a **plain** `Wasm.Event` to a `Cost.Event`.
`Wasm/Costed.lean`'s contribution law is stated on `Wasm.CostedEvent`, which
carries data the plain event does not: `Wasm.Event.step` is the erasure of
fourteen different rules, `Event.growAttempt (.grown p)` records the pages held
*before* the grow and not the delta, and `Event.enterGemm` does not record how
many raw bytes were installed.  A configuration-free charge therefore cannot be
the contribution law, and pretending otherwise would invent numbers.

`Release.costEvent` charges, for each plain event, the largest charge that the
event alone determines: the pinned rule-step row for every event, plus the
pinned dispatch row for the six dispatching rules whose erasure is
distinguishable (`branch`, `enterGemm`, `throwEvent`, `returnEvent`).  Two
theorems below say exactly what that buys and exactly what it costs:

* `costEvent_charge_wasmRuleSteps` — **no event is free**: every plain event is
  charged one `wasmRuleSteps` unit, which is the pinned `ruleStepUnit` of the
  release cost table.
* `costEvent_le_eventContribution` — **nothing is over-charged**: for every
  costed event, the charge of its erasure is componentwise `≤` the contribution
  law's charge for it.  The inequality is strict exactly on the coordinates the
  plain event cannot determine (`scalarOps` on arithmetic, `bytesRead` and
  `bytesWritten` on memory access and raw installation, `memoryGrowPages` on a
  successful grow, the GC coordinates on a throw, `outputBytes` on a return).

So this is an honest *lower* bound on the cost model, not the cost model.  It is
disclosed here and again in `Theorems/Status.lean`. -/

/-- **SPEC §7.5**, the release event charge: the pinned per-rule row that the
plain event determines on its own.  See the section header for what it under-
charges and why. -/
def costEvent : Wasm.Event → Cost.Event
  | .step => .ruleStep
  | .branch => .dispatchStep
  | .growAttempt _ => .ruleStep
  | .enterGemm => .dispatchStep
  | .trapEvent _ => .ruleStep
  | .throwEvent _ => .dispatchStep
  | .returnEvent _ => .dispatchStep

/-- **No event is free.**  Every plain event is charged exactly one
`wasmRuleSteps` unit — the pinned `ruleStepUnit` of the release cost table. -/
theorem costEvent_charge_wasmRuleSteps (e : Wasm.Event) :
    (costEvent e).charge.wasmRuleSteps = 1 := by
  cases e <;> rfl

/-- The rule-step charge is under the contribution law's charge for every costed
event. -/
theorem ruleStep_charge_le_eventContribution (ce : Wasm.CostedEvent) :
    Cost.DynamicVector.ComponentwiseLE (Cost.Event.ruleStep).charge
      (Wasm.eventContribution wasmProfile.costTableBody ce) := by
  intro dc
  have h := Wasm.eventContribution_wasmRuleSteps wasmProfile ce
  cases dc <;>
    simp [Cost.DynamicCoordinate.value, Cost.Event.charge,
      Cost.DynamicVector.zero, h]

/-- The dispatch charge is under the contribution law's charge for every costed
event whose rule dispatches. -/
theorem dispatchStep_charge_le_eventContribution (ce : Wasm.CostedEvent)
    (hd : Wasm.IsDispatchRule ce.rule = true) :
    Cost.DynamicVector.ComponentwiseLE (Cost.Event.dispatchStep).charge
      (Wasm.eventContribution wasmProfile.costTableBody ce) := by
  intro dc
  have h := Wasm.eventContribution_wasmRuleSteps wasmProfile ce
  have hdisp := Wasm.eventContribution_dispatchSteps wasmProfile.costTableBody ce
  rw [hd] at hdisp
  simp only [if_pos] at hdisp
  cases dc <;>
    simp [Cost.DynamicCoordinate.value, Cost.Event.charge,
      Cost.DynamicVector.zero, h, hdisp]

/-- **Nothing is over-charged.**  For every costed event, the charge this
repository's configuration-free `costEvent` assigns to its erasure is
componentwise below the contribution law's exact charge. -/
theorem costEvent_le_eventContribution (ce : Wasm.CostedEvent) :
    Cost.DynamicVector.ComponentwiseLE (costEvent ce.erase).charge
      (Wasm.eventContribution wasmProfile.costTableBody ce) := by
  cases ce <;>
    first
      | exact ruleStep_charge_le_eventContribution _
      | exact dispatchStep_charge_le_eventContribution _ rfl

/-- The charge of a taken branch is *exactly* the pinned dispatch row: on the
dispatch rules the plain event loses nothing. -/
theorem costEvent_branch_charge :
    (costEvent .branch).charge = Wasm.dispatchCharge wasmProfile.costTableBody := by
  have h : wasmProfile.costTableBody.ruleStepUnit = 1 := wasmProfile.lawful.ruleStepUnit
  simp [costEvent, Cost.Event.charge, Wasm.dispatchCharge, Wasm.ruleStepCharge,
    Cost.DynamicVector.zero, h]

/-- The charge of a trap is *exactly* the pinned rule-step row. -/
theorem costEvent_trap_charge (t : Wasm.Trap) :
    (costEvent (.trapEvent t)).charge = Wasm.ruleStepCharge wasmProfile.costTableBody := by
  have h : wasmProfile.costTableBody.ruleStepUnit = 1 := wasmProfile.lawful.ruleStepUnit
  simp [costEvent, Cost.Event.charge, Wasm.ruleStepCharge,
    Cost.DynamicVector.zero, h]

/-! ### 5.2 The released costed semantics -/

/-- **SPEC §7.5**, `Release.semantics`: the release event charge together with
`Wasm.validationCost` and `Wasm.instantiatedStaticBytes`, assembled by
`Release.semanticsFor`. -/
def semantics : Universal.Semantics wasmProfile := semanticsFor costEvent

@[simp] theorem semantics_costEvent : semantics.costEvent = costEvent := rfl

@[simp] theorem semantics_validationSteps (m : Wasm.Module) :
    semantics.validationSteps m = Wasm.validationCost wasmProfile.costTableBody m := rfl

@[simp] theorem semantics_staticDataBytes (m : Wasm.Module) :
    semantics.staticDataBytes m = Wasm.instantiatedStaticBytes wasmProfile m := rfl

/-! ### 5.3 The released costed machine -/

/-- **SPEC §10.1**, `Release.machine`: `Wasm.initialGemmInvocationCosted` and
`Wasm.exploreAllCosted`, from `Wasm/CostedExplore.lean`. -/
def machine : Universal.CostedMachine semantics := Wasm.releaseCostedMachine semantics

@[simp] theorem machine_initial (m : Wasm.Module) (inv : Wasm.Invocation wasmProfile) :
    machine.initialGemmInvocationCosted m inv =
      Wasm.initialGemmInvocationCosted m inv := rfl

@[simp] theorem machine_explore (bound : Nat) (initial : Wasm.Config) :
    machine.exploreAllCosted bound initial =
      Wasm.exploreAllCosted semantics bound initial := rfl

/-! ### 5.4 The per-invocation resource limit (SPEC §8.2)

`Gemm.releaseResourceContract` names six numbers; `Cost.DynamicVector` has
sixteen coordinates.  Five coordinates are *pinned*: they are a contract field
verbatim.  The remaining eleven are *supplied at the profile maximum* implied by
the pinned costed-step budget, under one uniform rule — a coordinate charged at
most `k` per costed reduction step is bounded by `maxCostedSteps * k`, and `k`
is read off the pinned per-rule rows of `Wasm.eventContribution`:

| coordinate | value | why |
|---|---|---|
| `wasmRuleSteps` | `maxCostedSteps` | **pinned** (§8.2, costed steps) |
| `peakPages` | `maxPages` | **pinned** (§8.2, ordinary memory) |
| `tableElementsAllocated` | `maxTableElements` | **pinned** (§8.2, tables) |
| `peakGcLiveBytes` | `maxHeapBytes` | **pinned** (§8.2, GC/exception heap) |
| `peakStackValues` | `maxValueSlots` | **pinned** (§8.2, live value slots) |
| `instantiationSteps` | `maxCostedSteps` | ≤ 1 per rule step |
| `dispatchSteps` | `maxCostedSteps` | ≤ 1 per rule step |
| `preparationSteps` | `maxCostedSteps` | ≤ 1 per rule step |
| `scalarOps` | `maxCostedSteps` | ≤ 1 per rule step |
| `vectorLaneOps` | `maxCostedSteps` | the released profile charges none; not narrowed to `0` |
| `gcObjectsAllocated` | `maxCostedSteps` | ≤ 1 per rule step |
| `bytesRead` | `4 * maxCostedSteps` | ≤ `Wasm.i32TransferBytes` per rule step |
| `bytesWritten` | `4 * maxCostedSteps` | ≤ `Wasm.i32TransferBytes` per rule step |
| `outputBytes` | `4 * maxCostedSteps` | ≤ `Wasm.i32TransferBytes` per rule step |
| `gcBytesInitialized` | `24 * maxCostedSteps` | ≤ one canonical exception object per rule step |
| `memoryGrowPages` | `maxPages * maxCostedSteps` | ≤ `maxPages` per successful grow |

The eleven derived values are *upper* bounds, so the admissible competitor set
is not narrowed by them; it is widened as far as the pinned budget permits,
which makes `GlobalOptimal`'s universal clause harder, not easier.  Nothing here
is claimed to be SPEC §8.2's own sixteen-coordinate vector, which SPEC §8.2 does
not write down. -/

/-- The pinned costed-step budget of SPEC §8.2. -/
def limitSteps : Nat := Gemm.releaseResourceContract.maxCostedSteps

/-- The abstract width of one live exception object under the release profile's
pinned GC layout: 24 bytes, not a free store. -/
theorem exceptionObjectBytes_release :
    Wasm.exceptionObjectBytes wasmProfile.costTableBody.layout = 24 :=
  Wasm.canonical_exceptionObjectBytes

/-- **SPEC §8.2**, `Release.limit`: the per-invocation resource limit vector. -/
def limit : Cost.DynamicVector where
  instantiationSteps := limitSteps
  dispatchSteps := limitSteps
  preparationSteps := limitSteps
  wasmRuleSteps := Gemm.releaseResourceContract.maxCostedSteps
  scalarOps := limitSteps
  vectorLaneOps := limitSteps
  bytesRead := Wasm.i32TransferBytes * limitSteps
  bytesWritten := Wasm.i32TransferBytes * limitSteps
  memoryGrowPages := Gemm.releaseResourceContract.maxPages * limitSteps
  tableElementsAllocated := Gemm.releaseResourceContract.maxTableElements
  gcObjectsAllocated := limitSteps
  gcBytesInitialized :=
    Wasm.exceptionObjectBytes wasmProfile.costTableBody.layout * limitSteps
  peakStackValues := Gemm.releaseResourceContract.maxValueSlots
  peakPages := Gemm.releaseResourceContract.maxPages
  peakGcLiveBytes := Gemm.releaseResourceContract.maxHeapBytes
  outputBytes := Wasm.i32TransferBytes * limitSteps

/-- **The five coordinates SPEC §8.2 pins**, read back off `Release.limit`. -/
theorem limit_pinned :
    limit.wasmRuleSteps = 2 ^ 320 ∧
      limit.peakPages = 65536 ∧
      limit.tableElementsAllocated = 2 ^ 32 - 1 ∧
      limit.peakGcLiveBytes = 2 ^ 32 - 1 ∧
      limit.peakStackValues = 2 ^ 64 - 1 :=
  ⟨rfl, rfl, rfl, rfl, rfl⟩

/-- **The eleven coordinates SPEC §8.2 does not pin**, and the per-step ceiling
each one was supplied at.  Every one of them is a multiple of the pinned costed
step budget, so none of them narrows the admissible set below what that budget
already allows. -/
theorem limit_unpinned :
    limit.instantiationSteps = limitSteps ∧
      limit.dispatchSteps = limitSteps ∧
      limit.preparationSteps = limitSteps ∧
      limit.scalarOps = limitSteps ∧
      limit.vectorLaneOps = limitSteps ∧
      limit.gcObjectsAllocated = limitSteps ∧
      limit.bytesRead = 4 * limitSteps ∧
      limit.bytesWritten = 4 * limitSteps ∧
      limit.outputBytes = 4 * limitSteps ∧
      limit.gcBytesInitialized = 24 * limitSteps ∧
      limit.memoryGrowPages = 65536 * limitSteps :=
  ⟨rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl⟩

/-- Every coordinate of `Release.limit` is positive: no coordinate is closed off
by the limit, so no competitor is excluded by a zero budget. -/
theorem limit_pos (dc : Cost.DynamicCoordinate) : 0 < dc.value limit := by
  have h : 0 < (2 : Nat) ^ 320 := Nat.two_pow_pos 320
  cases dc <;>
    simp only [Cost.DynamicCoordinate.value, limit, limitSteps,
      exceptionObjectBytes_release, Gemm.releaseResourceContract,
      Wasm.i32TransferBytes] <;> omega

/-! ### 5.5 The seam -/

/-- **GO-008.**  The constructed release seam: the costed semantics of §5.2, the
costed machine of §5.3 and the resource limit of §5.4.  It is a closed term:
every field is a definition of this repository, none is a parameter. -/
def seam : Seam where
  semantics := semantics
  machine := machine
  limit := limit

@[simp] theorem seam_semantics : seam.semantics = semantics := rfl
@[simp] theorem seam_machine : seam.machine = machine := rfl
@[simp] theorem seam_limit : seam.limit = limit := rfl

/-- The release setting's resource limit is `Release.limit`. -/
theorem setting_limit : (setting seam).problem.limit = limit := rfl

/-! ### 5.6 The cost model of the constructed seam is not free -/

/-- **Non-degeneracy (i): the cost model is not free.**  There is a plain event
whose charge under the constructed seam is not the zero vector — indeed the
taken-branch event is charged the pinned dispatch row exactly. -/
theorem seam_costEvent_not_zero :
    ∃ e : Wasm.Event,
      (seam.semantics.costEvent e).charge ≠ Cost.DynamicVector.zero ∧
        (seam.semantics.costEvent e).charge =
          Wasm.dispatchCharge wasmProfile.costTableBody := by
  refine ⟨.branch, ?_, costEvent_branch_charge⟩
  intro h
  have : (1 : Nat) = 0 := congrArg Cost.DynamicVector.wasmRuleSteps h
  exact absurd this (by omega)

/-- The same fact for *every* event: the constructed cost model charges one
pinned rule-step unit per reduction, so no execution is free. -/
theorem seam_costEvent_charge_pos (e : Wasm.Event) :
    0 < (seam.semantics.costEvent e).charge.wasmRuleSteps := by
  rw [seam_semantics, semantics_costEvent, costEvent_charge_wasmRuleSteps]
  omega

/-! ### 5.7 A validating module costs at least one validation step -/

/-- **Non-degeneracy (ii): validation is not free.**  Every module, and in
particular every module that validates, is charged at least one validation
step by the constructed seam. -/
theorem seam_validationSteps_pos (m : Wasm.Module) :
    0 < seam.semantics.validationSteps m :=
  Wasm.validationCost_pos wasmProfile m

/-! ### 5.8 Generic explorer lemmas

Four small facts about `Wasm.exploreTree`, `Wasm.prefixes` and `Wasm.Reduces`
that are used to discharge the machine on a concrete module.  They are stated
here rather than in `Wasm/Fuel.lean` because they are proof plumbing for this
file, not part of the explorer's contract. -/

/-- A `some` option has a nonempty `toList`. -/
theorem toList_ne_nil_of_isSome {α : Type} {x : Option α} (h : x.isSome = true) :
    x.toList ≠ [] := by
  cases x with
  | none => simp at h
  | some a => simp

/-- A terminal configuration contributes exactly its own observation, at any
remaining fuel. -/
theorem exploreTree_of_terminal (n : Nat) (tr : List Wasm.Event) (c : Wasm.Config)
    (h : ¬ c.status = Wasm.Status.running) :
    Wasm.exploreTree n tr c = (Wasm.observationOfConfig tr c).toList := by
  cases n with
  | zero => rfl
  | succ k => exact Wasm.exploreTree_succ_of_terminal k tr h

/-- A terminal configuration has no still-running prefix, at any remaining
fuel. -/
theorem prefixes_of_terminal (n : Nat) (tr : List Wasm.Event) (c : Wasm.Config)
    (h : ¬ c.status = Wasm.Status.running) : Wasm.prefixes n tr c = [] := by
  cases n with
  | zero => simp [Wasm.prefixes, h]
  | succ k => simp [Wasm.prefixes, h]

/-- A configuration with exactly one permitted successor advances into it. -/
theorem exploreTree_of_single (n : Nat) (tr : List Wasm.Event) {c c' : Wasm.Config}
    {e : Wasm.Event} (hrun : c.status = Wasm.Status.running)
    (hs : Wasm.successors c = [(e, c')]) :
    Wasm.exploreTree (n + 1) tr c = Wasm.exploreTree n (tr ++ [e]) c' := by
  rw [Wasm.exploreTree_succ_of_running n tr hrun, hs]
  simp

/-- The same, for the still-running prefixes. -/
theorem prefixes_of_single (n : Nat) (tr : List Wasm.Event) {c c' : Wasm.Config}
    {e : Wasm.Event} (hrun : c.status = Wasm.Status.running)
    (hs : Wasm.successors c = [(e, c')]) :
    Wasm.prefixes (n + 1) tr c = Wasm.prefixes n (tr ++ [e]) c' := by
  simp [Wasm.prefixes, hrun, hs]

/-- A reduction out of a configuration with exactly one permitted successor is
either empty or begins with that successor. -/
theorem reduces_of_single {c c' : Wasm.Config} {e : Wasm.Event}
    (hs : Wasm.successors c = [(e, c')]) {tr : List Wasm.Event} {final : Wasm.Config}
    (h : Wasm.Reduces c tr final) :
    tr = [] ∨ ∃ tr', tr = e :: tr' ∧ Wasm.Reduces c' tr' final := by
  cases h with
  | refl _ => exact Or.inl rfl
  | cons hstep hrest =>
      rename_i e₂ c₂ tr₂
      have hm : (e₂, c₂) ∈ Wasm.successors c :=
        (Wasm.mem_successors_iff_step c e₂ c₂).mpr hstep
      rw [hs] at hm
      simp only [List.mem_singleton, Prod.mk.injEq] at hm
      obtain ⟨rfl, rfl⟩ := hm
      exact Or.inr ⟨tr₂, rfl, hrest⟩

/-- Nothing reduces out of a terminal configuration. -/
theorem reduces_nil_of_terminal {c : Wasm.Config}
    (h : ¬ c.status = Wasm.Status.running) {tr : List Wasm.Event} {final : Wasm.Config}
    (hr : Wasm.Reduces c tr final) : tr = [] := by
  cases hr with
  | refl _ => rfl
  | cons hstep _ => exact absurd (Wasm.Step.running hstep) h

/-! ### 5.9 The module the explorer is discharged on

`Artifact.baselineModule` — the compiled GEMM witness plan — is **not** the
module used here, and the reason is stated rather than hidden: discharging
`Wasm.exploreAllCosted` on it means proving that every branch of the compiled
blocked traversal terminates inside `2 ^ 320` steps, which needs a termination
argument for `GNAF.bodyCode`'s loops that this repository does not have, and
which no amount of `decide` can supply at that fuel.

Since BI-006, part of that is no longer vague.  §5.13 below carries the *real*
GEMM kernel — `GNAF.gemmKernel`, whose loop bounds are the header's declared
extents — as far as the Wasm layer allows: its compiled module is profile valid
on a closed literal, its plan-level static step bound is proved to be about
`2 ^ 99` and therefore strictly inside the `2 ^ 320` budget
(`GNAF.gemmKernel_stepBound_le_maxSteps`), and the whole machine and evaluation
chain is proved from two explicitly named propositions,
`Release.GemmKernelReducesBounded` and `Release.GemmKernelReachesTerminal`.
Those two are Wasm-execution facts, not plan facts, and neither is proved here;
the second one is `Wasm.validation_progress`, OUTSTANDING under `O-6`.

What is used instead is the smallest module `GNAF.moduleOf` emits: the same
module *shape* — one recursive type group, one function at the pinned `gemm` ABI
type, one exported single-page memory, no imports, no start function, exactly
the two required exports — with the shortest body the ABI admits, `i32.const 0`.
Every branch of it terminates in at most three reduction steps: install the raw
bytes and cross the harness boundary (or trap out of bounds), push the status
word, return it.

So the machine below is discharged on a real, profile-valid, decoding module,
and **not** on a GEMM implementation.  `Release.witnessModule` computes no
product; it is a witness that the costed machine completes, and it is cited as
nothing else. -/

/-- The compilation environment of the smallest module `GNAF.moduleOf` emits:
no scalar registers, no GNAF memory, no scratch, no tables, no nesting.  Its
declared memory is exactly one page. -/
def witnessEnv : GNAF.CompileEnv where
  regs := 0
  memWords := 0
  scratchWords := 0
  tableCount := 0
  tableStride := 0
  depthBound := 0

theorem witnessEnv_pages : witnessEnv.pages = 1 := rfl

theorem witnessEnv_declaredLocals : witnessEnv.declaredLocals = 1 := rfl

/-- The body of the witness `gemm` export: push the status word `0`.  The
pinned ABI type returns one `i32`, so no shorter body validates. -/
def witnessBody : List Wasm.Instr := [Wasm.Instr.i32Const 0]

/-- **The module the costed machine is discharged on.**  See the section header
for what it is and, emphatically, what it is not. -/
def witnessModule : Wasm.Module := GNAF.moduleOf witnessEnv witnessBody

/-- The canonical encoding of `Release.witnessModule`: a closed `ByteArray`. -/
def witnessBytes : ByteArray := Artifact.emit witnessModule

/-- **SPEC §7.3.**  The witness module passes the release validator. -/
theorem witnessModule_validate : Wasm.validate witnessModule = true :=
  GNAF.validate_moduleOf witnessEnv witnessBody rfl

/-- The witness bytes decode back to the witness module. -/
theorem witness_decode : Wasm.decode witnessBytes = .ok witnessModule :=
  Artifact.decode_emit witnessModule

/-- **SPEC §7.2.**  Every index-space population of the witness module is inside
the release profile's limit table. -/
theorem witnessModule_withinProfileLimits :
    Universal.WithinProfileLimits wasmProfile witnessModule = true := by decide

/-- **SPEC §10.1.**  The witness module passes `Universal.validateUnder`. -/
theorem witnessModule_validateUnder :
    Universal.validateUnder wasmProfile witnessModule = true := by
  unfold Universal.validateUnder
  rw [witnessModule_validate, witnessModule_withinProfileLimits]
  rfl

/-- The witness module declares no import: it is closed. -/
theorem witnessModule_imports : witnessModule.imports = [] := rfl

/-- The witness module's export list, in full. -/
theorem witnessModule_exports :
    witnessModule.exports =
      [{ name := Wasm.gemmExportName, desc := .func 0 },
       { name := Wasm.memoryExportName, desc := .mem 0 }] := rfl

/-- The witness module exports the `gemm` function and the memory, and nothing
else at all. -/
theorem witnessModule_hasExactGemmExports :
    Universal.HasExactGemmExports wasmProfile witnessModule := by
  have h := witnessModule_validate
  unfold Wasm.validate at h
  simp only [Bool.and_eq_true] at h
  refine ⟨h.1.1.2, h.1.2, ?_⟩
  intro e he
  rw [witnessModule_exports] at he
  simp only [List.mem_cons, List.not_mem_nil, or_false] at he
  rcases he with rfl | rfl
  · exact Or.inl rfl
  · exact Or.inr rfl

/-- **SPEC §10.1.**  The witness bytes are profile valid: they decode, the
module validates under the profile's limit table, it imports nothing, and it
carries exactly the two required exports.  Hypothesis-free, on a closed
literal. -/
theorem witness_profileValid : Universal.ProfileValid wasmProfile witnessBytes :=
  ⟨witnessModule, witness_decode, witnessModule_validateUnder,
    witnessModule_imports, witnessModule_hasExactGemmExports⟩

/-! ### 5.10 Every branch of the witness module terminates in three steps -/

/-- The initial configuration of the witness module at a plain raw invocation. -/
def witnessInitialOf (raw : Wasm.RawInvocation) : Wasm.Config :=
  GNAF.compiledInitialConfig witnessEnv witnessBody raw

/-- Initialization of the witness module never faults. -/
theorem witness_initialConfig (raw : Wasm.RawInvocation) :
    Wasm.initialConfig witnessModule raw = .ok (witnessInitialOf raw) :=
  GNAF.initialConfig_moduleOf witnessEnv witnessBody witnessModule_validate raw

/-- The configuration after the harness has installed the raw bytes and crossed
the entry boundary. -/
def witnessEntered (raw : Wasm.RawInvocation) (s : Wasm.Store) : Wasm.Config :=
  { witnessInitialOf raw with
    store := s
    locals := (witnessInitialOf raw).harness.args ++
      List.replicate (witnessInitialOf raw).harness.gemmNumLocals 0
    stack := [], code := (witnessInitialOf raw).harness.gemmBody, ctrl := []
    phase := .afterEntry, entry? := some s.observable }

/-- The configuration after the status word has been pushed. -/
def witnessPushed (raw : Wasm.RawInvocation) (s : Wasm.Store) : Wasm.Config :=
  { witnessEntered raw s with code := [], stack := [Wasm.wrapI32 0] }

/-- The terminal configuration of the normal branch: `gemm` returns `0`. -/
def witnessReturned (raw : Wasm.RawInvocation) (s : Wasm.Store) : Wasm.Config :=
  { witnessPushed raw s with status := .returned (Wasm.wrapI32 0) }

/-- The terminal configuration of the branch on which the raw invocation does
not fit in the declared page: the harness traps out of bounds before entry. -/
def witnessTrapped (raw : Wasm.RawInvocation) : Wasm.Config :=
  { witnessInitialOf raw with status := .trapped .outOfBounds }

theorem witnessInitialOf_running (raw : Wasm.RawInvocation) :
    (witnessInitialOf raw).status = Wasm.Status.running := rfl

theorem witnessEntered_running (raw : Wasm.RawInvocation) (s : Wasm.Store) :
    (witnessEntered raw s).status = Wasm.Status.running := rfl

theorem witnessPushed_running (raw : Wasm.RawInvocation) (s : Wasm.Store) :
    (witnessPushed raw s).status = Wasm.Status.running := rfl

theorem witnessReturned_terminal (raw : Wasm.RawInvocation) (s : Wasm.Store) :
    ¬ (witnessReturned raw s).status = Wasm.Status.running := by
  intro h
  exact Wasm.Status.noConfusion h

theorem witnessTrapped_terminal (raw : Wasm.RawInvocation) :
    ¬ (witnessTrapped raw).status = Wasm.Status.running := by
  intro h
  exact Wasm.Status.noConfusion h

/-- The harness step, when the raw invocation fits in the declared page. -/
theorem witness_successors_initial_ok (raw : Wasm.RawInvocation) {s : Wasm.Store}
    (h : (GNAF.compiledStore witnessEnv).storeBytes raw.ptr raw.bytes = some s) :
    Wasm.successors (witnessInitialOf raw) =
      [(Wasm.Event.enterGemm, witnessEntered raw s)] := by
  show Wasm.successorsAtEnd (witnessInitialOf raw) = _
  unfold Wasm.successorsAtEnd
  simp only [witnessInitialOf, GNAF.compiledInitialConfig, GNAF.compiledHarness]
  rw [h]
  rfl

/-- The harness step, when it does not. -/
theorem witness_successors_initial_trap (raw : Wasm.RawInvocation)
    (h : (GNAF.compiledStore witnessEnv).storeBytes raw.ptr raw.bytes = none) :
    Wasm.successors (witnessInitialOf raw) =
      [(Wasm.Event.trapEvent .outOfBounds, witnessTrapped raw)] := by
  show Wasm.successorsAtEnd (witnessInitialOf raw) = _
  unfold Wasm.successorsAtEnd
  simp only [witnessInitialOf, GNAF.compiledInitialConfig, GNAF.compiledHarness]
  rw [h]
  rfl

/-- The `i32.const` step. -/
theorem witness_successors_entered (raw : Wasm.RawInvocation) (s : Wasm.Store) :
    Wasm.successors (witnessEntered raw s) =
      [(Wasm.Event.step, witnessPushed raw s)] := rfl

/-- The return step. -/
theorem witness_successors_pushed (raw : Wasm.RawInvocation) (s : Wasm.Store) :
    Wasm.successors (witnessPushed raw s) =
      [(Wasm.Event.returnEvent (Wasm.wrapI32 0), witnessReturned raw s)] := rfl

/-- **No branch of the witness module is still running after three steps.** -/
theorem witness_prefixes (raw : Wasm.RawInvocation) (fuel : Nat) (hf : 3 ≤ fuel) :
    Wasm.prefixes fuel [] (witnessInitialOf raw) = [] := by
  obtain ⟨n, rfl⟩ : ∃ n, fuel = n + 1 + 1 + 1 := ⟨fuel - 3, by omega⟩
  cases hstore : (GNAF.compiledStore witnessEnv).storeBytes raw.ptr raw.bytes with
  | some s =>
      rw [prefixes_of_single (n + 1 + 1) [] (witnessInitialOf_running raw)
            (witness_successors_initial_ok raw hstore),
          prefixes_of_single (n + 1) _ (witnessEntered_running raw s)
            (witness_successors_entered raw s),
          prefixes_of_single n _ (witnessPushed_running raw s)
            (witness_successors_pushed raw s)]
      exact prefixes_of_terminal n _ _ (witnessReturned_terminal raw s)
  | none =>
      rw [prefixes_of_single (n + 1 + 1) [] (witnessInitialOf_running raw)
            (witness_successors_initial_trap raw hstore)]
      exact prefixes_of_terminal (n + 1 + 1) _ _ (witnessTrapped_terminal raw)

/-- Consequently the plain explorer reports completion. -/
theorem witness_prefixes_head_none (raw : Wasm.RawInvocation) (fuel : Nat)
    (hf : 3 ≤ fuel) :
    (Wasm.prefixes fuel [] (witnessInitialOf raw)).head? = none := by
  rw [witness_prefixes raw fuel hf]
  rfl

/-- **The witness module produces at least one observation**, so the frontier
the costed machine returns is nonempty.  This is the conjunct that a machine
answering `.initializationFailure` unconditionally could never supply. -/
theorem witness_exploreTree_ne_nil (raw : Wasm.RawInvocation) (fuel : Nat)
    (hf : 3 ≤ fuel) : Wasm.exploreTree fuel [] (witnessInitialOf raw) ≠ [] := by
  obtain ⟨n, rfl⟩ : ∃ n, fuel = n + 1 + 1 + 1 := ⟨fuel - 3, by omega⟩
  cases hstore : (GNAF.compiledStore witnessEnv).storeBytes raw.ptr raw.bytes with
  | some s =>
      rw [exploreTree_of_single (n + 1 + 1) [] (witnessInitialOf_running raw)
            (witness_successors_initial_ok raw hstore),
          exploreTree_of_single (n + 1) _ (witnessEntered_running raw s)
            (witness_successors_entered raw s),
          exploreTree_of_single n _ (witnessPushed_running raw s)
            (witness_successors_pushed raw s),
          exploreTree_of_terminal n _ _ (witnessReturned_terminal raw s)]
      exact toList_ne_nil_of_isSome rfl
  | none =>
      rw [exploreTree_of_single (n + 1 + 1) [] (witnessInitialOf_running raw)
            (witness_successors_initial_trap raw hstore),
          exploreTree_of_terminal (n + 1 + 1) _ _ (witnessTrapped_terminal raw)]
      exact toList_ne_nil_of_isSome rfl

/-- **Every reduction of the witness module has at most three steps.** -/
theorem witness_reduces_length (raw : Wasm.RawInvocation) {tr : List Wasm.Event}
    {final : Wasm.Config} (h : Wasm.Reduces (witnessInitialOf raw) tr final) :
    tr.length ≤ 3 := by
  cases hstore : (GNAF.compiledStore witnessEnv).storeBytes raw.ptr raw.bytes with
  | some s =>
      rcases reduces_of_single (witness_successors_initial_ok raw hstore) h with
        rfl | ⟨tr₁, rfl, h₁⟩
      · simp
      rcases reduces_of_single (witness_successors_entered raw s) h₁ with
        rfl | ⟨tr₂, rfl, h₂⟩
      · simp
      rcases reduces_of_single (witness_successors_pushed raw s) h₂ with
        rfl | ⟨tr₃, rfl, h₃⟩
      · simp
      rw [reduces_nil_of_terminal (witnessReturned_terminal raw s) h₃]
      simp
  | none =>
      rcases reduces_of_single (witness_successors_initial_trap raw hstore) h with
        rfl | ⟨tr₁, rfl, h₁⟩
      · simp
      rw [reduces_nil_of_terminal (witnessTrapped_terminal raw) h₁]
      simp

/-- **The witness module never diverges.**  This is what `CoversEveryMaximal
Execution` needs and what no fuel bound can supply on its own. -/
theorem witness_not_diverges (raw : Wasm.RawInvocation)
    (events : Nat → Wasm.Event) (configs : Nat → Wasm.Config)
    (starts : configs 0 = witnessInitialOf raw)
    (hstep : ∀ i, Wasm.Step (configs i) (events i) (configs (i + 1))) : False := by
  have h4 : Wasm.Reduces (configs 0)
      [events 0, events 1, events 2, events 3] (configs 4) :=
    .cons (hstep 0) (.cons (hstep 1) (.cons (hstep 2) (.cons (hstep 3) (.refl _))))
  rw [starts] at h4
  have := witness_reduces_length raw h4
  simp at this

/-- Every finite execution of the witness module has a trace of at most three
events, hence far inside the released step budget. -/
theorem witness_finiteExecution_length (raw : Wasm.RawInvocation)
    {o : Wasm.ExecutionObservation} (h : Wasm.FiniteExecution (witnessInitialOf raw) o) :
    o.trace.length ≤ 3 := by
  obtain ⟨final, hred, _⟩ := h
  exact witness_reduces_length raw hred

/-! ### 5.11 The costed machine completes on the witness module

This is the conjunct GO-008 exists to establish.  `Release.machine` is
`Wasm.releaseCostedMachine`, an honest all-branch explorer: it answers
`.initializationFailure` exactly when the plain explorer finds no observation at
all (`Wasm.exploreAllCosted_initializationFailure_iff`) and `.nonterminalPrefix`
exactly when some branch overruns.  On the witness module neither happens. -/

/-- The initial configuration of the witness module at a GEMM raw invocation. -/
def witnessInitial (raw : Gemm.RawInvocation wasmProfile) : Wasm.Config :=
  witnessInitialOf (Wasm.rawOfInvocation (Universal.toWasmInvocation raw))

/-- Three reduction steps fit inside the released costed step budget with room
to spare. -/
theorem three_le_maxSteps : 3 ≤ (setting seam).problem.maxSteps := by
  show (3 : Nat) ≤ 2 ^ 320
  have h : (2 : Nat) ^ 2 ≤ 2 ^ 320 := Nat.pow_le_pow_right (by omega) (by omega)
  have h4 : (2 : Nat) ^ 2 = 4 := rfl
  exact Nat.le_trans (by omega : (3 : Nat) ≤ 4) (h4 ▸ h)

/-- **Non-degeneracy (iii.a): costed initialization succeeds.** -/
theorem seam_machine_init (raw : Gemm.RawInvocation wasmProfile) :
    (setting seam).machine.initialGemmInvocationCosted witnessModule
        (Universal.toWasmInvocation raw) =
      .ok { initial := witnessInitial raw,
            cost := Wasm.initializationCost wasmProfile witnessModule } :=
  Wasm.initialGemmInvocationCosted_ok (witness_initialConfig _)

/-- **Non-degeneracy (iii.b): the costed explorer completes with a nonempty
frontier**, on every raw invocation. -/
theorem seam_machine_explore (raw : Gemm.RawInvocation wasmProfile) :
    ∃ (frontier : Foundation.NonemptyCanonicalFrontier
          (Universal.CostedExecutionObservation (setting seam).semantics
            (witnessInitial raw)))
      (coverage : Universal.CostedCoverage (setting seam).semantics
          (setting seam).problem.maxSteps (witnessInitial raw) frontier),
      (setting seam).machine.exploreAllCosted (setting seam).problem.maxSteps
          (witnessInitial raw) = .complete frontier coverage :=
  Wasm.exploreAllCosted_complete_of_exploreTree_ne_nil semantics
    (setting seam).problem.maxSteps (witnessInitial raw)
    (witness_prefixes_head_none _ _ (by have := three_le_maxSteps; omega))
    (witness_exploreTree_ne_nil _ _ (by have := three_le_maxSteps; omega))

/-- **Non-degeneracy (iii): the machine both starts and completes.**  The seam's
machine is not the fake that answers `.initializationFailure` unconditionally;
on `Release.witnessModule` it initializes and returns a completed, nonempty,
canonically ordered costed frontier at every raw invocation.

`Artifact.baselineModule` is **not** the module here; see §5.9 for why. -/
theorem seam_machine_completes (raw : Gemm.RawInvocation wasmProfile) :
    seam.machine.initialGemmInvocationCosted witnessModule
        (Universal.toWasmInvocation raw) =
      .ok { initial := witnessInitial raw,
            cost := Wasm.initializationCost wasmProfile witnessModule } ∧
    ∃ (frontier : Foundation.NonemptyCanonicalFrontier
          (Universal.CostedExecutionObservation semantics (witnessInitial raw)))
      (coverage : Universal.CostedCoverage semantics
          (setting seam).problem.maxSteps (witnessInitial raw) frontier),
      seam.machine.exploreAllCosted (setting seam).problem.maxSteps
          (witnessInitial raw) = .complete frontier coverage :=
  ⟨seam_machine_init raw, seam_machine_explore raw⟩

theorem seam_not_nonterminal (raw : Gemm.RawInvocation wasmProfile)
    (overrun : Universal.NonterminalPrefix (witnessInitial raw)
      ((setting seam).problem.maxSteps + 1)) :
    (setting seam).machine.exploreAllCosted (setting seam).problem.maxSteps
        (witnessInitial raw) ≠ .nonterminalPrefix overrun := by
  obtain ⟨f, c, h⟩ := seam_machine_explore raw
  rw [h]
  intro hh
  simp at hh

theorem seam_not_initializationFailure (raw : Gemm.RawInvocation wasmProfile)
    (fault : Wasm.InstantiationFault) :
    (setting seam).machine.exploreAllCosted (setting seam).problem.maxSteps
        (witnessInitial raw) ≠ .initializationFailure fault := by
  obtain ⟨f, c, h⟩ := seam_machine_explore raw
  rw [h]
  intro hh
  simp at hh

/-- A costed exploration that is neither an overrun nor an initialization
failure yields its completed frontier as *data*, so the per-input evaluation
below is a plain term and needs no choice principle. -/
def completeWitnessOf {P : Wasm.Profile} {W : Universal.Semantics P} {bound : Nat}
    {initial : Wasm.Config} (r : Universal.CostedTreeResult W bound initial)
    (hn : ∀ overrun, r ≠ .nonterminalPrefix overrun)
    (hi : ∀ fault, r ≠ .initializationFailure fault) :
    Σ' (frontier : Foundation.NonemptyCanonicalFrontier
          (Universal.CostedExecutionObservation W initial))
       (coverage : Universal.CostedCoverage W bound initial frontier),
      r = .complete frontier coverage :=
  match r, hn, hi with
  | .complete frontier coverage, _, _ => ⟨frontier, coverage, rfl⟩
  | .nonterminalPrefix overrun, hn, _ => absurd rfl (hn overrun)
  | .initializationFailure fault, _, hi => absurd rfl (hi fault)

/-- The completed exploration of the witness module, as data. -/
def witnessCompleteWitness (raw : Gemm.RawInvocation wasmProfile) :
    Σ' (frontier : Foundation.NonemptyCanonicalFrontier
          (Universal.CostedExecutionObservation (setting seam).semantics
            (witnessInitial raw)))
       (coverage : Universal.CostedCoverage (setting seam).semantics
          (setting seam).problem.maxSteps (witnessInitial raw) frontier),
      (setting seam).machine.exploreAllCosted (setting seam).problem.maxSteps
        (witnessInitial raw) = .complete frontier coverage :=
  completeWitnessOf
    ((setting seam).machine.exploreAllCosted (setting seam).problem.maxSteps
      (witnessInitial raw))
    (seam_not_nonterminal raw) (seam_not_initializationFailure raw)

/-! ### 5.12 The per-input and system evaluations -/

/-- One `Universal.InputEvaluation` of the witness module, assembled from a
completed costed exploration. -/
def witnessInputEvaluationOf (raw : Gemm.RawInvocation wasmProfile)
    (frontier : Foundation.NonemptyCanonicalFrontier
      (Universal.CostedExecutionObservation (setting seam).semantics
        (witnessInitial raw)))
    (coverage : Universal.CostedCoverage (setting seam).semantics
      (setting seam).problem.maxSteps (witnessInitial raw) frontier)
    (h : (setting seam).machine.exploreAllCosted (setting seam).problem.maxSteps
      (witnessInitial raw) = .complete frontier coverage) :
    Universal.InputEvaluation (setting seam) witnessModule raw where
  initial := witnessInitial raw
  initialization :=
    { initial := witnessInitial raw
      cost := Wasm.initializationCost wasmProfile witnessModule }
  initialConfigEq := rfl
  initialEq := seam_machine_init raw
  observations := frontier
  treeComplete := ⟨_, h⟩
  resourceVector :=
    Cost.sequentialCompose (Wasm.initializationCost wasmProfile witnessModule)
      (Cost.maxOverCosts (frontier.elements.map (·.cost)))
  resourceExact := rfl

/-- **SPEC §10.1, `CoversEveryMaximalExecution`.**  No maximal execution of the
witness module escapes the frontier: the finite ones are inside the bound, and
there are no divergent ones. -/
theorem witnessInputEvaluationOf_covers (raw : Gemm.RawInvocation wasmProfile)
    (frontier : Foundation.NonemptyCanonicalFrontier
      (Universal.CostedExecutionObservation (setting seam).semantics
        (witnessInitial raw)))
    (coverage : Universal.CostedCoverage (setting seam).semantics
      (setting seam).problem.maxSteps (witnessInitial raw) frontier)
    (h : (setting seam).machine.exploreAllCosted (setting seam).problem.maxSteps
      (witnessInitial raw) = .complete frontier coverage) :
    (witnessInputEvaluationOf raw frontier coverage h).CoversEveryMaximalExecution := by
  intro execution
  cases execution with
  | finite o run maximal =>
      have hlen : o.trace.length ≤ (setting seam).problem.maxSteps :=
        Nat.le_trans (witness_finiteExecution_length _ run) three_le_maxSteps
      obtain ⟨c, hc, hco⟩ := coverage.2 o run hlen
      exact ⟨c, hc, hco.symm⟩
  | diverges events configs starts step =>
      exact (witness_not_diverges _ events configs starts step).elim

/-- The per-input evaluation of the witness module, at every raw invocation. -/
def witnessPerInput (raw : Gemm.RawInvocation wasmProfile) :
    Universal.InputEvaluation (setting seam) witnessModule raw :=
  witnessInputEvaluationOf raw (witnessCompleteWitness raw).1
    (witnessCompleteWitness raw).2.1 (witnessCompleteWitness raw).2.2

theorem witnessPerInput_covers (raw : Gemm.RawInvocation wasmProfile) :
    (witnessPerInput raw).CoversEveryMaximalExecution :=
  witnessInputEvaluationOf_covers raw _ _ _

/-! ### 5.13 The compiled GEMM kernel: what closes, and the two things that do not

`GNAF.gemmKernel` is the input-dependent GEMM plan of `GNAF/CompileCorrect.lean`
— it loads `m`, `n`, `k` out of the ABI header and runs an `i`/`j`/`k` nest of
`Plan.loopReg` loops bounded by exactly those registers — and
`GNAF.gemmKernel_writes_C` proves that evaluating it deposits the modular-`u32`
product into the declared `C` region.  This section carries that plan as far
toward the costed machine as the Wasm layer of this repository permits, and
states the remainder as two named propositions rather than assuming them.

**What is discharged here, unconditionally.**

* `Release.gemmKernelModule` — the module `GNAF.compile` emits from
  `GNAF.gemmKernelChecked` — validates, stays inside the profile limit table,
  imports nothing and carries exactly the two required exports, so
  `Release.gemmKernel_profileValid` holds on the closed byte literal
  `Release.gemmKernelBytes`.
* `GNAF.gemmKernel_stepBound_le_maxSteps` (proved in `GNAF/CompileCorrect.lean`)
  puts the plan's static step bound — `792281624699921518248014643209`, about
  `2 ^ 99`, the three clamped `loopReg` loops — strictly inside the released
  costed step budget `2 ^ 320`.  `Release.gemmKernel_stepBound_lt_maxSteps`
  restates it against `(setting seam).problem.maxSteps` itself.
* Given the two propositions below, the *whole* machine chain closes:
  `Release.gemmKernel_machine_completes_of`, and the evaluation
  `Release.gemmKernelSystemEvaluationOf` in §5.14.

**What does not close, stated exactly.**  `Plan.stepBound` bounds the step count
of the *GNAF* interpreter `Plan.eval`.  Nothing in this repository relates it to
the length of a *Wasm* reduction of the compiled module.  `GNAF.compile_resources`
does not: it bounds the emitted module's static instruction count and says so.
The two missing facts are exactly:

* `Release.GemmKernelReducesBounded` — every reduction of the compiled kernel
  from its initial configuration stays inside the released step budget.  This is
  the termination half of the omitted `compile_refines`: it needs a simulation
  between `Wasm.Step` on `GNAF.bodyCode` and `Plan.eval`, or at least a direct
  Wasm-level trip-count argument for every `countLoopVar` the compiler emits,
  and neither exists here.  `Release.gemmKernelReducesBounded_of_stepBound_multiple`
  shows that any bound of at most `2 ^ 220` Wasm steps per charged plan step
  suffices, so the gap is the existence of a bound and not its size.
* `Release.GemmKernelReachesTerminal` — some branch reaches a *terminal*
  configuration (`Wasm.observationOfConfig` is `none` on a running one, and a
  stuck-but-running configuration is exactly what a progress theorem excludes).
  This is `Wasm.validation_progress`, listed as OUTSTANDING under `O-6` in
  `Theorems/Status.lean`; no progress theorem exists for the modelled subset.

Neither is assumed anywhere: they appear only as explicit hypotheses of the
theorems that need them.  `Release.seam_machine_completes` and
`Release.systemEvaluation_inhabited` continue to be discharged on
`Release.witnessModule` alone. -/

/-- **The compiled input-dependent GEMM kernel.**  Unlike
`Artifact.baselineModule` (compiled from the fixed-extent `GNAF.gemmWitness`),
this is compiled from the plan whose loop bounds are the header's declared
extents. -/
def gemmKernelModule : Wasm.Module := GNAF.compile GNAF.gemmKernelChecked

/-- The canonical encoding of `Release.gemmKernelModule`: a closed `ByteArray`. -/
def gemmKernelBytes : ByteArray := Artifact.emit gemmKernelModule

/-- **SPEC §7.3.**  The compiled kernel passes the release validator. -/
theorem gemmKernelModule_validate : Wasm.validate gemmKernelModule = true :=
  GNAF.gemmKernel_compiles

/-- The kernel bytes decode back to the kernel module. -/
theorem gemmKernel_decode : Wasm.decode gemmKernelBytes = .ok gemmKernelModule :=
  Artifact.decode_emit gemmKernelModule

/-- **SPEC §7.2.**  Every index-space population of the compiled kernel is
inside the release profile's limit table. -/
theorem gemmKernelModule_withinProfileLimits :
    Universal.WithinProfileLimits wasmProfile gemmKernelModule = true := by decide

/-- **SPEC §10.1.**  The compiled kernel passes `Universal.validateUnder`. -/
theorem gemmKernelModule_validateUnder :
    Universal.validateUnder wasmProfile gemmKernelModule = true := by
  unfold Universal.validateUnder
  rw [gemmKernelModule_validate, gemmKernelModule_withinProfileLimits]
  rfl

/-- The compiled kernel declares no import: it is closed. -/
theorem gemmKernelModule_imports : gemmKernelModule.imports = [] := rfl

/-- The compiled kernel's export list, in full. -/
theorem gemmKernelModule_exports :
    gemmKernelModule.exports =
      [{ name := Wasm.gemmExportName, desc := .func 0 },
       { name := Wasm.memoryExportName, desc := .mem 0 }] :=
  GNAF.compile_exports GNAF.gemmKernelChecked

/-- The compiled kernel exports the `gemm` function and the memory, and nothing
else at all. -/
theorem gemmKernelModule_hasExactGemmExports :
    Universal.HasExactGemmExports wasmProfile gemmKernelModule := by
  have h := gemmKernelModule_validate
  unfold Wasm.validate at h
  simp only [Bool.and_eq_true] at h
  refine ⟨h.1.1.2, h.1.2, ?_⟩
  intro e he
  rw [gemmKernelModule_exports] at he
  simp only [List.mem_cons, List.not_mem_nil, or_false] at he
  rcases he with rfl | rfl
  · exact Or.inl rfl
  · exact Or.inr rfl

/-- **SPEC §10.1.**  The compiled GEMM kernel's bytes are profile valid: they
decode, the module validates under the profile's limit table, it imports
nothing, and it carries exactly the two required exports.  Hypothesis-free, on
a closed literal, and — unlike `Release.witness_profileValid` — on a module
compiled from a plan that computes a product. -/
theorem gemmKernel_profileValid : Universal.ProfileValid wasmProfile gemmKernelBytes :=
  ⟨gemmKernelModule, gemmKernel_decode, gemmKernelModule_validateUnder,
    gemmKernelModule_imports, gemmKernelModule_hasExactGemmExports⟩

/-- The initial configuration of the compiled kernel at a plain raw
invocation. -/
def gemmKernelInitialOf (raw : Wasm.RawInvocation) : Wasm.Config :=
  GNAF.compiledInitialConfig (GNAF.envOf GNAF.gemmKernelSig GNAF.gemmKernel)
    (GNAF.bodyCode (GNAF.envOf GNAF.gemmKernelSig GNAF.gemmKernel)
      GNAF.gemmKernelSig.scratch GNAF.gemmKernel) raw

/-- Initialization of the compiled kernel never faults. -/
theorem gemmKernel_initialConfig (raw : Wasm.RawInvocation) :
    Wasm.initialConfig gemmKernelModule raw = .ok (gemmKernelInitialOf raw) :=
  GNAF.compile_initialConfig GNAF.gemmKernelChecked raw

/-- The initial configuration of the compiled kernel at a GEMM raw
invocation. -/
def gemmKernelInitial (raw : Gemm.RawInvocation wasmProfile) : Wasm.Config :=
  gemmKernelInitialOf (Wasm.rawOfInvocation (Universal.toWasmInvocation raw))

/-- **The kernel's static step bound is strictly inside the released costed step
budget.**  `GNAF.gemmKernel_stepBound_lt_maxSteps` against
`(setting seam).problem.maxSteps` itself. -/
theorem gemmKernel_stepBound_lt_maxSteps :
    GNAF.gemmKernelChecked.plan.stepBound < (setting seam).problem.maxSteps :=
  GNAF.gemmKernel_stepBound_lt_maxSteps

/-- **The first missing fact.**  Every reduction of the compiled kernel out of
its initial configuration stays inside the released costed step budget.

This is the termination half of SPEC §11.4's `compile_refines`, and it is
**not proved anywhere in this repository**: `Plan.steps_le_stepBound` bounds the
GNAF interpreter `Plan.eval`, `GNAF.compile_resources` bounds the emitted
module's static instruction count, and no theorem relates either to the length
of a `Wasm.Reduces` sequence.  Proving it needs a simulation between `Wasm.Step`
on `GNAF.bodyCode` and `Plan.eval`, or at least a direct Wasm-level trip-count
argument for every `countLoopVar` the compiler emits — the counter local is
incremented by one per iteration and compared unsigned against an extent local
that the loop body never writes, so the argument exists in outline and is simply
not mechanized here.

The bound is stated against the budget rather than against
`Plan.stepBound` on purpose: one plan node compiles to several Wasm
instructions, so a Wasm trace is *not* bounded by the plan's step count, and a
hypothesis that said so could be false and would make everything below vacuous.
`Release.gemmKernelReducesBounded_of_stepBound_multiple` records exactly how
much slack there is.  It appears below only as an explicit hypothesis. -/
def GemmKernelReducesBounded : Prop :=
  ∀ (raw : Gemm.RawInvocation wasmProfile) {tr : List Wasm.Event} {f : Wasm.Config},
    Wasm.Reduces (gemmKernelInitial raw) tr f →
      tr.length ≤ (setting seam).problem.maxSteps

/-- **The second missing fact.**  Some branch of the compiled kernel reaches a
*terminal* configuration inside the same budget.

`Wasm.observationOfConfig` is `none` on a running configuration, so a
stuck-but-running configuration produces no observation at all and the costed
frontier would be empty.  Excluding that is `Wasm.validation_progress`, listed
OUTSTANDING under `O-6` in `Theorems/Status.lean`; it appears below only as an
explicit hypothesis. -/
def GemmKernelReachesTerminal : Prop :=
  ∀ raw : Gemm.RawInvocation wasmProfile,
    ∃ (tr : List Wasm.Event) (f : Wasm.Config) (o : Wasm.ExecutionObservation),
      Wasm.Reduces (gemmKernelInitial raw) tr f ∧
        tr.length ≤ (setting seam).problem.maxSteps ∧
        Wasm.observationOfConfig tr f = some o

/-- **How much slack the plan's step bound leaves.**  A Wasm-level bound of
*any* fixed multiple `K ≤ 2 ^ 220` of the plan's static step bound already
implies `Release.GemmKernelReducesBounded`: the plan bound is about `2 ^ 99`
(`GNAF.gemmKernel_stepBound_eq`) and the released budget is `2 ^ 320`.  So what
is missing is the qualitative fact that the compiled loops terminate at all, and
not headroom in the budget — which is why weakening the budget would be the
wrong repair. -/
theorem gemmKernelReducesBounded_of_stepBound_multiple (K : Nat) (hK : K ≤ 2 ^ 220)
    (h : ∀ (raw : Gemm.RawInvocation wasmProfile) {tr : List Wasm.Event}
      {f : Wasm.Config}, Wasm.Reduces (gemmKernelInitial raw) tr f →
        tr.length ≤ K * GNAF.gemmKernelChecked.plan.stepBound) :
    GemmKernelReducesBounded := by
  intro raw tr f hred
  refine Nat.le_trans (h raw hred) ?_
  show K * GNAF.gemmKernelChecked.plan.stepBound ≤ 2 ^ 320
  exact Nat.le_trans (Nat.mul_le_mul hK (Nat.le_refl _)) (by decide)

/-! ### 5.13.1 Generic bridges from a reduction bound to the explorer

These three are the general form of the ad-hoc unfoldings §5.10 performs on the
three-step witness module.  They are proved here, without hypotheses about any
particular module: a length bound on every reduction kills every still-running
prefix, one terminal branch makes the tree nonempty, and a length bound rules
out divergence. -/

/-- **A bound on every reduction length empties the still-running prefixes.**
If no reduction out of `c` is longer than `N` and the fuel exceeds `N`, then
`Wasm.prefixes` reports nothing: no branch is still running at the horizon. -/
theorem prefixes_eq_nil_of_reduces_bound {c : Wasm.Config} {N fuel : Nat}
    (hlt : N < fuel)
    (hb : ∀ {tr : List Wasm.Event} {f : Wasm.Config},
      Wasm.Reduces c tr f → tr.length ≤ N) :
    Wasm.prefixes fuel [] c = [] := by
  cases hp : Wasm.prefixes fuel [] c with
  | nil => rfl
  | cons t ts =>
      exfalso
      have hm : t ∈ Wasm.prefixes fuel [] c := by rw [hp]; exact List.mem_cons_self
      obtain ⟨suffix, final, ht, hred, -⟩ := Wasm.prefixes_sound fuel [] c t hm
      have hlen := Wasm.prefixes_length fuel [] c t hm
      rw [ht] at hlen
      simp only [List.nil_append, List.length_nil, Nat.zero_add] at hlen
      have := hb hred
      omega

/-- **One terminal branch makes the explored tree nonempty.** -/
theorem exploreTree_ne_nil_of_reduces_terminal {c f : Wasm.Config}
    {tr : List Wasm.Event} {o : Wasm.ExecutionObservation} {fuel : Nat}
    (hred : Wasm.Reduces c tr f) (hobs : Wasm.observationOfConfig tr f = some o)
    (hlen : tr.length ≤ fuel) : Wasm.exploreTree fuel [] c ≠ [] := by
  have hm : o ∈ Wasm.exploreTree fuel [] c :=
    Wasm.exploreTree_complete fuel [] tr c f o hred (by simpa using hobs) hlen
  intro h
  rw [h] at hm
  simp at hm

/-- An infinite step stream contains reductions of every length. -/
theorem reduces_of_stream (events : Nat → Wasm.Event) (configs : Nat → Wasm.Config)
    (hstep : ∀ i, Wasm.Step (configs i) (events i) (configs (i + 1))) :
    ∀ (n k : Nat), ∃ tr : List Wasm.Event,
      tr.length = n ∧ Wasm.Reduces (configs k) tr (configs (k + n)) := by
  intro n
  induction n with
  | zero => intro k; exact ⟨[], rfl, Wasm.Reduces.refl (configs k)⟩
  | succ m ih =>
      intro k
      obtain ⟨tr, hlen, hred⟩ := ih (k + 1)
      have hk : k + 1 + m = k + (m + 1) := by omega
      rw [hk] at hred
      exact ⟨events k :: tr, by simp [hlen], .cons (hstep k) hred⟩

/-! ### 5.13.2 The costed machine on the compiled kernel, modulo those two facts -/

/-- Under the reduction bound, the compiled kernel never diverges. -/
theorem gemmKernel_not_diverges (h₁ : GemmKernelReducesBounded)
    (raw : Gemm.RawInvocation wasmProfile)
    (events : Nat → Wasm.Event) (configs : Nat → Wasm.Config)
    (starts : configs 0 = gemmKernelInitial raw)
    (hstep : ∀ i, Wasm.Step (configs i) (events i) (configs (i + 1))) : False := by
  obtain ⟨tr, hlen, hred⟩ :=
    reduces_of_stream events configs hstep ((setting seam).problem.maxSteps + 1) 0
  rw [starts] at hred
  have := h₁ raw hred
  omega

/-- Under the reduction bound, every finite execution of the compiled kernel has
a trace inside the released costed step budget. -/
theorem gemmKernel_finiteExecution_length (h₁ : GemmKernelReducesBounded)
    (raw : Gemm.RawInvocation wasmProfile) {o : Wasm.ExecutionObservation}
    (h : Wasm.FiniteExecution (gemmKernelInitial raw) o) :
    o.trace.length ≤ (setting seam).problem.maxSteps := by
  obtain ⟨final, hred, -⟩ := h
  exact h₁ raw hred

/-- Under the reduction bound, no branch of the compiled kernel is still running
at the released horizon. -/
theorem gemmKernel_prefixes_head_none (h₁ : GemmKernelReducesBounded)
    (raw : Gemm.RawInvocation wasmProfile) :
    (Wasm.prefixes ((setting seam).problem.maxSteps + 1) []
      (gemmKernelInitial raw)).head? = none := by
  rw [prefixes_eq_nil_of_reduces_bound (Nat.lt_succ_self _) (h₁ raw)]
  rfl

/-- Under the terminal-branch fact, the compiled kernel produces at least one
observation. -/
theorem gemmKernel_exploreTree_ne_nil (h₂ : GemmKernelReachesTerminal)
    (raw : Gemm.RawInvocation wasmProfile) :
    Wasm.exploreTree ((setting seam).problem.maxSteps + 1) []
      (gemmKernelInitial raw) ≠ [] := by
  obtain ⟨tr, f, o, hred, hlen, hobs⟩ := h₂ raw
  exact exploreTree_ne_nil_of_reduces_terminal hred hobs (Nat.le_succ_of_le hlen)

/-- The seam's machine initializes with `Wasm.initialGemmInvocationCosted`: a
projection equation, so it holds without unfolding either the module or the
validator. -/
theorem machine_initialGemmInvocationCosted (m : Wasm.Module)
    (inv : Wasm.Invocation wasmProfile) :
    (setting seam).machine.initialGemmInvocationCosted m inv =
      Wasm.initialGemmInvocationCosted m inv := rfl

/-- **Costed initialization succeeds on the compiled kernel**, unconditionally:
this half needs neither missing fact. -/
theorem gemmKernel_machine_init (raw : Gemm.RawInvocation wasmProfile) :
    (setting seam).machine.initialGemmInvocationCosted gemmKernelModule
        (Universal.toWasmInvocation raw) =
      .ok { initial := gemmKernelInitial raw,
            cost := Wasm.initializationCost wasmProfile gemmKernelModule } := by
  rw [machine_initialGemmInvocationCosted]
  exact Wasm.initialGemmInvocationCosted_ok (gemmKernel_initialConfig _)

/-- **The costed explorer completes with a nonempty frontier on the compiled
kernel**, on every raw invocation — given the two facts of §5.13. -/
theorem gemmKernel_machine_explore (h₁ : GemmKernelReducesBounded)
    (h₂ : GemmKernelReachesTerminal) (raw : Gemm.RawInvocation wasmProfile) :
    ∃ (frontier : Foundation.NonemptyCanonicalFrontier
          (Universal.CostedExecutionObservation (setting seam).semantics
            (gemmKernelInitial raw)))
      (coverage : Universal.CostedCoverage (setting seam).semantics
          (setting seam).problem.maxSteps (gemmKernelInitial raw) frontier),
      (setting seam).machine.exploreAllCosted (setting seam).problem.maxSteps
          (gemmKernelInitial raw) = .complete frontier coverage :=
  Wasm.exploreAllCosted_complete_of_exploreTree_ne_nil semantics
    (setting seam).problem.maxSteps (gemmKernelInitial raw)
    (gemmKernel_prefixes_head_none h₁ raw) (gemmKernel_exploreTree_ne_nil h₂ raw)

/-- **BI-006, conditional form.**  On the module compiled from `GNAF.gemmKernel`
— a module that really computes a product — the costed machine initializes and
returns a completed, nonempty, canonically ordered costed frontier at every raw
invocation, **given** the two facts §5.13 names and does not prove.

The unconditional statement is not available: see `GemmKernelReducesBounded` and
`GemmKernelReachesTerminal` for exactly what is missing.  The unconditional
`Release.seam_machine_completes` remains the one on `Release.witnessModule`. -/
theorem gemmKernel_machine_completes_of (h₁ : GemmKernelReducesBounded)
    (h₂ : GemmKernelReachesTerminal) (raw : Gemm.RawInvocation wasmProfile) :
    seam.machine.initialGemmInvocationCosted gemmKernelModule
        (Universal.toWasmInvocation raw) =
      .ok { initial := gemmKernelInitial raw,
            cost := Wasm.initializationCost wasmProfile gemmKernelModule } ∧
    ∃ (frontier : Foundation.NonemptyCanonicalFrontier
          (Universal.CostedExecutionObservation semantics (gemmKernelInitial raw)))
      (coverage : Universal.CostedCoverage semantics
          (setting seam).problem.maxSteps (gemmKernelInitial raw) frontier),
      seam.machine.exploreAllCosted (setting seam).problem.maxSteps
          (gemmKernelInitial raw) = .complete frontier coverage :=
  ⟨gemmKernel_machine_init raw, gemmKernel_machine_explore h₁ h₂ raw⟩

theorem gemmKernel_not_nonterminal (h₁ : GemmKernelReducesBounded)
    (h₂ : GemmKernelReachesTerminal) (raw : Gemm.RawInvocation wasmProfile)
    (overrun : Universal.NonterminalPrefix (gemmKernelInitial raw)
      ((setting seam).problem.maxSteps + 1)) :
    (setting seam).machine.exploreAllCosted (setting seam).problem.maxSteps
        (gemmKernelInitial raw) ≠ .nonterminalPrefix overrun := by
  obtain ⟨f, c, h⟩ := gemmKernel_machine_explore h₁ h₂ raw
  rw [h]
  intro hh
  simp at hh

theorem gemmKernel_not_initializationFailure (h₁ : GemmKernelReducesBounded)
    (h₂ : GemmKernelReachesTerminal) (raw : Gemm.RawInvocation wasmProfile)
    (fault : Wasm.InstantiationFault) :
    (setting seam).machine.exploreAllCosted (setting seam).problem.maxSteps
        (gemmKernelInitial raw) ≠ .initializationFailure fault := by
  obtain ⟨f, c, h⟩ := gemmKernel_machine_explore h₁ h₂ raw
  rw [h]
  intro hh
  simp at hh

/-- The completed exploration of the compiled kernel, as data. -/
def gemmKernelCompleteWitness (h₁ : GemmKernelReducesBounded)
    (h₂ : GemmKernelReachesTerminal) (raw : Gemm.RawInvocation wasmProfile) :
    Σ' (frontier : Foundation.NonemptyCanonicalFrontier
          (Universal.CostedExecutionObservation (setting seam).semantics
            (gemmKernelInitial raw)))
       (coverage : Universal.CostedCoverage (setting seam).semantics
          (setting seam).problem.maxSteps (gemmKernelInitial raw) frontier),
      (setting seam).machine.exploreAllCosted (setting seam).problem.maxSteps
        (gemmKernelInitial raw) = .complete frontier coverage :=
  completeWitnessOf
    ((setting seam).machine.exploreAllCosted (setting seam).problem.maxSteps
      (gemmKernelInitial raw))
    (gemmKernel_not_nonterminal h₁ h₂ raw) (gemmKernel_not_initializationFailure h₁ h₂ raw)

/-- One `Universal.InputEvaluation` of the compiled kernel. -/
def gemmKernelInputEvaluationOf (raw : Gemm.RawInvocation wasmProfile)
    (frontier : Foundation.NonemptyCanonicalFrontier
      (Universal.CostedExecutionObservation (setting seam).semantics
        (gemmKernelInitial raw)))
    (coverage : Universal.CostedCoverage (setting seam).semantics
      (setting seam).problem.maxSteps (gemmKernelInitial raw) frontier)
    (h : (setting seam).machine.exploreAllCosted (setting seam).problem.maxSteps
      (gemmKernelInitial raw) = .complete frontier coverage) :
    Universal.InputEvaluation (setting seam) gemmKernelModule raw where
  initial := gemmKernelInitial raw
  initialization :=
    { initial := gemmKernelInitial raw
      cost := Wasm.initializationCost wasmProfile gemmKernelModule }
  initialConfigEq := rfl
  initialEq := gemmKernel_machine_init raw
  observations := frontier
  treeComplete := ⟨_, h⟩
  resourceVector :=
    Cost.sequentialCompose (Wasm.initializationCost wasmProfile gemmKernelModule)
      (Cost.maxOverCosts (frontier.elements.map (·.cost)))
  resourceExact := rfl

/-- **SPEC §10.1, `CoversEveryMaximalExecution`** for the compiled kernel: the
finite executions are inside the bound by `GemmKernelReducesBounded`, and that
same fact excludes the divergent ones. -/
theorem gemmKernelInputEvaluationOf_covers (h₁ : GemmKernelReducesBounded)
    (raw : Gemm.RawInvocation wasmProfile)
    (frontier : Foundation.NonemptyCanonicalFrontier
      (Universal.CostedExecutionObservation (setting seam).semantics
        (gemmKernelInitial raw)))
    (coverage : Universal.CostedCoverage (setting seam).semantics
      (setting seam).problem.maxSteps (gemmKernelInitial raw) frontier)
    (h : (setting seam).machine.exploreAllCosted (setting seam).problem.maxSteps
      (gemmKernelInitial raw) = .complete frontier coverage) :
    (gemmKernelInputEvaluationOf raw frontier coverage h).CoversEveryMaximalExecution := by
  intro execution
  cases execution with
  | finite o run maximal =>
      have hlen : o.trace.length ≤ (setting seam).problem.maxSteps :=
        gemmKernel_finiteExecution_length h₁ _ run
      obtain ⟨c, hc, hco⟩ := coverage.2 o run hlen
      exact ⟨c, hc, hco.symm⟩
  | diverges events configs starts step =>
      exact (gemmKernel_not_diverges h₁ _ events configs starts step).elim

/-- The per-input evaluation of the compiled kernel, at every raw invocation. -/
def gemmKernelPerInput (h₁ : GemmKernelReducesBounded) (h₂ : GemmKernelReachesTerminal)
    (raw : Gemm.RawInvocation wasmProfile) :
    Universal.InputEvaluation (setting seam) gemmKernelModule raw :=
  gemmKernelInputEvaluationOf raw (gemmKernelCompleteWitness h₁ h₂ raw).1
    (gemmKernelCompleteWitness h₁ h₂ raw).2.1 (gemmKernelCompleteWitness h₁ h₂ raw).2.2

theorem gemmKernelPerInput_covers (h₁ : GemmKernelReducesBounded)
    (h₂ : GemmKernelReachesTerminal) (raw : Gemm.RawInvocation wasmProfile) :
    (gemmKernelPerInput h₁ h₂ raw).CoversEveryMaximalExecution :=
  gemmKernelInputEvaluationOf_covers h₁ raw _ _ _

section Evaluation

variable [Foundation.Fintype (Gemm.RawInvocation wasmProfile)]

/-- **SPEC §9.1.**  The complete charged system cost of the witness bytes: the
four static coordinates read off the profile's own counters, the dynamic sum
scaled by the released workload multiplicity, and the dynamic maximum over every
raw invocation. -/
def witnessCost : Cost.CompleteSystemCost where
  static :=
    { moduleBytes := witnessBytes.size
      decodeSteps := wasmProfile.body.costTableBody.decodeCost witnessBytes
      validationSteps := (setting seam).semantics.validationSteps witnessModule
      staticDataBytes := (setting seam).semantics.staticDataBytes witnessModule }
  dynamicSum :=
    Cost.scale (setting seam).problem.workloadRepetitions
      (Cost.fullDomainSum (fun raw => (witnessPerInput raw).resourceVector))
  dynamicMax := Cost.fullDomainMax (fun raw => (witnessPerInput raw).resourceVector)

/-- **A `Universal.SystemEvaluation` of a real byte sequence at the constructed
seam.**  Every field is a theorem of this file: the decode equation, one
completed costed exploration per raw invocation, maximal-execution coverage at
each of them, and the exact aggregate cost equation of SPEC §9.1. -/
def witnessSystemEvaluation :
    Universal.SystemEvaluation (setting seam) witnessBytes where
  module := witnessModule
  decodeEq := witness_decode
  perInput := witnessPerInput
  observationsComplete := witnessPerInput_covers
  cost := witnessCost
  costExact := ⟨Nat.le_refl 1, witness_decode, rfl, rfl, rfl, rfl, rfl, rfl⟩

/--
  **THE HEADLINE OF GO-008.**

  `Universal.SystemEvaluation` at the constructed release setting is inhabited.

  This is what defeats vacuity.  `Release.exists_globalOptimal_of_nonempty` and
  every theorem of `Theorems/Release.lean` quantify over `(seam : Seam)` and are
  gated on nonemptiness of the admissible, *evaluated* set; until now nothing
  inhabited `Seam` at all, so the whole family was an implication with a
  possibly-unsatisfiable antecedent.  It is now an implication about a
  constructed seam whose evaluation type is provably nonempty.

  **What this does not say.**  It does not say the witness bytes are
  `Universal.Admissible`: `Universal.ProfileValid` is proved
  (`Release.witness_profileValid`), but `Universal.SemanticCorrect` — which
  demands `Gemm.Reference.Accepts` at *every* raw invocation — is false for a
  module whose `gemm` returns the constant `0`, and is not claimed here.  The
  nonemptiness antecedent of the release theorem needs admissibility as well as
  evaluation, and that half stays open (`GO-006`, `BI-002`).
-/
theorem systemEvaluation_inhabited :
    ∃ bytes : ByteArray, Nonempty (Universal.SystemEvaluation (setting seam) bytes) :=
  ⟨witnessBytes, ⟨witnessSystemEvaluation⟩⟩

/-- The release decider answers `.complete` on the witness bytes: the decider,
the machine and the evaluation agree on a concrete literal. -/
theorem seam_decider_complete_on_witness :
    (decider seam).evaluate witnessBytes = .complete witnessSystemEvaluation :=
  evaluateClassically_eq_complete seam witnessSystemEvaluation

/-- **Exactly what is left of the release antecedent at the constructed seam.**
Conjuncts (a) profile validity and (d) evaluation are discharged on a closed
literal; conjuncts (b) and (c) are the caller's, and this repository supplies
neither. -/
theorem globalOptimal_of_witness_semantics
    (hcorrect : Universal.SemanticCorrect (setting seam) witnessBytes)
    (hresources : Universal.SemanticWithinResources (setting seam) witnessBytes) :
    ∃ bytes : ByteArray,
      Universal.GlobalOptimal (setting seam) (decider seam) (costObjective seam)
        bytes :=
  exists_globalOptimal_of_admissible_evaluation seam
    ⟨witnessBytes, ⟨witness_profileValid, hcorrect, hresources⟩,
      ⟨witnessSystemEvaluation⟩⟩

/-! ### 5.14 The evaluation of the compiled GEMM kernel, modulo §5.13's two facts -/

/-- **SPEC §9.1.**  The complete charged system cost of the compiled kernel's
bytes, in the same shape as `Release.witnessCost`. -/
def gemmKernelCost (h₁ : GemmKernelReducesBounded) (h₂ : GemmKernelReachesTerminal) :
    Cost.CompleteSystemCost where
  static :=
    { moduleBytes := gemmKernelBytes.size
      decodeSteps := wasmProfile.body.costTableBody.decodeCost gemmKernelBytes
      validationSteps := (setting seam).semantics.validationSteps gemmKernelModule
      staticDataBytes := (setting seam).semantics.staticDataBytes gemmKernelModule }
  dynamicSum :=
    Cost.scale (setting seam).problem.workloadRepetitions
      (Cost.fullDomainSum (fun raw => (gemmKernelPerInput h₁ h₂ raw).resourceVector))
  dynamicMax :=
    Cost.fullDomainMax (fun raw => (gemmKernelPerInput h₁ h₂ raw).resourceVector)

/-- **BI-006, conditional form.**  A `Universal.SystemEvaluation` at the
constructed seam of the bytes of a module that really computes a product —
**given** `Release.GemmKernelReducesBounded` and
`Release.GemmKernelReachesTerminal`, neither of which is proved anywhere in this
repository (see §5.13 for exactly what each one needs).

The unconditional inhabitant of `Universal.SystemEvaluation` at this seam
remains `Release.witnessSystemEvaluation`, on the module whose `gemm` returns
the constant `0`. -/
def gemmKernelSystemEvaluationOf (h₁ : GemmKernelReducesBounded)
    (h₂ : GemmKernelReachesTerminal) :
    Universal.SystemEvaluation (setting seam) gemmKernelBytes where
  module := gemmKernelModule
  decodeEq := gemmKernel_decode
  perInput := gemmKernelPerInput h₁ h₂
  observationsComplete := gemmKernelPerInput_covers h₁ h₂
  cost := gemmKernelCost h₁ h₂
  costExact := ⟨Nat.le_refl 1, gemmKernel_decode, rfl, rfl, rfl, rfl, rfl, rfl⟩

/-- The same fact in existential form: under §5.13's two facts, the *kernel*
bytes — not just the witness bytes — carry an evaluation at the constructed
seam. -/
theorem gemmKernel_systemEvaluation_of (h₁ : GemmKernelReducesBounded)
    (h₂ : GemmKernelReachesTerminal) :
    Nonempty (Universal.SystemEvaluation (setting seam) gemmKernelBytes) :=
  ⟨gemmKernelSystemEvaluationOf h₁ h₂⟩

end Evaluation

end WasmGemmGnaf.Release
