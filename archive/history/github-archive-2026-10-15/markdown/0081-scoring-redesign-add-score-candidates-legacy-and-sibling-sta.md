# #81 — scoring redesign: add `score_candidates_legacy` and sibling-stacking test (issue #64)

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-23T00:09:17Z
- **Updated:** 2026-10-09T14:17:23Z
- **Closed:** 2026-07-23T02:04:12Z
- **Assignees:** auser, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/81
- **Merged:** not merged
- **Base / Head:** main / UOR-Foundation:copilot/scoring-redesign-correlated-residual-stacking
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The Σ-over-cloud formula `S(v) = B(v) + ΣΔE + ΣΔT + ΔX` double-counts correlated evidence when multiple sibling regions are simultaneously active — one token received +91.7 nats from 13 stacked sibling-subtree emission lists, producing 0.3% top-1 vs 31.7% TLA3 baseline at Gate C.

The chain-telescoped redesign (Rule 1: apply ΔE only along one selected refinement chain; Rule 2: EXCT precedence gate) was already implemented in `score_candidates`. Two pieces were missing:

## `score_candidates_legacy` (`score_runtime.rs`)

Implements the original Σ-over-cloud formula — documented in the module header as `GraphScorer::score_candidates_legacy` but never added. Sums emission residuals from **all** active regions without chain selection; EXCT is additive without D4 precedence. Retained for Gate C side-by-side comparison only; `score_candidates` is the shipping path.

## Sibling-stacking test (`tests/score.rs`)

`sibling_stacking_is_prevented_by_chain_selection`: hand-computed artifact with two sibling depth-2 regions (same parent) both within range. Asserts:

- **New scorer** selects one chain → `S(99) = B(99) + ΔE(A,99) = 10 + 1000 = 1010`
- **Legacy scorer** stacks both siblings → `S(99) = 10 + 1000 + 2000 = 3010`

```rust
// Only the selected chain's residual applies
assert_eq!(outcome.selected_score, ScoreQ::from_raw(1010));
assert_eq!(emission_count, 1);  // no double-count

// Legacy stacks both siblings
assert_eq!(legacy_99.1, ScoreQ::from_raw(3010));
assert!(legacy_99.1 > token_99.1);
```

<!-- START COPILOT CODING AGENT SUFFIX -->

- Fixes #64

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5053513911 — 2026-07-23T02:04:12Z — auser

Both pieces were completed as part of the #64 finish (commit 94501bd, verified by the maintainer agent):

1. score_candidates_legacy: exists at crates/uor-r4-core/src/transformerless/score_runtime.rs:1128 — the faithful old Σ-over-cloud accumulation, retained for comparison only. It is what produces the 'graph Σ-cloud (old)' column (0.3%/76.96, reproducing the incident exactly) in the 4-way Gate C table; the shipping path is score_candidates (Rule 1/2).

2. Sibling-stacking test: tests/score.rs:549 rule1_chain_telescopes_and_sibling_emissions_do_not_stack — the fixture has sibling depth-2 regions sharing a refinement parent with correlated emission lists; it asserts the new scorer applies exactly one chain residual while the legacy scorer stacks both (the contrast case), plus the self-transition Theorem-10 duplicate-contribution rejection. Passing in CI (45 suites green).

The full redesign verdict is recorded on #64 (closed): Rule 1+2 is argmax-identical to the baseline on all 30,036 held-out positions with better bits/token (9.86 < 11.88).


---
_Comment/review ids in this file: 5053513911_

