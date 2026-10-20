# #1834 — Record selected contextual credit CUDA validation and limits

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T06:40:13Z
- **Updated:** 2026-10-09T13:17:55Z
- **Closed:** 2026-10-07T06:45:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1834
- **Merged:** 2026-10-07T06:45:41Z
- **Merge commit:** 762384daa97edfdd885bdb47877e4d58ce7d229a
- **Base / Head:** main / UOR-Foundation:codex/selected-context-credit-record-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Record the merged selected-pair CUDA contextual-credit implementation and its executed validation in current-state. The entry distinguishes the retained full-bank objective and exact native anchors from host preparation, diagnostic timings, and unmeasured learning/whole-fit performance. It links #1833, records the independently audited second completed matched pair, and preserves the six immutable fits and their next decision. Four of six arms have finished; the final pair remains running/queued, so no three-order policy verdict is asserted.

Documentation only, based directly on main after #1833 landed. At exact head `e14586e8c5cd54897f6bbf3b8cf109716e037905`, `python3 scripts/check_claim_wording.py` and `git diff --check` pass. The three changed code blobs in merge `5bc6f31ea32cb680f9eb6814051190dd485e50a7` were separately checked equal to tested head `c0949cef01ecc48fc65da0a6180cedb152c7fbb5`. No additional model, CUDA or Rust code is changed by this record.

References #820.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6032543169 — 2026-10-07T06:44:49Z — Casey-allard

Exact-head self-review READY at e14586e8c5cd54897f6bbf3b8cf109716e037905. Documentation only: one current-state file, no Rust/model/runtime change. At this head the claim-wording gate and git diff --check origin/main...HEAD both exit0. The #1833 merge/code-blob identities and exact-head 4 CPU / 3 CUDA executions are verified separately, and the new order1002 result is bound to original producer7fbfc940, 128 updates and sealed artifacts. Independent artifact audit ran the existing Rust BLAKE3 complete-file verifier, checked all512 final-row hashes/IDs and matched row/order identity against the clipped arm. Teacher-prefix gains remain conditional; complete/entry/qualified own-prefix remain0/512, final order pair remains unfinished, and there is no held-out or chat verdict. No redundant model/CUDA run is required for this documentation-only change.


---
_Comment/review ids in this file: 6032543169_

