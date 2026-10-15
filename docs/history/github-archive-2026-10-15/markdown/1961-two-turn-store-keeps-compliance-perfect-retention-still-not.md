# #1961 — Two-turn store keeps compliance perfect; retention still not correct (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T20:34:08Z
- **Updated:** 2026-10-09T13:22:53Z
- **Closed:** 2026-10-08T20:35:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1961
- **Merged:** 2026-10-08T20:35:49Z
- **Merge commit:** 338751adbba07354b7033a7aae4cc3f8524bf910
- **Base / Head:** main / UOR-Foundation:deepseek/two-turn-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Built a two-turn demand store from the single-turn one: 2,860 documents, 5,720 response runs, 66.68 percent dependency-carrying, verified with zero offenders on all 16 mask clauses by two independent implementations, a corruption test whose broken bytes the project EpisodeIndex rejected, and 5,720 of 5,720 responses admitted with none excluded. On panel v2 it keeps form compliance COMPLETE - Green., Honey., Yes., No., No., 7., 4., 8., Red, blue,, Paris. for the ten form-demanded turns, with correct content for seven - and retention is still not correct: the cats turn gives Woof and, the name turn 1. Dogs,. So the store shape is not why retention failed, and the pre-registered fork fires for its second branch. Retention is no longer degenerate, though. The prime suspect is now the terminal weight interacting with multi-turn sequences: retention held at weight 60 with NO demand store and fails in every arm that has one. The arm now running is the same two-turn mixture at UOR_TERMINAL_WEIGHT=1. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

