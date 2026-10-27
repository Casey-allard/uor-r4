# ORDER: one-decoder, claude-150

Lab: claude. Item: [one-decoder](../../../../roadmap/items/one-decoder.md). Base: `54be88b72`.

## Goal

The item's done-when reads:

> 1. A pre-registered run (its own prep sheet, FIT and pre-run review, CHARTER §4.5) trains the in-space trunk with `--objective voronoi`, past step 16,384 (for example 32,768 steps), at two seeds. Every checkpoint reads both decoders' top-1 and openings on DEV.
> 2. The sheet's bar decides one of two outcomes, and the REPORT records the decision:
>    - **the writer holds** (no collapse past step 16,384, and within the sheet's margin of whole-table nearest): the writer stays the served decoder;
>    - **it does not:** the decision is to serve whole-table nearest. The REPORT gives its per-token cost at 4,096 tokens (1,048,576 absolute differences) and the point at which a sublinear index is needed. Changing the serving session (`stack/session.rs`, `fbs-toks`) is then its own unit; this item's scope is the run and the decision.
> 3. One notebook row; the result on main.

## Cancel probe (run first)

The item's cancel rule: cancel if a merged unit already serves the decoder the float runs are evaluated with, or a run on main already reads both decoders past step 16,384 with `voronoi`.

Result (2026-10-27T22:45Z, `origin/main` at `54be88b72`):
- The serving session still decodes with the writer: `fbs-toks` feeds back "the token the writer chose" (the map's stage 8), and no merged unit changes it.
- claude-142's only `voronoi` run (`long3/v-voronoi`, in `caseyallard/uor-r4-store:results/claude/train-in-space-2026-10-27/`) stops at 16,384 steps, with lr 5e-4 → 0 and retention on; it reads the writer's top-1 only. The run that collapsed (`long2/g-long`, the writer objective) is the 32,768-step schedule: 7,593 at step 12,288, 1,363 at 18,432.
- No run on main records the writer's openings at all: the progress line has `dev_top1_hits` (the writer's top-1) and `dev_nearest` (whole-table nearest's top-1 and openings), but no writer openings.
- The open PRs are this unit's draft #3336 and antigravity's #3323, which touches none of this scope.

**Not cancelled (instrument: none needed).** One instrument gap, fixed first: the runner reads no writer openings, so this unit adds `dev_writer` (the writer's top-1, openings and distinct opening picks on the same rows and points as `dev_nearest`).

## The run this unit makes

`voronoi` on `long2`'s schedule, so the collapse is tested where it happened: 32,768 steps, batch 16, lr 5e-4 → 0, retention on, decay multiplier 1, evaluated every 2,048 steps, seeds 1 and 2, one arm per pod. The prep sheet fixes the bar before the run.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/native_vector_writer.rs`
- `docs/harness/prep/one-decoder*`
- `docs/loop/tasks/one-decoder/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-decoder.md`

## Done when

The item's done-when, above, with:
1. `dev_writer` in every progress line, with a host-compiled change and the CUDA tests green on the pod's build;
2. the pre-registered run read by its sheet's bar at both seeds, the decision recorded in the REPORT, and both decoders' decode speed timed on the final checkpoint;
3. one notebook row; the result on main.

## Stop and decide (never stop and wait)

- **`dev_writer.hits` differs from `dev_top1_hits` at any checkpoint:** an instrument fault, not a result. Find why before reading any bar.
- **A run diverges or crashes:** stop it, classify it (instrument or object), and rerun that seed; never read a missing number as a fail.
- **The seeds split on the bar:** the sheet says how a split is read; follow it.
