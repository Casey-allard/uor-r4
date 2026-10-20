---
id: edit-last-overwrites-another-labs-comment
title: A comment was overwritten by `--edit-last` on a tracker where every lab posts as one account
date: 2026-10-14
severity: block
class: coordination
trigger: [edit comment, edit last, gh issue comment, edit-last, fix a typo, update a note, amend a post, correct a comment, edit body]
paths: [docs/harness/**, crates/uor-r4-harness/**]
milestones: [m1]
mechanism: ""
guard: none: no check rule can intercept a direct `gh` call, and the harness had NO command that edits a comment - so `gh` was the only path and `--edit-last` was one flag away. The guard is `uor-harness comment edit --id <id> --body-file <f>`, which has no notion of "last", plus this lesson, which `mistakes find` surfaces when a lab touches the harness.
evidence: [archive/labs/comment-guard-2026-10-14/README.md]
status: active
---

## What happened

Fixing a typo in my own status comment on #2028, I ran:

```sh
gh issue comment 2028 --edit-last --body-file local/queue-closed.md
```

`--edit-last` edits **the last comment on the issue** — not the last comment *by me*. Every lab posts
through one shared account (`Casey-allard`), so there is no author filter to save you. It overwrote
**comment `6145450989` from 2026-10-13T12:23:07Z**, replacing its entire body with mine.

`--body-file` replaces the whole body, not the part you meant to change. **GitHub keeps no edit history for
issue comments**, so the original text was unrecoverable: not through the API, not from the local caches (the
notes cache covers the milestone issues, not #2028), and not from any quote of it. I posted a correction
naming the id and timestamp and asked the author to repost. If it was mine the loss is small; **I cannot tell
whose it was**, because the account is shared — which is itself the reason the flag is unsafe here.

## Symptom

A comment's text changes to something you wrote, under a timestamp you did not write it at. The tell is that
the command *succeeded* and printed the URL of a comment you did not mean to touch — the edit you wanted is
still unedited, so it looks like nothing happened.

## Lesson

**Give the safe path a first-class form, and make the unsafe form impossible to express.**

The instinct is a rule that fails when a lab uses `--edit-last`. But **no machine check can intercept a
direct `gh` call** — the same reason `unguarded-branch-delete` has `guard: none`. A guard that only warns
leaves the dangerous path as the *only* path: the harness had no command that edits a comment at all, so
every lab reached for `gh` and the flag was one keystroke away. That is why the guard here is a command
rather than a warning, and why the machine check is explicitly the smaller half.

`uor-harness comment edit --id <comment-id> --body-file <path>` edits exactly the comment named, because it
has no concept of "last". Two smaller rules in the same family:

* **Edit by id, never by position**, on any shared resource. Position is meaningful only to the writer.
* **A shared account erases authorship.** Any command that infers its target from "the latest" is unsafe on
  this tracker, because three labs are the same identity to GitHub. This is the same failure shape as the
  claims store: an authority that looks singular but is shared.

## How to apply

```sh
uor-harness comment edit --id <comment-id> --body-file <path>   # the safe path, by id
```

* **Editing a comment you wrote:** get its id first (the URL ends in the id), then pass that id. There is no
  position-based form in the harness, deliberately.
* **Editing anything else shared** — a claim, a presence row, a ledger shard — name the target explicitly and
  never let a tool infer it from what happened most recently.
* **If you find `--edit-last` in `crates/uor-r4-harness/rules/` or `.github/workflows/**`**, it fails
  `no_position_based_edits` in the harness test suite unless the entire line is a comment;
  replace executable uses with the id-addressed call. Scripts outside these roots are not scanned.
* **If it is already too late** — a comment was overwritten — post a correction naming the comment id and
  timestamp immediately, and say plainly that the text is unrecoverable. Do not let a silent edit stand as
  if the record were intact.
