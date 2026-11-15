# ORDER: native-space-state-contract, codex-15

Lab: codex. Item: [native-space-state-contract](../../../../roadmap/items/native-space-state-contract.md).
Parent: [codex-14](../codex-14/ORDER.md). Reviewed head: `44e2a6f062e9196fe68291259bff459dde0bde44`.

## Why this unit exists

codex-14 taught the container to carry the per-lane state under `uor-r4.lut-stack/13`.
This slice, the third of the codex-12 decision, makes the real serving recurrence use it:
`IntegerStackModel::parse` attaches a `/13` state, and each lane advances through its own
gate instead of the one shared gate. `/12` stays exactly as it is.

## Claim record

`loop claim native-space-state-contract --lab-topic q31-lanes-serving` was refused with exit 2
because `Cargo.lock` in this item's scope overlaps claude-390's PR #4006. This slice adds no
dependency and does not touch `Cargo.lock`; it continues under the owner's D85 direction.

## Design

- New `Q31LaneServingSession` in `q31_serving.rs`, beside `Q31ServingSession`: restore the exact
  `32K + 16` bytes, expose `lanes`, `gate`, the shared `epoch`/`step_shift`, and
  `transition(lane, value)` (the shipped `Q31LaneGates` operator; no new arithmetic).
- `IntegerStackModel` gets `native_q31_lanes: Option<Q31LaneServingSession>`. `parse` attaches the
  artifact's lane state when the container carries one; each attach refuses if the model already
  holds the other state, if there is no recurrence layer, or if `K != shape.width / 4`.
- The serving digest binds the tag `uor-r4.native-q31-lanes-serving/1`, the model digest, `K` and
  the exact state bytes.
- `stack_native_q31_lanes` selects the transition by lane when the lane state is attached; the
  existing single-gate path is unchanged. The carry trace keeps one `NativeQ31Carry` per lane.
- The views that cannot execute the transport (`fbs_image`, `fbs_trunk_image`), the snap-trace
  refusal and the carry-domain preflight all treat the lane state exactly as the single state.

## Files in scope

- `crates/uor-r4-integer/src/stack/q31_serving.rs`
- `crates/uor-r4-integer/src/stack/session.rs` (and its test module)
- `crates/uor-r4-integer/src/stack/mod.rs` if a re-export is needed
- `docs/loop/tasks/native-space-state-contract/codex-15/**`

Out of scope: the `uor-r4-lut` facade (a separate slice; a `/13` artifact is not readable through
it yet), `Cargo.lock`, any CUDA, GPU or model-quality run.

## Done when

1. A `/13` artifact parses into a model that serves through the per-lane gates, and each traced
   lane's transport equals `q31_hamilton_rotate(gate[lane], before)`.
2. A `/13` state whose `K` gates all equal the one `/12` gate serves the same tokens and carry
   words as the `/12` path over the same steps (bit-identical replay). `K = 1` cannot occur: the
   shape contract requires `width` to be a multiple of 32, so `K = width / 4` is a multiple of 8.
3. Changing one lane's gate changes the served tokens or carries (a planted fault), and a wrong
   `K` is refused at attach.
4. Attaching both states, or a lane state on a model without a recurrence layer, is refused.
5. A `/12` artifact and a plain artifact still serve exactly as before.
6. Focused and library tests pass; formatting, touched-file Clippy, `git diff --check`,
   `uor-harness check --only harness.native_space_contract`, the REPORT check and the exact-head
   review pass.

## Cancel

Stop if the per-lane path needs new arithmetic in the model crate, a change to `/12`, a dependency,
or a kernel/GPU change. Do not start the facade or the training/objective work here.

STOP and report when done.
