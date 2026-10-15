# #323 — feat(certify): #318 dot-residual rows (positive) + Phase B design note

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T01:01:05Z
- **Updated:** 2026-10-09T14:17:15Z
- **Closed:** 2026-08-01T01:19:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/323
- **Merged:** 2026-08-01T01:19:20Z
- **Merge commit:** 0e49a5eb58e39c16b03496e96920f59bbc60564b
- **Base / Head:** main / UOR-Foundation:issue-318-dot-resid
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Refs #318. Follow-up to #243/#310.

Two commits:

1. **`feat(certify)`: dot-residual composition rows + po2x3 sweep.** Adds the missing matrix cell — dot assignment *with* per-stage residual updates at unit scale: `A-dot-resid` (f32 values, isolates the residual effect), `A-dot-po2-resid` / `A-dot-po2x2-resid` (assignment AND subtraction from the same quantized tables — the kernel-buildable form), plus `A-dot-po2x3` completing the terms sweep.

   Measured (fresh-era rows-only run; baselines reproduce #310 exactly): **A-dot-po2x2-resid 31.2/35.3** vs shipped-shape 30.7/34.3 and A-f32 ceiling 32.1/36.4 — **recovers ~36-48% of the remaining gap; po2 value-set costs nothing; terms saturate at 2**. WB worsens on doubled keys (trade-off on record). Full matrix: #318 comment.

2. **`docs`: Phase B design note** (`docs/dot_residual_phase_b_design.md`) — integer residual wiring for the shift-add dot path: po2 norm fold via L1 bit-length (division-free normalization candidate, ⚑ validate certifier-side first), per-stage integer centroid copies (token stage-book precedent), WB/key-dispersion Empirical Criterion with a store-shape decision on the #244 harness, and a 3× bit-for-bit reproduction requirement before any kernel code.

No runtime-kernel changes; no format changes; no κ impact. Certifier-side instrumentation + docs only.

Local gates: clippy `-D warnings` + fmt clean on touched crates; claim-wording gate ✓. Full workspace gates run before merge per batch discipline.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

