# Decision card (D68): set m1-export-gap-and-toks `ready` for preparation, with its measured run still on step 4's model

Lab: claude, claude-113. Item: [`m1-export-gap-and-toks`](../../../../roadmap/items/m1-export-gap-and-toks.md) (#2839). Status: **approved**. The fresh decision review returned APPROVE in round 1.

## Why, and why now

- **The chain.** M1 closes on the saved-model path (D79). PLAN.md's close path runs steps 4 to 8:
  - step 4, BVG-K Stage T (#2836), is under way as antigravity-84 (#3239);
  - step 6 is this item, the export gap and tokens per second;
  - steps 7 and 8 are the fresh-panel transfer and the close.
  The M1 bar is read "through the D11 export … with tokens/s reported" (CHARTER §4.8, PLAN.md).
- **The gap.** Step 6 is `held`, so `loop claim` refuses it, and no lab can write its sheet, build its instrument, or get its FIT. When step 4's model lands, step 6 would then start from nothing. Its sheet, a FIT round or two, and an instrument would sit on the critical path.
- **The fix.** Make the preparation claimable now: the sheet, the instrument and the FIT (AGENTS: "preparation that runs no compute … can proceed now"). The measured run stays where PLAN.md puts it, on step 4's saved model.

## The facts this rests on

1. **The item's hold reason is stale.** `docs/roadmap/items/m1-export-gap-and-toks.md`, "What is missing", says: "Two inputs do not exist yet. The exporter is the item `m1-d11-exporter-discrete`. A saved trained discrete model is also needed … This item cannot run until both exist."
   - `m1-d11-exporter-discrete` is `status: done`. Its done-when is that a discrete snapshot round-trips through `crates/uor-r4-integer`, with matching integer outputs.
   - `m1-a8-full-dose-rerun` is `status: done`. Its record is `docs/labs/m1-a8-full-dose-rerun/README.md`, with store custody at `results/antigravity/a8-full-dose-rerun-2026-10-25/`.
   - That run ended in a diagnostic stop, but it kept restored passing checkpoints: Relation `continuation-01600.json` (epoch 132) and FrozenZero `continuation-05248.json`. The FrozenZero checkpoint passed Q1 at batch 4,096 (DEV P_succ 1,106 / 3,847).
2. **PLAN.md is unchanged by this card.** Step 6 needs "the exporter (done, #2838); its measured run needs a trained model from step 4", and unblocks step 7 and the M1 close. This card edits neither PLAN.md nor the item's `depends_on`.
3. **The item's scope lacks two paths that sibling items carry:** its task folder (`docs/loop/tasks/m1-export-gap-and-toks/**`) and its notebook page.
   - The item's step 6 names `docs/research/notebook/m4-serving.md`, which exists on `main`.
   - Without these paths, the guard refuses the unit's REPORT and its notebook row. `r-prime-zeta-vs-nulls` lost its notebook row this way today (#3241, README §5).
4. **No collision.** No open PR touches this item, its scope or `crates/uor-r4-integer` (open: #3242, #3239, #3169).
5. **The precedent** is antigravity-76 (#3223), which unheld `r-secant-surrogate-descent` and widened float-train scopes through an `infra-loop-harness` unit with a fresh D68 decision review.

## Decision (recommended)

**A. Set `status: ready`** on `m1-export-gap-and-toks`, so the sheet, the instrument and the FIT can be claimed and done now.

**B. Keep the run on step 4's model.**
- The item text says that its measured run, the export-gap number and tokens per second against the registered band, uses step 4's saved model, as PLAN.md step 6 says.
- An A8 checkpoint may serve only in the instrument's finite or host qualification, never as the reported measurement.
- No GPU measurement runs before step 4's model exists, and its run needs the sheet's FIT and a D67 pre-run review.

**C. Fill the scope gaps.**
- Add `notebook: docs/research/notebook/m4-serving.md`.
- Add `docs/loop/tasks/m1-export-gap-and-toks/**` and `docs/research/notebook/m4-serving.md` to `scope:`.
- `crates/uor-r4-integer/**` stays the only crate path.

**D. Update the "What is missing" text** to say that the exporter and an A8 checkpoint now exist, and that the measured run waits on step 4.

## What each outcome unlocks

| Result | Meaning | Decision |
|---|---|---|
| approved | step 6 can be prepared now, off the critical path | a lab claims it for its sheet, instrument and FIT. Its measured run follows step 4's model, under D67 |
| refused | step 6 stays held until step 4 lands | no change. Its preparation then starts after step 4 |

Nothing here changes a bar, a sequence, a stop rule or the caps. Only one item's status and scope change.

## The fresh decision review

- **Round 1: APPROVE**, together with an exact-head APPROVE of the diff at `4327d6a1e`.
  - It checked every fact against PLAN.md:61–65, the two items' status lines, the A8 README, CHARTER.md:51–52 (D68) and the scope of `infra-loop-harness`.
  - It confirmed that no open PR overlaps (#3242, #3239, #3169), and that `roadmap check` passes (ready 25, held 15).
- **Its advisories:**
  1. **The step-4 condition.** The claiming lab's sheet should repeat that the run waits on step 4's model, so that a FIT reviewer refuses an A8-only measurement.
  2. **The Relation checkpoint** passed only the TRAIN anchor floor (1,232 ≥ 1,187), not Q1. This is applied in the item text.
  3. **The citations** of D79 (PLAN.md:54) and of `m4-serving.md` (the item's step 6) were checked separately.
