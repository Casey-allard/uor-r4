# #1052 — A1Q-H10: reproduce released Zoology width-64 attention control

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-02T13:26:42Z
- **Updated:** 2026-10-09T14:17:09Z
- **Closed:** 2026-09-02T13:28:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1052
- **Merged:** 2026-09-02T13:28:03Z
- **Merge commit:** 0cffa3fb0a17a891383aa76d34104b9a906237fc
- **Base / Head:** main / UOR-Foundation:issue-1050-zoology-release-reproduction
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #1050

## Outcome

- Reproduces the released ICLR24 Zoology T=64, four-K/V, one-head width-64 causal-softmax configuration and training semantics on CPU, with declared CPU-placement and query-only-projection adaptations.
- The first frozen learning rate passed at epoch 20: 11,900/12,000 = 99.1666667% source-test top-1, NLL 0.05124610455830892, arm wall 577.834602 seconds.
- The predeclared early stop fired, so the remaining three rates were not run.
- Result CID: blake3:bd16d012c01262ffb8c5197e4cf316c6fee1d722cf0700a0048386180a8122e0.

## Claim boundary

This establishes copied ordinary causal-softmax key/value binding on independently generated rows. The source test split is held out from gradient updates but drives early stopping; it has zero full-row overlap and 294 recurring K/V pairs. It rules out a broken copied cell and localizes #1049 to bundled training-contract differences without isolating a causal factor. It does not establish R4/geometric attention, English, generation, reasoning, modulo-256 softmax, or exact lowering.

## Verification

- six focused reproduction tests passed
- fresh-process artifact/result verification passed
- independent dataset rebuild and checkpoint/artifact byte audit passed
- claim-wording gate passed
- cargo fmt --check passed
- git diff --check passed
- Python format and baseline E/F lint passed

Final audit records two non-blocking lifecycle limits for repair in the exact-#1045 successor: dependency files are not recursively included in this result CID, and arbitrary interruption immediately after a passing checkpoint is not claimed deterministic.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

