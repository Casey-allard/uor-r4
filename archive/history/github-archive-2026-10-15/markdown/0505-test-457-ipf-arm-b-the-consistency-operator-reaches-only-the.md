# #505 — test(#457): IPF Arm B — the consistency operator reaches only the unigram floor

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-08T17:17:12Z
- **Updated:** 2026-10-09T13:08:50Z
- **Closed:** 2026-08-08T17:36:15Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/505
- **Merged:** 2026-08-08T17:36:15Z
- **Merge commit:** d91267dcc696dc3d1a371fc78e09ef3d1aa7eb6c
- **Base / Head:** main / UOR-Foundation:fix/457-ipf-arm-b
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Builds and runs the #457 **Arm B**: does an IPF-consistent reconstruction of the next-token distribution beat backoff+sum on held-out bits/top-1? This is the definitive test the #499 cheap-check deferred (that probe found the ~7.5 co-activated regions *disagree*, so a consistency operator was warranted, not moot).

## Result — NEGATIVE (5000 held-out, default cover, 500k fixture)

| arm | bits/token | top-1 |
|---|---:|---:|
| root null (unigram) | 8.5635 | 0.0620 |
| A — naive backoff+sum | 23.9196 | 0.0018 |
| **B — IPF-consistent** | **8.5995** | **0.0620** |

per-region marginal inconsistency: **A 5.3953 bits → B after IPF 0.0956**

Two decisive readings:

1. **IPF works, and recovers nothing.** It drives the per-region marginal inconsistency from 5.40 bits down to 0.096 — it genuinely reconciles the overlapping blocks into a consistent joint. But that joint lands **exactly at the unigram floor** (8.60 bits, 0.0620 top-1 = the null's 0.0620). The overlapping regions carry no reconcilable next-token signal. The consistency operator is not the missing lever.

2. **Harness validated by reproducing #64.** The naive additive Arm A blows up to 23.92 bits / **0.18% top-1** — reproducing #64's known Σ-over-cloud double-counting collapse (~0.3% top-1) almost exactly. That both validates the instrument and shows why the chain-telescoped redesign was needed in the first place.

This corroborates **#456** (the region reconstruction is sub-unigram) and **#459** (k≥3 marginals are counting noise) from a third, independent angle.

## Honesty caveat (recorded in the harness)

Arm A here is the *naive* additive sum of every active region's residual — not the deployed chain-telescoped/EXCT scorer — so the "+15.3 bits B-over-A" is against the #64 strawman, not the shipped scorer. The load-bearing number is **B = unigram floor**, which needs no Arm A to interpret: even the principled max-entropy consistent reconstruction extracts no signal.

## Scope

New `#[ignore]`d, fixture-gated harness `marginal_reconstruction_457.rs` (measurement only, no compiler change), following the #499/#456 pattern. Anti-vacuity guards per Rule 5: asserts IPF actually moved the distribution (mean L1 1.97) and reports the floor control. RESEARCH.md updated to record the NEGATIVE; the #488 row and closed-note are reconciled in the same pass. fmt + clippy (`-D warnings`) green.

Closes #457.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

