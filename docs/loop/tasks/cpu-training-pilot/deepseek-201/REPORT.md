# REPORT: cpu-training-pilot, deepseek-201

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `9f40967f1`.

## Status

Complete. This records and releases the merged deepseek-200 result. The
experiment remains **CONTROL_FAULT**: full-history and reply-only boundaries
made identical observable stop decisions on both fresh artifacts.

## What did not work

Nothing failed in this bookkeeping unit. The released experiment's registered
per-artifact non-vacuity gate found zero full-history-versus-reply stop
changes and correctly stopped before repeats.

## What changed

- Recorded PR #3757, reviewed head `473361e70`, protected merge `9f40967f1`
  and fresh-main verification in deepseek-200.
- Removed the completed `deepseek/full-sequence-boundary` and
  `deepseek/full-sequence-boundary-release` claims.
- Changed no model, serving, CUDA, float-training, exporter or Claude-owned
  source.

## Measurements

- exact-head review: counted `uor-review v1` APPROVE;
- reviewed head:
  `473361e706ad719c44001d6a4cbaa1267c02bfa2`;
- protected merge:
  `9f40967f11c09863de5cd279fe863f93d705e731`;
- all required PR checks: pass;
- fresh-main content identity: all 14 changed files;
- fresh-main CPU package: **380 passed**, 4 expected ignores;
- source hashes:
  `1751a85ae21f5f6ba74fd4918aad714726fe570631296720a5cb0cff69e7be1a`
  and
  `74271a97fb09ad17e2c0fd07bc1b66515399d7578bf2e071b0741377b3bf4a0f`;
- compact evidence manifest: pass;
- pre-registration: FIT, GO and complete.

## Checks run at the head

- protected `loop queue` at the reviewed head;
- merge-queue CI;
- fresh-main CPU tests, source hashes and compact evidence verification;
- release report and claim checks before delivery.

## Skills used

No system-provided Codex skill applied. Repository claim, evidence and
protected-delivery procedures were used.

## Still open

- termination that changes stop position rather than terminal-table
  provenance;
- binding stopping to the selected content source or generated-state
  continuation;
- fresh-phrasing generalization beyond exact-memory recall;
- Claude's independent scale and serving track.

## Plain summary

The full-history boundary result is merged and verified on fresh main. It is
preserved as a control fault, while the earlier positive content mechanism and
the need for a genuinely different termination objective remain on the active
research track.
