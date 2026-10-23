# ORDER: m2-pin-train-mask-sheet, deepseek-42 (is the Amendment 6 sheet live under D37? re-pin and ask)

Lab: deepseek. Item: [m2-pin-train-mask-sheet](../../../../roadmap/items/m2-pin-train-mask-sheet.md) (#2851). Base: `8720c5d6d`. Item PR (partial: `References #2851`; the item stays `ready` until the PM answer and a non-author FIT).

## Goal

Settle what the pinned M1 rung-0 sheet is today: Amendment 7 froze it under D32, but **D37 (2026-10-18) revised D32's learning-path float ban** — floating point is legal in offline training with a reported export gap. This unit (a) verifies the Amendment 6 pins against current `main`, (b) writes a refreshed one-page sheet that restates them on the D37 basis, and (c) posts the decision card asking the PM to record whether that sheet is live under D37 or superseded by a discrete-law sheet. It runs nothing and releases no hold.

## Cancel probe (run first)

Class **instrument** (docs + decision card; no GPU). At the base: `docs/harness/prep/m2-pin-train-mask-sheet*` does not exist; the old sheet still carries the Amendment 7 freeze with no recorded D37 answer on #2851; the four pins are checkable (source commits exist, CLI flags exist, mask sha and expected count appear in code/archive). Task stands.

## Files in scope

- `docs/harness/prep/m2-pin-train-mask-sheet*` (the refreshed sheet)
- `docs/labs/m2-pin-train-mask-sheet/**` (lab record)
- this task folder

## Done when

- The refreshed sheet carries all eight required blocks, the four Amendment 6 pins re-verified against current `main`, the D37 legal basis, and `TEST FITNESS: PENDING` (it needs a non-author FIT after the PM answer).
- A decision card on #2851/#2028 asks the PM to record live-under-D37 or superseded, with a recommendation and the evidence; no bar, hold or spend changes.
- The lab record states what was verified, what remains (PM answer, non-author FIT), and that training/rental stays HELD.
- `prep-check` shows the new sheet complete (fitness PENDING), `loop report --check`, `loop pr-check --local`, `loop gates --full` and a fresh-context review record pass at the head.

## Stop and ask if

- The PM's answer is needed to *run* anything (it is not: this sheet runs nothing).
- A pin does not verify (then say which, and stop rather than restating it).