# Prompt-conditioned evidence arms — bake-off instrument (#834)

- **Issue:** #834 — "research/#822-B: fit prompt-conditioned evidence arms against
  causal and equal-budget controls" (item B of S1 tracker #822, programme #820).
- **Parent tracker:** #822 (S1 — persistent prompt-conditioned predictive state).
- **Date:** 2026-08-20.
- **Status:** The **binding cheap instrument** and its five Verification items are
  frozen and green (reference-only). The S1 causal-relevance **verdict**
  (`SELECT` / `REVISE` / `NO PROMPT-CONDITIONING ARM ESTABLISHED`) is **UNAVAILABLE**:
  it is measured on the #833 canonical broad bundle with real teacher evaluations,
  a maintainer-gated long run whose identities are not present here (§5). This
  record is append-only.
- **Claim language:** normative per [`docs/formal_vocabulary.md`](formal_vocabulary.md).
  Every labeled statement carries one claim class (**Definition**, **Objective**,
  **Guarantee**, **Assumption**, **Empirical Criterion**) and, for **Guarantee** /
  **Empirical Criterion**, a status (**Structural**, **Witnessed**, **Empirical**,
  **Assumed**, **Unproven**).
- **Evidence file:** `crates/uor-r4-api/tests/prompt_arms_bakeoff_834.rs`
  (default `cargo test -p uor-r4-api --test prompt_arms_bakeoff_834`).

## 1. Problem and scope

#784 measured **0.0% full-depth context-code collisions** while 11/15 distinct rows
still favored the same continuation, and historical sampled canaries were only about
5–6/15 distinct — **diversity diagnostics, not causal-relevance results**. The open
question of S1 is *causal*: does prompt meaning move prediction beyond suffix/exact-
context memory? A single bidirectional-anchor implementation would confound four
distinct mechanisms — persistent-state discrimination, candidate availability,
conditional evidence, and decoding — so #834 fits and ablates **several** artifact-only
arms against strong causal and equal-resource controls before selecting at most one.

**Definition (execution scope).** This deliverable is a **reference-only /
off-serving-path** instrument in the sense of
[`docs/conformance_execution_scope_830.md`](conformance_execution_scope_830.md). It
provides (a) an offline, integer, deterministic **bake-off harness** over the arms and
controls #834 enumerates, built on the frozen #835 reference model and the frozen #832
evaluation vocabulary, and (b) the **binding cheap instrument** the run contract
requires to pass *before* any long run may launch. It is **not** deployed-serving
evidence, and it is **not** the S1 causal verdict — that measurement is the
maintainer-gated long run of §5. Tuning any deployed decision on the harness fixture is
a #834 non-goal ("Tune on the held-out intervention suite").

## 2. Arms and controls

**Definition (the five arms).** Each arm is a pure, integer, decode-independent reader
over a candidate set (`crates/uor-r4-api/tests/prompt_arms_bakeoff_834.rs`):

