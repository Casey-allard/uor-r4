# #921 — feat(#843): Amendment A1 — repair the S4 benchmark generator so it can separate

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-22T14:39:03Z
- **Updated:** 2026-08-22T17:39:23Z
- **Closed:** 2026-08-22T15:01:05Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/921
- **Merged:** 2026-08-22T15:01:05Z
- **Merge commit:** 328791ffdd5f2d28c46b5c3028d7578c63d329b1
- **Base / Head:** main / UOR-Foundation:issue-843-a1-benchmark-repair
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Increment 2 of 6 for issue #843 (item B of S4 tracker #826). Implements **Amendment A1**, frozen in `docs/compositional_planning_spec_844.md` §11 by the previous increment, and re-runs the binding cheap instrument that had returned DEGENERATE.

**All four A1 gate instruments now pass**, so the packed-section, planner, and measurement increments may proceed. Before the repair, the S4 promotion statistic was at or below zero *by construction* for every candidate mechanism, geometric or not.

## Scope and non-goals

**In scope:** the generator half of `crates/uor-r4-graph-compiler/src/compositional_planning.rs`, the four gate instruments, and the outcome record appended to the two specifications.

- **A1-a — low-horizon non-vacuity.** The goal is now placed on the BFS layer at *exactly* the requested depth, so the gold plan is exactly `H` steps long and a horizon-1 instance is a genuine one-step task. Non-vacuity is restored by making instances easier at low `H`, **not** by moving the frozen horizon grid.
- **A1-b — real split-axis cardinality.** Eight entity-naming schemes, eight operator surface vocabularies and eight goal templates (all *surface* axes — semantic no-ops, so splitting on them isolates label-keyed mechanisms), plus eight topologies. **Topology varies the operator effect set**, drawn from a shared eight-effect pool, and topologies 0–3 and 4–7 each cover the whole pool. That is deliberate: a held-out cell must be a novel *composition*, never a novel primitive, or an inducer fitted on the low half would find the high half unsolvable rather than hard.
- **A1-c — content-derived identity.** `TaskInstance::id()` no longer mixes in the generation seed, so structurally identical instances share an id and an identity-keyed control can fire.
- **A1-d — strongest-null headroom.** Measured below.
- **F5 is now a counterfactual at every horizon.** A *twin* operator carries one effect before the declared intervention and another after; the additive field `TaskInstance.counterfactual_base` records the pre-intervention optimal plan by operator name (resolved through the new `resolve_actions`), and the generator deterministically scans the layer for a goal whose base plan uses the twin and *fails* once the twin's declared effect changes. Previously the family had no way to express a counterfactual at H = 1 at all.

**Non-goals:** no observation record, no inducer, no packed section, no planner, no conformance id — those are increments 3–6. Nothing here is deployed-serving evidence; the module stays reference-only / off-serving-path.

## Acceptance-criteria status

None of #843's five acceptance criteria are claimed by this increment. It clears the **entry gate** that its own run contract places before any packed or full fitting, and it does so by measurement rather than by assertion.

## Tests and verification

New file `crates/uor-r4-graph-compiler/tests/amendment_a1_843.rs` — the four gate instruments plus a frozen-number guard, at the frozen n = 512 per cell per horizon, deterministic, teacher-free, fixture-free, about 40 s in the default `cargo test`.

Existing suite: the module's tests grew from 11 to 16 and all pass, including the pinned seed-0 fixtures. Topology cell 0 deliberately keeps the historical `(2, 0)` forbidden cell and vocabulary cell 0 the compass names, so `features/suites/compositional_planning_benchmarks.feature` (RF-32) needs no change and its five scenarios pass unaltered.

