# #2357 — D30 port 3: git clean-merged is Rust (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T18:09:23Z
- **Updated:** 2026-10-13T18:29:47Z
- **Closed:** 2026-10-13T18:28:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2357
- **Merged:** 2026-10-13T18:28:22Z
- **Merge commit:** 3184e8529cb4702874db0f74b0874c596908f64d
- **Base / Head:** main / UOR-Foundation:deepseek/port-gitmerge
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**D30 port 3.** Tooling only.

Decision: **KEEP** (tooling).

## What moved

`crates/uor-r4-harness/src/githelp.rs` — new, **11 tests**. `uor-harness git clean-merged` is Rust and **out of the delegated set** (verified: zero *"not ported"* lines). `install-hooks`, `status`, `switch`, `delete-merged` still delegate **by name**.

**This is the command every lab runs at merge time**, so it was the most-used Python path left.

## The safety property, and the three proofs that were tried

**A squash merge leaves no ancestry link**, so `git branch -d` *refuses* a branch whose every change landed — and this repository's queue squash-merges. **Two of the three content proofs tried on this line were wrong:**

| proof | verdict |
|---|---|
| `git rev-list` | fails on a squash merge, same reason as ancestry |
| `git diff main..branch` | **flags a branch that is merely behind** — the diff includes everything main gained since |
| **`git cherry origin/main <branch>`** | **correct**: `-` upstream, `+` not |

Ancestry first (cheap, covers a real merge commit), then `git cherry`. The squash case **and** the merely-behind case are each tested against **real git fixtures** — the second because that is the one the rejected proof got wrong.

## Shaped around a recorded failure

A guard once **reported "would delete" for a branch it had refused**: the caller set a flag, one guard set a reason but left the flag, and the summary read the flag.

**`removable` is one flag with one meaning, and every refusing path clears it** — a reason is advisory text no caller may read as permission. Tested directly: a non-removable state is not deleted even when handed to `apply`.

**A proof that did not run is not a proof.** With no `origin/main` every branch is `Unknown` and **nothing is removable** — tested, because defaulting to permission there is the failure mode that deletes work.

## Agreement with Python, on the real repository

```
removable     Rust ["worktree-agent-a496b1b9ce41d9752"]   Python ["worktree-agent-a496b1b9ce41d9752"]
checked out   Rust 4                                      Python 4, same names
unmerged      Rust 6                                      Python 6, same names
```

One presentational difference: `deepseek/port-gitmerge` is `merged + checked_out` in Rust, and appears only under Python's `checked_out`. **Both refuse to delete it**, which is the property that matters.

## A bug in my own test

It **created** a branch and passed the main checkout's `held` list, so `held` did not contain it and the test asserted against a state the guard was never given. **It failed, which is how I found it.** Fixed to actually check the branch out, with an assertion that the fixture holds it.

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 173 passed (11 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
uor-harness check                               -> 0 errors
Rust vs Python on the real repo                 -> same removable set, same states
delegation notice for `git clean-merged`        -> gone
```

Record: `docs/labs/port-gitmerge-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`switch` and `delete-merged` still run Python, and they are the risky ones.** `delete-merged` deletes **on the remote**; `switch` moves a checkout without losing an uncommitted claim. Neither should be ported until its guards come with it.
- **The `--json` shape differs from Python's.** Python emits `{removable, checked_out, unmerged, stale_worktree_refs}`; Rust emits a per-branch list. Same facts, different projection — a caller parsing Python's keys breaks.
- **`status.rs` still carries its own inline branch classification.** There are now two classifications in Rust, and while they agree today, **two implementations of one rule is the defect class this line keeps producing.** Consolidating is follow-up, recorded rather than claimed fine.
- **`has_prunable` is unused**, so a prunable worktree holding a branch shows only as "checked out in a worktree" with no hint that a prune would free it.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

