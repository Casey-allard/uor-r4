---
id: r-ranking-with-admission-intact
title: Test ranking with admission intact
milestone: research
area: training
status: held
open_to: lab
needs_gpu: true
depends_on: [m1-a8-full-dose-rerun]
scope: [docs/harness/prep/r-ranking-with-admission-intact*, docs/labs/r-ranking-with-admission-intact/**, docs/loop/tasks/r-ranking-with-admission-intact/**]
---

## Why it matters

Separates the admission failure from the ranking failure behind the low top-1.

## What is missing

No run has had admission intact and then failed or succeeded at ranking, so ranking is untested on its own. The A8 rerun with the admission anchor is the natural vehicle.

## Done when

A run with admission demonstrably intact reports ranking results (top-1 among admitted rows) against the count tables.

## How to start

Open these first:

- `docs/labs/k16-outcome-call-2026-10-19/README.md`
- `docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md`
- `docs/mechanisms/admission.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:45`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/model/architecture.md:84`](../../../docs/model/architecture.md)
- [`docs/research/open-questions.md:51`](../../../docs/research/open-questions.md)

