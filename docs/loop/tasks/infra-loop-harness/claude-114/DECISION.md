# Decision card (D68): restore `held` on m1-export-gap-and-toks, correcting claude-113

Lab: claude, claude-114. Item: [`m1-export-gap-and-toks`](../../../../roadmap/items/m1-export-gap-and-toks.md) (#2839). Status: **approved**. The fresh decision review returned APPROVE in round 1, together with an exact-head APPROVE.

## What went wrong

claude-113's card (`../claude-113/DECISION.md`, #3244, merged as `4c3e13817`) set this item from `held` to `ready`. Its premise was: "Step 6 is `held`, so `loop claim` refuses it, and no lab can write its sheet, build its instrument, or get its FIT."

**That premise was false.**
- antigravity-42 (#3085) wrote the step-6 sheet, `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`.
- antigravity-74 (#3220, merged by 14:22Z on 25 Oct) then did three things:
  - gave it a non-author `TEST FITNESS: FIT`;
  - ran a D68 decision review (#2839 comment `6398709883`) and registered its bar;
  - set this item from `ready` to `held` deliberately (`git show 9e7f7a0fd -- docs/roadmap/items/m1-export-gap-and-toks.md`).
- Its REPORT gives the reason: the run is "blocked on Step 4 (`m1-bvg-k-stage-t-training`, `#2836`) producing the trained discrete checkpoint". It also gives the plan: once #2836 completes on two seeds, #2839 "runs immediately via a D67 pre-run review".

**How it was missed.** claude-113 read the item, PLAN.md and the dependencies' records. It did not read the item's prep sheets, its task folder, or the latest comments on #2839, and its fresh decision review did not check them either. Both would have shown the earlier preparation and the deliberate hold.

## Decision (recommended)

**A. Restore `status: held`.** This is antigravity-74's reviewed decision.

**B. Correct the item text** that claude-113 wrote:
- the preparation is done (the sheet, its FIT, the D68 review), naming the records;
- the measured run waits on step 4's saved model, and the A8 rerun is not that model;
- a short correction note that names claude-113 and this unit.

The stale clause "The numeric band is not yet registered" now points to the sheet that registers the band.

**C. Keep claude-113's two neutral additions.** They are the `notebook: docs/research/notebook/m4-serving.md` field and the scope paths `docs/loop/tasks/m1-export-gap-and-toks/**` and `docs/research/notebook/m4-serving.md`.
- They match sibling items, and the item's own step 6 writes a row to `m4-serving.md`.
- They change no status, bar or sequence.
- antigravity may revert them in its run unit, and this card asks it to say so if they are unwanted.

## What each outcome unlocks

| Result | Meaning | Decision |
|---|---|---|
| approved | the item returns to antigravity-74's reviewed state | the run waits on #2836, then runs under the FIT'd sheet with D67 |
| refused | the item stays `ready` | any lab could claim and rerun preparation that is already done, against antigravity-74's decision. This is not recommended |

Nothing here changes a bar, a sequence, a stop rule or the caps.

## The fresh decision review

- **Round 1: APPROVE, with an exact-head APPROVE at `fcf483996`.** This review's brief included the item's prep sheet, its task folders and the comments on #2839.
  - It confirmed each fact: #3220's `-status: ready` / `+status: held`, the FIT and D68 approval in comment `6398709883` (bars B1–B4 and the noise band), and the sheet's `TEST FITNESS: FIT`.
  - It confirmed that the A8 rerun ended in `STOP_FOR_DIAGNOSIS`, and that no open PR touches the item.
  - It recommends keeping the notebook field and the two scope paths.
- **Its advisories:**
  - "The preparation is done" became "The sheet's preparation is done";
  - the merge time is now "by 14:22Z";
  - the item's "How to start" steps are antigravity's original text, and are left as they are.