| Arm | What it reads | Isolates |
|---|---|---|
| `current-scoring` | suffix-local score (the deployed baseline) | the no-whole-prompt floor |
| `longer-local-context` | a longer suffix window | context length ≠ whole-prompt state |
| `persistent-state` | the whole-prompt fold (#835 `Ψ`) | state discrimination |
| `conditional-residuals` | persistent state minus the corpus marginal | evidence beyond the marginal |
| `candidate-support-expansion` | persistent-state scoring over an expanded candidate set | candidate availability |

**Definition (the controls / nulls).** Mapped to the frozen `ControlKind` vocabulary
(#832): `prompt-swap`, `suffix-only`, `shuffled-state`, and `trivial-prior`
(constant/root prior), plus the equal-budget controls the run contract names —
equal-candidate, equal-table-read, and a decoder held constant (a single fixed argmax
decoder across every arm and control). Each null is a real transform; none may pass
through zero variance or identical outputs.

## 3. The binding cheap instrument (Verification)

**Guarantee (the instrument operates and has teeth). Status: Structural** (machine-
checked; the named tests in `prompt_arms_bakeoff_834.rs`). The five Verification items
of #834 are each an executable, falsifiable test:

- **Interventions / attribution / shuffles operate** — `fixture_interventions_attribution_and_shuffles_operate`:
  a committed, document- and template-disjoint paired fixture across two domains with
  all six intervention families (paraphrase, subject, relation, negation, role,
  constraint); the shuffled-state control provably corrupts the recovered meaning; the
  attribution decomposition credits gains.
- **Power fixes n and the MDE before fitting** — `power_fixes_n_and_mde_before_fitting`:
  the sample count and minimum decision-relevant effect are compile-time constants set
  before any fitting; the design resolves the MDE.
- **Double-run / reordered-shard determinism** — `double_run_and_reordered_shard_determinism`:
  the CID-bound per-pair record reduces identically under repetition and under
  reversed/sharded order (a stable-id keyed, ordered reduction).
- **Planted negatives fail the primary gate** — `planted_negatives_fail_the_primary_gate`:
  a **prompt-insensitive** model and a **diversity-only** model (distinct output, no
  per-intervention relevance) are each flagged degenerate against the prompt-swap null
  and cannot reach the causal arm's separation — the #834 non-goal "promote distinctness
  without relevance", made a falsifier.
- **Independent report recomputation** — `independent_report_recomputation_from_cid`:
  the report CID recomputes from the canonical record bytes.

**Guarantee (candidate availability and score conditioning are separately identified).
Status: Structural** (`candidate_and_score_conditioning_separately_identified`). On
in-vocabulary pairs the persistent-state gain attributes to state discrimination and
never to candidate availability; on needs-expansion pairs the gain attributes to
candidate availability and never to state discrimination.

**Guarantee (every control is non-degenerate). Status: Structural**
(`every_control_is_non_degenerate`, `causal_arm_separates_from_all_nulls`). A genuinely
causal arm separates from every null, and each null changes at least one prediction —
so a zero reading means "no effect", not "broken harness" (the anti-vacuity property the
#835 planted-negative teeth already established for the state model, extended here to the
full arm/control matrix).

## 4. Reachability and power (projected, before fitting)

**Objective (reachability ceiling).** For each arm the harness computes, separately, the
fraction of pairs whose **candidate set** it can change (candidate-recall ceiling) and
whose **scorer** it can move (scoring ceiling); headline movement over the baseline
cannot exceed their sum (`reachability_ceiling_bounds_headline_movement`). This is the
run contract's reachability arithmetic instantiated on the instrument fixture.

**Empirical Criterion (power design). Status: Structural** (the design; not a corpus
result). The instrument's sample count and MDE are fixed before fitting; the powered `n`
of the real S1 study is set by its own protocol on #833 and is **UNAVAILABLE** here.

## 5. Run contract and entry-gate assessment

The #834 run contract, with the availability of each required input recorded (a missing
input is **UNAVAILABLE**, never a vacuous pass):

- **Metric / current value:** primary paired `causal-influence-delta` on EXCT-disabled,
  document-disjoint pairs; the historical surface diagnostics (11/15 newline argmax rows;
  ~5–6/15 distinct canaries) are diversity, not causal relevance, and are to be
  remeasured on #833.
- **Reachability ceiling:** the per-arm candidate and scoring ceilings above; on the
  real corpus these are computed from the #833 attribution, **UNAVAILABLE** here.
- **Pinned identities:** the #835 state-spec revision and the #832 suite vocabulary are
  **AVAILABLE and frozen** (`docs/prompt_state_spec_835.md`,
  `docs/capability_suites_832.md`, `s1-causal-prompt-pairs`). The canonical
  teacher / tokenizer / corpus / partitions / **bundle from #833** and the fixed decoder
  identities for the real evaluation are **UNAVAILABLE** in this offline instrument.
- **Nulls / falsifier:** implemented and non-degenerate (§3).
- **Binding cheap instrument:** **PASS** — the committed fixture's state variance,
  intervention direction, candidate change, and every null are non-degenerate, and the
  reachability arithmetic exceeds the effect floor (§3, §4).
- **Exit rule:** select only when the primary causal-relevance lower confidence bound
  clears the frozen floor on both domains while relevance, bits/accuracy, and resource
  budgets satisfy non-regression. Evaluated on #833, **UNAVAILABLE** here.
- **If positive:** open the selected layout/runtime lowering in sibling item **#836** and
  ledger all other arms as rejected/dormant with evidence.
- **If negative:** publish the falsifier, keep current serving semantics, and return S1
  to representation/compiler redesign — not code-space or sampling tuning.
- **Cost estimate (projected for the long run, not incurred here):** the real bake-off
  runs every arm × control on the #833 canonical bundle across ≥2 domains with a powered
  `n`; wall time, peak RSS/storage, teacher-evaluation count, and per-arm artifact bytes
  are set by that protocol and are posted to #834 before the run is launched.

**Guarantee (no long run launched here). Status: Structural.** This deliverable launches
**no** hours-scale run. It lands the binding cheap instrument the contract requires
first; the real multi-arm study on #833 is a **maintainer-gated** decision (post exact
run values, obtain go/no-go) with programme-wide blast radius, because only a predeclared
positive causal verdict may trigger deployed lowering (#836) and gate the S1→S7 lane.

## 6. Decision status

**Empirical Criterion (S1 causal verdict). Status: Unproven / UNAVAILABLE.** No
`SELECT` / `REVISE` / `NO PROMPT-CONDITIONING ARM ESTABLISHED` verdict is asserted by this
deliverable. The harness's `verdict_vocabulary_is_complete` and
`instrument_selects_causal_arm_and_rejects_planted_negatives` tests demonstrate the
decision function can produce all three verdicts and, on the controlled fixture, selects
a genuinely causal arm while rejecting planted non-causal negatives — this validates the
**instrument**, and is explicitly **not** the S1 result.

## 7. Repository conformance

**Definition (RF mapping).** This instrument extends the evidence of existing capability
IDs and introduces **no new built RF capability**; following the #835/#832 spec-leaf
precedent it adds **no** `model/ids.toml` row and triggers **no** `CONFORMANCE.md`
regeneration (generated conformance is never hand-edited):

- **RF-27 / RF-28** (semantic state / typed transitions; state–emission separation,
  reference/f32): the arms are prompt-conditioned instantiations, scored decode-
  independently.
- **RF-01** (unsupervised intervention and counterfactual behavioral probes): the paired
  interventions and the planted negatives.
- **RF-21 / RF-22** (R4G1 compilation quality gates / pathology filter): the equal-budget
  and shuffled controls.

**Definition (built-capability order for #836).** When #836 lowers the *selected* arm
into a deployed capability it follows the required order — `model/ids.toml` row → tagged
Gherkin → failing marker/behavior test → implementation → regenerated `CONFORMANCE.md` —
and either extends an existing suite or justifies a new capability then. Unselected arms
remain ledgered with activation or retirement gates.

## 8. Claim status and next action

**This deliverable freezes #834's binding cheap instrument; it does not establish that
any arm has prompt-causal relevance.** The next action is the **maintainer-gated long
run**: post the exact #833-scoped run values to #834, obtain go/no-go, then run the
multi-arm bake-off and record a `SELECT` / `REVISE` / `NO PROMPT-CONDITIONING ARM
ESTABLISHED` verdict with CID-bound records. A positive verdict opens #836; a negative
verdict narrows or retires the S1 conditioning claim under the #822 kill/redesign
criterion and is first-class completion evidence, not an implementation backlog.
