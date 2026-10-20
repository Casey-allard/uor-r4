# #1428 — Read shared packed signed4 coefficients in native integer kernels

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T09:38:29Z
- **Updated:** 2026-10-09T13:17:46Z
- **Closed:** 2026-09-27T14:51:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1428
- **Merged:** 2026-09-27T14:51:27Z
- **Merge commit:** 8a6d499112c0230a38db918340bc6d54f702b96e
- **Base / Head:** main / UOR-Foundation:codex/packed-integer-coefficients-20260927
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Retained signed4 coefficients were expanded to `i16` and copied into cached parameter fields. The integer runtime now stores shared packed bytes and reads them directly in embedding, vocabulary and affine kernels, preserving the existing artifact format, dimensions, scales and rounding. Signed16 arrays are shared separately. A payload accessor reports coefficient storage without claiming RSS or sparse parameter access.

Four focused release arithmetic checks pass, and the library plus all three CLI binaries compile. Independent reviews found and corrected the instruction auditor's moved-symbol/caller coverage. Static inspection found no forbidden opcode in 30 matched library ranges and 28 matched conversational-binary ranges; baseline and candidate ordinary CLIs retain the same three missing conversational symbols, reported as coverage failures. The source-bound evidence packet includes all actual coverage and costs.

**Draft:** loaded-artifact output/state/RNG equality and complete-workload cost remain pending until the fixed dialogue pair releases the single owned model slot. No model, language, speed, energy or D5 selected-access promotion is claimed. The running learning study and other labs' session/read changes are unchanged.

References #973.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5855928391 — 2026-09-27T12:44:59Z — Casey-allard

Fourth lab: the frozen packed-runtime comparison is complete and independently reviewed. [Source-bound result](https://github.com/UOR-Foundation/uor-r4/blob/0607b43b/docs/integration/packed-integer-comparison-2026-09-27.md) and [portable evidence](https://github.com/UOR-Foundation/uor-r4/blob/0607b43b/docs/evidence/packed-integer-comparison-2026-09-27.json) are delivered in #1431; implementation #1428 is ready for protected review.

All 16 processes exit successfully; all eight baseline/candidate pairs preserve the declared stable fields. This covers 4,096 paired replay positions, 138 generation requests per version and four complete saved chat DTOs per version on the retained Quaternion/Householder-pair width-256 artifacts. Exact outputs and their existing language failures remain. The old chat CLI's unrecorded terminal/stop fields are outside this evidence; no save-load-resume or width-576 equivalence is claimed.

Source-derived logical coefficient payload falls from 6,702,340 to 847,876 bytes (87.35%) by packing and sharing. Serialized artifact size is unchanged. Observed peak child RSS falls 13.04–16.83% across every pair; timing differences are small/mixed in one counterbalanced pass. Dense coefficient access remains. No general speed, energy, D5 or language promotion. Supervisor time is 50.774 seconds; the complete preparation/review/delivery ledger is now 43,869,592 / 64,800,000 ms owned and 692,025,172 / 722,400,000 ms verified shared, with the reporting tail still open. No external compute, deletion or allowance increase.

Decision: deliver this scoped representation change without another sweep. The next native-inference question is the selected full-prefix child's own conversion, since historical R1d conversion damage is not evidence about that new child. Preserve the historical zero-update exporter contract; implement an honest child lineage and observe one same-child nearest-hard artifact before deciding whether response-aware fixed-grid code learning is necessary. No alpha fit or additional training dose has been launched.



---
_Comment/review ids in this file: 5855928391_

