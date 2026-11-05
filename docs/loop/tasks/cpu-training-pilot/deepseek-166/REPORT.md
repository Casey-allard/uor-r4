# REPORT: cpu-training-pilot, deepseek-166

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `38ab13578`.

## Status

Complete bookkeeping release for the merged deepseek-165 geometric-copy
experiment.

## What did not work

Nothing in this bookkeeping unit. The released experiment remains a bounded
**REJECT**: raw token-row sign distance was worse than depth-first exact copy.

## What changed

- Removed the completed `deepseek/copy-geometric` claim.
- Recorded PR #3710, merge `38ab13578`, and fresh-main verification in the
  deepseek-165 report.
- Changed no model, source, serving, CUDA, float-training, termination or
  Claude-owned path.

## Measurements

- released experiment reviewed head:
  `50f22f86962078d8b2a310f07f3c6ee23ff074fc`;
- merge: `38ab1357832be444bba248b729e4aa7b46dac7a0`;
- fresh-main CPU library: **295 passed**, 4 expected ignores;
- fresh-main CPU binaries: **82 passed**;
- published evidence manifest and no-`.u16` boundary: pass.

## Checks run at the head

- Claims JSON parses and contains no completed copy-geometric claim.
- Report, scope, policy and `git diff --check` pass.
- Fresh main contains the merged implementation, result and evidence.

## Size

This release changes four bookkeeping paths against base `38ab13578`.

## Skills used

No system-provided Codex skill applies. Project delivery procedures were used.

## Still open

Test a contextual or learned geometric-copy ranking, then combine selected
content with independent termination.

## Plain summary

The rejected geometric-copy result is merged and verified on main. This PR
closes the completed coordination claim.
