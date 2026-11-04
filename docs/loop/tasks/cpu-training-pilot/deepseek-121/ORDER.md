# ORDER: cpu-training-pilot, deepseek-121 (long-window CPU acceptance)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `b6561fe6e`.

## Goal

Support Claude's active 2,048-token training/export track without editing
Claude-owned float-train files. Add a CPU-side full-path FBS acceptance fixture
at the 2,048-position boundary, with Hamming reads, turn memory and point-key
behavior active, and prove exact CPU-versus-integer decisions plus deterministic
replay.

The existing `cpu-served-forward-audit` binary remains the reusable host check
for Claude's first long-window artifact. This unit creates the matching
full-path fixture and boundary regression so that artifact is not the first
2,048-position CPU test.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-121/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/harness/claims.json`

## Done when

1. A finite fixture exports at context 2,048 with Hamming copy/type reads,
   opening/reply memory and a point-key record.
2. A 2,048-token audit checks every Q12 coordinate, token, kept cell and
   decision hash against integer serving with zero mismatches.
3. The sequence reaches long-range Hamming candidates and exercises memory and
   point fallback behavior; a control proves the named records are present.
4. Exactly 2,048 positions are accepted and position 2,049 is refused as
   context exhaustion.
5. A second audit is byte-identical without timing.
6. Focused/full CPU release tests, strict Clippy, formatting, report, roadmap
   and loop gates pass.
7. Fresh exact-head review, protected merge and fresh-main verification pass.

## Stop

Stop on CPU/serving divergence, nondeterministic output, more than 2 GiB peak
RSS, a projected focused test above 60 seconds, any edit to Claude-owned files,
or any model training/evaluation outside D84.
