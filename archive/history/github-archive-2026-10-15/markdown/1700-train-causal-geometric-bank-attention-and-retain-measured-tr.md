# #1700 — Train causal geometric bank attention and retain measured tradeoffs

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T22:44:24Z
- **Updated:** 2026-10-09T13:17:19Z
- **Closed:** 2026-10-04T22:45:36Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1700
- **Merged:** 2026-10-04T22:45:36Z
- **Merge commit:** 2e458888bb74eb06c0efd29eea47a7544f12fdc7
- **Base / Head:** main / UOR-Foundation:codex/geometric-bank-fit-20261004
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The native geometric bank learner now trains on causal multi-source contexts instead of supplied single records. A Rust preparer retains all 64 original controls and adds 64 counterbalanced same-bank/different-query episodes, with 32 prospective fresh familiar-value combinations. The joint fit uses ordinary native Copy/Period/Stop alias loss, integer export/reload and parent-inclusive checkpoint selection.

Actual 64-update CPU fit improves bank CE from 2.162916 to 1.735702 at update 48, while all original single-source rows regress. Combined selection retains the parent. Parent/selected generation is 31/128 complete, all old controls, and fresh 0/32; learned checkpoint generation remains unmeasured. All candidates and the initial preparation failure/repair are preserved. The next action is a zero-update crossed context/readout diagnostic and development generation, not an unchanged fit or geometry-family rejection.

Validation: 13 focused Rust tests and optimized build pass on Linux CPU; actual broadbatch checks native parity/support/finite credit; independent saved audits pass 6,521 panel/batch and 27,137 fit checks. Formatting, diff and claim-wording checks pass locally. Exact source/executable/artifact bindings and complete cost are in the evidence record. Rust source seal checks and remote/local complete-file-set SHA256 verification are separate from unavailable local BLAKE3 and independent backward reconstruction. No GPU, Metal qualification, GitHub research runners, shared session changes or general-chat claim.

References #1552. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

