# `multilab.claim_not_merged` is Rust — and its documented fallback does not do what it says

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028; `check` module 3 completed

## The rule, and the ordering that had to be preserved

`multilab.claim_not_merged` — **2 findings in Rust, 2 in Python.** Both committed claims
(`codex/count-reference-closeout-20261013`, `kimi/d29-nvrtc-amendment`) have **a merged PR and no
local ref**, so **the GitHub-first path produced both findings in both implementations.**

The ordering is not an optimisation, it is a bug fix, and the port keeps it:

- **GitHub is asked FIRST.** Content comparison first lies in the dangerous direction: a lab that has
  just claimed a branch and pushed it may not have it in local remote-tracking refs yet, so the branch
  looks **gone** — and the first version treated *gone* as *merged*, which would tell a lab to release
  a claim on **unpushed work**. The Python records this as a real false positive on live work
  (`kimi/plan-refresh-2026-10-13`), because **a rule that cries wolf on a lab's live branch is worse
  than no rule.**
- **The answer is three-valued.** `Some(true)` merged, `Some(false)` evidence of not-merged,
  **`None` not established** — and the rule reports only on `Some(true)`. **Collapsing `None` into
  `false` is how a check starts guessing**, and `None` is asserted distinct in the tests.

## What I got wrong in the report, and how it was caught

**I wrote "GitHub unreachable" and "the fallback" as though I had measured them. I had not.** Casey
asked what I meant. Checking:

- **`gh auth status`: logged in, exit 0.** GitHub was reachable throughout.
- The probe's `gh pr list` returned **`[]` with exit 0** — an answer, not a failure. I had picked a
  branch name with no PR, so of course it was empty.
- **I was paraphrasing the Python docstring's fallback and narrating a code path as if it were a
  result.** It landed directly under a probe's output, which made it read as an observation.

**I then did the thing I should have done first: tested the path instead of describing it.**

## The fallback does not work for this rule, and that is now measured

With **`gh` absent from `PATH`**:

```
codex/count-reference-closeout-20261013  ->  None
zzz/never-existed-at-all                 ->  None
```

**Both `None`.** The fallback needs a **visible ref** to compare against — and **the project's rule is
to delete the branch after merging.** Neither live claim has a local or remote-tracking ref, so **with
GitHub down this rule reports nothing at all.**

So the fallback is **not a safety net for this rule**; it helps only the narrow case of a branch that
was merged **and kept**. That is confirmed from the other side too: the content path *does* fire when a
ref exists (`a_branch_with_nothing_ahead_of_main_is_reported` passes), so the fallback is not dead code
— it is simply inapplicable to how this project actually works.

**The comment described a general offline path. The measurement says otherwise.** Both are now in the
code where the next reader of that comment will find them.

## The lesson, stated plainly

**Acting on a docstring is not the same as knowing.** I read a comment about a fallback and reported it
as a live concern twice — once in a plan, once in a probe header — without ever running it. **A code
path that has never run is not a feature, it is a comment**, and the only reason this surfaced is that
the owner asked a question about a phrase I had used loosely.

## Verification

| check | result |
|---|---|
| Rust vs Python, `claim_not_merged` | **2 = 2**, measured on the real repo |
| `gh` reachability during measurement | authenticated, exit 0 — **the fallback never ran** |
| fallback with `gh` absent and no ref | `None`, `None` |
| fallback with a ref present | fires correctly (test passes) |
| `Some(true)`/`Some(false)`/`None` distinction | pinned by test |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **239 passed** |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `parity.py` | exit 0 |

## Limitations, stated

- **Two `multilab` rules remain** (`no_shared_ledger`, `ledger_lab_recorded`), deliberately deferred to
  the `receipts` module since they read receipt shards.
- **`check` still delegates.** 3 of 10 modules complete; coverage **12 of 40 rules**.
- **The agreement measurement is a hand-run probe**, read and removed. **Nothing in CI compares
  per-rule finding counts.**
- **The `gh` dependency is a shell-out with no timeout.** Python passes `timeout=20` to its `git`
  calls; the Rust `gh` and `git` calls here have **no timeout**, so a hung `gh` would hang `check`.
  That is a real difference between the implementations and it is not fixed here.
