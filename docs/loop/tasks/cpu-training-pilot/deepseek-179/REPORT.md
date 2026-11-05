# REPORT: cpu-training-pilot, deepseek-179

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `d416181c1`; implementation pin:
`57ef4cdc1`.

## Status

Complete as **REJECT** for the corrected K16 anchored correction ranker, with
the K12/seed-2769 attempt preserved separately as an instrument/selection
fault.

## What did not work

The first registration selected the wrong row from its own train-only sweep.
At 12 cells artifact seed 2 had **2,043** target-present groups, five below the
2,048 floor. The driver stopped before evaluation. Its raw
`INSUFFICIENT_TRAINING_DATA` outcome and the corrected classification
`INSTRUMENT_FAULT / K12_SELECTION_ERROR` are both preserved. Seed 2769 is
closed.

The corrected K16 run cleared every training floor but failed every substantive
comparison. The anchored ranker overrode longest-32 on all **27,264** decisions
on each artifact and scored only **13 / 4** correct body positions. Longest-32
scored **603 / 603**.

The full seed-1/seed-2 readings were:

- anchored trained: **13 / 4**;
- anchored shuffled: **2 / 8**;
- anchored geometry-only: **202 / 68**;
- unanchored trained: **218 / 603**;
- writer: **47 / 48**.

The trained arm passed only the seed-1 shuffled comparison. It passed no
required control comparison on seed 2. Every arm had zero premature BOS and no
exact reference. All control-vs-longest checks were non-vacuous.

## What changed

- Added a CPU-only candidate order over a configurable number of geometric
  writer cells without changing the served eight-cell writer.
- Added twelve bounded absolute and longest-relative candidate features.
- Added a longest-32-anchored correction perceptron that updates only
  baseline-wrong groups and chooses an alternative only on a strictly positive
  score margin.
- Added anchored shuffled-target, anchored geometry-only and unanchored trained
  controls over the identical union.
- Added direct BOS-preservation behavior, depth-32/deepest-source/fallback
  tests and required control-vs-longest non-vacuity.
- Added a train-only cell sweep and committed normalized calibration evidence.

No serving, CUDA, float-inference, termination or Claude-owned source changed.

## Measurements

- FIT head for the corrected run:
  `ef9413826d90d2dec5b026cbbd97ca95f4ed9734`;
- GO head: `181dc76e902882e024fe9c3a6b0c179d5f635571`;
- fresh panel: seed 2770, `dev,dev`, 128 conversations, 15,591 tokens;
- fresh tokens SHA-256:
  `5f18d4aa8a1b0a048d797cc5becc34efe42aebb718e90ba580e8e1b01960eddc`;
- K16 training: seed 1 **2,753** groups / 348 targets / 865 baseline-wrong;
  seed 2 **2,219** / 343 / 635;
- observed writer candidates: **255-256** against a 256 cap;
- reachable replies: **426** per artifact;
- decisions per arm: **27,264**;
- generated tokens per arm: **26,838**;
- K16 seed walls: **10.31 / 10.10 / 10.33 / 10.09 s**;
- K16 peak RSS: **254,272 / 263,128 / 244,360 / 238,168 KiB**;
- build/generator/panel: **0.05 / 0.05 / 0.70 s**;
- K12 failed seed-2 eligibility wall: **0.43 s**;
- CPU library: **335 passed**, 4 expected ignores;
- driver tests: **3 passed**;
- focused anchored tests: **9 passed**;
- raw evidence: **147,114 bytes**, sealed and verified;

Published evidence is in
[`../../../../labs/cpu-training-pilot-copy-key-reply-2026-11-05/`](../../../../labs/cpu-training-pilot-copy-key-reply-2026-11-05/)
under `evidence/expanded-candidate-*`.

## Checks run at the head

- Independent fresh-context TEST FITNESS: both registrations reached FIT after
  three review rounds each.
- Independent fresh-context pre-run review: GO.
- CPU library, driver tests, strict Clippy, package rustfmt and
  `git diff --check`.
- Source, generator, tokenizer, artifact, train-panel and calibration pins.
- Registered shell/jq syntax, six-arm gates and control-vs-longest checks.
- Both fresh-process repeats matched after timing fields were removed.
- Raw and published evidence seals.
- Lane-scope and no-corpus checks.

## Size

The implementation adds the new anchored candidate module and driver mode.
No corpus payload is committed.

## Skills used

No system-provided Codex skill applied. Repository preregistration,
compute-admission and delivery procedures were used.

## Still open

The fitted linear correction is not a viable substitute for longest-32. A later
copy/body experiment should change the candidate-conditioned representation or
objective rather than widen cells or add more similar features. Independent
termination remains the larger completion blocker. Seed 2769 and seed 2770 are
closed for these constructions.

## Plain summary

The first corrected selection used too few cells and stopped before evaluation;
it is recorded as a selection instrument fault. The corrected 16-cell run
cleared all training floors and completed the full comparison, but the anchored
linear ranker overrode the stronger longest-32 baseline almost everywhere and
lost decisively on both artifacts. This closes the fitted K16 correction
configuration, not geometry-based candidate ranking generally.
