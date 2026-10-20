# #512 — docs(#509): P3 broad-corpus decider — competent-broad teacher lifts the substrate off the floor (POSITIVE)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-08T22:41:34Z
- **Updated:** 2026-10-09T13:08:50Z
- **Closed:** 2026-08-08T23:01:09Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/512
- **Merged:** 2026-08-08T23:01:09Z
- **Merge commit:** 5bf0720d583162ab88718dc720dc915130525d0c
- **Base / Head:** main / UOR-Foundation:docs/509-broad-corpus-p3
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Records the **#509** result (P3 for #320).

**Question.** If the only change is teacher breadth — SmolLM2-360M observed on broad text instead of the narrow stories15M — does the substrate move off the broad-text floor?

**Result (Gate C, 4,358 held-out D3 positions on Simple-Wiki).**

| scorer | top-1 | bits/token |
|---|---|---|
| Rule 1+2 (D4 precedence, causal) | **10.2%** | 12.72 |
| Rule 1+2 + LATENT-MIX (#446 M2, causal) | **29.0%** | 11.71 |
| narrow-teacher off-distribution (recorded) | ~0.1% | — |

Teacher floor on this partition: **3.6015 bits/token**. The substrate sits ~8–9 bits above it — compiler-/data-bound, not saturated.

**Decision: POSITIVE.** Teacher breadth was the binding constraint #320 predicted; the broad-corpus program is warranted. Baseline re-pin stays a maintainer step per AGENTS.md. Full run, honesty bounds, and κ provenance in the doc.

Docs only:
- docs/smollm2_teacher_baseline_320.md — new P3 section.
- docs/RESEARCH.md — ledger rows for #320, #509, #273.

Refs #509.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

