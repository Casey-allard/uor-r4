# #1035 — #973: record direct retained-readout partial result

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-01T16:22:00Z
- **Updated:** 2026-10-09T14:16:56Z
- **Closed:** 2026-09-01T16:23:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1035
- **Merged:** 2026-09-01T16:23:33Z
- **Merge commit:** d36c16b8217dae7eb68cf8d32483d4f786ceb2d1
- **Base / Head:** main / UOR-Foundation:issue-973-readout-seam
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Implements and executes the independently frozen #973 prompt-state-to-logit readout rung, then preserves its sealed evidence.

`R4DirectRetainedReadoutLanguagePathV1` keeps qualified V1 recurrence, parameters, state, data/order, seed, optimizer dose, and one tied vocabulary matmul fixed. It changes only the final readout to include the already-computed retained layer residuals.

Terminal: `DIRECT_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`

- prompt gain: `0.0215897894` versus V1 `0.0076304198`
- candidate-minus-V1 gain: `0.0139593696`
- directional wins: `343/512`
- fresh held-out NLL: `3.7374367989` versus V1 `3.9010778353`
- fresh held-out top-1: `31.542433%` versus V1 `29.632946%`
- state-off cost: `1.1234286047` nats and `20,179` correct decisions
- result CID: `blake3:71dd85e610dcc50b74cb2bb2068e5a1a433ac5df5db2a4f8fde22fb41735889c`
- independent verifier CID: `blake3:b8ad3b6fa6d6ab9e429b3bd8d2a5060215d15230cd272e7272f27b7eef54785b`

Wins, language, causal, state-off, forbidden-read, replay, and independent-verification gates passed. The frozen absolute and incremental prompt-gain floors did not, so this is PARTIAL rather than PASS.

## Consequence

Freeze the artifact. Do not generate, retry, widen, lower, or tune gain after reveal. The sole fresh successor is the zero-parameter layerwise-normalized readout recorded in the roadmap; a miss ends the parameter-free readout ladder and pivots to learned associative binding/readout.

This does not establish prompt-coherent generation, reasoning, H4 superiority, exact/table runtime, browser readiness, or release readiness. #973 remains open and #954 remains blocked.

## Verification

- 35 focused `unittest` cases passed
- 90 independent provenance/arithmetic/mode checks passed with zero discrepancies
- 13/13 fresh-process terminal comparisons reproduced exactly
- seven canonical raw envelopes are byte-identical to the sealed outputs
- claim-wording gate passed
- `git diff --check` passed

References #973


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

