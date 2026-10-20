/-
# SPEC §15 required-declaration ledger

This module carries no mathematics.  It is the single place where every one of
the 58 declarations SPEC §15 requires is listed with **either** the Lean name
that discharges it **or** the obligation that blocks it.  A name appears in the
"discharged by" column only if a kernel-checked, `sorry`-free proof of that
proposition exists in this repository.  Where the repository proves something
*near* a required name but strictly weaker, the required name is recorded as
OUTSTANDING and the nearer result is named as such — never promoted.

Score: **25 of 58 discharged, 33 outstanding.**

One of the 25, `Wasm.costed_erase_iff_plain_run`, is discharged in the `DEV-001`
amended form because SPEC's literal biconditional is false as written; the row
says so.

## Obligation legend

The obligation IDs are the ones already fixed by `CERTIFICATION.md` §2, so this
ledger and the certification document cannot drift apart:

| ID | Obligation |
|----|------------|
| `O-1` | Competitor universe defined extensionally over all byte strings — *definable; stated at full strength in `Universal/Competitor.lean`* |
| `O-2` | Sublevel is finite and decidable — *closable; `Cost.objective_sublevel_finite` is the hinge* |
| `O-3` | Complete admission: `SystemEvaluationRel` sound / complete / functional |
| `O-4` | Attainment: the shipped bytes' exact score computed |
| `O-5` | A universal lower bound `F`, attained — **no known technique** |
| `O-6` | Mechanized Wasm Core 3.0 semantics (GC, EH, SIMD, tail calls) and a compiler that can emit the four SPEC §8.2 arithmetic modes |

Two supporting records are cited where they are the precise reason:

* `DEV-001` (`model/spec-deviations.json`) — the literal SPEC §7.5 erasure
  biconditional is false as written; the amended form and the unconditional
  intent are both proved.
* `BI-002` / `BI-003` (`model/claims.json`) — `GNAF.compile` emits `unreachable`
  for the `checked`, `strictFloat` and `exactDyadicRoundOnce` arithmetic modes,
  so SPEC §13 Phase B (an input-total scalar GNAF GEMM plan) is not achievable
  with the current compiler and target.

`O-6` blocks `O-4` structurally: no baseline means no attained upper bound,
hence no sublevel, hence no argmin.  `O-5` is obstructed independently.

## Wasm — 4 of 11 discharged

