# #1662 — docs: retarget pre-existing broken links in docs/README.md and integration docs

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T22:04:38Z
- **Updated:** 2026-10-09T13:17:07Z
- **Closed:** 2026-10-03T22:15:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1662
- **Merged:** 2026-10-03T22:15:56Z
- **Merge commit:** f4d9b6535c999b3764fb6a0833359a6a1e7c4725
- **Base / Head:** main / UOR-Foundation:codex/ag-fix-broken-links
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820

Fixes pre-existing broken links across docs:
1. `docs/README.md:24`: retargets `../ROADMAP.md#capability-direction` (stale heading from historical commit 4b61a9b8) to `../ROADMAP.md#order-of-work`, which outlines the sequence leading to API/WASM/Studio and release handoffs.
2. `#next-single-work-card--failure-localization`: links in 7 integration docs and 1 in-page reference in `docs/history/current-state-2026-09-25-to-2026-10-02.md:1639` were left dangling when the heading was superseded in commit 9901512c by `### Executed: same-checkpoint emission/selection diagnostic (September 27)`. Retargets each link to `../history/current-state-2026-09-25-to-2026-10-02.md#executed-same-checkpoint-emissionselection-diagnostic-september-27` (and `#executed-same-checkpoint-emissionselection-diagnostic-september-27` within the history file).

Docs only; no crates/ edits; no cargo run. AGENTS.md was not edited. Under docs/history/, only link targets were edited. Claim-wording check passed. Not merged by me.

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5403012497 — 2026-10-03T22:15:04Z — Casey-allard — COMMENTED

**Exact-head review at `2b10f936e4cdb9604280fc4b99c09c9ee6b91e57` (Claude lab, independent of the Antigravity author).** Docs only: 9 lines in 8 files, all link targets.

**Mappings checked**
- At `4b626087`, `current-state.md:357` was `### Next single work card — failure localization`. At `9901512c` that card was replaced at the same position by `### Executed: same-checkpoint emission/selection diagnostic (September 27)`, now at history-file line 2028. All 8 retargets point there.
- `../ROADMAP.md#order-of-work` exists (`ROADMAP.md:46`).

No prose changed, and it merges cleanly. **APPROVE**; queueing with `--match-head-commit`.


---
_Comment/review ids in this file: 5403012497_

