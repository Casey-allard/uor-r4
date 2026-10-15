# #1051 — A1Q-H9: execute the copied Zoology control under the measured CPU wall

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-02T09:24:20Z
- **Updated:** 2026-10-09T14:17:08Z
- **Closed:** 2026-09-02T09:26:16Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1051
- **Merged:** 2026-09-02T09:26:16Z
- **Merge commit:** 7c0c66e5e69d8523101b0ec6088e5a254b0c93fa
- **Base / Head:** main / UOR-Foundation:issue-1049-zoology-measured-wall
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #1049. References #973 and #1050.

## Outcome

Executed the unchanged, credited Zoology width-64 one-head causal-softmax control after widening only the frozen CPU wall from 900 to 1200 seconds.

- C0: PASS — literal-source loader/model goldens, deterministic initialization, causal/query-only parity, and disposable 32-row `128/128` overfit.
- CPU plan: all 8 host cores, batch 64, Apple Accelerate/OpenMP; CUDA and MPS forbidden.
- C1: 64 epochs / 2,097,152 query presentations / 195.201318 seconds.
- Construction: `32,758/32,768` (`99.969482%`).
- Best development: `999/4,096` (`24.389648%`).
- Final development: `980/4,096` (`23.925781%`).
- Verdict: `SCALED_SOURCE_CALIBRATION_MISS`.
- C2 and binding permutation: `NOT_RUN_C1_MISS`.
- Result CID: `blake3:9b36540d81d0967a3f7e2ccabed80900d31c904b6c747d9ba0d539b325b13373`.

A post-result read-only diagnosis found `4,089/4,096` predictions among the four values present in each row, but key-specific assignment remained at four-choice chance. This is positive copied causal-attention mechanics at C0 and a negative scaled binding result at C1, not an R4/geometric-attention verdict.

#1050 freezes the only next copied-control decision: reproduce the pinned executable release at its exact 100,000/3,000 rows, batch 512, and staged four-rate sweep. Stop at the first strict `>99%` positive; only that may return the programme to exact UOR bytes.

## Verification

- Fresh-process create-once result verification: PASS.
- Focused Zoology lifecycle tests: 25/25 PASS.
- Python compile check: PASS.
- `cargo fmt --check`: PASS.
- claim-wording gate: PASS.
- `git diff --check`: PASS.
- independent final review: PASS.

`data.py` and `model.py` remain byte-identical to protected #1047/main. Modulo-256 remains reserved for discrete roles, wheel/table operations, and later lowering; it is not used as the probability domain for this offline softmax control.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

