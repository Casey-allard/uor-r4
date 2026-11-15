# REPORT: m4-export-uorlut01-cli, codex-3

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim m4-export-uorlut01-cli --lab-topic q31-train-serve` was refused with exit 2 because
  PR #3991 claims overlapping `crates/uor-r4-integer/src/discrete_d11.rs` scope. The work
  continued under the owner's open D85 direction and did not touch that file.
- Clippy flagged the new test's first seed literal (`0x5eed_0f_31`) for unequal hex grouping; the
  literal is now `0x5eed_0f31` and the test is clean.
- This unit uses the **Rust training oracle**, not the AOT CUDA kernel. The CUDA twin's
  bit-parity was qualified earlier (16,384 transport words, 4,096 training records, 64 shared
  steps); rerunning it is a separate GPU unit, not claimed here.
- The training records are deterministic synthetic Q31 transitions, not a language corpus; the
  result proves the state contract and the product path, not learning.

## What changed

- A new integration test, `crates/uor-r4-integer/tests/q31_training_to_serving.rs`:
  1. trains the 48-byte state in place with `Q31HamiltonModel::step_reference` (16 batches of 64
     deterministic Q31 records) and requires the bytes, gate and epoch to move from the initial
     state;
  2. exports the trained bytes into a `uor-r4.lut-stack/12` artifact and requires the parsed model
     to expose exactly those bytes with the same gate, epoch and shift;
  3. serves the prompt in a fresh `uor-chat` process and in-process and requires identical tokens,
     last token and `UR4SESS1` digest; and
  4. serves the same trunk without the state and requires different tokens.
- One M4 notebook row records the result.
- No production code, artifact format, CLI behaviour or existing test changed.

Size: one new test file plus the loop and notebook records.

## Measurements

- The trained 48 bytes and gate differ from the initial `[1 << 31, 0, 0, 0]`; the trainer's epoch
  advances.
- The artifact carries exactly the trained bytes; the attached serving state exposes the same
  gate, epoch and step shift.
- Two fresh `uor-chat` processes and the in-process library run serve the same tokens and session
  digest on prompt `[3, 5, 8, 13]`, four generated tokens.
- The same trunk without the state serves different tokens.
- `1 / 1` training-to-serving control and `8 / 8` focused Q31 units pass.
- No language, capability, speed, memory, M4-bar or milestone result is measured.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --test
  q31_training_to_serving` — `1` passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib q31_serving`
  — `8` passed.
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm,fbs-synthetic --test
  q31_training_to_serving` — exit 0; no warning in the new test.
- `git diff --check` — passed.
- `uor-harness check --only harness.native_space_contract` — 1 rule, 0 failed.
- `uor-harness loop report --check`, `loop pr-check --local` and the exact-head review — at the head.

## Skills used

- `native-space-cycle`: training updates the one state; serialization is not translation; the
  served runtime reads the same words.
- `solution-integrity`: added the initial-vs-trained, artifact-bytes and bare-artifact legs so the
  test cannot pass by determinism alone.
- `scope-guard`: avoided the file claimed by PR #3991 and every out-of-scope path.
- `test-freeze`: added a new test; changed or deleted none.
- `working-discipline`: classified the Clippy grouping and the oracle-only scope as instrument and
  scope facts, not mechanism negatives.

## Still open

- The **GPU qualification** of the same loop through the AOT CUDA trainer, on the now-idle RTX
  4090, is a separate unit. It needs its own pre-run review and, if it decides a claim, a
  pre-registration with a non-author FIT.
- A **real trained model** exported and scored at the M4 bar is still open; this control uses
  synthetic records.
- The `uor-r4-lut` metadata facade still refuses `/12` by schema (safe).
- The org runner-capacity bottleneck remains an owner/PM-level issue.

## Plain summary

The gate is now trained, not copied. The trainer updates the 48-byte state, the artifact stores
exactly those bytes, and the chat program serves them in a new process. Turning the state off
changes the tokens, so the trained state is what the product runs. This is a finite training
control; the GPU rerun and a real model remain open.
