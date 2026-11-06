# REPORT: cpu-training-pilot, deepseek-222

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `b594be94d`.

## Status

Ready as release bookkeeping; this PR removes the completed
`deepseek/nearest-rival-gain` claim.

## What did not work

No further source work was needed. Deepseek-221 remains a
**CONTROL_FAULT**: a 2,529/2,538-target cyclic shuffle selected the same eight
gain moves and artifact as true-target training, so the fixed control cannot
support a mechanism verdict.

## What changed

Records protected delivery of #3780 and removes the completed claim.

## Measurements

- reviewed head:
  `fc3b37dfd34408cd38fe4a014b7be035d76a205d`;
- protected merge:
  `b594be94d1554e03079ce5d3ef96844d548e9e18`;
- final driver source:
  `1df4ffbba613e274ae353ead9b2d72f77f53fd769b0293deb35ac2631e20dee1`;
- compact evidence:
  `991f6dc11cdf0ba883d6fab7e8bbe301d11dcd570863858b10a048813ab55f67`;
- local evidence manifest:
  `fabfea413d03a7bdf22963c54c53fa4b8b0f81632583f6c244039c53f0e9a686`.

Fresh-main source, driver tests, compact evidence and all 11 local evidence
entries verified.

## Checks run at the head

- fresh-main CPU library: 402 passed, 4 expected ignores;
- fresh-main driver verdict tests: 2 passed;
- fresh-main prep-check, source and evidence checks passed;

## Skills used

No system-provided Codex skill applied.

## Still open

- a matched target shuffle that changes the optimization path;
- a model-derived stop that removes reliance on retrieved length;
- Claude's independent end-to-end serving track.

## Plain summary

The nearest-rival control fault is on `main` and its implementation claim is
released by this bookkeeping PR.
