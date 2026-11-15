# ORDER: m4-export-uorlut01-cli, codex-4

Lab: codex. Item: [m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md).
Parent: [codex-3](../codex-3/ORDER.md). Reviewed head: `5f440006f5e1a88fdffd0d32184d75a061ce0889`.

## Why this unit exists

codex-3 closed the host loop: the Rust training oracle updates the 48-byte state, the `/12`
artifact carries it, and a fresh `uor-chat` process serves it. This unit closes the **GPU** side
of the same loop: the AOT CUDA kernel trains the state on the RTX 4090, writes the exact 48
bytes, and a host check proves those bytes are the Rust oracle's state and serves them through
the artifact. It is finite (no language claim) and uses the kernel that already passed raw-word
parity.

## Claim record

`loop claim m4-export-uorlut01-cli --lab-topic q31-gpu-loop` was refused with exit 2:

```text
scope crates/uor-r4-integer/** overlaps crates/uor-r4-integer/src/discrete_d11.rs of item
one-model-end-to-end (PR #3991 (docs/loop/tasks/one-model-end-to-end/claude-387/ORDER.md))
```

This unit continues under the owner's open D85 direction, adds a small example output and a new
ignored host test, and does not touch `crates/uor-r4-integer/src/discrete_d11.rs`.

## Pre-run review (D67)

- **Chain:** the AOT CUDA trainer (`native_vector.cu` via `NativeVectorEngine::train_model_step`)
  is the native training operator; its state is the same 48-byte state the serving container
  carries. A pass proves the trained GPU state is the served state.
- **Tools and inputs:** the CUDA toolkit at `/usr/local/cuda/bin/nvcc`, the existing
  `native_vector_qualify` example, the 4,096-record parity fixture, and the now-idle local RTX
  4090. No pod lease and no cost.
- **Pass/fail:** a pass confirms the GPU-trained state equals the Rust oracle's bit for bit and
  that the `/12` artifact serves it. A fail is a mechanism finding on the GPU update, not an
  artifact result.
- **Sheet:** none required. This is a finite kernel/serving test, not a training or evaluation run
  a milestone claim rests on (CHARTER §4.5). The non-author FIT is not required for this class.
- **Risk:** the CUDA build or driver can refuse; that is an instrument result and is recorded.

## Native state schema

Unchanged: four little-endian Q31 gate words, epoch, step shift, version, padding. The CUDA model
update changes the gate and epoch in place; the step shift is a fixed hyperparameter.

## Shared operators and implementations

- AOT CUDA: `crates/uor-r4-discrete/src/native_vector.cu` through `NativeVectorEngine`.
- Rust oracle: `uor_r4_native_vector::Q31HamiltonModel::step_reference`.
- Serving: `q31_hamilton_rotate` / `Q31ServingSession::transition`; container `/12`.

## Transition replay

The GPU example trains 64 shared model steps on the fixed batch (values `[1 << 31, 0, 0, 0]`,
target `[1_518_500_249, 1_518_500_249, 0, 0]`), writes the 48 bytes and prints the gate, epoch,
loss and state digest. The ignored host test then:

1. reads the GPU state file and requires it to equal the Rust oracle's 64-step state bit for bit;
2. exports it into the `/12` artifact and requires the parser to expose exactly it; and
3. serves a prompt through a fresh `uor-chat` process and the in-process model, requiring
   identical tokens and session bytes, while the same trunk without the state serves others.

## Files in scope

- `crates/uor-r4-discrete/examples/native_vector_qualify.rs`
- `crates/uor-r4-integer/tests/q31_gpu_state_to_serving.rs`
- `docs/research/notebook/m4-serving.md`
- `docs/loop/tasks/m4-export-uorlut01-cli/codex-4/**`

Out of scope: `crates/uor-r4-integer/src/discrete_d11.rs`, the CUDA kernel source, any training
corpus or language result, any other lab's files.

## Size

About 150 to 250 lines of example, test and records.

## Done when

1. The CUDA example trains on the RTX 4090 and writes the exact 48-byte state with the gate,
   epoch and loss recorded.
2. The host test passes with that state file and proves bit equality with the Rust oracle.
3. The `/12` artifact exposes the GPU state, and a fresh CLI process and the in-process model
   serve identical tokens and session bytes; the bare artifact differs.
4. The existing focused Q31 tests still pass, and formatting, touched-file Clippy,
   `git diff --check`, `uor-harness check --only harness.native_space_contract`, the REPORT check
   and the exact-head review pass.

## Cancel

Stop if the CUDA kernel cannot train or write the state, if the GPU state differs from the Rust
oracle (record it as a mechanism finding), or if the artifact or serving needs a conversion or a
side-load. Do not start a language training run.

STOP and report when done.
