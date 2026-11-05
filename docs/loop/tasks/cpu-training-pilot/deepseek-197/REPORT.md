# REPORT: cpu-training-pilot, deepseek-197

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `03c929896`.

## Status

Complete. This records and releases the merged deepseek-196 preflight stop. The
experiment remains **STOPPED_AT_PREFLIGHT_INSUFFICIENT_FALLBACK_COVERAGE**; no
mechanism or model result is promoted.

## What did not work

Nothing failed in this bookkeeping unit. The released experiment itself could
not activate its treatment: local exact copy supplied a candidate at every
reachable generated decision, so the confident global fallback made zero
decisions against the registered minimum of 100.

## What changed

- Recorded PR #3752, reviewed head `ae6a6916e`, protected merge `03c929896`
  and fresh-main verification in deepseek-196.
- Removed the completed `deepseek/suffix-fallback` source claim.
- Changed no model, serving, CUDA, float-training, exporter or Claude-owned
  source.

## Measurements

- exact-head review: counted `uor-review v1` APPROVE;
- reviewed head:
  `ae6a6916e4af27eec578427f0606a21480a4af00`;
- protected merge:
  `03c929896efe795a62745281b84ebafa7fba0c77`;
- all required PR checks: pass;
- fresh-main content identity: all 15 changed files;
- fresh-main CPU package: **371 passed**, 4 expected ignores;
- source hashes:
  `3dfd23c11f74af472a94868e2c9cdc21ff37a914b93e7a17bdc87b4a17c4f926`,
  `a5d8a13a15beae8112b4575e31985a501a7949e34ace71fbe95c20d08f3de7c2`,
  and
  `00c9b41cb1fa235ccc2d21e79c4a0a3d78d86b2c08ee3aeba51243caf6776df2`;
- compact evidence manifest: pass;
- pre-registration: FIT and complete.

## Checks run at the head

- protected `loop queue` at the reviewed head;
- merge-queue CI;
- `loop close --verify`: state, squash, content, all 11 gates, worktree and
  branch cleanup;
- fresh-main CPU tests and source/evidence identity;
- release report and claim checks before delivery.

## Skills used

No system-provided Codex skill applied. Repository claim, evidence and
protected-delivery procedures were used.

## Still open

- a depth-aware continuation arbiter that can compete with shallow local copy;
- complete-reply/fresh-phrasing capability on the one-model path;
- Claude's independent scale and serving track.

## Plain summary

The fallback preflight stop is merged and verified on fresh main. This unit
releases its source claim.
