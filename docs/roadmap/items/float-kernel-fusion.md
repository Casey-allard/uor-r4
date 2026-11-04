---
id: float-kernel-fusion
title: One kernel list for the float crate, and fused kernels for the served quantizers
milestone: research
area: training
status: ready
open_to: lab
needs_gpu: true
depends_on: []
notebook: docs/research/notebook/float-training.md
scope: [crates/uor-r4-float-train/build.rs, crates/uor-r4-float-train/src/lib.rs, crates/uor-r4-float-train/src/cuda_stack.cu, crates/uor-r4-float-train/src/cuda_stack_kernels.rs, crates/uor-r4-float-train/src/fused_quantize*.rs, crates/uor-r4-float-train/src/hamilton_sheet.rs, crates/uor-r4-float-train/src/bin/train-in-space.rs, crates/uor-r4-float-train/tests/no_training_crate_dep.rs, crates/uor-r4-float-train/tests/twenty_four_kernel_controls.rs, crates/uor-r4-float-train/tests/fused_*.rs, crates/uor-r4-float-train/Cargo.toml, docs/harness/prep/float-kernel-fusion*, docs/loop/tasks/float-kernel-fusion/**, docs/research/notebook/float-training.md, docs/roadmap/items/float-kernel-fusion.md]
---

## Why it matters

Casey, 4 Nov 2026 (about 17:05Z, in claude's chat): "I ordered a 4090 for my local computer I built, so when it comes tomorrow we can write some custom drivers for it so we can actually reduce our workload in training".

Casey was then told that a fused kernel moves a pinned count: the float crate pins its 26 kernel entry points, and every new kernel also passes a check at load, so each pin and page that counts 26 moves with it. Casey answered (about 18:00Z, recorded on #2028, comment 6638209391): "Yes, please build this too to do it the smarter way going forward." So fused kernels are an approved scope change, and the count moves from one kernel list that every check reads.

Why these two kernels first:
- **The step is host-bound.** [claude-157](../../loop/tasks/fbs-cost-profile/claude-157/REPORT.md) found the backward 62% of a training step in one configuration on an RTX 5090, and 54% in the other, with host time 0.98–0.99 of GPU time. That is launch overhead or a wait at a sync inside the backward; the run does not tell them apart. The cancel rule below is there for that reason.
- **The served quantizers are op chains.** Under `--train-as-served 1`:
  - `activation_quantize16` (`hamilton_sheet.rs:723`) is 43 stock tensor ops a call, 51 with the straight-through bridge the served wrapper (`:761`) always takes. It runs twice a layer: on the recurrence's output gate (`geometric_stack_native.rs:2109`, in `composed_recurrence`) and on the MLP gate (`:1757`, in `mlp`). The layer loop (`:1634-1639`) calls both in every forward, and gate telemetry adds one more call to each.
  - `q16_residual` (`:646`) is 6 ops, or 14 with the bridge. It rounds the residual 1 + 2 × layers times a trunk, once more after each read, and again on every FBS projection's output (`served_projection_q16`, `:774`, through `StackModel::linear`).
  - Each has a one-line backward: the gradient times the in-range mask.
- **Scoring has the same floor.** [claude-279](../../loop/tasks/one-model-end-to-end/claude-279/REPORT.md) put scoring's decode on the exact cache. A cached step is then estimated at 8–9 ms for (L) at both 192 and 2,048 tokens. A step computes one position, so most of that is a fixed cost per forward, and the forward runs the same quantizer chains.

## What is missing

1. **One kernel list.** `KERNEL_ENTRY_POINTS` (`src/lib.rs:36`) and `EXPECTED_KERNELS` (`tests/no_training_crate_dep.rs:9`) are two hand copies of the same 26 names (22 in `cuda_stack.cu`, 4 in `spiralcore_fbs.cu`), and the test asserts the count 26 (`:103`). At run time, only the `HISTORICAL_KERNELS` deny list (defined at `src/cuda_stack_kernels.rs:46`, checked at `:356`) and the PTX load (`:376`) stand between a name and its launch. Nothing ties a launched name to the list.
2. **A measured share.** No GPU profile has yet split a served training step by op chain.
   - [served-rounding-cost](served-rounding-cost.md) (#3594, DeepSeek's item, M4) is the item for training as served's 6.83–6.96× slowdown against plain training.
   - Its host probe points at the export rounding instead: `grid_code` searches 2,032 grid candidates on the host for every rounded gain on every step, and nine matrices are copied to the host and back each step. Replacing the activation-scale loop saved about 2% of one CPU activation there.
   - That is a host probe, not a GPU profile, but it is the reason the fusion below waits on a measurement.
   - The two items share `hamilton_sheet.rs`, so the claim's overlap check orders their units. This item adds the kernel list that any new GPU kernel for that repair also needs. It does not take on the export rounding.
3. **The fused quantizers,** forward and backward, one kernel each, in `cuda_stack.cu`. That source builds both the f32 and the bf16 modules: `act_t` is defined at `cuda_stack.cu:61-85`, and the bf16 module is the same source built with `-DUOR_STORAGE_BF16` (`build.rs:40`). `spiralcore_fbs.cu` needs no change.

## Done when

1. **The kernel list is generated.**
   - `build.rs` scans each `MODULES` entry's combined `sources` (`build.rs:26-54`) for `extern "C" __global__ void <name>` on every build, CPU builds included. It writes one list per PTX module, which `lib.rs` includes. The arms module is built from three files, `spiralcore_fbs_arms_prelude.cu`, `spiralcore_fbs.cu` and `spiralcore_fbs_arms.cu` (`:44-53`). So its list is the 4 FBS kernels plus `spiralcore_fbs_arm_fwd` and `spiralcore_fbs_arm_bwd`.
   - The dispatch refuses a name not in its own module's list. The arms kernels, launched through `launch_prebuilt` (`geometric_stack_native_cuda.rs:798-805`), must pass that check, not fail it.
   - No literal kernel count is left in code, tests or their doc comments: `lib.rs:3-4, :33-35`, `tests/no_training_crate_dep.rs:3-4`, and the messages at `:83` and `:104`.
   - The test re-scans the sources itself and compares with the generated lists. The banned-token scan stays.
2. **Fused `activation_quantize16`,** behind a flag, equal bit for bit to the op chain it replaces. The flag is a thread-local in `hamilton_sheet.rs`, where the served wrappers switch to the kernel, set by a `train-in-space` option. It is equal:
   - forward and backward, on random rows that include zeros, −0, saturating rows, subnormals and exact .5 ties;
   - inside one training step at P2b's recipe: the loss and every gradient.
   The order of the bridge's ops (`free = x · inside`, `d = q − free`, `out = free + d`) and nvcc's `--fmad` choice are part of the gate.
3. **Fused `q16_residual`,** the same way.
4. **The cost,** on an RTX 5090 at P2b's recipe: kernel launches and seconds per training step, chain against fused, by the trainer clock (and `nsys`, if the pod has it). Also the seconds per cached decode step at 2,048, set against claude-279's estimate of 8–9 ms.
5. The flag defaults on only when 2 and 3 hold. One notebook row and a REPORT.

## Cancel if

- The two chains' share of wall seconds per training step at P2b's recipe, with `--train-as-served 1`, is under 5%, with their launch counts reported alongside. Then done-when 1 lands alone, and the fusion is reported as not worth its pin.

## How to start

- **Measure before fusing.** Done-when 1 needs no measurement. Before done-when 2 or 3, time a training step at P2b's recipe with `--train-as-served 1`, chain by chain: the two quantizer chains against the rest of the step (`train-in-space --profile N` and `nsys` where the machine has it). If the cancel rule fires, report it, and the fusion does not start.
- The list's two copies: `src/lib.rs:3-4, :33-36` and `tests/no_training_crate_dep.rs:9, :81-108`. `build.rs` names the three PTX modules at `:26-54` and builds them with nvcc at `:76-95`. The scan must run before the early return at `:68-70`, so CPU builds get the list too.
- The chains and their CPU twins: `hamilton_sheet.rs:646-780` (`q16_residual_ste_scalar :675`, `activation_scale_scalar :691`, `activation_quantize16_ste_scalar :705`). Kernels load at `cuda_stack_kernels.rs:376`; the PTX is embedded at `:85-86`.
- Stale "24" text to correct on the way: `tests/twenty_four_kernel_controls.rs:1-5` and the crate's `Cargo.toml` description.
- **Where it runs.** Kernel development may use Casey's local RTX 4090 (Linux) once it is up. The gates that count, done-when 2 to 4, run on an RTX 5090 pod. That is Casey's call 1 of 4 Nov (#2028 6638209391): qualification stays on 5090 pods until the 4090 bring-up's step 4 shows the cards agree. A 4090 needs its own PTX target (it cannot load `compute_120`).

## Rules that apply

- **Trained as served:** every value the model trains on and serves stays the same, which is what the bit-for-bit gate checks.
- **No squashing, no dense matrix multiplication** (CHARTER §3.2–§3.3). The quantizers are clamps, floors and masks.
- **No NVRTC.** Kernels are nvcc-built PTX in the crate's own modules (`build.rs`).
- **D29's module boundary:** the float crate does not depend on `uor-r4-training`.

## Sources

- Casey's words, 4 Nov 2026, quoted above; #2028 comment 6638209391.
- [claude-157's REPORT](../../loop/tasks/fbs-cost-profile/claude-157/REPORT.md).
- [claude-279's REPORT](../../loop/tasks/one-model-end-to-end/claude-279/REPORT.md), the cached step's cost.
- [served-rounding-cost](served-rounding-cost.md) and [deepseek-5's REPORT](../../loop/tasks/one-model-end-to-end/deepseek-5/REPORT.md), training as served's slowdown.
