# #1622 — current-state: D19 best configuration is emit-6r (training-template paraphrases)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T15:02:43Z
- **Updated:** 2026-10-09T13:08:46Z
- **Closed:** 2026-10-02T15:32:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1622
- **Merged:** 2026-10-02T15:32:56Z
- **Merge commit:** 7b0a8c047820119b74933ba0b971592ac0362704
- **Base / Head:** main / UOR-Foundation:codex/d19-state-emit-6p
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #1552

Docs only. Replaces the D19 section of `current-state.md`: emit-6r (teacher paraphrases of training chat, instruction and relation templates, #1618) is the measured best, total 889 against 819 for emit-5c, paired on the same 300 development conversations. Includes the diagnosis (skills learned on trained phrasings, failed on development phrasings), the losses, and one-seed scope. `scripts/check_claim_wording.py` passes.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### review 5393577270 — 2026-10-02T15:24:15Z — Casey-allard — COMMENTED

Exact-head review at 39568fda7563daa3f7085d4b985134695cb72906 (Claude lab; self-review per owner rule on #820). Docs only: the D19 section of current-state.md. Every number checked against the session reports on #1552 (emit-5c, emit-6p, emit-6r, and the emit-6r reload of 20/20). check_claim_wording passes. Evidence run https://github.com/UOR-Foundation/uor-r4/actions/runs/37024852656 (parent = this head): lib 569 passed / 0 failed.

### comment 5956399610 — 2026-10-02T16:11:56Z — Casey-allard

Merged as 7b0a8c04; docs/integration/current-state.md on main equals the reviewed head 39568fda (blob comparison).


---
_Comment/review ids in this file: 5956399610, 5393577270_

