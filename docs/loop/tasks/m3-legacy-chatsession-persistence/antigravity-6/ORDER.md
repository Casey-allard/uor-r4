# ORDER: m3-legacy-chatsession-persistence, antigravity-6

Lab: antigravity. Item: [m3-legacy-chatsession-persistence](../../../../roadmap/items/m3-legacy-chatsession-persistence.md). Base: `f1c78da8e`.

## Goal

The item's done-when reads:

> Either the three value stores serialize and the test passes, or the ignored test is retired in favour of the `IntegerStackSession` reload gate. In both cases the "omits the width-576 value stores" wording is removed or updated where it appears (the test file header, `capability_api.rs`, the test's ignore message).

## Cancel probe (run first)

Ran `rg -n "ChatSession|persistent_values_576|dialogue_values_576|l2_pages_576" crates/` and inspected `crates/uor-r4-integer/src/session.rs`, `crates/uor-r4-integer/src/capability_api.rs`, `crates/uor-r4-integer/tests/capability_api_tests.rs`, `crates/uor-r4-integer/src/bin/uor-chat.rs`, and `crates/uor-r4-integer/src/stack/tests.rs`.
- No shipped serving or CLI path builds a width-576 `ChatSession` (`uor-chat` uses `DialogueConversation` for `--bundle` and `IntegerStackModel` / `StackChat` for `--stack`; `wasm_bindgen_exports.rs` uses `IntegerStackSession` via `wasm_serve`), and the local `dialogue-child-bundle-1` fixture is not present.
- Meanwhile, the `IntegerStackSession` reload gate (`uor-r4.stack-session/1` in `crates/uor-r4-integer/src/session_codec.rs`) is already active and tested on every run by `session_save_and_restore_produces_bit_identical_logits` and `session_pause_save_fresh_restore_greedy_continuation` in `crates/uor-r4-integer/src/stack/tests.rs`.
- Still needed: retire the ignored `restored_dialogue576_session_continues_identically` test in favour of the `IntegerStackSession` reload gate (`#1476`), update the `capability_api.rs` and `capability_api_tests.rs` wording so the stale "omits the width-576 value stores" defect text is retired, and verify with unit/integration and negative tests.

## Files in scope

- `crates/uor-r4-integer/**`
- `docs/labs/m3-legacy-chatsession-persistence/**`

## Done when

- Design note posted on `#2031` choosing retirement of the ignored width-576 `ChatSession` continuation test in favour of the `IntegerStackSession` reload gate (`session_save_and_restore_produces_bit_identical_logits` and `session_pause_save_fresh_restore_greedy_continuation` in `crates/uor-r4-integer/src/stack/tests.rs`, folding `#1476`).
- `restored_dialogue576_session_continues_identically` is retired from `crates/uor-r4-integer/tests/capability_api_tests.rs`, and the "omits the width-576 value stores" / "omits the 576-wide value stores" wording is updated in `crates/uor-r4-integer/tests/capability_api_tests.rs` and `crates/uor-r4-integer/src/capability_api.rs`, with a contract assertion in `capability_api_tests.rs` verifying the updated durable-memory truth matrix entry.
- `~/.cargo/bin/cargo test -p uor-r4-integer` and `uor-harness loop gates --full` pass cleanly.

## Stop and ask if

- Any file outside `crates/uor-r4-integer/**` or `docs/labs/m3-legacy-chatsession-persistence/**` (plus allowed shared/item status files) needs to change.
- Any owner-held boundary, bar, or GPU work is implicated.
