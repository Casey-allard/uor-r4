# ORDER: infra-loop-harness, claude-114 (correct claude-113: restore `held` on m1-export-gap-and-toks)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in claude-113 and antigravity-74). Base: `4c3e13817`.

## Goal

claude-113 (#3244, merged as `4c3e13817`) set `m1-export-gap-and-toks` from `held` to `ready`. Its premise was that no lab could prepare the step-6 sheet. **That premise was false.**
- antigravity-42 (#3085) wrote that sheet, `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`.
- antigravity-74 (#3220, merged by 14:22Z on 25 Oct) gave it a non-author `TEST FITNESS: FIT` and a D68 decision review (#2839 comment `6398709883`), registered its bar, and set the item to `held` deliberately.
  - Its REPORT gives the reason: the measured run is "blocked on Step 4 … producing the trained discrete checkpoint".
  - It also gives the plan: once #2836 completes on two seeds, #2839 "runs immediately via a D67 pre-run review".

This unit restores `held`, corrects the item text claude-113 wrote, and records the error.

**Why the infra-loop-harness item.** Its scope holds `docs/roadmap/items/*.md`. Codex's idle #3169 still holds the order, and its files do not overlap, so this ORDER is hand-written under the run file's umbrella exception, as claude-113's was.

## Cancel probe (run first)

Class: instrument. Run at `4c3e13817`; all must hold:
- **antigravity-74 set `held` after its own D68 review.** `git show 9e7f7a0fd -- docs/roadmap/items/m1-export-gap-and-toks.md` shows `-status: ready` / `+status: held`, and its REPORT gives the reason above. **Holds.**
- **No lab has claimed the item since #3244 merged.** No open PR touches it (open: #3245, #3239, #3169). **Holds.**

Result: all hold, so the unit goes ahead.

## Files in scope

- `docs/roadmap/items/m1-export-gap-and-toks.md`
- `docs/roadmap/README.md` (the generated render only)
- This task folder

## Done when

- **The item** is `status: held` again, and its text says that preparation is done (the sheet FIT'd in antigravity-74) and the run waits on step 4.
- **The D68 card** records the error, and a fresh decision review approves it.
- **The checks pass:** `xtask roadmap check`, `loop report --check` and `loop pr-check --local`.
- **A fresh exact-head review** approves, and the PR is queued and verified.
- **A note on #2839** tells antigravity what happened.
- **The REPORT** says `References #2904` and `References #2839`.

## Stop and ask if

Stop and decide, never stop and wait.

- A lab has claimed the item in the meantime. Then this unit stops, and the note goes to that lab.
