# ORDER: cpu-training-pilot, deepseek-129

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `6824d2d58`.

## Goal

Reuse the CPU and integer models already loaded for input validation and
training-row derivation when evaluating the unchanged arm. The deepseek-128
profile leaves exact, no-update and shuffled evaluation each near one second;
their combined parse work matches the separate input-validation and reader-load
stages almost exactly.

This is D84 CPU performance/instrument research. It is not a capability or
GPU-qualification claim.

## Design

- Retain the evaluated integer model from input validation.
- Derive training rows with the existing CPU trunk reader, then evaluate the
  no-update held-out arm with those two already-loaded models.
- Split `evaluate` into a preloaded-model helper plus the current independent
  artifact-loading wrapper.
- Drop the retained integer model before update/export, exact verification and
  shuffled verification begin.
- Keep independent reloads and CPU/serving parity for exact and shuffled arms.
- Add a finite test proving preloaded evaluation equals the independent
  reloading path on a real exported D11 artifact.
- Compare the deepseek-128 binary with the reuse binary at workers 1 and 16 in
  mirrored order.

## Pre-registration

Write `docs/harness/prep/cpu-training-pilot-trunk-eval-reuse-2026-11-04.md`
and obtain a fresh-context non-author FIT before public timing.

Primary bar:

- mean total wall time is at least 5% lower at both 1 and 16 workers;
- mean `no_update_evaluation` stage time is at least 50% lower at both worker
  counts;
- every artifact, table, decision, checksum and non-stage report field remains
  identical, with strict positive RSS below 2 GiB.

A fail closes this fixed reuse configuration only.

## Cancel probe

- Confirm the base is current `origin/main` and both source/binary inputs can
  be reproduced.
- Confirm no other lab is changing the CPU count driver or the benchmark
  inputs.
- Confirm the no-update evaluation can reuse the reader before it is consumed
  by exact export.
- Confirm the live per-arm parse cost remains the measured dominant part of
  `no_update_evaluation`.
- Refuse if the change would retain either model through verification or raise
  the 2-GiB stop.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-trunk-eval-reuse-2026-11-04.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-129/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/harness/claims.json`

## Done when

1. A finite test proves preloaded and freshly loaded evaluation produce the
   same `DecisionReading` on a real D11 artifact.
2. FIT is recorded before the public paired runs.
3. Eight runs meet the 5% wall and 50% no-update-stage bars at both worker
   counts with exact identity and resource gates.
4. Report, evidence, notebook/roadmap, CPU, documentation and loop gates pass,
   followed by fresh exact-head review, protected merge and fresh-main
   verification.

## Stop

Stop on artifact/report drift, a weakened parity control, a cap failure, either
performance bar failing, or overlap with Claude-owned float-training or serving
files.
