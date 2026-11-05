# REPORT: float-kernel-fusion, claude-285 (the fused served quantizers)

Lab: claude. [ORDER](./ORDER.md). PR #3691, item #3678. Plan step: THE ORDER of 4 November (#2028 6634218223), step 5, the kernel unit. Base `33b42d30e`.

**In one paragraph.** The two served quantizers, `activation_quantize16` and `q16_residual`, now each have one fused CUDA kernel a direction. On one RTX 5090 every kernel equals its op chain bit for bit:
- **the ops:** forward and gradient, on edge rows;
- **one training step at P2b's recipe:** the loss and every gradient, at (S) and (L);
- **eight training steps:** the same models, samples and progress bytes as two chain runs.

The kernels sit behind a flag, `train-in-space --fused-quantize 1`, whose default stays 0. What the fusion saves (done-when 4) and whether the flag defaults on (done-when 5) are later units. Run 1 failed two gates; no kernel bit differed from its chain in what run 1 compared. G1 failed on the test side: its reference model of candle's gradients was wrong. G3 failed on the kernel source's form: nvcc compiled four of its adds without explicit rounding. Both were repaired before run 2.

## Tokens per second first

Not measured as a claim; done-when 4 is that measurement. The eight-step equality runs at (S) 192 × 16 gave, by the trainer clock:

| Run | Flag | Train seconds, 8 steps | Train context tok/s |
|---|---|---|---|
| F0a | 0 | 0.963 | 25,530 |
| F0b | 0 | 0.967 | 25,425 |
| F1 | 1 | 0.882 | 27,851 |

F1 is 8.5% faster than the A/A pair's mean, which is more than claude-283's 7.5% ceiling at (S) for removing both chains outright. A fused kernel cannot save more than removing the work, so this short run is not the measure of the saving. It is eight steps, one run per arm, with the first step's warm-up inside the clock.

## What did not work

1. **Run 1** (pin `a93ac5bd0`, `runs/c285`): EXIT 4, one fault on the test side and one in the kernel source's form; no kernel bit differed from its chain in what run 1 compared.
   - **S1 passed:** 350 passed, 0 failed, 12 ignored (154.7 s).
   - **G1, the activation and the residual tests, failed at "the twin's gradient".** Each test's kernel-against-chain forward and gradient checks had already passed on its first case ([9, 8] and [4, 6]).
     - **The difference:** the twin had +0 where the device had −0 (`0x80000000`), exactly where w = −0, or a negative weight met inside = 0.
     - **The cause:** the twin modelled upstream candle, which fills a first gradient contribution with zeros, then adds. The patched candle stores it directly (`backprop.rs` `store_directly`, unless `UOR_CANDLE_GRAD_ZERO_THEN_ADD=1`).
     - **The class:** instrument, the twin's model of candle.
     - **The fix:** the twin stores directly, and the gradients are compared without an accumulation step.
   - **G3 failed** at the first plain `add.f32` without `.rn`: `add.f32 %f96, %f91, 0f00000000`. Reading the PTX on the pod found four, beside 42 `.rn` ops: lines 208 and 457 were `x + 0`, and lines 241 and 543 were `x + 0.5`. That PTX was not kept; run 2's G3 passed on the rebuilt one.
     - **The cause:** nvcc folds `__fmaf_rn(1, x, c)`, whose product is exact, into a plain add, which ptxas may contract. The value is the same today, but the module no longer rounded every op by name.
     - **The class:** form, in the kernel source. It broke done-when 1's rule that every add is a non-contracting intrinsic, and the gate caught a real compiler behaviour.
     - **The fix:** in `fused_quantize.cu`, the four affines with a factor of 1 (an input of 1, or a multiplier of 1) are `__fadd_rn`, one rounding of the same sum, since their product is exact.
   - G2, S2, S3, G3b and E did not run, because the job stops at its first failed gate.
2. **The first pod never started.** `vast-59193389` (South Korea, $0.563/h) never answered SSH, and `uor-pod up` deleted it at 900 s. The five cheapest 5090 offers were all on that one host, and `up` cannot exclude a host. Pinning the driver (`UOR_POD_VAST_DRIVER_VERSION=595.91.07`, which needs all three parts) moved the next `up` to Czechia.
3. **Deviations from the ORDER** (done-whens 1, 2 and 4):
   - **No CPU model compute.** The patched candle refuses every CPU model op (D24/D29: "CPU model compute is unavailable"). So:
     - the ops' CPU forwards refuse with the same message;
     - the f32 twins could not be compared with the chains on the CPU, because the chains cannot run there. They moved into the GPU test module, where G1 compares the kernel, the chain on the device and the twin three ways;
     - the laptop test checks only that, with the flag on, a CPU tensor takes each chain (which then refuses), and that the ops refuse the CPU.
   - **Not every affine is an `__fmaf_rn`.** The four with a factor of 1 are `__fadd_rn`, for run 1's G3 finding. The `.cu` header says why.
   - **An empty input keeps the chain.** `fused_applies` requires `elem_count() > 0`: the chain returns an empty tensor as it is, and the activation op would refuse a row of width 0.
