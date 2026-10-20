# #1809 — bf16 Phase 2b: flash-style tensor-core fused read (opt-in UOR_R4_CUDA_READ=flash; +6.3–6.7% end to end; References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T22:36:56Z
- **Updated:** 2026-10-09T13:08:59Z
- **Closed:** 2026-10-06T22:37:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1809
- **Merged:** 2026-10-06T22:37:57Z
- **Merge commit:** f49409195eb25b157ec3d11d9879d900f73846fc
- **Base / Head:** main / UOR-Foundation:claude/bf16-flash-read
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820, Compute board #1750. Phase 2b of `docs/compute/bf16-phase2-design.md` (handoff `docs/compute/bf16-phase2-handoff.md`).

**What:** a flash-style CUDA fused read behind `UOR_R4_CUDA_READ=flash` (`CudaReadKernels::{Fused, Flash}`, `set_cuda_read_kernels`). Fused stays the default and is unchanged.
- **Forward `read_flash_fwd`:** online softmax with the NoRead logit seeding the max, any value width in chunks, no T×T buffer.
- **Backward:**
  - `read_flash_bwd_query`: row stats max/total/D = Σp·dp recomputed in f64; dQ, NoRead, per-row beta/offset and age partials.
  - `read_flash_bwd_key`: dK, dV.
  - `read_flash_bwd_reduce`: ordered f64 sums. No float atomics, so the read is deterministic.
- **bf16-storage build:** all three run on tensor cores (bf16 WMMA 16×16×16, f32 accumulate). P and dS operands are split hi+lo bf16 for accuracy. The f64 islands are kept: lifts, excess, distances, ds, per-head sums. The f32 build uses scalar kernels.
- **NVRTC:** the bf16 build now targets compute_80, with `$CUDA_HOME/include` added for `<mma.h>`.

**Evidence (RTX 5090, sm_120, CUDA 12.8; at d67ff4ff, before the merge of main, which touched no CUDA/read code):**
- `cuda_stack_ops_parity`: **36 passed, 0 failed**.
  - New tests: `test_flash_read_forward_parity`, `test_flash_read_parity` (Dot/Lorentz/L2 × NoRead × age; shapes incl. T=129/384 and odd V). They compare Flash against Fused and against the CPU reference.
  - Precision: f32 max diff ≈ 1e-7 (fwd) / 1.5e-6 (bwd). bf16 stays within the existing bound plus one bf16 step (2⁻⁷ rel); worst dv 1.95e-3, dq 9.8e-4.
  - Determinism: two runs are bit-identical.
- **Op bench** (bf16, B16 H8 K72 V72, L2 null+age, fwd+bwd ms/call):

  | T | Fused | Flash |
  |---|---|---|
  | 384 | 7.31 | **5.98** |
  | 1024 | 32.54 | **27.59** |

  Forward alone: 0.598 → 0.408 at T384; 4.32 → 2.46 at T1024.
- **End-to-end training** (precision=bf16, tf32, batch 16/32, ctx 384, alternating rounds):

  | Model | Rounds | Fused tok/s | Flash tok/s | Gain |
  |---|---|---|---|---|
  | 29M | 3 | 112,469 / 111,349 / 112,824 | 119,216 / 120,019 / 120,052 | **+6.7%** |
  | 96M (width 1024 / 14 layers) | 2 | 63,034 / 62,957 | 66,901 / 66,960 | **+6.3%** |

  Memory also drops: no T×T probabilities, excess, dp or ds buffers.

**Not yet run:** the frozen 2-seed 70,609-step NLL comparison from `bf16-phase2b-gate.md` before making Flash the default. Until then it is opt-in.

Laptop at the merge head: `cargo check -p uor-r4-training --lib --tests` 0 errors; rustfmt clean on touched files.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6026721441 — 2026-10-06T22:37:05Z — Casey-allard

Exact-head self-review at `87fe352992b4ef08c5e41c1637d31a2accb9b64a` (owner rule: self-review allowed). Diff read: 4 kernel/selector files + tests. At this head on the laptop: `cargo check -p uor-r4-training --lib --tests` 0 errors, rustfmt clean on touched files. CUDA parity (36/0) and benches ran at `d67ff4ff`; this head only adds a merge of main (AGENTS.md, mqar-bench example, docs — no CUDA/read code), verified by `git diff d67ff4ff 87fe352992b4ef08c5e41c1637d31a2accb9b64a --stat`. Default path unchanged (Fused); Flash is opt-in. APPROVE.


---
_Comment/review ids in this file: 6026721441_

