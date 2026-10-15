# #1261 — Learn relative language binding across mixed constructions

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T23:12:28Z
- **Updated:** 2026-10-09T13:17:53Z
- **Closed:** 2026-09-13T23:14:09Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1261
- **Merged:** 2026-09-13T23:14:09Z
- **Merge commit:** 277a787e57e445f933cefd53fbac4b3cccb8610a
- **Base / Head:** main / UOR-Foundation:codex/relative-language-binding
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The contextual language reader used absolute positions and required four-word inputs. This adds learned relative match topology across sentences of four to seven words and questions of four to eight words, using the same recurrent emission loop and frozen byte/EOS writer. The corrected fit generates all 768 development answers with exact occurrences: 256 each for unseen vocabulary, unseen combinations of familiar active components, and both together.

Training expands from 512 to 2,048 examples. The first fit failed at 384/768 because the descriptor collapsed distinct role-reversed match paths. A training-only diagnosis identified missing endpoint correspondence; schema 2 retains it and allows four-literal conjunctions. Two learned rules then cover all training examples. Both attempts and source/binary snapshots are preserved, with identical data and unchanged existing behavioral thresholds. Removing endpoints restores the 384/768 result. Exact-identity, final-root-only and feedback-disabled controls also pass, so no metric advantage or general prose capability is claimed. Final held-out evaluation is NOT_RUN; retain normal model 15baec48.

Validation: four new and 27 retained focused tests; actual generation and 768 reload comparisons; 13,248 old outputs, 512 recurrent traces, 640 ordered controls and 1,408 language controls unchanged. All 128 old language examples also pass through the new selector. All 42 prior sealed roots and 1,137 files, plus original dirty checkouts, are preserved. Formatting, claim wording and diff checks pass. Queue statuses are compatibility acknowledgements only.

The plan, current state, project map and evidence are updated. Next integrate raw-language binding with dependent reads and query updates, with causal intermediate-value interventions and no supplied gold intermediate answers. Complete model/build/fit/check charge is 416,147 of 2,400,000 ms. The parent allowance was increased by 1,200,000 ms before execution under standing authorization; the shared ceiling is unchanged. No paid compute or cleanup.

References #973. References #964.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

