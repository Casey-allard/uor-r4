# #1675 — Run the pointer-copy mixture loss on CUDA (forward and backward)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T05:44:06Z
- **Updated:** 2026-10-09T13:08:43Z
- **Closed:** 2026-10-04T05:45:09Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1675
- **Merged:** 2026-10-04T05:45:09Z
- **Merge commit:** 71a57f3671a36bd981375585b9586e49b83a60ae
- **Base / Head:** main / UOR-Foundation:codex/cuda-pointer
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Dialogue fine-tunes (`dialogue-train pointer=32`) ran the pointer-copy loss on the host. Under `cuda`, the forward always went through `via_host3` and the backward was host-only. That moved [rows × vocab] f32 logits over PCIe three times per step. This adds CUDA kernels for both directions; the logits stay on the device.

**Kernels** (`cuda_stack_kernels.rs` section 9: `pointer_lift`, `pointer_rows`, `pointer_side_grad`, `pointer_sum`). The single existing `unsafe` launch block is unchanged.
- **Log-sum-exp:** reuses the cross-entropy row kernel (f64).
- **Per row (one warp):**
  - Dot scores in f32 with the CPU dot's 16 partial sums, scaled by 1/√dim;
  - Lorentz −β·arcosh(1+e) in f64 with the same clamp;
  - then the host's f64 softmax, copy mass over matching input ids, generate/copy log-probabilities and the log-sum-exp mixture.
- **Forward:** row losses summed in row order, divided by the total weight.
- **Backward:** logits (cross-entropy gradient scaled by c·share_gen), gate, scores (overflow-safe copy-share form; zero when the copy mass is 0), query, key (summed over reading positions), and β (row partials summed in row order). Zero-weight rows get no gradient.

**Still on the host:** pointer `pointer_select` and `pointer_route` configurations, and non-contiguous or non-F32 inputs. L2 pointers are refused by validation.

**Evidence at `71c602e7`** (Runpod RTX 4090, driver 595.91.07, CUDA 12.6, GPU 0):
- Command: `UOR_REQUIRE_CUDA=1 CUDA_VISIBLE_DEVICES=0 cargo test --release -p uor-r4-training --features cuda --test cuda_stack_ops_parity -- --test-threads=1`.
- Result: **19 passed, 0 failed, 1 ignored** (the existing benchmark), with no SKIPPED.
- The new `test_pointer_mixture_parity` covers Dot and Lorentz, unweighted and weighted, and three shapes, including rows with no matching source and a zero-weight row.
  - Lorentz: the loss and every gradient are bit-identical to the CPU.
  - Dot: the loss and gate are exact; the other gradients are within ~9e-8 relative.
- Laptop: `cargo fmt --all --check` passes, and the non-cuda `cargo check -p uor-r4-training --all-targets` passes.

**Not measured:** the fine-tune speed-up (a timing run follows once the pod's GPUs free up). Not run: clippy and the full suite.

References #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### review 5404500033 — 2026-10-04T05:44:16Z — Casey-allard — COMMENTED

**Exact-head review at `71c602e7cf50ee5a3744ed777af5936b388633f1` (Claude lab; self-review under the owner's 2026-09-30 rule).**

**Scope:** additive CUDA kernels behind the `cuda` feature. Unsupported configurations still route to the host, the single `unsafe` launch block is unchanged, and the non-cuda build is unaffected (laptop check passes).

**Evidence at this head:** 19 parity tests passed on an RTX 4090 with `UOR_REQUIRE_CUDA=1` (no skips). Pointer Lorentz loss and gradients are bit-identical to the CPU; Dot is within 9e-8 relative.

**APPROVE**; queueing with `--match-head-commit`.

### comment 5981084557 — 2026-10-04T14:31:56Z — Casey-allard

**Measured speed-up after merge.** I ran the ~96M chat fine-tune with the same settings on both builds: `dialogue-train`, pointer=32, batch 16, context 384, `mixed-c` corpus, one RTX 4090, with the other GPU busy in both cases.

| Build | Seconds to step 2,000 |
|---|---|
| `74d8189` (CPU pointer path) | 924 |
| main `e8226b76` (CUDA pointer kernels) | 516 |

That is **1.79× faster**. The run is `chat-100m-Cx`, compared against the morning's `chat-100m-C`. References #820


---
_Comment/review ids in this file: 5981084557, 5404500033_

