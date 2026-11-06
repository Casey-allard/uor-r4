# REPORT: cpu-training-pilot, deepseek-232

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `349b833cf`.

## Status

Ready as claim-release bookkeeping.

## What did not work

Deepseek-231 remains **INSUFFICIENT_POWER**. TRAIN did not expose a supervised
generated-prefix false boundary, so the selector stopped before fresh seed 2787.

## What changed

Records delivery of #3790. The manual claim path did not write a
`docs/harness/claims.json` entry, so this release removes no committed claim
record. It removes the merged remote implementation branch only after verifying
the PR state.

## Measurements

- reviewed head:
  `574bb6bc98c6f8746465101edd6fbc14b30f5d1d`;
- protected merge:
  `349b833cf76764a9e9f23be40edfe49c5d31fc80`;
- full CPU library at the reviewed head: **414 passed, 4 expected ignores**;
- evidence files: **8**, all checksums valid;
- full loop gates at the reviewed head: **11/11 green**.

Fresh-main focused tests and evidence checks passed after merge.

## Interpretation

The complete-branch selector and its fail-closed preflight are on `main`.
TRAIN-only endpoint supervision is exhausted for this representation; the next
unit needs distribution-matched or reference-free supervision.

## Checks run at the head

- fresh-main joint-branch tests: **6 passed**;
- fresh-main driver test: **1 passed**;
- all seven payload checksums passed;
- main became `349b833cf`, the protected merge of #3790.

## Skills used

No system-provided Codex skill applied.

## Still open

- distribution-matched or reference-free endpoint supervision;
- complete CPU models beyond the bounded supported mechanisms;
- Claude's independent end-to-end local-chat track.

## Plain summary

The negative complete-branch result is on `main`, and this release verifies and
closes its implementation branch.
