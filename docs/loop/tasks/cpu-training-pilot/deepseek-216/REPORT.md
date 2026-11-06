# REPORT: cpu-training-pilot, deepseek-216

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `10977505e`.

## Status

Complete as release bookkeeping.

## What did not work

No source work was needed. Deepseek-215 remains a **REJECT** for the lexical
linear reply score because it was artifact-stable but weaker than raw Jaccard.

## What changed

Recorded the protected delivery and removed the completed
`deepseek/prompt-reply-lexical` coordination claim.

## Measurements

- reviewed head:
  `1f16de326b596a5f65d3cbdc84a067e0e745fc98`;
- protected squash merge:
  `10977505e67b076baadff8d53c60dec02ff83d13`;
- fresh-main ranker source:
  `8588af2f0e71718e96ed73170d22e42f49a226289071a5e86f13504d93c97454`;
- fresh-main ranker binary source:
  `d38faa656fef2dcad681394fd705d1eb170464d9800b3daaaa9b0a38d477d0dc`;
- fresh-main CPU crate export:
  `b900a16aec7e08f3ec9b34c3490ecabfcfe224b77f807cb7e7c4340168ff78b8`;
- fresh-main compact evidence:
  `bd2c8c204de5bbdb3ec36d050edd7f561729cb4920341413d230652509e85b84`.

Fresh main passed:

- focused ranker tests: **5 passed**;
- CPU library: **399 passed, 4 expected ignores**;
- published evidence checksum verification.

## Checks run at the head

- fresh-main source and evidence hash checks;
- fresh-main focused and full CPU suites;
- release policy and claim checks.

## Skills used

No system-provided Codex skill applied.

## Still open

- a sequence-compatibility or geometric-continuation reply score;
- fresh response generation beyond stored replies;
- Claude's independent human-text and end-to-end serving track.

## Plain summary

The lexical-only ranker rejection is on `main`, its fresh-main tests and hashes
pass, and its coordination claim is released.
