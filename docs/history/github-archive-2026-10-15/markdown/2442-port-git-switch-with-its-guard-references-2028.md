# #2442 — Port git switch with its guard (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T04:59:16Z
- **Updated:** 2026-10-14T05:02:09Z
- **Closed:** 2026-10-14T05:01:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2442
- **Merged:** 2026-10-14T05:01:40Z
- **Merge commit:** 2ca037c17279eb9977bcf9cc67af5612655f9553
- **Base / Head:** main / UOR-Foundation:deepseek/delegation-tail
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`git switch`, ported WITH its guard.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

**`uor-harness git switch` is Rust — and the guard came with it.**

The objective says to port `git switch`/`delete-merged` **"only with their guards"**, and **this unit is what that means in practice: the guard is not a wrapper around the port, it is the reason the command exists.**

## The guard, and why it *is* the command

`claims.json` is **TRACKED**, because a claim has to reach the other labs, and it is **appended to by every `team claim` and `team release`**. So an ordinary checkout fails:

```
error: Your local changes to the following files would be overwritten by checkout:
    docs/harness/claims.json
```

**The wrong workaround is the easy one.** `git checkout -f` discards the claim — **and a claim nobody has committed is still a lab saying "I am working here".** Silently dropping it is **exactly the collision this plane exists to prevent.**

**So the port reads the modified claims, checks out, and writes them back.**

**This guard has a recorded cost**: Python's docstring says the failure *"happened three times in one session, and each time the workaround was to copy the file aside by hand."*

## Verified against Python on a scratch repo with a dirty claim

```
rust:   on other: an uncommitted claim was carried across the checkout
python: claims after: {"x":{"lab":"probe"}}
rust:   claims after: {"x":{"lab":"probe"}}
```

**Same claim, same survival.** And Rust accepts **`git switch other --repo <path>`**, which Python's argparse rejects as `unrecognized arguments` — **a divergence in the port's favour, stated rather than hidden.**

## Four guard tests — the third is the one that matters

| case | asserted |
|---|---|
| an uncommitted claim **survives** | the claim text is still there |
| a **clean** claim reports *"nothing to carry"* | it does not claim credit |
| **a FAILED checkout does not report preservation** | `switched: false`, `claims_preserved: false` |
| `-b` creates the branch | `rev-parse --abbrev-ref HEAD` |

**The third is why the guard is tested rather than assumed**: the tree was not changed, so **reporting `claims_preserved: true` would claim work the command did not do** — the same "reports success having done nothing" shape this port keeps finding.

## `delete-merged` is NOT ported, and its guard is why

**I read the whole chain and ported no fragment of it.** Deleting without proving the merge **removed the only remote copy of unmerged work once already.**

1. **A live claim, checked FIRST, needing no network** — *"a merged PR is not proof a branch is finished"*; gating on the merge first meant **an unreachable GitHub let a claimed branch be deleted whenever the network was down** — *"the protection failed exactly when the evidence was weakest."*
2. **A merged PR whose merge commit is an ancestor of `origin/main`.**
3. **`git cherry`** — after **two earlier attempts wrong in opposite directions** (`rev-list --count` refused every squash merge; `diff --stat` refused branches merely behind main).
4. **`safe_to_delete` is the ONLY flag the caller acts on**, because an earlier version used `merged` and *"a guard returned early without clearing it, so a REFUSED branch still printed 'would delete'."*

**A port that skipped any of those would be a weaker check wearing the command's name.** It stays delegated, and the dispatch site says so.

```
cargo test -p uor-r4-harness                    -> 395 passed, 4 ignored
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0
```

Record: `docs/labs/git-switch-guard-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`delete-merged` is still delegated, as are `git install-hooks` and `git status`** — **six `delegate_to_python` sites remain.**
- **The `--repo` acceptance is a divergence in the port's favour** — it makes documented invocations work that Python rejects. **A divergence is still a divergence**, recorded rather than celebrated quietly.
- **The guard preserves the claim by rewriting the file after the checkout.** **If the process dies between the two, the claim is lost** — Python has the same window, **and it is the window this command exists to close.** A `git stash`-based approach would be atomic and **was not attempted**, because matching the reference implementation was the job here.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

