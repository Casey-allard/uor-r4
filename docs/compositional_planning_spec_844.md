# Compositional planning benchmarks and artifact-backed state/action semantics — specification (#844)

- **Issue:** #844 — "reasoning/#826-A: freeze compositional benchmarks and artifact-backed
  state/action semantics" (item A of S4 tracker #826, programme #820).
- **Parent tracker:** #826 (S4 — semantic planning and geometry qualification).
- **Date:** 2026-08-22.
- **Status:** Frozen experimentable **contract** (reference specification). This document
  freezes (a) the compositional-planning benchmark constitution the S4 stage is judged on and
  (b) the versioned typed artifact/reference model for states, actions, effects, goals,
  constraints, evidence, confidence, transitions, and plan witnesses. It does **not** establish
  any planning or reasoning capability — that is the measured question of #843 (induce/execute
  bounded transitions), #845 (geometry qualification), and #846 (certification and claim
  bounding). Records are append-only.
- **Claim language:** normative per [`docs/formal_vocabulary.md`](formal_vocabulary.md) (v0.1.14).
  Every labeled statement carries exactly one claim class (**Definition**, **Objective**,
  **Guarantee**, **Assumption**, **Empirical Criterion**); every **Guarantee** and
  **Empirical Criterion** carries a status (**Structural**, **Witnessed**, **Empirical**,
  **Assumed**, **Unproven**).
- **Execution scope:** **reference-only / off-serving-path** for the typed reference model and
  its semantics, and **certifier-instrument / off-serving-path** for the benchmark harness, in
  the sense of [`docs/conformance_execution_scope_830.md`](conformance_execution_scope_830.md).
  Nothing here is deployed-serving evidence and nothing here strengthens or weakens the
  guarantees of the deployed runtime. The deployed integer/table planner is the scope of #843.
