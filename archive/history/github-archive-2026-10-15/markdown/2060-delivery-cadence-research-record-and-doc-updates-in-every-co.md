# #2060 — Delivery cadence: research record and doc updates in every completing PR; PR checklist template

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T17:08:47Z
- **Updated:** 2026-10-09T17:10:03Z
- **Closed:** 2026-10-09T17:09:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2060
- **Merged:** 2026-10-09T17:09:49Z
- **Merge commit:** 5c14e047162f6747e7c8d66712ca24abfa5f0280
- **Base / Head:** main / UOR-Foundation:claude/delivery-cadence
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner direction (9 October 2026): everything the labs accomplish is logged canonically, and every document a change touches stays current.

- **`CONTRIBUTING.md`:** the workflow now runs refresh → claim → branch → check → **record and update docs** → PR → review → merge → verify → report → clean up. There's a new **Delivery cadence** section:
  - the canonical research record: a dated `docs/labs/<topic>-<date>/README.md` plus a newest-first `current-state.md` entry with **Next:**, negatives included;
  - update every document the change makes stale (with the `git grep` to find them);
  - update STATUS, ROADMAP and the #2028 table when state changes;
  - update the README only when something a reader relies on changed, in its existing sections, never as a log;
  - honest wording.

  The stale `codex/lab-state` mentions are removed.
- **`AGENTS.md`:** a short October 9 block pointing to the cadence, so every lab sees it on entry.
- **`.github/pull_request_template.md`:** the checklist appears on every new PR.

Checks: claim wording passes; CONTRIBUTING links resolve.

References #2028

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6085607353 — 2026-10-09T17:08:56Z — Casey-allard

Exact-head self-review at `aff4983ac71d98c6fd51ca7224c43101b436d4d1`: read the CONTRIBUTING diff and the AGENTS block; anchor slug matches the heading; claim wording passes.


---
_Comment/review ids in this file: 6085607353_

