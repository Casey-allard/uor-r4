---
id: one-decoder
title: Make the decoder that serves the decoder that was measured
milestone: research
area: training
status: done
open_to: lab
needs_gpu: true
depends_on: [float-training-module]
notebook: docs/research/notebook/float-training.md
scope: [crates/uor-r4-float-train/src/in_space_long.rs, crates/uor-r4-float-train/src/bin/train-in-space.rs, crates/uor-r4-float-train/src/native_vector_writer.rs, docs/harness/prep/one-decoder*, docs/loop/tasks/one-decoder/**, docs/research/notebook/float-training.md, docs/roadmap/items/one-decoder.md]
---

## Why it matters

This is unit U2 of the [pipeline map](../../model/pipeline-map.md)'s order of work, its "first new unit". The map's stage 7 (readout) finds two decoders that disagree on the same checkpoint:
- **the writer** (`native_vector_writer.rs`): 256 cells, 8 kept, the nearest token inside them. The serving session decodes with it.
- **whole-table nearest** (`nearest_scores`, `in_space_long.rs`): every float run is evaluated with it.

On one checkpoint the writer's top-1 is 345 and whole-table nearest's is 7,813, at lr 1e-3 ([float-training](../../research/notebook/float-training.md), as the map's stage 7 cites it). So every served number so far is measured on a decoder that is not the one served. The writer's address hinge at margin 1 cannot be met, because its 16 roots are at least 0.5 apart. The `voronoi` objective sets the margin to 0.25, half that separation, and gave the best writer-decoded score, 7,906 at 16,384 steps. But "the long writer run collapsed only after step 16,384, so whether the margin prevents that collapse is untested" ([claude-142 REPORT](../../loop/tasks/in-space-trunk-to-text/claude-142/REPORT.md)).

The map puts this first because "nothing downstream means anything until the prompt reaches the reply through a decoder that is the one served".

## What is missing

- No run trains with `voronoi` past step 16,384, where the writer collapsed.
- No run reads the writer and whole-table nearest on the same checkpoints across a whole schedule.
- No record says which decoder serving uses, and why.

## Done when

1. A pre-registered run (its own prep sheet, FIT and pre-run review, CHARTER §4.5) trains the in-space trunk with `--objective voronoi`, past step 16,384 (for example 32,768 steps), at two seeds. Every checkpoint reads both decoders' top-1 and openings on DEV.
2. The sheet's bar decides one of two outcomes, and the REPORT records the decision:
   - **the writer holds** (no collapse past step 16,384, and within the sheet's margin of whole-table nearest): the writer stays the served decoder;
   - **it does not:** the decision is to serve whole-table nearest. The REPORT gives its per-token cost at 4,096 tokens (1,048,576 absolute differences) and the point at which a sublinear index is needed. Changing the serving session (`stack/session.rs`, `fbs-toks`) is then its own unit; this item's scope is the run and the decision.
3. One notebook row; the result on main.

## Cancel if

A merged unit already serves the decoder the float runs are evaluated with, or a run on main already reads both decoders past step 16,384 with `voronoi`.

## How to start

Read:
- the map's stages 7 (readout) and 8 (decode loop);
- [claude-142's REPORT](../../loop/tasks/in-space-trunk-to-text/claude-142/REPORT.md), on the `voronoi` arm and the writer's collapse;
- `voronoi_loss` and `nearest_scores` in `in_space_long.rs`, and `native_vector_writer.rs`.

The runner already has `--objective voronoi`; the work is the run, the reading of both decoders at each checkpoint, and the decision.

## Rules that apply

- **Geometry only, no softmax or squashing, no dense products** (CHARTER §3.1–§3.3). Both decoders score by negative mean L1 distance to table points; keep it that way.
- **GPU-only research compute** (§3.5); a pre-registration before the run (§4.5).
- **Lead with tokens/s** (decode against context), then the decoders' agreement.

## Sources

- [The pipeline map](../../model/pipeline-map.md), stage 7 and unit U2.
- [claude-142 REPORT](../../loop/tasks/in-space-trunk-to-text/claude-142/REPORT.md).
- [float-training notebook](../../research/notebook/float-training.md).
