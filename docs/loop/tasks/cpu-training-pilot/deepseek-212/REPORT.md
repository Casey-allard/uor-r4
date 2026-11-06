# REPORT: cpu-training-pilot, deepseek-212

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `2253ede36`.

## Status

Complete as release bookkeeping.

## What did not work

No additional source or mechanism work was needed. The deepseek-211 result
remains a **REJECT** for the selected 1,024-bit modulo compression.

## What changed

Recorded the protected delivery and removed the completed
`deepseek/prompt-vsa-compression` coordination claim.

## Measurements

- reviewed head:
  `eb97c5ae4ca88f7657073bec240679787dd56fc2`;
- protected squash merge:
  `2253ede36c22afb09d89eb6e5f27c950145939b4`;
- fresh-main prompt-reply source:
  `bb472b9bc27ce22b607be7073e5ba31c33d4dab489a2a44eeb3374e9b302b7a3`;
- fresh-main stack export source:
  `d40d3683ce4ab681aaefe3ee990203071bb541da33c37faf6eb7ba3bdb59c28c`;
- fresh-main serving check:
  `b454260ad0789902d9c76698c30a3b92be20cc4b2bd7d7126a8ba450353be6c2`;
- fresh-main compact evidence:
  `c84c41f5ef89ffe32848066d6d3498aa69153802af9a18efa7b06db88293fcfb`.

Fresh main passed:

- prompt-reply focused tests: **8 passed**;
- calibration tests: **2 passed**;
- serving library: **477 passed, 1 expected ignore**;
- OpenAI loopback: **17 passed**;
- CPU library: **394 passed, 4 expected ignores**;
- published evidence checksum verification.

## Checks run at the head

- fresh-main source and evidence hash checks;
- fresh-main focused, serving, API and CPU suites;
- release policy and claim checks.

## Skills used

No system-provided Codex skill applied.

## Still open

- learned or geometry-aware compressed signatures that preserve retrieval
  ordering;
- fresh response generation beyond learned TRAIN replies;
- Claude's independent scale, export and serving track.

## Plain summary

The compressed prompt-VSA rejection is on `main`, its fresh-main tests and
hashes pass, and its coordination claim is released.
