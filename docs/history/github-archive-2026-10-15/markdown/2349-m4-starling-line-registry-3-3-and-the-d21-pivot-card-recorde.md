# #2349 — M4 starling line: registry 3/3 and the D21 pivot card recorded (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T16:55:04Z
- **Updated:** 2026-10-13T16:57:02Z
- **Closed:** 2026-10-13T16:56:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2349
- **Merged:** 2026-10-13T16:56:40Z
- **Merge commit:** db506673abcee390db5f88dd8e89ee76d377e903
- **Base / Head:** main / UOR-Foundation:claude/starling-pivot-card-2026-10-13
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #2032 · D21 / D22 / D28

**Line:** starling read · count 3/3 · pivot card filed · headline unchanged. Docs and registry only; no code, no compute.

- **Registry:** `docs/harness/milestones.json` moves the starling line 2/3 → **3/3**. #2338, the rung-0b source PR, was the third PR that did not move the headline, and the old note preserved the historical count.
- **Pointers:** `docs/mechanisms/starling-read.md` and `current-state.md` link the [pivot card](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6150703214).
- **The card:** continue with ONE decisive run (design §5) with amendments P1 (the native read), P2 (key-address credit first) and P3 (no RoPE arm, no transformer comparison). It is gated on rung 0, P2's FIT and the owner's hold release.

**Checks:** `jq` validates the JSON, and `uor-harness check-formats` passes.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6150715232 — 2026-10-13T16:55:10Z — Casey-allard

**Exact-head APPROVE at `b1ed12330`** (self-review, owner rule 09-30). Docs/registry only; `jq empty` OK, `uor-harness check-formats` 8/8 OK at this head.


---
_Comment/review ids in this file: 6150715232_

