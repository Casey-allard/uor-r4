# #2351 — M4: the murmuration read replaces the rung-0b rank read (design record) (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T17:17:11Z
- **Updated:** 2026-10-13T17:19:06Z
- **Closed:** 2026-10-13T17:18:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2351
- **Merged:** 2026-10-13T17:18:48Z
- **Merge commit:** a7a4f853d72a600dce46da66fd8bcc2964684c3c
- **Base / Head:** main / UOR-Foundation:claude/murmuration-design-2026-10-13
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #2032 · D21 / D22 / D28 · owner-approved 13 October

**Line:** starling read · count 3/3 · this is the decisive run's mechanism · headline unchanged. Docs only: no code, no compute.

- **New record** `docs/labs/murmuration-design-2026-10-13/`:
  - a read-through of SpiralCore v69's source (four agents): what it computes and what it doesn't — exact transport, no learning, no flock rule;
  - the murmurations paper's findings;
  - the condensed design (card [6151093135](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6151093135)).
- **`docs/mechanisms/starling-read.md`:**
  - **corrects** the overstated Kuramoto line: SYNC-10 shows only that the trajectories differ;
  - adds the murmuration section;
  - settles the shared-address question;
  - retires the rank table as evidence.
- **Registry and current state:** `milestones.json` M4 `next` updated, plus a new current-state entry.

**Checks:** `jq` validates the JSON, and `uor-harness check-formats` passes.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6151117505 — 2026-10-13T17:17:17Z — Casey-allard

**Exact-head APPROVE at `7d4928705`** (self-review, owner rule 09-30). Docs/registry only; `jq empty` OK and `uor-harness check-formats` OK at this head.


---
_Comment/review ids in this file: 6151117505_

