# #1170 — Retain reverse relation phrases at learned writer endpoints

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-07T14:23:41Z
- **Updated:** 2026-10-09T13:17:54Z
- **Closed:** 2026-09-07T14:25:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1170
- **Merged:** 2026-09-07T14:25:40Z
- **Merge commit:** 3dd7180206b31a91998e9052e02eeed334655c3d
- **Base / Head:** main / UOR-Foundation:codex/reverse-relation-spans
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Reverse relation writes currently retain only the writer-selected final word. This reuses that learned endpoint and the existing H4 Continue/Finish operator to retain earlier contiguous value words, without including the following linker or owner. The selected WordAtom identity is preserved; the earlier phrase start is stored separately. An optional artifact component reconstructs the complete unchanged parent, with no new fit.

Construction improves 3/6 to 6/6; open sessions pass 6/6, short reads 3/3, and six fresh turns after selection pass 6/6. All 663 prior construction answers, 18 forward session turns, 28 long-window answers and 48 dependent reads remain correct, alongside the other selected preservation checks. Unseen plain introductions still overextend and two-space gaps truncate (0/2 boundary diagnostics); these are preserved without retuning. The next direction is learned start selection over admitted candidates.

Validation: release probe and focused test binaries compile; five relation-span tests, integer source guard, architecture policy, formatting and claim wording pass. The actual artifact passes short/evicted reads, preserved terminal identity, malformed parent/start rejection, every-step checkpoint replay and zero measured allocations. Root CLI/Studio and broad release QA are NOT_RUN for this artifact. Required GitHub statuses remain queue acknowledgements, not model-test evidence.

Actual model work is 43.770 seconds; cumulative 4,327.369/4,410 leaves 82.631 seconds after the announced 120-second continuation allocation. Engineering is 269.887 seconds; peak sampled growth is 17,833,984 bytes within 40 MiB. No cleanup or external model compute. README, roadmap, research and current-state pointers link to the complete result and evidence.

Continues #1139 and #1140; no broad capability issue is closed by this bounded result.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

