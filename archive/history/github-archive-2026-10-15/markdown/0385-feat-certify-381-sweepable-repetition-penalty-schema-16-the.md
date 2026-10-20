# #385 — feat(certify): #381 — sweepable repetition penalty (schema 16): the windowed 10pp cost is 100% the penalty, 0% the window

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-03T17:43:21Z
- **Updated:** 2026-10-09T13:09:05Z
- **Closed:** 2026-08-03T17:57:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/385
- **Merged:** 2026-08-03T17:57:07Z
- **Merge commit:** deff9a90192f566bac629a58d7278fffd0d9516f
- **Base / Head:** main / UOR-Foundation:i381-penalty-knob
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements and answers #381's fixture-scale question with a declared 3-point sweep (rows-off, window-on):

| penalty | rule12 top-1 | bits |
|---|---|---|
| 0 | **36.47%** | **8.3534** |
| −1,000,000 | 26.69% | 13.2265 |
| −2,000,000 (shipped) | 26.69% | 18.6749 |

**Attribution is exact:** penalty 0 reproduces the window-OFF row to the digit — the #375 window mode is itself measurement-free, and the entire ~10pp windowed top-1 cost is the repetition penalty. Top-1 damage saturates by half magnitude (identical argmax flips at −1M and −2M); magnitude only inflates bits. On this teacher-forced distribution the penalty is pure cost at every tested magnitude. The −2M arm reproduces the #374 campaign row digit-for-digit.

Mechanics: `GraphScorer.repetition_penalty_raw` (shipped default unchanged; setter clamps ≤0) consumed at the three scorer sites and three Gate C analysis sites; `ScoreConfig` → all six Gate C scorers; CLI `--repetition-penalty-raw`; report records it (schema 16). **Serving keeps the shipped default** — the knob is certifier-side; any serving change is a maintainer decision, and the natural-corpus arm (#381 item 1) remains open on the issue.

Refs #381, #374, #375. Gates from cleaned targets incl. lib-only clippy `-D warnings`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_018UJdEQFdckSK2et9G44jsm

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

