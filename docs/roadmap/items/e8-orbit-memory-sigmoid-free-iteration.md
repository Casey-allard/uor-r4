---
id: e8-orbit-memory-sigmoid-free-iteration
title: Build a sigmoid-free gate for E8 orbit memory and rerun it on GPU
milestone: research
area: mechanism
status: held
open_to: lab
needs_gpu: true
depends_on: [float-training-module]
---

## Why it matters

It reopens a PARKED feed-forward replacement under the project's rules.

## What is missing

E8 orbit memory replacing half the MLPs cost +0.033 BPB at about 10× fewer operations, but learned keys beat E8 by 0.023, the gate was a sigmoid, and training used 2,000 CPU steps. No sigmoid-free gate form or GPU run exists, and the architect finds no E8-memory site on the discrete path.

## Done when

A sigmoid-free gate form, with the record's named next iteration, runs on the GPU and is compared with learned keys, reporting the 0.023 gap.

## How to start

Open these first:

- `docs/mechanisms/e8-orbit-memory.md`
- `archive/labs/e8-orbit-memory-2026-10-11/README.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Held** (§5.1): training stays held until the owner releases it, and a FIT review releases nothing. Preparation that runs no held compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:17`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:62`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:112`](../../../docs/mechanisms/README.md)

