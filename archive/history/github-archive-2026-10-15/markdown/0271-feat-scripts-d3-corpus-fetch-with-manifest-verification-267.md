# #271 — feat(scripts): D3 corpus fetch with manifest verification (#267 fallback path)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-28T18:12:13Z
- **Updated:** 2026-10-09T13:09:07Z
- **Closed:** 2026-07-28T18:23:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/271
- **Merged:** 2026-07-28T18:23:45Z
- **Merge commit:** 21941ce3007ea704dd2f29229ba9bed05798d111
- **Base / Head:** main / UOR-Foundation:issue-267-fetch-script
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #267 (does not close it — authoritative pinned-bytes storage and the re-pin decision remain with the team).

Lands `scripts/fetch_d3_corpus.py`, the script used for today's #242 measurement, as the documented fallback re-materialization path: fetches the 3000-article sample per the manifest's own recipe, verifies against pinned totals, and reports upstream DRIFT loudly (exit 1 unless `--allow-drift`) — because drift is now known-present, numbers from a drifted fetch must not be silently compared to pinned baselines.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_017tu2Y4AGZvcgM4vmoSz5x6

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

