# #325 — feat(certify): #318 Phase A.5 rows — po2 norm fold + centroid-copy widths

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T01:25:02Z
- **Updated:** 2026-10-09T14:17:16Z
- **Closed:** 2026-08-01T03:54:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/325
- **Merged:** 2026-08-01T03:54:37Z
- **Merge commit:** 7887af429938f0da409a9508166bb3105e08ea62
- **Base / Head:** main / UOR-Foundation:issue-318-phase-a5
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Refs #318. Follow-up to merged #323 — Phase A.5 of docs/dot_residual_phase_b_design.md, certifier-side only:

- **A-dot-po2-nf-resid**: the division-free normalization candidate — po2 norm fold (L1 bit-length − train-median CONST) replacing true-norm division in the residual-dot row. Design-note gate: must lose ≤0.1pp vs A-dot-po2-resid (31.1/35.2).
- **A-dot-po2-resid-cpy8 / cpy16**: 1-term po2 assignment with i8/i16 per-stage integer centroid copies (IEEE-exponent stage scale, libm-free) for residual subtraction — settles the ⚑ copy-width decision.

Plus the 3× bit-for-bit reproduction required by the design note (results posted to #318 when the runs complete). No runtime changes; no κ impact.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

