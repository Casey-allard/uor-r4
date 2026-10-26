---
id: m1-wider-code-carrier
title: Prototype a wider code carrier (more roots or blocks)
milestone: M1
area: mechanism
status: done
open_to: lab
needs_gpu: true
depends_on: [m1-address-key-full-vector]
scope: [crates/uor-r4-discrete/**, docs/harness/prep/m1-wider-code-carrier*, docs/labs/m1-wider-code-carrier/**, docs/mechanisms/discrete-learning.md]
---

## Why it matters

It settles whether vector width, not the address key, caps how many words the addresses can separate. Each token's vector today is 4 blocks of 16 roots, which is 16 bits over a 4,096-piece (12-bit) vocabulary. This is an optional Stage 1 step, behind the cheaper full-vector address metric. See [the model page](../../model/architecture.md).

## What is missing

Whether the 16-bit carrier limits capacity is untested. The model uses 16 of the 240 E8 roots, from one orbit. Full 240-root coverage was never tried (`docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/discrete-learning.json`, never_tested). A wider code (more roots or more blocks) is optional in Stage 1. It is not cheap reuse: it needs new `build.rs` and `discrete.cu` code and a fresh CUDA qualification before any pre-flight. The sources name no design or sheet.

## Done when

A wider-carrier build passes finite CUDA qualification and a zero-training capacity measurement against the Stage 1 bar: the alias ceiling clears Bar (i) 1,941 net of admission (G-Cap), and admission at checkpoint 0 is above A8's 1,351 of 3,847 on the same panel. The synthesis also lists within-cell tie mass as a gate.

## How to start

1. Read the carrier and its measurements. Open `docs/model/architecture.md` (the token table, line 23), `docs/mechanisms/discrete-learning.md`, `docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md` (Stage 1, lines 120–123; the bar is line 122), `docs/research/open-questions.md` (line 43), `crates/uor-r4-discrete/build.rs`, `crates/uor-r4-discrete/src/discrete.cu`, `crates/uor-r4-discrete/src/address_cells.rs`, `coverage_census.rs` and `attention_preflight.rs` (the pre-flight pieces Stage 1 reuses). Coordinate with `m1-address-key-full-vector`: its metric is how a wider carrier is measured.
2. Post a design note on the issue as your first deliverable. It must say which dimension widens (more roots, up to the 240 in E8, or more blocks), the new vector width, and how the address distance, cell order and writer extend. It must state how it stays integer-only with no softmax or matmul, and how it keeps the D11 export. Propose finite CPU controls: the new table is injective over the 4,096 tokens, and a host reference of the distance agrees with the kernel on small cases.
3. Implement the host side in Rust and run `~/.cargo/bin/cargo test -p uor-r4-discrete`. Use `crates/uor-r4-discrete/src/bin/coverage-census.rs` and `address-cells-codebook.rs` to count distinct vectors and aliasing at zero updates.
4. Build the kernel changes ahead of time with nvcc through `build.rs` (NVRTC is banned; CUDA is prebuilt by nvcc, CHARTER §3.4 and `docs/model/gpu-and-cuda.md`). Write a finite CUDA qualification on the pattern of `crates/uor-r4-discrete/src/rows_qualification.rs`, `bvgk_qualification.rs` and the `rows-qualify` binary. Run it on a GPU (the CUDA code builds only with `--features cuda`; `scripts/pod/uor-pod status`, or your own GPU with hardware specs attached to the PR).
5. Draft the one-page pre-registration sheet in `docs/harness/prep/` for the zero-training capacity measurement (format model: `docs/harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md`). Use the same 3,847-row panel as A8, no updates, and the Stage 1 bar above. A non-author must give TEST FITNESS: FIT before it runs (CHARTER §4.5).
6. Open a PR that says `Closes #<this issue>` when the build, qualification and measurement are in, or `References #<this issue>` if partial. Add one row to `docs/research/notebook/m1-discrete-attention.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:121`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:37`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/model/architecture.md:23`](../../../docs/model/architecture.md)
- [`docs/research/open-questions.md:43`](../../../docs/research/open-questions.md)
- [`docs/start/where-were-going.md:44`](../../../docs/start/where-were-going.md)

