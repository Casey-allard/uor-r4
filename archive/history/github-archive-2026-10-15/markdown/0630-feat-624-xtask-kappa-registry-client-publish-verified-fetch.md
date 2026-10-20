# #630 — feat(#624): xtask kappa-registry client — publish, verified fetch, CAS κ pins

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-12T01:06:14Z
- **Updated:** 2026-10-09T14:17:06Z
- **Closed:** 2026-08-12T01:25:25Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/630
- **Merged:** 2026-08-12T01:25:25Z
- **Merge commit:** 0e5fae14fd67dc7cc78b40d529c22bf585ce2b14
- **Base / Head:** main / UOR-Foundation:i624-kappa-client
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

feat(#624): xtask kappa-registry client — publish, verified fetch, CAS pins

Adds `cargo xtask kappa <publish|fetch|tag-get|tag-cas>`: a std-only
HTTP/1.1 client (zero shipped-crate dependencies; xtask gains blake3 at
the workspace's exact =1.5.0 pin). Publish PUTs by the artifact's
existing blake3:<hex> label — byte-identical to the registry's
KappaLabel — and the registry re-derives the hash on write, making every
publish an out-of-process witness of the deterministic-rebuild claim.
Pins move only through If-Match compare-and-set (Gate-E discipline);
blind overwrite is not offered. Verified end-to-end against a locally
built kappa-registry (transcript on #624): round-trip, stale-CAS
refusal, correct-CAS move, fetch re-derivation, corrupt-label 400.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

