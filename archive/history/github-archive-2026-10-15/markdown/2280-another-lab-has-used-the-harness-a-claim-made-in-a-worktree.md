# #2280 — Another lab HAS used the harness; a claim made in a worktree was invisible (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T03:51:14Z
- **Updated:** 2026-10-13T03:53:24Z
- **Closed:** 2026-10-13T03:53:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2280
- **Merged:** 2026-10-13T03:53:04Z
- **Merge commit:** a1ce5a552f2a93cbb5c584f89d10c3ba90354bc8
- **Base / Head:** main / UOR-Foundation:fix/claims-land-in-main
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only: this touches the harness, which runs no model code.

## A correction first

I reported twice — and wrote into the record — that **no other lab had ever run this harness**.
That was wrong. The committed ledger says otherwise:

| receipt | lab | branch / work | outcome |
|---|---|---|---|
| `22:10:42` `skill` x2 | kimi | the no-matmul interlock record | PR **#2265** merged |
| `22:44:16` `skill` x2 | kimi | the D25 hygiene docs sweep | PR **#2268** merged |
| `23:15:06` `claim` | kimi | `kimi/rung0-prep-2026-10-13` | PR **#2272** merged |
| `23:44:17` `claim` | kimi | `kimi/d27-rust-first-2026-10-13` | PR **#2278** merged |

Those branch names are Kimi's own work — not strings I produced. So a foreign lab has run the
tool, claimed branches through `team claim`, and had its record land in the sharded ledger. The
adoption question is answered for the claim and ledger paths.

What it does **not** establish: that presence was seen by a peer while Kimi was live. The presence
files for that window were the fabricated ones removed in #2275, and the live tier holds nothing now.

## Result

Decision: **KEEP** — the harness stays, and the claim path is repaired.

## The bug the correction exposed

Those claim receipts exist while `docs/harness/claims.json` is **empty**. A claimed branch left no
trace in the shared store, so the next lab could not see it had ever been held — which defeats the
entire point of moving claims into git.

**Cause:** a claim made from a linked worktree was written to *that worktree's own*
`docs/harness/claims.json` — invisible to every other lab, and deleted along with the worktree by
routine cleanup. The receipt survived; the claim did not.

**Fix:** `claims_file()` resolves the main checkout via `git worktree list` and always writes there.

Verified from inside a real worktree (created and removed for the test):

```
paths.repo      = .../.worktrees/claim-test
main_worktree() = /Users/casey.allard/uor-r4
claims_file()   = /Users/casey.allard/uor-r4/docs/harness/claims.json
```

```
python3 -m unittest discover -s tests/harness -t .   -> 278 tests, OK
python3 scripts/harness/cli.py check                 -> 43 rules, 0 errors
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record corrected: docs/labs/multilab-harness-2026-10-12/README.md
- [x] Branch deleted after merge

### Stated rather than hidden

- **Presence remains unproven with a foreign lab.** Kimi's claims prove the claim path; nobody has
  watched a peer appear live on the presence board.
- Kimi's two claims were **released or lost**, and no release receipt exists for them. I could not
  determine which, and I have not invented an explanation — the fix above prevents the loss from
  recurring regardless of which it was.
- I still cannot rule out other synthetic records from earlier in this work. If a lab appears in the
  board that you know did not check in, that is the same failure and I want to know.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

