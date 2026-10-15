# #1717 — Trainer efficiency: bit-identical multi-block gradient-norm reduction on CUDA (1.38x tf32 / 1.22x fp32 on A100) (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T05:02:24Z
- **Updated:** 2026-10-09T13:08:47Z
- **Closed:** 2026-10-05T08:45:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1717
- **Merged:** 2026-10-05T08:45:29Z
- **Merge commit:** f815be9b4c4ba4ab9715098762660336eb0db1a4
- **Base / Head:** main / UOR-Foundation:codex/eff-gradnorm-single-block-sum
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Global gradient-norm clipping in `StackAdamW::update` ran `grad.sqr()?.sum_all()?` once per variable. On CUDA, Candle's `fast_sum_f32` reduces a whole tensor in **one block** (`grid=(1,1,1)`, up to 1024 threads), at about 527 µs per variable. In the A100 profile of the 96M benchmark this phase took **87 ms per step: 29% of the step with tf32 and 19.5% without**.

This PR replaces that path on CUDA with two stack kernels (`sq_lanes` and `sq_tree` in `cuda_stack_kernels.rs`) that **replay Candle's exact f32 summation order**:

- Candle's `fast_sum` has thread `t` of a `width = min(1024, n).next_power_of_two()` block add elements `t, t+width, …` in index order, then runs a shared-memory halving tree.
- `sq_lanes` computes the same per-lane sums, but spreads the lanes over up to 32 one-warp blocks with 8 loads in flight. It uses `__fmul_rn` and `__fadd_rn`, so nothing is contracted into an FMA, and it does not use ftz.
- `sq_tree` replays the halving tree, one block per variable.
- The final `cat(..).sum_all()` over the per-variable sums goes through the same two kernels without squaring.
- Non-contiguous gradients are made contiguous first, which keeps their logical order.

The result is **bit-identical** to the previous path, so no option is added. There is still exactly one host read per step. Metal and CPU paths are unchanged. `gradient_square_sum_reference`, the old Candle-op path, is kept as the parity reference. No model format, serving or D11 path is affected.

## Parity (all on the A100-SXM4-80GB, CUDA 12.8 libs, driver 570)

| check | result |
|---|---|
| `test_gradient_square_sum_parity`: kernel vs Candle (bits) and vs an f64 CPU sum. Covers sizes 1, 3, 16, 1000, 1024, 1025, 2049, 703×1024, 2048×1024, 4096×1024 and 7×11×13, a rank-0 tensor, transposed and narrowed (offset) views, subnormal squares, values near 1e16, a 169-tensor model-like list, and 1,500 tensors (where the final reduction chains) | **bit-identical in every case**; relative error vs f64 at most 1.1e-7 (3.5e-5 for the subnormal case, which is the same in Candle) |
| whole `cuda_stack_ops_parity` suite with `UOR_REQUIRE_CUDA=1` | 21 passed, 0 failed, 1 ignored |
| 300-step benchmark-config training, new vs base (origin/main 578c3073), same seed, `eval_every=1` | **0 differing fields over 300 steps** (per-step train NLL, f64 `grad_norm`, dev NLL, pointer stats); `model.safetensors` **byte-identical** |
| 2000-step speed runs, new vs base, tf32=true | final metrics identical (dev NLL 0.060572503937957184 in both); `model.safetensors` **byte-identical** |

## Speed (2,000 steps of the benchmark config: 96M dialogue-train, pointer 32 Dot, batch 16, ctx 384, mixed-c, init geo-100m, `RAYON_NUM_THREADS=8`, under `flock /root/gpu.lock`)

| setting | before | after | speedup |
|---|---|---|---|
| tf32=true, A100 | 577.7 s (base binary, same session; reference 578 s) | **419.3 s** | **1.38× (-27.4%)** |
| tf32=false, A100 | 865 s (reference) | **706.2 s** | **1.22× (-18.4%)** |
| 300-step curve runs, tf32=true (`eval_every=1`) | 0.290 s/step | 0.211 s/step | 1.38× |

Microbenchmark (`test_gradient_square_sum_speed`, 57 tensors and 73M elements): kernel 6.2 ms vs Candle 67.0 ms.

## Caveats and next step

- The kernels still launch once per variable, because exactness fixes at most 1024 lanes per tensor. At about 6 ms per 73M elements, the norm phase is now roughly 2-3% of a step. A single launch over a device pointer table would remove most of what is left; that is left for a separate change.
- The tf32=false "before" figure is the given reference, not a run in the same session. The tf32=true before/after pair was measured back to back under the GPU lock.
- Other labs' jobs shared the pod's queue through `flock`; no other GPU job overlapped the timed runs.
- Builds: `cargo fmt` is clean. The CUDA build of `uor-r4-training` reports the same 3 lib warnings as base (pre-existing, none in the touched files). The laptop non-CUDA `cargo check --tests` passed.

References #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5990138738 — 2026-10-05T07:35:28Z — Casey-allard

Adversarial review at 4b57dd1932b686cfec7e6bb3198760468fa68cb2:

**Verdict: no blocking findings.** The kernels match Candle's `fast_sum` exactly. I re-ran the parity tests and an extra set of adversarial cases on the A100, and both pass bit for bit. My own back-to-back 2,000-step runs reproduce the claimed speedups, with identical metrics and byte-identical models.

