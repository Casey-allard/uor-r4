# ORDER: float-kernel-fusion, deepseek-1 (fused quantizer cost and default)

Lab: deepseek. Item: [float-kernel-fusion](../../../../roadmap/items/float-kernel-fusion.md)
(#3678). Base: `f66f023d1`.

## Goal

Complete the training half of the item's done-when 4 and the training default in done-when 5 on
the current one-model line:

> 4. **The cost,** on an RTX 5090 at P2b's recipe: kernel launches and seconds per training
> step, chain against fused, by the trainer clock (and `nsys`, if the pod has it).
> 5. The flag defaults on only when 2 and 3 hold. One notebook row and a REPORT.

Claude-285's done-when 2 and 3 already hold on an RTX 5090. This unit prepares a measurement of
the fused quantizers on the current b64L human-text recipe at both short and long context,
checks every saved 256-step training artifact, and turns fusion on for served `train-in-space`
training only if both shapes improve. The item's cached-decode half of done-when 4 remains open:
the current flag is training-only, and this ORDER does not claim the item's complete done-when 4.

The latest owner direction is pipeline first ([claude-307's
REPORT](../../one-model-end-to-end/claude-307/REPORT.md)): no new training run starts until the
current model's export, integer serving and parity path works. This ORDER may prepare, test and
review the reference implementation now, but it must not launch the GPU experiment while that
direction stands. The experiment runs only after the project's current one-model pipeline clears
that direction, or after a top-session decision review explicitly classifies this microbenchmark
as an exception.

## Claim check

- **The conditional project need:** once pipeline-first is cleared, the next training repeat at
  0.633-0.698 s a step pays the fused quantizers' cost throughout. A 32,768-step repeat is
  projected near 14.6 h for two seeds and roughly $8.50 at the recorded rates. The result is
  useful only when such repeats resume.
- **What changes in the model:** no value or gradient. `activation_quantize16` and
  `q16_residual` already execute their exact fused replacements under `--fused-quantize 1`;
  claude-285 proved the losses, gradients, samples and eight-step checkpoints bit-identical.
  This unit extends the equality window across the current b64L recipe and both timing shapes,
  then changes the training default only if the measured clock says the fused path is faster.
- **Claude's work is isolated:** this branch does not touch `claude/d3-tokens`, Claude's pods or
  files. The source change, if earned, is one trainer default plus its docs and tests on
  `deepseek/float-kernel-fusion`.

## Cancel probe (run first)

1. `--fused-quantize` is present at the base, the fused PTX is built from its own module, and
   claude-285's GPU equality gates passed at `a1588bcc4`.
2. The current b64L command reaches the flag under `--train-as-served 1`, and the option's
   explicit `0` remains available for the chain arm.
3. This branch owns `deepseek/float-kernel-fusion` in `docs/harness/claims.json`, and no other
   claim names the files this unit changes.

If any check is false, stop and re-scope before compute.

## Files in scope

- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/fused_quantize.rs` (test-only if needed)
- `docs/harness/prep/float-kernel-fusion-fused-cost-2026-11-07.md`
- `docs/loop/tasks/float-kernel-fusion/deepseek-1/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/float-kernel-fusion.md`

No kernel source, model objective, data, checkpoint or serving artifact changes.

## Done when

1. The laptop checks pass: the float-train library tests, the focused flag-default test,
   stub-CUDA `cargo check`, `cargo clippy` for the crate, `git diff --check`, the loop report
   and local PR checks.
2. On one leased RTX 5090, four 256-step runs execute at (S), 192 x 16 with batch 64 in four
   micro-batches of 16, and four at (L), 2,048 x 4, each in the registered `OFF, ON, ON, OFF`
   order with equal seeds, data and eval cadence.
3. The step clock is read from the three post-warm-up 64-step windows in every run. Per shape,
   the ON/OFF gain is reported against the within-run window spread, the OFF pair spread, the ON
   pair spread, and the registered noise floors of 0.46% at (S) and 0.25% at (L).
4. Every `models/step-*` directory at steps 0, 64, 128, 192 and 256 is byte-identical across the
   four runs of a shape, and full `progress.jsonl` records are equal after removing only the two
   timing fields. Any difference fails the correctness gate before a speed result is read.
5. At each shape and mode, one 1-step and one 32-step smoke run with the same two mandatory
   checkpoint evaluations. Their difference gives 31 training steps without evaluation. The
   count difference and, where present, the `nsys` launch difference provide the observed launch
   counts and call counts per training step. If `nsys` is absent, report the source tensor-op
   counts as implementation counts and mark total kernel launches UNAVAILABLE.
6. If both shapes have `gain > 2 x noise`, `train-in-space` defaults served training on the
   qualified CUDA path to fused quantizers; an explicit `0` still selects the chain. If either
   shape has a non-positive gain, or a gain no larger than its noise, the default stays `0`. If
   either shape remains between noise and `2 x noise`, one complete additional
   `OFF, ON, ON, OFF` block named `A2, B2, B3, A3` is run for that shape. Its four checkpoint
   trees and normalized progress must also pass R4. R2 and R3 are then recomputed over all
   twelve windows a mode and all four `OFF`/`ON` pair comparisons. Both shapes must have
   `gain > 2 x noise` after that single extension to turn the default on; otherwise it stays
   `0` and the result is recorded as inconclusive.
7. The RTX 5090 run is sealed and read back, the pod is down, and one notebook row and a
   negatives-first REPORT record the result. The item remains open for its cached-decode cost.

## Stop and decide

Ask means decide, record and continue.

- **Checkpoint or loss mismatch:** stop the speed reading. Classify instrument or kernel, keep
  the default off unless the exact cause is repaired and the run repeats.
- **Fewer than three usable 64-step windows:** add one complete balanced block on the same pod.
- **Cost or pod cap:** one RTX 5090, planned for $3: a 50-minute first run, a possible
  24-minute extension, and one 50-minute retry projection at no more than $1.15/h. A healthy run
  is not cancelled for a self-set budget; extend, renew and record instead, within the owner's
  four-pod and $8/h caps.
- **Project priority:** if the current `PLAN.md`, STATE or newest notebook row still says the
  one-model pipeline comes before further training, do not launch. Finish the documentation and
  review, record the run as NOT_RUN for that project state, and leave this ORDER ready for the
  next pre-run review after the direction clears.
- **Other lab's pod:** never lease, reuse or inspect its GPUs. Create our own within the shared
  caps or stop compute and finish the code review.
