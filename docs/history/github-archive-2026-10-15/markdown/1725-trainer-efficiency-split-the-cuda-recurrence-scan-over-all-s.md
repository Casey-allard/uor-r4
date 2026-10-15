# #1725 — Trainer efficiency: split the CUDA recurrence scan over all SMs (1.16x tf32 / 1.10x fp32) (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T06:49:43Z
- **Updated:** 2026-10-09T13:08:48Z
- **Closed:** 2026-10-05T10:41:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1725
- **Merged:** 2026-10-05T10:41:07Z
- **Merge commit:** eaaf248bbf943d5ccb7dbdee56e953853e7db022
- **Base / Head:** main / UOR-Foundation:codex/eff-recurrence-scan-underoccupied
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820.

## What

On the A100 profile of the 96M benchmark run, the quaternion recurrence (`recurrence_core_fwd`/`_bwd`, the 10 `r` layers) used one thread per (window, lane), serially over time. That is 16 blocks of 256 threads, so it ran on 16 of the 108 SMs and took about 51.7 ms/step (18%).

This PR adds a **split** CUDA path, which is now the default:

- `recurrence_prep` runs one thread per (window, position, lane) and computes the 4-tap drive convolution, gate sigmoid/exp, transport unit, `lambda·unit` and keep.
- `recurrence_scan_fwd` / `recurrence_scan_bwd` are serial over time and carry only the 4-float quaternion. Each processes chunks of 8 positions and is launched in 32-thread blocks (`SCAN_GROUP`), so the 4,096 scan threads spread over 128 blocks.
- `recurrence_out` runs per element. `recurrence_bwd_direct` and `recurrence_bwd_post` run per position. `recurrence_bwd_params` runs per (window, channel) and computes the convolution gradient and the f64 parameter partials. The existing `recurrence_param_reduce` is unchanged.
- **Exactness:** the split kernels write the contracted operations with non-contracting intrinsics (`__fmaf_rn` / `__fmul_rn` / `__fadd_rn`), in the same fusion order as the single kernels' sm_80 machine code (`ptxas` from CUDA 12.8). Every output and gradient is therefore bit-identical to the original kernels.
- `UOR_R4_CUDA_RECURRENCE=single` (or `set_cuda_recurrence_kernels`) keeps the original single kernels selectable. Split is the default because it is exact.
- Saved model formats are unchanged, and serving (D11) is not affected.

## Parity (A100-SXM4-80GB, CUDA 12.8, driver 570)

| check | result |
|---|---|
| new `test_recurrence_split_matches_single_bitwise`: out, d_branches, d_gates, d_parameters for 8 shapes (Quaternion and U1, with and without rotation, T below, at and off the scan chunk, plus the training shape B16·T364·W1024) | **0 differing bits** |
| existing CPU vs CUDA recurrence tests (`test_recurrence_core_parity`, `_backward_parity`, `_large_gate_parity`), which now run on the split default | pass (max diff 4.7e-10 forward, 1.2e-7 backward, unchanged) |
| `cuda_stack_ops_parity` suite at the PR head (rebased on 86e85291) | 20 passed, 0 failed, 1 ignored (the benchmark) |
| 300-step benchmark config, tf32=true, same seed, old binary (`geometric-stack-9fc84ce8`) vs new | **per-step losses identical at all 30 eval points (train NLL, dev NLL, grad norm: max diff 0)**, final dev NLL 0.083088 in both, **saved model SHA-256 identical** (`6eafad4f9ee75c00…`) |

## Speed: 2,000-step benchmark

Config: 96M dialogue-train, pointer 32, batch 16, ctx 384, mixed-c, init geo-100m, `RAYON_NUM_THREADS=8`, under `flock /root/gpu.lock`, eval_every=1000.

| | reference (old) | new | speedup |
|---|---|---|---|
| tf32=true | 578 s (re-measured today on the old binary: 577.7 s) | **499.7 s** (249.8 ms/step mean) | **1.16x** (-78 s) |
| tf32=false | 865 s | **786.7 s** (393.4 ms/step mean) | **1.10x** (-78 s) |

Final dev NLL of the 2,000-step runs: 0.06057 (tf32=true) and 0.05874 (tf32=false).

The saving is about 39 ms/step, roughly three quarters of the recurrence's 51.7 ms.

