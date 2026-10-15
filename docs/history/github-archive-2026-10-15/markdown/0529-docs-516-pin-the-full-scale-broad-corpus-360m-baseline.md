# #529 — docs(#516): pin the full-scale broad-corpus 360M baseline

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T06:07:51Z
- **Updated:** 2026-10-09T13:08:55Z
- **Closed:** 2026-08-09T06:26:34Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/529
- **Merged:** 2026-08-09T06:26:34Z
- **Merge commit:** 381d3f7c1c7de45b344478c062546dcfc7dc07f7
- **Base / Head:** main / UOR-Foundation:issue-516-repin
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #516.

Re-runs the P3 broad-corpus program (#509) at **serving depth** — the same 3,000 Simple-Wiki articles observed with `--sequence-length 128`, a **17× denser** observation — and pins the result as the working baseline for broad-text substrate accuracy.

## Gate C — 72,864 held-out D3 positions (SmolLM2-360M-Instruct teacher)
| row (causal serving) | top-1 | bits/token |
|---|---|---|
| **Rule 1+2 precedence** (canonical) | **24.30%** | 11.94 |
| best live arm (`rule12_on_fwd_strict_live`) | **31.48%** | 10.43 |
| TLA-3 baseline | 28.21% | 13.62 |
| Rule 1 chain | 1.78% | 15.68 |

- **EXCT-miss 25.7%**, down from 62.5% at the P3 thin scale. Accuracy (Rule 1+2 `10.2% → 24.3%`) and coverage both scale with observation density, as `docs/scaling_law.md` predicts.
- 64 witness replays, **0 failures** (bit-deterministic). Graph ceiling (10.43 bits) stays well above the teacher floor (3.6015 bits) — compiler- and data-bound, not saturated.

## Decision
Adopt **360M-on-broad-corpus** as the pinned baseline for broad-text claims; **retain stories15M** alongside as the home-distribution reference (near-degenerate on broad text at 6.4% next==argmax, which is what makes the broad pin meaningful).

Corpus hygiene: a 0.15% concurrent-loop artifact (213 byte-identical dup records + 6 non-contiguous stories) was removed before compile; the pinned corpus is 360,924 records over 2,994 contiguous stories.

Docs-only (`smollm2_teacher_baseline_320.md`, `transformerless/BASELINE.md`).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

