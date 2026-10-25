# REPORT: infra-loop-harness, claude-82

Lab: claude. Order: [ORDER.md](ORDER.md).
Closes #2828. References #2904.

## What did not work, first

- **My first plan page said `m1-bvg-k-block-apply-c2` had the wrong status. It didn't.** Its C2 controls are met, but the one-page Stage T sheet is still to do, so `ready` is right. Only its "until FIT and an owner release" wording is stale. D67 supersedes owner-release wording in item bodies; the bodies are not rewritten here.
- **`m2-donor-corpus-generation` stays `held`.** CHARTER §5.1 keeps D41's donor pass as the one run that needs the owner at dispatch (§3.10), so D67 does not release it. The ORDER's stop rule, a held item that needs the owner for another reason, is met for this one item only. It is handled by leaving it held, not by stopping the unit.
- **`loop render` reads the committed HEAD, not the working tree.** The first render showed 31 ready items, so I committed the item changes and rendered again. The render also restamps `AGENTS.md` and `LESSONS.md` with no content change. Both are restored, so no owner path changes for a timestamp.
- **The change log's "Entries in this log" count was stale.** It said 3 against 8 dated entries. It now says 9, counting this one.
- **The M1 path could never be handed out.**
  - **Cause.** `loop next` offers only ready items that have a `scope:`, and `loop claim` refuses an item without one. Five of the eight M1 steps had no scope: W2, the A8 qualification, Stage T, the A8 full dose and the M1 close. So did all 25 held items.
  - **Fix.** This unit gives each of the 25 a default scope: its prep sheet, lab folder and task folder, plus `crates/uor-r4-discrete/**` for the m1/m2 items. Scope is advisory under D76, so the default only makes them claimable.
  - **The "any" order is still alphabetical**, which is the next unit's fix.
- **The fresh review (round 1, CHANGES) found:**
  1. **The released items still said to wait.** All 25 kept the rule bullet "**Held** (§5.1): training stays held until the owner releases it". They now carry the D67 rule, and only `m2-donor-corpus-generation` keeps the owner-at-dispatch hold.
  2. **`d41-driver-math-pilot` is not the donor pass.** It is the driver-math optimization and a ~1M-token pilot ahead of distillation, so it stays `ready`.
  3. **Step 6 overstated its dependency.** `PLAN.md` said it needs a trained model, but `m1-export-gap-and-toks` depends only on the done exporter. The cell now says that, and that its measured run needs step 4's model.
  4. **Step 1 overstated what it unblocks.** W4 is unblocked only together with step 4, and only if W2 gives H_law and W3 fails.
- **`PLAN.md` and `DECISIONS.md` conflict with open work.** Codex's #3169 changes a `PLAN.md` premise row and adds D75, and #3170 adds D76. This branch merges `main` before queue, so D76–D79 sit in order.

## What changed

- **`PLAN.md`:**
  - It is now the one plan (D77): the M1 path in eight steps, ranked by unblock count with issue numbers, the route M2–M8, and the side lines.
  - Five owner-direction rows for 24 Oct.
  - The approved sequences without the "queued on a lease" and "PR 8 in flight" text.
  - Two "Not doing" lines: no infrastructure or doc-clarification unit unless a milestone item needs it, and no certification chase on a closed configuration.
  - The Premises and Owner directions tables keep their headers for `loop next`.
- **Items:**
  - 25 move from `held` to `ready`. `legal-construction-exact-solve` is `done`, with its result written under Done when.
  - `m1-trunk-assembly-and-training` is milestone M2.
  - `docs/roadmap/README.md` is re-rendered. The tally is ready 55, held 1, decision 1, done 34.
- **Pointers:**
  - CHARTER §5.3 and §5.8 name `PLAN.md`.
  - The README's stages run on pre-run reviews.
  - `ROADMAP.md`'s caption and paragraph name D67 and `PLAN.md`.
  - `docs/README.md`'s three `NOW.md` links point to `PLAN.md`.
  - The loop README's step 6 and its owner-releases paragraph follow D67.
- **Records:**
  - D77 (one plan, held work released), D78 (#2828 closed on its result) and D79 (M1 without the trunk) quote the owner.
  - `docs/rules/changes.md` has the 25 Oct entry.
  - The legal-construction mechanism page notes the closure.
- **`docs/harness/milestones.json`:**
  - M1's headline was first changed to the §4.8 bar, then restored, because the harness's `prgates` headline gate and its test read M1's numeric registry headline (`reply acceptable 37/232`). Moving M1's registry headline to the §4.8 bar needs that gate changed too, which is its own unit. `PLAN.md` states the bar.
  - M1 and M4 `training_hold` are false.
  - M4 is `not-started` instead of `blocked-on-owner`.
  - M2 keeps its hold for the donor pass.
- **Rendered:** `STATE.md` and `HANDOFF.md` are re-rendered (HANDOFF ready 55).

## Measurements

- `cargo xtask roadmap check`: 91 items OK. Ready 55, held 1, decision 1, done 34. M1 has 20 items, M2 has 8.

## Checks run at the head

- `cargo xtask roadmap check`: PASS. `cargo xtask roadmap render`: written.
- `uor-harness loop report --check`, `loop pr-check --local` and `loop gates --quick`: run before queue, recorded on the PR.

## Skills used

- None.

## Still open

- Ranking `loop next` by unblock count toward a milestone close is its own unit, and the next one. Until it lands, `loop next` can still hand out a ready research item ahead of step 1 of the M1 path.
- 26 item bodies still say "until FIT and an owner release". D67 supersedes that wording; rewriting the bodies is not needed to run.

## Plain summary

The project now has one plan, `PLAN.md`. It lists the eight steps to close M1, in the order that unblocks the most work, and every other plan document points to it. The work that had been waiting for an owner release since D67 is open again, except the one donor pass the charter keeps for the owner. The old legal-construction solve is closed on its near-optimal result, and the trunk moves to M2.
