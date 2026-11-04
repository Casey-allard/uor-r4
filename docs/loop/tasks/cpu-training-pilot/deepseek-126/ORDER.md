# ORDER: cpu-training-pilot, deepseek-126

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `a1c22df3e`.

## Goal

Parallelize the merged public 19-table CPU count driver without changing its
model math, result bytes or source ordering. Measure the wall-time gain at
1, 8, 16 and 32 workers on the fixed public artifact and require exact and
shuffled artifacts, decisions, table contents and checksums to remain
byte-identical.

This is D84 CPU performance/instrument work. It is not a capability,
training-quality or GPU-qualification claim.

## Design

- Add an explicit bounded `--workers` argument and run the driver in one local
  Rayon pool. Do not create a nested pool.
- Derive independent training rows with indexed parallel iteration and collect
  into deterministic source order before the serial online count update.
- Evaluate independent held-out CPU/integer decisions in parallel, then fold
  outcomes and checksums in the original row order.
- Preserve all existing arms, selection, bounds and artifact serialization.
- Add finite tests that compare row derivation and updated artifacts at 1, 8,
  16 and 32 workers with the serial reference.
- Run the fixed 4,096-row public comparison under fresh output roots and the
  registered 600-second and 2-GiB caps. Normalize only timing, RSS and worker
  count when comparing reports.

## Pre-registration

Write `docs/harness/prep/cpu-training-pilot-trunk-parallel-2026-11-04.md`
and obtain a fresh-context non-author FIT before public worker-count compute.
The required result is identical artifacts and decisions at every worker count
plus a measured wall-speedup; a slower or equal parallel path is an instrument
result, not a model negative.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-trunk-parallel-2026-11-04.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-126/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/harness/claims.json`

## Done when

1. The driver records its actual worker count and uses one bounded Rayon pool.
2. Finite tests reproduce serial training rows and artifacts at 1, 8, 16 and
   32 workers.
3. FIT is recorded before the fixed public comparison.
4. All public runs preserve byte-identical exact and shuffled artifacts,
   decision checksums, readings and deterministic binary outputs while staying
   below the registered caps.
5. Report, evidence, notebook/roadmap, CPU, documentation and loop gates pass,
   followed by fresh exact-head review, protected merge and fresh-main
   verification.

## Stop

Stop on source/artifact drift, any decision or artifact mismatch, failure of a
non-vacuous control, a cap failure, nested Rayon pools, or any need to edit
Claude-owned float-training or serving files.
