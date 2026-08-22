# Bounded semantic transitions — measurement record and verdict (#843)

- **Issue:** #843 — "compiler/runtime: learn and execute bounded semantic transitions with
  replayable plan witnesses" (item B of S4 tracker #826, programme #820).
- **Date:** 2026-08-22. **Records are append-only.**
- **Frozen contract:** [`docs/bounded_semantic_transitions_spec_843.md`](bounded_semantic_transitions_spec_843.md),
  measured against the benchmark constitution of
  [`docs/compositional_planning_spec_844.md`](compositional_planning_spec_844.md) including its
  appended §11 Amendment A1.
- **Claim language:** normative per [`docs/formal_vocabulary.md`](formal_vocabulary.md).
- **Execution scope:** the measured planner is **normative deployed-serving**; the harness that
  measures it is **certifier-instrument / off-serving-path**.

---

## 1. Verdict — LIMITED

**Empirical Criterion (S4 item B outcome). Status: Empirical.**

> A bounded, fixed-capacity, allocation-free, P-4-only semantic-transition planner is **established
> on the deployed path**, and is **lowered as the non-geometric production planning baseline** for
> the cells where a baseline exists to beat. It clears the frozen effect floor over the strongest
> non-oracle baseline on **12 of the 20** joint-split cells. It is **not established** on the
> remaining 8, whose tasks are solvable by greedy continuation, so no bounded planner can show
> headroom there. Typed planning is therefore **LIMITED**, not promoted and not refuted.

