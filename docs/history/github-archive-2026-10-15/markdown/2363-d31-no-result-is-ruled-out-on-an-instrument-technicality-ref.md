# #2363 — D31: no result is ruled out on an instrument technicality (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T20:03:19Z
- **Updated:** 2026-10-13T20:04:57Z
- **Closed:** 2026-10-13T20:04:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2363
- **Merged:** 2026-10-13T20:04:26Z
- **Merge commit:** b54bcbfeee849bea2b02423f98e49afc57259c61
- **Base / Head:** main / UOR-Foundation:kimi/d31-instrument-technicality-20261013
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner direction, relayed 13 Oct ~19:57Z (#2028 comment 6153979044): file it now.

Files **D31** in docs/integration/DECISIONS.md, companion to PM rule R12 (#2361, merged 76bb988a1):
1. Classify a failing check before stopping on it — mechanism vs instrument; a magnitude analysis showing the instrument can reject a correct result makes it an instrument defect at once, not an open hypothesis that keeps results blocked.
2. An instrument defect is never a BLOCKER on code, never a negative, never a stop; repair the instrument without changing the registered law/bar/fixture and re-run; never counts toward D21, never closes a configuration (D22).
3. A qualification runs every independent registered control in one pass; first-failure stops reserved for setup/safety failures only.
4. Bars stay frozen — no tolerance is widened.

First application recorded: murmuration control 11 was an instrument defect; the repaired device-F64 secant passed the unchanged bar; the qualification runs all 33 controls in one pass.

Docs-only, one file.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

