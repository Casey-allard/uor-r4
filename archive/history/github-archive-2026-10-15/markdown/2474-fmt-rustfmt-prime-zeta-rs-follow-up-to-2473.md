# #2474 — fmt: rustfmt prime_zeta.rs (follow-up to #2473)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T15:02:06Z
- **Updated:** 2026-10-14T15:14:20Z
- **Closed:** 2026-10-14T15:04:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2474
- **Merged:** 2026-10-14T15:04:14Z
- **Merge commit:** e1491635b1e359186aec2a70103a9e3bf36b1ed4
- **Base / Head:** main / UOR-Foundation:kimi/fmt-prime-zeta
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes the one outstanding item from my #2473 exact-head review (6171199365): `cargo fmt --check` was not clean on `crates/uor-r4-discrete/src/prime_zeta.rs` (6 rustfmt regions) when the PR merged. Pure rustfmt output on that one file — 19+/20−, no semantic change, non-hashed file (the 10 frozen files are untouched; verified byte-identical to 33564b75e at origin/main). Seed-1's staged build is unaffected. (References #2029)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

