# `git status` — the worktree half, ported whole and not wired

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** `git status`, the largest remaining git action

## What landed

**`worktree_facts`** — `githelp._worktree_facts`, ported whole with its counts, its cap, and its
unavailable path.

**Counts verified against Python on the same tree**: `{'total': 5, 'staged': 2, 'unstaged': 1,
'untracked': 2}` for the porcelain lines `A .gitignore`, ` M a.txt`, `D  b.txt`, `?? b.txt`,
`?? untracked.txt`.

## It is NOT wired, and here is the honest reason

**`git status` is ~220 lines of Python across five functions** — `status` (99), `_pr_facts` (40),
`_next_command` (42), `_delivery_state` (19), plus the worktree half. **A full port is ~400 lines of
Rust.**

**I ported the one piece that is complete and verifiable in this unit** rather than a partial
`status` that would print fewer facts than Python's **while looking finished** — **which is the exact
failure this port spent three units finding**, most recently in `cmd_check` running one rule of
forty-one.

**So `git status` still delegates, `worktree_facts` has no caller, and the next unit finishes the
job.** That is stated rather than dressed up: **this unit is a well-tested part, not a delivery.**

## The three counts, and the asymmetry that is copied rather than corrected

Porcelain v1 is two columns then a space — **`XY path`**:

* **staged** — `l[:1] not in (" ", "?", "!")`: **the first column is the index**, and `?`/`!` there
  mean untracked/ignored, **so neither is a staged change.**
* **unstaged** — `l[1:2] not in (" ", "?")`: **the second column is the worktree**, and **`!` is NOT
  excluded here** — **a real asymmetry with the first column.**
* **untracked** — `startswith("??")`, both columns at once.

**The asymmetry is copied, not fixed.** An ignored file cannot be staged, **but the worktree column of
an ignored entry is not a modification either** — and **`check` and the delivered reports quote these
counts, so "correcting" it would change numbers labs have already read.** A port that silently
improved a count would be a divergence nobody asked for.

**The counts do not sum to `total`, and the fixture shows why**: `b.txt` is both a staged delete and
an untracked file, so it is counted twice. **That is Python's behaviour** — the counts answer *"how
many files are in this state"*, not *"how many files are there"*.

## Four tests

| test | asserted |
|---|---|
| the counts match Python on a mixed tree | `5/2/1/2`, and **all five lines listed** |
| a clean tree says so | `clean: Some(true)`, zero counts, no lines |
| **an unreadable repo is unavailable, NOT dirty** | `available: false`, **`clean: None`** |
| the line list is capped but the total is not | `total == 25`, `lines.len() == 20`, each `<= 120` chars |

**`clean: None` is why the field is an `Option` rather than a `bool`.** A tree that cannot be read is
**not** a dirty one, and **a `bool` would report a broken checkout as dirty** — the same
"unavailable is not a pass, and it is not a failure either" distinction this port has had to fix
before.

**The cap test matters for the same reason**: **a cap without the true count is a report that lies by
omission**, so `total` counts every line while `lines` shows twenty.

## Verification

| check | result |
|---|---|
| counts vs Python on the same tree | **identical** |
| four tests | pass |
| `cargo test -p uor-r4-harness` | **406 passed, 4 ignored** |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0 |

## Limitations, stated

- **`git status` still delegates, and `worktree_facts` is a tested library with no caller** — reachable
  from tests and from nothing else, **as `githubcards`, `github_issues` and `checkdispatch` were before
  their wiring landed.**
- **What is left is the larger half**: `_pr_facts` (a `gh pr view` with eight fields and a
  merge-commit-vs-PR-head containment check), `_delivery_state`, `_next_command`, and the JSON/text
  assembly. **`_next_command` is the part with the most judgement in it** — it decides what a lab is
  told to do next.
- **The port reads `git status --porcelain` with no `-z`**, so **a filename containing a newline is
  split into two lines and counted twice.** Python has the same defect, **and it is real**: a path with
  a newline is legal on every platform this runs on. **`-z` plus NUL splitting would fix it and would
  change the counts**, so it is named as a known-bad case rather than fixed in a port whose job is
  parity.
- **`untracked` excludes ignored files only because porcelain omits them by default** — with
  `--ignored` the same code counts `!!` lines as untracked, since the check is a prefix test. **The
  port never passes `--ignored`, so it cannot happen today**, and the fragility is inherited.
