# #1907 — A 500-step snapped fine-tune improves the served score 0.91333 → 0.89113 and turns the snap into a gain (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T13:35:34Z
- **Updated:** 2026-10-09T13:22:27Z
- **Closed:** 2026-10-08T13:37:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1907
- **Merged:** 2026-10-08T13:37:15Z
- **Merge commit:** 4d1a3a21ffed2700de496951b600b7bd2fff33cd
- **Base / Head:** main / UOR-Foundation:deepseek/snapped-finetune
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The experiment

Round 27 priced full snapped training at **~an order of magnitude of throughput** and proposed the economical form: brief fine-tune from a continuously trained checkpoint. This is that test — the **round-16 checkpoint**, 500 steps at lr 1e-4 with `transport_snap=icosian`, then `snap-evaluate` on the same held-out stream (`e5f400b0…`), 196,608 targets.

| model | fused float | **snapped (served representation)** |
|---|---:|---:|
| round 16, trained unsnapped | 0.87755 | 0.91333 — snap **costs** 0.03578 BPB |
| rounds 28–29, 500-step snapped tune | 0.91855 | **0.89113 — snap *gains* 0.02742 BPB** |

**The served-representation score improves by 0.02220 BPB**, and the sign of the snap's effect **flips**: after fine-tuning, the snapped evaluation beats the unfused float evaluation by **0.05125 nats**.

## What it means

1. **The cheap form works.** 500 steps recover more than half the transport snap's original cost, at a small fraction of full snapped training's throughput price. Recommended recipe: **train continuous, then fine-tune snapped briefly.**
2. **The snap can be a regulariser, not just a requirement.** Once weights are trained in the representation, snapping *improves* the model's nll relative to its own unfused path. The 120-icosian constraint may be buying generalisation, not merely serving compliance — worth the project's attention.
3. **The unfused score worsens (0.87755 → 0.91855) while the served score improves.** For a project whose deliverable *is* the served artifact, the snapped number is the one that tracks the deliverable, and it moved correctly.

## A measurement caution, recorded

The in-training dev line reads as a **reversal** — 1.8907 → 1.8612 (step 250) → **1.9017** (step 500) — while the authoritative 196,608-target evaluation shows improvement. They score different slices, and the in-training line is evidently not a reliable proxy at this granularity. Recorded because **a reader of the training log alone would draw the opposite conclusion — as this session briefly did**, before the evaluation landed.

## Next

- **Export the fine-tuned model** (GPTQ, 64 windows, damp 0.01 — the measured optimum) and score with `d11-evaluate` for what the artifact actually delivers.
- **Sweep the fine-tune length** (250 / 500 / 1,500 steps): the dev line and the evaluation disagree, and only the evaluation can choose.
- The full snapped run remains the **control** for whether training snapped from scratch beats a short tune into the basin.

Scope: one model, one corpus, one stream, one fine-tune length, single seed. The 500-step tune differs from its parent in **both** the extra steps and the snap — not separated here. Evidence: `docs/evidence/snapped_finetune_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

