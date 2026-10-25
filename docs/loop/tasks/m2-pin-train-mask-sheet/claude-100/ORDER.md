# ORDER: m2-pin-train-mask-sheet, claude-100 (close on the done-when's discrete-law branch)

Lab: claude. Item: [m2-pin-train-mask-sheet](../../../../roadmap/items/m2-pin-train-mask-sheet.md) (#2851). Base: `54a88ae0b`.

## Goal

The item's done-when reads:

> A recorded answer from the PM or owner on whether the Amendment 6 sheet is live under D37 or superseded, and a one-page sheet that is ready for an owner hold-release decision: either the pins re-verified against current main and restated in a refreshed sheet, or the pins carried into the new discrete-law sheet, with a non-author FIT. The sheet itself runs nothing: no training or optimizer update is authorised by it.

claude-99 (#3196) recorded the owner's answer and left the discrete-law branch open. This unit checks whether that branch is already met on `main`. If it is, the item is set to done with `Closes #2851`.

## Cancel probe (run first)

**Result at `54a88ae0b`:** the branch is met. All three must hold, or the unit is cancelled.
1. **The owner's answer is recorded.** Casey, on 24 Oct (#2028 comment `6379430093`, item 3): "I want both because we do not know which we will ultimately need." It is quoted in the item's 25 Oct note and in the D37 sheet's status line.
2. **A discrete-law sheet carries the pins.** `docs/harness/prep/m1-discrete-rung0-training-2026-10-13.md` names itself the successor ("supersedes: `m1-rank-margin-rung0-2026-10-13.md` … its lawful pins carry over here as marked"). It pins:
   - the parent `data/ft-balp.tar` (sha256 `b1d22113…473f`, the digest re-verified on 25 Oct);
   - the member `mw-balp/train/response_mask.u8` (7,512,797 bytes, sha256 `b5e7352d…49e3`);
   - the count, 2,425,925 response-masked targets;
   - the tokens digest;
   - the dev pins;
   - its own discrete executable source (`e7d926fbe` at merge `6ba08ab3f`).

   Its CLI is the discrete law's own, as the item requires ("the CLI line changes with the learning law and must not be reused as written").
3. **It has a non-author FIT.** Its Verdict reads `TEST FITNESS: FIT (design), Amendment 2`, from two independent non-author reviews (Claude `6159328025`/`6159394631`, Codex `6159308818`/`6159343268`). It runs nothing ("Nothing executes"), and it lists the execution gates that a hold-release decision would act on.

**Class:** records. The item's question and its sheet were both settled earlier; nothing recorded the item as done.

## Files in scope

- `docs/roadmap/items/m2-pin-train-mask-sheet.md` (its status line and one "Done" note)
- `docs/roadmap/README.md` (re-rendered)
- this task folder

## Done when

- The item is `status: done`, and its note names the sheet, the pins and the FIT that meet the done-when.
- The note also says the D37 sheet goes on separately, as the owner wants both, and that its FIT follows `float-training-module` (#2823) and `d11-exporter-float-trunk` (#2820).
- A fresh decision review (D68, a status change) passes and is logged on #2851.
- `loop report --check` and `loop pr-check --local` pass, and a fresh-context review counts at the exact head.
- The REPORT says `Closes #2851`.

## Stop and ask if

- The decision review finds the discrete sheet does not meet the done-when as written: it is not a one-page sheet ready for a hold-release decision, or its FIT is stale.

## Outcome (after the decision review)

The fresh decision review REJECTED the close. The 13 Oct sheet's own Verdict gates (qualify the admission diff, review it, bind the identities) are not on record as done. Its 14 Oct FIT is stale, and it is an M1 sheet with no M2 bar. The item stays `status: ready`. This unit now records that finding in the item, so the next lab starts from it. The done-when above is replaced by: the item note records the finding; the decision review passes on that; the checks pass; and the REPORT says `References #2851`.