Full local ladder before push: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --no-fail-fast --offline`, the `no_std` ladder both ways, the `wasm32-unknown-unknown` lib build, `xtask validate` (R1/R4/R5/gnaf), `cargo test -p repo-conformance` (R2/R3), `cargo deny check bans` (R6), and `scripts/check_claim_wording.py`.

## Evidence artifacts

`docs/compositional_planning_spec_844.md` §11.5 carries the full per-cell tables. Headline, with the pre-repair figure alongside:

| Instrument | Before | After |
|---|---|---|
| A1-a non-vacuity | 13 of 20 cells at 0/512 | 1.0000 at H ∈ {2,4,8}; 0.7500 at H = 1 |
| A1-b axis cardinality | 5 of 6 axes had one cell | 8 per axis; 32–64 compositions |
| A1-c identity | seed-carrying; 4096/4096 unique | content-derived; structurally identical instances share an id |
| A1-d strongest null | 1.0000 in every non-vacuous cell | 0.2773 – 0.9355, against a 0.95 ceiling |

The controls are not strawmen: they are fitted on the honest declines as well as on the plans, they transfer plans by **effect** rather than by name or slot — so they already see through all three surface axes — and the retrieval control indexes goal displacement. The split is taken on the *semantic* topology axis, so a held-out cell never shares an operator effect set or a forbidden configuration with fitting data.

## Negative and unavailable results

**One result is negative and is recorded rather than engineered away.** At horizon 1, a retrieval baseline scored exactly 1.0000 in four of five families even after A1-a/b/c had passed. This is structural: a one-step answer is a deterministic function of the initial state, the goal and the operator set — all observable at inference — and the fitting split must cover the whole operator pool or induction has nothing to learn from. Valid-plan rate cannot separate planning from retrieval at horizon 1, and no generator repair changes that.

Per maintainer sign-off (2026-08-22), the horizon-1 cell is therefore gated on **correct-outcome rate** — a valid plan on a solvable instance, or a correct `Decline(no_plan)` on one with no plan inside the horizon — and a quarter of horizon-1 instances now place the goal one step beyond the horizon so that decision is real. Deciding which is which requires evaluating reachability, which is planning; a baseline that always answers cannot. On every H ≥ 2 cell, where all instances are solvable, the correct-outcome rate *is* the frozen valid-plan rate. Reasoning and the rejected alternative are in §11.6.

**A tight cell is flagged, not hidden.** Symbolic-transformation at H = 1 leaves headroom 0.0645 — only 1.29 × δ_min. The instrument prints a `TIGHT` marker for any cell under 2 × δ_min, and §11.5 names that cell as the one to watch when the arms are measured.

## Compatibility and migration

No format, artifact, API-response or wire change. `TaskInstance` gains one additive field (`counterfactual_base`) and the module gains one function (`resolve_actions`); both are additive to a reference-only, off-serving-path model that no deployed path reads. `CONFORMANCE.md` is untouched.

The generated *content* of the benchmark changes, by design — that is the whole point of the amendment. Any figure measured against the pre-amendment generator is superseded; the only such figures are the degeneracy measurements themselves, which §11.1 keeps as the record of what was believed and when, per the append-only rule.

## Conformance effects

None. No RF id is added, removed or altered, and `CONFORMANCE.md` is byte-identical. RF-32 keeps its Gherkin, its marker and all five scenarios unchanged. The reserved RF-33 is still unregistered and stays that way unless an arm is actually lowered in increment 5/6.

## Claim-boundary changes

No claim is strengthened; the module still establishes a falsifiable target and a byte-level meaning, not reasoning performance. Two boundaries move, both toward honesty: the item-A no-split-leakage guarantee stops being vacuous and becomes a partition of a genuine eight-cell axis, and the baseline-non-degeneracy guarantee gains a measured headroom figure instead of an untested assertion. The horizon-1 reading is explicitly narrowed to honest decline.

## Intentionally excluded

The two scratch probe harnesses used to find the degeneracy; their shipped successors are the four gate instruments in this diff. No generated file, model, cache, credential or user-owned file is staged. The maintainer's day-branch checkout is untouched; all work is in a dedicated worktree.

References #843


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

