# #270 — docs: #243 design — f32 residual-VQ store + mul-free graded-signature query assignment (REVIEW)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-28T17:56:34Z
- **Updated:** 2026-10-09T13:08:54Z
- **Closed:** 2026-07-29T15:05:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/270
- **Merged:** 2026-07-29T15:05:24Z
- **Merge commit:** 548ae5cd83490d72b20538b68660f687de46c09f
- **Base / Head:** main / UOR-Foundation:issue-243-design-doc
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #243 (the issue closes when Phase C ships; this PR is the design gate).

**Review requested from @Casey-allard, Ari, and Alex (Maura welcome)** — decision points are marked ⚑ in the doc: Design R (integer residual subtraction) vs Design G (multi-bit thermometer-graded signatures) vs R∘G; grading width b; centroid integer width; runtime op budget. Phase A is certifier-side measurement only and can start as soon as the shape of the rows is agreed, even before all ⚑ are resolved.

Design highlights: compile-time is contract-unrestricted (settled), so the store builds with true f32 residual-VQ and only the query side needs a mul-free approximation; thermometer-coded graded signatures make Hamming equal L1 — order-preserving under truncation, i.e. the 'chop bits for locale' property, delivered in the relational layer where it belongs (docs/addr_prism_correspondence.md); every representational choice is treated as an address-space interface change (#263/#264/#247 consume it; #265 gates cross-platform ladder determinism). Empirical Criterion with teeth: recover ≥ half the A-f32 gap at unchanged op census, or the rows stand as a recorded negative result and nothing ships.

Claim-wording gate passes.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_017tu2Y4AGZvcgM4vmoSz5x6

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

