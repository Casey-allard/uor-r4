---
id: r-vsa-hamming-binding
title: Test discrete Hamming binding (VSA) on the current writer
milestone: research
area: mechanism
status: held
open_to: lab
needs_gpu: true
depends_on: []
---

## Why it matters

A possible lawful binding mechanism for memory and relations.

## What is missing

Discrete Hamming binding on the current writer has never been tested; the earlier VSA test used the banned CPU learner and is a configuration failure only.

## Done when

A registered GPU run of Hamming binding on the current writer, with a result compared against the count tables.

## How to start

Open these first:

- `archive/labs/vsa-native-test-2026-10-09/README.md`
- `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/vsa.json`
- `docs/mechanisms/vsa.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Held** (§5.1): training stays held until the owner releases it, and a FIT review releases nothing. Preparation that runs no held compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:22`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:49`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:66`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:117`](../../../docs/mechanisms/README.md)
- [`docs/research/open-questions.md:55`](../../../docs/research/open-questions.md)
- [`docs/research/open-questions.md:74`](../../../docs/research/open-questions.md)

