---
id: r-secant-surrogate-descent
title: Check whether a secant surrogate gives descent for recurrence gates
milestone: research
area: training
status: done
open_to: lab
needs_gpu: true
depends_on: [float-training-module]
notebook: docs/research/notebook/float-training.md
scope: [crates/uor-r4-float-train/**, docs/mechanisms/recurrence-gates.md, docs/mechanisms/spiralcore-fbs.md, docs/harness/prep/r-secant-surrogate-descent*, docs/labs/r-secant-surrogate-descent/**, docs/loop/tasks/r-secant-surrogate-descent/**]
---

## Why it matters

Cheap gate before funding any float-trained trunk.

## What is missing

It is untested whether a secant surrogate gives descent for the recurrence gates and for SpiralCore/FBS. This is a small float check meant to precede any larger run; float is legal in a separate scanned module for offline training only.

## Done when

A recorded descent or no-descent result for each of the recurrence gates and SpiralCore/FBS, with instrument versus mechanism classified.

## How to start

Open these first:

- `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/recurrence-gates.json`
- `docs/mechanisms/recurrence-gates.md`
- `docs/mechanisms/spiralcore-fbs.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:40`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:102`](../../../docs/mechanisms/README.md)
- [`docs/mechanisms/README.md:103`](../../../docs/mechanisms/README.md)
- [`docs/research/open-questions.md:46`](../../../docs/research/open-questions.md)

