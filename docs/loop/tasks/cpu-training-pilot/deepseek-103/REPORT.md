# REPORT: CPU-reading correction, deepseek-103

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base:
`6729e5d507bd39ef400c6515bc29b75f97e682d7`.

## What did not work, first

The third fresh review found three stale CPU-utilization ranges in the
superseded deepseek-100 report. They came from the first timing run, not the
repaired eight-run evidence. The superseded marker explained the stale size and
timing fields but did not explicitly cover these percentages, and the active
repair ORDER required every summary to agree with its raw evidence.

## What changed

The deepseek-100 report now records the raw repaired ranges:

- 8 workers: 758-760%;
- 16 workers: 1,440-1,451%;
- 32 workers: 2,751-2,761%.

The deepseek-102 report points to this active record. No timing evidence, code,
artifact, panel reading, worker recommendation or model result changed.

The exact PR diff is 55 files: 5,591 insertions and 17 deletions.

## Measurements

The retained timing logs establish:

| Workers | Forward CPU | Reverse CPU |
|---:|---:|---:|
| 8 | 758% | 760% |
| 16 | 1,440% | 1,451% |
| 32 | 2,751% | 2,761% |

The previously accepted worker result is unchanged:

- 32-worker single-run walls: 3.534 s forward and 3.522 s reverse;
- one-worker average: 62.147 s;
- average 32-worker speedup: 17.62x;
- two concurrent 16-worker artifact jobs: 6.60 s versus 7.06 s sequential,
  a 6.5% wall reduction.

## Checks run at the head

- Raw CPU ranges were re-read from all eight retained `/usr/bin/time` logs.
- Documented semantic comparison returned exactly `1`.
- `git diff --check origin/main...HEAD`: passed.
- Evidence `SHA256SUMS`: passed.
- `uor-harness loop report --check`, `loop pr-check --local` and
  `loop gates --full`: passed.
- The accepted implementation's focused release test, release package tests,
  strict Clippy and formatting were unchanged by this record-only repair.

## Skills used

No available Codex skill applied.

## Still open

- A training configuration that clears a registered capability or fresh-reply
  bar.
- Unseen-key transfer, trained float shadow and credentialed complete-reply and
  fresh-phrasing evidence under #3506.
- Automatic independent-job scheduling remains outside this measured
  record-only unit.
- CPU training remains D84 development research and does not replace GPU
  qualification.

## Plain summary

The final cleanup fixes three old percentage ranges that were left from the
first timing attempt in an earlier, superseded report. The raw repaired timing
logs now agree with every summary. The useful conclusion does not change:
32 workers make one public panel-heavy run 17.62 times faster than one worker,
and two independent jobs finish slightly sooner as concurrent 16-worker lanes.
