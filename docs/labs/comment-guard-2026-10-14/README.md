# The safe path had to exist, not just be documented

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** PM tasking — a harness guard against the `--edit-last` comment-overwrite hazard

## What happened

Fixing a typo in my own status comment on #2028, I ran `gh issue comment 2028 --edit-last --body-file ...`.
`--edit-last` edits the **last comment on the issue**, not the last comment *by me* — and every lab posts
through one shared account, so there is no author filter. It replaced the entire body of **comment
`6145450989` (13 Oct 12:23Z)**. `--body-file` replaces the whole body, GitHub keeps no edit history, and the
text was unrecoverable: not via the API, not from the local caches. I posted a correction naming the id and
timestamp and asked the author to repost; I cannot tell whose it was, which is itself the reason the flag is
unsafe here.

## The guard, in three parts

**1. The safe path exists — and that is the substantive fix.** The harness had **no command that edits a
comment**, so `gh` was the only path and the hazardous flag was one keystroke away. A warning would not have
prevented what happened to me. `uor-harness comment edit --id <comment-id> --body-file <path>` now edits
exactly the comment named and **cannot express "last"** — there is no position-based form to reach for.

**2. The lesson surfaces where the mistake is one step from being repeated.**
`docs/harness/mistakes/edit-last-overwrites-another-labs-comment.md`, severity `block`, `class: coordination`,
carrying the trigger words a lab would be typing (`edit comment`, `fix a typo`, `gh issue comment`, `edit
body`) so `mistakes find -f` raises it before the command, not after.

**3. A machine check on the executable workflow roots covered by CI.** `no_position_based_edits` scans
`scripts/harness/**` and `.github/workflows/**` for the flag and fails with the file, line and the
id-addressed replacement. It is explicit
about its limit: **it cannot intercept a hand-typed `gh`** — which is exactly why part 1 had to be built — but
it stops the hazard being baked into a script or a workflow where it would fire on a schedule.

## Verified

| case | result |
|---|---|
| `comment edit` with no `--id` | refuses, and the message names the hazard and why |
| an empty `--body-file` | refused **before** `gh` runs (a blank comment is the same loss with less noise) |
| **a real edit by id against the live tracker** | `edited comment 6173738543: .../issues/2028#issuecomment-6173738543` |
| the lesson's four required sections | present: `What happened`, `Symptom`, `Lesson`, `How to apply` |
| the lesson loads in the registry | `block  edit-last-overwrites-another-labs-comment` |

## Stated rather than hidden

* **The overwritten comment's text is gone.** This unit prevents the next one; it cannot restore that one.
* **The machine check is narrow on purpose.** It guards what executes, not what a lab types, and the record
  says so rather than implying the hazard is fully closed.
* **A note for the other task:** the same principle — a weaker claim must not be mistakable for a stronger
  one — is why the id form refuses to infer a target, and why the DAG reader's range-verified reads will be
  reported alongside the unverified remainder.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

## Codex continuation — 15 October 2026

Owner-authorized continuation of parked #2488 preserves DeepSeek's authorship and
source: `2b4256d6e` and `c0f03685d` were cherry-picked with provenance into
`3c5a25e32` and `5c12dfa9f` on fresh main `5fd44f578`. The
[PM re-review](https://github.com/UOR-Foundation/uor-r4/pull/2488#issuecomment-6182786755)
verified the functional fixes at `c0f03685d`: 504 Rust unit tests passed, plus
2 integration tests; its doccheck resolver found 146 rows and no bad commands.
Those are historical results at that source, not counts inferred for this continuation.

This continuation corrects the scan scope in the record, current-state and lesson,
separates the current-state footer from the next heading, and documents that only
full-line comments are skipped. Inline comments containing the flag still trigger
the scan. These are text/docstring corrections only; no runtime behavior changes,
live comment edits, model/GPU work or training release are part of the continuation.

Current continuation validation, run from the claimed Codex worktree with
`CARGO_TARGET_DIR=local/harness-target`:

| Check | Observed result |
|---|---|
| `cargo test -p uor-r4-harness --offline` | 504 unit tests passed, 6 ignored; 2 integration tests passed; 1 doc-test passed |
| Explicit ignored `doccheck::documented_commands_tests::the_table_agrees_with_pythons_live_parser` | 1 passed; this does not imply the other ignored tests ran |
| `python3 scripts/harness/doccheck.py --json` | 146 commands checked, 0 problems |
| `python3 -m unittest discover -s tests/harness -p test_doccheck.py` | 14 passed |
| `cargo fmt -p uor-r4-harness --check` | PASS |
| `cargo clippy -p uor-r4-harness --offline --all-targets -- -D warnings` | PASS |

The original live comment-edit exercise above is retained historical evidence;
Codex did not repeat it during takeover. All current checks are local tooling checks.

### Integration with current main — 15 October 2026

The protected-delivery reconciliation uses main `57ca5a96d`, preserving its
bounded inbox refresh and unread-note reporting together with the approved
explicit-comment-ID edit implementation. `notes.rs` retains main's complete
file plus the approved 139-line comment-edit section; the command dispatcher
merges without changing either interface. DeepSeek's original commits remain
in the continuation's history; its parked branch is not rewritten.

Fresh integrated tooling checks: **520 unit tests passed, 6 ignored; 2 integration
tests and 1 doc-test passed**. The explicitly selected ignored Rust/Python parser
parity test also passed; the remaining ignored tests were not run. The command
resolver checked **151 commands, 0 problems**, and all **14 Python doccheck tests**
passed. Rust formatting, clippy with warnings denied, and append-only ledger
verification passed. These are harness/tooling checks, not model execution or
research results. No live comment-edit exercise was repeated.
