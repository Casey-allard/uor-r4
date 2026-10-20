# #2059 — Illustrated geometry: seven computed figures and docs/geometry.md

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T17:05:52Z
- **Updated:** 2026-10-09T17:07:32Z
- **Closed:** 2026-10-09T17:07:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2059
- **Merged:** 2026-10-09T17:07:13Z
- **Merge commit:** 11d619557d44c623cf0a2a2cbbcb320be6f569cc
- **Base / Head:** main / UOR-Foundation:claude/geometry-illustrations
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner request (9 October 2026): pictures that show the geometric process, so people can imagine it.

- **docs/figures/geometry/*.svg:** seven figures computed from the real geometry by `generate.py` (stdlib Python, deterministic):
  - a quaternion rotating a vector;
  - the stack state transported around S³;
  - the Hopf fibration;
  - the 600-cell's 120 unit icosians (720 edges, 30-fold Coxeter projection; vertex and edge counts asserted);
  - the golden integers Z[φ];
  - zeta-zero phases;
  - prime addressing.
- **docs/geometry.md:** each picture with what it is, what the model does with it (formulas and file paths confirmed from the code), and its status. Also a life-of-one-token walkthrough and a worked example of serving without multiplication. Details that couldn't be confirmed are marked.
- **README:** links the page and shows the 600-cell in How it works.

Checks: claim wording passes; regenerating leaves the tree unchanged; every SVG parses with no external references; every link resolves; each figure was rendered and inspected.

References #2028

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6085560173 — 2026-10-09T17:05:54Z — Casey-allard

Exact-head self-review at `c9c10cb7266b273f19a45a43b655313c7fc0f4b3`: viewed the Hopf and 600-cell renders; generator deterministic; claim wording passes; links resolve.


---
_Comment/review ids in this file: 6085560173_

