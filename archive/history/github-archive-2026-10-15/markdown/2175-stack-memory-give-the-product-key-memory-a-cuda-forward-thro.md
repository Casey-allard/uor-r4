# #2175 — stack_memory: give the product-key memory a CUDA forward through the host bridge (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T20:02:45Z
- **Updated:** 2026-10-10T21:13:17Z
- **Closed:** 2026-10-10T21:09:28Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2175
- **Merged:** 2026-10-10T21:09:28Z
- **Merge commit:** 1873d776b5a8348177e62e68886f204511e694b4
- **Base / Head:** main / UOR-Foundation:deepseek/memory-cuda-20261010
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**The product-key memory was the one op in the training path with no CUDA implementation, so `device=cuda` refused a memory-equipped stack outright — measured last cycle as `no cuda implementation for geometric-stack-product-key-memory`, which forced the memory fit onto CPU at 3.7 s/step and made that run unfit to judge the mechanism. It now has a CUDA forward: its exact CPU forward on host copies of the three inputs, uploaded back through the bridge the stack's other kernel-less configurations already use.** References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: exact addressed memory (D22 order 2) · count 0/3 · headline unchanged (preparation; the run it prepares is pre-registered on #2029 before any compute)

## What changed

| | |
|---|---|
| `ProductKeyMemory::cuda_fwd` | calls [`cuda_ops::via_host3`](crates/uor-r4-training/src/geometric_stack/cuda_ops.rs), which runs `cpu_fwd` on `to_cpu_storage()` copies of the three inputs and uploads the F32 result — the same rule the stack's kernel-less configurations follow, so **CUDA training computes exactly what the CPU computes** |
| visibility | `cuda_ops` and `via_host3` widened from `pub(super)` to `pub(crate)` so the sibling `stack_memory` module can use the bridge; the module doc records the new caller |
| the CPU path | **untouched** — the op's maths, its tie-breaking and its hand-written backward are unchanged, so every existing artifact and reading keeps its exact numbers |
| the backward | already device-generic: `bwd` reads host copies and rebuilds its gradients on the input's device |

`cuda_fwd` is behind `#[cfg(feature = "cuda")]`, so CPU-only builds are unaffected. The configuration is F32 (`precision=f32`); a bf16 activation still takes the F32-expecting host path and is refused there rather than silently computed in another precision.

## Checks

| check | command | result |
|---|---|---|
| CPU build | `cargo build --release -p uor-r4-training --lib` | clean |
| memory tests | `cargo test -p uor-r4-training --lib stack_memory` | **7 passed**, including the finite-difference gradient check |
| CUDA build | the pod bootstrap (`--ref` this branch) | compiles and passes the CUDA parity test — this is the check the local CPU build cannot perform |
| claim wording | `python3 scripts/check_claim_wording.py` | **pass** |

The per-step cost of the three transfers is measured on the pod and reported with the run; if it turns out material next to the trunk, the honest kernel is the follow-up rather than a claim that the bridge is free.

This is a preparation PR and it names the run it prepares: the **two-seed memory fit on the GPU at D10's own 2,000 steps**, with the brief's bar (memory ≥ 27/40 and BPB within 0.01), pre-registered on #2029 with its `TEST FITNESS: FIT` review and two declared deviations.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

