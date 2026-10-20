# #844 — reasoning/#826-A: freeze compositional benchmarks and artifact-backed state/action semantics

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:39:19Z
- **Updated:** 2026-08-22T07:51:16Z
- **Closed:** 2026-08-22T05:59:55Z
- **Labels:** conformance
- **Milestone:** R4 Intelligence / S4 — Semantic planning and geometry
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/844
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #826

## Context

The repository contains reference planning/semantic-state ideas, but their owned/f32 scope is not a deployed capability. A defensible reasoning programme needs intervention-resistant tasks and a byte-level semantic contract before choosing a planner or geometry.

## Problem

Multi-hop continuation, graph retrieval, memorized paths, and smooth routing can all look like reasoning on non-interventional benchmarks. Undefined state/action/goal semantics also make witnesses, capacity bounds, and proofs impossible to pin.

## Goal

Freeze verifiable compositional task suites and a versioned artifact/reference model for typed states, actions, effects, goals, constraints, evidence, uncertainty, and plan witnesses.

## Scope

- [ ] Define task families for graph navigation, symbolic transformations, constraint satisfaction, multi-hop evidence composition, and counterfactual intervention.
- [ ] Split by entity, surface vocabulary, topology, operator composition, prompt template, and reasoning horizon.
- [ ] Add entity relabeling, irrelevant context, counterfactual edge/effect changes, and unseen-composition controls.
- [ ] Include retrieval-only, direct-continuation, memorized-trajectory, shuffled-state/action, shortest-path oracle, and trivial-prior baselines.
- [ ] Use objectively checkable outcomes and replayable intermediate states; version any secondary rubric/judge.
- [ ] Freeze primary exact/valid-plan metrics, horizon progression, effect floors, uncertainty, resource budgets, and sample sizes.
- [ ] Define stable artifact IDs and reference semantics for state, action, precondition, effect, goal, constraint, evidence/provenance, confidence, transition, and plan witness.
- [ ] Define deterministic goal satisfaction, transition validity, conflict/unknown handling, tie-breaking, saturation, capacity/overflow, and typed decline.
- [ ] Specify interaction with prompt/session state and selective prediction while preventing hidden future-answer leakage.
- [ ] Project optional sections, IDs/offsets, fixed capacities, operations, bytes, and proof obligations; exclude owned strings, unbounded collections, and f32 from the deployed form.

## Non-goals

- Treat nearest-neighbor retrieval, paraphrase stability, or path smoothness as reasoning.
- Design around W(3,3) before non-geometric state/action semantics exist.
- Use benchmark examples that share entities/topologies/compositions with fitting data.

## Dependencies and entry gate

Requires promoted S1/S2/S3 verdicts for final freeze. Schema/generator drafting may start earlier but cannot assume capabilities those stages do not establish.

## Acceptance criteria

- [ ] Every task has a deterministic verifier and replayable gold/intermediate state.
- [ ] Split axes prevent entity, topology, surface, and composition memorization.
- [ ] The reference transition/witness semantics are total over valid inputs and decline deterministically at capacity/unknown boundaries.
- [ ] Each public reasoning phrase is mapped to an exact measured task/horizon and formal-vocabulary status.
- [ ] A retrieval/memorized-path and a shuffled-state planted model fail the primary generalization gate.

## Verification

- [ ] Generator determinism, split leakage, relabeling/counterfactual metamorphic, and tamper tests.
- [ ] Reference transition/precondition/effect/goal/witness unit and property tests.
- [ ] Capacity/overflow/conflict/unknown/tie/saturation negative fixtures.
- [ ] Baseline non-degeneracy and power-analysis reproduction.
- [ ] Schema examples round-trip and map to the conformance/proof matrix.

## Compatibility and migration

Introduce only optional/versioned experimental schemas until a planner is selected. Historical artifacts remain valid and cannot be reported as planning-capable without the new sections and report identities.

## Repository conformance

