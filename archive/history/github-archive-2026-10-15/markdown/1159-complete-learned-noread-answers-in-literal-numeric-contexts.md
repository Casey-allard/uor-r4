# #1159 — Complete learned NoRead answers in literal numeric contexts

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-07T00:53:50Z
- **Updated:** 2026-10-09T13:17:17Z
- **Closed:** 2026-09-07T01:20:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1159
- **Merged:** 2026-09-07T01:20:32Z
- **Merge commit:** be32b4103d8f870f8b1f81057c451a609156172e
- **Base / Head:** main / UOR-Foundation:codex/geometric-answer-entry
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Literal numeric prompts can correctly decline arithmetic and select NoRead, then loop into unrelated prose. This adds a bounded learned continuation after the selected NoRead token is actually observed. It reuses the existing word-binding prefix features and integer/table token scorer, preserves the complete accepted parent, and applies only to retained literal numeric records with no derived result. No new session-state field or response provider is added.

Retain `e7c14c99` at that scope. Exposed complete answers improve12/16 to14/16; reserved changed-name/value answers improve14/16 to15/16. All12 reserved numerics and3/4 abstentions pass. The remaining failure copies `coins` as a location answer and remains a source-selection problem. The roadmap and current-state point to joint supported-source/NoRead selection next.

Validation:13 focused tests; actual NoRead and three-turn numeric zero allocations/bytes; entire-parent equality, checkpoint and mutation checks; kernel source scan; formatting, architecture-policy and claim-wording checks. The rebuilt CLI emits the correct abstention and preserved identifier with EOS. Preservation passes48/48 dependent,62/62 earlier responses,24/24 prior transfer,28/28 exposed names,28/28 long-context,5/5 persistent turns,6/6 earlier numeric, three12-case role sets,58/58 computed construction,16/16 independent-result transfer,16/16 complete chains and8/8 identifier returns. Construction is99/103. Prior24 generated-Rust assertions were not rerun; the eight complete identifier outputs are preserved.

Two intermediate candidates remain recorded negatives: one shortened four explanatory answers; the broader binding-conditioned candidate broke a relation-conflict session. Restricting the final operator to its training state scope restores preservation with the same learned table. No new angular-distance or general-language capability claim.

Resource receipts include all corrections, failures and retries:282.150s model work;1364.981s engineering. Cumulative model use3450.276/3570s. A standing-authorized160MiB scoped growth extension followed a guarded CLI build stop; the broader storage ceiling stays unchanged. No material deletion or external model compute. Protected CI remains required; compatibility statuses are not behavioral qualification. Broad dormant release QA is NOT_RUN.

Refs #1139, #1140. Full result and compact evidence: `docs/native_geometric_no_read_completion_1139.md` and `docs/evidence/native_geometric_no_read_completion_1139.json`.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

