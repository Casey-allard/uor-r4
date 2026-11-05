---
id: writer-admission
title: Make the native writer's admission hold at every seed
milestone: research
area: mechanism
status: ready
open_to: lab
needs_gpu: true
depends_on: [one-decoder]
notebook: docs/research/notebook/float-training.md
scope: [crates/uor-r4-float-train/src/native_vector_writer.rs, crates/uor-r4-float-train/src/in_space_long.rs, crates/uor-r4-float-train/src/bin/train-in-space.rs, crates/uor-r4-float-train/src/hamilton_sheet.rs, docs/harness/prep/writer-admission*, docs/loop/tasks/writer-admission/**, docs/research/notebook/float-training.md, docs/roadmap/items/writer-admission.md]
---

## Why it matters

The owner, 28 Oct 2026, in claude's chat: "I don't care what we serve at long as it is cheap". This is the [pipeline map](../../model/pipeline-map.md)'s R2 and its gap 2.

The decision is to serve whole-table nearest, because the writer failed at one seed of two ([claude-150](../../loop/tasks/one-decoder/claude-150/REPORT.md)). On the M1 that costs 486 µs a token, 21 times the writer's 23 µs ([claude-158](../../loop/tasks/fbs-cost-profile/claude-158/REPORT.md)), and it grows with the vocabulary. A writer that holds at every seed makes the cheap readout the correct one.

## What is missing

- **claude-150's evidence** (its REPORT and the float-training notebook's row): with the `voronoi` objective, the writer held at seed 1 (`8,430` top-1 at step 32,768, against nearest's `6,762`). At seed 2 it never learned (`493` against `6,909`). There, the target's cell was admitted on only `208`–`714` of 14,287 rows from step 4,096, against `8,558`–`9,563` at seed 1.
- **The diagnosis (done-when 1, [claude-289](../../loop/tasks/writer-admission/claude-289/REPORT.md)): seed 2's ports went flat.**
  - From step 4,096, both of seed 2's ports sit at the roots' centre, where all 16 roots tie: their mean |x| falls to `5.4e-7`, and every row's 256 cell scores lie within `1e-4`.
  - Admission is then a near-tie lottery, about chance (8 of 256 cells).
  - The trunk learned anyway (nearest `6,909`). Seed 1's ports point at the target's roots instead (class L).
  - Both seeds start alike, and they split before step 2,048.
  - So done-when 2's change must be one the tie point cannot satisfy. A margin alone is not expected to leave it (an inference, untested). The REPORT's Still open, item 1, cautions that a symmetric pull toward the target's root may share the centre as its minimum while the port is uninformed.
- **The change (done-when 2, [claude-290](../../loop/tasks/writer-admission/claude-290/REPORT.md)): anchored ports, `train-in-space --writer-anchor 1`, off by default.**
  - The shared table's two port blocks hold exactly each token's own cell roots. They are set at model build and after every main step, with their gradient masked to 0, in both training loops. The other columns stay learned.
  - A token's own vector then addresses its own cell first at score 0, so the recipe's own `cell` or `nearest` objective would train admission (argued from the code, untested in training).
  - Four CUDA tests and three host tests pass on a pod, and both negative gates fail as required. No run beyond a 32-step smoke has used it, and no training result exists.
  - **What done-when 3's sheet must carry:**
    - the query ports' mean |x| before step 2,048, with a stop rule, since the centre stays a fixed point in expectation for an uninformed trunk and the anchor fixes only the table side;
    - what the anchor costs nearest, since the table is also the input embedding and a cell's 16 tokens share their port code;
    - claude-289's `admission` reading, run again, with that REPORT's instrument limits (its Still open, item 5).
  - **Held in reserve:** excluding the port-producing weights from decay (over claude-150's schedule decay alone scales them about 1.9×, against the ports' 40× fall), and a kept count chosen by the margin.
- **Main refuses checkpoints saved before `bcf2f57f7`** (29 Oct): their projection fingerprint pins `spiralcore_fbs.cu`, whose backward changed. A reading of claude-150's checkpoints is built on `074d54b2`, as claude-289's evidence branch was.
- **The writer** ([`native_vector_writer.rs`](../../../crates/uor-r4-float-train/src/native_vector_writer.rs), `address_scores`) keeps 8 of 256 cells. A cell is a pair of roots: the query's first and last 8 coordinates are each scored against 16 SpiralCore roots, by mean absolute distance. A token outside the kept cells cannot be decoded.
  This is the float-train crate's copy, the one `train-in-space` builds against (`crates/uor-r4-float-train/src/lib.rs`). `crates/uor-r4-training` holds another copy, which is not in scope.
- **The writer has not been read under the cell objective,** which the current recipe (claude-152's (a)) trains for whole-table nearest. The one earlier disagreement, `345` against nearest's `7,813` (the map, stage 7), came from a checkpoint trained with the writer's own `MarginHinge { rivals: 8 }` objective, whose margin could not be met (the float-training notebook's 27 Oct "Exploration" row).

## Done when

1. **A diagnosis of seed 2's failure,** from saved checkpoints: the ports' scores against the roots, the cells admitted, and how they move over training. The first unit may be this alone, on the sealed claude-150 checkpoints.
2. **A geometric change to admission,** for example a margin the 16 roots' spacing allows, cells kept by the target's own ports, or a kept count chosen by the margin. It ships with host tests, and with CUDA tests where tensors are involved.
3. **A pre-registered run** (a sheet with a fresh FIT, CHARTER §4.5): claude-152's (a) recipe plus the change, two seeds, the writer and whole-table nearest read on the same checkpoints.
   - **Bar:** the writer within a pre-registered margin of nearest at both seeds.
   - **The cost per token** of each decoder, measured.
4. One notebook row and a REPORT.

## How to start

Read:
- [claude-150's REPORT](../../loop/tasks/one-decoder/claude-150/REPORT.md) and the map's stage 7;
- `native_vector_writer.rs` and the `voronoi` and cell objectives in `in_space_long.rs`;
- the writer's `MarginHinge` in `hamilton_sheet.rs`.

claude-150's results are sealed under `caseyallard/uor-r4-store:results/claude/` (87 files per seed; its REPORT names the folder). Check with `uor-store ls` that the checkpoints the diagnosis needs are among them before planning on them.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): admission is fixed with the project's geometry (roots, cells, ports), never by a softmax-like or squashing score. No dense matrix multiplication.
- **A negative closes this configuration, never the mechanism** (§4.1). Classify instrument against mechanism first (§4.2).

## Sources

- The owner's words, 28 Oct 2026, in claude's chat (quoted above).
- [claude-150](../../loop/tasks/one-decoder/claude-150/REPORT.md), [claude-158](../../loop/tasks/fbs-cost-profile/claude-158/REPORT.md) and the [pipeline map](../../model/pipeline-map.md).
