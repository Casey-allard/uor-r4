# #466 — docs(process): #460 — long-run discipline (reachability, instrument-as-gate, pre-declared decision)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-06T17:05:35Z
- **Updated:** 2026-10-09T14:17:06Z
- **Closed:** 2026-08-07T00:17:44Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/466
- **Merged:** 2026-08-07T00:17:44Z
- **Merge commit:** 13949f169d9fe8aca3a5875bf0baa91127862ead
- **Base / Head:** main / UOR-Foundation:i460-discipline
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #460. Process amendment in AGENTS.md, motivated by this week's losses.

Three ordered gates before any run measured in hours: **reachability arithmetic** (compute the ceiling on the metric you intend to move — 97.9% ExactContext means a cover change is capped at ~2.1pp, a five-minute calculation that invalidates a four-hour run), **the cheap instrument as a hard gate** (`capacity_scaling` reported `records_per_full_key: 36.02 SATURATED` before a multi-hour Gate C run that then confirmed exactly what that line implied), and **pre-declared decisions** (if the positive and negative branches lead to the same next action, the run has no decision value).

Adds a run-contract template to paste into issues before launching, plus the issue-hygiene rules: assignment means actively-working-now, and a PR that ships partial scope says "References #N" not "Closes #N" (the #460 auto-close incident).

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

