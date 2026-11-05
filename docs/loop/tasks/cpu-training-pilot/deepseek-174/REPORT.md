# REPORT: cpu-training-pilot, deepseek-174

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `cc1fb4d6b`.

## Status

Complete bookkeeping release for the merged deepseek-173 pairwise
copy-candidate ranker.

## What did not work

Nothing in this bookkeeping unit. The released experiment remains
**INSUFFICIENT_TRAINING_DATA**: 74 supervised groups against the registered
640-group floor, with no perceptron fit or reply evaluation.

## What changed

- Recorded reviewed head
  `528b169cd948cf821417e940ef9d9aaa83f5cbae`, PR #3720 and protected merge
  `cc1fb4d6b36dbfbe50a56a17311efc8906d90fd1`.
- Recorded fresh-main CPU library and binary verification.
- Removed the completed `deepseek/copy-ranker` claim.
- Changed no model, serving, CUDA, float-inference, termination or Claude-owned
  source.

## Measurements

- experiment reviewed head:
  `528b169cd948cf821417e940ef9d9aaa83f5cbae`;
- protected merge:
  `cc1fb4d6b36dbfbe50a56a17311efc8906d90fd1`;
- fresh-main CPU library: **313 passed**, 4 expected ignores;
- fresh-main CPU binaries: **83 passed**;
- fresh-main published evidence and no-`.u16` boundary: pass;
- protected queue: `WOULD-QUEUE`; merge: `QUEUED`.

## Checks run at the head

- Fresh-main CPU tests at the protected merge.
- Published evidence manifest and corpus boundary.
- Claims JSON parsing and absence of the released claim.
- Report, scope, policy, links, catalog and `git diff --check`.

## Size

This release changes four documentation paths against base `cc1fb4d6b`.

## Skills used

No system-provided Codex skill applies. Project delivery procedures were used.

## Still open

Useful local chat remains open. The next pairwise-ranker attempt needs
materially more candidate-labelled states, a new seed and a new registration.

## Plain summary

The pairwise copy-ranker implementation and its fit-stop evidence are merged
and verified on main. This PR records delivery and releases the completed
claim.
