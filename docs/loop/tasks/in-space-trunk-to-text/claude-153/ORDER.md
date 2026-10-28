# ORDER: in-space-trunk-to-text, claude-153

Lab: claude. Item: [in-space-trunk-to-text](../../../../roadmap/items/in-space-trunk-to-text.md). Base: `e2f15d3fc`.

## Goal

Casey, 28 Oct 2026, in claude's chat, answering whether to run the combination claude-152 left unrun:

> "Yes, pre-register and run the combined retrain: cell objective, pinned TRAIN, width 512 with 8 read heads, seeds 1 and 2, plus a width-512 no-read control."

claude-152 ran the three changes one at a time. Its readings, by its sheet:
- the cell objective at width 256 was cell C: copies rose, openings did not;
- `balp`, the 13 times larger TRAIN, was cell D with flag F: fit was lost;
- width 512 was cell C, but only on `balp`, against the failed (b).

So width 512 was never run on the TRAIN that works. This unit runs it there:

- **(w):** the cell objective, the pinned TRAIN, `--width 512 --hamming-read 8`, at seeds 1 and 2. Its reference is claude-152's (a): the same, at width 256 with 4 heads.
- **(w0):** (w) with no read (`--hamming-read 0`), at seeds 1 and 2. It is (w)'s no-read floor for copies, on the same TRAIN and at the same width.

They are scored by claude-152's prompt-use test, on the pinned DEV, at every checkpoint.

The item's own done-when (unchanged by this unit):

> On the pinned DEV panel, decoded as the whole table's nearest token:
>
> 1. Reply-opening top-1 is at least `197` of 1,226. That is always naming DEV's commonest opening token (`161`) plus three binomial standard deviations (`36`). The same checkpoint's whole-panel top-1 must be at or above the M1 count table's `7,002`.
> 2. On that same checkpoint, the relative reach from `--reach`, `R16 = shift(16) / shift(0)`, is at least `0.05`. The 27 Oct best checkpoint reads `0.011`. An untrained trunk reads more, but it cannot meet (1).
> 3. Greedy replies to the six fixed DEV prompts are recorded beside the true replies, at every checkpoint, in the lab record.
>
> Lead with tokens per second, decode and context separately.

## Cancel probe (run first)

Run 2026-10-28 against `origin/main` `e2f15d3fcc91a6042b77fbcf717a79c39eaf215d`. **Not cancelled (instrument: a search of the store, the notebook and the open PRs).**

- **No such run exists.** `--width` reached main only in claude-152's merge, `e2f15d3fc` (`git log -S'"--width"'` on `train-in-space.rs`). Before it, every trunk was width 256. claude-152's only width-512 runs are (c), on `balp` (`pod3-c-s1.tar`, `pod5-c-s2.tar`). The notebook's one width-512 entry is that row.
- **No code is needed.** Every flag exists on main. `git diff d102c594 e2f15d3fc -- crates/ Cargo.lock` is empty, so main's training code is byte for byte the code claude-152's four pods built and tested.
- **No overlap.** The only open PR on this item is this unit's own draft, #3341.

## Files in scope

This unit changes no code:

- `docs/harness/prep/in-space-trunk-to-text*`
- `docs/loop/tasks/in-space-trunk-to-text/**`
- `docs/research/notebook/float-training.md`

## Done when

1. **The prep sheet** `docs/harness/prep/in-space-trunk-to-text-wide-cell-2026-10-28.md`, with a fresh non-author FIT and a fresh pre-run review (CHARTER §4.5, D67), each logged on #2028.
2. **The pre-registered runs:** (w) and (w0), each at seeds 1 and 2, at main's `e2f15d3f`, each pod building and testing its own binary. Results sealed in `caseyallard/uor-r4-store:results/claude/wide-cell-2026-10-28/` and read back before any pod comes down.
3. **The REPORT** leads with tokens/s, training and decode against context separately. It gives the sheet's cell for (w), and the item's done-when 1 and 2 on each final checkpoint, with the greedy replies (done-when 3).
4. **One notebook row** in `docs/research/notebook/float-training.md`, and the result on main.

## Stop and decide (never stop and wait)

- **A probe projects an arm past 2 hours:** profile first, and record why in the sheet before training.
- **A run diverges or crashes:** stop it and classify it, instrument or object, then rerun that seed. A missing number is never read as a fail.
- **I-2 fails** (step 0 differs from claude-152's width-512 step 0 at the same seed): the build or the data is not what the sheet says. Stop every reading of that pod and find out why before any rerun.
- **No 5090 offer, or the caps are reached:** run (w) first, at both seeds, and (w0) after them on the same pods.
