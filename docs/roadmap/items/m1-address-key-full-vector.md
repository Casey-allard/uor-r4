---
id: m1-address-key-full-vector
title: Replace the 5-value address key with a full-vector metric
milestone: M1
area: mechanism
status: done
open_to: lab
needs_gpu: true
depends_on: [m1-w2-capacity-capture]
scope: [crates/uor-r4-discrete/**, docs/mechanisms/discrete-learning.md, docs/harness/prep/m1-address-key-full-vector*, docs/labs/m1-address-key-full-vector/**]
---

## Why it matters

This tests the leading hypothesis for the low top-1 and is Stage 1, "make the capacity measurable". It is the first step of the recovery plan for the discrete-attention line. See [discrete learning](../../mechanisms/discrete-learning.md) and [where we are](../../start/where-we-are.md).

## What is missing

The address key uses only the first and last block and takes only 5 values (0, 4, 8, 12, 16) to order up to 256 cells (proof, read from `crates/uor-r4-discrete/src/discrete.cu`: `address_distance` at line 274, `cell_precedes` at line 301). Which cells open is therefore largely decided by arbitrary numbering. The architect's likely lever is a full-vector or interior-block address metric, built and measured at zero training. It needs new kernel code and a fresh CUDA qualification.

## Done when

Stage 1 bar: "enough distinct vectors that the number of positions still winnable clears the run bar of 1,941, and a starting reach above the address-cell model's 1,351 of 3,847", measured before any training. The audit also lists within-cell tie mass. The bar is not G-Pre of 4,894 at checkpoint 0, which every K16 arm already met.

## How to start

1. Read `docs/labs/course-correction-2026-10-18/audits/d37-reactivation-architect.md` (section 3.2 and section 6 W1), `docs/labs/address-cells-preflight-2026-10-19/README.md` and `docs/mechanisms/discrete-learning.md`.
2. Open the reusable host code in `crates/uor-r4-discrete/src/`: `address_cells.rs`, `coverage_census.rs`, `attention_preflight.rs`, and the kernel in `crates/uor-r4-discrete/src/discrete.cu`. This item waits on [m1-w2-capacity-capture](m1-w2-capacity-capture.md) for the same measurement setup.
3. First deliverable, posted on the issue: a design note. It gives the metric (full vector or interior block), why it uses no softmax and no matmul, and finite CPU controls on `address_cells.rs` (distinct-vector count, alias ceiling, tie mass on a fixed fixture).
4. Build the host version first and run `~/.cargo/bin/cargo test -p uor-r4-discrete --lib address_cells` (no feature is needed; the build script skips nvcc unless `--features cuda` is set). Add the kernel change only after the host numbers look worth it; the kernel needs a stub-nvcc `--features cuda` type check (as in `docs/labs/bvgk-c2-block-apply-2026-10-18/README.md`) and a fresh CUDA qualification.
5. Write the one-page pre-registration for the zero-update GPU pre-flight on the A8 placement. Freeze and hash the panel, give the Stage 1 bar above, and get a non-author FIT before any GPU run. A FIT review releases nothing: if `docs/integration/NOW.md` lists a hold on this run, only Casey types the owner release, and no lab types it.
6. Run it on the GPU through `scripts/pod/uor-pod` (lease your GPU first) and save results off the pod.
7. Open a PR that says `Closes #<this issue>` (or `References` if partial), and add one row to `docs/research/notebook/m1-discrete-attention.md`.


## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:120-123`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:155`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:36`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/model/architecture.md:86`](../../../docs/model/architecture.md)
- [`docs/research/open-questions.md:42`](../../../docs/research/open-questions.md)
- [`docs/start/where-we-are.md:53`](../../../docs/start/where-we-are.md)
- [`docs/start/where-were-going.md:44`](../../../docs/start/where-were-going.md)

