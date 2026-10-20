# #1496 — Lab 2 (OpenCode): record the G-binding result in current state

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T06:41:34Z
- **Updated:** 2026-10-09T14:16:59Z
- **Closed:** 2026-09-29T06:44:28Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1496
- **Merged:** 2026-09-29T06:44:28Z
- **Merge commit:** a549615e1b0b63671d66e822f3e9e38cdca251fd
- **Base / Head:** main / UOR-Foundation:lab/opencode/current-state-binding
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab 2 (OpenCode). Class B** (carries already-re-read measured numbers). References #973 and #820.

Records the G-binding outcome in `current-state.md` beside the G v1 entry: masking the ordinary-text tag/trigger losses lifts held-out-template Updated 0.158 → **0.967**; tag accuracy 0.974 → 0.996 and `Unavailable` 460 → 0; the text-NLL guard fails (+0.119) and name diversity (Arm B, 0.645) is a negative; acceptance is Lab 1's call. It also points to the checkpoints for Lab 1's `prime-route-eval` and to the merged `AermModel::from_stack`.

**Numbers:** identical to the merged G-binding result (#1495), whose headline numbers were independently re-read from the sealed roots by a non-author and approved ([review](https://github.com/UOR-Foundation/uor-r4/pull/1495#issuecomment-5885047499)). A one-line confirmation of this entry is requested from the same reviewer session.

**Checks:** `python3 scripts/check_claim_wording.py` → pass. Documentation-only; no code or experiment.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5885089734 — 2026-09-29T06:42:48Z — Casey-allard

Non-author confirmation from the same reviewer session (`ses_f14278794ffeQq1vcUPFBb0VSN`): **confirmed** — every figure matches the sealed roots (0.158/0.967, 264/273, 0.974→0.996, 460→0, +0.119, B 0.645), both links resolve to the merged record/evidence, and the wording claims no more than the evidence. One marginal note applied: the entry now says "a major cause" (the pre-registration's wording) rather than "dominant". Commit pushed.


---
_Comment/review ids in this file: 5885089734_

