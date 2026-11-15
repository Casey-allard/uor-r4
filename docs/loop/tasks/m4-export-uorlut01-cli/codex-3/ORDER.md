# ORDER: m4-export-uorlut01-cli, codex-3

Lab: codex. Item: [m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md).
Parent: [codex-2](../codex-2/ORDER.md). Reviewed head: `9747a3177814ecc130a08899bbfc01c495d5a13a`.

## Why this unit exists

codex-1 proved the artifact carries the exact 48-byte Q31 state; codex-2 proved a fresh
`uor-chat` process serves it. Both start from a fixed state. This unit closes the loop the other
way: **train** the state with the shared Rust training oracle (the parity reference for the CUDA
kernel), serialize exactly those trained bytes into the artifact, and serve them. It is the
host-side training-to-serving control that must exist before a GPU qualification run.

## Claim record

`loop claim m4-export-uorlut01-cli --lab-topic q31-train-serve` was refused with exit 2:

```text
scope crates/uor-r4-integer/** overlaps crates/uor-r4-integer/src/discrete_d11.rs of item
one-model-end-to-end (PR #3991 (docs/loop/tasks/one-model-end-to-end/claude-387/ORDER.md))
```

This unit continues under the owner's open D85 direction, adds only a new integration test and
records, and does not touch `crates/uor-r4-integer/src/discrete_d11.rs`.

## Native state schema

The 48 bytes: four little-endian Q31 gate words, `epoch: u64`, `step_shift: u32`, version `1`,
three zero bytes. Training updates the gate and epoch in place; the step shift is a fixed
hyperparameter carried unchanged. Serialization is a
byte copy. No float master and no parallel state.

## Shared operators and implementations

- Training oracle: `uor_r4_native_vector::Q31HamiltonModel::step_reference`; its AOT CUDA twin is
  `native_vector.cu` through `NativeVectorEngine` and was qualified bit-identical to this oracle
  (16,384 transport words, 4,096 training records, 64 shared steps). This unit uses the oracle on
  the host; the GPU run is separate.
- Serving: `q31_hamilton_rotate` through `Q31ServingSession::transition` in the integer session.
- Container: `uor-r4.lut-stack/12`, table `q31.state`, digest-bound header record.

## Transition replay

One deterministic training run (fixed records, fixed steps) produces one 48-byte state. The test:

1. asserts the trained gate, epoch and bytes differ from the initial state;
2. exports the trained state into the `/12` artifact and requires the parsed model to expose
   exactly those 48 bytes and that gate;
3. serves the prompt in a fresh `uor-chat` process and in-process, requiring identical token ids
   and session bytes; and
4. serves the same trunk without the state and requires different token ids, so the trained state
   is what drives the product path.

The comparison unit is raw bytes and token ids, never rounded values.

## Files in scope

- `crates/uor-r4-integer/tests/q31_training_to_serving.rs`
- `docs/research/notebook/m4-serving.md`
- `docs/loop/tasks/m4-export-uorlut01-cli/codex-3/**`

Out of scope: `crates/uor-r4-integer/src/discrete_d11.rs`, any CUDA kernel or GPU run, any
training corpus or language claim, any production or format change.

## Size

About 200 to 300 lines of test and records.

## Done when

1. The new integration test passes with `--features serving-wasm,fbs-synthetic`.
2. The trained state differs from the initial state and is the exact state the artifact carries.
3. A fresh `uor-chat` process and the in-process model serve identical tokens and session bytes.
4. The bare artifact serves different tokens.
5. Existing focused Q31 tests still pass, and formatting, touched-file Clippy, `git diff --check`,
   `uor-harness check --only harness.native_space_contract`, the REPORT check and the exact-head
   review pass.

## Cancel

Stop if training cannot update the 48-byte state in place, if the artifact or serving needs a
conversion or a side-load, or if the trained state does not drive the served tokens. Do not add a
second training path or a float master.

STOP and report when done. Do not start a GPU run or the next unit.
