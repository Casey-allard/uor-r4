# ORDER: wire served quantization into training, deepseek-4

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `2f8f665c5`.

## Goal

Connect the served-surrogate primitives merged in deepseek-3 to the real
geometric hidden path and `train-in-space --train-as-served 1`. In this mode
the training forward must use the D11 snaps, Q12 scored query, Q16 residual
grid and dynamic per-row i16 activations, while the optimizer continues to move
the float master values. Checkpoint scoring and saving must use the same served
forward on rounded variables.

## Cancel probe

Check whether `train_as_served` already constructs `with_served_surrogate_training`
around the training forward, applies `activation_quantize16` at the served
recurrence and MLP boundaries, rounds the scored query to Q12, and evaluates
checkpoints through the D11 path. If every item is already wired and tested,
stop.

## Scope

- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-4/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

The roadmap item must add `hamilton_sheet.rs` and
`geometric_stack_native.rs` to its own scope: both are now on this item's
served-training path. This is a file-scope correction, not a bar, status or
stop-rule change.

## Decisions

- Reject `reply_read_train` with `--train-as-served 1`: the cut limits a
  training read to the prompt, but no equivalent cut is exported or served.
- Keep memory dropout as an explicit training-only augmentation. Scoring uses
  the full memory, and the training report must preserve that distinction.
- Do not add float serving, dense matmul, softmax, squashing or a transformer
  fallback.

## Done when

- The geometric recurrence and MLP activation boundaries call the dynamic i16
  quantizer only inside the served-surrogate wrapper; plain training and
  evaluation remain bit-for-bit on their previous path.
- `run_long` uses `with_served_surrogate_training` for a `train_as_served`
  step, rounds its scored query to Q12, and restores all float masters before
  the optimizer update.
- A served checkpoint is scored through the D11 panel and writer paths while
  variables are rounded as exported, and its saved bytes remain unchanged by
  re-rounding for export.
- Host controls prove the training-mode selector enables and restores the Q16,
  activation, D11 and Q12 bridges, and that unsupported training-only read
  cuts are refused. Any tensor/autograd check that D24/D29 permits only on CUDA
  is marked `NOT_RUN` on this CPU-only host, never passed by inference.
- Focused float-model tests, the full float crate library suite, formatting,
  diff and policy checks pass at the head.

## Stop and ask if

- The change requires a serving-format change or a new exported mechanism.
- The only way to test the graph would violate D24/D29 or require a silent host
  compute fallback.
