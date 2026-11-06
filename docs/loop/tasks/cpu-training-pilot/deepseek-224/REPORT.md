# REPORT: cpu-training-pilot, deepseek-224

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `ed0295ba1`.

## Status

Ready as release bookkeeping; this PR removes the completed
`deepseek/content-length` claim.

## What did not work

No further source work was needed. Deepseek-223 remains a **REJECT** for the
fixed stored-length substitution: the full-sequence content candidate made
6 exact replies and 560 correct body positions, below the boundary control at
14 exact and 760 body positions. The mechanism still emitted every reachable
reply without premature BOS and beat local and shuffled length controls.

## What changed

Records protected delivery of #3782 and removes the completed claim.

## Measurements

- reviewed head:
  `70c151e6e8ae32e8d753d14668013b637dc9e563`;
- protected merge:
  `ed0295ba1b9900bb0312b545b3d6dc24e4b0aaed`;
- final evaluator source:
  `b404b33856e21f4599d30434acd9ef71ba00ac339cc93aded3676aed2d1fad49`;
- final driver source:
  `4a4da3804e32b6faec78a24357146ed6e61b43a804fc576d688e8397512273a5`;
- release binary:
  `609343405c1f8ae38f699cb971755ff51fe6144be26743858478bcf423725977`;
- compact evidence:
  `04e6658fda4c91e73ae807b749785140fc8351de79a41ae91c26a14fb39782b4`;
- local evidence manifest:
  `440b75dd443a18600c967095f2807c29e043272681e1c3a44a997ed9dd6fb293`.

Fresh-main source, binary, compact evidence and all 19 local evidence entries
verified. Seed-1 and seed-2 reports remain byte-identical to their repeats.

## Checks run at the head

- fresh-main CPU library: 404 passed, 4 expected ignores;
- fresh-main stored-length tests: 2 passed;
- fresh-main driver usage test: 1 passed;
- fresh-main source, published evidence and raw evidence checks passed;
- D23, D25, D32 and claim wording passed;
- prep-check reports the content-length sheet FIT and complete;
- local PR policy and all 11 full loop gates passed;
- required CI checks passed at the reviewed head.

## Skills used

No system-provided Codex skill applied.

## Still open

- a model-derived stop that preserves the full-sequence content trajectory;
- a matched termination objective that changes behavior rather than only table
  assignments;
- Claude's independent end-to-end serving track.

## Plain summary

The stored-length REJECT is on `main` and its implementation claim is released
by this bookkeeping PR.
