# #1860 — Record geometric entry learning and close the full-pass dose comparison

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T22:03:00Z
- **Updated:** 2026-10-09T13:17:11Z
- **Closed:** 2026-10-07T22:20:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1860
- **Merged:** 2026-10-07T22:20:29Z
- **Merge commit:** 04b993bef45d92e081c315899a929bda6c118bfb
- **Base / Head:** main / UOR-Foundation:codex/continuation-fit32-result-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Two matched shared-action H4 learning runs establish entry acquisition while complete answers regress. On the exposed 512-case panel, correct entries rise 8 → 455 → 511 at updates 0/32/64, but complete answers with EOS fall 8 → 5 → 1, with no new complete success. The 64-update run reproduces the earlier checkpoint32 exactly before completing the remaining half-pass.

Record verified own-feedback results, source/data/artifact identities, CPU/CUDA checks, costs and HF preservation. Retain the accepted parent, end the unchanged U-only dose comparison, and scope the next change to the existing live joint-learning graph with explicit prototype and cache boundaries. These are construction results from one lineage, not generalization or a geometric-family verdict.

Validation: documentation only; git diff --check and scripts/check_claim_wording.py pass. Independent review reconstructed complete replies/EOS, native CE, feedback chronology and data answerability. No Rust source changed and no extra model run is required for this record.

References #820.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6048018029 — 2026-10-07T22:19:36Z — Casey-allard

Exact-head self-review: 0d49822dd47df2f494101178f596f3f0d9cd9864.

Documentation-only change to current-state and the canonical project plan. `git diff --check` and `python3 scripts/check_claim_wording.py` pass at this head; no Rust code or executable behavior changed, so additional compilation/model execution is unnecessary.

Independent evidence reviewer read both changed files at this exact head and approved the table, gained/lost replies, EOS, teacher-versus-own-feedback scope, accepted-parent decision and prospective joint-learning boundary. The reviewer independently reconstructed the saved 32/64 endpoints. The architecture reviewer identified the existing live graph and prototype-adjoint/fixed-cache limits. I verified HF fresh-download archive hashes and both lease releases/pod deletions separately; these operational facts are not attributed to the evidence reviewer.

The record does not promote either fitted model, retire the geometric family, or call first-token success complete prediction. References #820; the broader project remains open.



---
_Comment/review ids in this file: 6048018029_

