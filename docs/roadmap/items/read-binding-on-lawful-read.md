---
id: read-binding-on-lawful-read
title: Port read binding to a Flock or Relation read
milestone: research
area: mechanism
status: ready
open_to: lab
needs_gpu: true
depends_on: []
scope: [docs/harness/prep/read-binding-on-lawful-read*, docs/labs/read-binding-on-lawful-read/**, docs/loop/tasks/read-binding-on-lawful-read/**]
---

## Why it matters

It would give the lawful reads a training signal that worked on a softmax read.

## What is missing

Read binding (a teacher marks which earlier value the question asks for and the read is rewarded for landing on it) cut wrong-value errors from 47 to 30 of 120 over 3 seeds, but on a softmax read. It has not been ported to a Flock or Relation read.

## Done when

Read binding trained on a Flock or Relation read reports wrong-value errors on the same 120-item panel against the unbound read over 3 seeds.

## How to start

Open these first:

- `docs/mechanisms/read-binding.md`
- `archive/labs/read-binding-powered-2026-10-11/README.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:65`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:115`](../../../docs/mechanisms/README.md)

