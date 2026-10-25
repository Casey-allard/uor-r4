---
id: m1-bvg-k-stage-t-training
title: Train the BVG-K batched-coordinate law on two seeds
milestone: M1
area: training
status: ready
open_to: lab
needs_gpu: true
depends_on: [m1-w2-capacity-capture, m1-bvg-k-block-apply-c2]
scope: [crates/uor-r4-discrete/**, docs/harness/prep/m1-bvg-k-stage-t-training*, docs/labs/m1-bvg-k-stage-t-training/**, docs/loop/tasks/m1-bvg-k-stage-t-training/**]
---

## Why it matters

Stage 2, 'pick the learning law': the first learning-law candidate that stays whole-number.

## What is missing

Stage T of W3 was held for the K16 outcome, which is now recorded (no KEEP), and for an owner release. Estimated at about $12 for two seeds. It is only warranted if W2 blames the learning law (H_law).

## Done when

Stage 2 bar: 'beat the unigram table, then the admission-matched next-word table (the version W2 produces, not the 6,300 full-panel table), on two seeds.'

## How to start

Open these first:

- `docs/integration/NOW.md`
- `docs/labs/bvgk-rowsk-scoring-2026-10-18`
- `docs/labs/injective-preflight-and-c2-qualify-2026-10-19/README.md`
- `docs/mechanisms/discrete-learning.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/integration/NOW.md:66`](../../../docs/integration/NOW.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:110`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:125-127`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:21`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/research/open-questions.md:5-13`](../../../docs/research/open-questions.md)
- [`docs/start/where-were-going.md:46`](../../../docs/start/where-were-going.md)

