# ORDER: float-kernel-fusion, claude-285

Lab: claude. Item: [float-kernel-fusion](../../../../roadmap/items/float-kernel-fusion.md). Base: `33b42d30e`.

## Goal

The item's done-when reads:

> 1. **The kernel list is generated.**
>    - `build.rs` scans each `MODULES` entry's combined `sources` (`build.rs:26-54`) for `extern "C" __global__ void <name>` on every build, CPU builds included. It writes one list per PTX module, which `lib.rs` includes. The arms module is built from three files, `spiralcore_fbs_arms_prelude.cu`, `spiralcore_fbs.cu` and `spiralcore_fbs_arms.cu` (`:44-53`). So its list is the 4 FBS kernels plus `spiralcore_fbs_arm_fwd` and `spiralcore_fbs_arm_bwd`.
>    - The dispatch refuses a name not in its own module's list. The arms kernels, launched through `launch_prebuilt` (`geometric_stack_native_cuda.rs:798-805`), must pass that check, not fail it.
>    - No literal kernel count is left in code, tests or their doc comments: `lib.rs:3-4, :33-35`, `tests/no_training_crate_dep.rs:3-4`, and the messages at `:83` and `:104`.
>    - The test re-scans the sources itself and compares with the generated lists. The banned-token scan stays.
> 2. **Fused `activation_quantize16`,** behind a flag, equal bit for bit to the op chain it replaces. The flag is a thread-local in `hamilton_sheet.rs`, where the served wrappers switch to the kernel, set by a `train-in-space` option. It is equal:
>    - forward and backward, on random rows that include zeros, −0, saturating rows, subnormals and exact .5 ties;
>    - inside one training step at P2b's recipe: the loss and every gradient.
>    The order of the bridge's ops (`free = x · inside`, `d = q − free`, `out = free + d`) and nvcc's `--fmad` choice are part of the gate.
> 3. **Fused `q16_residual`,** the same way.
> 4. **The cost,** on an RTX 5090 at P2b's recipe: kernel launches and seconds per training step, chain against fused, by the trainer clock (and `nsys`, if the pod has it). Also the seconds per cached decode step at 2,048, set against claude-279's estimate of 8–9 ms.
> 5. The flag defaults on only when 2 and 3 hold. One notebook row and a REPORT.

**This unit takes done-whens 2 and 3**, under the plan claude-284 recorded (its ORDER, "The plan for claude-285"), copied below as this unit's done-when. Done-when 1 is claude-280's, and done-whens 4 and 5 are a later unit's.

## Claim check

