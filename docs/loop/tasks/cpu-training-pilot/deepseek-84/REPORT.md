# REPORT: stale margin reading repair, deepseek-84

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base: `de9cfc84a`.

## What did not work

- The deepseek-82 REPORT retained the superseded seed-2 K4 margin-loss target
  `29,602,710` after the registered trainer-seed-1 rerun.
- The bound seed-2 K4 receipt records `29,471,323`; the deepseek-83 REPORT,
  comparison and receipt were already correct.

This was a documentation consistency fault. It did not change the experiment,
the exact export checks or the negative ranking result.

## What changed

Corrected the stale value in the deepseek-82 REPORT and searched the bound
reports, notebook, program and roadmap for the superseded reading. No other
bound value remained.

## Measurements

The repaired seed-2 K4 arm reduced the margin-loss sum from `29,717,075` to
`29,471,323`, accepted 238 four-coordinate blocks and moved 952 coordinates.
Its top-1/top-8/admitted readings stayed 224/285/292.

The six bound receipts retain exact CPU/serving parity over 262,144 Q12 query
coordinates and 8,192 kept cells per arm. Seed-1 K4 exits 2 at the required
admission gate; the other five runs exit 0.

## Checks run at the head

- All bound JSON parses and evidence `SHA256SUMS` passes.
- The stale and superseded value search returns no bound reading.
- Formatting, `git diff --check`, local PR gate and required GitHub checks are
  recorded at the repaired head.

## Skills used

No available Codex skill applied. The review finding and bound evidence were
compared directly.

## Still open

- No tested CPU configuration has cleared a registered public ranking or
  capability bar.
- Trained float-shadow, complete-reply and fresh-phrasing outcomes remain
  outside this public CPU result.

## Plain summary

The rerun evidence was correct, but one older report still displayed the
superseded seed-2 margin value. This repair makes every bound report agree with
the trainer-seed-1 receipt without changing the negative result.
