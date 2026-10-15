# #2405 — M1 discrete rung-0 sheet Amendment 1: spec/dose/cost pinned from landed D1/D2 (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T00:59:28Z
- **Updated:** 2026-10-14T01:01:32Z
- **Closed:** 2026-10-14T01:00:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2405
- **Merged:** 2026-10-14T01:00:38Z
- **Merge commit:** 4b41c7143186b9cb37816629f7487029ace88805
- **Base / Head:** main / UOR-Foundation:kimi/discrete-rung0-amendment1-20261014
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Amendment 1 to `docs/harness/prep/m1-discrete-rung0-training-2026-10-13.md`, against merged D1/D2 (PR #2400, `6ba08ab3f`, executable `e7d926fbe`, 25/25 finite controls PASS):

1. **Spec pinned (D3 resolved):** V4096 / B4 / C192 / batch 64+tail / writer K8 / 16,404 coordinates. B4 lifts the blocks=2 alias bound (audit 1.3 / Codex item 1).
2. **Schedule pinned:** one ascending coordinate per batch, common across seeds, checkpointed resume.
3. **Anti-vacuity guard redefined:** strict distance-win rate >= max(5x untrained, 1%) — supersedes the margin-based zero-hinge guard; zero-hinge/hinge distribution demoted to diagnostics; no unbuilt histogram claimed (6158076739).
4. **Steps unblocked:** init → in-run masked-TRAIN count reference → coordinate schedule over BOS-segmented windows → propose/choose/apply → dev scoring at 25/50/75/100% of the pass.
5. **Dose/cost from measured D2:** forward 37.8ms/64-row batch (1,691 rows/s); proposal ~3.9s; dose = 1 coordinate pass/seed ≈ 17.8h, ~$32.3 both seeds at $0.908/h quoted — UNVERIFIED projection, within caps.
6. **Outcome decisions:** KEEP/iterate/blocker defined; named successor configurations (stage-indexed BRANCH, orbit-indexed candidates) recorded, not adopted.
7. Scope unchanged: Route/VectorBlock only; Gate/flock excluded; no FP (zeta addendum unused). Training HELD; D4 owner release remains the execution gate.

Verdict: TEST FITNESS: PENDING — review requested from Claude lab, independent non-author fallback documented. Milestones M1 `next` updated. References #2029

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

