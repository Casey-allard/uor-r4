# #1721 — Trainer efficiency: first gradient contribution stored directly, no zero fill for fully written CUDA outputs (1.175x tf32 / 1.11x fp32) (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T05:48:10Z
- **Updated:** 2026-10-09T13:08:47Z
- **Closed:** 2026-10-05T08:45:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1721
- **Merged:** 2026-10-05T08:45:29Z
- **Merge commit:** 8788427d641d18dbdba57a45f1b40a9d8d47e4c8
- **Base / Head:** main / UOR-Foundation:codex/eff-candle-backprop-zero-then-add
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820.

## What changes

The A100 profile of the benchmark config (96M dialogue-train, pointer 32 Dot, batch 16, ctx 384, mixed-c, init geo-100m) showed 876 `badd_f32` and 1,290 memsets per step: about 57 ms of a 291 ms step at tf32=true. This PR removes the work that came from two causes.

1. **Vendored Candle backprop** (`third_party/candle-core-0.9.2/src/backprop.rs`). Every gradient contribution used `grads.or_insert(x)` (a `zeros_like` fill) followed by `add`. `GradStore::accumulate` and `accumulate_neg` now store a tensor's first contribution directly. They add only from the second contribution on, in the same order. `0 + g == g` holds exactly in IEEE-754; only the sign of a zero can differ. A strided, broadcast or offset first contribution is copied to a contiguous buffer, which keeps upstream's layout. `Gather`, `IndexSelect` and upsample keep `or_insert`, because they scatter into the zero tensor. `UOR_CANDLE_GRAD_ZERO_THEN_ADD=1`, or `backprop::set_upstream_zero_then_add(true)`, restores the upstream path for comparison. The change is recorded in `UOR-PATCH.md` and `UPSTREAM.json`.
2. **Fully written CUDA outputs** (`cuda_stack_kernels.rs`, `geometric_stack/cuda_ops.rs`). A new `uninit()` replaces `zeros()` for buffers whose every element is written before it is read. These are the elementwise and row kernels, the recurrence core forward and backward, the fused-read outputs and causal square buffers, and the cross-entropy logit gradients. Partially written buffers (`d_aux` and the pointer scratch) stay zeroed. Setting `UOR_CUDA_POISON_UNINIT=1` fills these buffers with NaN words, so the parity tests catch any read of an unwritten element.

The change does not affect serving (D11) or the saved model format.

## Numerical parity (A100-SXM4-80GB, CUDA 12.8)

| test binary | normal | `UOR_CUDA_POISON_UNINIT=1` |
|---|---|---|
| `backprop_first_contribution_parity` (new, 5 tests) | 5 pass | 5 pass |
| `cuda_stack_ops_parity` (existing, 19 tests + 1 ignored bench) | 19 pass | 19 pass |

- Old vs new backprop, synthetic graph, on both CPU and CUDA: bit-identical up to the sign of zero.
- Old vs new on the geometric stack, CPU and CUDA: all 82 gradients and all 82 variables are bit-identical after 3 AdamW steps.
- New path, CUDA vs CPU: the worst relative max diff across 82 gradients is 8.9e-6.

## Training-curve parity: 300 steps, same seed, tf32=true

Old binary: `geometric-stack-9fc84ce8` (main trainer sources). New binary: this branch.

| metric (300 steps) | identical steps | max abs diff |
|---|---|---|
| train_response_nll | 300 / 300 | 0.0 |
| dev_response_nll | 300 / 300 | 0.0 |
| grad_norm | 300 / 300 | 0.0 |
| dev_first_four_nll | 300 / 300 | 0.0 |

At step 300, dev NLL is 0.08308795039256849 for both binaries. The 2,000-step runs below also finish bit-identical: train 0.9045331028997898, dev 0.060572503937957184, grad norm 1.854942745137444.

## Speed: 2,000-step benchmark on the A100 under `flock /root/gpu.lock`

| run | train_seconds | vs reference |
|---|---|---|
| old binary, tf32=true (same session, rerun) | 577.7 | (reference 578) |
| **new, tf32=true** | **491.6** | **1.175x (−86.1 s, −14.9%)** |
| old, tf32=false (earlier reference) | 865 | |
| **new, tf32=false** | **778.4** | **1.11x (−86.6 s, −10.0%)** |

