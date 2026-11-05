# REPORT: cpu-training-pilot, deepseek-189

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `9577aebc9`.

## Status

Complete. This records and releases the merged deepseek-188 raw query-port
control failure.

## What did not work

Nothing in this bookkeeping unit. The experiment remains `CONTROL_FAULT`; no
model result is promoted.

## What changed

- Recorded PR #3742, reviewed head `eddaf1cb4`, protected merge `9577aebc9`
  and fresh-main checks in deepseek-188.
- Removed the completed `deepseek/raw-query-port` claim.
- Changed no model, serving, CUDA, float-training, exporter or Claude-owned
  source.

## Measurements

- reviewed delivery head:
  `eddaf1cb479126aede8579223c83b3b8ff1a617d`;
- protected merge:
  `9577aebc9b7dbb0964906efffd13662b7d4984c8`;
- fresh-main CPU library: **351 passed**, 4 expected ignores;
- fresh-main CPU binaries: all passed;
- fresh-main scoped strict Clippy, formatting, diff, report/FIT evidence:
  passed.

## Checks run at the head

- exact-head review before queue;
- required CI checks for PR #3742;
- protected `loop queue` merge with the reviewed head pinned;
- clean detached `origin/main` CPU and evidence verification;
- release report, FIT/prep and claim checks.

## Size

This unit changes only the deepseek-188 delivery record, this release record
and the completed claim.

## Skills used

No system-provided Codex skill applied. Repository claim, evidence and
protected-delivery procedures were used.

## Still open

The query-port objective needs a behaviorally distinct control before it can
produce a valid mechanism comparison. No model capability changes here.

## Plain summary

The input-preserving query-port experiment is merged and verified on main,
correctly classified as a control failure. This PR records delivery and
releases the completed claim.
