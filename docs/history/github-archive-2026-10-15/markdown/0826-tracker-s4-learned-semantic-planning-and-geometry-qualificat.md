# #826 — TRACKER/S4: learned semantic planning and geometry qualification

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:32:22Z
- **Updated:** 2026-08-23T02:14:30Z
- **Closed:** 2026-08-23T02:14:30Z
- **Milestone:** R4 Intelligence / S4 — Semantic planning and geometry
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/826
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent programme: #820

## Objective

Establish artifact-backed typed state/action transitions and bounded planning that generalize under interventions; qualify geometric steering only by downstream value against strong equal-budget baselines.

## Current evidence

- The f64 W(3,3) router/dashboard is exploratory and explicitly outside the graph migration path.
- #605/#804 produced a real route-attention run, but 119/120 heads were vacuous and the arm failed its fit contract.
- Trajectory variance under paraphrase is not by itself evidence of semantics or reasoning.

## Entry criteria

S2 and S3 have promoted verdicts, so planning can be evaluated on prompt-causal, selectively served, free-running behavior.

## Native dependencies

Blocked by #824 and #823.

## Promotion gate

Typed-state planning beats retrieval-only, memorized-trajectory, shuffled-state, and direct-continuation baselines on held-out entities, compositions, topologies, relabelings, and horizons. Any geometry must add a reproducible gain under equal bytes/candidates/operations.

## Kill / redesign criterion

Gains that disappear under relabeling or unseen composition are memorization. Geometry that cannot beat Hamming/binary/VSA/spectral controls remains offline/reference-only.

## Native children

- [ ] #844 — Freeze compositional benchmarks and typed state/action semantics
- [ ] #843 — Learn and execute bounded transitions with plan witnesses
- [ ] #845 — Qualify W(3,3) against equal-budget baselines
- [ ] #846 — Certify compositional planning and bound the claim

GitHub's native parent/sub-issue relationship is the hierarchy source of truth.

## Non-goals

- Directly wire the current f64 router into R4G1.
- Call path smoothness, nearest-neighbor retrieval, or memorized chains general reasoning.

## Repository conformance

Primary IDs: RF-01, RF-08, RF-12, RF-13, RF-27, RF-28. Reference geometry/planners remain explicitly scoped; machine claims attach only to the lowered deployed implementation.

Every leaf declares execution scope, preserves the applicable P-4/determinism/allocation/no_std/error/witness contracts, maps to existing RF capabilities or justifies a new one, and updates generated conformance only through its TOML/Gherkin/test sources. Empirical work posts the repository run contract before expensive execution.

## Closure rule

Close only after every native child has a completed, negative, not-triggered, or explicitly descoped verdict and this tracker records `PROMOTE`, `REVISE`, `LIMIT`, or `RETIRE` with exact evidence links.

## Comments (5)

### comment 5377754414 — 2026-08-22T04:02:15Z — Casey-allard

## S4 entry reconciliation — AMEND + PROCEED (2026-08-22)

