# ORDER: bounded CPU rank-margin step-size comparison, deepseek-9

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `9c9b712a4`.

## Goal

Run a materially new CPU training comparison rather than repeat the known
256-step, one-Q12-unit rank-margin configuration. Expose the trainer's existing
margin, coordinate-step and parameter-limit settings through the corpus driver,
bind them into checkpoint/resume identity, pre-register a fixed two-seed
comparison, and run it on the public FBS artifacts and document-disjoint corpus.

The arms test whether a larger admissible coordinate step produces held-out
ranking movement while preserving target admission and exact CPU/serving export.
This remains development research under D84, not GPU qualification.

## Cancel probe

Read `cpu-corpus-driver`, `RankMarginConfig`, the failed 256-step pilot and its
prep sheet. If the driver already accepts and checkpoint-binds these three
settings, reuse it and do not duplicate them.

## Scope

- `crates/uor-r4-cpu-train/src/bin/cpu-corpus-driver.rs`
- `docs/harness/prep/cpu-training-pilot-rank-step-2026-11-02.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-9/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Required design

- Preserve the existing CLI and defaults when no new option is supplied.
- Accept positive `--margin-q12`, `--step-q12` and `--parameter-limit-q12`
  overrides.
- Bind all three values into checkpoint sidecars and refuse a resume whose
  update geometry differs.
- Pre-register, before compute, the public artifacts, corpus, split, baseline,
  update counts, arms, pass/fail bars, cost and stop rules.
- Compare at least the default unit step against one materially larger step at
  two published artifact seeds over the same 1,024-row document-disjoint panel.
  The exact arms are fixed by the prep sheet before any run.

## Done when

- Parser and checkpoint-binding tests cover defaults, overrides, malformed
  values, positive-value refusal and resume mismatch.
- The full CPU library test suite and CPU-local strict Clippy pass; formatting
  and diff checks pass.
- A fresh FIT review approves the prep sheet before compute.
- Every registered arm completes with exact CPU/serving export; an admission
  or ranking miss against the pre-registered bar is retained as a negative
  outcome rather than relabelled as success.
- The REPORT and notebook preserve raw panel readings, update acceptance,
  throughput/cost and the registered verdict on both seeds.
