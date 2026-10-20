---
id: m1-a8-full-dose-rerun
title: Run the full A8 dose with reach protected
milestone: M1
area: training
status: held
open_to: lab
needs_gpu: true
depends_on: [m1-a8-finite-cuda-qualification]
---

## Why it matters

Tests whether reach survives training, which is M1 bar (b), and gives the first run in which ranking can be judged with admission intact.

## What is missing

The admission-preserving A8 rerun is pre-registered and FIT-reviewed but never run. The first A8 dose hit the registered Q1 stop: Relation P_succ fell 1,351 -> 551 (below half of init) while FrozenZero held 857 at Q1 and 797 at completion. Full-dose approval and release are separate from the finite qualification.

## Done when

Paired DEV verdict recorded for the registered arms and seeds; reach kept (M1 bar (b), G-Adm) or a registered diagnostic stop classified as instrument versus mechanism; fresh slice stays sealed.

## How to start

Open these first:

- `docs/labs/a8-dose-2026-10-19/README.md`
- `docs/labs/a8-training-configs-2026-10-19/README.md`
- `docs/labs/address-cells-preflight-2026-10-19/README.md`
- `docs/labs/admission-anchor-controller-2026-10-19/README.md`
- `docs/mechanisms/address-cells.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Held** (§5.1): training stays held until the owner releases it, and a FIT review releases nothing. Preparation that runs no held compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/integration/NOW.md:26-30`](../../../docs/integration/NOW.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:14`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:32`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:88`](../../../docs/mechanisms/README.md)
- [`docs/research/open-questions.md:34`](../../../docs/research/open-questions.md)
- [`docs/start/where-we-are.md:56`](../../../docs/start/where-we-are.md)

