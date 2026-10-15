# #1277 — Diagnose unseen-neighbor role transfer on frozen geometric model

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-14T16:37:17Z
- **Updated:** 2026-10-09T13:17:45Z
- **Closed:** 2026-09-14T16:38:46Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1277
- **Merged:** 2026-09-14T16:38:46Z
- **Merge commit:** 6ce5a5e2b441925b6dc388364b1907a04c943498
- **Base / Head:** main / UOR-Foundation:codex/neighbor-role-transfer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Test the qualified 0175ad86 artifact on unfamiliar neighboring names at the existing grammar, context and depth limits. The unchanged-artifact gate is **FAIL_UNCHANGED_NEIGHBOR_TRANSFER: 468/492 complete cases**. All 300 latest sealed, 200 earlier sealed and 6,688 current0175/current8055 paths remain exact, including 512 typed unresolved cases. No model parameter or serving algorithm changes.

The 24 failures have correct actual first payloads and complete rewritten queries, but reject valid/conflicting second-hop relations for the name “selka will.” Its name-will shares the query-role key `(13,64,15)` with auxiliary-will in retained training because both the unfamiliar name and question-only “who” become unknown neighbors. Paired observations show 12 query-role changes, zero source-role changes and zero required-mask changes. Affected missing siblings pass through the same rejection; the report does not treat that as successful relation understanding. A direct one-role intervention is NOT_RUN, so a proposed observation repair is not yet a qualified fix.

The Rust report freezes 492 unique raw cases and independent expectations before model load, checks vocabulary novelty, runs Full/ExactIdentity/reload/read/update controls, retains previous paths, and records source/query/participation observations and collision witnesses. The conflict construction preserves the inactive-source intervention. One optimized offline build passes 48 focused tests; one sealed actual evaluation, zero fits/retries. Formatting, claim wording and staged diff checks pass. Model/build/preparation/evaluation charge is 213,440 ms within the projected 600,000 ms allowance; necessary local cumulative extensions were recorded before use. No paid compute or cleanup; prior artifacts, source versions and dirty checkouts remain preserved.

Current state recommends training-only question-inclusive anchors and predetermined withheld-name observation views, with separability checked before fitting and actual generation/retention required afterward. Independent final holdout remains NOT_RUN; normal15baec48 is unpromoted. This PR delivers the diagnostic and precise next action, not a new model qualification. Protected queue status names are compatibility acknowledgements; local executed checks carry validation.

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

