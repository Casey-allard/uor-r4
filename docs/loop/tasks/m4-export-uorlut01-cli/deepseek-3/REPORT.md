# REPORT: m4-export-uorlut01-cli, deepseek-3 (default the trained protocol)

Lab: deepseek. Item: [m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md) (#2859). Base: `7394c344d`. ORDER: [ORDER.md](ORDER.md).

## What did not work

- **The old default was the wrong instrument for the trained path.** Protocol 1 leaves the role marker's space as a separate byte token; protocol 2 carries that space into the next word, which is how the pinned training corpus is encoded. The model did not change between claude-259's bad and repaired conversations.
- **No model-quality result is claimed.** Both protocols still produce malformed replies from the public dual-c131 artifact, which is not a chat model. The change makes the ordinary CLI use the trained representation; it does not make those artifacts conversational.
- **No new protocol is introduced.** Protocol 1 remains available because old artifacts and their tests depend on it.
- **The unconfigured full integration command needs external fixtures.** `cargo test --locked -p uor-r4-integer --features serving-wasm` ran the 443-test library and bin targets successfully, then six inherited tests in `adversarial_challenger_m1_token_boundary` failed because their bundle and tokenizer paths are not present on this workstation. None is in this unit's code path.
- **Workspace-wide formatting is not clean at the base.** `cargo fmt --all -- --check` reports widespread pre-existing drift in unrelated files. `rustfmt --check --edition 2021 crates/uor-r4-integer/src/bin/uor-chat.rs` passes for the changed Rust file.

## What changed

`uor-chat --stack` now defaults `DialogueProtocol` version 2. The help text, `CliArgs::default()` and crate README say so. Explicit `--protocol 1` still selects version 1. No artifact bytes, model arithmetic, tokenizer format or state codec changed.

## Measurements

- `cargo test --locked -p uor-r4-integer --features serving-wasm --bin uor-chat`: **6 passed, 0 failed**. The new control is `stack_protocol_defaults_to_the_trained_corpus_encoding`.
- `cargo test --locked -p uor-r4-integer --features serving-wasm`: library **443 passed, 1 ignored**; bin tests passed; the command then stopped on six absent-fixture integration tests as above.
- `cargo build --locked -p uor-r4-integer --features serving-wasm --bin uor-chat`: passed.

## Checks run at the head

- `rustfmt --check --edition 2021 crates/uor-r4-integer/src/bin/uor-chat.rs`: passed.
- `uor-harness ratchet d23`, `d25` and `d32`: all unchanged; claim-wording gate passed.
- Public artifact `dual-c131-seed-1-raw_both.uorlut`, SHA-256 `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`, with its matched tokenizer:
  - no `--protocol`: JSON reported `uor-r4.literal-role-dialogue/2`, one generated id `268`, session SHA-256 `d022a9a6c3bf026ffb22431971c6d9b0a24f225d6e642bc7673377010f1e2740`;
  - explicit `--protocol 1`: JSON reported version 1, the same one-token generation for the raw-id probe;
  - `--say Hi --max-tokens 4`: default version 2 used 16 history tokens and reply ids `[563, 971, 456, 340]`; explicit version 1 used 17 history tokens and reply ids `[1044, 316, 77, 583]`. The extra protocol-1 history token is the documented separator-space difference. The reply text was malformed in both cases and is not scored.

## Skills used

No available Codex skill applied. The protocol implementations, training corpus encoding, CLI parser and existing serving records were read directly.

## Still open

- The full PR CI suite and exact-head review.
- Claude's copy-key run and the later batched scorer, which are separate work.
- Useful chat quality on a trained model. This unit only removes a default encoding defect from its serving path.

## Plain summary

`uor-chat --stack` now uses protocol 2 when no protocol is named, matching the pinned training corpus and avoiding the known protocol-1 separator-space error. Protocol 1 remains available explicitly. The focused CLI and integer checks pass, and the public model's malformed replies are not treated as a quality result.
