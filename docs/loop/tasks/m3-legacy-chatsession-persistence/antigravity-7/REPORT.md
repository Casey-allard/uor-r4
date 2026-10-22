# REPORT: m3-legacy-chatsession-persistence, antigravity-7

Lab: antigravity. Order: [ORDER.md](ORDER.md).

Closes #2854. References #2031.

## What did not work, first

- In `antigravity-6`, updating `CapabilityTruthMatrix::default().durable_memory` and `IntegerCapabilityApi::restore_session`'s doc comment without rejecting width-576 sessions in `save_session` / `restore_session` hid the fact that `ChatSession::from_serialized` (`crates/uor-r4-integer/src/session.rs:971, 1207-1230`) still reconstructs empty width-576 value stores, and citing `cargo test -p uor-r4-integer --lib` without `--features serving-wasm` did not compile `serving_root.rs`.
- Negative gate (disabling the width-576 rejection guards in `IntegerCapabilityApi::save_session` and `IntegerCapabilityApi::restore_session`):
  - `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --test capability_api_tests restore_session_rejects_width_576_serialized_state` failed as expected (`panicked at crates/uor-r4-integer/tests/capability_api_tests.rs:157:10: restore_session must reject width-576 serialized state: 2`).
  - `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --lib capability_api::tests::save_and_restore_session_reject_width_576_sessions` failed as expected (`panicked at crates/uor-r4-integer/src/capability_api.rs:438:14: restore_session must reject width-576 state: 2`).
  - Both tests passed once the width-576 guards were restored.

## What changed

- Updated `IntegerCapabilityApi::save_session` and `IntegerCapabilityApi::restore_session` in `crates/uor-r4-integer/src/capability_api.rs` to reject width-576 sessions (`self.bundle.model().config().width == 576 || state.len() == 576`) with a typed `IntegerError::Invalid` error (`"uor-r4.integer-session/1: width-576 value stores are not persisted; ..."`).
- Updated `CapabilityTruthMatrix::default().durable_memory` in `crates/uor-r4-integer/src/capability_api.rs` to include `"width-576 value stores are not persisted, and save_session/restore_session reject width-576 sessions"` alongside `uor-r4.integer-session/1` (width-256) and `uor-r4.stack-session/1` (`IntegerStackSession`).
- Added synthetic unit test `capability_api::tests::save_and_restore_session_reject_width_576_sessions` in `crates/uor-r4-integer/src/capability_api.rs` and integration test `restore_session_rejects_width_576_serialized_state` in `crates/uor-r4-integer/tests/capability_api_tests.rs`, updated `capability_api_lifecycle_on_dialogue576_bundle` to assert that `save_session` rejects width 576, and updated `docs/labs/m3-legacy-chatsession-persistence/README.md`.

## Measurements

- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --test capability_api_tests` on macOS aarch64: 4 passed, 0 failed, 1 ignored (`capability_api_lifecycle_on_dialogue576_bundle`) in 0.38s.
- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --lib capability_api::tests::` on macOS aarch64: 2 passed, 0 failed in 0.04s.
- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --lib session::tests::` on macOS aarch64: 7 passed, 0 failed in 0.06s.
- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --lib stack::tests::session_` on macOS aarch64: 7 passed, 0 failed in 0.10s.

## Checks run at the head

- `~/.cargo/bin/cargo fmt -p uor-r4-integer -- --check`: exit 0.
- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --test capability_api_tests`: 4 passed, 0 failed, 1 ignored.
- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --lib capability_api::tests::`: 2 passed, 0 failed.
- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --lib session::tests::`: 7 passed, 0 failed.
- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --lib stack::tests::session_`: 7 passed, 0 failed.
- `~/.cargo/bin/cargo run -q --locked -p xtask -- roadmap render --check`: exit 0.
- `uor-harness loop gates --full`: `fmt-claim-wording` GREEN, `fmt-harness` GREEN, `claim-wording` GREEN, `loop-gates-watch` GREEN, `loop-drive` GREEN, `git-hooks` GREEN.
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: PASS.

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`).

## Still open

- None for `#2854`; `m3-legacy-chatsession-persistence` is complete (`status: done`). Whether to mirror the `m3-durable-memory` notebook row from `docs/labs/m3-legacy-chatsession-persistence/README.md` into `docs/research/notebook/m3-durable-memory.md` is queued on the PM decision card in `#2028` (`#2028#issuecomment-6335479675`).

## Plain summary

- In `crates/uor-r4-integer/src/capability_api.rs`, `IntegerCapabilityApi::save_session` and `IntegerCapabilityApi::restore_session` now reject width-576 sessions with a typed `IntegerError::Invalid` error because `uor-r4.integer-session/1` does not persist width-576 value stores (`persistent_values_576`, `dialogue_values_576`, `l2_pages_576`), and `CapabilityTruthMatrix::default().durable_memory` explicitly states `"width-576 value stores are not persisted, and save_session/restore_session reject width-576 sessions"` alongside `uor-r4.stack-session/1` (`IntegerStackSession`).
- In `crates/uor-r4-integer/src/capability_api.rs` and `crates/uor-r4-integer/tests/capability_api_tests.rs`, synthetic tests (`save_and_restore_session_reject_width_576_sessions` and `restore_session_rejects_width_576_serialized_state`) verify under `--features serving-wasm` that both `save_session` and `restore_session` reject width 576, while the ignored width-576 continuation test remains retired in favour of the `IntegerStackSession` reload gate (`#1476`).
