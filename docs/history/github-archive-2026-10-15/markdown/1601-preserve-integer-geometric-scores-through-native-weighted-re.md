# #1601 — Preserve integer geometric scores through native weighted read reduction

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T04:26:12Z
- **Updated:** 2026-10-09T13:17:29Z
- **Closed:** 2026-10-02T06:08:55Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1601
- **Merged:** 2026-10-02T06:08:55Z
- **Merge commit:** 18b0041f379a764e1043698eecaa73f6fc144f7d
- **Base / Head:** main / UOR-Foundation:codex/geometric-read-reducer-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The geometric context scorer still fed a floating weighted reduction. This adds an artifact-bound integer geometric reader: Q24 scores and age/NoRead, Q31 table weights, widened Q16 value reduction and exact restoring division. It preserves occurrence identity, null weight and cancellation separately, exposes the actual prediction-producing trace, and independently recompiles on reload.

Actual release replay of both saved context models completed on original/stress development panels (512 rows total): all answer predictions retained, scores100/110 for seed1 and128/127 for seed2. All64 comparison chunks restore the baseline exactly. Logits are close but not bit-identical; largest full actual-position drift0.000223205. The receipt binds executed source753b2311, executable9dc45595 and allfour summaries.

Validation: four integer kernel checks, four compiler/binding checks, integration and trace checks passed. Actual release evaluator built and ran twice; reports sealed and Rust-verified. Scoped numerical-symbol opcode inspection found no mul/div/float arithmetic in six inspected test-binary bodies; external callees/fullmodel were not qualified. Rebase preserves all executed changed-source blobs. Claim wording check passed for documentation.

Donor values/NoRead and downstream layers remain floating interim boundaries. No general language, complete float-free serving, geometry advantage or energy claim. Next work is geometric value representation and learned production. References #1512 and #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

