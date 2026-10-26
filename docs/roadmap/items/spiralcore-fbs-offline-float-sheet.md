---
id: spiralcore-fbs-offline-float-sheet
title: Measure SpiralCore/FBS descent and tokens per second in a float sheet
milestone: research
area: mechanism
status: done
open_to: lab
needs_gpu: true
depends_on: [float-training-module, r-secant-surrogate-descent]
notebook: docs/research/notebook/float-training.md
scope: [crates/uor-r4-float-train/**, docs/mechanisms/spiralcore-fbs.md, docs/harness/prep/spiralcore-fbs-offline-float-sheet*, docs/labs/spiralcore-fbs-offline-float-sheet/**, docs/loop/tasks/spiralcore-fbs-offline-float-sheet/**]
---

## Why it matters

It tests the matmul replacement for projections, and its speed, which no record has measured.

## What is missing

SpiralCore/FBS passes exact fixtures on an RTX 5090 but has never been trained outside its integer form, which runs inside discrete learning. Neither descent nor tokens per second has been measured for the offline-float form.

## Done when

Descent and tokens per second are reported from an offline-float sheet with a matched control.

## How to start

Open these first:

- `docs/mechanisms/spiralcore-fbs.md`
- `archive/labs/spiralcore-fbs-projection-2026-10-12/README.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:17`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:103`](../../../docs/mechanisms/README.md)
- [`docs/start/transformer-vs-geometry.md:71`](../../../docs/start/transformer-vs-geometry.md)

