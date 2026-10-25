---
id: r-rank-objective-keeps-cells
title: Design a rank objective that keeps cells occupied
milestone: research
area: training
status: held
open_to: lab
needs_gpu: true
depends_on: [m1-a8-full-dose-rerun]
scope: [docs/harness/prep/r-rank-objective-keeps-cells*, docs/labs/r-rank-objective-keeps-cells/**, docs/loop/tasks/r-rank-objective-keeps-cells/**]
---

## Why it matters

Gives a way to raise reach without collapsing into a few cells.

## What is missing

The registered route after the K16-R run, which collapsed to 19 or 20 occupied cells, has not been tried. A rank objective must hold occupancy above the G-Occ floor while still ranking.

## Done when

A rank objective that passes G-Occ (more than 16 of 256 cells, plus the D39 condition) and is evaluated against the count tables.

## How to start

Open these first:

- `crates/uor-r4-discrete/src/discrete.cu`
- `docs/labs/k16-outcome-call-2026-10-19/README.md`
- `docs/mechanisms/rank-margin.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:20`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:46`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:93`](../../../docs/mechanisms/README.md)
- [`docs/research/open-questions.md:52`](../../../docs/research/open-questions.md)
- [`docs/start/where-we-are.md:49`](../../../docs/start/where-we-are.md)

