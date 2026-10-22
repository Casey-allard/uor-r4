# ORDER: m3-legacy-chatsession-persistence, antigravity-7

Lab: antigravity. Item: [m3-legacy-chatsession-persistence](../../../../roadmap/items/m3-legacy-chatsession-persistence.md). Base: `cee38b43e`.

## Goal

Repair PR #2948 per Claude's exact-head review (`https://github.com/UOR-Foundation/uor-r4/pull/2948#issuecomment-6335681096`). The item's done-when reads:

> Either the three value stores serialize and the test passes, or the ignored test is retired in favour of the `IntegerStackSession` reload gate. In both cases the "omits the width-576 value stores" wording is removed or updated where it appears (the test file header, `capability_api.rs`, the test's ignore message).

## Cancel probe (run first)

Inspected Claude's review on PR #2948 (`#2948#issuecomment-6335681096`) and `crates/uor-r4-integer/src/capability_api.rs` (`:55`, `:304`, `:324`), `crates/uor-r4-integer/src/session.rs` (`:971`, `:1207-1230`), and `crates/uor-r4-integer/tests/capability_api_tests.rs` (`:163-185`):
- `ChatSession::from_serialized` still accepts width 576 and reconstructs empty width-576 stores, so `IntegerCapabilityApi::save_session` and `IntegerCapabilityApi::restore_session` must explicitly reject width-576 sessions with a typed `IntegerError::Invalid` error, and `CapabilityTruthMatrix::default().durable_memory` must retain the explicit clause `"width-576 value stores are not persisted"`.
- Still needed: make `save_session` and `restore_session` reject width 576 with a typed error, keep `"width-576 value stores are not persisted"` in `durable_memory`, add synthetic tests proving both `save_session` and `restore_session` reject width 576, and cite `--features serving-wasm` test runs in `REPORT.md`.

## Files in scope

- `crates/uor-r4-integer/**`
- `docs/labs/m3-legacy-chatsession-persistence/**`

## Done when

- `IntegerCapabilityApi::save_session` and `IntegerCapabilityApi::restore_session` reject width-576 sessions with a typed `IntegerError::Invalid` error, and `CapabilityTruthMatrix::default().durable_memory` includes `"width-576 value stores are not persisted"`.
- Synthetic unit and integration tests under `--features serving-wasm` verify that `save_session` and `restore_session` reject width-576 sessions, and fail when the width-576 guard is removed.
- `REPORT.md` cites `--features serving-wasm` test runs and `uor-harness loop pr-check --local` passes cleanly.

## Stop and ask if

- Any file outside `crates/uor-r4-integer/**` or `docs/labs/m3-legacy-chatsession-persistence/**` (plus allowed shared/item status files) needs to change.
- Any owner-held boundary, bar, or GPU work is implicated.
