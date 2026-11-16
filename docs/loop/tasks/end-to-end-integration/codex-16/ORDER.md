# ORDER: end-to-end-integration, codex-16

Lab: codex. Item: [end-to-end-integration](../../../../roadmap/items/end-to-end-integration.md).
Parent: [codex-14](../codex-14/ORDER.md). Reviewed head: `3c3d27f5e`.

## Why this unit exists

codex-14 made a type-head Hamming session save and restore its per-position payloads in the JSON
snapshot. The compact `UR4SESS1`/`UR4SESS2` codec still dropped them, so the object-store path could
not persist such a session. This unit adds a third magic that carries them, leaving every
payload-free session byte-identical.

## Claim record

`loop claim end-to-end-integration --lab-topic type-head-codec` was refused because the item is
claimed by this lab's codex-15 worktree (PR #4033, in the merge queue). The claim is this lab's own;
the unit adds no dependency and does not change `Cargo.lock`.

## Method

- `UR4SESS3` is used **only** when `hamming_payloads` is non-empty. It writes a flags byte for the
  optional content hash, then the hash if set, then one `i64` vector per read; the layer section is
  unchanged.
- `UR4SESS1` and `UR4SESS2` keep their exact bytes; decode accepts all three magics.
- A bad content tag is refused; restore still validates every payload length against
  `payload_caches`.
- One unit test round-trips payloads with and without a content hash and pins the old magics; one
  integration test saves a type-head session, encodes, decodes, restores and continues.

## Files in scope

- `crates/uor-r4-integer/src/session_codec.rs`
- `crates/uor-r4-integer/tests/type_head_restore.rs` (one test; rustfmt also wraps one pre-existing
  line in this file)
- `crates/uor-r4-integer/tests/session_codec_interop_fixture.rs` (one missing field inherited from
  codex-14; the pinned `UR4SESS1` bytes are unchanged)
- `docs/loop/tasks/end-to-end-integration/codex-16/**`

Out of scope: the JSON snapshot, the `UORMEM01` envelope version, place/turn-memory/Q31 restore, and
any schema or dependency change.

## Done when

1. A type-head snapshot round-trips through the codec with its payloads (with and without a content
   hash) and restores to the same continuation.
2. A snapshot without payloads encodes with its old magic and bytes.
3. A malformed content tag is refused.
4. The library suite passes.
5. `git diff --check`, the harness contract rule, the REPORT check and the exact-head review pass.

## Cancel

Stop if the new magic must change the bytes of a payload-free session, or if the object-store
interop pin cannot be kept.

STOP and report when done.
