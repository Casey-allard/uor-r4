# #1276 — Learn styled occurrence roles from paired output credit

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-14T16:02:26Z
- **Updated:** 2026-10-09T13:18:00Z
- **Closed:** 2026-09-14T16:03:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1276
- **Merged:** 2026-09-14T16:03:45Z
- **Merge commit:** b2c94494bfeedfbd3498d732bd94ddce54516a76
- **Base / Head:** main / UOR-Foundation:codex/styled-role-learning
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The occurrence-role learner confused known endpoints with grammatical context and failed 36 styled source/query cases. Paired final-output credit, bounded actual-generation source-rule selection and constrained query-witness refinement now pass all 96 styled cases while preserving the retained query-participation parameters and prior complete paths.

Actual qualification: **PASS_LEARNED_STYLED_ROLES**, 300/300 authored development structures and 2,976/2,976 direct training byte/EOS answers. Removing query refinement returns 2,904/2,976. All 200 latest sealed Full traces and 6,688 candidate/current-parent traces remain exact, including 512 typed unresolved results. Both negative candidates are preserved; all their successful development structures are retained. Serving uses the existing geometric/integer/table path with no new inference algorithm.

Validation: optimized offline touched-package tests pass 46 focused tests; actual fit, controls, reload/exact comparisons and retained reports are sealed with source/binary identities. `cargo fmt --check`, claim wording and staged diff checks pass. The delivered test report differs from fitted source only by joining two import lines; exact formatting lineage is recorded, all runtime source is identical. Three successful builds and one corrected report compile failure are charged. Seven coordinate fit stages across three attempts; total model/build/fit/evaluation charge 1,311,773 ms, within the recorded 1,600,000 ms step allowance. No external spending or cleanup.

Normal model 15baec48 remains unpromoted; authored open development does not establish general grammar/prose, and independent final holdout is NOT_RUN. Current state selects unchanged-artifact lexical-neighbor transfer next. Evidence and restart paths are in the result document and machine receipt. Protected queue CI names are compatibility acknowledgements; local executed checks carry validation.

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

