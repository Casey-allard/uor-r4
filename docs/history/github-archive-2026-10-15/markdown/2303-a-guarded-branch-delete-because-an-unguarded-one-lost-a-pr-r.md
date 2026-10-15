# #2303 — A guarded branch delete, because an unguarded one lost a PR (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T05:56:46Z
- **Updated:** 2026-10-13T05:59:52Z
- **Closed:** 2026-10-13T05:59:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2303
- **Merged:** 2026-10-13T05:59:00Z
- **Merge commit:** d4dd3caa99ec15d51c55bc94b54c22a9ecf19cee
- **Base / Head:** main / UOR-Foundation:fix/guarded-branch-delete
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only.

## What happened

I merged with `gh pr merge --auto`, polled the PR state, and then ran the branch cleanup
**unconditionally**. Partway through the poll the PR **auto-closed as `DIRTY`** — a concurrent merge
made it conflict, so it never merged. The loop exited without noticing and the delete ran anyway,
removing the only remote copy of the Rust heartbeat work.

**It was recovered** — the commit was still reachable locally, the branch was recreated, rebased and
merged as #2302. But nothing in the sequence *prevented* the loss, and recovery depended on the local
clone still holding the commit.

Two failures compounded: a poll loop that treated an unexpected state (`CLOSED`) as "done", and a
delete with **no precondition**, acting on the assumption the merge had happened. The repository's
own rule — verify the merge commit on a fresh `origin/main`, *then* delete — was followed in spirit
and not in fact.

## Result

Decision: **KEEP** (tooling).

`cli.py git delete-merged --branch <name> [--apply]` refuses unless the merge is **proven**:

| situation | result |
|---|---|
| merged PR whose merge commit is an ancestor of `origin/main` | allowed |
| merged PR whose merge commit is **not** on `origin/main` | **refused** |
| no merged PR | **refused** |
| GitHub unreachable, so the merge cannot be proven | **refused** |
| `origin/<branch>` holds commits the merge did not carry | **refused** |
| **any lab holds a live claim on the branch** | **refused** |

### Two of those were found by *using* the tool, not by reasoning about it

**1. A claim is checked first, independently of the merge.** A merged PR is not proof a branch is
finished — a lab can merge and keep going, or hold it while a follow-up lands. Gatekeeping on the
merge first *also* meant an unreachable GitHub returned **before** the claim guard ran, so the
protection failed exactly when the evidence was weakest. The first version cheerfully recommended
deleting `kimi/plan-refresh-2026-10-13`, which was under **Kimi's live claim**.

**2. The refusal flag and the refusal reason must be set together.** Twice in one function a guard
explained itself in `reason` but left the caller's flag set, so a **refused** branch still printed
"would delete". The flag is what acts; the reason is what a human reads.

Recorded as mistake `unguarded-branch-delete` so `mistakes find` surfaces it to every lab — with
`guard: none`, because no machine check can intercept a direct `git push --delete`. The guard is the
CLI refusing, which only helps if it is the command you run.

```
python3 -m unittest discover -s tests/harness -t .   -> 306 passed on 3.15 AND 3.9
python3 scripts/harness/cli.py check                 -> 50 rules, 0 errors
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Mistake recorded: docs/harness/mistakes/unguarded-branch-delete.md
- [x] Record: docs/labs/harness-rust-port-2026-10-13/README.md
- [x] Branch deleted after merge — **using the guarded command this PR adds**

### Stated rather than hidden

- **The guard only helps if it is the command you run.** A raw `git push origin --delete` is still
  unguarded, and no check can intercept it. This is a tool that makes the safe path easy, not an
  interlock that makes the unsafe path impossible — and the mistake entry says so.
- The poll-loop pattern in the mistake entry is illustrative. I have not turned it into a script,
  because the fix in the moment is to check the state you actually need and abort on anything else.
- The recovery was lucky: the local clone still held the commit. Had the work been pushed from a
  different machine, it would have been gone.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

