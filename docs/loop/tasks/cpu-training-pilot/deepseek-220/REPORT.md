# REPORT: cpu-training-pilot, deepseek-220

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `00e30f316`.

## Status

Ready as release bookkeeping; this PR removes the completed
`deepseek/prefix-length` claim.

## What did not work

No further source work was needed. Deepseek-219 remains a bounded **KEEP**:
retrieved-prefix length completion clears its matched controls but produces
only 9/459 and 10/459 exact replies, and native termination remains unresolved.

## What changed

Records protected delivery of #3778 and removes the completed
`deepseek/prefix-length` claim.

## Measurements

- reviewed head:
  `c92609064dc43de6ea27ea2ff89e5ae065a2506b`;
- protected merge:
  `00e30f316fe820d6c8290fa5cca051446f6bfb85`;
- binary source:
  `98a6c26a757c0b00dce2414119d49af95d36d0a55c82ad207c1e6c770c269312`;
- release binary:
  `91ed9c0182cf4350873e6e8fd17f3b6de0553955513c2184149664e8692c67a1`;
- compact evidence:
  `8adb04faba7df02584563039c35cbc93fa628f423373314fb4de83ccc09b9452`;
- local evidence manifest:
  `1cee8582ca5f374371f2e1f0cd7249b56babf0b39770b879187379ab6baedc41`.

Fresh-main source, binary source, compact evidence and all 24 local evidence
entries verified.

## Checks run at the head

- fresh-main focused prefix-length tests: 4 passed;
- fresh-main full CPU library: 399 passed, 4 expected ignores;
- fresh-main D23/D25/D32 ratchets and claim wording passed;
- fresh-main prep-check reports FIT and no failure;
- fresh-main report check passes with the release ORDER named.

## Skills used

No system-provided Codex skill applied.

## Still open

- a model-derived stop that removes reliance on retrieved length;
- a serving integration only if it adds unique complete replies over full
  retrieval;
- Claude's independent end-to-end serving track.

## Plain summary

The prefix-length KEEP is on `main` and its implementation claim is released
by this bookkeeping PR.
