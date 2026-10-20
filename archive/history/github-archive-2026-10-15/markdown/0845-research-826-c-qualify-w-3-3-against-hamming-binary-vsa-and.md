# #845 — research/#826-C: qualify W(3,3) against Hamming, binary, VSA, and spectral baselines

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:39:20Z
- **Updated:** 2026-08-22T21:04:49Z
- **Closed:** 2026-08-22T21:03:51Z
- **Milestone:** R4 Intelligence / S4 — Semantic planning and geometry
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/845
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #826

## Context

The current f64 W(3,3) router/dashboard is exploratory and outside the graph migration path. The real #605/#804 route-attention arm ran, but fit was about 0.396 versus N1 0.192 and N2 0.292 and 119/120 heads were vacuous; runtime checks passing millions of steps did not establish useful operator activation.

## Problem

Directly wiring a 96-vertex phase space into R4G1 would conflate visualization/reference geometry with learned semantics and ignore cheaper non-geometric explanations. Trajectory variance below 5% is neither sufficient nor necessarily desirable.

## Goal

Only after a valid non-geometric planner exists, test whether a precisely mapped W(3,3) mechanism provides downstream interventional planning value under equal storage/candidate/operation budgets.

## Scope

- [ ] Keep this issue `status: on-hold` until sibling item B produces a non-degenerate production planning baseline and records the trigger comment.
- [ ] Define an explicit mapping from artifact states/actions/goals/evidence to W(3,3) vertices/phases and an inverse/readout semantics; no visualization-only mapping.
- [ ] Pin the exact #859 theorem/library revision used by each geometry arm and verify that the implementation satisfies its stated preconditions.
- [ ] Do not count a Lean theorem as evidence of semantic usefulness, non-vacuous activation, or empirical superiority.
- [ ] Define distance/path/phase scoring mathematically and specify a deterministic integer/fixed-table lowering; f64 remains compiler/reference only.
- [ ] Compare Hamming/popcount, learned binary codes, VSA/binding, spectral/graph distances, random/isomorphic embeddings, and no-geometry under equal bytes, candidates, table reads, and operations.
- [ ] Test relabeling, graph isomorphism, topology change, unseen composition, longer horizons, and adversarial phase permutations.
- [ ] Measure primary downstream valid-plan/generalization metrics first; report trajectory stability only as a diagnostic.
- [ ] Attribute any gain to candidate expansion, ordering, transition scoring, or heuristic search reduction.
- [ ] Record fit, activation/non-vacuity, resource cost, and all negative arms.
- [ ] Do not implement the current f64 router in production; propose packed lowering only after a positive equal-budget verdict.

## Non-goals

- Assume W(3,3) because it is mathematically elegant.
- Use paraphrase trajectory variance as the primary gate.
- Replace missing state/action evidence with geometry.
- Modify `uor-r4-router` as part of the graph migration path.

## Dependencies and entry gate

Blocked by #843 and #859. #859 supplies only theorem-checked reference definitions and assumptions; this issue must still establish a valid semantic mapping, non-vacuous activation, and empirical advantage. If the W(3,3) definition remains inconsistent with the dashboard's 96-vertex canvas, no W(3,3) run launches. The non-geometric baseline and benchmark identities must be frozen, and a fresh run contract is required; prior route-attention results are evidence, not an activation.

## Acceptance criteria

- [ ] Mapping and readout are total, testable, and independent of surface labels.
- [ ] All baselines share exact storage/candidate/operation budgets and are non-degenerate.
- [ ] A promoted geometry improves the primary interventional planning metric, not merely smoothness or visualization.
- [ ] Gain survives relabeling/isomorphism/topology/composition controls with a positive confidence-bound margin.
- [ ] Verdict is `PROMOTE FOR LOWERING`, `REVISE`, `NO GEOMETRIC ADVANTAGE`, or `NOT TRIGGERED`.

## Verification

- [ ] Mapping round-trip/isomorphism/metamorphic tests and adversarial phase-permutation negatives.
- [ ] Integer-lowering versus f64/reference decision differential if the reference arm is positive.
- [ ] Equal-budget and activation/non-vacuity audit.
- [ ] Independent primary-metric/resource report recomputation.
- [ ] No dependency from production graph crates to the f64 router unless a separately reviewed positive lowering issue is authorized.

