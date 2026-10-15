# #2072 — Correction: the pointer line's mechanism is softmax-based and cannot be served (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T18:34:40Z
- **Updated:** 2026-10-09T18:37:50Z
- **Closed:** 2026-10-09T18:36:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2072
- **Merged:** 2026-10-09T18:36:10Z
- **Merge commit:** e9bc3ba438cdea3c6a848540eeda0894a4922ea7
- **Base / Head:** main / UOR-Foundation:deepseek/softmax-correction
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Recorded at the owner's request after a session review, and it corrects my own framing rather than any measurement. The pointer's attention is a softmax over sources (geometric_stack.rs:88-90, :8463, :1053) and the control layer is RoPE softmax attention (:27). Geometry is in the SCORE (lorentz, prime routes); the normalisation and mixture are not geometric. D11 forbids float and transcendentals in served kernels so exp/log-sum-exp cannot be served at all, and export refuses a pointer with an identity term for want of an integer port - so the line is offline-only, which the entries say, but they present it in the same register as the serving work. A capability gain on an unservable mechanism is not progress toward the served model. Also narrows the recorded VSA negative to the fixed token-id-hash codebook implementation it actually measured, rather than the geometry as a family. Unchanged: the served chat stack at 0.93277 BPB, D11 vs D10 exact on 16,384 targets, integer vs float +0.0077 nats, 0.8802 top-1. References #2029

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

