# #632 — test(#628): two-path κ certificate for the r4g1 header + CID chain

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-12T01:54:09Z
- **Updated:** 2026-10-09T14:17:07Z
- **Closed:** 2026-08-12T02:13:10Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/632
- **Merged:** 2026-08-12T02:13:10Z
- **Merge commit:** afd642acd45e99759eb89ef3d4a8252c3b22d70e
- **Base / Head:** main / UOR-Foundation:i628-two-path
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

test(#628): two-path kappa certificate for the r4g1 header + CID chain

Atlas canonical_kappa_certificate discipline, ported per the #628
decisions (Casey 2026-08-12): an independent serializer written from the
header.rs layout documentation alone must reproduce the production
writer byte-for-byte and kappa-for-kappa; a mutation arm proves the
certificate can fail. Closes the same-writer blind spot that verify-on-
parse and deterministic-rebuild share. During construction the
certificate caught a real doc-reading drift (u64 vs u32 table fields) -
the exact failure class it exists for. Item 1 (census kappa-injectivity)
recorded already-satisfied by inventory: r4 counts by canonical
structure everywhere. Test-only; no shipped code, no new deps.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

