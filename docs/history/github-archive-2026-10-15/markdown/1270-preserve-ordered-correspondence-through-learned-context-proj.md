# #1270 — Preserve ordered correspondence through learned context projection

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-14T07:50:33Z
- **Updated:** 2026-10-09T13:17:12Z
- **Closed:** 2026-09-14T07:52:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1270
- **Merged:** 2026-09-14T07:52:52Z
- **Merge commit:** fb8c2de9de8c4929fe6dfec93abac05673d8bf32
- **Base / Head:** main / UOR-Foundation:codex/correspondence-runs
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Ordered occurrence witnesses previously collapsed distinct phrase relationships into the same selector features. Preserving matched runs lets final-output learning select an ordering constraint. Its initial raw-gap version regressed 960 retained traces; the corrected predicate projects adjacency through inherited learned context roles while preserving content gaps and exact occurrence order.

The versioned experimental wrapper passes **PASS_ORDERED_CORRESPONDENCE**: 2,816/2,816 actual training outputs, 384/384 new development cases and 6,304/6,304 earlier complete traces, including 512 typed unresolved outcomes. It uses one fit; the correction reuses its learned mask with no second fit. Restoring raw gaps reproduces the same 960 failures. Structure-disabled and joint multiplicity controls demonstrate the tested constraints. Normal model 15baec48 and all embedded parent parameters remain unchanged.

Validation: optimized Rust build; 59 focused tests; cargo fmt --check; claim-wording and diff checks. All 64 sealed report roots/1,620 files and original dirty checkouts are preserved. The first failed candidate, reporting correction and compile failure retain their original evidence. Model/build/evaluation charge 667,826 ms; no external cost or cleanup. Local checks carry validation; merge queue compatibility acknowledgements are not tests.

[Result](docs/native_geometric_correspondence_973.md) and [source/artifact evidence](docs/evidence/native_geometric_correspondence_973.json) record exact counts, controls and lineage. This qualifies familiar authored grammar with finite inherited roles; independent final holdout, general prose, metric advantage and default-model promotion remain unestablished. Current state points to unchanged-artifact phrase-order and gapped-distractor qualification.

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

