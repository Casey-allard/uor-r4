---
id: r-full-epoch-revisit-schedules
title: Test full-epoch and frequent-token revisit schedules
milestone: research
area: training
status: held
open_to: lab
needs_gpu: true
depends_on: []
scope: [docs/harness/prep/r-full-epoch-revisit-schedules*, docs/labs/r-full-epoch-revisit-schedules/**, docs/loop/tasks/r-full-epoch-revisit-schedules/**]
---

**November 17, 2026 · retired from the schedule under D68.** The canonical 16,404-batch dose covers 1,049,856 of 2,425,925 targets (43.28%) by arithmetic. Stage T separately attempted 3,008 batches per frozen-zero seed and 4,096 per relation seed. Its full-epoch and frequent-token revisit arms were never compared. P1 under #4063 measures a native count stream. Reopen the schedule comparison if a chosen learner uses those updates and target coverage would decide its schedule. [November queue decision](../../loop/tasks/infra-loop-harness/codex-10/DECISION.md) · [Written-state experiment](https://github.com/UOR-Foundation/uor-r4/issues/4063) · [antigravity-66](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/loop/tasks/r-full-epoch-revisit-schedules/antigravity-66/REPORT.md) · [Stage T attempted batches](https://github.com/UOR-Foundation/uor-r4/blob/11c2901d90af122c53069aff62191bc4f147ff9a/docs/labs/m1-bvg-k-stage-t-training/README.md#L80)

### Historical registration and evidence

The original rationale, acceptance criteria and work instructions follow.

## Why it matters

Checks whether the low top-1 is partly a coverage problem.

## What is missing

**November 17, 2026.** The historical one-pass schedule visited 43.28% of targets. The registered revisit implementation and controlled comparison remain open. Stage T has concluded. The next prerequisite is an applicable native training schedule on which this measurement would change a decision. [antigravity-66 report](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/loop/tasks/r-full-epoch-revisit-schedules/antigravity-66/REPORT.md) · [Issue #2836](https://github.com/UOR-Foundation/uor-r4/issues/2836)

One full run sees 1,049,856 of 2,425,925 training targets, 43.3% (proof, by arithmetic). Full-epoch and frequent-token revisit schedules have never been run.

## Done when

Schedules that cover all training targets are run under a registered sheet and compared with the one-pass baseline.

## How to start

Open these first:

- `docs/integration/NOW.md`
- `docs/labs/k16-outcome-call-2026-10-19/README.md`
- `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/discrete-learning.json`
- `docs/mechanisms/discrete-learning.md`
- `docs/model/training.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:21`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:47`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/research/open-questions.md:53`](../../../docs/research/open-questions.md)
- [`docs/start/where-we-are.md:54`](../../../docs/start/where-we-are.md)

