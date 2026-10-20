# #1451 — Integrate Lab 3's T3 energy harness (#1444) and complete the serving auditor's patterns (References #820, #964)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T05:39:57Z
- **Updated:** 2026-10-09T13:08:56Z
- **Closed:** 2026-09-28T05:40:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1451
- **Merged:** 2026-09-28T05:40:59Z
- **Merge commit:** 81998fcfb5d19de7900a4a792789cc3f52117386
- **Base / Head:** main / UOR-Foundation:lab/claude/integrate-1444
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab:** Claude, director, integrating Lab 3's #1444 under the owner's instruction to clear the PR backlog. Lab 3's own branch is not modified.

**Commits.**
1. #1444's commits, unchanged: the energy harness, the energy evidence, the stack mission-port design and the auditor artifact binding.
2. `45d59d89`, a merge of main (#1450 and #1435).
   - `sampling.rs` and `same-input-step.rs`: #1444 contributed no lines to any conflicting hunk, so main's side is kept.
   - The audit script: main's call-graph auditor, plus #1444's source-commit TAP line. The FULL PASS scope text is written per mode.
3. The auditor's patterns completed.
   - Newly matched: `fmla`/`fmls`, `fmax`/`fmin` and their families, float compares and estimates, bf16 dot products, scalar `fabs`, and float ops with any vector arrangement.
   - Also newly matched: widening and matrix integer multiplies (`smull2`, `umull2`, `sqdmull`, `smmla`, ...).
   - Each is a failing sentinel, and 8 benign integer instructions must stay unflagged.

**Checks.**
- `cargo check -p uor-r4-integer --all-targets` and `-p uor-r4-training --example same-input-step` pass.
- The sentinel self-test passes.
- The completed auditor on the frozen #1450 `uor-chat` (`4a4253a1…`): **0 forbidden instructions** across 52 symbol ranges and 112 reachable functions.

**Scope.** The energy result stays as recorded: whole-system marginal, SoC counters unavailable, different checkpoints. No quality-matched energy claim follows.

Supersedes #1444. References #820, #964, #963, #973

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

