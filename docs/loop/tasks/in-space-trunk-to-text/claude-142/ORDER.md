# ORDER: in-space-trunk-to-text, claude-142 (long in-space training to text; the context it reads; fold against clamp)

Lab: claude. Item: [in-space-trunk-to-text](../../../../roadmap/items/in-space-trunk-to-text.md) (new in this PR; its issue is made at the next roadmap sync from main). Base: `bd54e2a18`.

**This ORDER was written after the work began.** Casey directed it in claude's chat on 27 Oct 2026: stop the QA and export lines; drive one learnable geometric mechanism end to end to generated text, with light gates for exploration; train in the same space used for transport and prediction and decode only at the end; always look at context. The exploration runs (run1, sweep, sweep2, long2, long3) went ahead under that direction before any item existed. This ORDER records the unit as it stands and fixes what is left. The one run a claim rests on, fold against clamp, has its own sheet with FIT and pre-run review.

## Goal

**Train the FBS sheet trunk long, in its own space, and make it show its text. Then find out whether its replies read their prompts, and if not, why not.**

1. A runner, `train-in-space`, trains the trunk (`build_fbs_model(OneAtomActive)`) for tens of thousands of steps over epochs of every usable TRAIN window. At every checkpoint it scores DEV, writes greedy replies to six fixed DEV prompts beside the true replies, and saves the model.
2. Three objectives, all in the trunk's own space:
   - `sheet`: the writer's margin, unchanged;
   - `voronoi`: the address hinge's margin set to half the nearest root separation;
   - `nearest`: no admission; the target must beat the whole table's 8 nearest wrong tokens, and decoding is the whole table's nearest token.
3. Readings of saved checkpoints: the loss in its parts, nearest top-1, reply openings against the commonest-opening count, the opening point's reach back through the context, and each layer's selected rate entries.
4. Two switches in `geometric_stack_native.rs`, both off by default so recorded runs reproduce:
   - the rate table read as retention;
   - the rate score folded into `[0, 1]` instead of clamped.
5. The fold-against-clamp run under [its sheet](../../../harness/prep/in-space-trunk-to-text-2026-10-27.md), read by the sheet's cells A–D and flag F.

## Cancel probe (run first)

Cancel if another open order changes the float-train runner or recurrence, or an in-space long-training runner already exists.

Result (2026-10-27T09:40Z, `origin/main` at `f3a6f0f72`):
- `git grep -n "fn run_long\|set_folded_scores\|set_retention_rates" origin/main -- crates/uor-r4-float-train` finds nothing.
- This lab's only other open draft, #3320, is closed, parked under the same direction. Its patch is in the store.
- The other open PRs (`gh pr list --state open`) touch no file in this scope.

**Not cancelled (instrument: none needed).**

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs` (new)
- `crates/uor-r4-float-train/src/bin/train-in-space.rs` (new)
- `crates/uor-r4-float-train/src/lib.rs`
- `crates/uor-r4-float-train/Cargo.toml`
- `Cargo.lock`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs` (the rate-score switches)
- `crates/uor-r4-float-train/src/secant_descent.rs` (a capture helper for rate entries)
- `docs/roadmap/items/in-space-trunk-to-text.md` (new), `docs/roadmap/README.md` (rendered)
- `docs/harness/prep/in-space-trunk-to-text-2026-10-27.md`
- `docs/research/notebook/float-training.md` (rows)
- `docs/loop/tasks/in-space-trunk-to-text/**` (this task folder)

## Done when

1. The runner and both switches build, with the unit tests green:
   - host: `cargo test -p uor-r4-float-train --lib in_space_long`;
   - GPU: the fold's CUDA test, in the run's `fold-test` step.
2. The fold-against-clamp run is read by its sheet's cells, at both rates, from numbers re-read with one binary, and recorded as a notebook row.
3. The REPORT gives, negatives first:
   - the text the models write;
   - next-token top-1 against the 6,300 bigram and 7,002 M1 counts;
   - reply openings against 161;
   - reach and rate entries;
   - training tokens per second.
4. Every run's outputs and checkpoints are in `caseyallard/uor-r4-store:results/claude/train-in-space-2026-10-27/`, and both pods are down.

The item's own done-when may or may not be met by this unit. The REPORT says which.

## Stop and decide (never stop and wait)

- If the fold reading is cell D or flag F, record it, keep the clamp as the default, and name the next unit in the REPORT.
- If a run diverges or a reading is missing, rerun that step; a missing number is never read as a fail.
