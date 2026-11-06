# REPORT: cpu-training-pilot, deepseek-214

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `e40abe7b9`.

## Status

Complete as release bookkeeping.

## What did not work

No additional source or mechanism work was needed. Deepseek-213 remains a
**REJECT** for the fixed whole-reply ranker because its gain did not replicate
across artifact seeds.

## What changed

Recorded the protected delivery and removed the completed
`deepseek/prompt-reply-ranker` coordination claim.

## Measurements

- reviewed head:
  `3b6e2a0bb5e38adaa05ab4dd8cb615e3fdf83bac`;
- protected squash merge:
  `e40abe7b94b060116a52a3c10fbede3ea845214f`;
- fresh-main ranker source:
  `714ecd92f79d7c4aa145f3e00211df793160d1a15a502b9fdbb8187ea9e37409`;
- fresh-main ranker binary source:
  `70a05489a4030cb982172d86328efa493e7515a224cdb3b12c72dbf48d61c115`;
- fresh-main CPU crate export:
  `0ac072c3aed799fc62f795c02dd35ce23077f986cc2f5a74cf3191e5c17446cf`;
- fresh-main compact evidence:
  `f1e54ff69b9578fde3c11634492c4887ccc72059041f16e7d7acb15069795d94`.

Fresh main passed:

- focused ranker tests: **4 passed**;
- CPU library: **398 passed, 4 expected ignores**;
- published evidence checksum verification.

## Checks run at the head

- fresh-main source and evidence hash checks;
- fresh-main focused and full CPU suites;
- release policy and claim checks.

## Skills used

No system-provided Codex skill applied.

## Still open

- artifact-stable ranking over correct top-16 candidates;
- fresh response generation beyond stored replies;
- Claude's independent human-text and end-to-end serving track.

## Plain summary

The artifact-unstable whole-reply ranker is on `main`, its fresh-main tests and
hashes pass, and its coordination claim is released.
