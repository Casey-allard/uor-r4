---
id: hamilton-gate-offline-float-sheet
title: Run a Hamilton gate offline-float sheet with language fit and export gap
milestone: research
area: mechanism
status: ready
open_to: lab
needs_gpu: true
depends_on: [float-training-module, r-secant-surrogate-descent]
scope: [docs/harness/prep/hamilton-gate-offline-float-sheet*, docs/labs/hamilton-gate-offline-float-sheet/**, docs/loop/tasks/hamilton-gate-offline-float-sheet/**]
---

## Why it matters

It would give the first language-fit evidence for any lawful feed-forward or gate part.

## What is missing

The Hamilton gate has 944 tests, GPU parity of 43 passed and 0 failed, and 18 CUDA controls, but it has never been trained. It still needs the icosian snap before export. The claim that a norm-preserving gate can suppress a signal is only a dossier reading.

## Done when

An offline-float (D37) sheet reports a language fit on the integer DEV metrics and an export gap, and states whether the gate suppressed signal.

## How to start

Open these first:

- `docs/mechanisms/hamilton-gate.md`
- `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/hamilton-gate.json`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:135`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:168`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:17`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:101`](../../../docs/mechanisms/README.md)

