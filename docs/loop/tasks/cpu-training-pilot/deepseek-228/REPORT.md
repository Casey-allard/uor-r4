# REPORT: cpu-training-pilot, deepseek-228

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `5f19e0d82`.

## Status

Ready as release bookkeeping.

## What did not work

Deepseek-227 remains a **REJECT** for fixed K=8 branch coverage. It doubled
endpoint stops from 23 to 48 and reduced premature BOS to 43/68, but only
175/208 of 437 replies completed and exact/body stayed at boundary 14 and
936/933. The shuffled guard also made 14 exact replies.

## What changed

Records protected delivery of #3786 and removes the completed claim.

## Measurements

- reviewed head:
  `89b8caab75ba2c5743cfc9c5c1f66074d0b4be17`;
- protected merge:
  `5f19e0d82e53ebb612accdf637ecdcd66fcd7b2f`;
- compact evidence:
  `34c067fffbec9d269ef9c38face684c96c93418c206a4863e72060e0a88a0979`;
- raw evidence manifest:
  `17869456a2d79691a3e7d2b37c35f37e276c4ec5bad9bd1ca5b886c241618d31`.

Fresh-main source, compact evidence, raw evidence and repeats verified.

## Checks run at the head

- fresh-main branch and driver tests passed;
- reviewed-head CPU library: 407 passed, 4 expected ignores;
- all 11 full loop gates passed;
- required CI checks passed at the reviewed head.

## Skills used

No system-provided Codex skill applied.

## Still open

- generated-state branch identity rather than retrieved-candidate membership;
- native termination and complete-reply readiness;
- Claude's independent end-to-end serving track.

## Plain summary

The K=8 branch-endpoint REJECT is on `main` and its claim is released here.
