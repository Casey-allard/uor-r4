---
id: m1-w4-float-credit-sheet
title: Register and run one float-credit training sheet (W4)
milestone: M1
area: training
status: ready
open_to: lab
needs_gpu: true
depends_on: [m1-w2-capacity-capture, m1-bvg-k-stage-t-training]
scope: [crates/uor-r4-discrete/**, docs/harness/prep/m1-w4-float-credit-sheet*, docs/labs/m1-w4-float-credit-sheet/**, docs/loop/tasks/m1-w4-float-credit-sheet/**]
---

## Why it matters

Last resort in Stage 2 if no discrete law learns to rank.

## What is missing

W4 is conditional: only if W2 = H_law and W3 (BVG-K) fails. It is one decimal-training run in a separate scanned module (float is legal in offline training only, D37), about $5 to $10 GPU plus build work, with a disqualifying export-gap rule fixed per sheet at FIT time (D39 item 5). M1 can only be closed by a whole-number-trained model, so the owner must first redefine the milestone if a float trunk is used.

## Done when

Sheet FIT'd and run once; integer top-1 through the D11 export reported against both count tables; export gap within the two-seed band on the same evaluator.

## How to start

Open these first:

- `docs/integration/DECISIONS.md`
- `docs/labs/course-correction-2026-10-18/audits/d37-reactivation-architect.md`
- `docs/mechanisms/geometric-stack.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/integration/NOW.md:67`](../../../docs/integration/NOW.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:112`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:125-127`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:164-165`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/research/open-questions.md:95`](../../../docs/research/open-questions.md)
- [`docs/start/where-were-going.md:46`](../../../docs/start/where-were-going.md)

