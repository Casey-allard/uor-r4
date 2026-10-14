# `git status` — ported whole, and the three `reason` strings that took four attempts

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the largest remaining git action, finished from the worktree half last unit

## What landed

**`uor-harness git status` is Rust** — worktree facts, the PR, the delivery state and the next
command — **and its `--json` matches Python's on every field.**

| | |
|---|---|
| keys | **30 both sides** |
| **differences** | **0** |
| state on the same tree | `dirty-worktree` both |

**Delegation sites: 6 → 5.**

## Four attempts to get three strings right

**The structure was right on the first attempt. The rendering was not, and every round of the
comparison found one more thing** — which is the case for comparing output rather than reading code:

| attempt | difference | what it was |
|---|---|---|
| 1 | `.reason` ×3 | Python wraps failures as `` `cmd` exited N: … `` and I passed raw stderr |
| 2 | `fetch.reason` | **a blank line becomes TWO SPACES** — Python strips then replaces `\n` with `" "`, **rather than joining non-empty lines**, which is what I had done |
| 2 | `origin_main.reason` | **STDERR FIRST, then stdout** — the reverse of the obvious order, **and for `git rev-parse origin/main` on an ambiguous ref stdout holds the ref while stderr holds the reason**, so the wrong order prints `origin/main` in front of the sentence explaining it |
| 3 | `origin_main.reason` | **`git_raw` concatenates stdout-then-stderr**, which is right for *reading an answer* and wrong for *rendering a failure* — so `git_run` was added rather than changing a dozen callers |
| 4 | **0 differences** | |

**`run_failure` now reproduces Python's `_run` character for character**: strip the combined text,
replace newlines with spaces, **redact**, **truncate to 240**, and say `no output` when there is none.

**`gh_json` had the same defect** — it returned trimmed stderr — so the fix was applied there too,
which is why a `gh` failure now reads `` `gh pr view …` exited 1: no git remotes found ``.

## The delivery state table, and the order that is the logic

**A dirty worktree outranks everything**, because a lab with uncommitted changes has a next action
that does not depend on any PR — **asking GitHub first answers the wrong question.**

**Three merged states, not two**: `merged-verified`, `merged-unverified`, and **`merged-unknown`** for
when `origin/main` could not be resolved. **Collapsing the third into either of the others is how a lab
is told to delete a branch whose merge was never verified.**

## Two command tables that tell a lab NOT to act

The port keeps both, because **they are the two a port is most tempted to simplify into a command**:

* **`merged-unverified`** → *"keep the branch and worktree until that is explained; **do not delete
  anything yet**"*, **and it is the one state where `git status` exits 1.**
* **an unrecognised state** → *"report this output **rather than acting on it**."*

**Five tests**, including that the two merged states meaning *finished* share a cleanup command while
`merged-unverified` does not, and that an empty branch name renders `<branch>` rather than
`git branch -D ` with nothing after it.

## Verification

| check | result |
|---|---|
| `git status --json` vs Python, field for field | **0 differences across 30 keys** |
| five status tests | pass |
| `cargo test -p uor-r4-harness` | **411 passed, 4 ignored** |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0 |

## Limitations, stated

- **The TEXT output is not compared against Python's**, only the JSON. **The text form is the port's
  own in places** — it interpolates the same fields but the line assembly was written to match Python
  by reading, not by diffing. **A lab reading `git status` sees the port's rendering**, and that is
  recorded rather than claimed identical.
- **`git_run` and `git_raw` now both exist and differ only in output order.** That is deliberate and
  documented, **but it is a footgun for the next caller**: picking the wrong one produces a correct
  answer with a wrong failure message, which is exactly the class of bug this unit spent four attempts
  on.
- **`pr_facts` calls `gh pr view` with no branch argument**, so it reports the PR for the *checked-out*
  branch. On a detached HEAD it reports whatever `gh` decides, **which is Python's behaviour and is not
  verified** — the comparison ran on an attached branch.
- **`git status` still fetches `origin`** on every invocation, so **a slow or unreachable remote makes
  the command slow.** Python does the same, and the failure is reported rather than fatal.
- **`git install-hooks` and `git status` were separate delegation sites**; `status` was the last git
  action, so **the `git` group is fully Rust except nothing** — the remaining five sites are
  `mistakes show|capture`, `multilab sync-live|migrate-ledger`, and `team roster|dispatch`.
