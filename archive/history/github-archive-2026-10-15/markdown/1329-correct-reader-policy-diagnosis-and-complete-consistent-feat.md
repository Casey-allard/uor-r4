# #1329 — Correct reader-policy diagnosis and complete consistent feature learning

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-21T06:03:13Z
- **Updated:** 2026-10-09T13:17:51Z
- **Closed:** 2026-09-21T06:06:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1329
- **Merged:** 2026-09-21T06:06:04Z
- **Merge commit:** 0492926d8ec3aecc28185e14607706595ac0bb57
- **Base / Head:** main / UOR-Foundation:codex/reader-utility-principal-review
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

PR #1328's held-out text loss improvement survives independent reconstruction, but the proposed contextual-state pivot rests on a training/serving feature mismatch: events are fitted with legacy zero gap thresholds, then the exported policy receives negative thresholds. This review retains the measured artifact result and redirects the next task to completing the intended finite-policy experiment before changing representation.

The review also corrects actual-action regret dispatch, emitted/absence predicates, admission denominators, artifact/source binding and unresolved timing claims. It synchronizes the README, canonical roadmap, current state, mechanism bridges and historical-record correction notices, and supplies a comprehensive DeepSeek execution prompt with bounded research discretion. Source code and sealed artifacts are unchanged; implementation repairs and corrected fitting are explicitly the next run, not claimed complete here.

Validation: six sealed manifest members and 24 document hashes verified; all five declared document intervals reconstructed exactly; independent source/evidence/mathematics reviews; claim-wording gate, JSON parse, 869 local Markdown links and `git diff --check`. No new Rust build, fit or model inference in this review; the prior passing Rust tests did not cover the discovered orchestration defect. No resource extension, deletion or paid compute.

References #973, #820, #1139, #962, #963, #964. Broad acceptance remains open.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

