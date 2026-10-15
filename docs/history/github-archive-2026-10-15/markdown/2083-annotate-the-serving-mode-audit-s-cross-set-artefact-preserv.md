# #2083 — Annotate the serving-mode audit's cross-set artefact, preserving the dated line (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T20:16:57Z
- **Updated:** 2026-10-09T20:20:07Z
- **Closed:** 2026-10-09T20:18:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2083
- **Merged:** 2026-10-09T20:18:31Z
- **Merge commit:** 36322f36533e24a97f05a26b39e60c997560d45e
- **Base / Head:** main / UOR-Foundation:deepseek/audit-annotation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Annotate the serving-mode audit's cross-set artefact, preserving the dated line (#2083)

bpb-split flagged the 0.0464 BPB 'un-attributed integer' line in the dated record
docs/evidence/serving_mode_audit_2026-10-08.txt and, correctly, declined to rewrite a dated record
or silently edit another owner's claim. This annotates it instead: the original line stays exactly as
written and a dated correction note follows it.

The figure is a CROSS-POSITION-SET SUBTRACTION ARTEFACT, not serving arithmetic. Re-measured on
IDENTICAL positions with lut-evaluate reference=true: at 64 windows / 24,576 targets float 1.796884,
reference 1.800973, integer 1.800973 BPB (quantisation 0.004089, engine -0.00000077 BPB); at 512
windows / 196,608 targets float 1.767493, reference 1.771344, integer 1.771342 (quantisation 0.003841,
engine -0.0000043 nats); d11-evaluate agrees exactly with d11 = d10, d11_minus_d10 0.0, top-1 1.0,
max_abs_logit_difference 0. The serving path is within 0.4% of its own float model.

What is large is the MEASUREMENT's position sensitivity - per-window float spread 0.50 BPB at 64
windows, 64-window mean standard error 0.063 BPB, and the two window sets differing by 0.029 BPB on
the SAME model - so a BPB comparison across window counts is not interpretable without naming the
window count and the stream's own byte basis.

No live claim needed changing: the current-state entry from #2081 already carries this correction
explicitly, above the older round-37 wording that it supersedes, which is what the newest-first
convention is for. This commit only makes the dated record self-consistent for a reader who arrives
at it directly.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

