# REPORT: m2-pin-train-mask-sheet, claude-100

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2851. References #2030.

## What did not work, first

- **The decision review rejected the close, and it was right.** I proposed to close the item on its discrete-law branch, reading the 13 Oct discrete rung-0 sheet (`m1-discrete-rung0-training-2026-10-13.md`) as "the new discrete-law sheet" with a non-author FIT. The sheet is the Amendment 6 record's successor by lineage, and it carries the tar, mask and count pins. But it does not meet the done-when:
  - **It is not ready for a release decision.** Its Verdict puts three gates before any release: Codex qualifies the admission diff, Claude reviews it, and Kimi binds the identities. No record shows them done, and it says "Today's source still refuses corpus training".
  - **Its FIT is stale.** The design FIT is from 14 Oct on source `e7d926fbe`. The sheet marks fresh transfer unavailable, but `m2-fresh-heldout-panel` has since been done.
  - **It is the wrong shape for this item.** It is a 392-line M1 dev-panel sheet with no M2 bar, and it does not carry the count-reference source `2fd6358bb`.
  - **The 24 Oct owner relay** says a discrete-law sheet "is also wanted" and is to be filed as an item, so it treats the sheet as work still to do.
- **My cancel probe passed on the sheet's lineage and its FIT line alone.** It never read the sheet's own execution gates, which carry the answer. That is the lesson: a FIT line on a sheet is not the same as that sheet being ready.
- **So the item stays open (`status: ready`).** Its discrete-law branch needs a new one-page M2 sheet.
- **An advisory scope WARN (D76)** for the item-text note and the roadmap render. No notebook row: nothing was measured.

## What changed

- **`docs/roadmap/items/m2-pin-train-mask-sheet.md`:** the status is unchanged (`ready`). One dated note before "Done when" records why the 13 Oct sheet does not meet the discrete-law branch, and what does: a one-page M2 sheet on the current discrete law. It carries the mask, count and source pins, the M2 bar and the sealed fresh panel, with a non-author FIT at a current head.

## Measurements

None.

## Checks run at the head

- **The 13 Oct sheet:** its pins (lines 47-50 and 80-86) match the 25 Oct re-verification. Its Verdict (lines 368-392) lists the execution gates and "Today's source still refuses corpus training", and its fresh-transfer clause (lines 66-70) marks fresh transfer unavailable.
- `cargo xtask roadmap check`: 92 items OK, and the rendered page matches `main`.
- **Review history:**
  - Round 1 returned CHANGES and DECISION REVIEW: REJECT on the close, with the findings above.
  - The round-2 review is in the PR's review record.

## Skills used

- holding-the-goal: the done-when's discrete branch is real work, and the record now says what it is.

## Still open

- **The one-page discrete-law M2 sheet.** It needs the M2 bar, the current discrete trainer's law and CLI, the carried pins, the sealed fresh panel from `m2-fresh-heldout-panel`, and a non-author FIT.
- **The D37 sheet** needs `float-training-module` (#2823), then `d11-exporter-float-trunk` (#2820), then its fresh slice, then a FIT.

## Plain summary

I tried to close the M2 mask-sheet item by pointing at an older discrete-training sheet. The review showed that sheet isn't ready: its own gates were never cleared, and its approval is stale. The item stays open, and it now says exactly what new sheet would finish it.
