# ORDER: one-model-end-to-end, deepseek-270 (reply-control bake-off)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `ef3944884`.

## Goal

Run a frozen controller bake-off over the real arm-r seed-2 reply-control
trace. The controller may only keep the served token or emit EOS at the current
online position; it may not rewrite a token or inspect the reference.

Compare:

1. current greedy/cycle;
2. fixed EOS rank threshold;
3. stability/tension threshold;
4. permeability-aware threshold;
5. shuffled-feature control;
6. seeded random control.

The primary metric is exact complete reference replies. Integration is allowed
only if a controller beats current greedy/cycle and the strongest fixed rule
without truncating more correct answers.

## Files in scope

- `bake-off/reply-control*`
- `docs/labs/one-model-end-to-end/instrument/src/lib.rs`
- `docs/labs/one-model-end-to-end/instrument/src/reply_control_controller.rs`
- `docs/labs/one-model-end-to-end/instrument/src/bin/reply-control-bakeoff.rs`
- `docs/labs/one-model-end-to-end/instrument/tests/reply_control_bakeoff_rules.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-270/**`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. Socket and rule tests are committed before any candidate.
2. The dumb control and its TOO WEAK rows are committed before candidates.
3. Each candidate has a kill line committed before its code and a first-run
   time after it.
4. Every candidate is scored on the same real trace and reference file.
5. The record passes the bake-off checker and prints one scoreboard.
6. The exact-complete metric, truncation rate and EOS-position error are
   reported for every arm.
7. No serving path consumes a candidate; any pick remains an owner decision.

## Stop and report

Stop if the trace/reference identities do not match, any candidate reads
reference ids while deciding, or the real trace is missing.
