# #1603 — Decode signed geometric values and distinguish representation loss before learning

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T05:29:04Z
- **Updated:** 2026-10-09T13:17:32Z
- **Closed:** 2026-10-02T06:17:40Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1603
- **Merged:** 2026-10-02T06:17:40Z
- **Merge commit:** b975fedc8fa0b639ee3bbe88516c69124c96fc86
- **Base / Head:** main / UOR-Foundation:codex/geometric-value-codec-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The native weighted reader still consumes donor values. This adds the fixed signed-H4/radius value alphabet and an oracle representation comparison before learning its producer: exact golden-interval Q16 construction, separate absent/presentzero/nonzero packets, source-bound compiler/reload, and direct rawQ16 integration into the existing reader. The canonical roadmap now separates context, weighted read, value/null production, geometric output and useful full-path qualification.

Both saved context models completed the unchanged-weight comparison on512 authored development rows. Seed1 original100→85 and stress110→96; seed2 original128→128 and stress127→127, with one gain and one loss on stress. All927232 read-coordinate error-bound checks hold; all64 chunks restore the baseline. Retain every changed row. Nearest-L2 K1 is not a preserving drop-in map for the frozen decoder; this does not disprove task-trained K1 or geometric values.

Validation: integer codec4, training compiler4, directQ16 test1 and loaded context/value integration1 passed. cargo fmt --check and claim wording passed; release example built and both actual reports were sealed/Rust-verified. Focused fixture failures and a pre-worker supervisor filename failure are retained and repaired. Rebase preserves executed source blobs. Independent source math/compiler/integration reviews completed.

This is an offline donor oracle comparison, not the native learned producer. Donor value/NoRead/downstream float remains explicitly unfinished. Next causal tasks are bounded residual representation diagnosis and learned values from actual geometric latent history with ordinary answer credit, then native NoRead/output. No general language, geometric advantage, complete serving or energy qualification. References #1512 and #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

