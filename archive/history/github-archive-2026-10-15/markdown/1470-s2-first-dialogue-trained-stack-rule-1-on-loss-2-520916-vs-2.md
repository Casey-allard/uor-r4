# #1470 — S2: first dialogue-trained stack — rule 1 on loss (2.520916 vs 2.773887); replies poor (0/10 recalls)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T20:25:35Z
- **Updated:** 2026-10-09T13:09:00Z
- **Closed:** 2026-09-28T20:46:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1470
- **Merged:** 2026-09-28T20:46:12Z
- **Merge commit:** 8f1698670b16cdac731f032f7cede56238cb7b6c
- **Base / Head:** main / UOR-Foundation:lab/claude/s2-dialogue-baseline
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab 1 (Claude main). Class B** (result). References #973 under #820. Pre-registration: https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5875334772.

**Result.** The record is [`docs/integration/s2-dialogue-stack-baseline-2026-09-28.md`](docs/integration/s2-dialogue-stack-baseline-2026-09-28.md). The evidence was assembled by script from the four sealed roots under `uor-r4-lab/claude-s2-dialogue-baseline/`: `lm-1`, `dialogue-1`, `export-1` and `chat-1`. All four verify.
- **Pooled development response NLL: 2.520916**, on the identical 161-response panel. The native full-prefix child scores 2.773887, and its parent 3.022131. That is **rule 1: S2 `8cb11d8f…` becomes the loss-level dialogue baseline.**
- **Worse than the native child** on Rewrite, Summarize, UltraChat and the first-four targets.
- **Complete replies:** 2 of 38 final turns answer, 0 of 10 memory recalls succeed, and the D10 integer replies match float on 14 of 58 turns. The native child stays the reference for complete replies and memory.

**Also updated:** ROADMAP §2b's pinned baseline, a §9 entry (which also records this session's merges), and `current-state.md`.

**Review:** this needs a non-author re-read of the headline numbers from the roots (class B).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

