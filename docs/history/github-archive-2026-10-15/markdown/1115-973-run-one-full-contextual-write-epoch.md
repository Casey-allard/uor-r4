# #1115 — #973: Run one full contextual-write epoch

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-04T14:33:38Z
- **Updated:** 2026-09-04T14:36:03Z
- **Closed:** 2026-09-04T14:34:56Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1115
- **Merged:** 2026-09-04T14:34:56Z
- **Merge commit:** 0704f8a24d8181b81b7eacdbad09b6c11c49ff4a
- **Base / Head:** main / UOR-Foundation:codex/973-contextual-full-fit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The 128-update contextual adaptation exercised only 2,048 of 43,680 training windows, so it could not distinguish an undertrained write path from a structurally inadequate one. This adds a dedicated `fit-contextual-retained-full` command that runs the existing contextual-write model over the complete deterministic training order once.

The command is fixed in source: 2,730 updates, batch size 16, four CPU threads, the original retained V1 initialization, and a 2,700-second hard wall. It has no update/thread/wall override and writes to `arms/contextual-retained-full`, preserving the earlier bounded artifact and historical V1 artifact.

Measured result from the sole authorized run:

- 2,730/2,730 updates; all 43,680 windows and 5,241,600 causal targets consumed once
- 1,919.399615 seconds on four-thread CPU, 780.600385 seconds below the hard wall
- all 24 parameter tensors had finite, nonzero gradients on the first update
- optimizer included all 252,160 parameter values
- first/last 16-batch mean training loss: 3.903525 / 3.551273; this is training behavior, not held-out evaluation
- artifact CID: `blake3:f1cecf1bc70d239470ad2e27112ee47e9f32899118cba9ab82686b7e85050617`
- validation, sealed, teacher, and source-model reads: zero

The two predeclared direct CLI generations from that artifact were:

- `A purple turtle found a clock in the garden` → `. She loved to play. He saw a big tree in her pocket. She`
- `Albert Einstein was born in` → ` the water.\nTim took the kitchen and saw a small girl playing in the`

The full fit improves local sentence form relative to the bounded artifact, but prompt-specific factual binding remains inadequate: Einstein is still completed with a false location and the continuation drifts to unrelated text. This establishes one completed contextual-write epoch and source-free prompt-dependent generation. It does not establish factuality, coherence, reasoning, general chat, or held-out quality. No retry, second epoch, validation run, tuning pass, or architecture change occurred.

Checks were limited to `python3 -m py_compile`, `git diff --check`, the one full fit, and the two named direct generations. The protected merge queue remains the repository gate.

References #973


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

