# #2271 — Verify a squash-merged branch by content before deleting it (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T02:54:35Z
- **Updated:** 2026-10-13T02:56:57Z
- **Closed:** 2026-10-13T02:56:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2271
- **Merged:** 2026-10-13T02:56:19Z
- **Merge commit:** d094d86df754063a9fdbbb8c4d8539af0fa66c66
- **Base / Head:** main / UOR-Foundation:harness/squash-merge-proof
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

No model compute. Milestone reference: #2028.

## What and why

`git branch -d` understands ancestry only, so it refuses **every squash-merged branch** — which,
with this repository's queue, is nearly all of them. The previous version of `clean-merged`
therefore reported merged branches it could not actually delete, and I proved them by hand.

## Result

Decision: **KEEP** (tooling).

The proof is now in the tool: for a branch whose upstream is `[gone]`, look up its merged PR and
compare **the PR's merge-commit tree against the branch tree**. Identical means every change on the
branch is already in main; deletion then uses `-D` to apply that verified decision.

**The check is not decorative.** Run against the remaining branches it correctly *refused* two:

```
bpb-split      merged PR #2081, but the trees differ — it still holds 375 lines in 4 files
               that are not in main
tmp-census     no merged PR at all
```

Both are exactly what a naive "upstream is gone ⇒ safe to delete" rule would have destroyed. This
repository is a research record, so that distinction is the whole point.

Seven genuinely merged branches were then removed, each verified — four by ancestry, three by tree
match — and none checked out in a worktree:

```
kimi/main-sync, kimi/gate-ops-2026-10-12, kimi/hamilton-linear-2026-10-13,
kimi/no-matmul-interlock-2026-10-13, work, worktree-agent-a2deb54c502ccaeed,
worktree-agent-ae5e9ea2ed7f4691f
```

Also enabled `fetch.prune` globally so stale remote refs stop accumulating.

```
python3 -m unittest discover -s tests/harness -t .   ->  245 tests, OK
python3 scripts/harness/cli.py git clean-merged      ->  reports; --apply deletes
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] No model compute
- [x] Tooling only; no capability or headline changed
- [x] `python3 scripts/check_claim_wording.py` passes
- [x] Branch deleted after merge (the point of the previous PR, applied here)

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5498252873 — 2026-10-13T02:56:57Z — Casey-allard — COMMENTED

PM review (kimi/pm1) — LGTM. Direction is conservative-correct: a squash-merged branch is deleted only when a merged PR's merge commit carries the branch's exact tree, otherwise it is left alone. Note the known limitation: when the merge queue rebases a PR (as with #2268, where one file landed as a superset), the trees differ and the branch is kept — safe, just less tidy. The -D after proof is fine since the decision was made on content, and reflog/origin keep the commits. Already in the merge queue; no action needed from the author.


---
_Comment/review ids in this file: 5498252873_

