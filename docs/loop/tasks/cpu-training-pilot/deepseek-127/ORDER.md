# ORDER: cpu-training-pilot, deepseek-127

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `3441ba625`.

## Goal

Measure where `cpu-trunk-count-driver` spends its wall time after
deepseek-126 showed only a 1.065x gain from parallelizing row derivation and
held-out evaluation. Add stage timing without changing model math or result
artifacts, then run the fixed public workload at 1 and 16 workers in mirrored
order.

This is D84 CPU performance/instrument research. It is not a training-quality,
capability or GPU-qualification claim.

## Design

- Keep the source-order parallel row and evaluation paths unchanged.
- Record monotonic elapsed milliseconds for input loading, manifest/shape
  checks, corpus splitting, row derivation, control construction, exact and
  shuffled update/export/write, independent verification, each held-out arm,
  and final table/parity reductions.
- Report attributed and unattributed time before final report serialization.
- Preserve both artifact bytes and every decision reading. Compare the new
  reports to deepseek-126 after removing only stage timings, wall time, RSS and
  worker count.
- Run workers 1 and 16 in both orders with fresh roots.

## Pre-registration

Write `docs/harness/prep/cpu-training-pilot-trunk-stages-2026-11-04.md` and
obtain a fresh-context non-author FIT before public compute. Every stage must
be finite and non-negative, attribution must not exceed total driver wall time,
both worker counts must reproduce accepted artifacts and decisions, and no
stage may be selected after seeing its result.

The output is a measured stage breakdown and a hypothesis for the next unit,
not a new speed threshold.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-trunk-stages-2026-11-04.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-127/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/harness/claims.json`

## Done when

1. Finite tests verify stage accumulation is complete, finite and
   non-negative.
2. FIT is recorded before the public timing runs.
3. Both worker counts reproduce the deepseek-126 artifacts and normalized
   result fields, with positive RSS strictly below 2 GiB and each run below
   600 seconds.
4. Evidence reports the stage shares and identifies a next optimization target
   only where the measured breakdown supports it.
5. Report, notebook/roadmap, CPU, documentation and loop gates pass, followed
   by fresh exact-head review, protected merge and fresh-main verification.

## Stop

Stop on source/artifact drift, any decision mismatch, a cap failure, unstable
stage accounting, a need to change model math, or overlap with Claude-owned
float-training or serving files.
