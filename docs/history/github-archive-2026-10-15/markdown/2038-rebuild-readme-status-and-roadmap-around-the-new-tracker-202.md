# #2038 — Rebuild README, STATUS and ROADMAP around the new tracker #2028; repoint live pointers; uor-pod posts to #2037

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T14:46:58Z
- **Updated:** 2026-10-09T14:50:28Z
- **Closed:** 2026-10-09T14:49:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2038
- **Merged:** 2026-10-09T14:49:53Z
- **Merge commit:** 43fc0935575734e07c2e39c868d678e336e06c87
- **Base / Head:** main / UOR-Foundation:claude/readme-roadmap-rebuild
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner-approved tracker and README rebuild (9 October 2026).

- **README.md:** rewritten as a research project + app: honest status, what works today (verified commands), quick start, how it works, results including negatives and retractions, the M1–M8 roadmap, how the project is run, and a map of all 20 workspace members.
- **STATUS.md:** a short page dated 9 October, covering line A (stack LM) and line B (native learner), serving, compute and labs. The stale 2×4090 and OpenCode wording is fixed.
- **ROADMAP.md:** the M1–M8 milestones (#2029–#2036), with PROPOSED thresholds marked.
- **Live pointers:** `AGENTS.md`, `docs/labs/{README,protocol,compute}.md` and the project-track Active programme section now point at tracker #2028, the milestones and compute board #2037, instead of the closed #820, lab boards and #1750. Dated historical mentions are unchanged, and no rule text changed.
- **scripts/pod/uor-pod:** the default board is now #2037.

Checks: claim wording script passes; every workspace member is named in the README; all relative links in README/STATUS/ROADMAP resolve; `bash -n scripts/pod/uor-pod` is clean.

References #2028

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6083258241 — 2026-10-09T14:47:05Z — Casey-allard

Exact-head self-review at `d63357f897e42a7abde14406cb3cd827b0c7032b`: read the AGENTS.md and protocol diffs line by line (pointer swaps only), the README opening and tables, and STATUS/ROADMAP. Re-ran `check_claim_wording.py` (pass) and `bash -n scripts/pod/uor-pod` (clean) at this head.


---
_Comment/review ids in this file: 6083258241_

