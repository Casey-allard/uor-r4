---
id: r-finite-relational-head
title: Train the finite relational head
milestone: research
area: training
status: done
open_to: lab
needs_gpu: true
depends_on: [m1-a8-full-dose-rerun, m1-bvg-k-stage-t-training]
notebook: docs/research/notebook/m1-discrete-attention.md
scope: [crates/uor-r4-discrete/**, docs/harness/prep/r-finite-relational-head*, docs/labs/r-finite-relational-head/**, docs/loop/tasks/r-finite-relational-head/**, docs/research/notebook/m1-discrete-attention.md]
---

## Why it matters

The relation read is one of the trunk's planned reads.

## What is missing

K16-A reached its 16,404 batches and can satisfy the prerequisite for the next stage (architect review section 5), but the finite relational head has not been trained. The Relation read passes 9 of 9 controls.

**Preparation done; released now that `#2836` is `done` (`antigravity-95`).** `antigravity-65` (#3210) pre-registered `docs/harness/prep/r-finite-relational-head-2026-10-25.md` (`TEST FITNESS: FIT`) and held the item until `m1-bvg-k-stage-t-training` (`#2836`) completed and `crates/uor-r4-discrete/**` was added to its scope. With `#2836` (`#3239`) `done` and `crates/uor-r4-discrete/**` free, this item is `ready` to run after a D67 pre-run review.

## Done when

A registered run of the finite relational head with admission and ranking reported against the count tables.

## How to start

Open these first:

- `docs/labs/a8-dose-2026-10-19/README.md`
- `docs/labs/course-correction-2026-10-18/audits/d37-reactivation-architect.md`
- `docs/labs/k16-outcome-call-2026-10-19/README.md`
- `docs/mechanisms/relation-read.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:48`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:91`](../../../docs/mechanisms/README.md)
- [`docs/research/open-questions.md:54`](../../../docs/research/open-questions.md)
- [`docs/start/transformer-vs-geometry.md:32`](../../../docs/start/transformer-vs-geometry.md)
- [`docs/start/where-we-are.md:29`](../../../docs/start/where-we-are.md)