### Numerical review (read against vendored `candle-core-0.9.2/src/cuda_backend/mod.rs` FastReduce and `candle-kernels-0.9.2/src/reduce.cu` `fast_sum`)
- **Lane sums.** Candle sets `block_dim = min(1024, n).next_power_of_two()` and does `shr[tid] = 0; shr[tid] += src[idx]` for `idx = tid, tid+width, ...`. `sq_lanes` uses the same width, starts each lane at the same zero, and adds in the same index order. The 8-wide unroll only batches the loads; the additions stay sequential. Lanes `>= n` stay 0, as in Candle. The index math is u64, so it cannot overflow.
- **Contraction and denormals.** Candle's `usqr` is a separate kernel, so its multiply is never fused with the add. `__fmul_rn`/`__fadd_rn` reproduce that. Candle's kernels are built with nvcc `-O3` and no fast-math. The stack PTX uses NVRTC `ftz=false, use_fast_math=false`, so both sides keep subnormals.
- **Tree.** `sq_tree` replays Candle's halving tree, `for s = width/2 ...; __syncthreads(); if (t<s) ...`. `width` is the same for the whole block, so every thread reaches the same barriers. Width 1 skips the loop, and thread 0 reads back its own write. Threads `>= width` never read uninitialised shared memory. No races.
- **Final reduction.** `cat(..).sum_all()` is replayed unsquared with `width = fast_sum_width(count)`. The lanes chain when there are more than 1024 tensors.
- **Order.** Non-contiguous inputs go through `force_contiguous`, which keeps logical order, the same order Candle's strided `usqr` writes in. Temporary buffers are freed in stream order, so no lifetime hazard. Every error path returns `Result`; I found no unwrap or panic in library code. Metal and CPU paths are unchanged.
- **Defaults.** No option is added. The result is bit-identical, so old behaviour is preserved by construction, and I verified this end to end (below).

### Parity on the A100 (exact head, CUDA 12.8 libs, under `flock /root/gpu.lock`)
- `cuda_stack_ops_parity` with `UOR_REQUIRE_CUDA=1`: **21 passed, 0 failed, 1 ignored**. Microbenchmark: kernel 6.01 ms vs Candle 65.20 ms for 57 tensors and 73.0M elements.
- An extra adversarial test I wrote for this review (not pushed). **Every case is bit-identical** to `gradient_square_sum_reference`:
  - an empty tensor inside a list, and a list of only `[3,0]` tensors
  - n = 7, 8, 9, 1023, 7168, 7169, 8191, 8192, 8193 and 9215 (around the unroll boundaries)
  - n = 33,554,437 (over 2^24; many elements per lane)
  - a broadcast stride-0 view, and `t().narrow()`
  - +inf, overflow to inf, and -0
  - 1024, 1025 and 4097 tensors
  
  NaN gives `0x7fffffff` on both sides. A bf16 list and a mixed-device list return `Err` on both paths; neither panics.

### Re-measured speed (2,000 steps; 96M dialogue-train, pointer 32, batch 16, ctx 384, mixed-c, init geo-100m, `RAYON_NUM_THREADS=8`)
Base and head were built from 578c3073 and 4b57dd19 respectively. The four runs ran back to back, each under the GPU lock. Times are `train_seconds`.

| setting | base 578c3073 | head 4b57dd19 | speedup |
|---|---|---|---|
| tf32=true | 577.68 s | 418.80 s | **1.379×** |
| tf32=false | 865.02 s | 705.86 s | **1.225×** |

Mean step time: tf32=true 0.2888 → 0.2094 s; tf32=false 0.4325 → 0.3529 s. Within each tf32 setting, the step-1000 and step-2000 metric rows are identical between base and head once `train_seconds` is excluded. `model.safetensors` is byte-identical: sha256 `66ae5773…` for tf32 and `a715c171…` for fp32. This also fills the PR's caveat about tf32=false: the "before" figure is now a same-session measurement, and it matches the 865 s reference.

### Non-blocking notes
1. The PR's parity test has no zero-element tensor inside a list. That path depends on `fast_sum_width(0) == 1` and on the zeroed lanes buffer. It passes in my extra test; consider adding the case.
2. `gradient_square_sum` errors on non-F32 CUDA lists instead of falling back. The old path errored as well (`to_scalar::<f32>`), so this is no regression.
3. Process hazard I hit while reviewing, not a PR defect: extracting a `git archive` of a second commit into a fresh directory that shares a `CARGO_TARGET_DIR` gives stale binaries. The archive mtimes predate the cached outputs, so cargo skips the rebuild. Anyone comparing base and head should use separate target dirs (as the author did) or `touch` the sources.

Evidence on the pod: `/root/eff/claude-adv1717/` (`tests-parity.log`, `tests-adv.log`, `speed-*.log`, `run.status` with the model sha256s, binaries `gs-head`/`gs-base`). I have not pushed or merged anything.


### comment 5991104913 — 2026-10-05T08:44:36Z — Casey-allard

Lab-lead exact-head check at 4b57dd1932b686cfec7e6bb3198760468fa68cb2: the adversarial reviewer independently re-ran parity and the 2,000-step benchmark on the A100 at this exact head. The results are bit-identical: same saved-model SHA-256, same dev NLL and same grad norm, and the speed-up is measured. Queuing. These efficiency PRs are queued in order (#1717, #1721, #1725); if a later one conflicts after an earlier merge, it will be rebased and re-checked.


---
_Comment/review ids in this file: 5990138738, 5991104913_

