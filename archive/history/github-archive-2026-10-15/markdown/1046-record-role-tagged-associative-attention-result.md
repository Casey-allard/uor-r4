# #1046 — Record role-tagged associative attention result

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-02T07:13:27Z
- **Updated:** 2026-10-09T14:17:11Z
- **Closed:** 2026-09-02T07:14:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1046
- **Merged:** 2026-09-02T07:14:45Z
- **Merge commit:** c74cdce75e90db9d5cb6926e5cdfa4173b99d3a7
- **Base / Head:** main / UOR-Foundation:issue-1045-role-tagged-associative
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Implements and executes the frozen open role-tagged associative-first attention ladder for #1045.

- R0 passed deterministically at `256/256` query top-1 with zero causal/source leakage.
- R1 completed 64 epochs and `4,194,304` query presentations on Apple CPU/Accelerate.
- Construction reached `65,500/65,536` (`99.945068%`).
- Assignment-disjoint development reached `7,137/8,192` (`87.121582%`), best `7,162/8,192` (`87.426758%`) at epoch 53.
- Frozen verdict: `OPEN_MQAR_NOT_LEARNED`.
- The result is plain causal-softmax evidence only; coherent R4, geometric attribution, English, generation, reasoning, and lowering remain unrun.

The documentation also records the algebraic boundary for UOR-Framework W8: categorical role/table/lowering operations are valid mod-256 targets, while continuous softmax normalization remains compiler-side. It corrects the earlier characterization of this experiment: #1045 borrowed Zoology's curriculum/optimizer shape but did not reproduce its released cell and loader.

The predeclared next action is a fresh issue porting the released ICLR24 Zoology causal-softmax cell and loader as a CPU integration control, not tuning this result.

## Evidence

- result CID: `blake3:d920ad7b7f373c55cb564e27b3ddb1af8949a20c432e0d7cd2b39f1f69999557`
- artifact CID: `blake3:92bb13caf71c9ef44885a9da39023d080de075118b5902b716d2ca9b0f61f611`
- zero forbidden/future/provider/teacher/sealed reads
- focused Python suite: 18 passed
- Python syntax compilation: passed
- `cargo fmt --check`: passed
- claim-wording gate: passed
- `git diff --check`: passed

Closes #1045.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

