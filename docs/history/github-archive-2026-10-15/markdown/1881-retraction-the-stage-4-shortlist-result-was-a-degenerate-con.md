# #1881 — RETRACTION: the Stage 4 shortlist result was a degenerate constant shift, not learning (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T07:18:43Z
- **Updated:** 2026-10-09T13:21:02Z
- **Closed:** 2026-10-08T07:20:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1881
- **Merged:** 2026-10-08T07:20:23Z
- **Merge commit:** 46243489666e8b9ef92891b92536cc09ee5174a1
- **Base / Head:** main / UOR-Foundation:deepseek/lut4-retraction
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Retraction

PR #1880 reported that a learned LUT-4 term re-ranking the top-64 shortlist improved held-out bits per byte on real text (baseline 4.211070 → hard 4.191505, −0.019565), supported by a uniform-boost control that lost 0.048235 BPB. **That claim is wrong and this retracts it.**

## What the artifact actually emits

Instrumenting the served output directly — now recorded on every harness run:

```
chosen_gates           [12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 0]
distinct_hard_outputs  1
distinct_soft_outputs  1   (to milli-precision)
hard_bit_tally         [4096, 0]    every candidate receives bit 0
```

Gate 12 is the pass-through gate, gate 0 is FALSE: **the term is the constant 0 for every input.** The correction was therefore a uniform **penalty** of `−scale/2` on the shortlist and nothing else.

A uniform shortlist penalty lowers BPB here for a reason unrelated to learning: the target lies outside the top-64 for **80 %** of held-out rows, so moving mass off the shortlist is usually right. The boost control pointed the other way only because it moves mass the wrong way — a fixed logit shift does not produce a mirror-image softmax response, so the two constants are not symmetric and **the control did not isolate learning**. My reading of it was mistaken.

## Corrected status

**The Stage 4 gate is NOT passed.** No measurement in this session shows a learned nonlinear term improving held-out BPB on real text. The first form moved BPB by +0.0000061 with the best of all 1,056 two-input terms; this form moved it by a constant shift that the degeneracy makes indistinguishable from a fixed shortlist penalty.

**Root cause:** training drives the final gate to a constant and the first layer to the pass-through gate. With balanced 0/1 labels the mixture's least-squares optimum at the initial wiring is a constant, and the only mechanism that could change *which* features a gate reads is the connectivity utility step — far too weak at `0.01 × rate / batch` to escape in 400 steps. The plan's "learnable connectivity (not fixed random)" requirement is consequently **still unmet** in this implementation.

**And the "cone defect" I listed as a bug was not a bug.** `cone_sizes()` correctly reported 0. I treated a correct degeneracy signal as faulty accounting. The accounting was right; the training is what is broken.

## What the harness now guards

- `artifact_output_diagnostic` (hard-bit tally, distinct hard and soft output counts, chosen gates) on every run.
- A `degenerate_term` flag plus a printed warning when the artifact has one distinct output, so a constant term can no longer be reported as an improvement.
- The uniform-boost control stays, but it is necessary and not sufficient; the degeneracy check is the load-bearing one.

## Next

1. Implement genuine connectivity learning (straight-through or score-based selection at a real gradient scale) and **assert non-degeneracy before measuring**: at least two distinct outputs on the training shortlists, and a gate distribution that is not a single gate.
2. Re-run the held-out measurement with a paired bootstrap interval, reporting learned-minus-uniform-shift rather than learned alone.

Evidence: `docs/evidence/native_geometric_lut4_shortlist_retraction_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

