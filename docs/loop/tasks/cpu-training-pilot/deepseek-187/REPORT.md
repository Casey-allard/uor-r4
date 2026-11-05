# REPORT: cpu-training-pilot, deepseek-187

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `63ce79e85`.

## Status

Complete. This records the final deepseek-186 delivery and removes the last
table-anchor claim.

## What did not work

Nothing in this bookkeeping unit. The merged instrument and its scope are
unchanged.

## What changed

- Recorded PR #3740, reviewed head `4ee7e1231`, protected merge `63ce79e85`
  and final fresh-main checks in deepseek-186.
- Removed both completed `deepseek/table-anchor-isolation-release` and
  bookkeeping claims.
- Changed no model, serving, CUDA, float-training, exporter or Claude-owned
  source.

## Measurements

- reviewed delivery head:
  `4ee7e12314b1908c11acc1cbc506d54a7ab601bf`;
- protected merge:
  `63ce79e85ed5f97d2860f629bb7369036f8a0935`;
- final fresh-main CPU library: **351 passed**, 4 expected ignores;
- final fresh-main CPU binaries: all passed;
- final scoped strict Clippy, formatting, diff, report/FIT evidence: passed;
- all table-anchor claims: absent.

## Checks run at the head

- exact-head review before queue;
- required CI checks for PR #3740;
- protected `loop queue` merge with the reviewed head pinned;
- clean detached `origin/main` CPU and evidence verification;
- release report, FIT/prep and claim checks.

## Size

This unit changes only the deepseek-186 delivery record, this release record
and two completed claims.

## Skills used

No system-provided Codex skill applied. Repository claim, evidence and
protected-delivery procedures were used.

## Still open

The model representation still needs separate writer-address rows or an
explicit input-code preservation objective. This bookkeeping unit changes no
model capability.

## Plain summary

The table-anchor attribution result and its release are merged on main, verified
from a clean checkout, and no longer hold a branch claim.