The saving is roughly the same in absolute terms in both modes (about 43 ms/step), as expected for removing memsets and adds. At 300 steps with per-step evaluation, train_seconds was 87.1 s old and 74.1 s new.

## Checks

- `rustfmt --check` on all touched files: clean. The `cargo fmt --check` diff in `relation_compiler.rs` already exists on main and this PR does not touch that file.
- The release build of the example and of the test binaries produced no new warnings. All 22 warnings are pre-existing ones in unrelated bins.
- Local and pod sources were checked by md5 for all four touched code files.

## Caveats

- Both options are exact, so the new path is the default. The upstream path remains available through the environment variable.
- A first gradient contribution is now stored by reference rather than as a fresh sum. Candle tensors are immutable and no op here mutates gradients in place, so this is safe. The bit-identical 300- and 2,000-step runs confirm it for this config. Any future in-place gradient mutation would need to copy first.
- The tf32=false comparison uses the earlier 865 s reference; that mode was not rerun in this session. The tf32=true old run was rerun.
- `zeros` is the safe default for new kernels. Use `uninit` only when the kernel writes every element, and run the poison test to confirm it.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5990914265 — 2026-10-05T08:30:47Z — Casey-allard

Adversarial review at f14ee033ea7ecacbf4cd7f23972a14be8fd67928:

**Verdict: approve. I found nothing blocking.** I built this exact head from a `git archive` of f14ee033 on the A100 (CUDA 12.8; build, tests and runs under `/root/eff/claude-rev1721/`). Every GPU job ran under `flock /root/gpu.lock` after `CTX DONE`.

### Measured (A100-SXM4-80GB, 96M dialogue-train benchmark config, 2,000 steps, `train_seconds` from report.json)

| run | train_seconds | dev NLL @2000 | grad norm @2000 | model_sha256 |
|---|---|---|---|---|
| new, tf32=true | 491.5 | 0.060572503937957184 | 1.854942745137444 | 66ae5773… |
| old, tf32=true | 577.8 | 0.060572503937957184 | 1.854942745137444 | 66ae5773… |
| new, tf32=false | 778.3 | 0.05873975691211757 | 1.8829684371004676 | a715c171… |
| old, tf32=false | 864.7 | 0.05873975691211757 | 1.8829684371004676 | a715c171… |

- **Speedup:** 1.176x with tf32=true (-86.3 s) and 1.111x with tf32=false (-86.4 s). Old and new ran back to back in the same session for both tf32 settings, so this measures the tf32=false comparison that the PR took from the 865 s reference. The claimed numbers reproduce.
- **Saved models:** the old and new models are byte-identical (same `model_sha256`) in both tf32 modes. This end-to-end check covers both changes together: the backprop change, and the switch from `zeros` to `uninit` buffers.
- **Old binary:** `/root/geometric-stack-9fc84ce8…` is commit 9fc84ce8, not the merge base 578c3073. The two commits differ in no file under `geometric_stack*`, `cuda_*` or `third_party/`, so it is a valid old baseline.

### Parity tests (re-run at this head, `UOR_REQUIRE_CUDA=1`, `--test-threads=1`)
- **`backprop_first_contribution_parity`:** 5/5 pass, both normally and with `UOR_CUDA_POISON_UNINIT=1`. On CPU and CUDA the 82 gradients and the 82 variables after 3 AdamW steps are bit-identical between the old and new paths. Worst relative max difference between CUDA and CPU on the new path: 8.9e-6.
- **`cuda_stack_ops_parity`:** 19 pass and 1 is ignored, both normally and with poison.

### Code review
- **Backprop rewrite:** every arm was rewritten from `or_insert` + `add`/`sub` to `accumulate`/`accumulate_neg`. Accumulation order is the same as before. `Gather`, `IndexSelect` and upsample still use `or_insert`, which they need because they scatter into or overwrite the zero tensor.
  - `0 + g == g` holds exactly.
  - `-g` versus `0 - g` can differ only in the sign of a zero (or of a NaN).
  - The Max/Min arms now broadcast to `arg.dims()` and ToDevice uses `arg.device()`. Both are equivalent to the old `sum_grad` values.
  - A first contribution whose shape, dtype or device differs falls back to the upstream path, so error behaviour is unchanged.
