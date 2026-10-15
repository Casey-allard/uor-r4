# #1584 — Preserve geometric binding through paired integer q/k projections

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T20:59:55Z
- **Updated:** 2026-10-09T13:17:10Z
- **Closed:** 2026-10-01T21:01:51Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1584
- **Merged:** 2026-10-01T21:01:50Z
- **Merge commit:** fe174f1195c453f1e4353669962c818aa12cf1c2
- **Base / Head:** main / UOR-Foundation:codex/attention-integer-qk-20261001
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The successful geometric reader still projected current-role and captured identity through floating q/k maps. This adds validated four-bit paired projections that retain independent dyadic scales, accumulate both maps exactly, and round once to Q16. It avoids the existing grouped dot's per-group rounding.

Actual integer gate/state/qk replay preserves both Held seeds at 128/128 answers and correct head-zero source majorities on original and longer-gap authored development panels. No Held predictions or source majorities change; equal-value source pairs remain correct. Read-output ablation drops answers to 4/6 and 8/14. Local controls' rowwise gains/losses are preserved. No fitting or answer-tuned quantizer change occurred.

Validation: five focused integer tests, one projection replay/reference test, and ten retained stack tests passed in dev profile; rustfmt, diff check and claim-wording passed. Initial fixture type compile failure was repaired and preserved. Independent architecture review verified source/algebra/causality and recomputed raw outputs with model/data/component/source/executable hashes. Evaluation: 114.623s process wall, two threads, maximum RSS237,191,168bytes, zero updates.

Remaining trunk/value/Lorentz/head computations stay float. This is q/k boundary behavioral fidelity on fixed development panels, not full serving, language, indexed access, opcode or energy qualification. Next lower Lorentz lift/score while preserving radius/orientation and NoRead behavior.

References #1512.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5940443414 — 2026-10-01T21:00:20Z — Casey-allard

Exact-head self-review: `48732a151fd7c2a2c9d01e348ddb88d59d528cf2`, tree `646a754bede98a641c5d4f16bc19e0d12a52b91c`. All five measured source hashes still match; subsequent commits add documentation only. Actual dev compile, five paired integer tests, one independent-projection replay test and ten retained stack tests completed on these source bytes. Formatting/diff/claim-wording passed. Source model/data/component/embedded source/executable identities verified independently; raw integer-latch references reproduce prior outputs. Independent architecture reviewer authored none of current kernel/seam/evaluator and reviewed source plus raw rows. Root approves scoped behavioral fidelity, with Local losses/gains, float remaining computations and authored development limitations retained. No opcode/performance/full-serving qualification. Self-review permitted by D19; protected delivery proceeds at this exact head. Next substantive boundary Lorentz lift/score, no repeat q/k fit or scale sweep.



---
_Comment/review ids in this file: 5940443414_