S3 (#824) closed `LIMIT` and S2 (#823) stands at `REVISE`; neither is a `PROMOTE`, so this tracker's entry prose ("S2 and S3 have promoted verdicts") is superseded and the native blocked-by edge to #823 is converted to a sanctioned parallel-start. Maintainer decision, recorded here as the source of record (readable mirror in `docs/r4_intelligence_completion_plan.md`).

**Amended entry criterion — S4 builds on the established substrate, not on promoted free-running generation:**
- **S1 — prompt causality PROMOTED** (RF-31; deployed skip-mix lane, +28.45‰ [25.57, 31.32]).
- **S2 — honest selective prediction frozen (REVISE):** typed eight-status abstention contract shipped; NO calibrator qualified; #839 phase-2 calibrated mode stays independently trigger-gated and does NOT block S4.
- **S3 — LIMIT:** deployed artifact is a teacher-forced retrieval/continuation system; coherent free-running generation is not established at this representation scope.

**What S4 evaluates.** Typed-state planning is measured as a self-contained capability against this tracker's existing promotion gate — beating retrieval-only, memorized-trajectory, shuffled-state, and direct-continuation baselines on held-out entities/compositions/topologies/relabelings/horizons, with any geometry gain shown under equal bytes/candidates/operations. That gate never depended on free-running generation, so the LIMIT verdict does not weaken it; planning results are NOT to be read as free-running coherence (the #824 boundary stands).

**Parallel-start sanction.** S4 may begin now in parallel with S2's open re-entry; #839 stays parked under its trigger. Item order: #844 (freeze benchmarks + typed state/action semantics) → #843 (bounded transitions with plan witnesses) → #845 (geometry vs equal-budget baselines) → #846 (certify + bound the claim). Frozen gates unchanged (20‰ causal floor, #838 selective gates, #841 §6 bar for any generation re-entry).

**Next executable issue = #844** (S4 item A, specification-first).


### comment 5377858394 — 2026-08-22T04:25:39Z — Casey-allard

Reopened. This tracker was auto-closed in error when PR #915 (the S4 entry-reconciliation readable mirror) merged — GitHub's keyword parser matched a "close" token near "#826" in the PR body (it ignores the surrounding negation), not an intentional closure. **S4 is the active stage and remains OPEN.** The AMEND + PROCEED decision stands (see the maintainer decision comment above); next executable issue = #844.


### comment 5381722954 — 2026-08-22T17:38:36Z — Casey-allard

## S4 item B (#843) — source-of-record verdict: **LIMITED**

Item B is closed. This comment is the tracker's source of record for what it established.

### What is now established

A bounded, fixed-capacity, allocation-free, P-4-only semantic-transition planner exists **on the deployed path** and is registered as **RF-33** (`normative-runtime` / `deployed-serving`). It reaches **correct-outcome rate 1.0000 in all 20 joint-split cells**, held-out topologies included.

**Lowered arm: `bounded-breadth-first`.** It ties exactly with `table-guided-beam` — both 1.0000 in all 20 cells with identical lower bounds in every cell. The tie is broken toward breadth-first because it uses **no scoring heuristic at all**, making it the plainest possible baseline for #845's geometry to have to beat. The tie is recorded rather than implied; the beam remains an equal-scoring alternative.

### What is not established, and why

The planner clears the frozen δ_min = 0.05 effect floor over the strongest non-oracle null on **12 of 20** cells. On the other 8 it does not — because **two of the five families are greedy-solvable**, so `direct-continuation` is also at 1.0000 and the reachable headroom is structurally below the floor. That is a property of the frozen #844 task generator, not a planner deficiency.

Amendment A1 repaired four real degeneracies in that generator and its four gate instruments all **PASSED before any fitting**. This ceiling is a fifth one A1's instruments were not shaped to detect. Recorded, not papered over.

### Item status

| Item | Issue | State |
|---|---|---|
| A — benchmark constitution, typed reference model, RF-32 | #844 | CLOSED |
| B — bounded semantic transitions, RF-33 | #843 | **CLOSED — LIMITED** |
| C — geometry qualification | #845 | **RELEASED, RESTRICTED** (see below) |
| D — S4 certification | #846 | Blocked on C; sealed partitions never opened |

### #845 trigger — RELEASED, RESTRICTED

Geometry qualification may begin. It must be measured **only on the 12 separating cells**.

Running it on the 8 greedy-solvable cells would pit geometry against a baseline already at 1.0000. That cannot separate anything, and a null result there would read as a geometry failure when it is a task property. The restriction protects #845 from being falsely refuted.

`status: on-hold` is removed from #845 accordingly.

### The S4 boundary that still stands

The S4 promotion verdict is **#846's to make**, against the sealed composition and topology cells that #843 never opened. Nothing in item B licenses a general reasoning claim; the #824 free-running-coherence boundary and the #823 calibrated-confidence boundary both stand unchanged.


### comment 5382592772 — 2026-08-22T21:04:50Z — Casey-allard

## S4 item C (#845) — source-of-record verdict: **NO GEOMETRIC ADVANTAGE**

Item C is closed. This comment is the tracker's source of record for what it established. Full measurement: [`docs/w33_geometry_qualification_845.md`](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/w33_geometry_qualification_845.md); frozen contract: [`docs/w33_geometry_qualification_spec_845.md`](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/w33_geometry_qualification_spec_845.md).

### What is now established

Under the frozen equal-budget, equal-byte comparison, the pinned W(3,3) mapping provides **no measurable planning advantage** on either pre-registered axis:

- **Budget axis (12 separating cells, frozen budget):** geometry holds correct-outcome 1.0000 everywhere — the mapping does no harm — but does 12–37% **more** search work than the incumbent goal-distance beam in every cell where reduction is structurally possible. Conjunction FAIL, 5/12 (the five passes are the degenerate H = 1 cells).
- **Correctness axis (9 primary tightened-frontier cells):** conjunction FAIL, 0/9 (0/9 secondary). The bar is an informed **non-geometric** ordering in every primary cell — goal-distance or Hamming/popcount at 1.0000 against geometry at 0.0742–0.6426.
- **Attribution:** geometry sits at the level of its own randomized, relabelled, and phase-swapped controls; its gain over breadth-first attributes to informed ordering generally, not to quadrangle structure. The mod-9 digit quantization's wrap-around severs slot distance from task-step distance.

The programme falsifier is discharged as written: a geometry arm that cannot beat Hamming/binary/VSA/spectral controls under equal bytes, candidates, and operation budgets **stays out of the production runtime**. This is the pre-registered negative branch — a successful falsification, recorded as first-class completion evidence.

### What this does not say

It does not refute W(3,3) as mathematics; it does not touch the f64 geometric router's separately measured content-retrieval results, which live outside the P-4 kernel by design; and it licenses no claim about other mappings. Re-entry requires a new mapping Definition with a pre-registered alignment rationale, a fresh design contract and probe, under the same frozen controls (record §7). F0 (#859) was never started; its hold stands; no Lean pin was needed because no lowering is proposed.

### Item status

| Item | Issue | State |
|---|---|---|
| A — benchmark constitution, typed reference model, RF-32 | #844 | CLOSED |
| B — bounded semantic transitions, RF-33 | #843 | CLOSED — LIMITED |
| C — geometry qualification | #845 | **CLOSED — NO GEOMETRIC ADVANTAGE** |
| D — S4 certification | #846 | **Next executable** — all native blockers closed |

### What S4 item D inherits

#846 certifies against the lowered non-geometric baseline (RF-33 `bounded-breadth-first`) on the sealed composition and topology partitions #843 never opened, with the geometry-gain clause of this tracker's promotion gate now settled negatively by evidence. The S4 promotion verdict remains #846's to make. Frozen gates unchanged (δ_min = 0.05, n = 512, H ∈ {1, 2, 4, 8}, the 20‰ causal floor, the #838 selective gates, the #841 §6 bar); the #824 and #823 boundaries stand; `CONFORMANCE.md` unchanged at 33 ids.


### comment 5383751839 — 2026-08-23T02:14:26Z — Casey-allard

## S4 closure verdict — LIMIT

All native children now have completed evidence-backed dispositions, satisfying this tracker closure rule:

- #844 **COMPLETED WITH CORRECTION** — [frozen benchmark/state-action constitution](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/compositional_planning_spec_844.md). The #846 append-only correction records that the intended final composition/topology seal was not actually materialized or access-bound.
- #843 **LIMITED** — [bounded semantic transitions](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/bounded_semantic_transitions_843.md). RF-33 clears the frozen effect floor in 12/20 previously evaluated joint-split cells, with deployed witness, bounded-work, allocation, P-4, deterministic, and typed-decline guarantees.
- #845 **NO GEOMETRIC ADVANTAGE** — [W(3,3) qualification](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/w33_geometry_qualification_845.md). The pinned geometry does not beat the informed non-geometric ordering arm under equal budgets and remains offline/reference-only.
- #846 **REASONING NOT ESTABLISHED** — [final certification record](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/compositional_planning_certification_846.md). The binding pre-access audit found zero valid untouched final cells, so the final grid is NOT_RUN and dependent evidence is UNAVAILABLE.

**Stage verdict: LIMIT.** Typed state-transition planning is retained only at the narrow #843 boundary; no unseen-final-partition, general-reasoning, free-running, or geometric capability promotion flows downstream. Re-entry requires a new versioned benchmark with materialized composition cells, non-empty partition CIDs, access-log binding, reserved cells, and a second domain frozen before candidate fitting/selection.

The readable programme mirror and current research summary landed through protected merge queue PR #930 at [9aeb5427c7fd3a965b0e4a0228f137889dbafa9b](https://github.com/UOR-Foundation/uor-r4/commit/9aeb5427c7fd3a965b0e4a0228f137889dbafa9b).


---
_Comment/review ids in this file: 5377754414, 5377858394, 5381722954, 5382592772, 5383751839_

