# REPORT: cpu-training-pilot, deepseek-168

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `f579da597`.

## Status

Complete bookkeeping release for the merged deepseek-167 learned-copy
experiment.

## What did not work

Nothing in this bookkeeping unit. The released experiment remains a bounded
**REJECT**: the learned candidate read tied longest-32 on one seed and gained
one body position on the other.

## What changed

- Removed the completed `deepseek/copy-learned-read` claim.
- Recorded PR #3712, merge `f579da597`, and fresh-main verification in the
  deepseek-167 report.
- Changed no model, source, serving, CUDA, float-training, termination or
  Claude-owned path.

## Measurements

- released experiment reviewed head:
  `9909feb356fad2a2157d1a66f327e1cb6d8f842f`;
- merge: `f579da5978a80684d47d47498d0817810f90710b`;
- fresh-main CPU library: **302 passed**, 4 expected ignores;
- fresh-main CPU binaries: **83 passed**;
- published evidence manifest and no-`.u16` boundary: pass.

## Checks run at the head

- Claims JSON parses and contains no learned-copy claim.
- Report, scope, policy and `git diff --check` pass.
- Fresh main contains the merged implementation, result and evidence.

## Size

This release changes four bookkeeping paths against base `f579da597`.

## Skills used

No system-provided Codex skill applies. Project delivery procedures were used.

## Still open

Test a different geometric-copy representation or objective, then combine
selected content with independent termination.

## Plain summary

The learned-copy negative result is merged and verified on main. This PR closes
the completed coordination claim.
