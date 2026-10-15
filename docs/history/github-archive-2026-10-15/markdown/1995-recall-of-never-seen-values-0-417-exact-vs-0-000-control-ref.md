# #1995 — Recall of never-seen values: 0.417 exact vs 0.000 control (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T03:55:13Z
- **Updated:** 2026-10-09T13:19:40Z
- **Closed:** 2026-10-09T03:56:50Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1995
- **Merged:** 2026-10-09T03:56:50Z
- **Merge commit:** 40e1039b8256e3ee0d615aba4a05549b6a2b5653
- **Base / Head:** main / UOR-Foundation:deepseek/copy-rate
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Inference only on the best binding arm (pointer=32, dot, no gate supervision) and its control, over 28 invented values absent from every store (verified by decoding ~9.2e8 token positions across 19 stores plus chat train and heldout - 28/28 zero occurrences), three conditions, EOS-bearing exact path. EXACT copy: pointer 35/84 = 0.417 Wilson [0.317, 0.523]; per value 16/28 = 0.571 in at least one condition, 24/28 = 0.857 emit somewhere; contains 55/84 = 0.655. NO-POINTER CONTROL: 0/84 exact and 0/84 contains in all three conditions. The gate is near-necessary and not sufficient: pointer mass >= 0.5 gives 0.493 exact while < 0.5 gives 0/13 = 0.000, and a 0.9 cut buys +3 points; DELTA at or above 0.7 holds in 81/84 cells so that split is degenerate. The dominant residual failure is the correct value followed by over-generation (Jorvex name is Jorvex., Lunavar is Lunavar., Nuvrix is Nuvrixxx), 20 of 49 misses. Another row value is emitted in zero cells and a store-family first word in only 2/84 against 18/84 in the control. So a learned copy pathway recalls never-seen values against a control that never emits them: binding is not architecturally impossible here, and the residual failure is the continuation after a correct copy. Next fix: a length-constrained or copy-terminating decoder, or a loss stopping the response once the copied span completes. Not a solved capability. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