## Caveats

- The 2,000-step timings were measured on the source at base 578c3073. The PR head is that change rebased onto 86e85291, which touches no trainer CUDA files. At the rebased head the binary and the parity suite were rebuilt and re-run on the A100 (all pass), but the 2,000-step benchmark was not repeated.
- The backward still recomputes the forward state (prep + scan) instead of keeping the forward buffers. That is part (b) of the proposal; it would save about 24 MB × 3 per layer at this shape, and it is left for a follow-up.
- Bit-identity is shown for sm_80 with CUDA 12.8 `ptxas`. Another compiler or architecture could fuse the single kernels differently. The split kernels' own rounding is fixed by the intrinsics.
- No chunked associative scan was used, because it would change rounding.
- `cargo fmt` is clean for the touched files. The non-CUDA `cargo check` (laptop) and the CUDA build (pod) show no new warnings; the remaining ones already exist on main.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5991094652 — 2026-10-05T08:43:51Z — Casey-allard

Adversarial review at 9b8843010c35786e3bd6900715789a529b398a72:

**Verdict: no blocking findings.** I would approve. The numbers below are from my own runs at the exact head on the A100 (driver 570, CUDA 12.8), each under `flock /root/gpu.lock` after `CTX DONE`. Artifacts are in `/root/eff/claude-adv1725/`. I did not push or merge anything.

### Kernel review (read against `recurrence_core_fwd` / `recurrence_core_bwd`)
- **Index math.** `prep`, `out`, `bwd_direct`, `bwd_post` and `bwd_params` index the same `row·width + 4·lane`, `row·2w (+w)` and `gate_row + lanes + ch` addresses as the single kernels. The two halves of `d_branches` are written by different kernels: the output half by `bwd_direct`, the input half by `bwd_params`. Between them every element is written exactly once.
- **Partials order.** The `p[s·w + c]`, `p[4w + c]` and `p[5w + lane]` partials have the single kernel's layout. They are summed in its reverse-time order: f64 accumulation of f32 `dd·branch` products, and d log a summed only by the channel that owns the lane.
- **Convolution adjoint.** The input-branch gradient for position s is accumulated in the single kernel's order: shift 3 first, then 2, 1, 0, with matching `t+k<time` guards.
- **Races and aliasing.** I found none. `q`→`dd` and `keep`→`log_term` are overwritten only in `bwd_post`, which recomputes lambda, keep and unit from the gates and never reads those buffers. In `scan_bwd`, each thread reads and overwrites only its own (window, lane) channels. Within a chunk it loads all values before any store. The kernels are ordered on one stream.
- **Dtypes.** fp32 values with f64 partials, unchanged. The NVRTC options are unchanged (no fast-math, `prec_div` on), so `__fdiv_rn` matches `/`.
- **Defaults and formats.** Saved formats are untouched. The default path is new code, but it produces identical bits (see below). `UOR_R4_CUDA_RECURRENCE=single` restores the old kernels.

### Parity, re-run at the head
- **Full `cuda_stack_ops_parity` suite** (`--test-threads=1`, `UOR_REQUIRE_CUDA=1`): all of the PR's tests pass, including `test_recurrence_split_matches_single_bitwise` with 0 differing bits over all 8 shapes. CPU-vs-CUDA recurrence forward and backward also pass on split.
- **Edge cases the PR test does not cover.** I added a test locally (`adv1725_split_matches_single_edges`, not pushed) that compares split and single bit for bit on:
  - **Clamped keep:** decay 40 gives lambda = 1 and `complement < 1e-6`. Covered both all-lanes-clamped and alternating clamped/unclamped lanes, for Quaternion and U1. With the PR test's decays {-1..7} this branch never runs.
  - **Length and chunk boundaries:** `time = 1`, `time = 9` and `time = 17` (chunk + 1), and `time = 16` (a multiple of the chunk).
  - **Partial scan block:** width 132, so lanes = 33 and windows are not a multiple of `SCAN_GROUP = 32`.
  - **No-rotation path:** rotation off with Quaternion, run under clamping.
  - **Middle shape:** b4 t129 w256.

  All 36 comparisons (out, d_branches, d_gates, d_parameters) have 0 differing bits, and every value is finite.

