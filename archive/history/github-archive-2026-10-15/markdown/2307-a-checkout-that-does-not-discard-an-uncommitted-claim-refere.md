# #2307 — A checkout that does not discard an uncommitted claim (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T06:17:58Z
- **Updated:** 2026-10-13T06:20:29Z
- **Closed:** 2026-10-13T06:19:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2307
- **Merged:** 2026-10-13T06:19:40Z
- **Merge commit:** 106f8399047a6e07cef5d5ef8abf9aa912246f60
- **Base / Head:** main / UOR-Foundation:fix/switch-preserving-claims
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only.

## What and why

`docs/harness/claims.json` is **tracked**, because a claim has to reach the other labs, and it is
**appended to** by every `team claim` and `team release`. So the ordinary sequence — claim a branch,
then check something out — fails:

```
error: Your local changes to the following files would be overwritten by checkout:
    docs/harness/claims.json
```

**I hit that three times in one session** and worked around it by copying the file aside by hand each
time. It's worth a command because **the wrong workaround is the easy one**: `git checkout -f`
discards the claim, and an uncommitted claim is still a lab saying *"I am working here."* Silently
dropping it is exactly the collision this plane exists to prevent.

## Result

Decision: **KEEP** (tooling).

`cli.py git switch <branch> [--create]` reads the modified claims, checks out, and writes them back.
The claim survives on the new branch, still uncommitted, exactly as the caller left it. When nothing
is uncommitted it says so rather than pretending to have preserved something.

```
$ cli.py git switch main
on main: an uncommitted claim was carried across the checkout
```

Three tests: an uncommitted claim survives, a clean tree is not disturbed, and a checkout that
**failed** reports that instead of success.

```
python3 -m unittest discover -s tests/harness -t .   -> 310 passed on 3.15 AND 3.9
python3 scripts/harness/cli.py check                 -> 51 rules, 0 errors
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/harness-install-2026-10-13/README.md
- [x] Branch deleted after merge

### Stated rather than hidden

- **This is a workaround for a design wart, not a fix for it.** A tracked file that every lab appends
  to will always fight a checkout. The real alternatives — a claims *directory* per lab like the
  ledger shards, or claims outside git entirely — are bigger decisions than this PR, and I am not
  making them silently.
- The command carries `claims.json` only. `presence/` is not tracked for the committing labs, and the
  ledger already shards per lab, so those do not hit the same wall.
- It reads, checks out, and writes back — not atomic. A crash between the checkout and the write
  would lose the uncommitted claim, which is why it reports what it did rather than assuming.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

