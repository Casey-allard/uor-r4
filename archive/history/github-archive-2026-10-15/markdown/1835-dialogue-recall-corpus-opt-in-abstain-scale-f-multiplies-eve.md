# #1835 — dialogue-recall-corpus: opt-in abstain_scale=F multiplies every abstention rate (Step 12; References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T07:26:46Z
- **Updated:** 2026-10-09T13:08:59Z
- **Closed:** 2026-10-07T07:28:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1835
- **Merged:** 2026-10-07T07:28:23Z
- **Merge commit:** 3fbf69f12bd382799a8e94e292186c91ee117a1d
- **Base / Head:** main / UOR-Foundation:claude/abstain-scale
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Adds `abstain_scale=F` (default 1.0, range [0, 4]) to `dialogue-recall-corpus generate`. It multiplies every abstention probability:
- the v1 primary share (renormalised);
- the follow-up abstain rate (clamped ≤ 0.9);
- all eight v2 family abstain weights.

At 1.0 the output is byte-identical to today's (the field is written to generator.json only when F ≠ 1). This is for Step 12: the 214M model answers unknowable questions instead of declining (v4 unknowable 17 → 10/13).

Tests: `abstain_scale_default_is_byte_identical`, `abstain_scale_raises_the_abstention_share` and out-of-range refusal, 3/3. At 400 dialogues the abstain/memory counts go 75/484 → 131/413 (v1) and 76/514 → 115/485 (v2).

The binary's other tests pass except `the_leak_check_detects_an_injected_panel_turn`, which fails identically on main (pre-existing: 'Quill' is now a panel value). rustfmt clean.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6033114401 — 2026-10-07T07:26:49Z — Casey-allard

Exact-head self-review at `fde8aaf8c65501f5f3c9d246244f68548df6989d` (owner rule: self-review allowed). Diff read (one file). At this head: `cargo check --bin dialogue-recall-corpus` 0 errors; `abstain_scale` tests 3/3; binary suite 20 passed / 1 failed — the failure (`the_leak_check_detects_an_injected_panel_turn`) reproduces on origin/main without this diff; rustfmt clean. Default path byte-identical. APPROVE.


---
_Comment/review ids in this file: 6033114401_

