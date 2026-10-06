# Where a 29M bf16 training step spends its GPU time (RTX 5090, 2026-10-05)

Profile of `geometric-stack train` on the branch `codex/bf16-phase2a-chunked-recurrence`
at `2e97c92e`, RTX 5090 (sm_120, CUDA 12.8), the 29M recipe (width 576, heads 8,
layers 10, `rrarrarrar`, context 384, batch 16), `precision=bf16`, `tf32=true`,
`data_parallel=1`, the serial split recurrence path. Captured with
`nsys profile --trace=cuda` over 5 steps and summarised from
`nsys stats --report cuda_gpu_kern_sum`.

## Kernel families

Total GPU kernel time 203.4 ms over 5 steps (40.7 ms/step; the run's own
`train_seconds` is 49 ms/step, so ~17% of the step is launch gaps and
non-kernel work). 67 kernel entries, 99.7% of runtime:

| family | share of kernel time |
| --- | --- |
| **fused geometric read** | **34.5%** (forward 8.9%, backward 25.6%) |
| gemm (cuBLAS/cutlass bf16 tensor cores) | 17.4% |
| rms_norm (forward, `bwd_dx`, `dw_partial`, `dw_reduce`) | 16.6% |
| Candle elementwise and dtype casts | 11.5% |
| **recurrence core** (prep, carry, out, backward, parameter partials) | **8.3%** |
| cross_entropy | 4.3% |
| gradient squared norm (`sq_lanes`, `sq_tree`) | 3.8% |
| adam_update | 1.3% |
| other | 2.0% |

Largest single kernels: `rms_norm_bwd_dx` 9.2%, `read_dkv` 6.9%,
`read_tile_inner` 5.9%, `read_key_self` 5.0%, `rms_norm_fwd` 4.5%, `read_dage`
4.5%, `ia_u32_f32` (embedding `index_select`) 4.0%, `read_row_grad_warp` 4.0%,
`recurrence_bwd_params` 3.9%, `sq_lanes` 3.8%, `ucopy_bf16` 3.2%,
`cross_entropy_rows_act` 2.8%, `read_dbeta` 2.8%, `rms_norm_dw_partial` 2.8%,
`badd_bf16` 2.7%, the largest cutlass bf16 GEMM 2.7%.

## What this says about prioritisation

- **The read is the target.** One third of the step is the fused geometric
  read, and three quarters of that is its backward: the O(T²) `ds`, `dp`,
  `inner_grad`, `excess` and `key_self` buffers and the six kernels that carry
  them. This is exactly what a flash-style read removes.
- **The recurrence cannot pay for a rewrite.** At 8.3% of the step (and only
  ~2% of it the serial carry), even a perfect recurrence kernel is worth a few
  percent. The chunked path measured below is consistent with that bound.
- **RMSNorm is the second target** (16.6%, `bwd_dx` alone 9.2%): its backward
  recomputes row statistics in f64 and writes a `row_r` buffer that a second
  kernel re-reads.
- The bf16 *storage* work is visible and bounded: `ucopy_bf16` (3.2%),
  `cast_f32_bf16`/`cast_bf16_f32` (1.0%) and `badd_bf16` (2.7%) are the price of
  Phase 1's activation dtype; the GEMMs (17.4%) are already on tensor cores.
- The gradient squared norm is fragmented into ~129 launches per step
  (`sq_lanes`, 3.8%) — a launch-count target, not a FLOP target.

## Phase 2a (chunked recurrence) measured: parity clean, speed neutral

The chunked path (fold per tile / carry per window / expand per tile) was
implemented and compared against the serial split path on the same GPU:

| measurement | serial | chunked |
| --- | --- | --- |
| op parity (32-test CUDA suite, incl. `test_chunked_recurrence_parity`) | — | **32/32 pass**; max \|chunked − serial\| ≤ 4.8e-6 (stated bound 1e-4 + 1e-3 rel), bit-identical for single-tile shapes, deterministic across runs |
| 29M bf16, 300 steps, alternating rounds (tok/s) | 124,117 / 124,174 / 124,446 | 123,369 / 124,613 / 124,448 |
| 29M bf16, tile sweep 8/16/32/64/128 (tok/s) | — | 124,217 / 123,023 / 124,805 / 124,033 / 122,611 |
| ~96M shape, single GPU, batch 32 (tok/s, bf16) | 60,260 | 59,522 |
| ~96M shape (tok/s, f32) | 56,014 | 55,638 |

Median 29M bf16: serial 124,174 vs chunked 124,448 tok/s (+0.2%, inside
run-to-run noise); at the 96M shape the chunked path is 0.7–1.2% *slower*. The
extra read of `q`, `drive` and `keep` by the fold and the expand costs what the
shorter serial depth saves, and the profile above explains why there was never
much to win: the carry is ~2% of the step.

**Conclusion.** The chunked scan is a correctness-clean alternative path, not a
speed-up; it must not become the default and does not justify a 29M adoption
gate. Phase 2 should attack the read's backward (25.6%) first, then RMSNorm
(16.6%), and revisit the recurrence only if a future shape (longer context,
larger batch) makes its carry material — the kernel-family method above is the
way to check that, not an assumption.
