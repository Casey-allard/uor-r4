---
id: fbs-cost-profile
title: Profile one training step and one served token, stage by stage
milestone: research
area: training
status: ready
open_to: lab
needs_gpu: true
depends_on: []
notebook: docs/research/notebook/float-training.md
scope: [crates/uor-r4-float-train/src/in_space_long.rs, crates/uor-r4-float-train/src/bin/train-in-space.rs, crates/uor-r4-float-train/src/geometric_stack_native.rs, crates/uor-r4-integer/src/bin/fbs-toks.rs, docs/harness/prep/fbs-cost-profile*, docs/loop/tasks/fbs-cost-profile/**, docs/research/notebook/float-training.md, docs/roadmap/items/fbs-cost-profile.md]
---

## Why it matters

The owner, 28 Oct 2026, in claude's chat, asking for the review that led to the pivot (the message's first clause):

> "ok can we review the entire mechanism one more time for what exactly is costing us speed and compute time throughout the whole cycle"

And after the pivot vote:

> "I don't care what we serve at long as it is cheap"

[The pipeline map](../../model/pipeline-map.md) ("The scale arithmetic") says: "No profile says where the time goes. This is the largest serving question at scale." Every cost figure since is end-to-end tokens/s, plus arithmetic on shapes.

## What is missing

- **Training:** the runs give one rate per arm, with no split by stage. Measured on one RTX 5090 at width 256 ([float-training notebook](../../research/notebook/float-training.md), rows of 27–28 Oct):
  - `32,049` context tokens/s with 4 Hamming heads on the pinned TRAIN, and `36,040` with no read on `balp`;
  - `12,573` with the learned register, `28,560` with the bag;
  - at width 512, `38,578` with no read and `17,033` with 8 heads.

  What no run shows:
  - the share spent in the six-layer recurrence, which steps through 192 positions in sequence, forward and backward (suspected launch-bound, untested);
  - the share spent in the cell objective's whole-table nearest-wrong-token search (about `1,048,576` differences per scored row, by arithmetic);
  - the share spent in the reads.
- **Serving:** `fbs-toks` on the M1 decodes at 231–268 tok/s for 1.1M values, which the map estimates at 5,000 to 6,000 times below the chip's bandwidth limit. There is no split between the trunk, the readout (writer or whole table) and the loop.
- **The test runner's decode** re-runs the whole window for every token, so its tokens/s is the runner's, not the model's.

## Done when

1. **One training step, profiled by stage** on an RTX 5090, at claude-152's (a) recipe (cell objective, width 256, 4 Hamming heads), and at width 512 with no read:
   - wall-clock per stage: embedding, each layer's recurrence and feed-forward, each read, the objective's table search, the backward, the optimizer;
   - from CUDA events or the driver's profiler;
   - summing to the measured step time within 10%;
   - with the top three named.
2. **One served token, profiled by stage** on the M1 in `fbs-toks`: each trunk layer, the readout (writer and whole-table nearest, each timed), and the loop overhead, summing within 10%.
3. For each of the top three costs, one line on the cheapest change that would remove most of it, with its effect on prediction stated as measured or as a hypothesis.
4. One notebook row and a REPORT. A one-page sheet with a fresh FIT comes before the GPU part (CHARTER §4.5), because it runs training steps.
