# `git switch`, ported WITH its guard

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the delegation tail, beginning with the one the objective names

## What landed

**`uor-harness git switch` is Rust, and the guard came with it.**

**The objective says to port `git switch`/`delete-merged` "only with their guards"** — and this unit is
what that instruction means in practice: **the guard is not a wrapper around the port, it is the
reason the command exists.**

## The guard, and why it is the command

`claims.json` is **TRACKED**, because a claim has to reach the other labs, and it is **APPENDED to by
every `team claim` and `team release`**. So the ordinary sequence — claim a branch, then check
something out — fails:

```
error: Your local changes to the following files would be overwritten by checkout:
    docs/harness/claims.json
```

**The wrong workaround is the easy one.** `git checkout -f` discards the claim — **and a claim nobody
has committed is still a lab saying "I am working here".** Silently dropping it is **exactly the
collision this plane exists to prevent.**

**So the port reads the modified claims, checks out, and writes them back**, and the claim survives on
the new branch still uncommitted, exactly as the caller left it.

**This is a guard with a recorded cost**: Python's own docstring says the failure *"happened three
times in one session, and each time the workaround was to copy the file aside by hand."*

## Verified against Python, on a scratch repo with a dirty claim

```
rust:   on other: an uncommitted claim was carried across the checkout
        claims after: {"x":{"lab":"probe"}}
python: claims after: {"x":{"lab":"probe"}}
```

**Same claim, same survival.** And the Rust form accepts **`git switch other --repo <path>`**, which
Python's argparse rejects as `unrecognized arguments` — **the port is more permissive about flag
position and identical in effect**, which is a divergence in the port's favour and is stated rather
than hidden.

## Four guard tests, and the third is the one that matters

| case | asserted |
|---|---|
| an uncommitted claim **survives** the checkout | the claim text is still there |
| a **clean** claim reports *"no uncommitted claim to carry"* | it does not claim credit |
| **a FAILED checkout does not report the claim as preserved** | `switched: false`, `claims_preserved: false` |
| `-b` creates the branch | `rev-parse --abbrev-ref HEAD` is the new name |

**The third is why the guard is tested rather than assumed**: the working tree was not changed, so
**reporting `claims_preserved: true` would claim work the command did not do** — the same "reports
success having done nothing" shape this port has found repeatedly.

## `delete-merged` is NOT ported, and the reason is its guard

**I read the whole guard chain and did not port a fragment of it.** Deleting a branch without proving
its merge **removed the only remote copy of unmerged work once already** — the docstring records it.

The chain is, in order:

1. **a live claim, checked FIRST and needing no network** — *"a merged PR is not proof a branch is
   finished"*, and gating on the merge first meant **an unreachable GitHub let a claimed branch be
   deleted whenever the network was down** — *"the protection failed exactly when the evidence was
   weakest"*;
2. **a merged PR whose merge commit is an ancestor of `origin/main`**;
3. **`git cherry`** to prove the branch introduces nothing upstream lacks — **after two earlier
   attempts were wrong in opposite directions** (`rev-list --count` refused every squash merge;
   `diff --stat` refused branches merely behind main);
4. **`safe_to_delete` is the ONLY flag the caller acts on**, because an earlier version used `merged`
   and *"a guard returned early without clearing it, so a REFUSED branch still printed 'would
   delete'"*.

**A port that skipped any of those would be a weaker check wearing the command's name** — which is
what this port refuses everywhere else. **It stays delegated, and the dispatch site says so.**

## Verification

| check | result |
|---|---|
| the guard vs Python on a scratch repo with a dirty claim | **same claim survives** |
| four guard tests | pass |
| `cargo test -p uor-r4-harness` | **395 passed, 4 ignored** |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0 |

## Limitations, stated

- **`delete-merged` is still delegated, and so are `git install-hooks` and `git status`.** **Six
  `delegate_to_python` sites remain.**
- **The port's `--repo` acceptance is a divergence in its favour** — it makes documented invocations
  work that Python rejects. **A divergence is still a divergence**, and this one is recorded rather
  than celebrated quietly.
- **The guard preserves the claim by rewriting the file after the checkout.** **If the process dies
  between the checkout and the write, the claim is lost** — Python has the same window, and it is the
  window this command exists to close. **A `git stash`-based approach would be atomic and was not
  attempted**, because matching the reference implementation was the job here.
- **`checkout_preserving_claims` returns JSON and the caller formats it**, so the text form is the
  port's own. **The JSON is what was compared against Python's dict.**
