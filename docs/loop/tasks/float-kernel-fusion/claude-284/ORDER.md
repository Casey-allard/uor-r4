# ORDER: float-kernel-fusion, claude-284

Lab: claude. Item: [float-kernel-fusion](../../../../roadmap/items/float-kernel-fusion.md) (#3678). Base: `bff50c993`.

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

Done-when 1 merged in claude-282 (#3684). claude-283 (#3685) found the cancel rule does not fire.

**This unit lands the cancel probe and the scope the kernels need.** The loop's scope rule reads an item's scope at the merge base (`loopprcheck.rs:431-446`), and a separate `infra-loop-harness` claim is refused while this item's claim is open: its `docs/roadmap/items/*.md` overlaps this item's own file. So the scope reaches main in this unit. **claude-285 then takes done-whens 2 and 3 together,** under the plan below: they share one kernel module, one flag and one GPU gate pod. Done-whens 4 and 5 follow.

## Claim check

- **The step it advances:** THE ORDER's step 5, the kernel unit (Casey, 4 Nov, #2028 6638209391: "Yes, please build this too to do it the smarter way going forward"). It serves every later trained-as-served run (P2b's and P5's).
- **What it changes in the model:** this unit, nothing; it unblocks claude-285. That unit changes nothing the model computes either: under `--train-as-served 1` and the new flag, each served quantizer runs as one kernel a direction, instead of 51 stock ops (`activation_quantize16` with its bridge) and 14 (`q16_residual`), with equal values. The flag is off by default.

## Cancel probe (run first)

**Question:** can one kernel reproduce each chain bit for bit, and is the work still open?

**What was checked:**
1. **Every op the chains launch has one IEEE binary32 rounding on CUDA.** The vendored `third_party/candle-kernels-0.9.2` is built with `-ftz=false -prec-div=true -prec-sqrt=true` and no fast math. The ops are:
   - `abs` (`fabsf`), and the row max (`fmaxf` from `-INFINITY`, exact in any order);
   - `clamp`, which is `maximum` then `minimum` (`fmaxf`, `fminf`);
   - compares against scalars, each bound cast f64 → f32 to nearest (`cmp`, `tensor.rs:1125-1131`; `maximum` and `minimum` cast the same way in `binary_op_scalar`, `:112-134`). Every bound the chains use is an f32 value after that cast: powers of two (the 2^62 saturation, `hamilton_sheet.rs:616`, included), ±32767 and −2^31 are exact, and `i32::MAX` rounds to 2^31, which the chain's own CPU twin notes (`:686`);
   - `where_cond`, `floor` (`floorf`), and broadcast multiply and divide (IEEE, precise division);
   - `affine`, which is `x * mul + add`. The build passes no `--fmad`, so nvcc may contract it to an fma. For the chains' constants (a power of two with add 0, or mul 1), the contracted and separate forms give the same bits, −0 included: a −0 product plus +0 is +0 both ways.
2. **The gradient.** `floor`, compares and `detach` carry none (`third_party/candle-core-0.9.2/src/backprop.rs:582-587`).
   - **With its bridge on,** each chain's input gradient is `grad · inside`: the bridge's `free = x · inside` is its only path back to `x`. Candle adds a custom op's input gradient to zeros exactly as it adds the chain's (`:640-643`).
   - **With its bridge off,** `q16_residual` returns `rounded`, whose only path back to `x` runs through `floor`. So `x` gets no gradient from it at all: no entry, not zeros.
3. **The callers.** `activation_quantize16` runs only under its bridge: `served_activation_quantize16` (`hamilton_sheet.rs:773`) is its only caller outside tests. `q16_residual` also runs without its bridge (the Q16 control in evaluation), so its kernel has both forms.
4. **Overlap.** The open PRs are #3689 (this unit) and #3675 (kimi, the harness IDE); neither touches the float crate's kernels.
5. **The pin.** `cuda_stack.cu`'s SHA-256 at the base (`c577fce9…`) is pinned as earlier sheets' source identity. `git grep` at the base finds it in 12 files:
   - in code: `spiralcore_fbs_sheet.rs:54`, whose test at `:4156` asserts `pins_match_sheet`, and `docs/labs/recurrence-gates-six-recurrence-fit/runner/src/lib.rs`;
   - in records: the receipts and probes of `docs/labs/spiralcore-fbs-offline-float-sheet` (four files), `docs/labs/recurrence-gates-six-recurrence-fit` and `docs/labs/r-secant-surrogate-descent` (receipt and README);
   - in text: two prep sheets (`docs/harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md`, `recurrence-gates-six-recurrence-fit-2026-10-25.md`) and antigravity-82's REPORT.

   A kernel appended to `cuda_stack.cu` turns that test red and re-identifies those sheets' source.

**Result:** not cancelled. **Class: instrument.** It decides where the kernels go, not whether they can work. They go in their own source and PTX module, as the SpiralCore arms did (`spiralcore_fbs_arms.rs:4`), and `cuda_stack.cu` stays unchanged.

## Decision: two scope additions (claude's own decision review)

- **`crates/uor-r4-float-train/src/fused_quantize*.cu`:** the kernels' own source, built by `build.rs` as their own PTX module. Why: probe item 5, and the owner's 13 Oct direction that new kernels are nvcc-built PTX in their own module. No earlier sheet's pin moves.
- **`crates/uor-r4-float-train/src/in_space_long.rs`, for a test in its `tests` module (from `:11199`):** the one-step gate needs the served model parts and the step's loss. `served_parts` (`:16095`) and `served_vars` (`:16124`) are in that module; `with_training_forward` (`:295`) and `batch_loss` (`:1116`) are private to the file, so only code in it reaches them. The glob admits the whole file, so "tests only" is this ORDER's commitment, which the review checks; no line outside `tests` changes.
- **Reversal cost:** low; the files are new or test-only. The item's scope line records both in this unit.

## Files in scope (this unit)

- `docs/loop/tasks/float-kernel-fusion/claude-284/`: this ORDER and the REPORT.
- `docs/roadmap/items/float-kernel-fusion.md`: the scope line, and one *How to start* bullet saying where the kernels go and why.

## Done when (this unit)

1. The cancel probe and the scope decision above, recorded.
2. The item's scope adds `crates/uor-r4-float-train/src/fused_quantize*.cu` and `crates/uor-r4-float-train/src/in_space_long.rs`, and its *How to start* says why.
3. The REPORT, negatives first. No notebook row: no result is measured.

## Stop and ask if (this unit; stop and decide, never wait)

- **The scope edit is refused at the head** (`loop pr-check`). Then the scope goes through whichever unit the harness admits, and this unit closes on the probe alone.

## The plan for claude-285 (done-whens 2 and 3)

### Files in scope

- `crates/uor-r4-float-train/build.rs`: the new module.
- `crates/uor-r4-float-train/src/fused_quantize.cu`: the four kernels.
- `crates/uor-r4-float-train/src/fused_quantize.rs`: the two ops and their CPU twins.
- `crates/uor-r4-float-train/src/lib.rs`: the module line.
- `crates/uor-r4-float-train/src/cuda_stack_kernels.rs`: the module's name, its PTX and a grouped prebuilt launch.
- `crates/uor-r4-float-train/src/hamilton_sheet.rs`: the flag, its dispatch and counts, and the carry-overs.
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`: the option, the manifest field and the carry-overs.
- `crates/uor-r4-float-train/src/in_space_long.rs`: the one-step gate test only.
- `crates/uor-r4-float-train/tests/fused_*.rs`, `tests/no_training_crate_dep.rs`: the GPU gates and the scan.
- `docs/loop/tasks/float-kernel-fusion/**`, `docs/research/notebook/float-training.md`, `docs/roadmap/items/float-kernel-fusion.md`.

### Done when

1. **The module.** `src/fused_quantize.cu` holds `activation_quantize16_fwd`, `activation_quantize16_bwd`, `q16_residual_fwd` and `q16_residual_bwd`, built f32 only as its own PTX module, in the generated kernel list, admitted by `admit_launch`, and covered by the banned-token scan. Every f32 add, subtract, multiply and divide is a non-contracting intrinsic (`__fadd_rn`, `__fsub_rn`, `__fmul_rn`, `__fdiv_rn`) and every affine an `__fmaf_rn`, in the chain's order. So its arithmetic does not depend on nvcc's `--fmad` choice.
2. **The ops.** `src/fused_quantize.rs` has two candle custom ops for contiguous f32 only, each with a CUDA forward. With its bridge on, each has a backward, `grad · inside`, with `inside` recomputed from the input. `q16_residual`'s bridge-off form has no gradient, as its chain has none: its output is untracked (`apply_op1_no_bwd`). Each kernel has a CPU twin in f32, in the kernel's order. The scale constants (`2^-ACTIVATION_EXP`, the 2^62 saturation, `Q16_SCALE` and its inverse) reach the kernels as arguments cast as the chain casts them, not as literals.
3. **The flag.** A thread-local in `hamilton_sheet.rs`. When it is on and the input is a contiguous f32 CUDA tensor, `activation_quantize16` and `q16_residual` call the ops; otherwise the chains run. Their early returns stay first: `q16_residual`'s `!is_rounding_residual_q16()`, and both chains' `timing_skips`, so the timing switch's skips and counts are unchanged. `train-in-space --fused-quantize 1` sets it (default 0; the default is done-when 5's, a later unit), and the run writes `DIR/fused-quantize.json` with the flag and each path's call count.
4. **CPU tests on the laptop:**
   - the twins equal the chains on the CPU bit for bit, forward and gradient, on the edge rows;
   - with the flag on, a CPU tensor still takes the chain;
   - the option parses, and its default is off.
5. **GPU gates on one RTX 5090 pod** (ignored tests, run with `--ignored`):
   - **G1, the ops:** each op's forward and backward equal its chain bit for bit. The rows are seeded random rows plus zeros, −0, subnormals, exact .5 ties, and saturating rows: rows whose maximum reaches the 2^62 integer clamp, values that land in the ±32767 clamp, and `q16_residual` values past ±2^31 / 2^16. Each chain runs with its bridge on, and `q16_residual` with it off too; there, the gate is that `x` gets no gradient from either form.
   - **G2, one training step at P2b's recipe** (`FbsArm::OneAtomActive`, Hamming reads, turn memory, trained as served), at (S) 192 × 2 and (L) 2,048 × 4: the loss and every variable's gradient are equal bit for bit, flag on against off. First an A/A: two flag-off steps must be bit-equal, or the gate is an instrument fault. A fused op is one graph node where the chain had many, so candle may sum a tensor's gradients in another order; G2 is the gate that would show it.
   - **G3, the PTX (supporting evidence; G1 is the gate):** the module's f32 add, subtract, multiply and divide are all `.rn` forms (`add.rn.f32`, `sub.rn.f32`, `mul.rn.f32`, `div.rn.f32`), so ptxas may not contract them, and no instruction is a `.ftz` form. An `fma.rn.f32` can come only from an affine's `__fmaf_rn`: the source has no plain `a * b + c` for nvcc to contract. G1's subnormal rows are what would catch a flush, and its random rows, with their exact .5 ties, a contraction or a reordered op.
   - **The CUDA library suite,** and claude-283's carry-over: `in_space_long::tests::three_steps_trained_as_served_save_a_step_the_export_leaves_unchanged` with `--ignored`.
6. **claude-283's two other carry-overs:** `QuantizerTiming::check` requires each forward's skipped calls to equal its kept calls (I-4). A record that fails its check is written as `quantizer-timing.invalid.json`; the checked name is written only after `check()` passes.
7. **The REPORT,** negatives first, with one notebook row and the item's text after the result.

### Stop and ask if (stop and decide; never wait)

- **A gate differs in any bit.** First the instrument: the A/A, and the counts that show both paths ran. Then repair the kernel. If a difference cannot be removed, the flag stays off, and the REPORT names the rows, the op and the bits.
- **The pod or a cap.** At most one 5090 pod, a $3 bound for the unit, a watcher on every job, and results off the pod before `down`.
- **The diff passes 600 lines.** Split the fresh review in two.
