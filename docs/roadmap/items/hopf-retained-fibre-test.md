---
id: hopf-retained-fibre-test
title: Run the Hopf retained-fibre language test
milestone: research
area: mechanism
status: held
open_to: lab
needs_gpu: true
depends_on: []
---

## Why it matters

It is the reopen condition for a PARKED attention-scoring replacement.

## What is missing

Hopf observation is telemetry only. Sector routing used 7 of 512 sectors, and the kept-fibre map is built and unit-tested, but the language test with the retained fibre was never run.

## Done when

A retained-fibre test is run on the integer DEV metrics against a control and recorded.

## How to start

Open these first:

- `docs/mechanisms/hopf-observation.md`
- `docs/labs/course-correction-2026-10-18/audits/mechanism-census.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Held** (§5.1): training stays held until the owner releases it, and a FIT review releases nothing. Preparation that runs no held compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/mechanisms/README.md:116`](../../../docs/mechanisms/README.md)
- [`docs/mechanisms/README.md:156`](../../../docs/mechanisms/README.md)

