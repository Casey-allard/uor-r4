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

## Post-run result, 3 November 2026

Claude ran all four registered jobs on seed 1's pod at head
`36ca53c81195969f9487747163cc13c50d145efa`, alone on the GPU, and sealed them
at `results/claude/one-model-end-to-end-copy-key-2026-11-02/s1/s1.tar`
(SHA-256 `738457b26af18e9176bd0e3ece76f0502d4505cdf0a3f249381fc0ccf574ad1d`).
This host has no Hugging Face credential, so the decision uses Claude's sealed
run summary on #3407 rather than a second local download.

**Instrument door: PASS.** All four runs were finite and loaded through the
served evaluator. Both served checkpoints were unchanged by export rounding;
four of five files changed for each plain checkpoint.

**Operational door: FAIL.** Served training took 355 seconds against 51
seconds plain at seed 1 (6.96×) and 355 against 52 at seed 2 (6.83×), above the
registered 2× ceiling.

| Seed | Plain top-1 | Served top-1 | Served − plain | Nearest hits, plain → served | Openings, plain → served |
|---:|---:|---:|---:|---:|---:|
| 1 | 36 | 11 | −25 | 225 → 23 | 22 → 1 |
| 2 | 61 | 6 | −55 | 229 → 20 | 15 → 1 |

**Quality: INCONCLUSIVE.** The two-seed spread is 30 rows, so the pre-registered
band is max(24, 30) = 30. Seed 2 is materially below, but seed 1's 25-row gap is
inside the band; the sheet requires both seeds.

Decode through the served evaluator was 49.2–49.4, 40.8–41.1 and 28.6–28.8
tok/s at contexts 16, 64 and 192. The observed served-training slowdown is not
yet attributed: the 49-pass activation-scale loop is a hypothesis, not a
measured cause.

**Decision:** the wiring is not ready for a longer run. The next unit should
cost-reduce the quantizer and rerun this comparison; quality is not reclassified
from the smoke.
