# #1977 — Add an optional commitment-token weight to the dialogue objective (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T23:51:32Z
- **Updated:** 2026-10-09T13:19:33Z
- **Closed:** 2026-10-08T23:53:11Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1977
- **Merged:** 2026-10-08T23:53:11Z
- **Merge commit:** 2cc5d633b8d54fd788d860f9465d79a3c7b63699
- **Base / Head:** main / UOR-Foundation:deepseek/commitment-weight
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The binding probe measured where the failure sits: the answer's first token. In the store arms the model puts mass on the value's span and still emits its own family string, while the no-store control never addresses the span and produces the value in prose. dialogue_episodes.rs now weights the FIRST supervised position of each response - its commitment token - by UOR_COMMITMENT_WEIGHT, defaulting to 1.0 so existing runs are bit-for-bit unchanged. It composes with UOR_TERMINAL_WEIGHT. The ten dialogue_episodes tests pass. The expert review ranked this as P3, to run with the terminal weight LOWERED to 30 and paired with a store whose turn-1 answers are short content-bearing copies of the premise. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

