---
id: m2-donor-corpus-generation
title: Generate the one-time offline donor answer corpus (distill-d1)
milestone: M2
area: training
status: done
open_to: lab
needs_gpu: true
depends_on: [d41-driver-math-pilot]
---

## Why it matters

It is the training data for the first simple conversation.

## What is missing

**Completed on main.** D87/D89 delivered 1,009,576,044 kept tokens in 528,615 records, with reasoning and answers stored as separate hard-text fields. Both storage copies were hash-checked. This closes corpus generation only. [Sealed receipt](https://github.com/UOR-Foundation/uor-r4/pull/4028)

**At registration**

Stage 5 trains the student on fixed example answers produced once by a donor model; these are not yet generated. D41 allows it as a one-time owner exception: the student sees only the answers, never the donor's probabilities. It runs only after the driver math optimization (scope not detailed in these sources), and its execution gate (release or PM-dispatch wording) is a separate owner/PM item. *(This sentence is superseded by D86 below.)*

**D86 (owner, 15 Nov 2026): released.** Casey: "I would say yes release it, but why is it going to cost money? if we are running it locally on wick, wouldn't it be free?"
- The donor is `openai/gpt-oss-20b` on wick, not gpt-oss-120b on a pod.
- Each prompt gets up to four hard answer texts, with no probabilities, scores or ranks.
- The driver math no longer gates the corpus. The ~1M-token pilot (`d41-driver-math-pilot`) still runs first, and this corpus is sized from it.
- The generation unit still needs its sheet with a non-author FIT (§4.5) and its receipt bound at generation.

## Done when

The distill-d1 corpus generated once under D41's conditions, hashed and recorded; no student training hold released by it.

## How to start

Open these first:

- `archive/labs/discrete-driver-cost-2026-10-13/README.md`
- `docs/integration/DECISIONS.md`
- `docs/integration/DECISIONS.md (D41)`
- `docs/integration/NOW.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Held** (§5.1): training stays held until the owner releases it, and a FIT review releases nothing. Preparation that runs no held compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/integration/NOW.md:108`](../../../docs/integration/NOW.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:113`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:140`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/start/where-were-going.md:52`](../../../docs/start/where-were-going.md)

