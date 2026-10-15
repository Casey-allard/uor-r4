# #1575 — Expose typed temporal memory queries in grounded sessions

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T14:05:29Z
- **Updated:** 2026-10-09T13:17:35Z
- **Closed:** 2026-10-01T14:17:55Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1575
- **Merged:** 2026-10-01T14:17:55Z
- **Merge commit:** 35c2c1f6b42d4dc6b1e8e6be7c405b18755f3ffd
- **Base / Head:** main / UOR-Foundation:codex/grounded-temporal-20261001
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Grounded sessions could request only the current value although their exact store already supports temporal views. Add an artifact-bound typed query action for initial, previous assertion, previous distinct value and current reads, preserving the existing compiler and current-only encoding.

Historical receipt validation reconstructs residency at the recorded commit, preserving successful earlier reads after later eviction and the previous-distinct eviction barrier. Additive actions use session schema2; legacy transcripts retain schema1. The saved language compiler remains current-only, so this is interface construction, not learned temporal-language qualification.

Validation: final exact-head CI [36874138518](https://github.com/UOR-Foundation/uor-r4/actions/runs/36874138518) passed formatting/policy checks, library + m-world + grounded-session CLI compilation,9 session tests,9 store tests and9 saved-compiler tests. The fresh-process child helper was explicitly invoked and passed. Independent nonauthor review approved the same head `7e60eb56`. These are scoped construction checks; the tiny pointer/snap fixture uses an explicit test compiler.

The advisory status publisher failed `Not Found`, separately recorded. D19's manual fallback uses the actual review/check receipts and protected queue, without fabricated status or bypass.

References #962. All source/receipts are internal; external SSD writes remain paused under the owner repair ruling.


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5933141909 — 2026-10-01T14:06:58Z — Casey-allard

Nonauthor source review — APPROVE at exact head `0def0198b2fc52a70641758d6e8df3f2e48fefa8`, module SHA256 `c604cf380893fc55c8d35cf1d4c0f76198dd4d1c03f22ea97bbd33555e149ac4`. Reviewer: Codex `/root/adoption_review_evidence`, distinct from author `/root/principal_engineer` and root delivery coordinator. No material blockers.

Reviewed actual source/tests: original-commit chain-prefix residency; PreviousDistinct eviction before retained-byte comparison; full StoreRead identity/value validation; legacy schema1 and additive schema2; NoRead receipt tampering isolation; 72 captured pre-eviction store reads across views/capacities/corrections/reassertions/interleaved commits; actual tiny pointer/snap current+initial continuation in a fresh process. SavedCompiler remains current-only. Documentation matches linked live storage/data-review statements.

Reviewer performed no Cargo/test/model execution. Actual exact-head CI36873623095 is still running; source approval is not a test or language-quality result.

Advisory delivery publisher36873630706 was inspected: `Not Found`, not PASS. D19 permits manual fallback with explicit review, executed scoped checks and protected merge; no advisory status will be fabricated.


### comment 5933196896 — 2026-10-01T14:09:45Z — Casey-allard

Nonauthor review refreshed — APPROVE exact rebased head `7e60eb5673416c13f720b40e29be8d05067912b4`, base `498d001eac331f14d6b354aa3d1efe387d718cdf`; no blockers. Reviewer `/root/adoption_review_evidence` confirmed clean tree, identical reviewed session module SHA256 `c604cf380893fc55c8d35cf1d4c0f76198dd4d1c03f22ea97bbd33555e149ac4`, unchanged mechanical m-world arm, and no upstream change to inspected store/memory/compiler/CLI/greedy/tokenizer contracts. The docs resolution preserves #1573/#1574 records and retains our temporal interface/checkpoint scope correction. Original detailed review remains valid for unchanged source. Reviewer ran no tests/model worker. Earlier CI36873623095 is superseded as final-head evidence; the rebased exact-head run is required before merge.


### comment 5933330121 — 2026-10-01T14:17:00Z — Casey-allard

Executed validation PASS at exact final PR head `7e60eb5673416c13f720b40e29be8d05067912b4`, base `498d001eac331f14d6b354aa3d1efe387d718cdf`. Free GitHub CI [36874138518](https://github.com/UOR-Foundation/uor-r4/actions/runs/36874138518) explicitly checked out that source; operational workflow commit `073379a12fedb3d834243b6914db1e356263c5d0` is separate.

Actual executed checks: workspace rustfmt; claim wording/policy JSON/diff checks; cargo check training library, m-world example and grounded-session CLI;9 session tests passed/0failed with1ignored parent helper explicitly invoked in a fresh child (1passed/0failed);9 store tests passed/0failed;9 saved-compiler tests passed/0failed. The session cross-product captures72 real store reads. The pointer/icosian model is an initialized test fixture, and its compiler is explicit test-only. No learned temporal-language, full #962, D11 serving or energy result is claimed.

Nonauthor exact-head review: comment5933196896. Original-head successful run36873623095 is retained separately; it is not substituted for this final run. Final advisory publisher36874014639 was inspected and failed `Not Found`; it is not PASS. D19's manual fallback is applied with actual compile/tests and review before ordinary protected queue submission, without admin bypass or status fabrication.



---
_Comment/review ids in this file: 5933141909, 5933196896, 5933330121_

