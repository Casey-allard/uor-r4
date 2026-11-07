# Prep sheet: the fused quantizers on b64L at short and long context

Lab: deepseek. Item: [float-kernel-fusion](../../roadmap/items/float-kernel-fusion.md)
(#3678). Order: [deepseek-1](../../loop/tasks/float-kernel-fusion/deepseek-1/ORDER.md).

TEST FITNESS: FIT - M2 #2030 (a fresh non-author Codex review at `820a1103b`, 7 Nov). Its
conditions are in the Verdict section. Compute remains deferred by the execution gate above.

## Why

Claude's one-model path trains as served. Two rounding chains run throughout its forward:

- `activation_quantize16`, with its straight-through bridge, is 49 tensor operations a call,
  excluding its `.detach()` bookkeeping;
- `q16_residual`, with its bridge, is 13 tensor operations, with the same exclusion.

claude-283 measured removal of both chains at 7.51% of a step at 192 x 16 and 2.58% at
2,048 x 4. claude-285 then built one fused forward and backward kernel for each chain and showed
the fused path bit-identical:

- each op against its chain on edge rows;
- one training step's loss and every gradient, at short and long context;
- eight steps' models, samples and progress values.

The true fused gain was never measured, and the training flag still defaults off. When training
repeats resume after the current pipeline-first work, their measured step is 0.63-0.70 s. This
experiment decides whether those future served-training runs default to the smaller launch
graph. The item's cached-decode half of done-when 4 remains a later unit because the current
flag is training-only.

**Execution gate:** the newest project direction is pipeline first. No GPU is leased while the
current one-model export, integer-serving and parity path remains the owner's prerequisite. The
sheet may receive FIT and the source and scripts may receive a D67 review now; the launch waits
for that direction to clear or for the top session's decision review to name this microbenchmark
as an exception.

## Question

At the current (b64L) human-text recipe on one RTX 5090, how much trainer-clock time per step
does the fused quantizer path save against the stock chains at both short and long context, and
do all saved training artifacts remain identical?

## Base artifact

- **Pin:** the reviewed head of `deepseek/float-kernel-fusion`, whose source starts at
  `f66f023d1`. The final reviewed commit and binary SHA-256 are recorded in the pre-run review.
  Any source change after review gets a new pin and review.
- **Build:** `UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR=/workspace/fusion/target`, with
  `cargo build --release --locked -p uor-r4-float-train --features cuda --bin train-in-space`.
- **Initial model:** fresh seed-1 initialization in every run. No checkpoint is resumed.

## Data split

The pod stages these exact inputs at the destinations below:

| Input | Store path | Pod destination | SHA-256 |
|---|---|---|---|
| TRAIN D3 | `results/claude/scale-probe-2026-11-05/inputs/d3` | `/workspace/fusion/data/train/d3/` | `tokens.u16` `ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52`; `response_mask.u8` `ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875` |
| DEV, in-run checkpoint evaluation | `w2-inputs/2026-10-25/driver-data/ft/dev` | `/workspace/fusion/data/ft/dev/` | `tokens.u16` `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; `response_mask.u8` `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| tokenizer | `w2-inputs/2026-10-25/driver-data` | `/workspace/fusion/data/tokenizer.json` | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |

The trainer checks D3's two hashes through `--train-sha256` and
`--train-mask-sha256`, and DEV and the tokenizer through its pinned source constants. No
held-out panel is scored.

## Arms

Both shapes use TRAIN D3, seed 1, the same objective and 256 steps:

```text
--data-root /workspace/fusion/data
--train-split /workspace/fusion/data/train/d3
--train-sha256 ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52
--train-mask-sha256 ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875
--objective nearest
--retain-rates 1
--decay-mult 1
--lr 0.001
--lr-end 0
--train-as-served 1
--width 256
--hamming-read 4
--steps 256
--eval-every 64
--seed 1
--samples 0
```

**(S)** adds `--context 192 --batch 64 --micro-batch 16`, which is four micro-batches of 16 and
12,288 positions an optimizer step. This is the current b64L shape, not claude-283's
192 x 16 batch.

**(L)** adds `--context 2048 --batch 4 --micro-batch 0`.

Each run adds one mode and one new output root:

```text
--fused-quantize MODE --out /workspace/fusion/runs/S-A0
```

The registered sequence is:

| Shape | Run | Mode |
|---|---|---|
| (S) | A0 | `0` |
| (S) | B0 | `1` |
| (S) | B1 | `1` |
| (S) | A1 | `0` |
| (L) | A0 | `0` |
| (L) | B0 | `1` |
| (L) | B1 | `1` |
| (L) | A1 | `0` |

## Readings

The checkpoint callback runs after steps 64, 128, 192 and 256. Its `train_seconds` value is
written before checkpoint evaluation, so differences between consecutive rows measure training
windows without evaluation time.

- **R1, per-window rate.** For each run and for each interval `64 -> 128`, `128 -> 192` and
  `192 -> 256`, compute `64 / delta_train_seconds`. The first 64 steps are warm-up and are not
  retained.
- **R2, gain and absolute cost.** Per shape, pool the six OFF window rates and six ON window
  rates from the two runs of each mode. Let `off` and `on` be their medians, and
  `gain = (off - on) / off`. Report `1 / off` and `1 / on` as seconds per optimizer step, and
  `off x batch x context` and `on x batch x context` as training context tokens/s.
  - **Extension:** if the bar reaches row 4, add `A2, B2, B3, A3` and pool twelve rates per
    mode instead. Recompute `off`, `on`, both absolute costs and `gain` from those twelve-rate
    medians.
- **R3, noise.** Per shape, take the largest of:
  - the three-window mean for A0 against A1;
  - the three-window mean for B0 against B1;
  - the largest within-run spread between a window rate and that run's median rate;
  - the registered floor: 0.46% at (S), 0.25% at (L).
  Normalize each spread by that shape's OFF median.
  - **Extension:** repeat both pair comparisons for A2 against A3 and B2 against B3, take the
    largest within-run spread across all eight runs, and retain the same floor. The bar uses the
    resulting extended R3.
- **R4, exactness.**
  - For every shape and every step in `0,64,128,192,256`, every file under each run's model
    directory is compared by relative path and SHA-256 across A0, B0, B1 and A1. This includes
    `model.safetensors`, `hamming_read.safetensors`, and every JSON or learned table saved there.
  - Each run's `progress.jsonl` is serialized with only `train_seconds` and
    `train_context_tokens_per_second` removed. The four normalized files must be byte-identical.
  - Each run root's `config.json` is byte-identical across the four mode runs, and any
    `samples.md` or `samples.jsonl` the run creates is byte-identical. This is the complete
    deterministic run output apart from `progress.jsonl`'s two timing fields and the expected
    `fused-quantize.json`.
  - If row 4 runs the extension, the same model-file, run-config, samples and normalized-progress
    checks apply across A0, B0, B1, A1, A2, B2, B3 and A3 at all five checkpoints. An extension
    run that cannot join that equality set stops the timing read.
  - The ON runs must report nonzero fused activation and residual calls; the OFF runs must
    report nonzero chain calls for each. The expected modes are also read from each
    `fused-quantize.json`.
- **R5, launches.**
  - At each shape and mode, run a 1-step and a 32-step count smoke with `--eval-every 64`,
    `--samples 0`, and all other flags unchanged. Each smoke has exactly two checkpoint
    evaluations, at step 0 and at its final step. The difference between the 32-step and 1-step
    `fused-quantize.json` counts isolates 31 training steps.
  - If `nsys` exists, profile both smokes under
    `nsys profile --trace=cuda --sample=none --stats=true` and use the difference between their
    total kernel launches, divided by 31, as launches per training step. Retain both reports.
  - If `nsys` is absent, report the isolated per-training-step calls for both paths, plus the
    source's 49 activation-chain and 13 residual-chain tensor-operation counts as forward-only
    implementation counts. These are not kernel-launch counts. Do not multiply them by two: the
    straight-through backward graph does not repeat the forward operation chain. Report ON as two
    fused launches per call only after I-0 confirms its backward kernel. If total launches are
    required, record them as UNAVAILABLE rather than inventing a count.

## Bar

Every row requires R4. The first matching row decides:

| Order | Reading | Result |
|---|---|---|
| 1 | R4 fails | No speed reading. Classify and repair the instrument or kernel; default stays off. |
| 2 | Either shape has a non-positive gain, or a gain no larger than its noise | Keep the default off and record the measured boundary. |
| 3 | Both shapes have `gain > 2 x noise` | Default served training to fused quantizers; explicit `0` remains the reference. |
| 4 | Otherwise | Run one complete `OFF, ON, ON, OFF` block for each unresolved shape on the same pod, within the owner caps, and re-read with all windows. |
| 5 | An extension run fails R4 or I-1 to I-5 | Stop without a speed reading or default change. |
| 6 | After the single additional block, both shapes have pooled `gain > 2 x noise` | Default served training to fused quantizers. |
| 7 | After the single additional block, either shape remains at or below pooled `2 x noise` | Keep the default off and record the result as inconclusive after the registered extension. |

The bar is about cost and output equality, not model quality. A positive result licenses changing
the source default for CUDA served training, with the RTX 5090 as its qualified performance
evidence and other cards remaining unqualified. It is not a serving or cached-decode result.

## Instrument checks

- **I-0:** on the pod, with `UOR_NVCC_ARCH=compute_120`, run the CUDA library suite with:

  ```text
  cargo test --release --locked -p uor-r4-float-train --features cuda --lib -- --test-threads=1
  ```

  Then run each named ignored test separately with:

  ```text
  cargo test --release --locked -p uor-r4-float-train --features cuda --lib TEST_NAME -- --ignored --exact --test-threads=1
  ```

  The four named ignored tests are:
  - `in_space_long::tests::one_step_as_served_is_bit_equal_with_the_fused_quantizers`;
  - `fused_quantize::tests::gpu::g1_the_activation_kernels_equal_the_chain_and_the_twin_bit_for_bit`;
  - `fused_quantize::tests::gpu::g1_the_residual_kernels_equal_the_chain_and_the_twin_bit_for_bit`;
  - `fused_quantize::tests::gpu::g3_the_ptx_rounds_every_float_op_to_nearest_and_flushes_nothing`.
  Each of the four `#[ignore]` commands uses `--ignored --exact` and must report exactly
  `1 passed; 0 failed`; zero selected tests is a failure.
- **I-1:** every run exits 0 and has progress rows at steps 64, 128, 192 and 256.
- **I-2:** D3's two configured hashes and the hard-coded DEV/tokenizer hashes match the table.
  `fused-quantize.json`, not `config.json`, records the requested path.
- **I-3:** each `fused-quantize.json` has the requested mode and the expected nonzero path
  counts. An OFF run with any fused call, or an ON run with a chain call on CUDA, fails I-3.
- **I-4:** R4's step-0 comparison passes before the speed readings are used.
- **I-5:** one lease holds the GPU throughout. Record the GPU UUID, `nvidia-smi` clocks, power,
  temperature, pstate and utilization after warm-up and after the last run; record host CPU
  model, allocation, load and memory. No other process may hold the GPU.

## Steps

1. Receive FIT and a D67 pre-run review of the exact pin, binary build, commands, equality
   grader, seal and watcher. Before creating a pod, re-read the current plan and newest notebook
   row. If pipeline-first still stands, deliver the reviewed preparation with compute NOT_RUN.
2. Run `uor-pod status`; create and lease one RTX 5090 at no more than $1.15/h inside the
   shared caps. Never touch another lab's pod.
3. Bootstrap the reviewed pin; build the binary; record its SHA-256, `nvcc --version`, driver,
   `UOR_NVCC_ARCH` and GPU identity; run I-0.
4. Stage the inputs and verify every hash above.
5. Run `S-A0, S-B0, S-B1, S-A1, L-A0, L-B0, L-B1, L-A1`, each into a new root.
6. Run the eight 1-step/32-step count smokes, each with `--eval-every 64 --samples 0`; wrap
   both members of each pair with `nsys` if available.
7. Seal with `SHA256SUMS.files`, `DONE` and `EXIT`; upload, list and read back before taking
   the pod down.
8. Grade R1-R5 and I-1 to I-5 from the read-back copy. Apply the bar, change the default only
   if earned, run the laptop checks, and write the REPORT and notebook row.

## Minimum test power

Six post-warm-up 64-step windows per mode at each shape, from two process runs in balanced
`OFF, ON, ON, OFF` order. The rate uses the trainer's synchronized whole-step clock. The
registered noise includes both cross-process and within-run variation. A result that does not
clear twice that noise adds a complete balanced block rather than adopting a small point
estimate. This is a deterministic structural-performance measurement, not a model-quality
decisive run: the same seed and inputs are deliberate because any model difference is a failed
instrument check. Repetition comes from two independent processes per arm initially and four
after the one registered extension, with three time windows per process and five exact
checkpoints per run.

## Cost

- **Preparation and build:** about 8 minutes.
- **I-0:** about 4 minutes.
- **Training:** eight runs x 256 steps, at about 0.63-0.70 s a step, about 24 minutes.
- **Eight count smokes and optional `nsys`:** about 8 minutes.
- **Seal and read-back:** about 5 minutes.
- Planned total: about 50 minutes, with an optional balanced extension adding about 24 minutes.
  Include one full 50-minute retry projection for a pod or bootstrap failure: 124 minutes and
  about $2.38 at $1.15/h. Add the extension only if the first run reaches its registered
  inconclusive branch. The planning bound is $3; a healthy overrun is extended and recorded.
- **Storage:** inputs about 0.1 GiB; eight output roots and five checkpoints each, about 4-6
  GiB temporary; eight count-smoke roots about 2 GiB; build target about 12 GiB; retained seal
  about 2 GiB. The extension adds its own roots within the same estimates. The pre-run records
  current free space, requires the project's 30 GiB + 128 MiB floor, and records the
  absent/present model ledger rather than inventing a balance.
- **Host:** one trainer process, no CPU model compute; record host CPU, thread allocation,
  available RAM, peak RAM and load in the pre-run. Record the active storage receipt or its
  absence before launch.
- **Hard limits:** the owner's four-pod and $8/h shared caps, the disk floor, and a failed or
  diverged run. A healthy overrun is extended and recorded, not cancelled.

## Stop rule

- On a failed I-0, input hash, I-3 or run exit, seal evidence and do not read speed.
- On an R4 mismatch, stop the timing read and classify it before another run.
- On an absent `nsys`, continue the trainer-clock decision, report calls and source tensor-op
  counts separately, and mark the kernel-launch count UNAVAILABLE.
- Never touch Claude's pods. If no own pod can be created under the caps, leave compute NOT_RUN
  and do not substitute the laptop CPU.

## Verdict

TEST FITNESS: FIT - M2 #2030
sheet: docs/harness/prep/float-kernel-fusion-fused-cost-2026-11-07.md
reviewer: codex
note: 820a1103b; extension pooling, noise, exactness and terminal outcomes specified; exact ignored tests require one pass each; paired 1/32-step smokes isolate 31 training steps; no-nsys fallback is labelled correctly; pipeline-first compute gate recorded.
date: 2026-11-07

A FIT verdict clears compute for the configuration in that sheet only (D22).

Every mechanism pre-registration gets a TEST FITNESS review on its milestone before compute.
