# #925 — feat(#843): equal-budget measurement, RF-33, and the LIMITED verdict

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-22T17:15:14Z
- **Updated:** 2026-08-22T17:39:28Z
- **Closed:** 2026-08-22T17:37:31Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/925
- **Merged:** 2026-08-22T17:37:31Z
- **Merge commit:** f1938377604f9a563f8b329fc7a8c978613b1545
- **Base / Head:** main / UOR-Foundation:issue-843-measurement-verdict
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Increment 6 of 6, the final one for issue #843 (item B of S4 tracker #826). Runs the frozen §10 measurement, registers the built capability, and records the verdict.

### Verdict: LIMITED

A bounded, fixed-capacity, allocation-free, P-4-only semantic-transition planner **is established on the deployed path** and **is lowered as the non-geometric production baseline** — for the **12 of 20** joint-split cells where a baseline exists to beat. Typed planning is **not established** on the other 8. Not promoted, not refuted.

**The planner scores correct-outcome rate 1.0000 in all 20 cells** — a verifier-accepted plan on every solvable held-out instance and a correct decline on every unsolvable one, including on held-out topologies whose operator effect sets it never saw during fitting. **Not one failing cell is a cell where the arm fell short**; all 8 are cells where the *null* is also at or near 1.0000.

## Scope and non-goals

