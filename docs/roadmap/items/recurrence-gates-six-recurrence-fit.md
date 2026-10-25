---
id: recurrence-gates-six-recurrence-fit
title: Run the six-recurrence fit for the recurrence gates
milestone: research
area: mechanism
status: ready
open_to: lab
needs_gpu: true
depends_on: [float-training-module, r-secant-surrogate-descent]
notebook: docs/research/notebook/float-training.md
scope: [crates/uor-r4-float-train/**, crates/uor-r4-integer/**, docs/mechanisms/recurrence-gates.md, docs/harness/prep/recurrence-gates-six-recurrence-fit*, docs/labs/recurrence-gates-six-recurrence-fit/**, docs/loop/tasks/recurrence-gates-six-recurrence-fit/**]
---

## Why it matters

It tells whether lawful recurrence can learn and carry state, and it feeds the position hypothesis.

## What is missing

The recurrence gates use a powers-of-two fade table with errors of at most 5.5e-8, a clamped-line input gate and rotation-carried state, and pass 18 CUDA controls. The six-recurrence fit never ran. The recurrence mode is also refused by export and QAT.

## Done when

The six-recurrence fit runs after surrogate descent is shown and reports integer DEV metrics against a control.

## How to start

Open these first:

- `docs/mechanisms/recurrence-gates.md`
- `archive/labs/rung0-complete-path-2026-10-13/README.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:135`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:102`](../../../docs/mechanisms/README.md)
- [`docs/start/transformer-vs-geometry.md:36`](../../../docs/start/transformer-vs-geometry.md)

