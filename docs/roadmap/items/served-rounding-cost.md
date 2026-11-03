---
id: served-rounding-cost
title: Bring training as served back inside the 2x operational gate
milestone: M4
area: serving
status: ready
open_to: lab
needs_gpu: true
depends_on: []
notebook: docs/research/notebook/m4-serving.md
done_class: gpu
scope: [crates/uor-r4-integer/src/fbs_snapshot.rs, crates/uor-r4-float-train/src/fbs_export.rs, crates/uor-r4-float-train/src/hamilton_sheet.rs, crates/uor-r4-float-train/src/in_space_long.rs, docs/harness/prep/served-rounding-cost*, docs/loop/tasks/served-rounding-cost/**, docs/research/notebook/m4-serving.md, docs/roadmap/items/served-rounding-cost.md]
---

## Why it matters

Training through the exported serving graph is the shortest route from a float
training update to the deterministic integer artifact that actually ships. The
registered four-run comparison proved that path is finite and export-consistent,
but served training took 6.83-6.96x plain training against a 2x ceiling. A
longer language run on that cost would spend most of its budget reproducing the
serving graph rather than learning.

The immediate target is not a smaller model or a wider time limit. It is to
remove host round-trips and repeated scalar work from the served forward while
preserving every exported value bit-for-bit.

## What is missing

The failed comparison is
[`one-model-end-to-end-served-comparison-2026-11-02.md`](../../harness/prep/one-model-end-to-end-served-comparison-2026-11-02.md).
It did not profile the slowdown. A host-side probe now points to the export
rounding path:

- `grid_code` tests 2,032 grid candidates for every rounded gain or convolution
  tap, on the host, on every served training step.
- The nine dense matrices are also copied to the host and back on every step.
- Replacing the 49-pass activation-scale loop preserves exact forward and
  straight-through gradients, but saved only about 2% of one full CPU
  activation in the bounded probe. It is not yet a proven dominant cause.

The probe is diagnostic evidence only. It is not a model result and does not
stand in for GPU profiling.

## Done when

1. A GPU stage profile measures table copy/rounding, the served forward, its
   backward, and restore, rather than inferring their cost from the four-run
   total.
2. Each repaired operation is bounded and bit-identical to the current export:
   every grid code and adjacent value, every activation threshold and adjacent
   value, and the straight-through gradient at every in-range and saturated
   boundary.
3. The repaired configuration passes the original four-job comparison unchanged
   at the same 256 steps. Both seeds must pass the instrument door and complete
   served training in at most 2x plain training; the registered two-seed
   quality-smoke rule must not find served materially below plain. A still-slow
   or materially lower-quality rerun records a negative and leaves this item
   open.
4. The dossier includes source and binary hashes, every data pin, per-stage
   timings, checkpoint hashes, and the sealed artifact checksum.

## Cancel if

The served graph's cost is not reproducible at the pinned source and data, or a
repair changes any exported value. Reclassify the run as an instrument result
and never trade determinism for speed.

## How to start

Read the failed comparison and the `train-as-served` wiring first. Profile the
existing path before changing it. On the host, prove any quantizer replacement
bit-exact across every code, threshold and adjacent value. Then run the existing
GPU stage profiler under a pre-registration and use its readings to choose the
next repair. Do not treat the host microbench as a substitute for the GPU
profile, and do not rerun the four jobs until the repaired graph is exact.

## Rules that apply

- Geometry only; no softmax, squashing, dense matrix multiplication or GEMM
  (CHARTER sections 3.1-3.3).
- Serving remains deterministic integer; float is offline training machinery
  (sections 3.6-3.7).
- GPU remains the qualification authority (section 3.5).
- An exact kernel repair is not a capability result; the four-run comparison
  remains the operational and quality evidence.

## Sources

- [Served comparison result](../../loop/tasks/one-model-end-to-end/deepseek-5/REPORT.md)
- [Served comparison sheet](../../harness/prep/one-model-end-to-end-served-comparison-2026-11-02.md)
- `crates/uor-r4-integer/src/fbs_snapshot.rs`, `grid_code`
- `crates/uor-r4-float-train/src/hamilton_sheet.rs`, `activation_quantize16`
- `crates/uor-r4-float-train/src/fbs_export.rs`, `round_tables_as_exported`
