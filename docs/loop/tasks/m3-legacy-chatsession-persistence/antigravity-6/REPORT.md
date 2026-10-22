# REPORT: m3-legacy-chatsession-persistence, antigravity-6

Lab: antigravity. Order: [ORDER.md](ORDER.md).

Closes #2854. References #2031.

## What did not work, first

- Running `~/.cargo/bin/cargo test -p uor-r4-integer --test capability_api_tests` without `--features serving-wasm` failed with `error[E0432]: unresolved import uor_r4_integer::capability_api` because `crates/uor-r4-integer/src/native_geometry.rs` includes `serving_root.rs` (`capability_api`, `session`, `session_codec`, `stack`) only when `--features serving-wasm` is enabled.
- Negative gate (reverting `CapabilityTruthMatrix::default().durable_memory` in `crates/uor-r4-integer/src/capability_api.rs` back to the retired `"That schema omits the width-576 value stores"` wording): `~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --test capability_api_tests` failed at `crates/uor-r4-integer/tests/capability_api_tests.rs:95:5` (`assertion failed: meta.truth_matrix.durable_memory.contains("uor-r4.stack-session/1")`), then passed when the updated `durable_memory` entry was restored.

## What changed

- Retired the ignored width-576 continuation test `restored_dialogue576_session_continues_identically` in `crates/uor-r4-integer/tests/capability_api_tests.rs` in favour of the `IntegerStackSession` reload gate (`session_save_and_restore_produces_bit_identical_logits` and `session_pause_save_fresh_restore_greedy_continuation` in `crates/uor-r4-integer/src/stack/tests.rs`, folding `#1476`), updated the file header in `crates/uor-r4-integer/tests/capability_api_tests.rs`, and added assertions in `metadata_declares_the_contract_without_certifying_it` verifying that `truth_matrix.durable_memory` names both `uor-r4.integer-session/1` and `uor-r4.stack-session/1` and no longer contains `"omits the width-576 value stores"`.
- Updated `CapabilityTruthMatrix::default().durable_memory` and the `IntegerCapabilityApi::restore_session` doc comment in `crates/uor-r4-integer/src/capability_api.rs` to state that width-256 session save/restore uses `uor-r4.integer-session/1` and exact stack-session reload uses `uor-r4.stack-session/1` (`IntegerStackSession`), while preserving `"Durable-memory capability is not established."`.
- Added `docs/labs/m3-legacy-chatsession-persistence/README.md` with the design note and the in-scope `m3-durable-memory` notebook row, and updated `docs/roadmap/items/m3-legacy-chatsession-persistence.md` (`status: done`) and `docs/roadmap/README.md`.

## Measurements

- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --test capability_api_tests` on macOS aarch64: 3 passed, 0 failed, 1 ignored (`capability_api_lifecycle_on_dialogue576_bundle`) in 0.26s.
- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --lib stack::tests::session_` on macOS aarch64: 7 passed, 0 failed (`session_save_and_restore_produces_bit_identical_logits` and `session_pause_save_fresh_restore_greedy_continuation` included) in 0.11s.
- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-integer --lib` (default native qualification feature set) on macOS aarch64: 42 passed, 0 failed in 0.01s.

## Checks run at the head

- `~/.cargo/bin/cargo fmt -p uor-r4-integer -- --check`: exit 0.
- `~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --test capability_api_tests`: 3 passed, 0 failed, 1 ignored.
- `~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --lib stack::tests::session_`: 7 passed, 0 failed.
- `~/.cargo/bin/cargo test -p uor-r4-integer --lib`: 42 passed, 0 failed.
- `~/.cargo/bin/cargo run -q --locked -p xtask -- roadmap render --check`: exit 0.
- `uor-harness loop gates --full`: `fmt-claim-wording` GREEN, `fmt-harness` GREEN, `claim-wording` GREEN, `loop-gates-watch` GREEN, `git-hooks` GREEN.
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: PASS.

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`).

## Still open

- None for `#2854`; `m3-legacy-chatsession-persistence` is complete (`status: done`). Whether to mirror the `m3-durable-memory` notebook row from `docs/labs/m3-legacy-chatsession-persistence/README.md` into `docs/research/notebook/m3-durable-memory.md` is queued on the PM decision card in `#2028`.

## Plain summary

- The integer serving crate (`crates/uor-r4-integer`) has two session persistence formats: `uor-r4.integer-session/1` for legacy `ChatSession` state, and `uor-r4.stack-session/1` (`crates/uor-r4-integer/src/session_codec.rs`) for `IntegerStackSession`.
- Previously, `crates/uor-r4-integer/tests/capability_api_tests.rs` kept an ignored failing test (`restored_dialogue576_session_continues_identically`, `#1476`) and `crates/uor-r4-integer/src/capability_api.rs` carried a caveat that `uor-r4.integer-session/1` omits the width-576 value stores (`persistent_values_576`, `dialogue_values_576`, `l2_pages_576`).
- Because no shipped binary or serving path builds a width-576 `ChatSession` (`uor-chat` uses `DialogueConversation` or `IntegerStackModel`/`StackChat`, and `wasm_bindgen_exports.rs` uses `IntegerStackSession`), and exact stack-session reload is already tested on every run by `session_save_and_restore_produces_bit_identical_logits` and `session_pause_save_fresh_restore_greedy_continuation` in `crates/uor-r4-integer/src/stack/tests.rs`, this change retires the ignored width-576 continuation test, updates the `capability_api.rs` and `capability_api_tests.rs` wording, and pins the updated `durable_memory` truth-matrix string in `capability_api_tests.rs`.
