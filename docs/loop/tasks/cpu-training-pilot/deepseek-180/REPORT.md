# REPORT: cpu-training-pilot, deepseek-180

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `02d0a814b`.

## Status

Complete protected release for merged PR #3728. The K12/seed-2769 selection
fault and K16/seed-2770 **REJECT** remain unchanged.

## What did not work

Nothing in this bookkeeping unit. The K16 anchored linear ranker scored
**13 / 4** body positions against longest-32 **603 / 603** and is closed.

## What changed

- Recorded reviewed head
  `627c057279fa876faaf39153d76e11166298970a`, PR #3728 and protected merge
  `02d0a814bc18b0dc5fdbd8531db1002d71c0992f`.
- Recorded CI and fresh-main verification in the deepseek-179 report.
- Removed the completed experiment and temporary release claims.
- Changed no model, serving, CUDA, float-inference, termination or Claude-owned
  source.

## Measurements

- reviewed delivery head:
  `627c057279fa876faaf39153d76e11166298970a`;
- protected merge:
  `02d0a814bc18b0dc5fdbd8531db1002d71c0992f`;
- fresh-main CPU library: **335 passed**, 4 expected ignores;
- fresh-main driver tests: **3 passed**;
- fresh-main strict Clippy: passed;
- fresh-main source/calibration/evidence/resource checks: passed;
- all required PR checks at the approved head: passed;
- protected queue: `WOULD-QUEUE`; merge: `QUEUED`.

## Checks run at the head

- Exact-head independent review: **APPROVE**.
- All required PR checks at the approved head.
- Fresh-main CPU library, driver tests, strict Clippy and evidence seals.
- Claims JSON parsing and absence of both completed claims.
- Report, scope, evidence and `git diff --check`.
- `loop gates --full`, docs catalog and `prep-check`.

## Size

This release changes four documentation paths against base `02d0a814b`.

## Skills used

No system-provided Codex skill applies. Project delivery procedures were used.

## Still open

Longest-32 remains the current body selector. The next copy/body mechanism
needs a different representation or objective; seeds 2769 and 2770 are closed.
Independent termination and completion remain the larger chat-model blockers.

## Plain summary

The K12 fault and K16 negative are merged and verified on main. This record
releases the completed claims.
