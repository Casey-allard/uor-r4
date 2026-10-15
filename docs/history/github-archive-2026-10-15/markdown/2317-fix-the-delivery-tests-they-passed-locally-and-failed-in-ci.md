# #2317 — Fix the delivery tests: they passed locally and failed in CI (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T07:20:13Z
- **Updated:** 2026-10-13T07:25:43Z
- **Closed:** 2026-10-13T07:24:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2317
- **Merged:** 2026-10-13T07:24:20Z
- **Merge commit:** 74bec5ef2704da6cd038dc0b6a2a1356dcce5cf8
- **Base / Head:** main / UOR-Foundation:harness/fix-ci-shallow
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only. **This fixes a red CI job on `main`.**

## Result

Decision: **KEEP** (tooling).

## What happened

#2316 merged while its `Rust harness and Python parity` job was **RED**, because only
`required-transport` is a required check. Four tests died with:

```
UNAVAILABLE: cannot resolve origin/main: fatal: ambiguous argument 'origin/main'
```

**GitHub Actions checks out shallow**, so there is no remote-tracking ref. My tests called
`delivery_view` against whatever repository they happened to be running in — which, locally, is a full
clone with `origin/main`.

**They asserted on live state and therefore tested the machine, not the code.** Green locally, red in
CI — and that gap *is* the bug.

## The fix

**A real git fixture.** `Fixture` builds a scratch repository with a known `origin/main` and branches
the test controls: one finished, one with new work, one checked out in a **dirty worktree**. Not a
mock — the code under test shells out to git, and a mock of git would only test the mock.

**Plus a test for the CI condition itself:** with no `origin/main`, the view must report `UNAVAILABLE`
and judge **nothing**. It asserts the render does *not* print "none safe to delete", because **a clean
bill of health from a view that could not look is the worst possible output.**

## Verified under CI's exact condition, not just locally

```
$ git clone --depth 1 ... && git update-ref -d refs/remotes/origin/main
$ cargo test -p uor-r4-harness
  test result: ok. 85 passed; 0 failed
```

That is the check that was missing the first time.

```
cargo fmt / clippy -D warnings        -> clean, 0 findings
cargo test -p uor-r4-harness          -> 85 passed + 1 doc-test, AND 85 passed with no origin/main
python3 -m unittest                   -> 315 passed
python3 scripts/harness/cli.py check  -> 52 rules, 0 errors
```

## Checklist

- [x] Checks run at this exact head (above), including the CI condition reproduced locally
- [x] Tooling only; no capability or headline changed
- [x] Branch deleted after merge

### Stated rather than hidden

**I merged #2316 with a red required-adjacent job.** I polled `gh pr checks` before merging, saw
`pending`, waited, and merged on "MERGED" without re-reading the final check states. The merge queue
let it through because that job is not required. That is the same shape as the mistake this whole
session started with — **acting on a loop that ended rather than on the state I actually needed** —
and it is the second time it has appeared in a different place. I have not added a guard for it;
`prlint` does not read check states, and inventing a rule that cannot see them would be a rule that
checks nothing. Recorded here so it is visible rather than silently repaired.

- The fixture uses `tempfile` as a dev-dependency, so `Cargo.lock` changes.
- Four tests still run against the live checkout (`status::tests::*`) — they assert structural
  properties that hold in any repository. If they ever depend on a branch name, they will fail in CI
  the same way.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

