# #2445 — Port the worktree half of git status; not wired (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T05:21:59Z
- **Updated:** 2026-10-14T05:24:31Z
- **Closed:** 2026-10-14T05:24:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2445
- **Merged:** 2026-10-14T05:24:07Z
- **Merge commit:** c6c48e80b921afdafbf0c9adff23dbf93e4b58ae
- **Base / Head:** main / UOR-Foundation:deepseek/git-status
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`git status`: the worktree half, ported whole and not wired.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

**`worktree_facts`** — `githelp._worktree_facts`, ported whole with its counts, its cap, and its unavailable path.

**Counts verified against Python on the same tree**: `{'total': 5, 'staged': 2, 'unstaged': 1, 'untracked': 2}` for `A .gitignore`, ` M a.txt`, `D  b.txt`, `?? b.txt`, `?? untracked.txt`.

## It is NOT wired, and the reason is honest

**`git status` is ~220 lines of Python across five functions** — `status` (99), `_pr_facts` (40), `_next_command` (42), `_delivery_state` (19), plus the worktree half. **A full port is ~400 lines of Rust.**

**I ported the one piece that is complete and verifiable here** rather than a partial `status` that would print **fewer facts than Python's while looking finished** — **the exact failure this port spent three units finding**, most recently `cmd_check` running one rule of forty-one.

**So `git status` still delegates, `worktree_facts` has no caller, and the next unit finishes it. This unit is a well-tested part, not a delivery.**

## The asymmetry copied rather than corrected

Porcelain v1 is **`XY path`**:

* **staged** — the first column, excluding `?` and `!`: **an untracked or ignored file is not a staged change.**
* **unstaged** — the second column, excluding ` ` and `?` but **NOT `!`** — **a real asymmetry with the first.**
* **untracked** — `startswith("??")`.

**The asymmetry is copied, not fixed**, because **`check` and the delivered reports quote these counts, and "correcting" it would change numbers labs have already read.** A port that silently improved a count would be a divergence nobody asked for.

**The counts do not sum to `total`** — `b.txt` is both a staged delete and untracked, so it is counted twice. **Python's behaviour**: the counts answer *"how many files are in this state"*, not *"how many files are there."*

## Four tests

| test | asserted |
|---|---|
| counts match Python on a mixed tree | `5/2/1/2`, all five lines listed |
| a clean tree says so | `clean: Some(true)`, zero counts, no lines |
| **an unreadable repo is unavailable, NOT dirty** | `available: false`, **`clean: None`** |
| the list is capped but the total is not | `total == 25`, `lines.len() == 20`, each `<= 120` chars |

**`clean: None` is why the field is an `Option`** — a tree that cannot be read is **not** a dirty one, and **a `bool` would report a broken checkout as dirty.**

**The cap test matters for the same reason**: **a cap without the true count is a report that lies by omission.**

```
cargo test -p uor-r4-harness                    -> 406 passed, 4 ignored
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0
```

Record: `docs/labs/worktree-facts-port-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`git status` still delegates; `worktree_facts` is a tested library with no caller** — as `githubcards`, `github_issues` and `checkdispatch` were before their wiring landed.
- **What is left is the larger half**: `_pr_facts`, `_delivery_state`, `_next_command`, and the JSON/text assembly. **`_next_command` carries the most judgement — it decides what a lab is told to do next.**
- **The port reads `git status --porcelain` with no `-z`**, so **a filename containing a newline is split into two lines and counted twice.** Python has the same defect and **it is real**. `-z` would fix it **and would change the counts**, so it is named as a known-bad case rather than fixed in a parity port.
- **`untracked` excludes ignored files only because porcelain omits them by default** — with `--ignored` the same prefix test counts `!!` lines as untracked. **The port never passes `--ignored`, so it cannot happen today**, and the fragility is inherited.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