- **Aliasing:** the stored first contribution can now share storage with another gradient or with the parent node's gradient. I looked for anything that writes into that storage:
  - Every CUDA kernel takes its inputs as `const`.
  - `adam_update` (CUDA and Metal) writes only into the parameter and moment buffers; it only reads the gradient.
  - The trainer's `grads.insert` calls replace entries rather than mutating them.
  - Nothing in `crates/` uses `slice_set` or `inplace_op` on gradients.
  - No backward rule returns a `Var`'s own storage, so `Var::set` cannot corrupt a gradient.

  Aliasing is therefore safe today, as the caveat says.
- **Layout:** a stored gradient is contiguous from offset 0. Downstream consumers are safe with a storage buffer longer than the tensor: CUDA `input()` slices by the layout's element count, and Metal adam uses `n` from offset 0.
- **`uninit` coverage:** I checked each kernel listed below against its buffer:
  - `straight_through_fwd`, `swiglu_fwd`/`swiglu_bwd`, `rms_norm_*`, `quaternion_scan_*` (dq at t=0 is written explicitly), `cross_entropy_grad`, `recurrence_log_a` and `recurrence_core_fwd` write every element.
  - `recurrence_core_bwd` writes every element. It zeroes the first half of d_branches; d_gates covers `lanes + width` when rotation is on and `lanes` when it is off, which matches `gate_width()`; partials cover `5w + lanes` per window. `recurrence_param_reduce` writes all elements too.
  - FusedRead's square buffers (probabilities, dp, d_scores, inner_grad, excess) are written only for j <= t. Every reader (`read_softmax_warp`, `read_mix`, `read_row_grad_warp`, `read_key_self`, `read_dq`, `read_dkv`, `read_dage`) reads only j <= t.
  - The Lorentz/L2-only buffers (lifts, query_self, row_beta, row_offset, key_self) are full-size only when `scaled()`, which is exactly when `score != 0`, the only case in which they are written and read.
  - `d_aux` and the pointer scratch correctly stay `zeros`.
- **Out of scope:** the `relation_compiler.rs` rustfmt diff is already on main.

### Non-blocking notes
1. **Stale provenance string:** `crates/uor-r4-training/src/joint_campaign.rs:540` still describes the vendored Candle as having only the "four-line Accelerate operand slice correction". This string goes into joint-campaign artifacts, so it now under-describes the vendored Candle. One line to update here or in a follow-up.
2. **Memory retention:** when a first contribution is an offset-0 contiguous view (for example the first `Cat` piece narrowed on dim 0), the stored tensor keeps the parent's full gradient buffer alive until the `GradStore` is dropped. Values are unaffected. I did not measure peak GPU memory; it is worth a look at much larger context or batch.
3. **Test coverage of `uninit`:** the old-vs-new bitwise unit test switches only the backprop path, and both sides use `uninit`. The `uninit` change is covered by CPU-vs-CUDA tolerance, the poison run, and the identical end-to-end `model_sha256` above.
4. **No poison negative control:** nothing shows that poison would catch a missing write, such as a deliberately unwritten element turning into a NaN failure. The `poison_words` kernel does launch, since the poison runs would fail otherwise, and my manual read of kernel coverage agrees with the poison results.

I did not push or merge.


### comment 5991105560 — 2026-10-05T08:44:39Z — Casey-allard

Lab-lead exact-head check at f14ee033ea7ecacbf4cd7f23972a14be8fd67928: the adversarial reviewer independently re-ran parity and the 2,000-step benchmark on the A100 at this exact head. The results are bit-identical: same saved-model SHA-256, same dev NLL and same grad norm, and the speed-up is measured. Queuing. These efficiency PRs are queued in order (#1717, #1721, #1725); if a later one conflicts after an earlier merge, it will be rebased and re-checked.


---
_Comment/review ids in this file: 5990914265, 5991105560_

