---
id: training-step-cost
title: Split the training backward, and run a cheaper rival search against (a)
milestone: research
area: training
status: ready
open_to: lab
needs_gpu: true
depends_on: [fbs-cost-profile]
notebook: docs/research/notebook/float-training.md
scope: [crates/uor-r4-float-train/src/in_space_long.rs, crates/uor-r4-float-train/src/bin/train-in-space.rs, crates/uor-r4-float-train/src/native_vector_writer.rs, crates/uor-r4-float-train/src/geometric_stack_native.rs, crates/uor-r4-float-train/src/geometric_stack_native_cuda.rs, crates/uor-r4-float-train/src/hamilton_sheet.rs, docs/harness/prep/training-step-cost*, docs/loop/tasks/training-step-cost/**, docs/research/notebook/float-training.md, docs/roadmap/items/training-step-cost.md]
---

## Why it matters

The owner, 28 Oct 2026, in claude's chat (the message's first clause, recorded in [`fbs-cost-profile`](fbs-cost-profile.md)): "ok can we review the entire mechanism one more time for what exactly is costing us speed and compute time throughout the whole cycle". This is the [pipeline map](../../model/pipeline-map.md)'s R4 and its gap 4.

[claude-157](../../loop/tasks/fbs-cost-profile/claude-157/REPORT.md) split one training step on an RTX 5090:
- **(a), 96.5 ms:** the backward is 62% (60.4 ms), the cell objective with its whole-table search 14% (13.1 ms), and the trunk's forward 5% (4.8 ms).
- **Width 512 with no read, 79.0 ms:** the backward is 54% (42.9 ms) and the objective 34% (26.7 ms).

## What is missing

- **The backward's split.** Its host time is 0.98–0.99 of its GPU time. That fits launches with the GPU keeping pace, or the host waiting at a sync inside it; claude-157 could not tell which. Which forward stage's gradient dominates is not measured.
- **A cheaper rival search.** The search is GPU-bound and grows with the table (4,096 × width). A search over a subset, for example the writer's kept cells, or a search every few steps with the rivals kept, has not been run. Its effect on prediction is a hypothesis.

## Done when

1. **The backward split:** a kernel-level profile of the profiled step with the driver's profiler (`nsys`) on an RTX 5090, at (a) and at width 512 with no read. Kernels are grouped by the forward stage they serve, and the top three are named.
2. **A cheaper rival search,** behind a flag that leaves every file a run writes unchanged when off, with host and CUDA tests.
3. **A pre-registered run** (a sheet with a fresh FIT, CHARTER §4.5): (a) with and without the cheaper search, two seeds.
   - **Bar:** the step's time falls by a pre-registered amount, and top-1, openings and copies stay within margin of (a). Optionally the same with claude-161's turn memory on.
4. One notebook row and a REPORT.

## How to start

Read:
- [claude-157's REPORT](../../loop/tasks/fbs-cost-profile/claude-157/REPORT.md);
- `nearest_loss` and `best_positions` in `in_space_long.rs`;
- `train-in-space --profile K`.

The pod bootstrap may lack `nsys`; check for it first, and install it from NVIDIA's packages if needed.

## Rules that apply

- **Geometry only, no squashing** (CHARTER §3.1–§3.2): the rival choice stays a nearest-wrong-token hinge. No softmax or normalized exponential.
- **No dense matrix multiplication** (§3.3).
- **A cost change is barred on prediction too:** a faster step that loses fit is reported as such.

## Sources

- The owner's words, 28 Oct 2026, in claude's chat (quoted above).
- [claude-157](../../loop/tasks/fbs-cost-profile/claude-157/REPORT.md) and the [pipeline map](../../model/pipeline-map.md).
