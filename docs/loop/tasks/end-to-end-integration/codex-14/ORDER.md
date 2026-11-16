# ORDER: end-to-end-integration, codex-14

Lab: codex. Item: [end-to-end-integration](../../../../roadmap/items/end-to-end-integration.md).
Parent: [codex-12](../codex-12/ORDER.md). Reviewed head: `2d7014a53`.

## Why this unit exists

A Hamming read with type heads writes one payload row per position, and `restore_state` refuses such
a session because the payloads were not saved. The copy-only path already restores exactly. This unit
saves the payload caches with the session and restores a type-head session byte-for-byte, instead of
refusing it.

## Claim record

`loop claim end-to-end-integration --lab-topic type-head-restore` was refused because the item is
claimed by this lab's codex-12 worktree (PR #4031, in the queue). The claim is this lab's own; the
unit changes the session and its tests, adds no dependency and does not change `Cargo.lock`.

## Method

- `SerializedStackSession` gains `hamming_payloads: Vec<Vec<i64>>` (one `context * width` cache per
  read with type heads, empty otherwise), saved and restored with the session.
- `restore_state` validates the lengths against `ServedHamming::payload_caches` and applies them;
  a snapshot without them is refused. Place, turn memory and the native Q31 read keep their
  refusals.
- The compact codec (`session_codec.rs`) does not carry payloads; a decoded session has none, so a
  type-head restore through the codec is refused rather than silently different.
- The existing refusal test becomes the restore-and-continue test, and a stripped snapshot is the
  new refusal case.

## Files in scope

- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/session_codec.rs` (initializers and a comment only)
- `crates/uor-r4-integer/src/stack/tests.rs` (the behaviour test)
- `crates/uor-r4-integer/tests/type_head_restore.rs` (new)
- `docs/loop/tasks/end-to-end-integration/codex-14/**`

Out of scope: the compact codec format, place/register, turn memory, the Q31 read history, a schema
version bump (the field is optional), a dependency or a GPU run.

## Done when

1. A type-head session saved after a prefix restores and continues with identical tokens and read
   choices.
2. A snapshot without the payloads is refused.
3. A copy-only session still restores as before.
4. No existing assertion is weakened except the refusal test that this unit replaces; the library
   suite passes.
5. `git diff --check`, the harness contract rule, the REPORT check and the exact-head review pass.

## Cancel

Stop if restoring the payloads changes any continuation token, or if the field needs a schema
version bump. Do not extend the compact codec here.

STOP and report when done.
