# #495 — feat(#488): phase timing across the whole score pipeline

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-08T03:06:48Z
- **Updated:** 2026-10-09T13:08:50Z
- **Closed:** 2026-08-08T03:25:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/495
- **Merged:** 2026-08-08T03:25:12Z
- **Merge commit:** b1a46412de7ffbac18ad4c6625bd3619001df4df
- **Base / Head:** main / UOR-Foundation:feat/score-phase-timing-488
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Refs #488 (**kept open** — see below). Generalizes the #471 Gate C instrument to the whole `score` pipeline, where a real corpus's hours actually live (#483 showed Gate C is not).

## What changed
`PhaseLog` (`crates/uor-r4-graph-certify/src/score.rs`) is now `pub` and carries a `prefix`, so one instrument reads as one log at two levels. `evaluate_gate_c` runs it as `"gate c phase"`; `graph-cli::score_command` runs it as `"score phase"` with a `mark()` after every stage plus `total()`. stderr only — a duration in `score_report.json` would break the deterministic-rebuild gate (#471 convention).

## Verified on the 500k fixture — phases sum to the total with ZERO remainder
```
score phase: inputs + observations           1.56s
score phase: cover induction                27.88s
score phase: graded store build (+sidecar)   1.78s
score phase: forward transitions             0.95s
score phase: context + forward-anchor rows   0.31s
score phase: emission compilation            0.41s
score phase: R4G1 artifact emission          0.71s
score phase: gate c evaluation             326.01s   (gate c phase: ... nested)
score phase: report build + artifact write   0.08s
score phase: TOTAL                         359.69s
```
Sum of stages = 359.69s = TOTAL, so no stage is unnamed — the remainder is the DoD's own completeness check. It immediately attributes the run: Gate C's **scoring pass dominates (247s / 100,306 positions)**, and within the pre-Gate-C stages **cover induction is largest (27.88s)**; the store build is 1.78s only because the #469 code sidecar HIT (vs the ~625s cold path).

## Why #488 stays open
The 500k run verifies the *instrument*. Attributing a real **2.11M** run's hours — the actual point of #488 — still needs a corpus at that scale, which is gone from disk and non-reproducible (`fetch_d3_corpus.py` warns of upstream drift). Sourcing one remains the open work.

fmt clean; clippy `--all-targets --all-features -D warnings` clean.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