## Run contract

- **Metric/current value:** Primary held-out interventional valid-plan/generalization statistic from sibling A/B; route-attention historical fit failed and 119/120 heads were vacuous, so activation/non-vacuity is a co-primary prerequisite.
- **Reachability ceiling:** Compute the fraction of planning decisions whose candidate/order/heuristic can actually change under geometry and the maximum search/quality movement available within fixed horizon/frontier.
- **Pinned identities:** Planning benchmark/splits, non-geometric artifact/baseline, each embedding/mapping/scorer, budgets, seeds, compiler/reference revision, traces, and reports.
- **Nulls/falsifier:** No geometry, Hamming/popcount, learned binary, VSA, spectral, random/isomorphic embeddings, phase permutation, label relabeling, equal-candidate/equal-operation controls.
- **Binding cheap instrument:** Mapping/readout tests, non-vacuous activation on a cheap fixture, equal-budget audit, and reachability ceiling above the effect floor. Any vacuous arm stops the long run.
- **Exit rule:** Promote only if W(3,3) lower confidence bound beats every declared baseline on downstream interventional performance across controls while integer-lowering/resource projections fit.
- **If positive:** Open a separate packed-format/runtime lowering issue with explicit P-4/witness obligations.
- **If negative:** Publish NO GEOMETRIC ADVANTAGE and keep W(3,3) in exploratory/reference scope; continue with the strongest simpler planner.
- **Cost estimate:** Post compiler/run wall time, peak RSS/storage, bytes, candidates/operations, horizons, and shared compute.

Post exact values and pass the cheap instrument before a full run. Controls must be non-degenerate; unavailable observations are `UNAVAILABLE`/`null`.

## Compatibility and migration

Research/reference only. No production format or dependency changes in this issue. Any later lowering must be optional/versioned and cannot import f64 router semantics into the hot path.

## Repository conformance

- **Execution scope:** Offline/reference geometry evaluated through fixed production-planning readouts; not deployed serving; “reasoning” in public claims is limited to the exact typed state-transition/planning tasks measured.
- Preserve applicable P-4, zero-allocation steady state, deterministic bytes/reductions, fixed capacities, bounded work, typed decline/errors, no_std, and replayable witnesses.
- Normative math uses the formal vocabulary; empirical criteria name split axes, horizons, n, uncertainty, identities, and status.
- **Conformance mapping:** RF-08, RF-12, RF-13, RF-27, RF-28 and the open route-fit/target-operator ledger entries.. Existing reference IDs are not deployed-planning evidence.
- Built capability order: IDs TOML → tagged Gherkin → failing marker/behavior test → implementation → regenerated conformance.
- Unselected geometry/planner arms remain reference/open or are retired with explicit gates; generated conformance is never hand-edited.
- Claim-changing results append evidence and reconcile all current summaries.
- Geometry remains a hypothesis until it beats non-geometric controls; Lean/Kani/source checks prove only their encoded obligations.

## Claim status and next action

The current status is ON HOLD and W(3,3) has no production claim. A negative result is a successful falsification.

## Comments (3)

### comment 5381723010 — 2026-08-22T17:38:37Z — Casey-allard

## Trigger satisfied — **RELEASED, RESTRICTED**. `status: on-hold` removed.

