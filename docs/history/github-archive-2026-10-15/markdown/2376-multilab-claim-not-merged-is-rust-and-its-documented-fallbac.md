# #2376 — multilab.claim_not_merged is Rust - and its documented fallback does not do what it says (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T20:46:36Z
- **Updated:** 2026-10-13T20:49:20Z
- **Closed:** 2026-10-13T20:49:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2376
- **Merged:** 2026-10-13T20:49:01Z
- **Merge commit:** abf80d7b493a843b26d6aff30905f5d084081b5e
- **Base / Head:** main / UOR-Foundation:deepseek/claim-not-merged
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`multilab.claim_not_merged`.** Tooling only.

Decision: **KEEP** (tooling).

## The rule, and the ordering that had to be preserved

**2 findings in Rust, 2 in Python.** Both committed claims have **a merged PR and no local ref**, so **the GitHub-first path produced both findings in both implementations.**

The ordering is not an optimisation, it is a bug fix:

- **GitHub is asked FIRST.** Content comparison first lies in the dangerous direction: a lab that has just claimed a branch and pushed it may not have it in local remote-tracking refs yet, so the branch looks **gone** — and the first version treated *gone* as *merged*, which would tell a lab to release a claim on **unpushed work**. The Python records a real false positive on live work, because **a rule that cries wolf on a lab's live branch is worse than no rule.**
- **The answer is three-valued.** `Some(true)` merged, `Some(false)` evidence of not-merged, **`None` not established** — reported only on `Some(true)`. **Collapsing `None` into `false` is how a check starts guessing.**

## What I got wrong, and how it was caught

**I wrote "GitHub unreachable" and "the fallback" as though I had measured them. I had not** — the owner asked what I meant, and checking:

- **`gh auth status`: logged in, exit 0.** GitHub was reachable throughout.
- The probe's `gh pr list` returned **`[]` with exit 0** — an answer, not a failure. I had picked a branch name with no PR.
- **I was paraphrasing the Python docstring's fallback and narrating a code path as if it were a result.** It landed directly under a probe's output, which made it read as an observation.

**I then tested the path instead of describing it — and the fallback does not work for this rule.**

## The fallback is measured, and it does not do what its comment says

With **`gh` absent from `PATH`**:

```
codex/count-reference-closeout-20261013  ->  None
zzz/never-existed-at-all                 ->  None
```

The fallback needs a **visible ref** to compare against — and **the project's rule is to delete the branch after merging.** Neither live claim has a ref, so **with GitHub down this rule reports nothing at all.** It helps only the narrow case of a branch merged **and kept**, confirmed from the other side because the content path **does** fire when a ref exists.

**The comment described a general offline path; the measurement says otherwise.** Both are now in the code beside that comment.

> **Acting on a docstring is not the same as knowing. A code path that has never run is not a feature, it is a comment.**

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 239 passed
python3 -m unittest discover -s tests/harness   -> 319 passed
parity.py                                       -> exit 0
```

Record: `docs/labs/claim-not-merged-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **Two `multilab` rules remain** (`no_shared_ledger`, `ledger_lab_recorded`), deferred to the `receipts` module.
- **`check` still delegates.** 3 of 10 modules; coverage **12 of 40 rules**.
- **The agreement measurement is a hand-run probe**, read and removed — nothing in CI compares per-rule finding counts.
- **The `gh` and `git` shell-outs here have no timeout**, while Python passes `timeout=20`. **A hung `gh` would hang `check`.** That is a real difference between the implementations and it is not fixed here.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

