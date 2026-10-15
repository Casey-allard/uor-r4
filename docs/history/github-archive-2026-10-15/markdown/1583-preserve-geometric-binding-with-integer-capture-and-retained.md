# #1583 — Preserve geometric binding with integer capture and retained state

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T20:36:30Z
- **Updated:** 2026-10-09T13:17:09Z
- **Closed:** 2026-10-01T20:37:53Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1583
- **Merged:** 2026-10-01T20:37:53Z
- **Merge commit:** c1dce3f385dcef2b06bf0fce70523bc2631d3ec8
- **Base / Head:** main / UOR-Foundation:codex/attention-integer-latch-20261001
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The successful geometric identity reader still used a floating capture gate and retained vector. This adds a checked four-bit shift/add gate and int16 captured state with separately retained dyadic scales, then feeds that actual state through the saved reader's q/k identity maps.

Both Held seeds preserve 128/128 answers and correct head-zero source majorities on both original and longer-gap development panels. All four saved models have zero gate-action, answer-prediction or source-majority changes relative to their hard references. No fitting occurred. This is integer gate/state fidelity; surrounding projections, Lorentz reader and output remain floating point. Constant Held input scale limits the model-panel scope; unequal scales are covered by component checks. Debug disassembly is a scoped observation, not strict optimized opcode qualification.

Validation: four focused integer arithmetic/state tests and ten focused stack tests passed with `--profile dev`; touched Rust rustfmt, diff check and claim-wording passed. Evaluation took 172.63 seconds wall time, two threads, 183,664,640 bytes maximum RSS. Exact source/executable/artifact identities and row-level results are preserved in the report. Independent mathematical review recomputed rows and artifact binding. No optimized speed or full-language claim.

Next: scale-aware current-role and identity q/k projections, combining wide contributions before rounding while retaining magnitude and orientation.

References #1512.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5940073820 — 2026-10-01T20:37:00Z — Casey-allard

Exact-head self-review: fe5e7b13ff509b20010d8de50aea0e796f969094, tree dc5bb289408318647c0f6e25f6e9e13302979aff. All four measured source hashes still match; final head adds documentation only after source commit 918c5009. Actual compile and four integer + ten stack tests completed on these unchanged source bytes; rustfmt/diff/claim-wording passed. Reviewed causal old-state ordering, separate input/weight scales, exact checked alignment, transactional errors, replay seam/refusals, exporter/reload binding, and report scope. Independent mathematical specialist recomputed raw outputs/artifact binding. No additional fitting, general-language, full-serving or opcode qualification is claimed. The 66 fmov transfers and constant Held-panel exponent are explicit limitations. Approve protected delivery of this scoped component. Source and tests are authored/reviewed in this lab; self-review allowed by D19. Retain all successful/negative parents and advance q/k lowering.



---
_Comment/review ids in this file: 5940073820_

