# #1846 — chat-grade reply: opt-in cycle_repeats=N so code fences are not cut as loops (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T16:02:29Z
- **Updated:** 2026-10-09T13:09:02Z
- **Closed:** 2026-10-07T16:07:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1846
- **Merged:** 2026-10-07T16:07:56Z
- **Merge commit:** 97430c1b91abdaec4d362dde65aa363c002885b4
- **Base / Head:** main / UOR-Foundation:claude/cycle-repeats
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Adds `short_cycle_period_with(tokens, repeats)`; `short_cycle_period` = repeats 3 (all existing callers unchanged). Threads `cycle_repeats` (2..=64, default 3) through `Reply::stop_with` / `greedy_reply_with` to `chat-grade reply`; recorded in replies.json only when not 3. A markdown fence is the backtick id x3, so the default rule cut every code answer at its fence.

Tests: new asserts in `historical_sampler_rng_and_cycle_boundaries_are_fixed`; `replies_stop_at_eos_then_at_a_short_terminal_cycle` passes.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6041823422 — 2026-10-07T16:07:03Z — Casey-allard

Exact-head self-review at `395f2bc8274357ee50f659996b67bd8a13155579` (owner rule: self-review allowed). Diff read. At this head: `cargo test -p uor-r4-training --lib -- historical_sampler replies_stop` passes (short_cycle_period_with: [7,7,7]@3→Some(1), @6→None, period-2 @6→Some(2); historical stop unchanged); rustfmt clean on touched files. Default `cycle_repeats`=3 keeps every existing evaluation byte-identical; opt-in only for code-heavy panels where a ```` fence (backtick×3) was being cut as a loop. APPROVE.


---
_Comment/review ids in this file: 6041823422_