| SPEC §15 name | discharged by / blocked by |
|---|---|
| `Wasm.decode_sound` | **OUTSTANDING — `O-6`.**  SPEC §7.3 states this against the vendored `DeclarativeBinaryRelation`, which is not mechanized here.  Nearest proved: `WasmGemmGnaf.Wasm.decode_is_encode` and `WasmGemmGnaf.Wasm.decode_error_or_encode` (intrinsic to this repository's codec, not to the pinned grammar). |
| `Wasm.decode_complete` | **OUTSTANDING — `O-6`.**  Nearest proved: `WasmGemmGnaf.Wasm.encode_decode_roundtrip`. |
| `Wasm.validate_iff_declarative` | **OUTSTANDING — `O-6`.**  Nearest proved: `WasmGemmGnaf.Wasm.validate_bool_iff`, which decides `Wasm.DeclarativelyValid` — this repository's judgment for the `i32` executable subset, not Core 3.0 validation. |
| `Wasm.validation_progress` | **OUTSTANDING — `O-6`.**  No progress theorem exists. |
| `Wasm.mem_successors_iff_step` | `WasmGemmGnaf.Wasm.mem_successors_iff_step`, re-indexed as `Theorems.mem_successors_iff_step`. |
| `Wasm.bounded_tree_covers_every_branch` | **OUTSTANDING — `O-6`.**  Absent. |
| `Wasm.runFuel_sound` | `WasmGemmGnaf.Wasm.runFuel_sound` (`Wasm/Fuel.lean`). |
| `Wasm.runFuel_complete_with_bound` | `WasmGemmGnaf.Wasm.runFuel_complete_with_bound` (`Wasm/Fuel.lean`). |
| `Wasm.costed_erase_iff_plain_run` | `WasmGemmGnaf.Wasm.costed_erase_iff_plain_run`, re-indexed as `Theorems.costed_erase_iff_plain_run`, in the `DEV-001` amended form; the unconditional intent is `Theorems.costed_run_iff_plain_run`. |
| `Wasm.costed_initialization_erase` | **OUTSTANDING — `O-6`.**  `Wasm/Erasure.lean` covers the reduction phase only. |
| `Wasm.profile_matches_pinned_revision` | **OUTSTANDING — `O-6`.**  Nearest proved: `WasmGemmGnaf.Wasm.ProfileLawful.revisionCommit`, which fixes the pinned commit *string* on a lawful profile body; it does not say the modelled semantics match that revision. |

## Gemm — 10 of 10 discharged

| SPEC §15 name | discharged by / blocked by |
|---|---|
| `Gemm.classify_total` | `WasmGemmGnaf.Gemm.classify_total`, re-indexed as `Theorems.classify_total`.  (`WasmGemmGnaf.GNAF.classify_total` is a different, unrelated theorem about `GNAF.Machine`.) |
| `Gemm.reference_total` | `WasmGemmGnaf.Gemm.reference_total` (`Gemm/Reference.lean`), re-indexed as `Theorems.reference_total`: every raw invocation — malformed, truncated, unsupported, resource-invalid or valid — has an accepted observation. |
| `Gemm.valid_input_finite` | `WasmGemmGnaf.Gemm.valid_input_finite` (`Universal/EnumerateInputs.lean`): a global, constructive `Foundation.Fintype` on `{raw : RawInvocation P // refValid raw.body = true}`, the subtype `Gemm.refValid_iff_classify` identifies with the classifier's `valid` verdict.  The enumeration `validRawInvocations` and both of its proofs are choice-free; the *carrier predicate* inherits `Classical.choice` from the pre-existing `Gemm/Layout.lean` proofs behind `Decidable (View.ElementsInInterval …)`, exactly as `Gemm.classify_total` and `Gemm.reference_total` already do. |
| `Gemm.raw_input_finite` | `WasmGemmGnaf.Gemm.raw_input_finite` (`Universal/EnumerateInputs.lean`): SPEC §8.4's `instance problem_input_fintype`, as a **global** instance with no instance hypothesis.  `Gemm.rawInvocations` ranges `ptr` and `len` over `Gemm.allUInt32` and `bytes` over `Gemm.byteArraysOfSize len.toNat`, attaching the SPEC §8.3 lawfulness proof; `Gemm.mem_rawInvocations` and `Gemm.rawInvocations_nodup` are proved structurally, never by evaluation.  Axiom closure: `[propext, Quot.sound]` — no `Classical.choice`.  The hypothesis `[Foundation.Fintype (Gemm.RawInvocation P)]` still *appears* on the older `Universal` and `Artifact` results, which were written before the instance existed; it is now discharged by this instance rather than assumed. |
| `Gemm.raw_invocation_roundtrip` | `WasmGemmGnaf.Gemm.raw_invocation_roundtrip`, re-indexed as `Theorems.raw_invocation_roundtrip`. |
| `Gemm.raw_invocation_surjective` | `WasmGemmGnaf.Gemm.raw_invocation_surjective`, re-indexed as `Theorems.raw_invocation_surjective`. |
| `Gemm.abi_roundtrip` | `WasmGemmGnaf.Gemm.abi_roundtrip`, re-indexed as `Theorems.abi_roundtrip`. |
| `Gemm.classifier_exact_domain` | `WasmGemmGnaf.Gemm.classifier_exact_domain`, re-indexed as `Theorems.classifier_exact_domain`. |
| `Gemm.mandatory_family_nonzero_witnesses` | `WasmGemmGnaf.Gemm.mandatory_family_nonzero_witnesses` (`Gemm/Reference.lean`), re-indexed as `Theorems.mandatory_family_nonzero_witnesses`, with `Gemm.mandatoryCases_covers` proving the family covers every mandatory combination.  **Scope**: this is a statement about the *classifier and the reference arithmetic*, which is what the `Gemm` namespace name asks for.  It does not claim the released compiler can emit code for those modes — that is `GNAF.compile_refines`, still blocked by `BI-002`/`BI-003`. |
| `Gemm.observation_covers_status_and_full_c` | `WasmGemmGnaf.Gemm.observation_covers_status_and_full_c` (`Gemm/Reference.lean`), re-indexed as `Theorems.observation_covers_status_and_full_c`. |

## Cost — 3 of 3 discharged

| SPEC §15 name | discharged by / blocked by |
|---|---|
| `Cost.module_bytes_exact` | `WasmGemmGnaf.Cost.module_bytes_exact` (`Cost/Aggregate.lean`), re-indexed as `Theorems.module_bytes_exact`.  Scope: `ExactAggregateCost` takes `decodes`/`decodeSteps`/`validationSteps`/`staticDataBytes` as parameters, so this is conditional on a cost vector being exact — it does not itself pin a released artifact's cost (`O-4`). |
| `Cost.transition_accounting_positive` | `WasmGemmGnaf.Cost.transition_accounting_positive` (`Cost/Event.lean`), re-indexed as `Theorems.transition_accounting_positive`. |
| `Cost.objective_sublevel_finite` | `WasmGemmGnaf.Cost.objective_sublevel_finite`, re-indexed as `Theorems.objective_sublevel_finite`. |

The Cost layer is complete against SPEC §15.  What is missing downstream is not
the objective but anything to apply it to.

## GNAF and the emitter — 3 of 5 discharged

| SPEC §15 name | discharged by / blocked by |
|---|---|
| `GNAF.normalize_semantics` | `WasmGemmGnaf.GNAF.normalize_semantics` (`GNAF/Normalize.lean`). |
| `GNAF.normalize_cost_le` | `WasmGemmGnaf.GNAF.normalize_cost_le` (`GNAF/Normalize.lean`). |
| `GNAF.compile_refines` | **OUTSTANDING — `O-6`** (`BI-002`).  `GNAF/CompileCorrect.lean` proves many compilation invariants (`compile_initialConfig`, `compile_runInvariant`, `compile_body_reachable`, `compile_emit_decodes_valid`, …) but no refinement of the GEMM reference semantics by the compiled module. |
| `GNAF.compile_cost_exact` | **OUTSTANDING — `O-6`.**  Absent. |
| `Artifact.decode_emit` | `WasmGemmGnaf.Artifact.decode_emit` (`Artifact/Emit.lean`); `emit` is *defined* as `Wasm.encode`, so this is the verified codec's round trip transported along the definition. |

## Universal — 1 of 12 discharged

`Universal/Competitor.lean`, `Correct.lean`, `Feasible.lean`, `Sublevel.lean`,
`LowerBound.lean`, `BilinearLowerBound.lean`, `Partition.lean` and `Argmin.lean`
exist and are proof-carrying, but none of them proves a §15 name: they establish
the *definitions* at full strength and the algebraic facts around them.
`Universal/EnumerateInputs.lean` now exists and proves one.
`Universal/EnumerateBytes.lean`, `CheckExecution.lean` and `Coverage.lean` do
not exist.

| SPEC §15 name | discharged by / blocked by |
|---|---|
| `Universal.possible_winner_within_sublevel` | **OUTSTANDING — `O-4`, `O-6`.**  Needs an attained upper bound, which needs a baseline.  Nearest proved: `WasmGemmGnaf.Universal.sublevel_bytes_size_le` and `sublevel_bytes_enumerated` (`UV-002`) — the finiteness half only. |
| `Universal.byte_enumerator_complete` | **OUTSTANDING — `O-4`, `O-6`.**  `Universal/EnumerateBytes.lean` does not exist.  Nearest proved: `WasmGemmGnaf.Universal.Enumerate.mem_boundedByteArrays_iff`, which is exactness of the *carrier* enumeration, not of a decoder/validator pipeline over it. |
| `Universal.input_enumerator_complete` | `WasmGemmGnaf.Universal.input_enumerator_complete` (`Universal/EnumerateInputs.lean`): every `raw : Gemm.RawInvocation P` occurs in `Universal.inputEnumerator P`, unconditionally — no `Fintype` hypothesis, no sublevel bound, no scope predicate.  `Universal.inputEnumerator_nodup` gives the exactness half. |
| `Universal.execution_checker_sound` | **OUTSTANDING — `O-3`, `O-6`.**  `Universal/CheckExecution.lean` does not exist. |
| `Universal.execution_checker_complete_within_sublevel` | **OUTSTANDING — `O-3`, `O-6`.** |
| `Universal.system_evaluation_rel_sound` | **OUTSTANDING — `O-3`, `O-6`.**  SPEC §10.1 states this as the *reflection biconditional* `(Correct ↔ SemanticCorrect) ∧ (Feasible ↔ SemanticWithinResources)` over the implemented `Universal.evaluate`, which does not exist here.  Nearest proved: `WasmGemmGnaf.Universal.correct_of_admissible` and `feasible_of_admissible` (the ⟸ direction of each, from the three extensional predicates), and `WasmGemmGnaf.Release.systemEvaluationRel_sound`, which is *structural* soundness (decode equation, coverage obligation, exact aggregate cost) at the classical release decider, not the biconditional. |
| `Universal.system_evaluation_rel_complete` | **OUTSTANDING — `O-3`, `O-6`.**  SPEC's statement **asserts existence**: profile-valid + semantically correct + within resources ⟹ `∃ evaluation, SystemEvaluationRel …`.  That is exactly the open nonemptiness obligation.  `WasmGemmGnaf.Release.systemEvaluationRel_complete` takes the evaluation as an *argument* and is therefore strictly weaker; it must not be cited for this row. |
| `Universal.system_evaluation_rel_functional` | **OUTSTANDING — `O-3`.**  The proposition is proved for *every* setting and *every* decider by `WasmGemmGnaf.Universal.systemEvaluation_subsingleton` (with `inputEvaluation_subsingleton`), re-indexed at the release decider as `WasmGemmGnaf.Release.systemEvaluationRel_functional`.  It is still recorded OUTSTANDING for two reasons and no others: those theorems carry `[Foundation.Fintype (Gemm.RawInvocation P)]`, which is `O-3`; and SPEC indexes the relation by `Universal.evaluate`, which does not exist here.  This is the closest of the twelve to closing. |
| `Universal.partition_cover_complete` | **OUTSTANDING — `O-5`.**  `Universal/Partition.lean` exists and transcribes SPEC §10.4 in full, including the well-founded `split` recursion.  Nearest proved: `WasmGemmGnaf.Universal.coverLeaves_covers`, which is *conditional on the root cell's own denotation* — refinement loses nothing.  `coverLeaves_covers_scope` proves, machine-checked, that this cannot be read as coverage of all competitor bytes.  No `dominated` instance is constructed: its `memberLowerBound` field is `O-5` itself. |
| `Universal.universal_sublevel_coverage` | **OUTSTANDING — `O-5`** (`UV-001`).  SPEC §10.5 requires this to have *no* coverage hypothesis; `Atlas.universalCoverCompleteCheck_scope_blind` proves the recorded seal cover cannot supply it. |
| `Universal.selected_le_every_sublevel_member` | **OUTSTANDING — `O-4`, `O-5`.** |
| `Universal.all_competitors_lower_bound` | **OUTSTANDING — `O-5`.**  No known technique.  Nearest proved: `WasmGemmGnaf.Universal.attained_lower_bound_is_optimal` — which says an *attained* bound would suffice, and `lower_bound_below_released_is_not_optimality`, which says an unattained one would not. |

## Atlas — 4 of 10 discharged

| SPEC §15 name | discharged by / blocked by |
|---|---|
| `Atlas.semantic_closure_least` | `WasmGemmGnaf.Atlas.semantic_closure_least`, re-indexed as `Theorems.semantic_closure_least`. |
| `Atlas.attention_no_optimum_relevant_false_negative` | **OUTSTANDING — `O-3`, `O-5`.**  The statement needs a notion of optimum, which does not exist here.  Nearest proved: `WasmGemmGnaf.Atlas.attend_determined_by_index`, `attend_monotone`, `attend_blind_to_optimizer_state`. |
| `Atlas.invalidation_complete` | `WasmGemmGnaf.Atlas.invalidation_complete` (`Atlas/Dependency.lean`). |
| `Atlas.incremental_eq_full_rebuild` | `WasmGemmGnaf.Atlas.incremental_eq_full_rebuild`, re-indexed as `Theorems.incremental_eq_full_rebuild`, with the hypothesis `state.body.scope = Scope.unscoped`, which is required for truth.  The unrestricted content is `Theorems.incremental_eq_full_rebuild_scoped`, strengthened past canonicalisation by `Theorems.incremental_eq_full_rebuild_exact`. |
| `Atlas.seal_verifier_reconstructs_every_preimage` | **OUTSTANDING — `O-3`.**  Nearest proved: `WasmGemmGnaf.Atlas.resolvesEveryReferencedPreimage_iff` — *referenced* preimages only, which is strictly weaker. |
| `Atlas.seal_implies_universal_coverage` | **OUTSTANDING — `O-5`, and deliberately so.**  `Theorems.universalCoverCompleteCheck_scope_blind` proves the seal's cover check is a function of three recorded components and therefore cannot witness any proposition quantified over `ByteArray`.  Deriving this name from the seal would be unsound. |
| `Atlas.lifecycle_prefix_conservation` | `WasmGemmGnaf.Atlas.lifecycle_prefix_conservation` (`Atlas/Lifecycle.lean`), matching SPEC §16's statement.  Scope: it holds of every `Atlas.LifecycleEvaluation` because that structure's `totalExact` field *demands* the exact mixed fold — the content is that the carrier stores no unchecked total, not that some particular lifecycle was measured. |
| `Atlas.lifecycle_native_bound` | **OUTSTANDING — `O-6`.**  `Atlas/Lifecycle.lean` exists but omits this deliberately: the inequality is false for an arbitrary primitive-cost table and an arbitrary trace, and the coefficients that would make it true are a property of a release table this repository has not pinned.  Nearest proved: `WasmGemmGnaf.Atlas.nativeLifecycleBound_scope_size_only`, which records machine-checked what the definition alone gives. |
| `Atlas.lifecycle_incremental_semantics_eq_full_rebuild` | **OUTSTANDING — `O-6`.**  Omitted with reasons at the end of `Atlas/Lifecycle.lean`; `canonicalFullRebuildEvaluation` does not exist. |
| `Atlas.lifecycle_full_rebuild_comparator_exact` | **OUTSTANDING — `O-6`.**  Same omission. |

## Artifact — 0 of 7 discharged

The layer is `Artifact/Emit.lean`, `Artifact/Release.lean` and
`Artifact/Baseline.lean`; `Select.lean`, `Bytes.lean`, `Manifest.lean` and
`Execute.lean` do not exist.  `Artifact/Baseline.lean` does supply a closed
literal `Artifact.baselineBytes` (the encoding of `GNAF.compile
GNAF.gemmWitnessChecked`) and proves `Universal.ProfileValid Release.wasmProfile
baselineBytes` for it outright, hypothesis-free — but a profile-valid literal is
not a *selection*: no baseline **score** is computed, no `SemanticCorrect` and no
`SemanticWithinResources` proof exists, and no evaluation inhabits
`Universal.SystemEvaluation` at it.  `Artifact/Release.lean` §5 supplies a
*second* closed literal, `Release.witnessBytes`, which is profile valid **and**
carries a `Universal.SystemEvaluation` at the constructed seam — but its module
returns the constant `0`, so it is not semantically correct and is not a
candidate release either.  So none of the seven release theorems exists, and —
per SPEC §1 and UOR-GNAF §13.3 — none is asserted.

| SPEC §15 name | discharged by / blocked by |
|---|---|
| `Artifact.released_bytes_equal_selection` | **OUTSTANDING — `O-4`, `O-6`.**  No selection exists.  `Release.exists_globalOptimal_of_nonempty` produces an existential by classical reasoning; it names no byte literal, so there is nothing to equate a commitment to. |
| `Artifact.committed_literal_equal_selection` | **OUTSTANDING — `O-4`, `O-6`.** |
| `Artifact.released_bytes_decode` | **OUTSTANDING — `O-4`, `O-6`.** |
| `Artifact.released_bytes_validate` | **OUTSTANDING — `O-4`, `O-6`.** |
| `Artifact.released_input_total` | **OUTSTANDING — `O-6`** (`BI-002`/`BI-003`).  Input totality fails for three of the four mandatory arithmetic modes. |
| `Artifact.released_attains_lower_bound` | **OUTSTANDING — `O-5`.**  No universal lower bound exists to attain. |
| `Artifact.released_wasm_gemm_gnaf_global_optimal` | **OUTSTANDING — `O-4`, `O-5`, `O-6`, and NOT DECLARED ANYWHERE IN THIS REPOSITORY.**  SPEC §15 forbids it from accepting `Coverage`, `LowerBound`, `Correct`, `FaithfulWasm`, `CompilerCorrect` or `GlobalOptimal` as parameters, so it cannot be stated as a conditional and must not be stated at all until its antecedents close.  The nearest proved result is `Theorems.release_globalOptimal_of_nonempty`, which is **conditional** and is named after its hypothesis for exactly that reason.  The repository's terminal answer for claim `WGG-GO-1` remains `WorkloadIncomplete` (`CERTIFICATION.md`). |

## The release layer: what is proved, and the four things that are not

`Artifact/Release.lean` instantiates SPEC §7.2 / §8.3 / §9.3 / §10.1 at the
release scope and `Theorems/Release.lean` re-indexes the result at top level
with every hypothesis written out.  Nothing there discharges a §15 name.  What
it does is reduce the release theorem to a single open antecedent, and make the
reduction checkable:

* `Theorems.release_scope_identities` — the profile, problem, setting and
  objective identity equations in one conjunction, including the machine-checked
  cost profile — `Release.wasmProfile.costTableBody.ruleRows` is an exact,
  duplicate-free cover of every pinned `Wasm.RuleId` and
  `initializationRows` of every `Wasm.InitEventId`, with every rule resolving
  through `rowFor?` (this was CO-006; the rows used to be empty) — and the
  **disclosed deviation** that remains: `Release.wasmProfile` is
  `Wasm.unitWitnessProfile`, so its cost table is
  `Wasm.canonicalCostTableUnits`, whose rows are checked against this
  repository's own contribution law rather than built by
  `Wasm.buildCanonicalCostTable` from the Core 3.0 conformance map, which does
  not exist and for which `vendor/wasm-spec` carries no data.  No theorem about
  `Release.wasmProfile` may be cited as being about SPEC §7.2's release
  literal.
* `Theorems.release_decider_answers_admissible` — claim `UV-003` in
  `model/claims.json` / `CONFORMANCE.md`, discharged in Lean by
  `Release.deciderAnswersAdmissible`.  Those two records still say `open`; that
  is drift in the JSON registry, not in the proofs.
* `Theorems.release_globalOptimal_of_nonempty` — the release theorem, at the
  strength it actually has.
* `Theorems.release_lower_bound_clause` and `release_tie_break_clause` — the
  clauses of `GlobalOptimal` extracted at an *arbitrary* `competitorBytes :
  ByteArray`, which is the machine-checked record that the competitor
  quantifier was never narrowed.
* `Theorems.release_seam_nondegenerate`, `release_systemEvaluation_inhabited`
  and `release_globalOptimal_of_nonempty_at_seam` — claim `GO-008`: the
  `Release.Seam` is now a constructed closed term, it is proved not to be the
  degenerate one, and the release theorem is restated with the seam no longer a
  parameter.  None of this discharges a §15 name, and none of it makes any
  byte sequence `Universal.Admissible`.

Four hypotheses were recorded here; the first is now closed.  All four are
explicit arguments of those theorems:

1. `[Foundation.Fintype (Gemm.RawInvocation Release.wasmProfile)]` — SPEC §8.4's
   `problem_input_fintype`.  **CLOSED.**  `Gemm.raw_input_finite`
   (`Universal/EnumerateInputs.lean`) is a global constructive instance, so this
   hypothesis is now satisfied rather than assumed.  It is still written as an
   explicit instance binder on the theorems listed above, which predate the
   instance; that is a residual signature, not a residual assumption.
2. `Release.Seam.semantics` — **CLOSED (GO-008).**  `Release.semantics` is
   `Release.costEvent` together with `Wasm.validationCost` and
   `Wasm.instantiatedStaticBytes`.  Disclosure, and it is not small:
   `Release.costEvent` is a configuration-free map out of the *plain*
   `Wasm.Event`, which does not carry the transferred byte count, the
   `memory.grow` delta or the installed byte count, so it is a **lower bound**
   on SPEC §7.5's contribution law and not the law.
   `Release.costEvent_le_eventContribution` proves the inequality,
   `Release.costEvent_charge_wasmRuleSteps` proves no event is free, and
   `Release.costEvent_branch_charge` / `Release.costEvent_trap_charge` prove it
   is exact on the branch and trap rules.  Exactness on arithmetic, memory
   access, grow, `enterGemm`, `throw` and `return` remains `O-6`.
3. `Release.Seam.machine` — **CLOSED (GO-008).**  `Release.machine` is
   `Wasm.releaseCostedMachine` (`Wasm/CostedExplore.lean`): the real costed
   initialization and bounded all-branch costed explorer, with the canonical
   schema, canonical sort and frontier construction
   `Universal.CostedTreeResult.complete` demands.
   `Theorems.release_seam_nondegenerate` proves it initializes and returns a
   **completed, nonempty** frontier at every raw invocation of
   `Release.witnessModule` — the smallest module `GNAF.moduleOf` emits, whose
   `gemm` is `i32.const 0` and every branch of which terminates in at most three
   reduction steps.  It is **not** `Artifact.baselineModule`: discharging the
   explorer on the compiled GEMM witness needs a termination proof for
   `GNAF.bodyCode`'s loops inside a `2 ^ 320` step budget, which is `O-6`.
   (`Release.Seam.limit` is the same seam's third field, also closed:
   `Release.limit` pins five of `Cost.DynamicVector`'s sixteen coordinates
   verbatim from `Gemm.releaseResourceContract` and supplies the other eleven at
   the profile maximum the pinned costed-step budget implies —
   `Release.limit_pinned` and `Release.limit_unpinned` say which are which, and
   `Release.limit_pos` proves no coordinate is budgeted at zero.)
4. **Nonemptiness** — one byte sequence proved `ProfileValid ∧ SemanticCorrect ∧
   SemanticWithinResources` with a system evaluation.  **No such witness exists
   in this repository and none is asserted.**  It is the antecedent `hne`, and
   `Theorems.release_obligation_reduction` proves that nothing else is left in
   it.  Blocked by `O-6` (no compiler output proved correct) and `O-4`.

   Two of its four requirements are now met on a closed literal, and they are
   met on *different* literals from each other's strength:
   `Artifact.baseline_profileValid` proves
   `Universal.ProfileValid Release.wasmProfile Artifact.baselineBytes`
   hypothesis-free for the compiled GEMM witness, and
   `Theorems.release_systemEvaluation_inhabited` proves
   `Universal.ProfileValid` **and** `Nonempty (Universal.SystemEvaluation
   (Release.setting Release.seam) ·)` for `Release.witnessBytes`.  Before
   GO-008 the evaluation conjunct was not known to be satisfiable at any seam
   at all; it now is, at the constructed one.  What is still missing, and what
   keeps `hne` open, is that the literal carrying the evaluation is **not**
   semantically correct: `Release.witnessModule`'s `gemm` returns the constant
   `0`, so `Universal.SemanticCorrect` is false for it, and
   `SemanticWithinResources` is unproved for it.  For the compiled GEMM witness
   the same conjunct is `GNAF.compile_refines`, omitted under `BI-002`/`O-6`,
   and no `Universal.SystemEvaluation` inhabits it.
   `Artifact.baseline_admissible_iff` states the residue at the baseline
   exactly; `Artifact.exists_globalOptimal_of_baseline_semantics` and
   `Theorems.release_globalOptimal_of_witness_semantics` are **reductions, not
   discharges** — their `hcorrect`/`hresources`/`e` arguments are the open
   conjuncts, supplied by the caller, and in the second case `hcorrect` is a
   false hypothesis.

`Release.decider` is noncomputable — `Classical.propDecidable` on
`Nonempty (SystemEvaluation …)` and `Classical.choice` on the inhabitant, both
Prop-level, neither on an executable path.  It is **not** SPEC §10.1's
implemented decoder / validator / input enumerator / all-branch explorer: it
decodes nothing, enumerates nothing and explores nothing, and it produces no
executable witness.  It satisfies the *relational* contract, which is all
`GlobalOptimal` consumes, and that is the whole of its content.

## Files in `WasmGemmGnaf/Theorems/`

Only the modules whose contents are fully proved exist:

* `WasmModel.lean` — Wasm decode/encode, validation, reduction, faults, cost erasure, cost-table totality.
* `GemmTotal.lean` — ABI round trip, raw-invocation carrier, total classifier and its exact domain, reference totality, the mandatory witness family, observation coverage.
* `CostModel.lean` — SPEC §9.1 composition algebra and exact aggregate (all three §15 Cost names), §9.2 coordinate bound, §9.3 monotonicity and sublevel finiteness.
* `AtlasLaws.lean` — semantic closure leastness and derivability, the merge law, update/rebuild equality, seal-body uniqueness, cover-check scope blindness.
* `Release.lean` — the conditional release theorem, the release scope identities, and the universal competitor clauses.  It declares no §15 name.
* `Status.lean` — this ledger.

`BaselineCorrect.lean`, `CompilerCorrect.lean`, `SublevelComplete.lean`,
`AttentionComplete.lean`, `UpdateEqualsRebuild.lean`, `UniversalLowerBound.lean`,
`Attainment.lean`, `ArtifactCorrect.lean`, `ArtifactGlobal.lean` and
`LifecycleBound.lean` from the SPEC §5 tree are **absent by design**.  An empty
or partially-proved file under one of those names asserts a result it does not
have, which is worse than its absence.  `Release.lean` is present because its
contents are proved *and* its name states a conditional, which is what it is.
-/
import WasmGemmGnaf.Theorems.WasmModel
import WasmGemmGnaf.Theorems.GemmTotal
import WasmGemmGnaf.Theorems.CostModel
import WasmGemmGnaf.Theorems.AtlasLaws
import WasmGemmGnaf.Theorems.Release

set_option autoImplicit false

namespace WasmGemmGnaf.Theorems

/-- Marker declaration.  `Status.lean` is a ledger, not a source of results:
its content is the module doc comment above, and this is the only declaration
it contributes. -/
theorem spec15_ledger_is_documentation : True := trivial

end WasmGemmGnaf.Theorems
