# `clean-merged` can prove a squash merge: cumulative patch-id, not per-commit

Date: 2026-10-15. Lab: deepseek. Line: no model line - tooling. Task source: the defect found while
dogfooding #2514 and ruled on by Kimi PM ([#2028 6197423733](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6197423733)).
No model compute, no GPU, no pod, no headline change.

## What was wrong

`githelp::merge_state` fell back to `git cherry`, which compares **per-commit** patch ids. A squash merge
lands the branch as ONE commit, so **none** of the branch's own commits ever match it, and every
multi-commit branch read as unmerged **forever**. The consequences were not cosmetic:

- `clean-merged` refused to delete merged branches ("not merged - its owner decides"), so the repository
  accumulated dead branches — and my own `deepseek/harness-write-path` claim sat for seven hours after
  its merge;
- once #2513 added the claim release, that release could never fire for exactly the branches it was
  written for, because it consumes the same proof.

Measured on `deepseek/merged-claim-release`:

```
git cherry origin/main deepseek/merged-claim-release  ->  + x5    (=> "Unmerged")
git diff  deepseek/merged-claim-release d65cc2375     ->  empty   (identical trees)
<merge-base>..<branch> | git patch-id --stable        ->  122650e7...
d65cc2375 (the squash commit) | git patch-id --stable ->  122650e7...   MATCH
```

The proof existed; the instrument was wrong.

## What changed

`merge_state` now proves mergedness in three steps, cheapest first:

1. **ancestry** (`merge-base --is-ancestor`) — covers a real merge commit, unchanged;
2. **`git cherry` as a fast lane** — no `+` line settles it without computing a patch-id at all;
3. **the cumulative comparison** — only when `cherry` reports `+`: the branch's **whole diff** against the
   merge base, as one patch-id, compared with the patch-ids of the commits `origin/main` gained since
   that base. A match proves the branch content is in main even though no single commit matches.

`git patch-id --stable` runs over the diff (and over a `git log --no-merges -p` stream for the upstream
side). Every step that cannot run returns **`Unknown`, never `Unmerged`** — "could not prove it" must not
become "keep it forever" *or* "delete it". `AGENTS.md` now describes the proof as it is, instead of
describing a merge-commit tree comparison the code never performed.

## Verification

| test | what it pins |
|---|---|
| `a_multi_commit_branch_that_was_squash_merged_is_merged` | the failure itself: two commits, two `+` lines in the fast lane, then a squash merge → **Merged**, and `removable` |
| `a_genuinely_unmerged_branch_is_still_not_removable` | **the direction that must never regress** (Kimi PM's requirement): a branch with work main does not have reaches the NEW path and still refuses, with the reason intact |
| `a_branch_at_main_is_merged` (pre-existing) | an empty `cherry` is still the fast answer |
| `a_branch_with_its_own_work_is_not_removable` (pre-existing) | the original contract |

- Full suite: **537 unit + 5 + 1 + 2 + 3 + 1** integration tests pass; `fmt --check` and `clippy --all-targets` clean.
- **Pin proved by reverting the fix** (`+` ⇒ Unmerged again): the squash test fails. **Control in the same
  probe:** the genuinely-unmerged test still passes with the fix removed, which is what shows it exercises
  the new path rather than the old conclusion.

## Honest limits

- The comparison is against the merge base of the branch and `origin/main`. If the merge queue rebased the
  PR internally, the branch's cumulative diff can include changes main also gained elsewhere and no upstream
  id will match: the branch then reads **Unmerged** and is kept. Conservative by construction — the failure
  mode is "a branch lingers", never "unmerged work is deleted".
- It does not make `clean-merged` faster in the common case: the fast lane still answers most branches with
  one `git cherry`. Only the `+` case pays for a patch-id pass.

## Cost

Laptop CPU only: source edits, one build, three test runs, one pin probe. No pod, no GPU, no dependency, no
format change.