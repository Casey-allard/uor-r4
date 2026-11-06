# REPORT: cpu-training-pilot, deepseek-226

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `326821252`.

## Status

Ready as release bookkeeping; this PR removes the completed
`deepseek/content-prefix-endpoint` claim.

## What did not work

No further source work was needed. Deepseek-225 remains a **REJECT** for the
fixed full-prefix endpoint guard. It preserved the boundary arm's 17 exact
replies and 644/642 body positions, changed 23 complete outcomes, and reduced
premature BOS from 73/91 to 67/85, but only 206/229 of 471 reachable replies
completed and the true guard did not beat shuffled retrieval on exact replies.

## What changed

Records protected delivery of #3784 and removes the completed claim.

## Measurements

- reviewed head:
  `70fa6a97488a91621e6867e945be1ad9d166c498`;
- protected merge:
  `326821252b2c2fc2e00a1f75856168d436943e03`;
- final evaluator source:
  `90fb52752b3a09315a43da775e0521a7d78524f8830a4613e4cad69ebed2c54b`;
- final retrieval source:
  `68f19750e8c14370a41a07b4897eec238e1d48313683250667e02052a58970c4`;
- final driver source:
  `6be78357935a900aba8e36611066c99e78218f85bffbe4dae4a637214ca0b3e5`;
- release binary:
  `4671b279340132fc1d9fa078de533d7f2ecf118d229deafe0118f02f78b4191e`;
- compact evidence:
  `1741a6328ec59df9cbf9a09837a6aa7edcd356de444cb93c59c7852672cbe5f6`;
- local evidence manifest:
  `eacbdbfc0f82f72d1f79574d9aef7b8b113cd32eebf8edc16718c6e6ef525340`.

Fresh-main source, binary, compact evidence and all 18 local evidence entries
verified. Seed-1 and seed-2 reports remain byte-identical to their repeats.

## Checks run at the head

- fresh-main aligned-endpoint tests: 2 passed;
- fresh-main driver test: 1 passed;
- reviewed-head CPU library: 406 passed, 4 expected ignores;
- fresh-main source, published evidence and raw evidence checks passed;
- D23, D25, D32 and claim wording passed;
- prep-check reports the aligned-endpoint sheet FIT and complete;
- local PR policy and all 11 full loop gates passed;
- required CI checks passed at the reviewed head.

## Skills used

No system-provided Codex skill applied.

## Still open

- a generated-state stop/resume policy that survives content divergence;
- a stop signal trained on generated trajectories rather than prompt-level
  retrieved replies;
- Claude's independent end-to-end serving track.

## Plain summary

The aligned-endpoint REJECT is on `main` and its implementation claim is
released by this bookkeeping PR.
