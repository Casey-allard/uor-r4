# #591 — feat(#515): PR-2 — compile-gate graph-certify dormant surfaces (default OFF)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T20:28:19Z
- **Updated:** 2026-10-09T14:17:25Z
- **Closed:** 2026-08-10T20:47:04Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/591
- **Merged:** 2026-08-10T20:47:04Z
- **Merge commit:** 9201e2ce6e70d66bb4fd31ba318032ef9b67a9b3
- **Base / Head:** main / UOR-Foundation:r5/gate-certify-dormant
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## #515 PR-2 — compile-gate graph-certify dormant surfaces

Second tranche of the preserve-and-gate work. The six off-serving-path
`graph-certify` mechanisms registered as `open` ledger claims in #590 are now
behind **default-off** cargo features, one per claim:

| feature | module | ledger claim |
|---|---|---|
| `anti-degeneracy` | `anti_degeneracy` | `anti-degeneracy-dormant` |
| `fairness-provenance` | `fairness_provenance` | `fairness-provenance-dormant` |
| `holographic-encoding` | `holographic_encoding` | `holographic-encoding-dormant` |
| `patch-lifecycle` | `patch_lifecycle` | `patch-overlay-dormant` |
| `predictive-sufficiency` | `predictive_sufficiency` | `predictive-sufficiency-dormant` |
| `shortlist-evaluator` | `shortlist_evaluator` | `shortlist-evaluator-dormant` |

### Dormancy is proven by the build
- **default** workspace build compiles and links with none of these modules; their five dedicated integration test files (`#![cfg(feature = ...)]`) reduce to 0 cases.
- **`--all-features`** compiles all six and runs their 20 tests (4+4+6+3+3), clippy clean both ways.
- The shipped default artifact is byte-identical whether or not these features are enabled.

On-path certify surfaces (`certificate`, `certify`, `compare`, `fmm`, `performance_certificate`, `score`, `score_runtime`) are untouched.

Refs #515.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

