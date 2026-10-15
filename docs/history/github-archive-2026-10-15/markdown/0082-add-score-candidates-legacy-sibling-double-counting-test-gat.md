# #82 — Add score_candidates_legacy, sibling double-counting test, Gate C legacy column, and canonical bits/token definition

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-23T00:12:38Z
- **Updated:** 2026-10-09T14:17:23Z
- **Closed:** 2026-07-23T03:33:44Z
- **Assignees:** auser, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/82
- **Merged:** not merged
- **Base / Head:** main / UOR-Foundation:copilot/uor-foundationuor-r4-64-chain-telescoped
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Gate C measured the Σ-over-cloud formula at 0.3% top-1 agreement vs 31.7% TLA3 baseline (issue #64). The chain-telescoped redesign and EXCT precedence are already in `score_candidates`, but `score_candidates_legacy` — referenced in module docs as the comparison baseline — was never implemented, and no test proved the stacking was actually fixed.

## `score_candidates_legacy` (`score_runtime.rs`)

Implements the original formula for Gate C side-by-side comparison only:
```
S(v) = B(v) + Σ_{n∈A} ΔE(n,v) + transition_offset + ΔX(X,v)
```
All active regions contribute ΔE (no chain restriction) — the double-counting path.

## Sibling double-counting test (`tests/score.rs`)

`legacy_formula_double_counts_sibling_regions_new_formula_does_not`: two sibling depth-1 regions, both active for the same context, both emitting ΔE=60 on token 10 against B(token 20)=100:

| Formula | S(token 10) | S(token 20) | Winner |
|---|---|---|---|
| Legacy Σ-cloud | 10+60+60 = **130** | 100 | token 10 ❌ |
| Chain-telescoped | 10+60 = **70** | 100 | token 20 ✓ |

## Gate C legacy column (`score.rs`, `command.rs`)

`GateCOutcome` gains `graph_legacy: GateCMetrics`; `evaluate_gate_c` runs the legacy scorer alongside the chain-telescoped and EXCT scorers. The CLI now emits a 4-row table: `graph (no EXCT)` / `graph (with EXCT)` / `graph (legacy Σ-cloud)` / `TLA3 store baseline`.

## Canonical bits/token definition (`BASELINE.md §4.1`, Chain 4 item 13)

Disambiguates the two figures in the baseline: Gate C scorer-accounting (11.88 TLA3 / 70.47 v1 graph) is the normative M.V.G. gating metric; Witten-Bell (6.54 P2) is for certificate continuity and is not the gate.

## Risk register update (`r4_graph_compiler_implementation_plan.md §10`)

Double-counting entry updated from CONFIRMED to RESOLVED, pointing at the implementation and the new test.

<!-- START COPILOT CODING AGENT SUFFIX -->

- Fixes #65

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5054005855 — 2026-07-23T03:33:43Z — auser

Superseded by 94501bd (already in main): score_candidates_legacy, the sibling non-stacking test with legacy contrast, the 4-way Gate C table (old formula column reproduced at 0.3%/76.96), and the canonical bits/token definition (1615127). Everything here duplicates merged work.


---
_Comment/review ids in this file: 5054005855_

