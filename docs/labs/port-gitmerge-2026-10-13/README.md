# D30 port 3: `git clean-merged` is Rust

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028; port 3 of the series

## What moved

`crates/uor-r4-harness/src/githelp.rs` — new, **11 tests**. `uor-harness git clean-merged` is Rust and
**out of the delegated set** (verified: zero *"not ported"* lines). `install-hooks`, `status`,
`switch` and `delete-merged` still delegate **by name**.

**This is the command every lab runs at merge time**, so it is the most-used Python path that was
left.

## The safety property, and the three proofs that were tried

**A squash merge leaves no ancestry link.** `git branch -d` only understands ancestry, so it *refuses*
a branch whose every change landed — and this repository's queue squash-merges. Three content proofs
were tried on this line and **two were wrong**:

| proof | verdict |
|---|---|
| `git rev-list` | fails on a squash merge, same reason as ancestry |
| `git diff main..branch` | **flags a branch that is merely behind** — the diff includes everything main gained since |
| **`git cherry origin/main <branch>`** | **correct**: `-` means upstream, `+` means not |

The port uses ancestry first (cheap, covers a real merge commit), then `git cherry`. Both are tested
against a **real squash-merged fixture**, and the merely-behind case is tested separately because that
is the one the rejected proof got wrong.

## The rule this module is shaped around

A guard once **reported "would delete" for a branch it had refused**: the caller set a flag, one guard
set a reason but left the flag, and the summary read the flag.

**`removable` is that flag, and every refusing path clears it.** No guard sets a reason "instead" —
a reason is advisory text that no caller can read as permission. Tested directly: a state that is not
removable is not deleted even when handed to `apply`.

**A proof that did not run is not a proof.** With no `origin/main`, every branch is `Unknown` and
**nothing is removable** — that path is tested, because defaulting to permission there is the failure
mode that deletes work.

## Agreement with Python, on the real repository

Not a synthetic fixture — the actual checkout:

```
removable          Rust: ["worktree-agent-a496b1b9ce41d9752"]   Python: ["worktree-agent-a496b1b9ce41d9752"]
checked out        Rust: 4 (held)                               Python: 4 (same names)
unmerged           Rust: 6                                       Python: 6 (same names)
```

One presentational difference: `deepseek/port-gitmerge` is `merged + checked_out` in Rust and appears
only under Python's `checked_out`. Both refuse to delete it, which is the property that matters.

## A bug in my own test

`a_checked_out_branch_is_not_removable_and_says_why` **created** a branch and passed the main
checkout's list, so `held` did not contain it and the test asserted against a state the guard was
never given. **It failed, which is how I found it** — but it would have passed if I had also asserted
`removable` from the wrong direction. Fixed to actually check the branch out, with an assertion that
the fixture holds it.

## Verification

| check | result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **173 passed** (11 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `uor-harness check` | 0 errors |
| Rust vs Python on the real repo | **same removable set, same states** |
| delegation notice for `git clean-merged` | gone |

## Limitations, stated

- **`switch` and `delete-merged` still run Python, and they are the risky ones.** `delete-merged`
  deletes **on the remote as well**; `switch` moves a checkout without losing an uncommitted claim.
  Neither is ported, and neither should be until its guards come with it.
- **The `--json` shape differs from Python's.** Python emits `{removable, checked_out, unmerged,
  stale_worktree_refs}`; Rust emits a per-branch list. Same facts, different projection — a caller
  parsing Python's keys breaks.
- **`status.rs` still carries its own inline branch classification.** There are now two
  classifications in Rust, and while they agree today, **two implementations of one rule is the
  defect class this line keeps producing.** Consolidating them is follow-up, and I am recording it
  rather than claiming it is fine.
- **`stale_worktree_refs` / prunable worktrees are detected but not reported.** `has_prunable` exists
  and is unused, so a prunable worktree holding a branch still shows as "checked out in a worktree"
  with no hint that a prune would free it.

## A fix I reported as delivered that never landed

**Port 2's CI failed, I fixed it, reported it delivered — and it was not on `main`.** `main` still
carried `date -r` in the `multilab` test helper, so **CI kept failing on the same test**, and this
port's first CI run failed for the port-2 bug.

What happened: the portability fix was a **second commit** on the port-2 branch
(`4b25c508d`, "Fix the BSD-only test helper"), pushed after the merge had already been queued. I then
deleted the branch. **Neither fix commit is an ancestor of `origin/main`** — verified with
`git merge-base --is-ancestor` — so the squashed merge landed the first commit's tree and not the fix.

**The mistake was mine and it was in the verification step, not the fix:** after merging I verified
that "the portability fix is on main" by grepping for a pattern that matched nothing either way, and
read the empty result as success. A grep with no matches looks exactly like a grep that passed.

**The rule this adds:** after a merge, verify the **specific hunk** is present, not that a keyword is
absent. `grep -cF '"-r"'` returning 0 and `grep -cF 'div_euclid'` returning 0 are not the same
result, and I treated them as the same.

The fix is now in this PR, in a commit whose file set is checked against `main` rather than assumed.
