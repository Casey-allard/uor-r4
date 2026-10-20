# #2446 — Port git status whole; the git group is fully Rust (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T05:33:29Z
- **Updated:** 2026-10-14T05:36:05Z
- **Closed:** 2026-10-14T05:35:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2446
- **Merged:** 2026-10-14T05:35:39Z
- **Merge commit:** c866ceea78eb847354f8736286d8560648d0f9cc
- **Base / Head:** main / UOR-Foundation:deepseek/git-status2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`git status` ported whole; the `git` group is fully Rust.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

**`uor-harness git status` is Rust** — worktree facts, the PR, the delivery state and the next command — **and its `--json` matches Python's on every field**:

| | |
|---|---|
| keys | **30 both sides** |
| **differences** | **0** |
| state on the same tree | `dirty-worktree` both |

**Delegation sites: 6 → 5.**

## Four attempts to get three strings right

**The structure was right on the first attempt; the rendering was not.** Every round of the comparison found one more thing — **which is the case for comparing output rather than reading code**:

| attempt | difference | what it was |
|---|---|---|
| 1 | `.reason` ×3 | Python wraps failures as `` `cmd` exited N: … ``; I passed raw stderr |
| 2 | `fetch.reason` | **a blank line becomes TWO SPACES** — Python strips then replaces `\n` with `" "`, **rather than joining non-empty lines** |
| 2 | `origin_main.reason` | **STDERR FIRST, then stdout** — the reverse of the obvious order, **and for `git rev-parse origin/main` on an ambiguous ref stdout holds the ref while stderr holds the reason**, so the wrong order prints `origin/main` in front of the sentence explaining it |
| 3 | `origin_main.reason` | **`git_raw` concatenates stdout-then-stderr** — right for *reading an answer*, wrong for *rendering a failure* — so `git_run` was added rather than changing a dozen callers |
| **4** | **0** | |

**`run_failure` reproduces Python's `_run` character for character**: strip, replace newlines with spaces, **redact**, **truncate to 240**, `no output` when empty.

**`gh_json` had the same defect** — it returned trimmed stderr — so a `gh` failure now reads `` `gh pr view …` exited 1: no git remotes found ``.

## The delivery state table, and the order that is the logic

**A dirty worktree outranks everything** — a lab with uncommitted changes has a next action that does not depend on any PR, so **asking GitHub first answers the wrong question.**

**Three merged states, not two**: `merged-verified`, `merged-unverified`, **`merged-unknown`** for when `origin/main` couldn't be resolved. **Collapsing the third is how a lab is told to delete a branch whose merge was never verified.**

## Two tables that tell a lab NOT to act

Kept, because **they are the two a port is most tempted to simplify into a command**:

* **`merged-unverified`** → *"keep the branch and worktree until that is explained; **do not delete anything yet**"* — **and it is the one state where `git status` exits 1.**
* **an unrecognised state** → *"report this output **rather than acting on it**."*

```
cargo test -p uor-r4-harness                    -> 411 passed, 4 ignored
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0
```

Record: `docs/labs/git-status-port-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **The TEXT output is not compared against Python's, only the JSON.** The text form is the port's own in places — **a lab reading `git status` sees the port's rendering**, recorded rather than claimed identical.
- **`git_run` and `git_raw` now both exist and differ only in output order.** Deliberate and documented, **but a footgun: picking the wrong one gives a correct answer with a wrong failure message** — exactly the class of bug this unit spent four attempts on.
- **`pr_facts` calls `gh pr view` with no branch argument**, so on a detached HEAD it reports whatever `gh` decides. **Python's behaviour, and not verified** — the comparison ran on an attached branch.
- **`git status` still fetches `origin` every invocation**, so a slow remote makes it slow. Python does the same; the failure is reported rather than fatal.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