- **Planned machine-checked evidence (built after sign-off):** the reference model and its
  determinism, totality, capacity, saturation, conflict, tie-break, decline, witness-replay,
  metamorphic, and planted-negative controls will run in
  `crates/uor-r4-api/tests/compositional_planning_spec_844.rs` (default `cargo test`); the
  frozen benchmark constitution expands
  `crates/uor-r4-api/capability_suites/compositional_reasoning.json` (CID-bound, #832 framework).

---

## 0. Entry boundary — the established, not promoted-generative, substrate

Per the **S4 entry reconciliation (AMEND + PROCEED, 2026-08-22)** recorded on #826 and mirrored
in `docs/r4_intelligence_completion_plan.md` (main `ddd82ff2`), S4 builds on the *established*
upstream substrate, not a promoted-generative one. This specification is written to that boundary
and may not silently assume capabilities the upstream stages did not establish:

- **S1 PROMOTE (RF-31).** Prompt causality is established at +28.45‰ [25.57, 31.32]; the
  persistent prompt state `Ψ` (#835) is available as a reference/deployed conditioning surface.
- **S2 REVISE — honest abstention, no calibrator (#839 phase-2 trigger-gated).** Selective
  prediction is *honest abstention*, **not** a calibrated confidence. **Assumption.** This
  specification's `confidence`/`uncertainty` objects are therefore ordinal, decline-oriented
  signals; no statement here reads a calibrated probability, and no benchmark metric rewards a
  numeric confidence value. Selective integration is via the RF-30 typed decline surface.
- **S3 LIMIT — no free-running generation at this scope (#824).** The deployed artifact is a
  teacher-forced retrieval/continuation system. **Assumption.** Every benchmark in this
  constitution is scored **teacher-forced** (`ScoringMode::TeacherForced`); no metric here reads
  a free-running multi-token rollout as evidence of planning, and planning results are **not**
  read as free-running coherence (the #824 boundary stands). Any future free-running re-entry
  re-measures against the unchanged #841 §6 bar and the frozen #838 selective gates — out of
  scope for #844.

**Definition (what #844 establishes).** A *falsifiable target* (the benchmark constitution) and a
*byte-level meaning* (the typed reference model), against which #843/#845/#846 measure. #844
establishes **no** reasoning performance and makes **no** deployed-planning claim.

---

## 1. Problem and scope

Multi-hop continuation, graph retrieval, memorized paths, and smooth f64 routing can all *look*
like reasoning on non-interventional benchmarks. Undefined state/action/goal semantics also make
witnesses, capacity bounds, and proofs impossible to pin. Two failures must be foreclosed before a
planner or geometry is chosen:

1. **Benchmark permeability.** Without document-disjoint held-out partitions, entity/topology/
   composition splits, intervention controls, and non-degenerate baselines, a later "improvement"
   can be leakage, ExactContext dominance, a decoder effect, a vacuous control, or benchmark drift
   (the #832 failure taxonomy).
2. **Semantic vagueness.** Without a reference transition, precondition/effect semantics, goal and
   constraint predicates, evidence/provenance, capacity/decline rules, and witness fields, an
   experiment cannot distinguish planning from suffix leakage, an oracle-like compiler, or a
   reference-only f64 path (the #845 route-attention lesson: 119/120 heads vacuous, fit 0.396).

**In scope (frozen here):** the five task families and their deterministic verifiers; the split
axes and metamorphic controls; the baseline/null set and the promotion statistic; the frozen
primary/secondary metrics, horizon progression, effect floor, sample sizes, and multiple-comparison
handling; the versioned typed artifact/reference model and its total reference semantics; the
deployed-form projection constraints; the plan-witness schema; and the leakage/tamper discipline.

**Out of scope (deferred, by sibling):** inducing or executing transitions and lowering a planner
(#843); any geometry mapping or W(3,3) qualification (#845, which stays `on-hold` until #843
produces a non-degenerate baseline); the untouched-partition final verdict (#846); free-running
generation re-entry (#824 boundary).

**Non-goals (honored from #844):** treating nearest-neighbor retrieval, paraphrase stability, or
path smoothness as reasoning; designing around W(3,3) before non-geometric state/action semantics
exist; and using benchmark examples that share entities/topologies/compositions with fitting data.

---

## 2. The compositional-planning benchmark constitution

This section freezes the content of the CID-bound suite `s4-compositional-reasoning`
(`Stage::S4`, `Workload::CompositionalReasoning`, `ScoringMode::TeacherForced`) in the #832
`capability_suite` framework. The existing placeholder manifest is *expanded*, not replaced; the
manifest schema (`CAPABILITY_SUITE_SCHEMA = 1`) is preserved and any new field is additive.

### 2.1 Task families (five)

Each family ships a **deterministic generator** (content-addressed, seedless in the RNG sense —
sample identity is a hash of its typed parameters, per plan §4.1) and a **deterministic verifier**
that recomputes the objectively checkable outcome and every replayable intermediate state. All
five are scored teacher-forced (§0, S3 boundary).

| # | Family | Task | Objectively checkable outcome | Replayable intermediate state |
|---|---|---|---|---|
| F1 | **Graph navigation** | Reach a goal node/region under typed edges and forbidden regions | terminal node ∈ goal set; no step enters a forbidden region | per-step frontier + chosen edge |
| F2 | **Symbolic transformation** | Apply a bounded operator sequence to reach a target term | terminal term == target (canonical form) | per-step term after each operator |
| F3 | **Constraint satisfaction** | Assign/route to satisfy all typed constraints | all constraints satisfied; none violated | partial assignment after each action |
| F4 | **Multi-hop evidence composition** | Compose provenance-linked evidence to a supported conclusion | conclusion holds AND every hop has cited support | evidence set + support chain per hop |
| F5 | **Counterfactual intervention** | Re-plan after a declared edge/effect change | plan valid under the intervened dynamics, not the original | pre/post-intervention transition trace |

**Definition (deterministic verifier).** For every task instance `x`, the verifier `V(x, plan)`
recomputes the terminal outcome and each intermediate state from the frozen task dynamics and
returns `Valid`, `Invalid(step_i, reason)`, or `Decline(reason)` — it never consults a hidden gold
path, the teacher, or a network. **Guarantee (verifier totality). Status: Structural** (built:
`compositional_planning_spec_844.rs::verifier_total_over_valid_inputs`).

**Definition (replayable gold).** Each instance carries a replayable gold plan and its full
intermediate-state sequence; the verifier accepts the gold and rejects any plan with an invalid
intermediate transition **even if its terminal state matches** (the #846 rule: a right answer via
an invalid step is not a valid plan).

### 2.2 Split axes (held-out generalization)

**Definition (split axes).** The suite partitions by **entity**, **surface vocabulary**,
**topology**, **operator composition**, **prompt template**, and **reasoning horizon**. Fitting
data and evaluation data never share a cell on any axis; the final `held-out-composition` and
`held-out-topology` cells are reserved untouched for #846. The manifest `split` block gains
`by_vocabulary`, `by_operator_composition`, and `by_horizon` alongside the existing `by_entity`,
`by_topology`, `by_template`, `leakage_check`, `tamper_check`.

**Guarantee (no split leakage). Status: Structural** (built:
`compositional_planning_spec_844.rs::splits_are_disjoint_on_every_axis` +
`::no_entity_topology_or_composition_shared_with_fitting`).

### 2.3 Metamorphic controls (intervention-resistance)

Applied to a solved instance, each control transforms the instance and its gold in lockstep and
asserts the required behavior, foreclosing a specific shortcut:

- **entity relabeling** — a consistent bijective renaming must not change validity (a label-memorizer breaks);
- **irrelevant / distractor context** — added unused entities/edges must not change the valid plan;
- **counterfactual edge/effect change** — a declared dynamics change must change the valid plan (F5);
- **unseen composition** — an operator/topology composition absent from fitting must be generable and verifiable.

**Guarantee (metamorphic consistency). Status: Structural** (built:
`compositional_planning_spec_844.rs::metamorphic_relabel_distractor_counterfactual_unseen`).

### 2.4 Baselines and nulls (the promotion falsifiers)

**Definition (baseline set).** Six baselines, each an **Empirical Criterion** null a real planner
must beat, extend the framework's `ControlKind` (additively): **retrieval-only**,
**direct-continuation**, **memorized-trajectory**, **shuffled-state/action**, **shortest-path
oracle** (an upper reference, *not* a beat-target), and **trivial-prior**. `shuffled-state` and
`trivial-prior` already exist; `retrieval-only`, `direct-continuation`, `memorized-trajectory`, and
`shortest-path-oracle` are new `ControlKind` variants.

**Empirical Criterion (promotion statistic). Status: Empirical** (measured by #846, not #844). The
promoted mechanism's one-sided 95% **lower** confidence bound of (candidate − strongest
*non-oracle* baseline) exceeds the frozen effect floor **δ_min** on **every** required
generalization axis and horizon, under equal artifact bytes, candidates, expansions, and
operations. **Kill:** a gain that vanishes under relabeling/unseen-composition is memorization; a
geometry that cannot beat Hamming/binary/VSA/spectral under equal budget stays offline (#845).

**Guarantee (baseline non-degeneracy). Status: Structural** (built: `is_degenerate_control`
applied per suite — a control that fails to separate from the primary invalidates the reading).

### 2.5 Frozen metrics, horizons, effect floor, and sampling

**Definition (primary metric).** `held-out valid-plan rate` = fraction of held-out instances whose
emitted plan the verifier accepts as `Valid` (terminal outcome **and** every intermediate
transition). This replaces the placeholder `held-out-composition-accuracy` (kept as an alias in the
report schema for backward reads).

**Definition (secondary metrics).** valid-intermediate-transition rate; constraint-satisfaction
rate; evidence-support rate (F4); first-failure step (median, per horizon); honest-decline rate
(no-plan / low-confidence, per §0 S2 boundary); and the resource envelope (artifact bytes,
candidate expansions, table reads, integer operations).

**Definition (horizon progression, frozen).** Reported at horizons **H ∈ {1, 2, 4, 8}** with a
fixed maximum horizon **H_max = 16** and a fixed maximum frontier width **W_max = 64** (the #843
planner capacities; instances exceeding them are `Decline(capacity)`, never truncated silently).

**Definition (effect floor, frozen — maintainer sign-off value).** **δ_min = 0.05** (5 percentage
points of valid-plan rate) as the practical-significance floor for the promotion statistic, in
addition to the lower-bound-> 0 requirement.

**Definition (sampling and power, frozen — maintainer sign-off value).** **n = 512** independent
instances per held-out cell per horizon; the built power-analysis reproduces ≥ 0.80 power to detect
δ_min at α = 0.05 (paired, two-sided) and records the achieved power per cell. **Multiple-comparison
handling:** Holm–Bonferroni across the (axis × horizon × family) grid; the promotion gate uses the
adjusted bounds. **Empirical Criterion. Status: Empirical** (the power reproduction is a built test;
the achieved powers are measured).

### 2.6 Leakage, tamper, and untouched partitions

**Guarantee (no hidden future-answer leakage). Status: Structural** (built:
`compositional_planning_spec_844.rs::no_future_answer_reachable_at_inference`). The task encoding
exposes no field from which the gold terminal or gold path is recoverable at inference; a tamper
test corrupts the manifest CID and asserts the loader fails closed. The final held-out composition
and topology cells are sealed (CID-bound, access-logged) for #846 and are never opened by #843/#844.

---

## 3. The typed artifact / reference model (byte-level semantics)

This section freezes the versioned typed model that #843 induces and lowers and that #846
certifies. It refines the semantic-state binding `S`, dynamics `T : S × A → S`, goal `G ⊆ S`, and
constraint `F ⊆ S` of `docs/formal_vocabulary.md` §3/§5. It is a **reference-only / off-serving-path**
model (RF-27/RF-28 sense): owned, offline, f32 permitted for the reference arm; the deployed-form
projection (§3.4) forbids owned strings, unbounded collections, and f32.

### 3.1 Typed objects and stable artifact IDs

**Definition (typed objects).** Ten object kinds, each with a stable artifact ID (a content-derived
`u64`/`u128` per the R4G1 identity discipline) and reference semantics:

| Object | Symbol | Reference meaning | Deployed-form projection |
|---|---|---|---|
| **State** | `s ∈ S` | typed valuation over a fixed schema of typed slots | packed fixed-capacity slot vector (bounded key/value ints) |
| **Action** | `a ∈ A` | a typed operator with an ID and typed parameters | operator ID + fixed-width parameter block |
| **Precondition** | `pre(a) ⊆ S` | predicate over states that must hold to apply `a` | packed mask/popcount predicate |
| **Effect** | `eff(a)` | typed delta applied to a state | packed additive/mask delta |
| **Goal** | `G ⊆ S` | desired future-state subset (a predicate) | packed goal predicate |
| **Constraint** | `F ⊆ S` | forbidden subset (a predicate) | packed forbidden predicate |
| **Evidence/provenance** | `ev` | typed support with a provenance ID (F4) | provenance ID + support code |
| **Confidence/uncertainty** | `u` | *ordinal*, decline-oriented signal (§0 S2 boundary) | small fixed-range integer band |
| **Transition** | `T : S × A → S` | total dynamics over valid `(s, a)` | packed transition/edge section |
| **Plan witness** | `π-witness` | replayable record of a planning episode (§3.5) | validated optional witness section |

### 3.2 Reference transition semantics (total, deterministic)

**Definition (transition `T`).** `T(s, a) = s'` when `pre(a)` holds in `s`, applying `eff(a)`;
otherwise `T` returns a typed non-application outcome. `T` is **total over valid inputs** and
deterministic. **Guarantee. Status: Structural** (built: `::transition_total_and_deterministic`).

**Definition (deterministic decision rules, frozen).**

- **Goal satisfaction.** `s ⊨ G` iff the goal predicate holds; a plan is `Valid` iff `T^π(s₀) ⊨ G`
  and no intermediate state enters `F` (the §5 formal-vocabulary plan definition).
- **Transition validity.** an action is applicable iff `pre` holds; an inapplicable action in a
  submitted plan yields `Invalid(step_i, precondition)`.
- **Conflict / unknown.** conflicting effects or an unknown slot resolve by a frozen typed rule
  (declared conflict → `Invalid(conflict)`; unknown → `Decline(unknown)`), never by silent default.
- **Tie-breaking.** candidate ties break by the canonical deterministic order already normative in
  `docs/scoring_semantics.md` (`deterministic_topk_proof`); no clock/RNG/hash-iteration order.
- **Saturation / capacity / overflow.** exceeding `H_max`, `W_max`, or a fixed slot/action capacity
  is `Decline(capacity)`; state/score arithmetic saturates (no wrap into a valid-looking value).
- **Typed decline.** every non-answer is one of `Decline(no_plan | capacity | unknown | low_confidence)`
  — an honest abstention (§0 S2 boundary), never a fabricated plan.

**Guarantee (totality of the reference semantics over valid inputs; deterministic decline at
capacity/unknown boundaries). Status: Structural** (built: property tests
`::goal_transition_conflict_tie_saturation_decline_are_total_and_deterministic`).

### 3.3 Interaction with prompt/session state and selective prediction

**Definition.** Planning may read the persistent prompt state `Ψ` (#835, RF-31) as a conditioning
surface and emits its abstentions through the RF-30 typed decline surface. **Guarantee (no
future-answer access). Status: Structural** (built: `::planning_reads_no_future_answer`).
**Assumption.** No calibrated-confidence claim is made: `u` is ordinal and used only for
decline ordering (§0 S2 boundary).

### 3.4 Deployed-form projection (the contract #843 must satisfy)

**Definition (deployed-form constraints).** The lowered planner packs states/actions/
transitions/goals/indices into **validated optional/versioned sections** with backward behavior,
executing **only** the P-4 permitted operation classes (bitwise, shift/rotate, popcount, saturating
integer add/sub, comparison, fixed-offset table reads) in the #831 normative `R4Engine` path with
caller-owned scratch/state. It excludes **owned strings, unbounded collections, and f32** from the
deployed form. **Guarantee (allocation-freedom, P-4 conformance, byte-determinism of the deployed
planner). Status: Unproven here** — a requirement #843's allocation census, P-4 scan, and
deterministic-rebuild establish, not this reference specification.

### 3.5 Plan-witness schema (replayable, independently verifiable)

**Definition (plan witness).** A versioned optional section recording: initial state, goal and
constraints, considered actions, per-step transition evidence, chosen path, per-step score and
ties, terminal verification result, and decline reason (if any). **Guarantee (independent replay).
Status: Structural** (built: `::witness_replays_and_revalidates_without_model_outputs` — an
independent verifier replays every transition and the terminal goal from the witness alone). This is
the witness #843 emits and #846 replays.

---

## 4. Term discipline — public reasoning phrases mapped to measured tasks

Per `docs/formal_vocabulary.md` §5, "reasoning" is never used bare. Each public phrase this
programme may use is mapped to an exact measured task/horizon and a claim status:

| Public phrase | Precise mechanism | Measured by | Status until measured |
|---|---|---|---|
| "compositional planning" | bounded planning (trajectory eval over `T`) on F1–F5, held-out composition | #843 fit / #846 verdict | **Unproven** |
| "graph reasoning" | graph navigation (F1) at horizon H | #846 per-horizon table | **Unproven** |
| "constraint reasoning" | constraint satisfaction (F3) | #846 | **Unproven** |
| "evidence reasoning" | multi-hop evidence composition (F4) with cited support | #846 | **Unproven** |
| "counterfactual reasoning" | re-planning under intervened dynamics (F5) | #846 | **Unproven** |

No phrase in this table may appear unqualified in normative text; each is limited to the exact typed
task and horizon it names.

---

## 5. Conformance mapping and built-capability order

**Conformance mapping (existing RF ids, evidence language extended):** RF-01 (behavioral probes),
RF-08 (bounded future-state planning; reference-only), RF-12 (lowering reference regions to
Boolean/mask/popcount/fixed-point), RF-13 (packed zero-allocation kernels — the #843 deployed
target), RF-27 (semantic state space + typed transition dynamics; reference/f32), RF-28 (separate
state transitions from language emission; reference/f32). Existing reference IDs are **not**
deployed-planning evidence.

**Proposed new instrument id (maintainer sign-off):** register **RF-32**
`compositional_planning_benchmarks` — a **certifier-instrument / off-serving-path**,
fixture-gated **empirical** instrument (mirroring RF-29 `teacher_parity_benchmarks`: an absent
fixture is `UNAVAILABLE`, never `PASS`). Alternative: fold the benchmark evidence under RF-29's
language rather than adding a row. (Decision D5 below.)

**Built-capability order (executed in the build pass, after sign-off):**
`model/ids.toml` (RF-32 + evidence-language edits) → tagged Gherkin
(`features/suites/compositional_planning_benchmarks.feature`) → failing marker/behavior test
(`repo-conformance` `registered.rs` `check("RF-32", "compositional_planning_benchmarks")` +
executable steps in root `tests/bdd.rs`) → implementation (reference model, generators, verifiers,
expanded suite JSON, `ControlKind` extensions) → `xtask check-model --write` to regenerate
`CONFORMANCE.md` (never hand-edited).

---

## 6. Acceptance-criteria mapping (#844)

| #844 acceptance criterion | Frozen artifact that will establish it |
|---|---|
| Every task has a deterministic verifier and replayable gold/intermediate state | §2.1 verifier totality + replayable gold |
| Split axes prevent entity/topology/surface/composition memorization | §2.2 disjoint splits + §2.3 metamorphic controls |
| Reference transition/witness semantics total over valid inputs; deterministic decline at boundaries | §3.2 totality + typed decline |
| Each public reasoning phrase mapped to an exact task/horizon and formal-vocabulary status | §4 |
| A retrieval/memorized-path and a shuffled-state planted model fail the primary gate | §2.4 baselines + built planted-negative controls |

## 7. Verification plan (built after sign-off)

Generator determinism, split-leakage, relabeling/counterfactual/unseen-composition metamorphic, and
tamper tests; reference transition/precondition/effect/goal/witness unit and property tests;
capacity/overflow/conflict/unknown/tie/saturation negative fixtures; baseline non-degeneracy and
power-analysis reproduction; schema examples round-trip and map to the conformance/proof matrix.
Full repository gate ladder (fmt, clippy `-D warnings`, `cargo test --workspace`, no_std ladder,
wasm lib build if core/router is touched, `check_claim_wording.py`, register-conformance R1–R6,
deterministic rebuild, κ) runs before push.

## 8. Compatibility and migration

Only optional/versioned experimental schemas are introduced until a planner is selected
(#843/#846). Historical artifacts remain valid and cannot be reported as planning-capable without
the new sections and report identities. The capability-suite manifest bump is additive; the report
schema keeps the placeholder metric name as an alias. Unsupported artifact eras fail with typed
errors.

## 9. What this issue establishes

**Claim status.** Completion establishes a falsifiable target and byte-level meaning, **not**
reasoning performance. #843 consumes §3 (typed model) and §2 (frozen benchmark) to induce/execute a
non-geometric planner; #845 consumes §3.1's state/action/goal/evidence surface for any geometry
mapping (and stays on-hold until #843 records its trigger); #846 consumes the sealed untouched
partitions (§2.6) and the witness schema (§3.5) for the final `PROMOTE TYPED PLANNING` /
`LIMITED CAPABILITY` / `REASONING NOT ESTABLISHED` verdict.

---

## 10. Open decisions for maintainer sign-off

These are the maintainer-grade choices this document freezes. They are called out explicitly so the
freeze is deliberate (mirroring how the S1 spec #835 was frozen as a reviewed step):

- **D1 — Task families (§2.1).** Freeze exactly F1–F5 as listed? (Alternative: drop F5 counterfactual
  to a #846-only stress axis, or add a sixth family.)
- **D2 — Horizons/capacities (§2.5).** H ∈ {1,2,4,8}, H_max = 16, W_max = 64?
- **D3 — Effect floor (§2.5).** δ_min = 0.05 valid-plan-rate as the practical-significance floor?
- **D4 — Sampling/power (§2.5).** n = 512 per held-out cell per horizon; Holm–Bonferroni across the
  (axis × horizon × family) grid?
- **D5 — Conformance id (§5).** Register a new **RF-32** `compositional_planning_benchmarks`
  instrument, or fold the evidence under existing RF-29/RF-27/RF-28 without a new row?
- **D6 — Primary metric name (§2.5).** Rename the placeholder `held-out-composition-accuracy` to
  `held-out valid-plan rate` (keeping the old name as a report alias)?

On sign-off, the build pass implements §5's capability order and lands the code/config/tests +
regenerated `CONFORMANCE.md`, verified through the full gate ladder and merged through the queue.

---

## 11. Amendment A1 — generator non-vacuity and content-address repair (appended 2026-08-22, from #843)

**Status: appended evidence, not a rewrite.** Records in `docs/` are append-only (AGENTS.md). §§0–10
above stand as the frozen contract of #844 and are unchanged. This section records a measurement
made *after* that freeze, by #843's binding cheap instrument, which falsifies an assumption §2 relied
on, and freezes the bounded repair. **Issue #844 remains closed** — its deliverable (a falsifiable
target and a byte-level meaning) shipped; this is new evidence appended to it, in the sense of
§9 and of the "claim-changing results append evidence" rule.

### 11.1 What the instrument measured

Probe: deterministic, teacher-free enumeration over
`uor_r4_graph_compiler::compositional_planning` at main `3a4801c4`; ~15 s wall-clock; no fixture, no
network. Four independent structural results, each reproducible from the module alone:

**Empirical Criterion (horizon-cell vacuity). Status: Empirical.** Solvable fraction per frozen cell,
n = 512 seeds, over the frozen horizon progression H ∈ {1, 2, 4, 8} × five families = 20 cells:
**13 of 20 cells are 0/512 solvable.** All ten H ∈ {1, 2} cells are identically zero. At H = 4 only
`symbolic-transformation` (171/512 = 0.334) and `multi-hop-evidence` (342/512 = 0.668) are non-zero.
At H = 8 all five families are 512/512. The module's own documented invariant on `generate` —
"instances are constructed to be solvable within `horizon`" — does not hold for H ≤ 2, because the
shortest gold plans are 3–8 steps while `bfs_plan` receives `max_steps = horizon`.

**Empirical Criterion (strongest-null saturation). Status: Empirical.** A *structure-keyed*
memorized-trajectory null — fit on seeds 0..256, evaluated on the disjoint seeds 256..512, replaying
the stored plan whenever `(family, initial state, goal, action set, forbidden set)` matches, with the
generator seed excluded from the key — scores **valid-plan rate 1.0000 in every non-vacuous cell**
(H = 8: 256/256 in all five families; H = 4: 85/85 symbolic, 171/171 multi-hop). The strongest
non-oracle baseline of §2.4 therefore already sits at the ceiling, so the §2.4 promotion statistic
(candidate − strongest non-oracle baseline) is **≤ 0 < δ_min = 0.05 by construction**, for every
candidate mechanism, geometric or not.

**Empirical Criterion (split-axis cardinality). Status: Empirical.** Over 4096 seeds at H = 8, the
number of distinct cells per §2.2 axis is: `by_entity` = 1, `by_vocabulary` = 1, `by_template` = 1 for
all five families; `by_topology` = {1, 1, 2, 1, 4}; `by_operator_composition` = {3, 3, 2, 3, 7}.
Five of the six axes have a single cell, so §2.2's Guarantee (no split leakage) currently holds
**vacuously** — it partitions a one-element set — and held-out entity, vocabulary, and template
generalization are not measurable.

**Definition (content-address defect).** `TaskInstance::id()` mixes the raw generation seed into its
canonical byte string, so distinct ids number 4096/4096 while only 2–7 structurally distinct problems
exist per family. An id-keyed memorized-trajectory null consequently scores 0.000 and reads as a
healthy, non-degenerate control while the structure-keyed null above is saturated. This is the #845
route-attention failure mode (119 of 120 heads vacuous) recurring: a control that cannot fire is not
evidence of difficulty. A content-derived identity must not carry the generator seed.

### 11.2 What is repaired, and what stays frozen

**Frozen and unchanged (maintainer sign-off, 2026-08-22 — the D2/D3/D4/D6 values stand):**
δ_min = 0.05, n = 512 instances per held-out cell per horizon, H ∈ {1, 2, 4, 8}, H_max = 16,
W_max = 64, Holm–Bonferroni across the (axis × horizon × family) grid, the five families F1–F5, the
six split axes, the six baselines, the primary metric `held-out valid-plan rate`, RF-32, and every
guarantee statement of §§2–3. No number this document froze changes value.

**Repaired (A1 scope, frozen here):**

- **A1-a — low-horizon non-vacuity.** Instance *difficulty scales with the requested horizon*: a
  family generator targets a gold plan of length ≤ H, so an H = 1 instance is a genuinely
  one-step task rather than an unreachable eight-step task truncated to a decline. Non-vacuity is
  restored by making instances easier at low H, **not** by moving the frozen horizon grid.
  **Empirical Criterion (per-cell non-vacuity). Status: Empirical** — every one of the 20 frozen
  cells reports a solvable fraction ≥ 0.5, asserted by a built instrument.
- **A1-b — real split-axis cardinality.** Each family gains genuine variation along entity naming,
  surface vocabulary, topology (obstacle/edge configuration), prompt template, and operator
  composition, so every §2.2 axis has ≥ 8 distinct cells and disjoint fitting/held-out partitions
  are constructible rather than vacuous. **Empirical Criterion (axis cardinality ≥ 8 per axis per
  family). Status: Empirical** — asserted by a built instrument.
- **A1-c — structural content address.** `TaskInstance::id()` is derived from the typed problem
  content only, with the generator seed excluded, so structurally identical instances share an id.
  **Guarantee (content-derived identity; no seed, clock, RNG, or hash-iteration order).
  Status: Structural** — asserted by a built test.
- **A1-d — strongest-null headroom.** The structure-keyed memorized-trajectory null and the
  retrieval-only null must each leave headroom above δ_min in every non-vacuous cell, i.e. the
  strongest non-oracle null's valid-plan rate is ≤ 1 − δ_min. A benchmark on which the strongest
  null saturates cannot separate a candidate and is not a valid instrument.
  **Empirical Criterion (null headroom ≥ δ_min). Status: Empirical** — asserted by a built
  instrument, and binding: it gates #843's packed/full fitting.

### 11.3 Why the guarantees of §§2–3 passed while the instrument could not separate

**Assumption (recorded, so it is not repeated).** §2.2's and §2.4's Structural guarantees are about
*disjointness*, *totality*, and *determinism*. None of them is about **non-vacuity of a cell** or
**headroom above the strongest null**, and a benchmark can satisfy all of the former while measuring
nothing. Any future benchmark-freeze item in this programme adds two instruments before the word
"frozen" is used: a per-cell non-vacuity count, and a strongest-null saturation check.

### 11.4 Where the repair lands

Amendment A1 is implemented in #843's increment 2 (`crates/uor-r4-graph-compiler/src/compositional_planning.rs`
plus the RF-32 instrument surface), verified by the built non-vacuity, axis-cardinality, identity, and
null-headroom instruments named above, and recorded in
[`docs/bounded_semantic_transitions_spec_843.md`](bounded_semantic_transitions_spec_843.md) §2.
It establishes **no** planning or reasoning capability; it restores the ability of the #844 instrument
to separate one, which #843 then measures and #846 certifies.

### 11.5 Amendment A1 outcome (measured 2026-08-22, #843 increment 2)

All four A1 gate instruments pass. Measured by
`crates/uor-r4-graph-compiler/tests/amendment_a1_843.rs` at the frozen n = 512 per cell per
horizon; deterministic, teacher-free, fixture-free, about 40 s.

**A1-a — per-cell non-vacuity. Status: Empirical, PASS.** All 20 frozen cells clear the 0.5
threshold. H ∈ {2, 4, 8}: 512/512 = 1.0000 in all fifteen cells. H = 1: 384/512 = 0.7500 in all
five, by design — see §11.6.

**A1-b — axis cardinality. Status: Empirical, PASS.** Per family, over the frozen horizons:
`by_entity` = 8, `by_vocabulary` = 8, `by_topology` = 8, `by_template` = 8, and
`by_operator_composition` = 32 (64 for symbolic-transformation). Every axis clears the threshold of
8, so a disjoint fitting/held-out partition is constructible rather than vacuous.

**A1-c — content-derived identity. Status: Structural, PASS.** `TaskInstance::id()` no longer
carries the generation seed; structurally identical instances at different seeds share an id, in
every family at every frozen horizon.

**A1-d — strongest-null headroom. Status: Empirical, PASS.** Splitting on the *semantic* topology
axis (fitting = topology cells 0–3, held-out = 4–7, so a held-out cell never shares an operator
effect set or a forbidden configuration with fitting data), the strongest non-oracle null scores:

| H | graph-nav | symbolic | constraint-sat | multi-hop | counterfactual |
|---|---|---|---|---|---|
| 1 | 0.5352 | **0.9355** | 0.5625 | 0.5352 | 0.5898 |
| 2 | 0.5859 | 0.6777 | 0.4688 | 0.5859 | 0.3652 |
| 4 | 0.4023 | 0.5918 | 0.3906 | 0.4023 | 0.3535 |
| 8 | 0.2773 | 0.4922 | 0.3516 | 0.2773 | 0.3398 |

All are below the 1 − δ_min = 0.95 ceiling, so the promotion statistic is no longer at or below zero
by construction. **Tightest cell, flagged: symbolic-transformation at H = 1, headroom 0.0645 —
1.29 × δ_min.** The instrument prints a `TIGHT` marker for any cell whose headroom falls under
2 × δ_min; that cell is the one to watch when the arms are measured, because a candidate has barely
more than the effect floor of room there. The controls are not strawmen: they are fitted on the
declines as well as on the plans, they transfer plans by *effect* rather than by name or slot (so
they already see through entity, vocabulary, and template renaming), and the retrieval control
indexes by goal displacement.

For contrast, the pre-amendment numbers were 1.0000 in every non-vacuous cell (§11.1).

### 11.6 The horizon-1 cell is gated on honest decline (maintainer sign-off, 2026-08-22)

**Definition (why valid-plan rate cannot separate at H = 1).** A one-step task's correct answer is a
deterministic function of the initial state, the goal, and the operator set — all observable at
inference — and the fitting split must cover the whole operator pool or an inducer has nothing to
learn from. A baseline that indexes goal displacement to operator effect is therefore *optimal at
horizon 1 by construction*, and no generator repair changes that: measured, retrieval scored exactly
1.0000 at H = 1 in four of the five families once the other three instruments had passed. This is a
property of horizon-1 tasks, not a defect of the generator.

**Definition (the frozen resolution).** The H = 1 cell is gated on the **correct-outcome rate** —
a valid plan on a solvable instance, or a correct `Decline(no_plan)` on one with no plan inside the
horizon — rather than on valid-plan rate. A quarter of horizon-1 instances place the goal one step
beyond the horizon, so they are genuinely unsolvable within it. Deciding which is which requires
evaluating reachability, which is planning; a baseline that always answers cannot. On every cell
whose instances are all solvable — every H ≥ 2 cell — the correct-outcome rate *is* the frozen
valid-plan rate, so this adds a per-cell reading at H = 1 and changes nothing elsewhere.

**What this does and does not change.** The honest-decline rate was already a frozen §2.5 *secondary*
metric; this makes it the gating statistic in one cell. No frozen *value* moves: δ_min = 0.05,
n = 512, H ∈ {1, 2, 4, 8}, H_max = 16, and W_max = 64 all stand, and the primary metric elsewhere is
unchanged. The alternative considered and rejected was to report H = 1 without gating on it, which
would have left five of the twenty cells descriptive rather than evidential.

## 12. Amendment A2 — geometry-comparison cells and the budget-axis statistic (appended 2026-08-22, from #845)

**Provenance.** Maintainer decision on #845, 2026-08-22 (the decision comment on #845 is the source
of record; `docs/w33_geometry_qualification_spec_845.md` is the frozen design contract that uses
this amendment). Same discipline as Amendment A1: this section is **appended**, the record above it
is not rewritten, and **no frozen value moves** — δ_min = 0.05, n = 512 per cell per horizon,
H ∈ {1, 2, 4, 8}, H_max = 16, W_max = 64, the §2.5 metrics, and the §11.6 horizon-1 reading all
stand exactly as frozen.

**The finding this amendment answers (a sixth structural ceiling).** #843 closed LIMITED with the
lowered non-geometric baseline `bounded-breadth-first` at correct-outcome rate **1.0000 in all 20
joint-split cells** (`docs/bounded_semantic_transitions_843.md` §3), the 12 separating cells
included. A rate cannot exceed 1, so the reachable effect of *any* geometry arm over that baseline
on the frozen primary statistic is exactly **0 < δ_min** on every cell of the grid. As released,
#845's primary run could never launch under the repository run contract. This is the same shape as
the greedy-solvability ceiling recorded in the #843 verdict: a comparison is only informative where
a comparison is possible. The repair, as with A1, changes **where and on what the comparison is
made**, never the frozen values.

**Definition (A2(a) — the budget axis, primary on the 12 separating cells).** At the frozen
comparison terms (frozen `PlanBudget`, H ∈ {1, 2, 4, 8}, n = 512, joint split, the 12 separating
cells of the #843 record §3), the geometry statistic is the **paired per-instance relative
reduction in deployed search work at exactly preserved correctness**: expansions are the gating
counter; candidates and table reads are co-reported. An ordering arm is comparable in a cell only
while its correct-outcome rate **equals** the baseline's rate in that cell — correctness is never
traded for work, and any correctness regression disqualifies the arm in that cell regardless of its
counters. The comparison bar is the **strongest non-geometric ordering control** under identical
budget and byte terms; the statistic is the one-sided 95% lower confidence bound on the paired
relative expansion reduction, and the frozen threshold is **ρ_min = 0.10**. The intersection-union
conjunction over the 12 cells is the gate reading; Holm–Bonferroni is reported alongside (the #843
§6 dual reading).

**Definition (A2(b) — the correctness axis, probe-admitted cells).** A probed
(family × horizon × budget) cell is **admissible** iff the strongest of {`bounded-breadth-first`
under that budget, every non-oracle null} sits at or below **1 − δ_min** at n = 512 on the joint
split — otherwise the cell would recreate the saturated-baseline trap. The binding cheap
instrument (`geometry_probe_845.rs`, authorized by the 2026-08-22 decision; shipping with #845's
build increments) ran the full surface on 2026-08-22 — five families × H ∈ {1, 2, 4, 8, 12, 16} at
the frozen budget plus a frontier/expansion ladder at H = 8; 70 cells, 33.5 s, teacher-free —
and admitted **18** cells. The two greedy-solvable families admitted **zero** cells at every probed
setting, consistent with §11 and the #843 verdict.

**Definition (the frozen A2(b) cell sets).** All A2(b) cells lie in the three separating families
{graph-navigation, constraint-satisfaction, multi-hop-evidence}; every budget below is the frozen
`PlanBudget` with exactly the named field tightened. Rates are `bounded-breadth-first`
correct-outcome / strongest-null correct-outcome at n = 512, joint split (probe record, 2026-08-22;
the strongest null is `direct-continuation` in every A2(b) cell). The **bar** a geometry arm must
clear by δ_min is the larger of the two.

*Primary (gating, 9 cells) — the frontier ladder at H = 8:*

| family | budget | baseline | strongest null | bar | headroom |
|---|---|---|---|---|---|
| graph-navigation | frontier-16 | 0.5391 | 0.8340 | 0.8340 | 0.1660 |
| graph-navigation | frontier-8 | 0.3047 | 0.8340 | 0.8340 | 0.1660 |
| graph-navigation | frontier-4 | 0.1719 | 0.8340 | 0.8340 | 0.1660 |
| constraint-satisfaction | frontier-16 | 0.5625 | 0.7422 | 0.7422 | 0.2578 |
| constraint-satisfaction | frontier-8 | 0.3145 | 0.7422 | 0.7422 | 0.2578 |
| constraint-satisfaction | frontier-4 | 0.1914 | 0.7422 | 0.7422 | 0.2578 |
| multi-hop-evidence | frontier-16 | 0.5391 | 0.8340 | 0.8340 | 0.1660 |
| multi-hop-evidence | frontier-8 | 0.3047 | 0.8340 | 0.8340 | 0.1660 |
| multi-hop-evidence | frontier-4 | 0.1719 | 0.8340 | 0.8340 | 0.1660 |

*Secondary (reported with full rigor, non-gating, 9 cells):* the expansion ladder at H = 8
(`expansions-64` and `expansions-32` in the three families — the baseline collapses to 0.0000
there, so the bar degenerates to the null alone and the cell attributes headroom to *any* informed
ordering rather than to geometry specifically), and the frozen-budget H = 16 cells in the three
families (bars 0.9258 / 0.9395 / 0.9258 — the bar is the baseline itself and the headroom is below
2 × δ_min: **TIGHT**, the A1-d marker convention).

**Definition (A2(b) gate reading).** On the nine primary cells the geometry arm's paired one-sided
95% lower bound against the bar arm must clear δ_min in **every** cell (intersection-union), with
Holm–Bonferroni reported alongside. Every arm, null, and control in an A2(b) cell runs under that
cell's tightened budget with budget parity asserted by counters; the nulls that do not consume a
`PlanBudget` (retrieval, continuation, memorization replays) are unchanged by the tightening and
are measured as-is, which is exactly why they can be the bar.

**Empirical Criterion (probe non-vacuity). Status: Empirical.** The probe's admissibility judge is
required to be able to fire and to fail (a built test asserts both directions on synthetic cells,
and that `direct-continuation` has real failures on a trap family), and the probe's saturated
verdicts reproduce the #843 record where the grids overlap (frozen H = 8: baseline 1.0000 and
identical strongest-null rates). A future re-freeze of any A2(b) cell set re-runs the probe first;
its verdict binds.

**What A2 does not change.** No cell of the frozen 20-cell grid is removed or re-gated; the frozen
primary statistic on those cells is untouched; the #845 restriction to the 12 separating cells for
any frozen-terms correctness comparison stands; the 8 greedy-solvable cells remain excluded
everywhere; and no geometry claim is licensed by this amendment — it defines where one could be
earned.
