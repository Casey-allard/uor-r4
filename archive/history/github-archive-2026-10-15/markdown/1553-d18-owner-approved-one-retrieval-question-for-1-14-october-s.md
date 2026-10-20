# #1553 — D18 (owner-approved): one retrieval question for 1–14 October; STATUS and current state

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-30T21:09:43Z
- **Updated:** 2026-10-09T13:08:45Z
- **Closed:** 2026-09-30T21:11:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1553
- **Merged:** 2026-09-30T21:11:23Z
- **Merge commit:** e4a6771f7f7f92d7b6ddfdd199b3e479d14ca7a2
- **Base / Head:** main / UOR-Foundation:codex/d18-direction
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

This PR delivers **D18**, which the owner approved on 30 September (#820). For 1–14 October the programme answers one question: can the native geometric stack copy a value it was just told, and does that survive D11 serving?

- **`docs/integration/DECISIONS.md`:** D18.
  - The A1 run as pre-registered, with an equal-step budget amendment.
  - The four day-14 outcomes, A to D.
  - The `TruncatedPrefix` data unlock and a staged 7M fit that runs only on outcome A.
  - A D11 port of the trained pointer, gated against its own float.
  - Parked items, each with a re-entry condition: Track B, QAT and codec reruns, kernel and energy work, the memory port, and long context.
  - **§9 records the owner's other rulings:**
    - two labs remain;
    - self-review is allowed;
    - the owner runs a count-only §8 token check;
    - DeepSeek writes a one-day D5 cost design memo;
    - the #1546 Adam guard is not needed, so it is not restored and the four legacy tests are ignored (separate code PR).
- **`docs/integration/direction-review-2026-09-30.md`:** the council's memo, from workflow `wf_0efd43f8-274` (four readers, three proposals, two critics and a judge), kept as written apart from one timing correction.
- **`STATUS.md`:** rewritten for two labs and the D18 items.
- **`docs/integration/current-state.md`:** D18 is the active execution contract. The 29 September contract is kept and marked superseded; its anchor is unchanged because `AGENTS.md` links to it.

No measurement changes and no capability is claimed.

## Checks at this head

- `git diff --check`: pass.
- `python3 scripts/check_claim_wording.py`: pass.
- The new relative link targets exist.
- Documentation only: no Rust changed.

References #1552, #820.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5919775586 — 2026-09-30T21:09:45Z — Casey-allard

**Claude lab, exact-head self-review at `d5a7672a`: APPROVE** (owner rule, #820). Documentation only. I checked that D18's text matches the owner-approved draft and the four menu answers (§9). The #1546 wording matches the owner's answer: the guard is not needed, so it is not restored and the tests are ignored. The `AGENTS.md` anchor into `current-state.md` still resolves. `git diff --check` and the claim-wording gate pass at this head.

### comment 5919861636 — 2026-09-30T21:14:50Z — Casey-allard

Merged as e4a6771f7f7f92d7b6ddfdd199b3e479d14ca7a2 at 21:11 UTC. Verified: `git merge-tree --write-tree 302e0ad7 d5a7672a` equals the squash tree (e64914e9). Docs only; D18 is now in force on main.


---
_Comment/review ids in this file: 5919775586, 5919861636_

