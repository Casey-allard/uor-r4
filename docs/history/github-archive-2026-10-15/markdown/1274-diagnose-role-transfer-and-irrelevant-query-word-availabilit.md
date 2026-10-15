# #1274 — Diagnose role transfer and irrelevant query-word availability

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-14T13:50:46Z
- **Updated:** 2026-10-09T13:17:55Z
- **Closed:** 2026-09-14T13:52:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1274
- **Merged:** 2026-09-14T13:52:04Z
- **Merge commit:** ecee92eba3ad0b30c72005801d6702057dd4e45c
- **Base / Head:** main / UOR-Foundation:codex/role-transfer-stability
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

An irrelevant record can make an optional question word mandatory for candidate admission, and can also create extra dependent update sites. The unchanged role artifact additionally blocks or truncates some styled name occurrences. This Rust diagnostic freezes 204 authored cases and records actual first payloads, queries, source/span identities and final outcomes without changing runtime or learned parameters.

The broader gate is **FAIL_UNCHANGED_ROLE_TRANSFER**: coverage 12/60, styled roles 60/96, familiar/unobserved predicate anchors 48/48. All 200 most recent prior Full traces remain exactly equal. CoverageDisabled restores all first payloads in the 60 coverage cases but only 24 complete results; 36 expose AmbiguousUpdate. Twelve styled cases return the expected text through an incorrect truncated intermediate, which the path criterion rejects. ExactIdentity and reload equal Full on all 204 structures, including failures. Required read/update disabled controls produce no correct answered responses.

Validation: optimized offline Rust build; 34 focused tests; one sealed nine-file diagnostic; cargo fmt --check; claim-wording check; git diff --check. No fitting, runtime changes or old 6,688-row replay. All 69 prior sealed roots/1,671 files, 17 source/binary versions and original dirty checkouts are preserved. Build/evaluation charge 167,797 ms, zero external cost or cleanup. Final holdout NOT_RUN; default model unchanged and unpromoted. CI compatibility acknowledgements are not behavioral test results.

The result and current state identify the next correction: separate learned query requirements and replacement eligibility from context/content roles, then repair styled role transfer while retaining actual paths and controls.

References #973


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

