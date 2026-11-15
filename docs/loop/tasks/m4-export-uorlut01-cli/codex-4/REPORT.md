# REPORT: m4-export-uorlut01-cli, codex-4

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim m4-export-uorlut01-cli --lab-topic q31-gpu-loop` was refused with exit 2 because
  PR #3991 claims overlapping `crates/uor-r4-integer/src/discrete_d11.rs` scope. The work
  continued under the owner's open D85 direction and did not touch that file.
- The first host test kept the copied `#[test]` attribute and added another, so the same function
  appeared twice and the compiler warned about a duplicate macro attribute. It is now one ignored
  test; the default run reports `0 passed, 1 ignored`, and the explicit run reports `1 passed`.
- The CUDA build emits pre-existing unused-code warnings from `uor-r4-core`; they are not from
  this change.
- This is a finite kernel/serving control. The training records are the fixed 4,096-record
  parity fixture and the 64 shared model steps, not a language corpus; no language result is
  claimed.

## What changed

- `crates/uor-r4-discrete/examples/native_vector_qualify.rs` now writes the exact 48-byte state to
  an optional path and prints its hex and SHA-256 beside the gate, epoch, step shift and losses.
  The CUDA kernel source is unchanged.
- A new ignored host test, `crates/uor-r4-integer/tests/q31_gpu_state_to_serving.rs`, reads that
  file, requires byte equality with the Rust oracle's 64-step state, exports it into `/12`, and
  serves it from a fresh `uor-chat` process and the in-process model; the same trunk without the
  state is the control.
- One M4 notebook row records the GPU numbers and the parity.

Size: one example change, one new ignored test, and the loop and notebook records.

## Measurements

- RTX 4090, `NVCC=/usr/local/cuda/bin/nvcc UOR_NVCC_ARCH=compute_89`, AOT CUDA image:
  `16,384` raw transport words and `4,096` training records bit-identical; operator entries
  `native_q31_hamilton_rotate` / `native_q31_hamilton_train_step`.
- Shared model: gate `[2008113292, 336469910, 0, 0]`, epoch `64`, step shift `8`, aggregate loss
  `17,179,869,184 -> 13,373,147,056`.
- Exact trained state: 48 bytes, SHA-256
  `3419349b6e9b9fccb3c0587e1ebe5b519698416605c8be0187df5298034f46a3`.
- The GPU-written bytes equal the Rust oracle's 64-step state bit for bit; the `/12` artifact
  exposes exactly them; a fresh CLI process and the in-process model serve identical tokens and
  session digest; the bare artifact serves different tokens.
- `1 / 1` host control (explicit `--ignored`) and `8 / 8` focused Q31 units pass.
- No language, capability, speed, memory, M4-bar or milestone result is measured.

## Checks run at the head

- `NVCC=… UOR_NVCC_ARCH=compute_89 UOR_REQUIRE_CUDA=1 cargo run --locked -p uor-r4-discrete
  --features cuda --example native_vector_qualify -- <state-file>` — passed; JSON above.
- `UOR_Q31_STATE=<state-file> cargo test --locked -p uor-r4-integer --features
  serving-wasm,fbs-synthetic --test q31_gpu_state_to_serving -- --ignored` — `1` passed.
- The same test without `--ignored` — `0` passed, `1` ignored (not a silent pass).
- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib q31_serving`
  — `8` passed.
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm,fbs-synthetic --test
  q31_gpu_state_to_serving` — exit 0; no warning in the new test.
- `git diff --check` — passed.
- `uor-harness check --only harness.native_space_contract` — 1 rule, 0 failed.
- `uor-harness loop report --check`, `loop pr-check --local` and the exact-head review — at the head.

## Skills used

- `native-space-cycle`: the GPU trainer updates the one state; the artifact carries it; serving
  reads the same words, with raw-byte comparison.
- `uor-research-architect`: kept the GPU run finite and source-bound; no language or milestone
  claim.
- `solution-integrity`: added the oracle-equality and bare-artifact legs so determinism alone
  cannot pass the control.
- `scope-guard`: avoided the file claimed by PR #3991 and every out-of-scope path.
- `test-freeze`: added an ignored test; changed or deleted none.
- `working-discipline`: recorded the duplicate-attribute repair and the pre-existing build
  warnings as instrument facts.

## Still open

- A **real trained language model** exported with the state, served through `uor-chat` and scored
  at the M4 bar remains open; this unit trains the finite parity fixture.
- The `uor-r4-lut` metadata facade still refuses `/12` by schema (safe).
- The org runner-capacity bottleneck remains an owner/PM-level issue.

## Plain summary

The graphics card trained the same tiny state the chat program serves. Its 48 bytes match the CPU
reference exactly, the artifact carries exactly those bytes, and a new chat process serves them.
Running the same file without the state changes the output. This is a finite GPU-to-serving
control, not a language result.
