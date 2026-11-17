---
id: m2-train-grounded-replies
title: Train the student for grounded replies from exact memory
milestone: M2
area: training
status: ready
open_to: lab
needs_gpu: true
depends_on: [m2-donor-corpus-generation, m2-fresh-heldout-panel, m2-pin-train-mask-sheet, m1-close-saved-model, m1-d11-exporter-discrete]
scope: [crates/uor-r4-discrete/**, docs/harness/prep/m2-train-grounded-replies*, docs/labs/m2-train-grounded-replies/**, docs/loop/tasks/m2-train-grounded-replies/**]
---

## Why it matters

This is the first simple conversation, Stage 5.

## What is missing

**November 17, 2026.** The D87/D89 data receipt covers 1,009,576,044 kept tokens in 528,615 records. The student still has to meet both reply bars under the registered grader, including genuinely fresh replies. The written-state P3 experiment is one proposed step toward that result. [PR #4028](https://github.com/UOR-Foundation/uor-r4/pull/4028) · [PR #4046](https://github.com/UOR-Foundation/uor-r4/pull/4046) · [Issue #2030](https://github.com/UOR-Foundation/uor-r4/issues/2030)

Replies must be grounded in the exact memory and the object store, with relation, action and span extraction running through the shared geometric carrier and no fallback to hand-coded phrase tables or external teachers (D28). The earlier 437/512 then 0/128 result failed on configuration. M2 is in progress: the older blanket training/rental holds are retired, so training proceeds under a registered FIT and the lab's D67 pre-run review. D41's donor pass is the named owner-at-dispatch exception.

## Done when

The same native student must pass both graders. It must give complete, correct replies on at least 256/512 saved questions. On the 256 fresh questions it must score at least 52/128 opposite-role history pairs with **both** replies in each qualifying pair complete and distinct. [Frozen paired grader](../../harness/panels/m2-fresh256.json) D85 requires lossless D11 serialization and bit-identical native train/serve replay. The historical D37 float route reported an export gap.

## How to start

Open these first:

- `ROADMAP.md`
- `crates/uor-r4-core/src/native_geometric/learner`
- `archive/labs/count-reference-2026-10-13/README.md` (the record of the M2 masked-train reference)
- `docs/mechanisms/exact-memory.md`
- `docs/mechanisms/object-store.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:15`](../../../ROADMAP.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:140-141`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:22`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/start/where-were-going.md:52`](../../../docs/start/where-were-going.md)
- issue#2030

