# REPORT: cpu-training-pilot, deepseek-236

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `6e5fcefe6`.

## Status

Ready as claim-release bookkeeping.

## What did not work

Deepseek-235 remains **INSUFFICIENT_POWER**. Fresh seed-2788 artifact 1 changed
21 closure selections and had 18 exact-candidate rows against power floors of
100 and 20, so seed 2 and repeats did not run.

## What changed

Records delivery of #3794. The manual claim path did not write a
`docs/harness/claims.json` entry, so this release removes no committed claim
record. It removes the merged remote implementation branch only after verifying
the PR state.

## Measurements

- reviewed head:
  `afa94adf6108a85736b346150164ef6f188f9579`;
- protected merge:
  `6e5fcefe6c37ca92d4cdd57264689766f860dea5`;
- full CPU library at the reviewed head: **417 passed, 4 expected ignores**;
- evidence payload checks: **8**, all valid;
- full loop gates at the reviewed head: **11/11 green**.

Fresh-main focused tests and evidence checks passed after merge.

## Interpretation

Reference-free closure and the matched tuple-shuffle path are on `main`. The
fresh panel itself showed candidate-spine coverage as the next blocker; this
release does not turn the power stop into a selector negative.

## Checks run at the head

- fresh-main joint-branch tests: **9 passed**;
- fresh-main driver test: **1 passed**;
- all eight evidence payload checks passed;
- main became `6e5fcefe6`, the protected merge of #3794.

## Skills used

No system-provided Codex skill applied.

## Still open

- denser candidate coverage for complete-reply generation;
- complete-reply and fresh-phrasing capability;
- broader full-model CPU training;
- Claude's independent end-to-end local-chat track.

## Plain summary

The reference-free closure power result is on `main`, and this release verifies
and closes its implementation branch.
