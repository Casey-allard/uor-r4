# #1908 — The snapped fine-tune improves the float model and degrades the served artifact (0.93277 → 0.95150) (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T13:48:14Z
- **Updated:** 2026-10-09T13:22:23Z
- **Closed:** 2026-10-08T13:49:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1908
- **Merged:** 2026-10-08T13:49:53Z
- **Merge commit:** ebcea18507108ef53d137959c887c01f0e21c27a
- **Base / Head:** main / UOR-Foundation:deepseek/snap-ft-served
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What was measured

Round 29 showed a 500-step snapped fine-tune improving the **float** served-representation score 0.91333 → 0.89113 BPB, with the snap turning from a cost into a gain. This asks the only question that matters for the deliverable: **what does the exported, multiplier-free artifact score?**

Both artifacts: GPTQ, 64 calibration windows, damp 0.01 (the round-23 optimum), then `d11-evaluate` on the same held-out stream, 24,576 targets.

| model | float snapped | **served (D11)** | quantisation cost |
|---|---:|---:|---:|
| round 16, trained unsnapped | 0.91333 | **0.93277** | 0.01944 BPB |
| 500-step snapped fine-tune | 0.89113 | **0.95150** | **0.06034 BPB** |

**The served artifact regressed by 0.01868 BPB.**

## Why, and why it matters

The float-side gain is real, but the **quantisation loss grew 3.1×** (0.0382 → 0.1187 nats). Snapped fine-tuning makes the weights **less amenable to the GPTQ grid**, and that extra loss more than consumes the 0.02220 BPB the fine-tune bought on the float side.

This is the **second time tonight a float-side improvement failed to survive to the served artifact** — and a sharper instance than the first, because here the float number moved in the *right* direction and the deliverable still regressed. The procedural lesson, recorded in the evidence: **the served number is the result; a float improvement is a hypothesis about it, not evidence of it.**

It also reopens round 20's attribution with a concrete question: if snapped training changed the weight matrices' distribution, `rounding-attribution` on this artifact says **which groups grew**, and a per-group bit allocation becomes the test of whether snapped training can be served profitably.

## What stands

- **Round 29's float-side result stands as measured** — the fine-tune does move the model into the snapped basin and the snap becomes a gain there.
- Round 30 doesn't contradict it; it shows the gain doesn't transfer through this quantiser.
- **Round 16's served artifact at 0.93277 remains the best served number measured on real held-out chat**, and the **4.70×** improvement over the 4.3885 currently served is unchanged.

## Next

1. `rounding-attribution` on the fine-tuned artifact — which parameter groups absorbed the tripled loss.
2. If it's the weight matrices again, **per-group bit allocation** guided by that attribution.
3. **Sweep fine-tune length** — a shorter tune may buy the basin without the quantisation damage.

Scope: one model, one corpus, one stream, one quantiser setting, one fine-tune length, single seed. `d10` is null because a snapped artifact is scored against its float side. Evidence: `docs/evidence/snapped_finetune_served_regression_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