4. **The fresh review's nits, not fixed:**
   - **N1:** G3 cannot tell whether an `fma.rn.f32` came from `__fmaf_rn` or from a contracted multiply and add; the two look the same in PTX. The source has no plain `a * b + c` and `build.rs` passes no `--fmad`, so the fma clause holds by the source, not by G3. G1 is the gate.
   - **N3:** `DIR/fused-quantize.json` is written only when `--fused-quantize` is given, 0 or 1, as the option's docs say. A run without the option writes no record.
   - **N4:** G2 leaves the thread's Hamming-read, turn-memory and served-model state set if a step returns an error. The test fails then anyway; a drop guard would keep later tests on that thread clean.
   - **N5:** the twin's forward comparison treats any NaN as equal to any NaN. Kernel against chain is compared strictly, NaN payload included, and so is the twin's gradient.
   - **The final review's code nits,** left for the done-when 4 unit, which runs G1 again. They are not fixed here because the GPU gates ran at `a1588bcc4`, and a code change after that pin would need another pod run.
     - G1's unbridged activation case runs the chain in both arms, so its bit comparison is an A/A. Its gradient comparison is `None == None`, since candle treats `Floor` as a leaf. The `(1, 0)` count assertion is the gate there; an `assert!(fused.1.is_none())` would state that `x` gets no gradient.
     - The `.cu` header says "an affine whose multiplier is 1". In `row_scale` the two such affines have an input of 1 instead (`ones.affine(unit, 0)` and `ones.affine(2^-bits, 0)`). The argument holds either way, because the product is exact.
     - Two lines are over the width: `fused_quantize.cu:16` and a doc line in `hamilton_sheet.rs`.

## What changed

- **The module** `src/fused_quantize.cu` has four kernels: `activation_quantize16_fwd` and `_bwd`, and `q16_residual_fwd` and `_bwd`.
  - **Rounding:** every f32 add, subtract, multiply and divide is an `__f*_rn` intrinsic, and every affine with no factor of 1 is an `__fmaf_rn`, in the chains' order.
  - **The build:** `build.rs` builds it f32 only as its own PTX module, `fused_quantize_f32.ptx` (module `uor_r4_fused_quantize`), so the pinned `cuda_stack.cu` is unchanged.
  - **The checks:** its four names are in the generated `kernel_lists::FUSED_QUANTIZE_F32`, `admit_launch` admits them, and the banned-token scan covers the file.
- **The ops** `src/fused_quantize.rs`: two candle custom ops for contiguous f32, `activation_quantize16(x)` and `q16_residual(x, bridge)`, each with a CUDA forward.
  - **The backward:** with the bridge on, each has a backward `grad · inside`, with `inside` recomputed from the input by a second kernel.
  - **Bridge off:** `q16_residual` with its bridge off is `apply_op1_no_bwd`, so `x` gets no gradient, as from its chain.
  - **The constants** reach the kernels as arguments, cast as the chains cast them.
- **The flag** in `hamilton_sheet.rs`: the thread-locals `FUSED_QUANTIZE` and `FUSED_COUNTS`.
  - `with_fused_quantize(on, f)` sets both, restores them on drop, and returns the counts.
  - Each chain keeps its early returns first. It then calls the op when the flag is on and the input is a non-empty contiguous f32 CUDA tensor, and counts which path ran.
- **The option** `train-in-space --fused-quantize 0|1` (default 0) needs `--train-as-served 1` and a training run. Given either way, the run writes `DIR/fused-quantize.json` with the flag and the four counts.
- **claude-283's carry-overs:** I-4 in `QuantizerTiming::check`, and `quantizer-timing.invalid.json` for a record whose check fails.
- **The item** `docs/roadmap/items/float-kernel-fusion.md`: "What is missing" item 3 now says where the kernels went and what their gates showed.

## Measurements

Run 2 is on `vast-59195291`: one RTX 5090 in Czechia at $1.002/h, driver 595.91.07, nvcc 12.8. The pin is `a1588bcc4`, the folder `runs/c285b`, and the job ran 02:49:08Z–02:54:09Z, 5 November. It exited 0.

