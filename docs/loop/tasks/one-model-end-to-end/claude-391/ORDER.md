# ORDER: one-model-end-to-end, claude-391 (a native update rule for the Q31 read's gates)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `62853d976`, which holds claude-390 (#4006).

## Why this unit

claude-389 built the read and claude-390 serves it. Neither can train it: D85's training half is missing (claude-390 Still open 1–2). Casey answered "Start claude-391?" with "Yes, start it" (15 Nov 2026).

## What is built

`Q31FirstPickRead::train_pick_step`, in `crates/uor-r4-native-vector/src/read_train.rs`. It takes one batch, a target pick per row (`s + 1`, or `0` for none), a margin `m` and a step shift.

- **Loss.** For each row with a target: `max(0, d(q, k*) − d(q, k_r) + m)`. Here `d` is the read's own L1 address distance, `k*` is the target row's key and `k_r` is the nearest other earlier key. There is no softmax and no squashing.
- **Credit.** The L1 subgradient on each address word is a sign. Each gate's credit is that sign residual, at `2^30` per unit, taken through `q31_hamilton_rotate` with the conjugate block. This is the rule of `q31_hamilton_train_step`. Credits sum exactly in `i128`. Each gate becomes `sat(gate − (credit >> step_shift))`.
- **Accept.** The candidate is kept only if the batch's total loss strictly drops. Otherwise the read is unchanged.

**Decisions.**
- The gain stays fixed and is not trained. claude-389 Still open 2 left this choice to this unit.
- No operator change for the quantum (claude-390 Still open 1). The credit scales with the block, about 2^18 per gate word on the served residual, which is above the about 2^13 a gate needs to move an address. A test checks this.
- The weights need a downstream error, and the stack has no native loss yet, so they are deferred and named.

## Native-space five items (SKILL)

- **State:** the read's query and key gates (trained), weights and gain (fixed here). These are the same Q31 words as `to_bytes`.
- **Operator:** the step calls the same `q31_read_address` and first-pick rule that `apply` and `step` use. The CUDA twin of the update is the next unit's work.
- **Replay:** after training, `apply` and `step` give back the picks the trainer recorded, raw `u32` against `u32`. A `to_bytes`/`from_bytes` round trip is identical too.
- **Forward, gradient, update:** all three stay in Q31 and `i64`/`i128` words, with no float anywhere.
- **Serialization:** unchanged; `to_bytes` holds the trained gates.

## Checks (no GPU, no sheet: builds and finite tests, §4.5)

1. Unit tests in `read_train.rs` cover:
   - a fixture whose wrong picks become right;
   - a rejected step leaving the read bit-identical;
   - a row with no target being ignored;
   - every refusal, matched by its message;
   - the replay.
2. Negative gates: removing the acceptance check, the conjugate, or the tie rule each fails a test.
3. `cargo test -p uor-r4-native-vector`, clippy on the changed lines, and `uor-harness loop pr-check --local`.

## Files in scope

- `crates/uor-r4-native-vector/src/read_train.rs` (new), plus `lib.rs` and `read.rs` (visibility only).
- This folder, and one row in `docs/research/notebook/cpu-training.md`.

## Stop list

- No GPU run and no training run.
- No change to `apply`, `step`, the artifact or the session.

## Done when

The checks pass, a fresh review approves the exact head, and the PR merges.
