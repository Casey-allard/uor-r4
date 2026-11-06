# REPORT: cpu-training-pilot, deepseek-230

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `a79f53652`.

## Status

Ready as claim-release bookkeeping.

## What did not work

Deepseek-229 remains a **REJECT**. Its continuation guard suppressed 23
boundaries but reduced exact replies from 17 to 14 and completed only 202/223
of 467 reachable replies.

## What changed

Records delivery of #3788 and removes the completed claim.

## Measurements

- reviewed head:
  `36e1249ff109fdc60879b0d597ee88248ad443b7`;
- protected merge:
  `a79f5365252e728b992f7455098b9345814a9845`;
- compact evidence:
  `392b8f1f08b89efb9e97f486c2e65f0084f4cadb4c2be4803f9361f00520b170`;
- raw evidence manifest:
  `bfa7b6103a1ec247dacd4bfaf4f216ff07a609799ececd391f5f33dd955d9edf`.

Fresh-main source, compact evidence, raw evidence and repeats verified.

## Checks run at the head

- fresh-main continuation-guard and driver tests passed;
- reviewed-head CPU library: 408 passed, 4 expected ignores;
- all 11 full loop gates passed;
- required exact-head CI checks passed.

## Skills used

No system-provided Codex skill applied.

## Still open

- joint content-and-termination training;
- native complete-reply stopping;
- Claude's independent end-to-end serving track.

## Plain summary

The continuation-guard REJECT is on `main`, and its implementation claim is
released here.
