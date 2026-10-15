# #378 — feat(certify): #375 opt-in Gate C context-window mode (schema 15)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-03T15:57:07Z
- **Updated:** 2026-10-09T13:09:08Z
- **Closed:** 2026-08-03T16:12:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/378
- **Merged:** 2026-08-03T16:12:52Z
- **Merge commit:** 73ab76b369d934f5833cc7aece0014cace072e4e
- **Base / Head:** main / UOR-Foundation:gate-c-window
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements the #375 decision (maintainer-approved): an opt-in flag feeding real story-bounded recent-token windows (≤32 tokens, the compiler's own trigram boundary rule) into every Gate C coded scorer call. **Default off reproduces historical rows byte-exactly.**

Window-on semantics, documented in the config: NGRAM rows fire AND the repetition penalty engages — both serving-path behaviors, so window-on Gate C is closer to serving semantics, and window-on rows are a **different measurement population** from every schema-≤14 row. The #371 `exact_context_ngram`/`exact_context_probe` split keeps the exact-context slice attributable under this mode. `score_report.json` records the flag (schema 14 → 15). New `story_bounded_window` unit test (boundary + cap); CLI defaults test extended.

Unblocks the #362/#364 trigram-replacement A/B (rows-on vs rows-off bundles, window on) — runs on the fixture corpus once this lands, results to #374.

Refs #375, #374, #362, #364. Gates from cleaned targets: fmt, clippy `--all-targets --all-features`, full workspace tests.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_018UJdEQFdckSK2et9G44jsm

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

