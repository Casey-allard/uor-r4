# ORDER: infra-loop-harness, codex-4

Lab: codex. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md).
Setup row: 10.
Reviewed head: `355bd9de24f0e5db095bc3745465817dc7b6215d`.

## Why this unit exists

The `loop-drive` gate's `post_note_with_pipes_the_body_to_gh_issue_comment` test writes a fake
`gh` script and executes it immediately. Twice now (the #3982 and #3992 fresh-main close
verifications) it failed with `Text file busy (os error 26)` and reddened the gate run, forcing a
re-run. The test closes the file before executing it, so this is a transient OS `ETXTBSY`, not a
leaked handle or a mechanism defect. This unit retries that one spawn a bounded number of times
and still fails on a persistent error.

## Claim record

`loop claim infra-loop-harness --lab-topic drive-etxtbsy` was refused with exit 2 because the
item's own scope lists the shared registration file `docs/harness/gates.json`, which no item scope
may list. The work continued under the owner's direction; the changed file is not a shared
registration file.

## What changes

`post_note_with` spawns `gh`/the fake `gh` once. The spawn moves into one helper that retries up
to five times with a short bounded backoff **only** when the OS returns `ETXTBSY` (errno 26). Any
other error, and the fifth `ETXTBSY`, is returned unchanged, so a permanently busy file still
fails. `post_note` and the test both use it.

## Files in scope

- `crates/uor-r4-harness/src/loopdrive.rs`
- `crates/uor-r4-harness/tests/loop_drive.rs`
- `docs/loop/tasks/infra-loop-harness/codex-4/**`

Out of scope: any other harness rule, any gate definition, any shared registration file.

## Size

About 25 to 60 lines including the comment and records.

## Done when

1. The `ETXTBSY` spawn is retried a bounded number of times; other errors are unchanged.
2. `cargo test -p uor-r4-harness --test loop_drive` passes, including the note test.
3. `cargo test -p uor-r4-harness`, formatting, touched-file Clippy, `git diff --check`,
   `uor-harness check --only harness.native_space_contract`, the REPORT check and the exact-head
   review pass.

## Cancel

Stop if the retry can hide a persistent failure, if the error is not the transient OS case, or if
the change needs a gate or shared-file edit.

STOP and report when done.
