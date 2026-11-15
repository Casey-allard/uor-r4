---
id: d41-driver-math-pilot
title: Optimize the driver math and run the ~1M-token pilot (D41 item 3)
milestone: M2
area: training
status: ready
open_to: lab
needs_gpu: true
depends_on: []
scope: [docs/harness/prep/d41-driver-math-pilot*, docs/labs/d41-driver-math-pilot/**, docs/loop/tasks/d41-driver-math-pilot/**]
---

## Why it matters

It gates distill-d1 supplying hard targets to the D32 discrete student.

## What is missing

D41 item 3 puts the driver math optimization and a ~1M-token pilot ahead of distillation. Neither is recorded as done in these documents, and distill-d1 does not exist yet.

**D86 (owner, 15 Nov 2026).** The driver math optimization no longer gates the donor pass ("Drop driver math, keep pilot (Recommended)"). This item's remaining part is the measured ~1M-token pilot. It runs once on `openai/gpt-oss-20b` on wick, the owner's RTX 4090, with up to four hard answer texts per prompt (owner terms). The lab's conditions: a pinned revision with the released MXFP4 weights, outside every compiled module of the repository, the receipt bound at generation (D41 item 2b), and a sheet with a non-author FIT (§4.5). D86 is the §3.10 release that the pilot unit cites at dispatch.

## Done when

Since D86: the ~1M-token pilot has run on gpt-oss-20b on wick under D41's conditions as amended by D86, with its receipt and its measured tokens/s and size recorded. The original text follows.

The driver math optimization lands with parity controls, and the ~1M-token pilot is run and recorded.

## How to start

Open these first:

- `docs/integration/DECISIONS.md`
- `docs/integration/NOW.md`
- `crates/uor-r4-discrete/src/discrete.cu`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:113`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)

