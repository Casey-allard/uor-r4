# ORDER: make public CPU corpus training restartable, deepseek-3

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `20422589a`.

## Goal

Close the public-run interval-restart gap without changing the completed rank-margin pilot, its negative result or any serving code. A driver run must be able to continue from a hash-bound interval checkpoint and produce the same final artifact and checkpoint bytes as the uninterrupted run.

## Scope

- `crates/uor-r4-cpu-train/src/bin/cpu-corpus-driver.rs`
- `crates/uor-r4-cpu-train/README.md`
- `docs/harness/prep/cpu-training-pilot-public-resume-control-2026-11-02.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-3/**`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/research/cpu-training-program.md`
- `docs/research/notebook/cpu-training.md`

No serving edit, trainer-algorithm change, GPU run, rewrite of the sealed pilot result, untuned extension of the negative, or complete-reply/fresh-phrasing claim.

## Implementation

- Add an optional resume-checkpoint argument while preserving the documented ten-argument fresh-run command.
- Treat the existing `<steps>` argument as the total target step, so a resumed invocation executes only the remaining attempts.
- Write a checkpoint binding beside every interval and final checkpoint. Bind the source artifact, token, mask and manifest hashes, tokenizer, trainer seed, split controls, completed step and checkpoint hash.
- Refuse a resume when the binding, pinned input, checkpoint hash or step count differs.
- Report the resumed-from step, executed-step count and verified checkpoint identity.
- Classify the public restart comparison as a host determinism check under CHARTER §4.5; do not fabricate a FIT clearance or present it as a pre-registered capability run.

## Done when

- A finite interval checkpoint resumed through the driver produces byte-identical final artifact and final checkpoint hashes to the uninterrupted run.
- Tampered source, corpus, seed, split or checkpoint identity is refused before training continues.
- A two-seed host restart comparison is executed from the same pinned inputs and records byte equality or an explicit instrument failure.
- Focused tests, the CPU crate suite, Clippy, formatting, roadmap checks and harness checks pass at the reviewed head.

## Stop

Stop on any checkpoint identity mismatch, non-exact CPU/serving export, admission loss or non-deterministic continuation. Retain a failure as an instrument result, never as a training negative.
