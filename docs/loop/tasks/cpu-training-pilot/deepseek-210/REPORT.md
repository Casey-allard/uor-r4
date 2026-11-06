# REPORT: cpu-training-pilot, deepseek-210

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `b8e01c13b`.

## Status

Complete as release bookkeeping.

## What did not work

- No additional source or mechanism work was needed.

## What changed

Recorded deepseek-209 delivery and removed its completed coordination claim.

## Measurements

- experiment PR: #3767;
- reviewed branch head:
  `19028323c6a3a688831d88ace307628cc69bcc3f`;
- protected merge:
  `b8e01c13bb07c69c9484a9a850489ce3c245d631`;
- source and compact evidence hashes matched on fresh main;
- serving library: **474 passed, 1 expected ignore**;
- OpenAI loopback: **19 passed**;
- CPU suite: **394 passed, 4 expected ignores** plus binary tests;
- persisted seed-1 VSA report reproduced after timing/path removal.

## Checks run at the head

- fresh-main source and evidence hash checks;
- fresh-main serving, API and CPU tests;
- release PR policy checks before protected merge.

## Skills used

No system-provided Codex skill applied.

## Still open

- compressed or learned VSA signatures;
- fresh generation beyond learned TRAIN replies;
- Claude's independent scale and serving track.

## Plain summary

The binary VSA selector is on `main`, its fresh-main checks pass, and the claim
is released.
