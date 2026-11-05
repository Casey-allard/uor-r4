# REPORT: cpu-training-pilot, deepseek-192

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `2f3c65855`.

## Status

Complete. This records and releases the merged deepseek-191 reply-boundary
delivery. The experiment remains `INSUFFICIENT_POWER`; no mechanism or model
result is promoted.

## What did not work

Nothing failed in this bookkeeping unit. The released experiment still has
only seven oracle-complete fresh replies, below its registered minimum of ten,
so it cannot assess the boundary mechanism.

The experiment used `loop claim`, not `team claim`, so it left no source branch
claim to remove. This unit removes its own release bookkeeping claim.

## What changed

- Recorded PR #3746, reviewed head `8c4ccca27`, protected merge `2f3c65855`
  and fresh-main checks in deepseek-191.
- Removed the completed `deepseek/copy-boundary-release` claim.
- Changed no model, serving, CUDA, float-training, exporter or Claude-owned
  source.

## Measurements

- reviewed delivery head:
  `8c4ccca27d659c2804eed623d7f22e793b39eefd`;
- protected merge:
  `2f3c6585568958fd3032c64acaddfc463ff8d6d0`;
- merge-group CI: success;
- fresh-main source identity: all 17 changed files identical;
- fresh-main CPU library: **359 passed**, 4 expected ignores;
- fresh-main CPU binaries: all passed;
- fresh-main scoped strict Clippy and package formatting: passed;
- fresh-main loop gates: **11 / 11 green**;
- fresh-main unscoped `cargo fmt --check`: red on pre-existing files outside
  this unit; no unrelated file was changed.

## Checks run at the head

- exact-head review before queue;
- required CI checks for PR #3746;
- protected `loop queue` merge with the reviewed head pinned;
- merge-group CI on the squash commit;
- `loop close --verify`: state, squash, content, gates and branch cleanup;
- fresh-main closed-book CPU verification;
- release report, scope, claims and loop checks after committing this record.

## Size

This unit changes only the deepseek-191 delivery record, this release record
and the completed bookkeeping claim.

## Skills used

No system-provided Codex skill applied. Repository claim, evidence and
protected-delivery procedures were used.

## Still open

- A fresh panel with at least ten oracle-complete longest-copy replies is
  required before exact reply-boundary memory can be assessed.
- Body reconstruction remains the limiting capability: stopping was exercised,
  but no fresh complete reply matched exactly.

## Plain summary

The reply-boundary experiment is merged and verified on fresh main, correctly
classified as insufficient power. This PR records that delivery and releases
the completed bookkeeping claim.
