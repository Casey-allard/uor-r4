# ORDER: cpu-training-pilot, deepseek-128

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `e5b7c500c`.

## Goal

Reduce repeated full-trunk decoding in `cpu-trunk-count-driver` verification.
The deepseek-127 profile found exact plus shuffled verification consuming
45.32%-48.05% of wall time. Compare the optimized driver against the current
127 binary in paired runs while preserving every artifact and non-stage report
field.

This is D84 CPU performance/instrument research. It is not a capability or
GPU-qualification claim.

## Design

- Parse the source trunk once for source semantics and source-table summaries,
  consuming those decoded tables into the exact expected-count reconstruction.
- Parse each exported arm once for semantic comparison, decoded-table
  comparison and table summaries.
- Parse the source a second time only to build the independent shuffled-arm
  expected tables.
- Reduce verification from ten full trunk imports to four without cloning a
  complete source table set or weakening independent reconstruction.
- Keep the existing 21-stage profile, source-order parallel work and model math.
- Compare old and new binaries at workers 1 and 16 in mirrored paired order.

## Pre-registration

Write `docs/harness/prep/cpu-training-pilot-trunk-verify-2026-11-04.md` and
obtain a fresh-context non-author FIT before public timing.

Primary bar:

- mean total wall time is at least 10% lower at both 1 and 16 workers;
- exact plus shuffled verification stage time is at least 20% lower at both
  worker counts;
- both optimized worker counts reproduce the deepseek-125/126 artifact and
  normalized non-stage report oracles, with strict positive RSS below 2 GiB.

A fail closes this fixed consolidation only.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-trunk-verify-2026-11-04.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-128/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/harness/claims.json`

## Done when

1. Finite tests show the consolidated import path returns the same semantics,
   expected tables and summaries on success, and refuses mismatch controls.
2. FIT is recorded before the paired public runs.
3. The eight runs satisfy the 10% total-wall and 20% verification-stage bars at
   both worker counts, plus all identity and resource gates.
4. Report, evidence, notebook/roadmap, CPU, documentation and loop gates pass,
   followed by fresh exact-head review, protected merge and fresh-main
   verification.

## Stop

Stop on artifact/report drift, a weakened independent table check, a cap
failure, either performance bar failing, or overlap with Claude-owned
float-training or serving files.
