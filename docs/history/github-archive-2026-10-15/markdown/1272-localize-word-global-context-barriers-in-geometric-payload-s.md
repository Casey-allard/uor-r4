# #1272 — Localize word-global context barriers in geometric payload selection

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-14T12:39:45Z
- **Updated:** 2026-10-09T13:17:11Z
- **Closed:** 2026-09-14T12:41:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1272
- **Merged:** 2026-09-14T12:41:06Z
- **Merge commit:** 5bcb5b17ebade7eb565dec65e0f8c737a9fc1614
- **Base / Head:** main / UOR-Foundation:codex/contextual-role-diagnostic
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The unchanged correspondence artifact treats every occurrence of a learned context word as ineligible answer text. This new independent raw-oracle diagnostic demonstrates the consequence for a person named `will`, using matched ordinary-name chains, both relation directions, source rotations and active/inactive changes.

**FAIL_LEXICAL_ROLE_REUSE; GLOBAL_CONTEXT_PAYLOAD_BARRIER_LOCALIZED.** Ordinary-name cases pass 24/24; `will` cases pass 0/24 and end Exhausted with empty output. All intended spans are enumerated, but every `will` target witness lacks only required bit 18. Actual direct first reads reproduce the failure. Supplied-payload update and oracle-query route probes succeed 24/24, explicitly counterfactual and excluded from qualification. Full/ExactIdentity and reload agree on all 48 complete structs. No runtime/artifact change or fit; normal 15baec48 remains unpromoted.

Validation: one optimized offline build, 30 focused tests, fmt, claim-wording and diff checks. All 66 sealed roots/1,636 files, 12 predecessor source/binary versions and original dirty checkouts are preserved. Charge 153307 ms; before execution a necessary parent storage extension of 128 MiB was recorded under standing authorization, retaining the 128 MiB stop margin. No external spending or cleanup. Older model-control panels were not rerun because runtime/artifact bytes remain unchanged. Queue acknowledgements are not tests.

[Result](docs/native_geometric_lexical_role_973.md) and [evidence](docs/evidence/native_geometric_lexical_role_973.json) record actual versus counterfactual behavior and limits. Current state specifies the next bounded learned occurrence-role implementation using existing output-compatible latent-span credit. Same-byte dual-role occurrences require structural disambiguation; an oracle span, first-match tie-break or global word exception must not substitute for learning. This remains authored syntax, with final qualification NOT_RUN.

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

