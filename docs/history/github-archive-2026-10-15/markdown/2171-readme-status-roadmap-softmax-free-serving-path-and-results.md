# #2171 — README/STATUS/ROADMAP: softmax-free serving path and results, M1 memory numbers, D22 framing (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T18:08:02Z
- **Updated:** 2026-10-10T18:21:02Z
- **Closed:** 2026-10-10T18:20:46Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2171
- **Merged:** 2026-10-10T18:20:46Z
- **Merge commit:** 7e49be4b31f8327f033d7606df745129af771ff6
- **Base / Head:** main / UOR-Foundation:claude/readme-sync
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: documentation sync (owner request: the README duty of the delivery cadence) · count n/a · headline unchanged (softmax at runtime yes)

## What and why

The owner found that the README had not followed two days of mechanism and result changes. This brings the reader-facing pages up to date:
- **README, Where we are:**
  - The M1 row gets the 29M chat line's memory numbers: v5 10 → 20/40 with a 10 % recall mix at no measurable reply cost (#2151, adopted as base under D22); served 1.20–1.24 BPB (#2163); product-key memory trained into a saved model (#2168).
  - The M4 row: softmax at runtime is still yes. D11 now serves softmax-free flock reads, but every trained configuration misses the 0.01 bar.
- **README, What works today:** a new capability row for training and serving softmax-free flock reads (schema /3, BitCode, audit FULL PASS).
- **README, Serving:** a softmax-free reads paragraph.
- **README, Results so far:**
  - a softmax-free reads row (B +0.040, D +0.259, L +0.0185, W +0.0172; support 17 → 66 buys only 0.0013);
  - an M1 memory and read-binding row.
- **README, negatives:** a D22 note (a negative closes a configuration, not a mechanism; four closures reopened) with a link to the mechanism briefs.
- **STATUS.md:** the softmax-free reads and BitCode rows go from "pre-registered" or "planned" to measured.
- **ROADMAP.md:** the M4 row.

## Checklist
- [x] Every number traceable to a merged PR or a #2032 card (W's result PR follows)
- [x] `check_claim_wording.py` passes
- [x] Docs only

References #2032

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6100601912 — 2026-10-10T18:08:04Z — Casey-allard

Exact-head self-review at `c0c3f5ddf1c1f80e414e70ab0b69319db4031de6`: docs only, based on fresh origin/main; the claim-wording gate passes; the Markdown table rows were checked (no stray pipes).


---
_Comment/review ids in this file: 6100601912_

