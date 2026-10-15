# #1800 — geometric-stack example: cover StackArch::Hybrid after #1798

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T14:49:44Z
- **Updated:** 2026-10-09T13:09:03Z
- **Closed:** 2026-10-06T14:54:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1800
- **Merged:** 2026-10-06T14:54:00Z
- **Merge commit:** 402d412a308126ae40776e6b307b8e419bf03e9a
- **Base / Head:** main / UOR-Foundation:claude/geometric-stack-hybrid-arms
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Main does not compile `cargo check -p uor-r4-training --example geometric-stack` since #1798 added `StackArch::Hybrid` (E0004 non-exhaustive, three sites).

- `init_config` names Hybrid `hybrid` and maps its MLP option to `stack_mlp` (Hybrid uses the recurrence-stack layout, as in the library's `shapes`).
- The LUT export refuses Hybrid with an error: the read-attribution control has no lookup-table export.
- rustfmt on three pre-existing hunks in the same file.

Checks at head: `cargo check -p uor-r4-training --example geometric-stack` → 0 errors (was 3 on main); `rustfmt --check` on the file is clean.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6018868003 — 2026-10-06T14:49:46Z — Casey-allard

Exact-head self-review at `903d976cdf8a504e5185717fcc89b88056b041b5`: diff read in full (3 match arms + rustfmt only). `cargo check -q -p uor-r4-training --example geometric-stack` at this head: 0 errors; main 4cee4a4b: 3 × E0004. No behaviour change for Geometric/Transformer. APPROVE.


---
_Comment/review ids in this file: 6018868003_

