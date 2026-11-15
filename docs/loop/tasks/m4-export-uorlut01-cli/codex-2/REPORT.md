# REPORT: m4-export-uorlut01-cli, codex-2

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim m4-export-uorlut01-cli --lab-topic q31-cli-fresh` was refused with exit 2 because
  PR #3989 claims overlapping `crates/uor-r4-integer/src/discrete_d11.rs` scope. The work
  continued under the owner's open D85 direction and did not touch that file.
- The first tokenizer fixture had 40 pieces while the synthetic FBS artifact has vocabulary 256.
  `uor-chat` correctly refused it (`tokenizer vocabulary 40 differs from the artifact vocabulary
  256`); the test now builds a 256-piece tokenizer that matches the artifact.
- The first generation length (8) exceeded the synthetic trunk's 8-position context and the CLI
  correctly refused the full session. The control now generates four tokens after the four-token
  prompt, staying inside the context.
- The control needs the `fbs-synthetic` feature for the shared synthetic FBS fixture, so it is
  compiled under `serving-wasm,fbs-synthetic`; the all-features CI job enables both.

## What changed

- A new integration test, `crates/uor-r4-integer/tests/q31_artifact_fresh_process.rs`, exports the
  codex-1 `/12` artifact and its plain counterpart, writes a matching 256-piece tokenizer, and:
  1. runs `uor-chat --stack` on the `/12` artifact in two separate processes and requires
     identical `generated_ids`, `last_token`, `session_sha256` and `UR4SESS1` bytes;
  2. runs the same sequence in-process on `IntegerStackModel::parse` and requires the same tokens
     and session digest as the CLI; and
  3. runs the same CLI on the artifact without the state and requires different generated ids.
- No production code, artifact format, CLI behaviour or existing test changed.
- One M4 notebook row records the result.

Size: one new test file plus the loop and notebook records.

## Measurements

- Two fresh `uor-chat` processes on the `/12` artifact: identical generated ids, last token,
  session SHA-256 and saved session bytes.
- The CLI and the in-process library run on the same artifact: identical tokens and session bytes.
- The same CLI on the bare artifact: different generated ids, so the artifact-carried Q31 state is
  what drives generation.
- `1 / 1` fresh-process control passes (prompt `[3, 5, 8, 13]`, four generated tokens, synthetic
  trunk seed `20_261_115`). `8 / 8` focused Q31 unit tests still pass.
- No language, capability, speed, memory, M4-bar or milestone result is measured.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --test
  q31_artifact_fresh_process` — `1` passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib q31_serving`
  — `8` passed.
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm,fbs-synthetic --test
  q31_artifact_fresh_process` — exit 0; no warning in the new test.
- `git diff --check` — passed.
- `uor-harness check --only harness.native_space_contract` — 1 rule, 0 failed.
- `uor-harness loop report --check`, `loop pr-check --local` and the exact-head review — at the head.

## Skills used

- `native-space-cycle`: the product path serves the artifact-carried state through the shared
  operator, with the comparison on tokens and raw session bytes.
- `solution-integrity`: checked that the fresh-process equality alone could hide a missing attach,
  and added the bare-artifact and library legs so the state's effect is measured.
- `scope-guard`: avoided the file claimed by PR #3989 and every out-of-scope path.
- `test-freeze`: added a new test; changed or deleted none.
- `working-discipline`: recorded the tokenizer/vocab and context refusals as fixture repairs, not
  mechanism negatives.

## Still open

- A **real trained model** exported with the state, served through `uor-chat` in a fresh process
  and scored at the M4 bar is still open. This control uses a synthetic finite model.
- The GPU is currently held by a 16-layer `train_v11_gpu` run that is not this lab's; no training
  or GPU work is part of this unit.
- The org runner-queue bottleneck remains an owner/PM-level capacity issue.

## Plain summary

Before, the file could carry the trained state and the library could attach it. This test proves
the user-facing chat program does the same: start `uor-chat` on the file, let it run, and a second
fresh start produces exactly the same tokens and saved session. Running the same file without the
state gives different tokens, so the state in the file is what changes the output. This is a
finite serving control, not a model-quality result.
