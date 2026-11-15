# ORDER: native-space-state-contract, codex-11

Lab: codex. Item: [native-space-state-contract](../../../../roadmap/items/native-space-state-contract.md).
Parent: [codex-8](../codex-8/ORDER.md). Reviewed head: `abfad0604`.

## Why this unit exists

PR #3977 merged the exact 48-byte native Q31 model and a serving-side adapter, but
`IntegerStackSession` does not consume that state. The real recurrence still derives every lane
transport from artifact tables, so loading the trained Q31 model cannot change generation.

This unit connects the loaded state to the actual recurrence. The model remains the model's
learned state: it is attached before sessions are created and is never reconstructed from a
different numeric representation.

## Claim record

`loop claim native-space-state-contract --lab-topic q31-session-integration` was attempted with
the harness built from this head and refused with exit 2:

```text
uor-harness: loop claim: refused: scope AGENTS.md overlaps AGENTS.md of item
infra-loop-harness (PR #3981 (docs/loop/tasks/infra-loop-harness/claude-383/ORDER.md))
```

The item also retains DeepSeek's open PR #3937 claiming overlapping `stack/**` scope. This unit
continues the owner's open Q31 direction without waiting, does not touch `stack/tests.rs`, and
does not alter the files named by PR #3937.

## Native state schema

- Token addresses remain the writer's `u32` token IDs.
- Vector coordinates remain the existing integer/Q31 values used by the trunk recurrence.
- The learned Q31 gate is exactly `[i64; 4]` in the shared `Q31HamiltonModel`.
- The optimizer cursor is the serialized `epoch: u64` and `step_shift: u32`.
- Persistent model bytes are exactly 48: four little-endian gate words, epoch, step shift,
  version `1`, and three zero bytes.
- A session step carries each recurrence lane's four Q31 words. The optional trace records those
  raw words without replacing the served state.

## Shared operators and implementations

- Training/reference transition: `uor_r4_native_vector::q31_hamilton_rotate`.
- CUDA implementations: the existing AOT entries in `uor-r4-discrete/src/native_vector.cu`,
  reached through `NativeVectorEngine`.
- Rust training step: `q31_hamilton_train_step` and `Q31HamiltonModel::step_reference`.
- Shared CUDA aggregation: `NativeVectorEngine::train_model_step`.
- Integer serving: `Q31ServingSession::transition`, which calls the same
  `q31_hamilton_rotate`; the actual `IntegerStackSession` recurrence will call that adapter.
- WASM: the integer stack is built from the same Rust source; no second operator is introduced.

## Transition replay

One fixed CUDA-trained state is loaded from its exact 48 bytes into two otherwise identical
synthetic FBS models, with the state attached to only one. A fixed prompt is stepped through both
sessions. On every recurrence lane in the attached session:

1. record the raw input carry;
2. call the shipped Q31 serving transition;
3. compare its raw output to `q31_hamilton_rotate` at the loaded gate;
4. record the raw post-drive carry;
5. compare emitted token IDs, kept cells and final query words.

The attached state must change at least one emitted token on the fixed prompt. If it does not,
the test cannot prove that the loaded state drives generation and fails.

## Training, gradient and optimizer state

This unit does not train a model. It consumes the merged CUDA-trained state whose gate, epoch and
step shift are already in the 48-byte representation. The training update and acceptance path are
not copied into serving. A finite RTX 4090 qualification must still reproduce the same raw gate
and epoch from the training operator before the serving fixture is accepted.

## Serialization

The Q31 model bytes are loaded without conversion and exposed losslessly for verification.
`UR4SESS1` continues to serialize the dynamic token-session state. A model with no attached Q31
state remains byte-identical to current serving; a model with one exposes the exact bytes and
refuses incompatible shapes.

## Goal

Make the merged Q31 state part of the real integer token path, prove raw-word transition and carry
parity, show that the state changes emitted tokens, and deliver the change through a protected PR.

## Files in scope

- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/q31_serving.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `docs/research/notebook/float-training.md`
- `docs/loop/tasks/native-space-state-contract/codex-11/**`

## Size

About 300 to 550 changed Rust and test lines, plus the required loop records.

## Done when

1. An `IntegerStackModel` can attach and expose the exact 48-byte Q31 state.
2. The real recurrence calls `Q31ServingSession::transition`; no diagnostic-only adapter can
   claim the result.
3. Requested traces expose raw pre-transport carry, raw Q31 transport and raw post-drive carry for
   every recurrence lane.
4. A fixed trained-state replay passes raw-word transition parity and changes at least one emitted
   token versus the same model without the attachment.
5. Models without an attached state follow the existing path unchanged.
6. Invalid Q31 bytes and models without a recurrence are refused before serving.
7. Focused CPU tests, the integer crate build with `serving-wasm`, CUDA raw-word qualification,
   formatting, touched-file Clippy, `git diff --check`, the native-space harness rule and REPORT
   checks pass.
8. The delivery is reviewed at one exact head, merged through the protected queue, and verified
   from fresh `main`.

## Cancel

Stop if the actual recurrence cannot call the shared typed Q31 transition without introducing a
second arithmetic implementation, if the loaded words require numeric conversion, or if no
fixed trained state changes token generation on a lawful in-context fixture. Do not substitute a
parallel diagnostic adapter for the served call path.

STOP and report when done. Do not start the next task.
