# #1260 — Learn contextual language relation selection from answer text

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T22:44:44Z
- **Updated:** 2026-10-09T13:17:37Z
- **Closed:** 2026-09-13T22:46:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1260
- **Merged:** 2026-09-13T22:46:08Z
- **Merge commit:** 618632e455357d01b36e03a84309bd40d97f46fe
- **Base / Head:** main / UOR-Foundation:codex/language-relation-attention
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Raw sentences and questions previously lacked a learned relation selector; the ordered-state experiment required supplied exact query keys. This adds a Rust output-supervised selector over ordered geometric word comparisons and reuses the existing recurrent byte/EOS writer. One fit learns two shared conjunctions and answers 128/128 open development queries with correct source/word occurrences, including role reversals and changed-source interventions with names and verbs absent from training.

The scope is fixed four-word syntax and learned structural alignment. ExactIdentity, FinalRootOnly and FeedbackDisabled also pass 128/128, so no geometric-metric advantage, feedback-dependent selection or general prose capability is claimed. Normal model 15baec48 remains retained; final held-out evaluation is NOT_RUN.

Validation: one new and 26 retained focused tests; actual generation, malformed artifact checks and 128 reload comparisons; 13,248 prior outputs, 512 recurrent traces and 640 ordered control outputs/paths unchanged. Read/scorer/order/relation-position/candidate-position/cursor/stop interventions produce zero complete answers. All 41 prior sealed roots / 1,125 files and original dirty checkouts are preserved. Formatting, claim wording and diff checks pass. A postflight rustfmt-only module declaration reorder is explicitly bound separately from the frozen fit source. CI status names remain queue compatibility acknowledgements.

Updates current state, roadmap, project map and source/artifact evidence. Next: shared relative-position/context learning across mixed sentence/question lengths, with lexical and syntactic transfer measured separately. Local cumulative resources remain within the recorded allowance; no extension, external cost or cleanup.

References #973. References #964.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

