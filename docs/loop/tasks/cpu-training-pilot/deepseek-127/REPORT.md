# REPORT: public 19-table CPU count stage profile, deepseek-127

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[stage profile](../../../harness/prep/cpu-training-pilot-trunk-stages-2026-11-04.md).
Base: `3441ba625`.

## What did not work, first

- **This did not produce a faster training configuration.** It measured the
  existing configuration after deepseek-126's 1.5x parallel-speed bar failed.
  The new profile includes timing overhead and is not a speed comparison.
- **A single-stage optimization would address less than half the wall.**
  `exact_verify` was the stable largest stage, but exact and shuffled
  verification together consumed 45.3% / 48.0% of wall time at one / sixteen
  workers. All three held-out evaluations consumed another 32.2% / 32.0%.
- **The first four-run attempt was superseded.** Exact-head review found that
  the finite test did not require the complete 21-stage set. The driver now
  rejects duplicate, missing and extra stages, and the retained result is a
  fresh second attempt under that binary.
- **No model or capability question was tested.** Artifact, table, decision and
  checksum identity with deepseek-125/126 was the required control, not a
  training result.

## What changed

`cpu-trunk-count-driver` now records 21 monotonic stages covering:

- input read/hash and validation;
- corpus split and reader load;
- training-row and control-row construction;
- exact and shuffled update/export/hash/write;
- exact and shuffled independent verification;
- all three held-out evaluations;
- parity, changed-decision and resource reductions.

The report includes each stage duration, attributed time and unattributed time
before final report serialization. Duplicate stage names, missing or extra
stages, and invalid accounting fail closed. Model math, update order and
artifact serialization are unchanged.

## Measurements

The fixed split again held 200 training documents / 7,080 targets and 193
evaluation documents / 512 targets. Four fresh runs covered `1,16` and the
mirrored `16,1` worker order.

| Workers | Pass | In-process wall | Peak RSS |
|---:|---|---:|---:|
| 1 | forward | 10.140 s | 1,543,452 KiB |
| 16 | forward | 9.376 s | 1,544,388 KiB |
| 16 | reverse | 9.487 s | 1,544,968 KiB |
| 1 | reverse | 9.967 s | 1,544,104 KiB |

Order-averaged walls were 10.053 s at one worker and 9.431 s at sixteen.

### Stable hotspot

| Stage | 1-worker mean | 1-worker share | 16-worker mean | 16-worker share |
|---|---:|---:|---:|---:|
| `exact_verify` | **2,684.618 ms** | **26.70%** | **2,666.162 ms** | **28.27%** |
| `shuffled_verify` | 1,871.711 ms | 18.62% | 1,865.407 ms | 19.78% |
| `shuffled_evaluation` | 1,097.514 ms | 10.92% | 1,007.868 ms | 10.69% |
| `no_update_evaluation` | 1,072.497 ms | 10.67% | 1,004.498 ms | 10.65% |
| `exact_evaluation` | 1,067.013 ms | 10.61% | 1,007.264 ms | 10.68% |

`exact_verify` is the largest stage at both worker counts, so the predeclared
hotspot rule selects it directly rather than a broader leading group.

The verification path currently calls full trunk import/decode work several
times per arm: semantic comparison, independent expected-count reconstruction,
decoded-table comparison and table reading each reconstruct the same data. The
next performance unit should consolidate those traversals while preserving the
independent full-table check. This is the measured next target; no optimization
was applied after seeing the profile.

All four runs produced exact artifact
`db4a672d6af83c4c56b7fbccc9ebc2279cb13fb8719b47829db64c910f7d2bb2`
and shuffled artifact
`19cbb1caa487473ac4344e1b3582060fbcb22b1e6066fa51540bcbc8977eabcd`.
Every normalized report after deleting `stages`, wall time, RSS and replacing
threads was
`64e6e9b971ae0e90281aa485112db595e440eddeb29dbf99c9976a1fc467722f`.
Every arm retained zero query, admitted-cell, top-8 and token mismatches over
512 rows / 16,384 Q12 coordinates.

Peak RSS ranged from 1,543,452-1,544,968 KiB, below the strict 2-GiB stop.
Every run completed below the 600-second timeout. The four roots occupy
**695,462,128 bytes**.

The compact evidence is [comparison.json](evidence/comparison.json);
[SHA256SUMS](evidence/SHA256SUMS) binds the binary, reports and both artifact
sets.

## Checks run at the head

- Fresh-context non-author TEST FITNESS: FIT at docs head
  `1994b289b1e15bddbb7305cccc17be3f8feba706`, after repairing the expected
  stage-key order and adding the canonical 21-stage fail-closed test.
- Full `uor-r4-cpu-train` release suite: **236 tests passed, 0 failed, 4
  benchmarks ignored**.
- Strict no-deps all-target CPU Clippy with `-D warnings`: passed.
- Focused complete/missing/extra/duplicate stage-accounting, ordered-fold,
  row/artifact/parity and fail-closed RSS tests: passed.
- All four runs passed the pre-registered 21-stage, artifact, normalized-report,
  timeout and strict RSS gates.
- Prep shell syntax, evidence checksum reconstruction and `git diff --check`
  passed.
- Full loop gates: **11/11 green**. Report, roadmap (139 items), documentation
  catalog (1,175 records) and D23/D25/D32 ratchets passed.
- `uor-harness check`: 0 errors and 135 pre-existing warnings, chiefly stale
  claims, old merged branches and old presence records outside this unit.

## Skills used

No system-provided skill applied.

## Still open

- Consolidate repeated full-trunk decoding in exact and shuffled verification,
  then measure whether it reduces wall time without weakening the independent
  table check.
- Complete replies, fresh phrasing, useful local chat and GPU qualification.

## Plain summary

The count driver's slower half is now measured rather than inferred. Checking
the two exported artifacts accounts for roughly half the wall time, with
`exact_verify` consistently the largest single stage. The next useful speed
work is to remove duplicate full-trunk parsing from verification, while keeping
the independent count reconstruction and exact CPU/serving controls.