| Gate | What it checks | Result |
|---|---|---|
| S1 | the CUDA library suite, `--lib` | 350 passed, 0 failed, 12 ignored (142.8 s), seven named tests among them |
| G1 | each op against its chain on the device, and its twin, forward and gradient, bit for bit. Rows: seeded random, zeros, −0, subnormals, .5 ties, saturating rows, a [6, 1000] and a [2, 2048] row past 256 threads; `q16_residual` with its bridge on and off; the activation chain unbridged | 2 of 2 passed |
| G3 | the PTX: every f32 add, subtract, multiply and divide a `.rn` form, no `.ftz` | passed (the same run as G1: 3 passed, 0.74 s) |
| G2 | one training step at P2b's recipe, flag on against off: the loss and every variable's gradient, after a flag-off A/A | 1 passed (4.2 s). At 192 × 2 and at 2,048 × 4 the chain step made 12 activation and 50 residual chain calls, and the fused step made 12 and 50 fused calls |
| S2 | claude-283's carry-over, `three_steps_trained_as_served_save_a_step_the_export_leaves_unchanged` | 1 passed |
| S3 | `no_training_crate_dep` (the kernel list and the D29 scan) and the `train-in-space` tests | 2 of 2 and 8 of 8 passed |
| G3b | each PTX module's `.entry` names against its generated list | `cuda_stack_f32` 26, `cuda_stack_bf16` 26, `spiralcore_fbs_arms_f32` 6, `fused_quantize_f32` 4; each equal |
| E | eight steps at (S) 192 × 16, seed 1: F0a and F0b flag off, F1 flag on. Fingerprint: the checkpoints' and `samples.jsonl`'s SHA-256, and `progress.jsonl` without its two timing fields | `equal`: the A/A pair equal, and F1 equal to it. Step 8's `model.safetensors` is `e1126c7c8b9db6aea1c8e93c7d49146b5963dac5cc9fd44549386b8016c78474` in all three. F0a and F0b counted 8,952 activation and 37,300 residual chain calls, and F1 the same numbers fused |

- **Binary:** `train-in-space-a1588bcc` `e24222d61ca4d2efb83bcc5aff5f1f7d4e5540689bd2e2c5644b9e4e24e630fe`.
- **Seals:**
  - run 2: `SHA256SUMS.files` `1b1bf5737a96c948cc191c9e0e2ea234ec517a7ab769bf7d012b13c6b58ad023`, 121 files;
  - run 1: `57ceee3c0fa0efad8a0927101ab22a0eed71cef2f1f53ff0b9bc957094228af6`, 13 files.
- **Store:** `results/claude/float-kernel-fusion-fused-quantize-2026-11-05/c285/` and `.../c285b/`. Both were read back 13 of 13 and 121 of 121 OK before the pod was deleted.
- **Cost:**
  - `vast-59195291` ran 02:37:57Z–02:56:24Z, about $0.31;
  - `vast-59193389` ran 02:19:22Z–02:34:27Z without SSH, at most $0.14 if it billed;
  - so at most $0.45 against the ORDER's $3.

## Checks run at the head

- **Laptop, at `a1588bcc4`:**
  - `cargo test -p uor-r4-float-train --target aarch64-apple-darwin --offline --no-fail-fast`: 288 passed, 0 failed.
  - **The stub-nvcc check** (`cargo check --features cuda --all-targets`): clean, no warnings in the crate.
- **The GPU gates above,** at `a1588bcc4`.
- The commits after `a1588bcc4` change this unit's REPORT, the notebook and the item text only.
- `uor-harness loop report --check`, `loop pr-check --local`, `loop gates --full` and the final fresh review: see the PR's review record.

## Skills used

None beyond the loop's own steps.

## Still open

- **Done-when 4, the cost,** on an RTX 5090 at P2b's recipe. It covers kernel launches and seconds per training step, chain against fused, by the trainer clock (and `nsys`, if the pod has it). It also covers the seconds per cached decode step at 2,048, against claude-279's estimate of 8–9 ms.
- **Done-when 5, the flag's default.** Done-whens 2 and 3 now hold on a 5090, so turning the flag on is that unit's call, after done-when 4.
- **The RTX 4090:** it needs its own PTX target and its own run of these gates before it counts (Casey, 4 Nov, #2028 6638209391).

## Plain summary

Training the model as it is served runs two rounding steps many times per step, each built from dozens of small GPU operations. We wrote one custom GPU kernel for each step, in each direction, and checked them on a rented RTX 5090. The new kernels give exactly the same numbers as the old operations, down to the last bit: in the steps alone, in one full training step, and over eight training steps.

The first try failed two checks. One was in our reference copy of the maths, which did not match how this project's GPU library adds up gradients. The other was the compiler turning four "add with exact rounding" instructions into plain adds. Both are fixed. The new kernels are off by default; the next step measures how much time they save.
