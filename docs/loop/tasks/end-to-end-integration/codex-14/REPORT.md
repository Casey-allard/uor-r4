# REPORT: end-to-end-integration, codex-14

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim end-to-end-integration --lab-topic type-head-restore` was refused because the item is
  claimed by this lab's codex-12 worktree (PR #4031, in the queue). The claim is this lab's own; the
  unit changes the session and its tests, adds no dependency and does not change `Cargo.lock`.
- **The compact session codec does not carry the payloads.** `session_codec.rs` encodes a fixed field
  set; a type-head session encoded there decodes with no payloads and is refused at restore. Extending
  that format is a separate unit.
- **The JSON snapshot grows, and the file bound had to grow with it.** A type-head read's cache is
  `context * width` i64 — about 393 KB at context 192, width 256 — and the snapshot carries one per
  read. The first version did not add them to `max_serialized_session_bytes`, so a
  production-shape type-head file would have been refused on restore (the in-memory path was fine).
  The bound now counts `payload_cache_elements * 24`, and a file test at context 1024 (about 3 MB,
  against the old ~272 KB bound) proves the file round-trip.
- **The payload contents are not validated, only their lengths**, consistent with logits, tokens and
  layer states: a same-length cache shifted by one position would restore and diverge. `save_state`
  is the trusted producer; a content digest would be a separate design.
- **Place, turn memory and the native Q31 read still refuse to restore.** Their state is not saved;
  only the Hamming payloads are new.
- **The Hamming trace is not saved.** The test clears the prefix before comparing step by step; the
  trace is a diagnostic, not session state.
- No model or quality claim.

## What changed

- `crates/uor-r4-integer/src/stack/session.rs`:
  - `SerializedStackSession.hamming_payloads` (serde default, skipped when empty).
  - `save_state` writes the caches; `restore_state` validates their lengths against
    `payload_caches(context)` and applies them.
  - the type-head refusal is replaced by the missing-payload refusal; place/memory/Q31 refusals stay.
- `crates/uor-r4-integer/src/session_codec.rs`: the initializers carry the new field (empty); a
  comment records that the compact codec drops it at encode and restore refuses it.
- `crates/uor-r4-integer/src/stack/hamming.rs`: `payload_cache_elements`, so the JSON bound counts
  the payloads without allocating them.
- `crates/uor-r4-integer/src/stack/tests.rs`: `a_session_with_type_heads_is_not_restored` becomes
  `a_session_with_type_heads_restores_with_its_payloads`, with a stripped-snapshot refusal.
- `crates/uor-r4-integer/tests/type_head_restore.rs` (new): a type-head trunk saved after four steps,
  restored, and continued with identical tokens and read choices.

## Measurements

- Type-head session: 4 prefix steps, save, restore, 4 more steps — tokens and Hamming traces
  identical at every step. A snapshot with the payloads cleared is refused.
- Production-shape file: context 1,024, width 256, two type-head reads; the saved JSON is about
  3 MB, above the old ~272 KB bound, and restores within the new bound; the continuation tokens
  match.
- Copy-only restore unchanged (the test's second leg).
- Library: **674 passed**, 1 ignored.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --test
  type_head_restore` — 1 passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib` — 674 passed.
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib
  --tests` — no warning in the changed lines (the pre-existing tests.rs warnings remain).
- `git diff --check`, the harness contract rule, the REPORT check and the exact-head review — at the
  head.

## Skills used

- `solution-integrity`: the saved payloads are validated and the stripped snapshot refused, so a
  session cannot restore to a different continuation.
- `scope-guard`: the session, the codec initializers and one test; the codec format, place, memory
  and Q31 history are untouched.
- `test-freeze`: the behaviour test that asserted the refusal is replaced by the restore test; the
  commit carries a `TEST-CHANGE:` line.

## Still open

- The compact codec's payload section (a format change).
- Place, turn-memory and Q31-read restore.
- The read-free ring buffer (codex-12's `O(context)` eviction).
- C (e-means) waits on a live codebook consumer.

## Plain summary

A type-head Hamming session now saves and restores its per-position payloads, so it continues exactly
after a restore instead of being refused. A snapshot without the payloads is refused, the compact
codec still does not carry them, and the other stateful reads keep their refusals.
