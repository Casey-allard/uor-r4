# REPORT: one-model-end-to-end, deepseek-4

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Base: `2f8f665c5`. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **No CUDA training comparison ran in this unit.** The CPU host cannot execute
  the tensor graph under D24/D29. The updated three-step CUDA control and the
  matched training comparison remain `NOT_RUN` here; host tests prove bridge
  selection and refusal behavior, not trained capability.
- **The earlier `--train-as-served` implementation was incomplete.** It rounded
  parameter tables but did not run the scored query at Q12, did not pass through
  the Q16 residual and dynamic i16 activation boundaries, and scored checkpoints
  through the ordinary float panel. Those four gaps are now closed in code.
- **Training-only reply cuts are incompatible with the served graph.** The cut
  restricts a training read to the prompt, but the artifact has no equivalent
  restriction. `--train-as-served 1` now refuses `reply_read_train` rather than
  silently training a different graph.

## What changed

- The geometric recurrence and MLP activation boundaries now call one gated
  dynamic-i16 quantizer. It is a no-op outside `with_served_surrogate_training`,
  so the plain path remains unchanged.
- `run_long` selects the full served-surrogate wrapper for
  `train_as_served`; the loss applies `q12_query` before the head, and all float
  master values are restored before the optimizer receives the gradients.
- Served checkpoints are scored while variables are rounded as exported through
  the D11 panel, served panel loss, Q12 nearest/writer reading and native writer.
- `--inspect`, `--decode-speed` and `--score-rows` now read
  `train_as_served` from the checkpoint's run config and use the same served
  forward, Q12 row/opening/decode paths when it is set.
- The native-writer decode branch now rounds the final hidden point to Q12
  before its writer call, matching the nearest branch and the training head.
- The rounded-master guard now restores variables on `Drop` as well as the
  explicit success path, so a failed evaluation or training forward cannot
  leave the optimizer's float masters silently rounded.
- Memory dropout remains an explicit training-only augmentation; scoring uses
  the full memory. This distinction is documented in the item and report.

## Measurements

- Host selector control: inside a served training step all five flags
  `ROUND_Q16`, `BRIDGE_Q16`, `BRIDGE_ACTIVATION_I16`, `BRIDGE_D11` and
  `SNAP_TRANSITION` are true; afterward all are false.
- Unsupported-read control: a reply training cut is refused with the served-mode
  error and writes no run directory.
- Guard control: dropping a rounded-master guard restores every variable to its
  exact float value.
- Full float crate library suite: 208 passed, 0 failed.

The CUDA-only autograd/training control and the served writer-step control now
exercise the same graph but were `NOT_RUN` on this host.

## Checks run at the head

- `cargo test -p uor-r4-float-train --release --lib` in
  `CARGO_TARGET_DIR=local/deepseek/served-quant-target`: 208 passed.
- Focused
  `the_training_forward_selects_and_restores_the_served_bridges`: passed.
- `cargo check -p uor-r4-float-train --lib`: passed.
- `git diff --check` and loop policy/report checks: run at the final head.
- Strict crate Clippy still reports the pre-existing findings in
  `fbs_export.rs`, `flock_read_memory.rs`, `flock_read_register.rs`,
  `in_space_long.rs` and `spiralcore_fbs_sheet.rs`; the localized new
  argument-count warning on `row_picks` is explicitly scoped.

## Skills used

No repository skill was invoked. The unit reuses the existing D11 bridge,
export rounding and panel-evaluation paths.

## Still open

- Run the CUDA three-step control and the matched four-arm comparison on a
  qualified GPU.
- Establish export-gap and throughput results for the fully wired served mode.
- Continue Claude's fresh-phrasing/export parity track after this training-mode
  repair lands.

## Plain summary

`--train-as-served` now means the actual served quantization graph, not merely
rounded parameter tables: D11 snaps, Q16 residuals, dynamic i16 activations and
Q12 query/writer evaluation are on during training and checkpoint selection.
The host suites pass. GPU execution and the matched training comparison remain
the next evidence step.
