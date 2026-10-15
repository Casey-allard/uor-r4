# #1679 — chat-grade grade-replies: judge replies another tool produced (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T06:47:29Z
- **Updated:** 2026-10-09T13:08:43Z
- **Closed:** 2026-10-04T06:48:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1679
- **Merged:** 2026-10-04T06:48:31Z
- **Merge commit:** f5436e1db7e798b9924a824da70d45dfcaaebe68
- **Base / Head:** main / UOR-Foundation:codex/chat-grade-replies
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Adds `chat-grade grade-replies out= replies= [grader] [ollama_url]`. It grades replies that another tool already produced (`lut-chat` chat.json `record`, or a replies.json `panel`). The judging loop is moved into `judge_panel` and shared with `grade`, so both paths score identically.

**Why:** the D11 integer chat (#1676) generates replies itself. Before this change, an integer model could not be put on the same 7B judge panel as the float model.

**Exercised on the real panel.** chat-29m-B-lr5e-4 was exported round-to-nearest to D11 and graded on the 232-request panel with the qwen2.5:7b judge:

| | Acceptable | Fluent | Relevant | Control acceptable |
|---|---|---|---|---|
| Integer (D11, round to nearest) | 31 | 54 | 43 | 3 |
| Float | 43 | 70 | 53 | 4 |

- Paired McNemar on acceptable: 5 rows integer-only vs 17 float-only (p=0.017).
- Integer vs its own control: p=5.8e-8.

**Checks:**
- `cargo fmt --check`: pass.
- `cargo check -p uor-r4-training --bin chat-grade`: pass.
- `cargo test --bin chat-grade`: 1 passed.

References #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5977438563 — 2026-10-04T06:47:39Z — Casey-allard

Exact-head self-review at b333036477b55fecfc7a18e71aecbbc0bf289eb5: a one-file change. The grade path now calls the shared judge_panel. grade-replies was run on the 232-row integer panel. fmt, check and test pass locally.


---
_Comment/review ids in this file: 5977438563_

