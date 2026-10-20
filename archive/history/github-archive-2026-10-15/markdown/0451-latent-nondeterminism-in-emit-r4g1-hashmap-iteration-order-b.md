# #451 — Latent nondeterminism in emit_r4g1: HashMap iteration order breaks candidate_weights ties before truncate(64)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-06T04:13:08Z
- **Updated:** 2026-08-06T13:44:15Z
- **Closed:** 2026-08-06T13:44:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/451
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Found during the #448 investigation (not the cause of it, and not in the score path). `induction.rs:2332-2353` in `emit_r4g1` builds `candidate_weights` in a `std::collections::HashMap`, collects to a Vec, then `sort_by_key(Reverse(weight))` — a stable sort over randomized HashMap iteration order, so **equal-weight ties are broken nondeterministically** before `truncate(64)`. Current fixtures happen to have no ties at the cut, so nothing is failing today; this is a latent reproducibility defect in the `cover` command's r4g1 emission path. Fix: iterate a BTreeMap, or make the sort key (weight, token) so ties are canonical.

## Comments (1)

### comment 5205448373 — 2026-08-06T13:44:14Z — Casey-allard

Fixed by PR #453 (merged): canonical tie-break in `emit_r4g1`. Verification recorded on the PR — three `transformerless cover` runs on identical inputs produced three different κ before the fix and a single stable κ after. Closing as resolved; reopen if any emitter path still shows digest drift.


---
_Comment/review ids in this file: 5205448373_