Measurement harness (`crates/uor-r4-graph-certify/tests/compositional_planning_measurement_843.rs`), the RF-33 registration chain, the results record, and the readable-mirror entry. **Non-goals:** no geometry (#845), no untouched-partition verdict (#846) — the sealed composition and topology cells were never opened.

## Acceptance-criteria status

| #843 criterion | Status |
|---|---|
| Compiler outputs deterministic, no evaluation leakage | **Met** (increment 3; byte-identical across order and shard counts 1/3/16/97, leakage scan refuses a held-out cell by name) |
| Packed planner bit-equivalent to reference on valid fixtures, fails closed on corrupt data | **Met** (three-way differential over 480 fixtures; all three fail closed on an incompatible table) |
| Witness replay independently validates each transition and terminal goal | **Met** (increments 4–5; a right answer through a forbidden step replays `Invalid`) |
| Production path meets horizon/frontier, bytes, latency, P-4, allocation, no_std, determinism budgets | **Met** (increment 5; 0 allocations, no runtime multiply/divide/float, checked probe bound) |
| Interventional performance beats the baselines under the frozen rule, **or the negative verdict is recorded** | **Recorded as LIMITED** — cleared on 12 of 20 cells, with the diagnosis for the other 8 |

## Evidence artifacts

Full per-cell tables: `docs/bounded_semantic_transitions_843.md`. Joint split (held-out shares no cell on **any** axis), n = 512 per cell, δ_min = 0.05:

| arm | cells at 1.0000 | cells clearing δ_min |
|---|---|---|
| `bounded-breadth-first` | 20 / 20 | **12 / 20** |
| `table-guided-beam` | 20 / 20 | 12 / 20 (identical bounds in every cell) |
| `bounded-iterative-deepening` | 4 / 20 | 5 / 20 |

**Arm lowered: `bounded-breadth-first`** (maintainer sign-off). It ties exactly with the beam, and the tie breaks toward breadth-first because it uses no scoring heuristic — the plainest possible bar for #845's geometry to clear. **`bounded-iterative-deepening` is rejected**: under the equal budget it re-expands and exhausts its expansion ceiling, falling to 0.2695 at H = 8 and *losing* to the nulls in several cells, down to −0.7117. That is the arm comparison doing its job.

## Negative and unavailable results — the finding that matters

**Every failing cell's strongest null is `direct-continuation`: greedy one-step descent on goal distance, no lookahead, no backtracking.** Symbolic transformation and counterfactual intervention are monotone toward the goal, so greedy is never trapped and reaches **1.0000**; constraint satisfaction is at 0.9844 / 0.9434 at H = 2 / 4. A bounded planner cannot show headroom over greedy on a task greedy already solves. Where a family *does* trap greedy — a wall with a gap, an obstacle to walk around — the arm wins by up to **+0.2260**.

**This is a ceiling on the benchmark, not on the mechanism, and Amendment A1 could not have caught it.** A1 established that the cells were non-vacuous and that the strongest *memorization* null left headroom. Headroom against a memorizer is not headroom against a *search* null. A **greedy-solvability probe** is therefore added as a standing requirement for any future benchmark freeze in this programme, beside A1's two instruments: a task solvable without lookahead cannot measure lookahead, and no amount of split discipline will change that.

## Two methodology corrections, recorded rather than quietly applied

**The split.** The gate is measured on the **joint** split, because #844 §2.2 requires that fitting and evaluation data never share a cell on *any* axis. Per-axis *isolating* splits let them share cells on the untested axes and are reported as a diagnostic only. On an isolating **surface** split nothing can beat a structure-keyed memorizer (0.98–1.00) — and that is the *correct* reading rather than a failure: a surface axis introduces no semantic novelty, so it tests **invariance**, not superiority. The arm is at 1.0000 on all of them.

**The statistic.** The #826 gate is a **conjunction**, so the intersection-union principle applies and each cell is tested at the full level; Holm–Bonferroni, which the constitution freezes, is the correct adjustment for a **disjunction** and is reported alongside rather than instead. Both give the same 12 cells. A first implementation of this harness inflated the required margin in the *wrong direction* and reported 0/80 on a grid holding bounds up to +0.2523; the arithmetic is now in p-value space with the step-down in the standard direction, and a test asserts the bound is conservative, signed, and never launders a loss.

## Tests and verification

Six default tests plus the `#[ignore]`d full grid. The defaults are non-vacuity gates, because a harness that has rotted into always-passing is worse than none: the packing path is asserted real, **every null is asserted able to fire *and* able to fail**, the shuffled-state control is asserted to collapse (**128/128 real → 4/128 shuffled**), and the lower-bound arithmetic is asserted conservative and signed.

Full local ladder: fmt, clippy `-D warnings`, `cargo test --workspace --no-fail-fast` (184 suites), bdd (33 features, 120 scenarios, 395 steps), the `no_std` ladder both ways, the wasm lib build, `xtask validate` (33 ids), `repo-conformance`, `cargo deny check bans`, and the claim-wording gate — with `audit-limits` re-run as the last step before push.

## Run contract — outcome against what was posted

The full posted-versus-actual table is in the record's §8. The headline: the reachability ceiling turned out to be **below δ_min in 8 of 20 cells**, because greedy continuation is at or near 1.0000 there — a ceiling the A1 gate did not measure and this run did. Cost was posted as "minutes, not hours"; the 300-cell grid ran in **44.7 s** in release on one core, teacher-free and fixture-free.

## Conformance effects

**RF-33 `bounded_semantic_transitions` registered** (`normative-runtime` scope, `deployed-serving` reachability), because an arm was in fact lowered. **Its statement carries the LIMITED boundary in-line**, so the registry cannot be read as claiming more than the 12 cells support. `CONFORMANCE.md` 32 → **33 ids**, regenerated by `xtask check-model --write`, never hand-edited. Built-capability order followed: `model/ids.toml` → tagged Gherkin (six `@RF-33 @build` scenarios) → marker in `registered.rs` plus executable steps in the root `tests/bdd.rs` → implementation → regenerated conformance.

## Claim-boundary changes

One claim is added and bounded in the same breath: bounded typed state-transition planning on the measured tasks and horizons, on 12 of 20 cells. It is **not** a general reasoning claim, **not** free-running coherence (#824 stands), and **not** a calibrated-confidence claim (#823 stands). Public phrasing stays inside #844 §4's term discipline.

## Compatibility and migration

Additive: one new test file, one new feature file, one new id, one new marker, and documentation. No format, artifact, API or wire change.

## Intentionally excluded

The sealed composition and topology cells reserved for #846 were never opened. No generated file, model, cache, credential or user-owned file is staged; `CONFORMANCE.md` is regenerated, not hand-edited. The maintainer's day-branch checkout is untouched.

Closes #843


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

