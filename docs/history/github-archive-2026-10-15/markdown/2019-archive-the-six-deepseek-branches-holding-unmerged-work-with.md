# #2019 — Archive the six DeepSeek branches holding unmerged work, with an index (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T13:18:11Z
- **Updated:** 2026-10-09T14:18:22Z
- **Closed:** 2026-10-09T13:28:25Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2019
- **Merged:** not merged
- **Merge commit:** e98d04f850f6972a7527fcc4a112e1ca61dec46e
- **Base / Head:** main / UOR-Foundation:deepseek/branch-archive
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Archive the six DeepSeek branches that still held unmerged work, with an index

Owner direction, 9 October 2026: origin/main is the shared source of truth, and work retained rather
than activated lands as INDEXED SOURCE PATCHES or a restorable source bundle on main. The canonical
DeepSeek lab is responsible for every deepseek/* branch that is not on main.

THE AUDIT, AND WHY THE OBVIOUS CHECK WAS WRONG
  The deepseek remote holds 146 branches. An ancestry test (git merge-base --is-ancestor) reports 141
  of them as unmerged, and a content diff reports almost all of them as differing. BOTH ARE WRONG:
  deliveries here are SQUASH-MERGED, so a branch tip is never an ancestor of main even when its content
  is fully present, and a content diff against a branch cut from an older main shows every unrelated
  change that landed afterwards.
  THE CORRECT INSTRUMENT IS PATCH-ID COMPARISON (git cherry), which compares what a commit DOES rather
  than where it sits. On that basis only SIX branches carried work not already on main, and the earlier
  reading would have had me re-merge 141 branches that are already delivered - exactly the
  "do not promote it by merging it" failure the direction warns against.

WHAT IS ARCHIVED HERE
  docs/archive/deepseek-branches-20261009/ holds one patch per branch, each the branch's own changes
  since its merge base, plus INDEX.txt listing every unmerged commit (hash, date, subject) and the
  patch identity. The six, dated 5-7 October: aa-dispatch (2 commits, an arithmetic reachability probe
  and an arithmetic oracle finding), bf16-phase2b-gate (3, the frozen Phase 2b read gate and handoff),
  d20-control (2, a parameter-matched control arm), d20-run (4, the D20 s2 pre-registration, run report
  and an INCONCLUSIVE verdict - it CONTAINS d20-control's commit, so the two overlap deliberately),
  entry-ceiling (4, an entry-ceiling mode and rank controls) and entry-isolation (2, an entry-only loss
  scope and a retraction).

STATUS, STATED RATHER THAN IMPLIED
  These are RETAINED, NOT ACTIVATED. The patches are an archive for restoration, not a merge of code
  into the live tree; nothing in them is promoted by being stored, and each can be applied from main
  with git apply when a successor needs it. Their branches can now be deleted without losing anything,
  which is the point of the exercise.

METHOD NOTE FOR ANYONE REPEATING THIS: use git cherry, not ancestry and not diffs. The same trap
applies to any claim that a branch "was never merged".

References #820


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6081868996 — 2026-10-09T13:28:24Z — Casey-allard

Superseded by #2021, which contains the same per-branch patches PLUS the verified restorable bundle. Keeping both open would make them conflict on the same files in docs/archive/deepseek-branches-20261009/ - the exact mechanism that turned the four direct-consolidation PRs DIRTY earlier today. #2021 is CLEAN and carries strictly more: the patches, the bundle, and the documented restoration procedure including the failed approaches and the size. References #820


---
_Comment/review ids in this file: 6081868996_

