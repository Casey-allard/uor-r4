# ORDER: cpu-training-pilot, deepseek-100 (measure the public worker budget)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `6729e5d507bd39ef400c6515bc29b75f97e682d7`.

## Goal

Measure the real public corpus driver at 1, 8, 16 and 32 Rayon workers, record
the stage costs behind its end-to-end wall, and verify that worker count changes
timing rather than the trained artifact, checkpoint, panel reading or audit.

## Cancel probe (run first)

The current driver already uses ordered parallel panel evaluation, while its
online rank-margin updates and recurrent served-forward audit are dependency
ordered. A 16-step public run at 16 workers determines whether a repeatable
wall-time difference is present; if no tested budget is faster, preserve the
reading as an instrument negative.

## Files in scope

- `crates/uor-r4-cpu-train/src/bin/cpu-corpus-driver.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-100/**`
- `docs/research/cpu-training-program.md`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Done when

1. The driver records its effective Rayon worker count and bounded stage
   timings for setup, baseline panel, online updates, final panel, checkpoint
   and final served-forward audit.
2. One unmodified public artifact seed runs the same 16-update configuration at
   1, 8, 16 and 32 workers.
3. All worker counts reproduce the same panel readings, accepted steps,
   artifact SHA-256, checkpoint SHA-256 and audit decision SHA-256.
4. Retained evidence records wall time, update time, panel time, audit time,
   CPU utilization and peak RSS, plus a recommendation for future local runs.
5. Focused release tests, strict Clippy, formatting and `git diff --check` pass.
6. The REPORT explicitly separates this performance instrument from capability
   evidence and does not change a model bar or claim.

## Stop and ask

- Stop if any worker count changes a trained byte or reading.
- Stop if the new stage counters perturb the published integer path.
- Do not broaden this unit into serving concurrency, GPU work or a new training
  configuration.