- **(a) The step it advances:** THE ORDER of 4 November (#2028 6634218223), step 5, the kernel unit, part (c) after the measurement (claude-283) and the probe (claude-284). Casey approved the fused kernels on 4 Nov (#2028 6638209391): "Yes, please build this too to do it the smarter way going forward."
- **(b) What it changes in the model that trains or serves:** nothing in its values. Behind a flag that defaults off, the trained-as-served forward and backward run the two served quantizers as one kernel each, in place of claude-283's measured ~1,240 stock ops per forward (12 activation calls and 50 residual calls). The gate requires every value, and every gradient, to be equal bit for bit. The integer session that serves is untouched.

## Cancel probe (run first)

1. **The probe's base still holds.** claude-284's probe read the chains at `bff50c993`. `git diff --stat bff50c993 33b42d30e -- crates/` is empty, so `hamilton_sheet.rs`'s two chains, their callers and the crate's kernel modules are unchanged at this base.
2. **No open order overlaps.** `loop claim` checked this item's scope against every open claim and opened this unit, so no other lab holds `hamilton_sheet.rs` (served-rounding-cost, #3594, shares it) or the float crate's kernel files.
3. **The rounding argument.** claude-284 found every op in both chains to be one IEEE binary32 rounding on CUDA. Candle's kernels are built with `-ftz=false -prec-div=true -prec-sqrt=true` and no fast math, and the clamp is `fmaxf` then `fminf`. Its probe also found that the gradient with the bridge on is `grad · inside`, and that the bridge-off `q16_residual` gives `x` no gradient.
4. **One host-only difference, found here.** Candle's CPU `maximum` and `minimum` are `if a < b { b } else { a }` and its mirror (`candle-core 0.9.2 src/op.rs:359-372`; the crate enables no `mkl` or `accelerate`, so no vector path replaces them). CUDA's are `fmaxf` and `fminf` (`candle-kernels 0.9.2 src/cuda_utils.cuh:142-144`, `binary.cu:69, :74`). So on a NaN the CPU chain and the CUDA chain disagree. On finite values the two agree.
   - **The kernels:** they follow the CUDA chain, the one they replace.
   - **The CPU twins:** they use `f32::max` and `f32::min` (`fmaxf`), so they model the kernels.
   - **The CPU tests:** they compare twin and chain on finite rows only, infinities included; NaN rows are out of scope.

**Result: not cancelled. Class: instrument** (a question about the numerics, answered by reading them; nothing about the mechanism).

## Files in scope

- `crates/uor-r4-float-train/build.rs`
- `crates/uor-r4-float-train/src/lib.rs`
- `crates/uor-r4-float-train/src/cuda_stack.cu`
- `crates/uor-r4-float-train/src/cuda_stack_kernels.rs`
- `crates/uor-r4-float-train/src/fused_quantize*.rs`
- `crates/uor-r4-float-train/src/fused_quantize*.cu`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/tests/no_training_crate_dep.rs`
- `crates/uor-r4-float-train/tests/twenty_four_kernel_controls.rs`
- `crates/uor-r4-float-train/tests/fused_*.rs`
- `crates/uor-r4-float-train/Cargo.toml`
- `docs/harness/prep/float-kernel-fusion*`
- `docs/loop/tasks/float-kernel-fusion/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/float-kernel-fusion.md`

Of these, this unit touches only: `build.rs`, `src/fused_quantize.cu`, `src/fused_quantize.rs`, `src/lib.rs`, `src/cuda_stack_kernels.rs`, `src/hamilton_sheet.rs`, `src/bin/train-in-space.rs`, `src/in_space_long.rs` (its `tests` module only), `tests/fused_*.rs`, `tests/no_training_crate_dep.rs` (only if the scan needs it), this task folder, the notebook and the item file. `cuda_stack.cu` is pinned, so this unit does not touch it.

## Done when

1. **The module.**
   - `src/fused_quantize.cu` holds four kernels: `activation_quantize16_fwd`, `activation_quantize16_bwd`, `q16_residual_fwd` and `q16_residual_bwd`.
   - It is built f32 only, as its own PTX module. Its kernels are in the generated kernel list, admitted by `admit_launch`, and covered by the banned-token scan.
   - Every f32 add, subtract, multiply and divide is a non-contracting intrinsic (`__fadd_rn`, `__fsub_rn`, `__fmul_rn`, `__fdiv_rn`), and every affine is an `__fmaf_rn`, in the chain's order. So its arithmetic does not depend on nvcc's `--fmad` choice.
2. **The ops.** `src/fused_quantize.rs` has two candle custom ops, for contiguous f32 only, each with a CUDA forward.
   - **The backward:** with its bridge on, each op has a backward, `grad · inside`, with `inside` recomputed from the input.
   - **Bridge off:** `q16_residual`'s bridge-off form has no gradient, as its chain has none. Its output is untracked (`apply_op1_no_bwd`).
   - **The twins:** each kernel has a CPU twin in f32, in the kernel's order.
   - **The constants:** `2^-ACTIVATION_EXP`, the 2^62 saturation, `Q16_SCALE` and its inverse reach the kernels as arguments, cast as the chain casts them, not as literals.
3. **The flag.** A thread-local in `hamilton_sheet.rs`.
   - **When it applies:** when the flag is on and the input is a contiguous f32 CUDA tensor, `activation_quantize16` and `q16_residual` call the ops. Otherwise the chains run.
   - **The early returns stay first:** `q16_residual`'s `!is_rounding_residual_q16()` and both chains' `timing_skips`, so the timing switch's skips and counts are unchanged.
   - **The option:** `train-in-space --fused-quantize 1` sets it. The default is 0; whether it becomes on is done-when 5's call, for a later unit.
   - **The record:** the run writes `DIR/fused-quantize.json` with the flag and each path's call count.
4. **CPU tests on the laptop:**
   - the twins equal the chains on the CPU bit for bit, forward and gradient, on the edge rows;
   - with the flag on, a CPU tensor still takes the chain;
   - the option parses, and its default is off.
5. **GPU gates on one RTX 5090 pod** (ignored tests, run with `--ignored`):
   - **G1, the ops.** Each op's forward and backward equal its chain bit for bit.
     - **The rows:** seeded random rows plus zeros, −0, subnormals, exact .5 ties, and saturating rows. Saturating rows are rows whose maximum reaches the 2^62 integer clamp, values that land in the ±32767 clamp, and `q16_residual` values past ±2^31 / 2^16.
     - **The bridge:** each chain runs with its bridge on, and `q16_residual` with it off too. There the gate is that `x` gets no gradient from either form.
   - **G2, one training step at P2b's recipe** (`FbsArm::OneAtomActive`, Hamming reads, turn memory, trained as served), at (S) 192 × 2 and (L) 2,048 × 4.
     - **The gate:** the loss and every variable's gradient are equal bit for bit, flag on against off.
     - **The A/A first:** two flag-off steps must be bit-equal, or the gate is an instrument fault.
     - **Why it matters:** a fused op is one graph node where the chain had many, so candle may sum a tensor's gradients in another order. G2 is the gate that would show it.
   - **G3, the PTX** (supporting evidence; G1 is the gate).
     - **The forms:** the module's f32 add, subtract, multiply and divide are all `.rn` forms (`add.rn.f32`, `sub.rn.f32`, `mul.rn.f32`, `div.rn.f32`), so ptxas may not contract them, and no instruction is a `.ftz` form.
     - **The fma:** an `fma.rn.f32` can come only from an affine's `__fmaf_rn`, since the source has no plain `a * b + c` for nvcc to contract.
     - **What catches what:** G1's subnormal rows would catch a flush. Its random rows, with their exact .5 ties, would catch a contraction or a reordered op.
   - **The CUDA library suite, and claude-283's carry-over:** `in_space_long::tests::three_steps_trained_as_served_save_a_step_the_export_leaves_unchanged` with `--ignored`.
6. **claude-283's two other carry-overs:**
   - `QuantizerTiming::check` requires each forward's skipped calls to equal its kept calls (I-4);
   - a record that fails its check is written as `quantizer-timing.invalid.json`, and the checked name only after `check()` passes.
7. **The REPORT,** negatives first, with one notebook row and the item's text after the result.

## Stop and ask if (stop and decide; never wait)

- **A gate differs in any bit.** First check the instrument: the A/A, and the counts that show both paths ran. Then repair the kernel. If a difference cannot be removed, the flag stays off, and the REPORT names the rows, the op and the bits.
- **The pod or a cap.** At most one 5090 pod and a $3 bound for the unit. A watcher on every job, and the results off the pod before `down`.
- **The diff passes 600 lines.** Split the fresh review in two.
