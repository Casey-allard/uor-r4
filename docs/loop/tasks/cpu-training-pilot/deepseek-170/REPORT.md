# REPORT: cpu-training-pilot, deepseek-170

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `4ec22a118`.

## Status

Complete bookkeeping release for the merged deepseek-169 lookahead
control-fault experiment.

## What did not work

Nothing in this bookkeeping unit. The released experiment remains
**CONTROL_INVALID**: target-shuffled lookahead changed no decisions because the
score used distance only. The primary arm was also strongly negative.

## What changed

- Removed the completed `deepseek/copy-lookahead` claim.
- Recorded PR #3715, merge `4ec22a118`, and fresh-main verification in the
  deepseek-169 report.
- Changed no model, source, serving, CUDA, float-training, termination or
  Claude-owned path.

## Measurements

- released experiment reviewed head:
  `8b0089c57297777b14b964285804bf331af383cf`;
- merge: `4ec22a118a30365fa1c9f6ce6b2180cd3017f53c`;
- fresh-main CPU library: **305 passed**, 4 expected ignores;
- fresh-main CPU binaries: **83 passed**;
- published evidence manifest and no-`.u16` boundary: pass.

## Checks run at the head

- Claims JSON parses and contains no lookahead claim.
- Report, scope, policy and `git diff --check` pass.
- Fresh main contains the merged implementation, control-fault evidence and
  updated research records.

## Size

This release changes four bookkeeping paths against base `4ec22a118`.

## Skills used

No system-provided Codex skill applies. Project delivery procedures were used.

## Still open

Test a target-sensitive lookahead score, then combine selected content with
independent termination.

## Plain summary

The distance-only lookahead control fault is merged and verified on main. This
PR closes the completed coordination claim.
