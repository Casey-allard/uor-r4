---
id: native-geometric-reregister-hinge
title: Re-register the native geometric engine with the hinge loss
milestone: research
area: training
status: held
open_to: lab
needs_gpu: true
depends_on: []
---

## Why it matters

It reopens a PARKED whole-model engine under the current loss.

## What is missing

The native geometric engine answered authored questions only and wrote no prose, and 437/512 fell to 0/128 on fresh rows, a result the synthesis attributes to panel fitting. It has not been re-registered with the rank-margin hinge, and it waits for the M2 hold to lift.

## Done when

A pre-registered hinge-loss sheet passes FIT and, once the hold lifts, runs once with fresh-row results reported.

## How to start

Open these first:

- `docs/mechanisms/native-geometric.md`
- `archive/labs/m2-fresh-437-2026-10-10/README.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Held** (§5.1): training stays held until the owner releases it, and a FIT review releases nothing. Preparation that runs no held compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:64`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:121`](../../../docs/mechanisms/README.md)

