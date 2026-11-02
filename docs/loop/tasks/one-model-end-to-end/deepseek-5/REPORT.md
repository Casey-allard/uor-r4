# REPORT: one-model-end-to-end, matched served-training comparison

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Base: `9c9b712a4`. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The four registered GPU runs did not run.** This host has no CUDA device,
  and `uor-pod status` reported no leaseable GPU. The comparison is
  pre-registered and executable but remains `NOT_RUN`; no training, throughput
  or quality result is inferred.
- **Checkpoint evaluation could not compare both modes like-for-like.** A plain
  checkpoint records `train_as_served: false`, so inspection would normally
  choose the plain path. The unit adds a scoring-only override and a documented
  rounded evaluation-copy procedure.
- **The initial sheet required an unsupported top-8 inspection field.** The
  inspection output does not report top-8, so the unsupported claim was removed
  before FIT.

## What changed

- `train-in-space --served-forward 0|1` overrides the saved training mode only
  for `--inspect`, `--decode-speed` and `--score-rows`.
- The override is refused outside those scoring modes and does not alter
  training.
- `fbs-snapshot --inspect` was updated for the current inspect signature.
- The prep sheet now pins TRAIN, DEV, mask and tokenizer identities, defines
  rounded evaluation copies for both modes, and registers a two-mode × two-seed
  engineering comparison with explicit instrument, operational and quality
  gates.
- Fresh FIT and pre-run review passed before any compute.

## Measurements

No model measurements were taken. The registered run is:

- two modes (`plain`, `served`) × two seeds (`1`, `2`);
- 256 steps per run;
- one RTX 5090, capped at one GPU-hour and `$1.50`;
- finite loss, served evaluator, export-rounding, decode throughput and smoke
  quality readings.

The pre-run reviewer confirmed that both modes can enter the same served
evaluator through rounded evaluation copies and that the comparison is an
engineering smoke, not capability evidence.

## Checks run at the head

- Fresh non-author `TEST FITNESS: FIT`, `PRE-RUN: PASS` on the pre-compute
  sheet.
- `cargo test -p uor-r4-float-train --release --bin train-in-space`:
  4 passed, including `served_forward_is_a_scoring_only_override`.
- Full `uor-r4-float-train --release --lib`: 208 passed, 0 failed, 0 ignored.
- `uor-harness prep-check`: the sheet is complete and FIT.
- Strict crate-local Clippy still reports the same pre-existing findings in
  `fbs_export.rs`, `flock_read_memory.rs`, `flock_read_register.rs`,
  `in_space_long.rs` and `spiralcore_fbs_sheet.rs`; no new finding class was
  introduced.
- Inserted source lines have no rustfmt diff. File-wide rustfmt still reports
  pre-existing formatting in unrelated lines.
- Diff, loop policy and exact-head review are recorded in the PR review record.

## Skills used

No repository skill was invoked.

## Still open

- Run the four registered arms on a leased RTX 5090.
- Record finite losses, artifact/checkpoint hashes, served-evaluator smoke
  metrics, export-rounding changes and decode tokens/s.
- Treat a failed instrument or operational door as a gate failure before any
  model-quality interpretation.

## Plain summary

This unit prepares a controlled comparison between ordinary float training and
training through the full served graph. It adds an explicit evaluation-only
switch so checkpoints from both modes can be scored through the same served
forward, and it fixes the pre-registration so GPU runs are reproducible and
bounded. The local code and tests pass, but the four GPU runs did not happen on
this machine.
