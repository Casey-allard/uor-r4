# #652 — research(riemann-lean): PrimeRiemannBridge RH formalization + scripts

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T00:08:30Z
- **Updated:** 2026-10-09T14:17:27Z
- **Closed:** 2026-08-13T00:28:28Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/652
- **Merged:** 2026-08-13T00:28:28Z
- **Merge commit:** ab0a83185b660295aca65148ca10105700bb2f7f
- **Base / Head:** main / UOR-Foundation:riemann-lean-followup
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Adds Casey's Lean 4 RH formalization (the PrimeRiemannBridge O1-O5/L1-L3 scaffold, 23 hand-written .lean, no axioms) + 185 research scripts + arXiv metadata under research/riemann-lean/. This was a standalone local project (Personal/Downloads/New project), never in AI-Research or Prime-Analysis, which is why the earlier consolidation didn't capture it. External Lean deps (Lean-RH/PhysLean/Carleson/PrimeNumberTheoremAnd/mathlib) are pinned in lake-manifest.json and fetched via lake rather than vendored (their local copies hydrate impractically slowly from iCloud; can be vendored in a follow-up if wanted). Excluded: .lake/.olean build caches, .vendor Python binaries, .npz caches, and AMN work files. research/ is outside the Rust workspace so CI gates skip it.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

