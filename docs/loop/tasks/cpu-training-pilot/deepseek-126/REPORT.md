# REPORT: parallel public 19-table CPU count driver, deepseek-126

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[parallel count driver](../../../harness/prep/cpu-training-pilot-trunk-parallel-2026-11-04.md).
Base: `a1c22df3e`.

## What did not work, first

- **The 1.5x performance bar failed.** Mean walls were 10.600 s at one worker,
  10.081 s at eight, 9.951 s at sixteen and 10.022 s at thirty-two. The best
  speedup was **1.065x** at sixteen workers, not 1.5x.
- **More workers did not keep helping.** Sixteen workers beat thirty-two by
  about 0.7%, and all parallel settings remained within about 6.5% of one
  worker.
- **The result shows the two parallelized loops are not the dominant cost.**
  GNU time reported 99-108% process CPU across the runs, while the wall times
  barely moved. This points to serial table update, export, reload,
  reconstruction and artifact I/O dominating this driver. No stage breakdown
  was registered in this unit, so that attribution remains a hypothesis.
- **This is not a model-math or capability negative.** The parallel result
  reproduced every accepted artifact and decision byte-for-byte. It closes
  only the tested parallelization configuration and the 1.5x claim for this
  driver.
- The first three fresh FITNESS reviews found and closed missing accepted-output
  pins, an accidental global Rayon initialization, insufficient evaluator test
  coverage, order-biased timing, non-positive RSS acceptance and finally the
  inclusive RSS boundary. The fourth review returned FIT.

## What changed

`cpu-trunk-count-driver` now:

- accepts an explicit bounded `--workers` count;
- creates one local Rayon pool and performs all parallel work inside it;
- derives the 4,096 training rows with indexed parallel iteration and collects
  them in source order before the serial online count update;
- evaluates held-out CPU/integer rows in parallel, then folds counters and both
  checksums in source order;
- reports the actual pool thread count; and
- fails closed when peak RSS is unavailable, malformed or zero.

Tests reproduce serial training rows, updated artifacts, decisions, CPU/serving
parity and the ordered checksum fold at 1, 8, 16 and 32 workers. A separate test
covers missing, zero and malformed RSS readings. No model math changed.

## Measurements

The accepted deepseek-125 split and dose were replayed exactly: 200 training
documents / 7,080 targets, 193 evaluation documents / 512 targets, the first
4,096 training rows and seed 1.

| Workers | Forward wall | Reverse wall | Mean wall | Speedup vs one |
|---:|---:|---:|---:|---:|
| 1 | 10.531 s | 10.670 s | **10.600 s** | 1.000x |
| 8 | 9.927 s | 10.235 s | **10.081 s** | 1.052x |
| 16 | 9.866 s | 10.037 s | **9.951 s** | **1.065x** |
| 32 | 10.037 s | 10.007 s | **10.022 s** | 1.058x |

All eight runs produced exact artifact
`db4a672d6af83c4c56b7fbccc9ebc2279cb13fb8719b47829db64c910f7d2bb2`
and shuffled artifact
`19cbb1caa487473ac4344e1b3582060fbcb22b1e6066fa51540bcbc8977eabcd`.
They were byte-identical inside and across both worker orders.

Every report had the same normalized SHA-256
`64e6e9b971ae0e90281aa485112db595e440eddeb29dbf99c9976a1fc467722f`
after replacing `threads` with `1` and deleting wall time and peak RSS. Thus all
13 table totals, table contents, semantic sections, selection, readings,
checksums and changed-decision counts reproduced deepseek-125.

No-update / exact / shuffled remained **332/246/270**, **331/245/269** and
**332/246/270** admitted / top-1 / top-8. Exact changed four rows and shuffled
changed zero. Every arm had zero query-coordinate, admitted-cell, top-8 and
token mismatches over 512 rows / 16,384 Q12 coordinates.

Peak RSS was 1,543,092-1,546,892 KiB, below the strict 2-GiB stop. Every run
completed below the 600-second timeout. The eight roots occupy **1,390,916,646
bytes**.

The compact evidence is [comparison.json](evidence/comparison.json);
[SHA256SUMS](evidence/SHA256SUMS) binds the driver, reports and both artifact
sets.

## Checks run at the head

- Four fresh-context non-author fitness reviews; the first three returned
  NOT_FIT and the repaired sheet returned FIT.
- Full `uor-r4-cpu-train` release suite: **235 tests passed, 0 failed, 4 release
  benchmarks ignored**.
- Strict no-deps all-target CPU Clippy with `-D warnings`: passed.
- Package formatting: passed.
- Focused ordered-fold, row/artifact/parity and fail-closed RSS tests: passed at
  workers 1, 8, 16 and 32.
- Prep check: complete and FIT; prep shell syntax, report check, roadmap check
  (139 items), documentation catalog (1,175 records), D23/D25/D32 ratchets and
  evidence checksum reconstruction passed.
- Full loop gates: **11/11 green**. The first attempt found only unlinked
  equivalence wording in the ORDER; that phrase was repaired and the complete
  gate set passed.
- `uor-harness check`: 0 errors and 135 pre-existing warnings, chiefly stale
  claims, old merged branches and old presence records outside this unit.

## Skills used

No system-provided skill applied.

## Still open

- Add stage timing before changing another performance threshold. The next
  useful analysis is to measure how much of the flat wall time lies outside
  the two parallel loops, including artifact update/export/reload.
- Do not retune this failed 1.5x bar after seeing the result. A later unit needs
  a fresh FIT and a different registered target.
- Complete replies, fresh phrasing, useful local chat and GPU qualification.

## Plain summary

The driver now uses up to 32 local workers while preserving every accepted
artifact and decision exactly. On this workload that did not make training
1.5x faster: the best measured gain was about 6.5%, and using 16 workers was
slightly better than 32. The flat wall time and roughly one-core utilization
suggest that the two parallel loops are a small share of this driver's total
work. Stage timing is required before naming the remaining bottleneck or
changing the design to target it.
