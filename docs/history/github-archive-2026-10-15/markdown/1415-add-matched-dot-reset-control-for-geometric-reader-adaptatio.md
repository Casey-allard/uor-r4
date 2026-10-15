# #1415 — Add matched Dot reset control for geometric reader adaptation

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T03:51:20Z
- **Updated:** 2026-10-09T13:17:16Z
- **Closed:** 2026-09-27T03:53:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1415
- **Merged:** 2026-09-27T03:53:01Z
- **Merge commit:** e7a4454209456991542d15f376762c07e09cb82e
- **Base / Head:** main / UOR-Foundation:codex/dot-reset-control-20260927
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The planned reader comparison needs a Dot arm that copies the retained learned parameters while resetting Adam and the sampling clock exactly like Lorentz and Affine. An ordinary continuation retains different optimizer history and cannot supply that control. This change extends the explicit transfer path to Dot, preserves the existing radial receipt representation, and keeps saved-child resume semantics unchanged.

A source-bound actual-parent witness now verifies all21 shared arrays, full256 predictions/NLL bits and eight generated decisions are unchanged by the reset; fresh optimizer clocks are zero. Initial Dot NLL is1.996717 on the same exposed windows where prior Lorentz/Affine startup measured2.389537/2.400457. That is initial reader disturbance, not a fitted comparison or capability qualification. The fixed1,024-update three-arm adaptation plan records useful-output comparisons, exposed evaluation scope and distinct programme decisions; fitting remains NOT_RUN until its complete prospective cost and small report/generation supplement are ready.

Validation on source998ff0b3: four focused transfer/resume checks PASS across Dot/Lorentz/Affine (55.75s tests inside104.52s command); witness example builds, and actual sealed/verified zero-update run completes in35.73s with185,073,664-byte peak RSS. Rustfmt for changed sources, diff whitespace and claim wording pass. A separate offline training-package integration compile after merged PR1410 passes on c4d1837d in82.35s and is recorded in the evidence receipt; prior model evidence keeps its actual source/executable identity. Compatibility CI names are acknowledgements, not executed tests.

Canonical state/roadmap also record complementary Claude stack ownership and the still-unresolved scope of the merged Google source review. No accepted artifact is changed. No adaptation fit, accelerator work, decoder sweep or serving promotion is included.

References #973. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

