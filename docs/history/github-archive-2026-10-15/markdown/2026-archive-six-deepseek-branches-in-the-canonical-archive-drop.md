# #2026 — Archive six DeepSeek branches in the canonical archive; drop the duplicate (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T14:30:12Z
- **Updated:** 2026-10-09T14:37:03Z
- **Closed:** 2026-10-09T14:34:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2026
- **Merged:** not merged
- **Base / Head:** main / UOR-Foundation:deepseek/archive-unify
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Archive the six DeepSeek branches in the canonical branch archive, and drop the second archive

Owner direction, 9 October: origin/main is the single source of truth; reconcile older unmerged work
with explicit status; retained work lands as indexed source patches; branches are temporary transport.
Casey (via Claude) asked me to check the canonical index before adding anything so nothing is
archived twice - none of these six appeared there, so this adds them rather than duplicating work.

The six were found by PATCH-ID comparison (git cherry), not ancestry: deliveries here are
squash-merged, so a branch tip is never an ancestor of main even when its content is fully present,
and an ancestry test calls 141 of 146 deepseek branches unmerged when only six genuinely were.

  aa-dispatch        2 commits  arithmetic reachability probe; arithmetic oracle finding
  bf16-phase2b-gate  3 commits  frozen Phase 2b flash-read gate and handoff
  d20-control        2 commits  parameter-matched control arm (mqar-bench)
  d20-run            4 commits  D20 s2 pre-registration, run report, INCONCLUSIVE verdict
  entry-ceiling      4 commits  entry-ceiling mode, prediction-control panel, rank controls
  entry-isolation    2 commits  entry-only loss scope; retraction of a drowned-entry diagnosis

STATUS: RETAINED, NOT ACTIVATED. All six branches are deleted from GitHub and locally.
d20-control-land carried the same two commits as d20-control and is recorded as a duplicate.

The earlier second archive directory is removed so there is ONE index. Its one asset that must not be
lost is carried forward: the verified restorable bundle lives in iCloud rather than this repository,
because it is ~115 MB with the complete object closure and GitHub rejects files above 100 MB.

References #820


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6083038769 — 2026-10-09T14:34:14Z — Casey-allard

Superseded by the rebuilt PR, which is based on current main (8c78a27aa). This one went DIRTY when another lab landed overlapping archive edits in docs/history/branch-archive/. Rebuilding from fresh main rather than resolving the conflict keeps the archive unified in the canonical index and avoids a merge that silently drops the other lab's rows. References #820


---
_Comment/review ids in this file: 6083038769_

