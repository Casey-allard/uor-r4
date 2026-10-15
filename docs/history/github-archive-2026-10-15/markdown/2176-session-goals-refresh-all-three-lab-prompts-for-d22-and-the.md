# #2176 — Session goals: refresh all three lab prompts for D22 and the 10 October state (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T20:41:08Z
- **Updated:** 2026-10-10T20:42:54Z
- **Closed:** 2026-10-10T20:42:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2176
- **Merged:** 2026-10-10T20:42:16Z
- **Merge commit:** ea396690f525fdd8103e2eb1fc1adf5193e9a079
- **Base / Head:** main / UOR-Foundation:claude/session-goal-d22-20261010
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Docs-only refresh of `docs/labs/session-goal.md`. The page itself says each prompt's DIRECTION is updated when its milestone gets new owner direction; all three still predated D22 (#2165).

- **All prompts:** read D22 and the line's brief in `docs/mechanisms/`; pre-register the panel noise; the count rule follows D22 (a met mechanism target resets, tooling failures are blockers, a negative closes a configuration, stopping a mechanism needs the owner).
- **DeepSeek (M1 #2029):** headline 37/232 + 20/40 on base D10; line "exact addressed memory" 0/3; next is the cycle-8 run re-registered per its NOT FIT review (control, memory-off ablation, adequate fine-tune, fresh set), then D22 §5d/§5e. Fixes the M4 issue number typo (#2029 → #2032).
- **Codex (M2 #2030):** headline 437/512 dev + 0/128 diagnostic; constructor line reopened (D22 §5a) with v2 FIT; then §5b joint learning. Removes "the constructor line is closed".
- **Claude (M4 #2032):** arm W missed (+0.0172 vs 0.01); next is the matched straight-through gradient as the decisive run, FIT before compute.

Measured prompt lengths: DeepSeek 3,900, Codex 3,596, Claude 3,847 characters (all under the 4,000 `/goal` limit). Precedent for a single-file prompt PR: #2147. No criterion, panel, grader or contract rule is changed; only the standing direction and the D22 wording already in the contract (item 3).

Line: lab operations (not a milestone line) · headline unchanged.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6101965081 — 2026-10-10T20:41:23Z — Casey-allard

Exact-head self-review at 210d67a2038e3503ff2cb87f3509e5a5cb491146: docs-only, one file. The diff changes only the three DIRECTION lines, the opening read list, step 2 (panel noise), step 5 (D22 count wording, already in contract item 3) and the Claude AUDIT line (D21/D22). No criterion, panel, grader or contract rule changes. Check executed at this head: prompt lengths measured from the file with a regex over the three ```text blocks (3,900 / 3,596 / 3,847 chars, all < 4,000). No Rust touched, so no cargo checks apply. APPROVE (self).


---
_Comment/review ids in this file: 6101965081_

