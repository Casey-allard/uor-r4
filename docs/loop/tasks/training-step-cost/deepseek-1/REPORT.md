# REPORT: training-step-cost, deepseek-1 (one loss read a step)

Lab: deepseek. Item: [training-step-cost](../../../../roadmap/items/training-step-cost.md).
Order: [ORDER.md](ORDER.md). Base: `f66f023d1`.

## What did not work, first

- **No device measurement ran, so this unit makes no speed claim.** The code removes three blocking
  reads from a four-micro-batch step by inspection; whether that shortens the step on an RTX 5090
  is still a measurement.
- **The proposed one-sort rival search does not preserve the trainer's tie rule by itself.**
  Candle 0.9.2 documents `arg_sort_last_dim` as unstable on ties. `best_positions` currently takes
  equal scores in lower-token-id order through repeated `argmax`. Replacing it with that sort needs
  an exact tie correction or a bounded top-eight kernel before it can go on the model path.
- **No free GPU exists to benchmark against today.** `uor-pod status` reports 18 verified RTX 5090
  offers, cheapest $0.563/h, and no running pod. A measured unit should lease its own card rather
  than touch another session's work.
- **Global `cargo fmt` is not a usable gate on this file.** The base intentionally carries large
  rustfmt drift in `in_space_long.rs`; formatting the file rewrote thousands of unrelated lines.
  That mechanical churn was removed, and this unit reports the formatter's baseline drift instead
  of presenting it as a pass.

## What changed

- **`step_gradient` keeps each micro-batch's detached scalar on the device.** It finishes every
  forward and backward, then `weighted_step_loss` stacks the detached scalars and makes one
  device-to-host read.
- **The arithmetic order is unchanged.** The host receives the same f32 values, converts each to
  f64, multiplies by the same forward's scored-row share, and sums in the same forward order.
- **The finite-loss refusal returns the same error, at a different failure point.** It now runs
  after every scheduled backward and the one read, rather than before each backward. A non-finite
  forward can therefore spend the remaining micro-batches' work before it is refused. The CPU test
  pins the deferred helper's error text; it does not exercise this failure path inside a full
  model step.
- **The gradient order is unchanged.** Each forward still contributes its scaled loss to a
  `GradStore`, and `add_grads` accumulates those stores in micro-batch order.
- **A CPU test pins exact weighted-sum bits and the non-finite refusal.** The existing CUDA test,
  `micro_batches_weighted_by_their_scored_rows_give_the_whole_batchs_step`, remains the
  split-versus-whole gradient gate for the later device unit.

Size: the code commit is 73 insertions and 19 deletions in `in_space_long.rs` plus the ORDER.

## Measurements

- By source inspection, a (b64L) step with `--batch 64 --micro-batch 16` changes from four
  `loss.to_scalar::<f32>()` calls, each a blocking copy, to one stacked copy after its fourth
  backward.
- No latency, loss-equivalence or gradient-equivalence reading was produced on a device in this
  unit. CPU parity is a correctness check, not a performance result.

## Checks run at the head

Code head: `93294106a`. The only later head at review is REPORT/ORDER text; this section separates
the code checks from the final documentation checks.

- `cargo test --release --locked -p uor-r4-float-train --lib`: 295 passed, 0 failed.
- The focused test
  `one_loss_read_keeps_the_steps_weighted_sum_and_refuses_non_finite`: passed.
- `cargo check --offline --locked -p uor-r4-float-train --lib`: passed.
- Stub-nvcc
  `cargo check --offline --locked -p uor-r4-float-train --features cuda --all-targets`:
  passed with `CUDARC_CUDA_VERSION=12080` and `UOR_NVCC_ARCH=compute_120`.
- `cargo clippy --offline --locked -p uor-r4-float-train --lib --no-deps`: completed with the
  crate's pre-existing warnings: two complex types, one range loop, one needless borrow, and two
  functions over the argument limit. None is in a changed line.
- `git diff --check HEAD^ HEAD`: passed.

Not run: a CUDA test, a training step on a GPU, a b64L A/B timing run, or any model scoring.

## Skills used

No system-provided Codex skill applied. The repository's charter, D84, geometry, loop and delivery
rules were used directly.

## Still open

1. Lease one RTX 5090 and run the existing split-versus-whole CUDA test at this head.
2. Time clean (b64L) steps with the old per-forward readbacks and with this one-read path, enough
   steps to separate the saving from run-to-run noise.
3. Measure the rival search before replacing it. Candle's stock sort is one launch but unstable on
   ties; a custom bounded top-eight search must preserve the lower-token-id rule and show a gain
   over the eight `argmax` passes.
4. If the readback saves measurable time, hand Claude the branch as an optional trainer
   acceleration after its exact-head review; do not move the active D3 recipe mid-run.

## Plain summary

The trainer used to pause four times a step to copy each micro-batch's loss to the CPU. It now
keeps those four small loss values on the GPU, finishes the gradients, and copies them together
once. A CPU test proves the returned loss and its error handling are unchanged, and the full CPU
suite passes. This is a safe code-level reduction, but it has not yet been timed on a GPU.
