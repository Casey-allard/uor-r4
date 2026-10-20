---
id: m1-w2-capacity-capture
title: Run the W2 read-code capacity capture
milestone: M1
area: evaluation
status: held
open_to: lab
needs_gpu: true
depends_on: []
---

## Why it matters

Routes all later spend: H_rep means fix the read and address metric and do not fund float training; H_law means run BVG-K (W3) and, if it fails, W4.

## What is missing

W2 is FIT-reviewed and queued on the retained lease but has not run; it needs an owner-typed release (CHARTER 5.1). It would decide whether the wall is representation (H_rep) or learning law (H_law), and it adds the unigram and admission-matched next-word count-table arms the headline bars lacked. Cost is about $0.91 (chief architect's estimate, not a recorded cost).

## Done when

The registered W2 sheet runs once with all arms; H_rep or H_law is recorded as one notebook row, and the admission-matched next-word table number (needed for the Stage 2 bar) is produced.

## How to start

Open these first:

- `docs/integration/DECISIONS.md`
- `docs/integration/NOW.md`
- `docs/labs/course-correction-2026-10-18/audits/d37-reactivation-architect.md`
- `docs/labs/k16-outcome-call-2026-10-19/README.md`
- `docs/research/open-questions.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Held** (§5.1): training stays held until the owner releases it, and a FIT review releases nothing. Preparation that runs no held compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/integration/NOW.md:65`](../../../docs/integration/NOW.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:110`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:115-118`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:31`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/research/open-questions.md:33`](../../../docs/research/open-questions.md)
- [`docs/start/where-were-going.md:42`](../../../docs/start/where-were-going.md)

