# #2180 — Trainer: softsort_rank, a rank-consistent training read for the learned rank table (preparation for arm S) (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T21:32:01Z
- **Updated:** 2026-10-10T21:42:25Z
- **Closed:** 2026-10-10T21:42:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2180
- **Merged:** 2026-10-10T21:42:12Z
- **Merge commit:** f07d2ff01d9ff24c477edb56b269e50b62c66ba2
- **Base / Head:** main / UOR-Foundation:claude/softsort-rank
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: wide learned flock read · count 2/3 · headline softmax at runtime yes → yes

**The one preparation PR** for arm S ([pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6102148887), [amendment](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6102170500)). Decision test: without a rank-consistent gradient, a REJECT cannot separate "rank reads lose information" from "the surrogate gradient misled training". The test-fitness review named exactly this.

## What and why

- **`ReadWeighting::SoftsortRank`:** `learned_rank`'s parameters, table and served form, trained through **SoftSort**.
  - Over the flock-kept sources plus NoRead, the soft permutation `P[r,i] = softmax_i(−|sort(s)_r − s_i| / τ)` maps scores to ranks, and the weights are `Wᵀ·P` with the learned per-head table.
  - Written as tensor ops (`softsort_learned_read`): exact autograd gradients to q, k, NoRead, age, scale/offset, the values and the rank logits.
  - Runs on CPU and CUDA alike.
- **Limit and serving:** τ → 0 is exactly the hard learned rank read, which is what export writes and D11 serves. The export treats `softsort_rank` as `learned_rank`.
- **When τ applies:** `StackModel::set_softsort_temperature` is a runtime setting, never saved. The `train` loop sets τ only around each optimizer step and clears it after, so evaluations, samples, saves and export read the hard form.
- **CLI:** `read_weighting=softsort_rank softsort_tau_start=1.0 softsort_tau_end=0.01` (geometric schedule), recorded in the report and lineage. Data-parallel replicas follow.

## Checks at this head

- `cargo test -p uor-r4-training --lib read_weighting`: **13 passed**, including the three new `read_weighting_softsort_*` tests:
  - τ = 1e-5 soft equals hard to < 1e-3 for L2, Dot and Lorentz; a warm τ differs; cleared τ equals `learned_rank` bit for bit;
  - a two-head function-level finite-difference check (> 10 query entries, the second head's logits) within 5 %;
  - save/load keeps `softsort_rank`, drops τ and refuses a bad τ.
- **Negative gate:** detaching the items inside the sort makes the gradient test fail; restored, it passes.
- `--lib fused_read`: 2 passed. `--test stack_softmax_free_export`: 4 passed. `cargo check -p uor-r4-training --examples`: ok.
- **CPU smoke:** `train … select=flock:4:4 read_weighting=softsort_rank`, 3 steps, exits 0. The report records `softsort_tau {start 1.0, end 0.01}`; the saved config says `softsort_rank`.
- `rustfmt --check` on the changed files and `check_claim_wording.py`: pass.

## Result
No model result: arm S runs after `TEST FITNESS: FIT`. Decision: NOT YET PROMOTED (preparation).

References #2032

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6102395074 — 2026-10-10T21:32:03Z — Casey-allard

Exact-head self-review at `cdee4985f251ce9b056ffa2ff738bd1694cecdd4`: the checks in the description ran at this head. Function-level gradient check plus negative gate; evaluation stays hard because the train loop sets τ only around optimizer steps (cleared τ is bit-identical to `learned_rank`, test 1). No independent agent review this round: subagents start above their context budget until the owner's plugin cleanup, so the review falls back to the documented exact-head self-review.

### comment 6102467043 — 2026-10-10T21:40:54Z — Casey-allard

Exact-head self-review at `bf7092612395426f7f2b5c004400fec5a7b421bc`: merged `origin/main` (only conflict: `docs/integration/current-state.md`, both entries kept, softsort entry newest-first). At this head: `cargo check -p uor-r4-training --tests --examples` clean (no errors), `cargo test -p uor-r4-training --lib read_weighting_softsort` 3 passed, `rustfmt --check` and `check_claim_wording.py` pass. APPROVE.


---
_Comment/review ids in this file: 6102395074, 6102467043_

