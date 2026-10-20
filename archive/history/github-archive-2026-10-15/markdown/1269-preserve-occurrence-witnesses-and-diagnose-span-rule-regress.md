# #1269 — Preserve occurrence witnesses and diagnose span-rule regressions

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-14T07:07:03Z
- **Updated:** 2026-10-09T13:17:46Z
- **Closed:** 2026-09-14T07:09:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1269
- **Merged:** 2026-09-14T07:09:03Z
- **Merge commit:** c7b18a747c808bec2f76f3bb588b6b2d6e85370a
- **Base / Head:** main / UOR-Foundation:codex/occurrence-correspondence
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Overlapping words in the known query phrase and proposed answer were collapsed into union-match barriers. Preserve candidate-specific injective occurrence witnesses, bounded search and per-witness span admission in an opt-in experimental path using the existing geometric comparison, phrase updater and completion/writer loop.

**Qualification failed; no model promotion.** The unchanged diagnostic gives 144/384 new answers. Correspondence alone gives 296/384 and retains 6304/6304 earlier traces. One bounded fit gives 384/384 new cases but only 3992/6304 retained traces and 1904/2816 training rows. A no-fit diagnostic establishes that 1904 is the ceiling of the frozen 137-proposal family under zero incompatible admission: 912 sole reachable targets share an admission signature with incorrect spans. Preserve all negative candidates and the qualified parent. Next expose ordered correspondence structure before feature reduction; do not repeat fitting of unchanged masks.

Validation: four optimized offline Rust builds; final 54 focused tests; four explicitly executed, sealed report attempts; cargo fmt --check, claim-wording and diff checks pass. Reports preserve all 55 prior roots/1361 files and both original dirty checkouts; total 59 roots/1499 files. Actual model/build/preparation/evaluation charge 757066 ms, no paid compute or cleanup. The normal 15baec48 model and qualified experimental 44c0 artifact remain unchanged. Final independent holdout and general prose qualification are NOT_RUN. Queue statuses are compatibility acknowledgements, not executed tests.

Result: docs/native_geometric_occurrence_973.md. Hashes, reports, source/binary versions and retention detail: docs/evidence/native_geometric_occurrence_973.json. Current state and navigation updated.

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