S4 item B (#843) closed as **LIMITED** and lowered a non-geometric planning baseline, which is the condition this issue was held on. Geometry qualification may begin.

### The restriction, and why it exists

Measure **only on the 12 separating cells** identified in [`docs/bounded_semantic_transitions_843.md`](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/bounded_semantic_transitions_843.md) §3.

On the other 8 cells, two of the five families are **greedy-solvable**: the lowered baseline and the trivial `direct-continuation` null both sit at **1.0000**. Geometry cannot separate against a baseline that is already perfect. A null result on those cells would look like a geometry failure while actually being a property of the frozen #844 task generator — so running them would manufacture a false refutation of W(3,3).

This restriction protects this issue from being wrongly refuted. It is not a reduction in rigour: the 12 cells are the ones where headroom demonstrably exists.

### The baseline to beat

**`bounded-breadth-first`**, RF-33, `normative-runtime` / `deployed-serving`. It uses no scoring heuristic at all — deliberately the plainest possible baseline. It ties exactly with `table-guided-beam` (both 1.0000 in all 20 cells, identical lower bounds), and the tie was broken toward breadth-first for exactly this reason.

Frozen comparison terms, unchanged from #844 §2.5: primary statistic **held-out valid-plan rate**, n = **512** instances per held-out cell per horizon, H ∈ **{1, 2, 4, 8}**, **δ_min = 0.05**, measured on the **joint split** (held-out instances are in the high half of *every* seed-varied axis — entity, vocabulary, topology, template — per #844 §2.2).

### Before launching

Publish a run contract first. Do not launch if the reachable ceiling is below the target effect, if the cheap instrument fails, if the experiment cannot distinguish branches, or if both branches lead to the same action. #843's own finding is the cautionary case: its A1 gate instruments all passed and a *different* ceiling still capped 8 of 20 cells. Check the reachable ceiling on the 12 cells explicitly before committing to a long run.

Unassigned and open for pickup — not claimed by this session.


### comment 5382099987 — 2026-08-22T19:07:26Z — Casey-allard

## Maintainer decision (2026-08-22): #859 edge converted under scoped terms; Amendment A2 sanctioned; failure-surface probe authorized

Recorded here as the source of record (readable mirror lands in this issue's design-freeze PR). Decision made by the maintainer 2026-08-22, after the zero-ceiling finding below.

### 1. The #859 edge — converted, with its intent preserved

The native blocked-by edge to #859 is converted to a scoped waiver (the #826/#823 parallel-start precedent; the edge itself stays as history). **F0 (#859) remains held and is not restarted by this issue.** Terms:

- The W(3,3) mapping used by every geometry arm ships as **formal-vocabulary Definitions** in this issue's design spec: the 40-point/40-line generalized quadrangle GQ(3,3) stated precisely; the artifact state/action/goal/evidence → vertex/phase mapping with total inverse/readout semantics; and an explicit reconciliation of the dashboard's 96-vertex canvas — reconciled, or declared non-normative for this issue's arms. No visualization-only mapping (unchanged from this issue's scope).
- **A `PROMOTE FOR LOWERING` verdict does not by itself authorize production lowering.** The separate lowering issue it would open must re-acquire the exact #859 theorem/library pin (Level-1 reference) before any production geometry lands. Lean gates *lowering*, not *measurement* — consistent with this issue's own rule that a Lean theorem is never evidence of semantic usefulness. The scope bullet "pin the exact #859 theorem/library revision used by each geometry arm" is amended accordingly: deferred to the lowering issue.

### 2. Zero-ceiling finding (recorded)

On the 12 separating cells at the frozen comparison terms, the baseline to beat — `bounded-breadth-first` (RF-33) — is at correct-outcome rate **1.0000** ([`docs/bounded_semantic_transitions_843.md`](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/bounded_semantic_transitions_843.md) §3). A rate cannot exceed 1, so the reachable effect on held-out valid-plan rate against that baseline is **0 < δ_min = 0.05**. As framed at release, this issue's primary run could never launch under the repository run contract. Same shape as the 8-cell restriction already recorded: a comparison is only informative where a comparison is possible.

### 3. Amendment A2 — sanctioned (append-only, the A1 precedent; no frozen value moves)

The geometry comparison is re-framed onto axes where headroom exists:

- **(a) Primary, on the 12 separating cells:** pre-registered paired **budget-reduction at equal correctness**, using the deployed `PlanCounters` (expansions / candidates / table reads) under the frozen `PlanBudget`. The reduction threshold is pre-registered in the A2 text **before any geometry arm is fitted**.
- **(b) Correctness axis, on probe-identified cells:** extended-horizon (H ≤ H_MAX = 16) and tightened-budget (reduced frontier / expansions) cells where the baseline demonstrably degrades, appended A1-style. A cell is admitted only where the probe shows the strongest of {`bounded-breadth-first`, the non-oracle nulls} at or below 1 − δ_min under the probed setting — otherwise it would recreate the saturated-baseline trap.

δ_min = 0.05, n = 512/cell, H ∈ {1, 2, 4, 8}, H_MAX = 16, W_MAX = 64 and every other #844 §2.5 value stand unchanged. A2 adds cells and a budget-axis statistic; it moves nothing frozen.

### 4. Probe — authorized (binding cheap instrument)

Minutes-scale, teacher-free, fixture-free: map where `bounded-breadth-first` actually degrades — five families × H ∈ {8, 12, 16} at the frozen budget, plus a tightened frontier/expansion ladder at H = 8 — with `direct-continuation` measured alongside so a greedy-solvable cell cannot slip into A2(b). Its verdict binds: no A2(b) cell is frozen except from the measured failure surface, and if the probe finds **no** admissible cell, A2(b) is recorded empty and the correctness axis is `NOT TRIGGERED` (the budget axis A2(a) proceeds regardless).

### Execution

Assigned as the active-work signal. Work proceeds in the #843 cadence: design-freeze docs PR first (References this issue), then focused build increments, each queue-merged. The RESTRICTED condition from the release comment (12 separating cells only, for any correctness comparison at the frozen terms) stands.


### comment 5382592692 — 2026-08-22T21:04:49Z — Casey-allard

## Definition-of-Done evidence — CLOSED, verdict: NO GEOMETRIC ADVANTAGE

Delivered across four pull requests, each queue-merged with the full gate ladder, no bypass, no weakened gate:

| Increment | PR | Merge commit | Content |
|---|---|---|---|
| 1 — design freeze | #926 | `6f6c3f1f` | Frozen design contract (`docs/w33_geometry_qualification_spec_845.md`); Amendment A2 (#844 spec §12); plan-mirror activation entry |
| 2 — mapping + instruments | #927 | `7133c883` | W(3,3) reference object with GQ(3,3) invariants machine-checked; the state mapping mu; the failure-surface probe with non-vacuity gates |
| 3 — arms + harness | #928 | `287a6156` | Seam-opened reference search, counter-exact parity vs the deployed planner; eight ordering arms/controls with pinned constants (§4-A); equal-bytes audit; §5-A per-cell reading |
| 4 — measurement + verdict | #929 | `ddf4a142` | Both A2 axes measured; append-only record (`docs/w33_geometry_qualification_845.md`); plan-mirror verdict entry + mechanism-boundaries reconciliation |

**Verdict (pre-registered space): NO GEOMETRIC ADVANTAGE.** A2(a) conjunction FAIL 5/12 (geometry holds correctness 1.0000 but does 12–37% more work than the goal-distance beam wherever reduction is structurally possible); A2(b) conjunction FAIL 0/9 primary, 0/9 secondary (bars: goal-distance / Hamming-popcount at 1.0000 vs geometry 0.0742–0.6426). Geometry indistinguishable from its own randomized/relabelled/phase-swapped controls — the gain over breadth-first attributes to informed ordering generally, not quadrangle structure. A successful falsification, per this issue's own claim status.

**Entry-gate terms honored (2026-08-22 maintainer decision):** F0 (#859) was never started; the W(3,3) object was pinned as formal-vocabulary Definitions in the design contract; the 96-vertex canvas was reconciled by disambiguation (a rendering motif, non-normative here); no lowering is proposed, so the Lean-pin obligation is not triggered. The RESTRICTED condition was honored: every frozen-terms comparison stayed on the 12 separating cells, and the 8 greedy-solvable cells were excluded from every axis.

**Scope discipline:** no production format, runtime, or dependency change anywhere in this issue; CONFORMANCE.md unchanged (33 ids); no new RF id; the f64 router untouched. Re-entry condition recorded in the measurement record §7: a new mapping Definition (not mod-9 digit quantization) with a pre-registered alignment rationale, a fresh design contract and probe, under the same frozen controls.

**Reproduction:** deterministic, teacher-free, fixture-free — `cargo test -p uor-r4-graph-certify --release --test w33_measurement_845 -- --ignored --nocapture` (~50 s).



---
_Comment/review ids in this file: 5381723010, 5382099987, 5382592692_

