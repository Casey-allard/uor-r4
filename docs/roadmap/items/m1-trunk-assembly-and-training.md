---
id: m1-trunk-assembly-and-training
title: Assemble and train the full lawful trunk (Stage 3)
milestone: M2
area: architecture
status: ready
open_to: lab
needs_gpu: true
depends_on: [m1-position-mechanism, m1-normalization-mechanism, m1-bvg-k-stage-t-training, m1-d11-exporter-discrete]
scope: [crates/uor-r4-discrete/**, docs/harness/prep/m1-trunk-assembly-and-training*, docs/labs/m1-trunk-assembly-and-training/**, docs/loop/tasks/m1-trunk-assembly-and-training/**]
---

## Why it matters

Supplies the missing parts, which are the main route to beating the count tables.

## What is missing

Build a trunk with a feed-forward layer (the Hamilton gate), a recurrence with quaternion rotation, and the Flock and Relation reads; none has been trained together and the supporting evidence is weak, so this is a hypothesis test. Scale and data are open: no trained stack has used the 1.39-billion-token chat corpus (UOR-Foundation/uor-r4#1660). A 24-kernel extract has passed finite controls only.

## Done when

Stage 3 bar: 'beat Stage 2's best by more than the seed-to-seed noise, with a fresh-panel number.'

## How to start

Open these first:

- `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/geometric-stack.json`
- `docs/mechanisms/flock-read.md`
- `docs/mechanisms/geometric-stack.md`
- `docs/mechanisms/hamilton-gate.md`
- `docs/mechanisms/recurrence-gates.md`
- `docs/mechanisms/relation-read.md`
- `docs/model/architecture.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:130-132`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:160`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:53`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/research/open-questions.md:59`](../../../docs/research/open-questions.md)
- [`docs/start/transformer-vs-geometry.md:35`](../../../docs/start/transformer-vs-geometry.md)
- [`docs/start/where-we-are.md:60`](../../../docs/start/where-we-are.md)
- [`docs/start/where-were-going.md:48`](../../../docs/start/where-were-going.md)

