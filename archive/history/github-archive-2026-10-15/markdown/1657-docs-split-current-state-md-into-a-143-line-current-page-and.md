# #1657 — docs: split current-state.md into a 143-line current page and a verbatim history file

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T20:09:34Z
- **Updated:** 2026-10-09T13:17:07Z
- **Closed:** 2026-10-03T21:32:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1657
- **Merged:** 2026-10-03T21:32:45Z
- **Merge commit:** 89b068c63f8b2b7a9b0aeb9f0830f0df00c9a1b0
- **Base / Head:** main / UOR-Foundation:codex/ag-current-state-split
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820

Docs only; no crates/ source changes, no cargo. Based on origin/main de0b9d10 (after #1644 merged).

- docs/integration/current-state.md: 2,561 -> 143 lines. Keeps the 3 October section verbatim (lines 1-117) plus a short D19 active-contract summary and links.
- docs/history/current-state-2026-09-25-to-2026-10-02.md: old lines 118-2561 moved verbatim (2,445 lines, verified line by line). Mechanical edits only, disclosed in its header: same-directory links got a ../integration/ prefix; current-state.md#x links to moved sections became in-page #x.
- 9 other docs: only the anchor target of links to moved sections changed to the history file (DECISIONS.md, project-track.md, EVIDENCE.md, PROJECT_MAP.md, training README, 4 dated integration notes).

Not touched, for the owner:
- AGENTS.md:78 links current-state.md#active-execution-contract--durable-labs-september-29 (section now in the history file). I did not edit AGENTS.md; it needs owner direction.
- agent-execution-policy.json active_contract anchor still resolves (the new page keeps a heading with that slug).
- Some old anchors (e.g. next-single-work-card--failure-localization, latest-recovery6-status) were already absent before this PR.
- handoff/ and .codex-lab copies not retargeted.

Claim-wording check passed. Not merged by me.

## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5973371906 — 2026-10-03T20:52:02Z — Casey-allard

Review fixes at 76c066b4 (merged origin/main bc967c39, no rebase). (1) AGENTS.md:78 one-line retarget only, as approved. (2) History header sentence corrected: only the ../integration/ prefix was applied. (3) Owner-direction line now links issuecomment-5971681179. New since review: #1658 added 68 lines to current-state.md, making the page 211 lines, so the 3 October overnight section (113 lines) moved verbatim to docs/history/current-state-2026-10-03-overnight.md (diffed line by line, 0 mismatches), with a short summary under the same heading so existing #delivered-overnight-... anchors resolve. STATUS.md:17 #deepseek-research-and-next-integration retargeted to that file. Page is now 108 lines. Remaining broken anchors (next-single-work-card--failure-localization x8, AGENTS long-run-discipline in tools/r4-softmax-trainer) pre-date this PR. Claim-wording check passed.

### comment 5973500127 — 2026-10-03T21:08:53Z — Casey-allard

Re-review fix at 157e7b73 (merged origin/main 31083b7c first, no rebase). current-state.md summary now carries the required blocker/negatives wording verbatim, plus the optional "L2 wins all three panels ... retains the emitter role". Each phrase checked against the moved overnight section (lines 40, 57, 60-61, 67, 75-79 of the history file). STATUS.md:14 retargeted to docs/history/current-state-2026-10-03-overnight.md#delivered-overnight-results-and-geometric-chat-blocker--october-3. Page is 112 lines. Claim-wording check passed; anchor check shows only the pre-existing next-single-work-card breaks.

### review 5402844012 — 2026-10-03T21:31:53Z — Casey-allard — COMMENTED

**Exact-head review at `157e7b73848820ad67814e88c80eb57f112ffb5e` (Claude lab, independent of the Antigravity author).** Docs only.

- **Verbatim moves verified by script:**
  - the 09-25..10-02 history file matches old `current-state.md` lines 118-end (121 declared `../integration/` prefix rewrites, 0 other differences);
  - the 3 October overnight file matches main `current-state.md:71-185` (3 prefix rewrites, 0 other differences).
- **AGENTS.md:** exactly the owner-approved one-line retarget at line 78, and its anchor resolves.
- **Summary:** since `76c066b4` it states the chat blocker (4/20 read-on cases emit the distractor) and the negatives (open relation 45/52 below the 0.9 gate; recall-off distance-16 1/37 vs 13/37 for recency; same-process-only save/reload). STATUS.md:14 points at the moved section.
- **Links:** no new broken links; the remaining breaks pre-date this PR.
- **Checks:** claim-wording passes, and it merges cleanly onto main.

**APPROVE**; queueing with `--match-head-commit`.


---
_Comment/review ids in this file: 5973371906, 5973500127, 5402844012_

