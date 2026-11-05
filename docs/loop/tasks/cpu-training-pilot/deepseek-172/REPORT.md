# REPORT: cpu-training-pilot, deepseek-172

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `fd6e6d4c8`.

## Status

Complete bookkeeping release for the merged deepseek-171
continuation-sensitive lookahead experiment.

## What did not work

Nothing in this bookkeeping unit. The released experiment remains a valid
**REJECT**: continuation lookahead scored 242 and 262 of 4,899 body positions
against 511 for no-lookahead learned and longest-32.

## What changed

- Recorded reviewed head
  `0d1e8d7574d7623e83f1633419f6cab950068cb4`, PR #3718 and protected merge
  `fd6e6d4c8c969b13410bca351ffbfa85a1821096` in the deepseek-171 report.
- Recorded fresh-main CPU library and binary checks.
- Removed the completed `deepseek/copy-continuation` claim.
- Changed no model, source, serving, CUDA, float-training, termination or
  Claude-owned path.

## Measurements

- released experiment reviewed head:
  `0d1e8d7574d7623e83f1633419f6cab950068cb4`;
- protected merge:
  `fd6e6d4c8c969b13410bca351ffbfa85a1821096`;
- fresh-main CPU library: **308 passed**, 4 expected ignores;
- fresh-main CPU binaries: **83 passed**;
- fresh-main published evidence and no-`.u16` boundary: pass;
- PR checks and all five required policy checks: pass;
- protected queue dry run: `WOULD-QUEUE`; merge: `QUEUED`.

## Checks run at the head

- Fresh-main CPU tests at the protected merge.
- Published evidence manifest and corpus boundary.
- Claims JSON parsing and absence of the released claim.
- Report, scope, policy, links, catalog and `git diff --check`.

## Size

This release changes four documentation paths against base `fd6e6d4c8`.

## Skills used

No system-provided Codex skill applies. Project delivery procedures were used.

## Still open

Useful local chat remains open. Longest-32 remains the stronger body selector;
the next program work should target independent termination rather than retune
this closed lookahead score.

## Plain summary

The continuation-sensitive lookahead negative is merged and verified on main.
This PR records its exact delivery identity and releases the completed claim.
