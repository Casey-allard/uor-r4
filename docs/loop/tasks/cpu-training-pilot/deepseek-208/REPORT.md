# REPORT: cpu-training-pilot, deepseek-208

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `a8d9a2e16`.

## Status

Complete as release bookkeeping.

## What did not work

- No mechanism work was required.

## What changed

Recorded deepseek-207 delivery and removed its completed coordination claim.

## Measurements

- experiment PR: #3765;
- reviewed branch head:
  `f9a9d0a05c6f7a7a4bc6af925d9e7c93134b7abf`;
- protected merge:
  `a8d9a2e16dc551f09a74c233e6fe640753db97f9`;
- fresh-main source and evidence identities matched;
- fresh-main serving library: **473 passed, 1 expected ignore**;
- fresh-main OpenAI loopback: **19 passed**;
- fresh-main CPU suite: **394 passed, 4 expected ignores** plus binary tests;
- persisted seed-1 serving report reproduced after timing/path removal.

## Checks run at the head

- source and compact-evidence hashes verified on fresh main;
- fresh-main serving, API and CPU tests passed;
- release PR policy checks required before merge.

## Skills used

No system-provided Codex skill applied.

## Still open

- VSA/vector prompt indexing with the same serving parity;
- generation beyond learned TRAIN replies;
- Claude's independent scale and serving track.

## Plain summary

The prompt-reply serving result is on `main`, fresh-main checks pass, and its
coordination claim is removed.
