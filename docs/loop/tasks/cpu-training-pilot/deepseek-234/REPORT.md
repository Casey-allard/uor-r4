# REPORT: cpu-training-pilot, deepseek-234

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `672fb6eea`.

## Status

Ready as claim-release bookkeeping.

## What did not work

Deepseek-233 remains **INSUFFICIENT_POWER**. Consumed dev panels 2783-2786
provided only 58 exact-endpoint groups and no group where a later exact
endpoint followed a wrong first boundary.

## What changed

Records delivery of #3792. The manual claim path did not write a
`docs/harness/claims.json` entry, so this release removes no committed claim
record. It removes the merged remote implementation branch only after verifying
the PR state.

## Measurements

- reviewed head:
  `c71437dd8817c9beda3cdb1c6a5b05c05c5e94f4`;
- protected merge:
  `672fb6eea7ce772f85f1dcad48ddca671945406a`;
- full CPU library at the reviewed head: **415 passed, 4 expected ignores**;
- evidence payload checks: **3**, all valid;
- full loop gates at the reviewed head: **11/11 green**.

Fresh-main focused tests and evidence checks passed after merge.

## Interpretation

The pinned calibration extension and fail-closed preflight are on `main`.
Exact-endpoint calibration is exhausted for the tested branch pool; the next
unit moves to reference-free completion scoring.

## Checks run at the head

- fresh-main joint-branch tests: **7 passed**;
- fresh-main driver test: **1 passed**;
- all three evidence payload checks passed;
- main became `672fb6eea`, the protected merge of #3792.

## Skills used

No system-provided Codex skill applied.

## Still open

- reference-free complete-branch scoring;
- complete-reply and fresh-phrasing capability;
- broader full-model CPU training;
- Claude's independent end-to-end local-chat track.

## Plain summary

The calibration insufficiency result is on `main`, and this release verifies
and closes its implementation branch.
