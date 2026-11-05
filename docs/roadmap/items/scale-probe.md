---
id: scale-probe
title: Measure how ability grows with width and data
milestone: research
area: training
status: ready
open_to: lab
needs_gpu: true
depends_on: [one-model-end-to-end]
notebook: docs/research/notebook/float-training.md
scope: [docs/harness/prep/scale-probe*, docs/labs/scale-probe/**, docs/roadmap/items/scale-probe.md, docs/loop/tasks/scale-probe/**, docs/model/pipeline-map.md, docs/research/notebook/float-training.md]
---

## Why it matters

The pipeline map's gap 9: "Scale caps, with one data point." Every reply result so far comes from one size of model and one corpus. Without a measured slope, nobody can say whether the architecture gets better with size, which is the question any capable model has to answer.

**What is measured:**
- Whole-table nearest top-1 is `9,328 / 10,573` at width 256 with the cell objective (claude-152), and `10,152 / 11,140` at width 512 (claude-153). Width 512 trains at about half the speed, with 8 heads.
- The corpus is 7.5M tokens (gap 9).

**The caps:**
- The readout's writer allows at most 8,192 identities and width 1,024 (pipeline map, stage 7).
- No run has measured how ability grows past these (pipeline map, stages 1, 4 and 7).

## What is missing

**A scale probe on the recipe [`one-model-end-to-end`](one-model-end-to-end.md) picks:** the same arms and decode, at more than one width and more than one corpus size, scored on prompts the model has not seen. This item waits for that recipe, so it measures the model the project will serve.

- **Widths:** 256, 512 and 1,024 (the writer's cap). A width past 1,024 needs the writer changed first, and is a separate item.
- **Data:** claude-293's real-text corpus (OASST2 and Wikipedia, about 28M TRAIN tokens) at several fractions. It is human-written text only, under the decided order's interim §3.10 reading. The templated corpus stays as one contrast cell. (This is claude-294's decision card on this item's issue. Casey can reverse it.)
- **Seeds:** at least two per cell, so a slope is not one seed's noise.

## Done when

1. **A prep sheet** `docs/harness/prep/scale-probe-<date>.md` with a fresh FIT and the D67 pre-run review, both posted on this item's issue. It registers:
   - the arms, the cost and the stop rule;
   - the panel: the real-text corpus's own held-out splits, cut by source id, with a census of their 8-token overlap with each TRAIN mix (claude-294's decision card). P-dev stays reserved for `one-model-end-to-end`'s run;
   - the bars, in advance.
2. **The runs,** sealed in the store with a SHA256SUMS that was read back. The pods are down.
3. **The REPORT** gives, per width and corpus size:
   - training tokens per second, and served decode tokens per second against context on the exported artifact;
   - the panel's reply scores, with paired bands across seeds;
   - the fitted slope, and whether it is distinguishable from flat.

   It also has one notebook row in `float-training.md`, and the pipeline map's gap 9 records the slope.

## How to start

- **Read:** claude-152's and claude-153's REPORTs (the width-256 and width-512 fits); `one-model-end-to-end`'s protocol sheet (`docs/harness/prep/one-model-end-to-end-protocol-2026-10-29.md`) for the arms and the panel; the pipeline map's stages 1, 4 and 7.
- **Throughput first:** a short probe projects each arm's wall time before the real run (the probe ladder, `run-claude` part 4).
- **Before claiming:** `docs-status-refresh`'s scope covers `docs/model/*.md`, the pipeline map included, and `one-model-end-to-end`'s covers `float-training.md`. Check that no open order holds either.

## Rules that apply

- **GPU-only research compute** (CHARTER §3.5), inside the caps: at most 4 pods and $8/h across all labs.
- **Geometry only, no softmax, no matmul** (§3.1–§3.3).
- **No transformer baselines or neural graders:** deterministic exact checks only (§3.10, §4.4).
- **A negative closes a configuration, never a mechanism** (§4.1). A flat slope at these sizes closes these sizes.

## Sources

- `docs/model/pipeline-map.md`, gap 9 and stages 1, 4 and 7.
- [claude-152](../../loop/tasks/in-space-trunk-to-text/claude-152/REPORT.md) and [claude-153](../../loop/tasks/in-space-trunk-to-text/claude-153/REPORT.md).
