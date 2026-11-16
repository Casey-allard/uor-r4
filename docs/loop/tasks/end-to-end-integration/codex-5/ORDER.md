# ORDER: end-to-end-integration, codex-5

Lab: codex. Item: [end-to-end-integration](../../../../roadmap/items/end-to-end-integration.md).
Parent: [codex-4](../codex-4/ORDER.md). Reviewed head: `79551bf0e`.

## Why this unit exists

The claude note on #2028 (point 3) asks codex to move the lane objective off
`synthetic_fbs_trunk` and onto real states. The c387 states and the D41 pilot are not local to this
checkout (the HF store is not materialised, and Claude is running its read training on wick). The
next-best real source is committed: `docs/labs/cpu-training-pilot-content-serving-2026-11-04/
evidence/seed-1/serving.uorlut`, a trained `/7` FBS trunk (width 256, six recurrence layers, real
tables, turn memory).

So this unit reruns codex-17's shared-versus-per-lane measurement on **real trained weights and
real recurrence carries**, with a fixed id sequence because no tokenizer is bound to that `/7`
artifact. It is not the c387 states and not real text; it is a real model.

## Claim record

`loop claim end-to-end-integration --lab-topic q31-real-states` was refused because the item is
still claimed by this lab's codex-3 worktree (PR #4015, queued). The claim is this lab's own; the
unit adds one test file and docs, no dependency, no `Cargo.lock` change.

## Method

- Parse the committed artifact; assert shape width 256 / six recurrence layers / K = 64.
- Attach an identity `/13` lane state so the native Q31 path runs and the carries can be traced.
- Step a fixed 32-id sequence (deterministic LCG in the artifact's 4,096 vocabulary; **artificial
  ids**, stated in the REPORT), tracing the 6 x 64 carries per step and the `query_q12` of each
  step.
- Records: for each transition, each lane's carry `before` is the value; the next step's query
  block `query_q12[4l..4l+4] << 14` is the target at Q26 (the queries reach 2^15, so a 16-bit lift
  would sit on the Q31 boundary; 14 bits keep four times the headroom).
- Train the shared gate on the pooled records and one gate per lane on its own records, 64 shipped
  `step_reference` steps each; no new arithmetic.
- Report the update's own L1 and the served tokens for the shared and the per-lane state, on the
  same ids.

## Files in scope

- `crates/uor-r4-integer/tests/q31_lane_objective_real.rs` (new file)
- `docs/loop/tasks/end-to-end-integration/codex-5/**`

Out of scope: source, schema, the artifact, the c387 data, a tokenizer, serving wiring, any GPU run,
and any capability or model-quality claim.

## Done when

1. The real artifact parses, attaches both states, and serves the fixed ids.
2. Both configurations train and the L1 falls (or the result is reported if it does not).
3. The served tokens and the per-lane breakdown are printed; the sign of every quantity is in the
   REPORT, negative first.
4. The test passes; `git diff --check`, the harness contract rule, the REPORT check and the
   exact-head review pass.

## Cancel

Stop if the artifact cannot attach a Q31 state, if the query target is unavailable, or if the run
needs a source change. Do not tune the id sequence after seeing the result.

STOP and report when done.
