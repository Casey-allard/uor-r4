# ORDER: m4-export-uorlut01-cli, codex-2

Lab: codex. Item: [m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md).
Parent: [codex-1](../codex-1/ORDER.md). Reviewed head: `c8d5f1feecf77777d64082801a5b52eb48a6b83d`.

## Why this unit exists

codex-1 proved in-process that `UORLUT01` carries the exact 48-byte Q31 state and that
`IntegerStackModel::parse` attaches it. It did not prove that the **product CLI** loads the state
from the artifact in a **fresh process** and serves the same tokens. The M4 item's step 3 asks for
exactly that finite control: a toy artifact, `uor-chat` on it in two fresh processes, identical
token output. This unit closes the serving half of the one-space path without a GPU.

## Claim record

`loop claim m4-export-uorlut01-cli --lab-topic q31-cli-fresh` was refused with exit 2:

```text
scope crates/uor-r4-integer/** overlaps crates/uor-r4-integer/src/discrete_d11.rs of item
one-model-end-to-end (PR #3989 (docs/loop/tasks/one-model-end-to-end/claude-386/ORDER.md))
```

This unit continues under the owner's open D85 direction, adds only a new integration test and
records, and does not touch `crates/uor-r4-integer/src/discrete_d11.rs`.

## Native state schema

Unchanged from codex-1: the `/12` section holds the exact 48 bytes in table `q31.state` with the
digest-bound header record. This unit adds no format or arithmetic change; it exercises the
shipped load path.

## Shared operators and implementations

- `IntegerStackModel::load` parses the artifact and attaches the state through
  `Q31ServingSession` / `q31_hamilton_rotate`, the same operator training uses.
- `uor-chat --stack` serves `--prompt-ids` through `IntegerStackSession::step` and `serve_step`,
  which route through the same recurrence core.
- No second path, no side-load, no float, no matmul.

## Transition replay

One synthetic FBS trunk (seed `20_261_115`, the codex-1 fixture) is exported twice: with the
trained 48-byte state (`/12`) and without it. The test then:

1. runs `uor-chat --stack q31.lut --tokenizer tokenizer.json --prompt-ids 3,5,8,13
   --max-tokens 8` in two separate processes and requires identical `generated_ids`,
   `last_token` and `session_sha256` (and the saved `UR4SESS1` bytes);
2. runs the same sequence in-process on `IntegerStackModel::parse(&q31_artifact)` and requires
   the same tokens and session digest as the CLI; and
3. runs the CLI on the bare artifact and requires different generated ids, so the test proves the
   artifact-carried state drives the process rather than merely repeating deterministically.

The comparison unit is token ids and raw session bytes, never rounded values.

## Files in scope

- `crates/uor-r4-integer/tests/q31_artifact_fresh_process.rs`
- `docs/research/notebook/m4-serving.md`
- `docs/loop/tasks/m4-export-uorlut01-cli/codex-2/**`

Out of scope: `crates/uor-r4-integer/src/discrete_d11.rs`, production CLI or format changes, any
CUDA kernel, any training run.

## Size

About 150 to 250 lines of test and records.

## Done when

1. The new integration test passes with `--features serving-wasm,fbs-synthetic`.
2. Two fresh `uor-chat` processes on the `/12` artifact produce identical tokens and session bytes.
3. The CLI output equals the in-process library run on the same artifact.
4. The same CLI on the bare artifact produces different tokens, proving the state is served.
5. `cargo test --features serving-wasm q31_serving` still passes, and formatting, touched-file
   Clippy, `git diff --check`, `uor-harness check --only harness.native_space_contract`, the
   REPORT check and the exact-head review pass.

## Cancel

Stop if the CLI cannot load a `/12` artifact in a fresh process without a production change, if
the state does not change the CLI output, or if parity requires changing an existing artifact or
test. Do not add a side-load or a second serving path.

STOP and report when done. Do not start a training run or the next unit.