### Speed: 2,000-step benchmark (96M dialogue-train, pointer 32, batch 16, ctx 384, mixed-c, init geo-100m, eval_every 1000)
Same head binary for the A/B. The only difference is `UOR_R4_CUDA_RECURRENCE`.

| mode | single (old kernels) | split (default) | speedup |
|---|---|---|---|
| tf32=true | 577.3 s | 499.6 s | **1.155x** |
| tf32=false | (reference 865 s) | 786.6 s | **1.10x** vs reference |

- **End-to-end identity (tf32=true).** Split and single give identical dev NLL (0.0605725039), identical grad norm (1.85494) and an identical saved `model.safetensors` SHA-256 (`66ae5773…`).
- **Agreement with the PR.** My split results match the PR's reported runs: 499.65 s and dev NLL 0.0605725 for tf32=true, and 786.7 s for fp32.
- **Rebase caveat closed.** The PR's caveat that the benchmarks predate the rebase no longer applies: these runs are at the head.
- **What was not run.** I did not do a same-day single run at fp32, so that row compares against the reference.
- **Timing source.** Times are `train_seconds` from the reports. Other labs' CPU builds were running on the pod during part of the window.

### Non-blocking findings
1. **The kernel selector is process-global and can race in tests.** `test_recurrence_split_matches_single_bitwise` switches the global `AtomicU8` to `Single` while it runs. Under the default multi-threaded `cargo test`, the CPU-vs-CUDA recurrence tests can run during that window and exercise `Single` instead of the split default. Values would not change, but split coverage would silently drop. The PR's claim that the parity tests pass on split holds only with `--test-threads=1`. Suggested fix: pass the selector into the op, or serialise these tests with a mutex.
2. **The PR test misses the clamped-keep branch** (`!clamped` skips the keep term). It passes in my run; I suggest adding a large-decay case to the checked-in test.
3. **The bit-identity caveat names the wrong compiler.** The PTX is built by NVRTC at runtime and compiled to SASS by the **driver's** JIT (driver 570 here), not by toolkit `ptxas`. Bit identity therefore depends on the driver version and the architecture (for example the RTX 4090, sm_89). If they differ, both paths remain valid fp32 code with the same expression order, so I see no correctness risk, but the "bit for bit" doc comments should say this.
4. **Unrecognised environment values select split silently.** Any value of `UOR_R4_CUDA_RECURRENCE` other than `single`, including typos, gives `Split`. A one-time warning for values other than `split` or `single` would help.
5. **Peak memory rises slightly.** The split backward holds drive, q, state, direct and keep: four full B·T·W fp32 buffers, against three for single. At the training shape that is about +24 MB per recurrence call. This is negligible on an A100 but is worth stating for laptop CUDA or Metal later.
6. **Follow-up already noted by the author.** The backward still recomputes the forward prep and scan (proposal part (b)). Keeping those buffers from the forward is the obvious next saving.


### comment 5991106296 — 2026-10-05T08:44:42Z — Casey-allard

Lab-lead exact-head check at 9b8843010c35786e3bd6900715789a529b398a72: the adversarial reviewer independently re-ran parity and the 2,000-step benchmark on the A100 at this exact head. The results are bit-identical: same saved-model SHA-256, same dev NLL and same grad norm, and the speed-up is measured. Queuing. These efficiency PRs are queued in order (#1717, #1721, #1725); if a later one conflicts after an earlier merge, it will be rebased and re-checked.

### comment 5992822313 — 2026-10-05T10:39:50Z — Casey-allard

Merged main in after #1717 and #1721, at head 46ccdc080d7e3b82f23d1c049147986ae4b202a0. There was one conflict in `cuda_ops.rs`'s recurrence backward.
- **Resolution:** I kept this PR's split/single dispatch, with `zeros` for its backward buffers. #1721's switch to `uninit` for this block was proven only for the old single kernel; whether the split kernels write every element has not been shown.
- **Signature check:** the `recurrence_core_bwd` kernel and its call both take 17 arguments.
- **Parity at this head,** on the RTX 4090 pod: `cuda_stack_ops_parity` gives 22 passed, 0 failed, 1 ignored, including the bitwise split-vs-single test.

Queuing.


---
_Comment/review ids in this file: 5991094652, 5991106296, 5992822313_