- **Execution scope:** Benchmark and reference semantics; no deployed planning claim; “reasoning” in public claims is limited to the exact typed state-transition/planning tasks measured.
- Preserve applicable P-4, zero-allocation steady state, deterministic bytes/reductions, fixed capacities, bounded work, typed decline/errors, no_std, and replayable witnesses.
- Normative math uses the formal vocabulary; empirical criteria name split axes, horizons, n, uncertainty, identities, and status.
- **Conformance mapping:** RF-01, RF-08, RF-12, RF-13, RF-27, RF-28.. Existing reference IDs are not deployed-planning evidence.
- Built capability order: IDs TOML → tagged Gherkin → failing marker/behavior test → implementation → regenerated conformance.
- Unselected geometry/planner arms remain reference/open or are retired with explicit gates; generated conformance is never hand-edited.
- Claim-changing results append evidence and reconcile all current summaries.

## Claim status and next action

Completion establishes a falsifiable target and byte-level meaning, not reasoning performance.

## Comments (1)

### comment 5379101194 — 2026-08-22T07:51:15Z — Casey-allard

## Definition of Done — #844 (S4 item A) complete

Delivered across four PRs, each independently green through the full gate ladder and merged via the protected merge queue:

- **#916** → main `3c691c33` — froze the design (`docs/compositional_planning_spec_844.md`): the compositional-planning benchmark constitution + typed state/action reference-model contract.
- **#917** → main `f631a133` — froze the `s4-compositional-reasoning` benchmark constitution in the #832 capability-suite framework (primary metric `held-out-valid-plan-rate`; six generalization split axes; the six planning baselines as `ControlKind` variants).
- **#918** → main `a51af02f` — the typed reference model `crates/uor-r4-graph-compiler/src/compositional_planning.rs`, extending the RF-27 `semantic_state` model and the RF-08 planner: `PlanWitness` + total/deterministic independent `verify()`, `Evidence`/provenance, ordinal `ConfidenceBand`, typed `DeclineReason`, and deterministic F1–F5 generators with a BFS reference solver producing replayable gold plans, a content-addressed sample id, and a relabeling metamorphic transform (11 module tests).
- **#919** → main `3a4801c4` — registered conformance id **RF-32** `compositional_planning_benchmarks` (certifier-instrument / off-serving-path, fixture-gated) with a tagged Gherkin suite, executable bdd steps, the repo-conformance marker, and regenerated `CONFORMANCE.md` (31 → 32 ids).

**Acceptance criteria:** every task family has a deterministic verifier + replayable gold; the split axes and the relabeling metamorphic foreclose entity/topology/composition memorization; the reference transition/witness semantics are total over valid inputs and decline deterministically at capacity/unknown boundaries; each public reasoning phrase is mapped to an exact task/horizon and formal-vocabulary status; a plan that reaches the goal via an invalid intermediate step, and a witness stripped of cited evidence, both fail verification.

**Boundaries held (S4 entry reconciliation):** teacher-forced scope only (S3 #824 LIMIT — no free-running generation); ordinal honest decline, not calibrated confidence (S2 #823 REVISE); reference-only / off-serving-path. This establishes a falsifiable target and byte-level meaning, **not** reasoning performance — no deployed-planning claim.

**Verification:** each PR passed fmt, clippy `-D warnings` (workspace/all-targets/all-features), `cargo test --workspace` (178 suites; the bdd run covers the five new `@RF-32 @build` scenarios), no_std ladder, wasm lib build, `xtask validate` (R1 check-model now 32 ids / R4 / R5 / gnaf), and `check_claim_wording`. `register conformance` stayed green through the queue.

**Process note:** this issue auto-closed on the #918 merge (05:59:55Z) rather than the #919 merge, because the #918 PR body used the phrase "(which closes #844)" and GitHub's keyword parser matched it despite the surrounding clause (the same trap as #915→#826). All work is now on main as of `3a4801c4`, so the closed state is accurate; recorded for future avoidance.

**Downstream:** S4 tracker #826 remains open; the next executable S4 item is **#843** (item B — learn/execute bounded semantic transitions with replayable plan witnesses), which consumes this frozen constitution and reference model.



---
_Comment/review ids in this file: 5379101194_

