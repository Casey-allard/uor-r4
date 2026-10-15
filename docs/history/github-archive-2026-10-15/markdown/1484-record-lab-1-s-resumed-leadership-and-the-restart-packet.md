# #1484 — Record Lab 1's resumed leadership and the restart packet

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T01:45:33Z
- **Updated:** 2026-10-09T13:08:56Z
- **Closed:** 2026-09-29T01:47:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1484
- **Merged:** 2026-09-29T01:47:29Z
- **Merge commit:** e3f9117909dad40cfe4266276aed4775ffdc20bc
- **Base / Head:** main / UOR-Foundation:lab/claude/leadership-resumed
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #973.

**Class C (organization and merge-criteria text), documents only.** The owner directed this change and approved its merge in chat at about 01:45 UTC on September 29 with the words "ALLOW MERGE". That approval is recorded here as the owner's ratification of the change to the merge criteria.

## What changes

- **The canonical handover.** The ROADMAP section "Temporary integration handover, September 28, 22:45 UTC" is replaced by **"Lab 1 leadership and restart packet"**, one page long:
  - the lead and lanes: Claude/Lab 1; Kimi as Lab 1's research-integration steward; Lab 2 (DeepSeek/OpenCode); Lab 3 (Anti-Gravity);
  - the one artifact contract;
  - the actual bottleneck (key formation) and the next scientific decision;
  - live state: merged and open PRs, the running job, pinned models;
  - the rules in force;
  - the restart procedure.
- **Astra's record is preserved** in `docs/integration/astra-temporary-cover-2026-09-28.md`. The handover text from ROADMAP, current-state and project-track at `9869229b` is copied word for word, except that headings are demoted and relative links are rewritten. The handback is on [#973](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5881962511), at 01:35 UTC.
- **README, current-state and project-track** now link to the new anchor, `#lab-1-leadership-and-restart-packet`. No reference to the old anchor remains (`git grep`).
- **Policy:**
  - "Every PR is driven to merge" is kept.
  - "Lab 1 enables auto-merge as soon as a PR is ready" is replaced by the owner's rule of 2026-09-29, quoted: *"No unconditional auto-merge, direct main push, admin bypass or force-push of shared work. Recheck the exact approved head and blockers before merging; verify the delivered patch and notify the consumer."*
  - The JSON contract is unchanged. Its `merge_path` already reads `merge_queue_when_clean_then_verify_squash_diff_equals_pr_diff_and_notify_consumer`.

## Checks

- `python3 scripts/check_claim_wording.py`: passed.
- `git grep -n temporary-integration-handover -- '*.md'`: no matches.
- No code, model or result changes. The only measured statements are the existing merged numbers, quoted with their PRs.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

