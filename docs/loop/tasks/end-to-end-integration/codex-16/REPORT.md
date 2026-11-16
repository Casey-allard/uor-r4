# REPORT: end-to-end-integration, codex-16

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The compact format gains a third magic**, and an old reader refuses `UR4SESS3` rather than
  silently dropping the payloads. That is the intended trade: the old magics are byte-identical,
  and a reader that cannot carry the payloads must not restore a type-head model.
- **The payloads make the compact stream large.** They are `context * width` `i64` per read: at
  context 1,024 and width 256 with two reads, about 4 MB, carried verbatim. The binary form is
  smaller than the JSON snapshot's ~3 MB at context 1,024 in the codex-14 test, but it is still the
  dominant part of the stream.
- **The codec stays infallible.** `encode` returns `Vec<u8>`; a malformed body is refused at
  `decode` (bad content tag, truncation, trailing bytes) and a payload of the wrong length is
  refused by `restore_state`.
- `rustfmt` wrapped one pre-existing line in `tests/type_head_restore.rs`; no logic changed.
- **The full `serving-wasm` suite did not compile on the parent.** codex-14 added
  `hamming_payloads` to `SerializedStackSession` but missed
  `tests/session_codec_interop_fixture.rs`, so `cargo test --features serving-wasm --no-run` failed
  with `E0063` before this unit's own tests ran. The fixture now sets the field empty; its pinned
  `UR4SESS1` bytes and sha256 still match.
- **The claim was refused** because codex-15 (PR #4033) holds the item in the merge queue.
- No model or quality claim.

## What changed

- `crates/uor-r4-integer/src/session_codec.rs`: `MAGIC_PAYLOADS` (`UR4SESS3`), chosen only when
  `hamming_payloads` is non-empty; a flags byte plus the optional content hash; one `i64` vector per
  read; decode accepts the three magics and refuses a bad content tag.
- `crates/uor-r4-integer/tests/type_head_restore.rs`: a type-head session saved, encoded, decoded,
  restored and continued with identical tokens.
- `crates/uor-r4-integer/tests/session_codec_interop_fixture.rs`: the missing
  `hamming_payloads: Vec::new()` restores the object-store interop control that codex-14 broke.
- Three unit tests: payload round-trip, payloads plus content, and the old magic for a payload-free
  session.

## Measurements

- Integration: 4 prefix steps, `encode`/`decode`, restore, 4 more steps — identical tokens.
- Codec unit tests: 9 passed (the previous 6 plus 3 new).
- Library: **677 passed**, 1 ignored.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm --lib session_codec` — 9 passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --test
  type_head_restore` — 3 passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm --test
  session_codec_interop_fixture` — 2 passed (the pinned bytes and sha256 are unchanged).
- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --no-run` — clean;
  the whole suite now compiles.
- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib` — 677
  passed, 1 ignored.
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib
  --tests` — no warning in the changed lines.
- `rustfmt --check` on the two changed files — clean.
- `git diff --check`, the harness contract rule, the REPORT check and the exact-head review — at
  the head.

## Skills used

- `solution-integrity`: the new magic never changes an old byte, and a decoded payload is validated
  by restore before it is used.
- `scope-guard`: one codec file and one test file; the JSON snapshot and the `UORMEM01` envelope are
  untouched.
- `test-freeze`: adds tests and wraps one pre-existing line; no assertion is weakened.

## Still open

- Place, turn-memory and native Q31 read restore keep their refusals.
- The `UORMEM01` envelope's own version is still 1; it embeds the codec bytes unchanged.
- codex-14's field addition left this fixture broken on `main`; codex-16 repairs it, but the full
  suite is not part of the per-PR gate, so another added field can repeat the break.
- I3 still needs a genuine live k-means site; the exact read index is not one (the index is exact
  over the 64-bit sign code under the read's own popcount metric).

## Plain summary

A type-head session now survives the compact codec: a third magic carries the per-position payloads
(and the content hash when present), while every payload-free session keeps its exact old bytes. A
decoded snapshot restores and continues token-for-token.