**What that does and does not license.** It licenses the exact phrase "bounded typed
state-transition planning on the measured tasks and horizons" and nothing wider. It is **not** a
general reasoning claim, **not** free-running coherence (the #824 boundary stands), and **not** a
calibrated-confidence claim (the #823 boundary stands). The S4 promotion verdict remains #846's to
make, against the sealed partitions this issue never opened.

**Lowered arm (maintainer sign-off, 2026-08-22): `bounded-breadth-first`.** It ties exactly with
`table-guided-beam` — both score 1.0000 in all 20 cells with identical lower bounds in every cell —
and the tie is broken toward breadth-first because it uses no scoring heuristic at all, so it is the
plainest possible baseline for #845's geometry to have to beat. The tie is recorded here rather than
implied, and the beam remains an equal-scoring alternative.

**#845 trigger — RELEASED, RESTRICTED.** Geometry qualification may begin, measured **only on the
12 separating cells**. Running it on the 8 greedy-solvable cells would compare geometry against a
baseline already at 1.0000, which cannot separate anything and would read as a geometry failure when
it is a task property.

---

## 2. What was measured

Frozen #844 §2.5 values, unchanged: primary statistic **held-out valid-plan rate**, n = **512**
instances per held-out cell per horizon, H ∈ **{1, 2, 4, 8}**, **δ_min = 0.05**. Per §11.6 the
horizon-1 cell is read as **correct-outcome rate** — a valid plan on a solvable instance, or a
correct `Decline(no_plan)` on one with no plan inside the horizon; on every H ≥ 2 cell, where all
instances are solvable, that *is* the frozen valid-plan rate.

**Definition (the split, and why it is the joint one).** #844 §2.2 requires that fitting and
evaluation data "never share a cell on **any** axis". The gate is therefore measured on the **joint
split**: a held-out instance is in the high half of *every* seed-varied axis — entity, vocabulary,
topology and template — so it shares no cell with anything fitted. Per-axis *isolating* splits are
also reported (§5) as a diagnostic; they are not the gate, and §5 records why.

Arms: `bounded-breadth-first`, `bounded-iterative-deepening`, `table-guided-beam`. Nulls:
`retrieval-only`, `direct-continuation`, `memorized-trajectory` (structure-keyed, with a modal
fallback so it can fire off-key), `shuffled-state`. The promotion statistic is taken against the
**strongest non-oracle** null in each cell, cell by cell.

---

## 3. Result — the joint split (the gate)

Every arm and null under one `PlanBudget`. Arm rate is the correct-outcome rate; `lb` is the
one-sided 95% lower confidence bound on the paired difference against the strongest null.

| H | family | arm | strongest null | rate | lb | |
|---|---|---|---|---|---|---|
| 1 | graph-navigation | 1.0000 | shuffled-state | 0.8047 | **+0.1665** | PASS |
| 1 | symbolic-transformation | 1.0000 | retrieval-only | 0.9258 | **+0.0552** | PASS |
| 1 | constraint-satisfaction | 1.0000 | shuffled-state | 0.7910 | **+0.1794** | PASS |
| 1 | multi-hop-evidence | 1.0000 | shuffled-state | 0.8047 | **+0.1665** | PASS |
| 1 | counterfactual-intervention | 1.0000 | direct-continuation | 0.7500 | **+0.2185** | PASS |
| 2 | graph-navigation | 1.0000 | direct-continuation | 0.9219 | **+0.0586** | PASS |
| 2 | symbolic-transformation | 1.0000 | direct-continuation | 1.0000 | +0.0000 | fail |
| 2 | constraint-satisfaction | 1.0000 | direct-continuation | 0.9844 | +0.0066 | fail |
| 2 | multi-hop-evidence | 1.0000 | direct-continuation | 0.9219 | **+0.0586** | PASS |
| 2 | counterfactual-intervention | 1.0000 | direct-continuation | 1.0000 | +0.0000 | fail |
| 4 | graph-navigation | 1.0000 | direct-continuation | 0.8574 | **+0.1172** | PASS |
| 4 | symbolic-transformation | 1.0000 | direct-continuation | 1.0000 | +0.0000 | fail |
| 4 | constraint-satisfaction | 1.0000 | direct-continuation | 0.9434 | +0.0398 | fail |
| 4 | multi-hop-evidence | 1.0000 | direct-continuation | 0.8574 | **+0.1172** | PASS |
| 4 | counterfactual-intervention | 1.0000 | direct-continuation | 1.0000 | +0.0000 | fail |
| 8 | graph-navigation | 1.0000 | direct-continuation | 0.8340 | **+0.1390** | PASS |
| 8 | symbolic-transformation | 1.0000 | direct-continuation | 1.0000 | +0.0000 | fail |
| 8 | constraint-satisfaction | 1.0000 | direct-continuation | 0.7422 | **+0.2260** | PASS |
| 8 | multi-hop-evidence | 1.0000 | direct-continuation | 0.8340 | **+0.1390** | PASS |
| 8 | counterfactual-intervention | 1.0000 | direct-continuation | 1.0000 | +0.0000 | fail |

**The planner is at 1.0000 in every one of the twenty cells.** It emits a plan the frozen #844
verifier accepts on every solvable held-out instance, and declines correctly on every unsolvable
one — including on held-out topologies whose operator effect sets it never saw during fitting. Every
failure in the table is a cell where the *null* is also at or near 1.0000, never one where the arm
fell short.

`table-guided-beam` is identical in all twenty cells. `bounded-iterative-deepening` is **not**: mean
0.7536, falling to **0.2695** at H = 8, because under the equal budget it re-expands and exhausts
`max_expansions`. It clears the floor in 5 cells and is *beaten by the nulls* in several, with a
lower bound down to −0.7117. That is the arm comparison doing its job.

---

## 4. Why the 8 cells fail — the diagnosis

**Definition (greedy-solvable tasks).** In every failing cell the strongest null is
`direct-continuation`: a greedy one-step descent on goal distance, with no lookahead and no
backtracking. It reaches **1.0000** on symbolic-transformation at every horizon and on
counterfactual-intervention at H ≥ 2, and 0.9434–0.9844 on constraint-satisfaction at H = 2 and 4.

The reason is a property of those task families, not of the planner: their state spaces are
**monotone toward the goal**. Symbolic transformation applies operators that each reduce the
distance to the target term, and counterfactual intervention (after the A1 redesign) places the goal
on a pure-east axis, so a greedy step is never a trap. Where a family *does* trap greedy — a wall
with a gap that must be threaded (constraint-satisfaction at H = 8, 0.7422) or an obstacle that must
be walked around (graph-navigation and multi-hop-evidence, 0.8340) — the planner beats it decisively,
by up to **+0.2260**.

**Assumption (recorded).** A bounded planner cannot demonstrate value over greedy continuation on a
task greedy already solves. This is a ceiling on the *benchmark*, and it was not visible before this
measurement: Amendment A1 established that the cells were non-vacuous and that the strongest
*memorization* null left headroom, but headroom against a memorizer is not headroom against a
*search* null. **Any future benchmark-freeze item in this programme adds a third instrument beside
A1's non-vacuity and null-saturation checks: a greedy-solvability probe**, because a task solvable
without lookahead cannot measure lookahead.

---

## 5. Diagnostic — the per-axis isolating splits

Reported because §2.5 asks for a per-axis reading, and recorded as a diagnostic rather than as the
gate. An isolating split varies one axis and lets the rest range freely, so fitting and held-out data
*do* share cells on the untested axes. Cells clearing δ_min, `bounded-breadth-first`:

| axis | cells clearing | mean strongest-null rate |
|---|---|---|
| `by_topology` (semantic) | **10 / 20** | 0.8991 |
| `by_entity` (surface) | 3 / 20 | 0.9764 |
| `by_vocabulary` (surface) | 2 / 20 | 0.9896 |
| `by_template` (surface) | 0 / 20 | 0.9916 |

**Definition (why a surface axis cannot separate a semantic mechanism).** On an isolating surface
split the held-out instances differ from fitting ones *only* in naming. A structure-keyed
memorized-trajectory null keys on goal, forbidden set and operator effects — all invariant under
renaming — so it transfers perfectly and scores 0.98–1.00. **Nothing can beat it there, and that is
the correct reading rather than a failure**: a surface axis introduces no semantic novelty for
anything to generalize over. What a surface axis tests is *invariance* — that the arm does not
degrade — and the arm is at 1.0000 on all of them. Superiority is only a meaningful demand on a
semantic axis, and on the semantic one the arm clears the floor in half the cells and is at 1.0000
in all of them.

This is the same shape of structural limit as the horizon-1 cell (#844 §11.6): a comparison is only
informative where the comparison is possible.

---

## 6. The statistic, and a correction to how it is read

**Definition (intersection-union, the reading the gate needs).** The #826 promotion gate is a
*conjunction*: the bound must clear δ_min on every required cell. For a conjunction the
intersection-union principle applies and each cell is tested at the full level α; the family-wise
error is already controlled because a single failing cell sinks the claim on its own. The 12/20
figure above is this reading.

**Definition (Holm–Bonferroni, as the constitution freezes it).** Holm controls false *rejections*
across many claims, which is the correct adjustment for a *disjunction* ("some cell shows an
effect") and the wrong one for the conjunction the gate actually states. It is reported alongside
rather than instead: under Holm, `bounded-breadth-first` rejects the null in 12 of 20 joint-split
cells — the same 12, because in this measurement the separating cells separate by margins far above
the adjusted level and the failing cells have a zero point estimate.

**Correction recorded.** A first implementation of this harness inflated the required margin in the
*wrong direction*, making the strongest-evidence cell face the strictest threshold, and reported
0/80 on a grid containing bounds up to +0.2523. The arithmetic is now in p-value space with the
step-down in the standard direction, and a built test asserts the bound is conservative, signed, and
does not launder a loss.

---

## 7. Verification behind the numbers

**Guarantee (reference, packed and production agree; all three fail closed). Status: Structural.**
A three-way differential over **480 fixtures** — five families × four frozen horizons × 24 held-out
seeds — asserts the reference planner (owned f32 semantic states), the offline packed planner
(reading the sections with owned collections) and the deployed planner (the P-4 runtime on the
`R4Engine` path) agree on whether a plan exists, on the emitted plan, and on its length, and that
whatever each produced is accepted by the frozen #844 verifier. A rule table declaring a different
operator count is refused by the packed *and* deployed readings alike; the reference path does not
read the artifact at all, which is precisely why it is the independent third opinion.

**Guarantee (control non-degeneracy). Status: Structural.** Every null is asserted able to fire *and*
able to fail. The shuffled-state control is asserted to collapse relative to the arm — measured
**128/128 real versus 4/128 shuffled** — so a mechanism that survived a broken operator/effect
correspondence would be caught rather than credited.

**Guarantee (budget parity). Status: Structural.** Every arm and null runs under one `PlanBudget`;
an arm whose recorded counters exceed the shared ceiling is reported invalid rather than compared.

Deployed-path evidence carried forward from increment 5: **0 allocations and 0 bytes** for a whole
episode and for emitting its witness; a source scan finding **no runtime multiply, divide or float**
with six compile-time constant expressions exempted and printed; a checked visited-probe bound of 16;
`size_of::<PlanScratch>()` = 81 548 bytes.

---

## 8. Run contract — outcome against what was posted

    metric to move:       held-out valid-plan rate; deployed value was 0.000 (no deployed planner
                          existed). OUTCOME: 1.0000 in all 20 joint-split cells.
    reachability ceiling: corpus-observation arm did not launch (0 of 10 typed object kinds in the
                          v4 record, coverage 0.000). Benchmark arm, post-A1: the ceiling is
                          (1 - strongest non-oracle null). OUTCOME: that ceiling is BELOW
                          delta_min in 8 of 20 cells, because greedy continuation is at or near
                          1.0000 there - a ceiling the A1 gate did not measure and this run did.
    instrument + verdict: the four A1 gate instruments all PASSED before any fitting.
    exit rule:            lower an arm only if its one-sided 95% lower bound of
                          (arm minus strongest non-oracle null) exceeds delta_min on every
                          required axis and horizon. OUTCOME: cleared on 12 of 20; NOT cleared on
                          every cell, so the literal gate is not met and the verdict is LIMITED
                          rather than PROMOTE.
    if positive:          freeze the arm as the non-geometric baseline, record the trigger that
                          releases #845, hand the sealed partitions to #846. APPLIED, restricted
                          to the 12 separating cells.
    if negative:          publish the limiting diagnosis and keep typed planning NOT ESTABLISHED.
                          APPLIED to the 8 greedy-solvable cells.
    cost estimate:        posted as "minutes, not hours, on one core, no fixture and no network".
                          ACTUAL: the full 300-cell grid - 3 arms x 5 splits x 4 horizons x
                          5 families at n=512 - ran in 44.7 s wall-clock in release on one core,
                          teacher-free and fixture-free. No long-run publication was required.

Positive and negative branches led to different next actions, so the measurement had decision value.

---

## 9. Conformance

**RF-33 `bounded_semantic_transitions`** is registered — `normative-runtime` scope,
`deployed-serving` reachability — because an arm was in fact lowered. Its statement carries the
LIMITED boundary explicitly, so the registry cannot be read as claiming more than the 12 cells
support. `CONFORMANCE.md` moves from 32 to **33 ids** and is regenerated by
`xtask check-model --write`, never hand-edited. Built-capability order was followed:
`model/ids.toml` → tagged Gherkin (`features/suites/bounded_semantic_transitions.feature`, six
`@RF-33 @build` scenarios) → marker in `crates/repo-conformance/tests/registered.rs` plus executable
steps in the root `tests/bdd.rs` → implementation → regenerated conformance.

Existing RF-01/08/12/13/27/28/32 remain reference or certifier-instrument evidence and are still not
deployed-planning evidence.

---

## 10. What this hands downstream

- **#845 (geometry qualification):** released with a **restricted** trigger — measure geometry only
  on the 12 separating cells, against `bounded-breadth-first` at 1.0000, under equal bytes,
  candidates, expansions and operations. Geometry that cannot beat it there stays offline.
- **#846 (certification):** the witness schema, the three-way differential, the planted-mutation
  table and the sealed composition and topology cells, which this issue never opened.
- **A benchmark note for whoever revisits S4:** two of the five families are greedy-solvable and
  therefore cannot measure planning. Repairing that is a benchmark change, and — unlike Amendment
  A1, which was made *before* seeing which arm failed — it would be an amendment made after, so its
  difficulty criterion must be declared before any re-run.
