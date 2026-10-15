# #1169 — Retain complete forward relation values with geometric extent

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-07T13:36:47Z
- **Updated:** 2026-10-09T13:17:54Z
- **Closed:** 2026-09-07T13:37:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1169
- **Merged:** 2026-09-07T13:37:56Z
- **Merge commit:** 218e11292385ed2f980bf9a884e5b47ecea5350f
- **Base / Head:** main / UOR-Foundation:codex/retained-value-spans
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Persistent relation writes currently retain only the first word of a value. This adds an opt-in artifact capability that reuses the accepted H4 Continue/Finish operator to retain complete forward values, commit one immutable version, compare complete values for conflicts, and emit them after raw-window eviction. All parent parameters remain unchanged; no fitting was required. Reverse statements retain their established single-word behavior because the writer does not supply a phrase endpoint.

Corrected behavior passes 6/6 construction and 6/6 open session turns plus 6/6 previously exposed preservation turns, including repeated assertions, same-prefix revisions/conflicts, unrelated memory and isolation. All 663 retained construction answers and earlier preservation pass. The initial runtime regressed two reverse-statement answers to 26/28; its source and reports are preserved, and the corrected runtime restores 28/28. Both runs use the same artifact bytes, so the evidence binds runtime source identities separately.

Validation: corrected release probe/test binaries compile; focused relation and source-span tests, integer source guard, formatting, architecture policy and claim wording pass. The initial actual-artifact pending-write/checkpoint/eviction/allocation test passes on the unchanged forward path; final corrected behavior and checkpoint replay pass in the Rust probe. CLI/Studio and broad release QA are NOT_RUN for this artifact. GitHub required statuses are queue acknowledgements only, not test evidence.

Model work: 54.294 seconds; cumulative 4,283.599/4,290, leaving 6.401 seconds without an increase. Engineering: 551.248 seconds; peak sampled storage growth 33,886,208 bytes inside 48 MiB. No cleanup or external compute. Result, source/artifact identities, exact boundaries and next role-aware endpoint direction are recorded in `docs/native_geometric_retained_spans_1140.md` and its evidence receipt; README, research, roadmap and current-state pointers are updated.

Continues #1139 and #1140; neither broader capability issue is closed by this bounded result.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

