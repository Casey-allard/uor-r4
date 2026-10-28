# ORDER: in-space-trunk-to-text, claude-152

Lab: claude. Item: [in-space-trunk-to-text](../../../../roadmap/items/in-space-trunk-to-text.md). Base: `5f8d981c7`.

## Goal

Casey, 27 Oct 2026, in claude's chat, after claude-151 (the type spin) read cell D:

> "ok so back to the main track... what about the learning path we were working on? i dont care about the loop right now, I care about making our damn program work"

and, answering which next step to take:

> "Ok so I want you to do all three. I want you to figure out what matters with a probe, if this isn't quick and short then do not do it. Then I want you to both widen the trunk with more training text and retrain the trunk with the hamming read under a geometric margin objective so that we can have a better test for if the response is taking the user's input into consideration"

This unit does those three things, in that order:

1. **The probe.** The quick probe already exists, so this unit runs none. claude-145's premise count (`train-in-space --opening-bound 1`, half a second on the laptop) looks up TRAIN's commonest reply opening by the prompt words a window shows. It names:
   - `161` of DEV's 1,226 openings from the last 1–4 tokens before the opening;
   - `227` from the prompt's first 4 tokens;
   - `460` from the first 6;
   - `607` from the first 8.

   What decides a reply's opening sits at the prompt's start, beyond the trunk's reach of about 6 tokens. The best model so far reads `302`. That gap, from about 300 to 607 from the words alone, is what the runs below measure against.
2. **A better test of whether a reply uses the user's input,** scored at every checkpoint on the pinned DEV:
   - **openings,** read against the `607` a plain lookup of the first 8 prompt tokens gets, not only against `161`;
   - **copied rows:** top-1 on the reply rows whose previous token and target stand side by side in the same window's prompt, the tokens before the reply. A code word or a number the user gave is such a pair; a single shared token is not enough, since this vocabulary splits words into common pieces. Only a model that reads the prompt can name them above the no-read floor. Top-1 on the other reply rows is reported beside it.
3. **The retrains,** each with the Hamming read (`--hamming-read 4`):
   - **the cell margin objective** (`--objective cell`): the predicted point must lie in the target token's cell, a margin nearer the target than the whole table's single nearest wrong token, as `relu(1 + max_r s(r) − s(target))`. The current `nearest` objective averages that hinge over the 8 nearest wrong tokens;
   - **more training text** (`--train-split DIR --train-sha256 HEX`): `balp`, the 99,322,259-token corpus already sealed in the store, in `data/ft-balp.tar` (sha256 `b1d2211377a2e99d5977cc95d46875027023b08867488b1372185b3f62a1473f`, the archive TRAIN's own pin names).
     - It is `mix-chat-corpus` of TRAIN (`mw-balp`) twice, `mw-orcp` and `chat-v0-p2`, so about 82M of its tokens are general chat.
     - Same tokenizer (`d36d3e87…`) and the same dialogue protocol 2.
     - Its tokens sha256 is `13261ba81180d8c59086787d80e34cd8ff2a144d6a818018725777d5a8d6bf62`;
   - **a wider trunk** (`--width 512`): width 256 stays the default, and every existing checkpoint and pin is unchanged.

The item's own done-when (unchanged by this unit):

> On the pinned DEV panel, decoded as the whole table's nearest token:
>
> 1. Reply-opening top-1 is at least `197` of 1,226. That is always naming DEV's commonest opening token (`161`) plus three binomial standard deviations (`36`). The same checkpoint's whole-panel top-1 must be at or above the M1 count table's `7,002`.
> 2. On that same checkpoint, the relative reach from `--reach`, `R16 = shift(16) / shift(0)`, is at least `0.05`. The 27 Oct best checkpoint reads `0.011`. An untrained trunk reads more, but it cannot meet (1).
> 3. Greedy replies to the six fixed DEV prompts are recorded beside the true replies, at every checkpoint, in the lab record.
>
> Lead with tokens per second, decode and context separately.

## Cancel probe (run first)

Run 2026-10-28T01:50Z against `origin/main` `5f8d981c7c4da532b671110e703085478913b1e0`. **Not cancelled (instrument: a search of main and the open PRs).**

- `git grep` of `crates/uor-r4-float-train/src` finds none of `PointObjective::Cell`, `copied_hits`, `extra_train`, `--extra-train` or `--width`. Main has no cell objective, no copied-row test, no second TRAIN split and no width flag.
- The only open PR on this item is this unit's own draft, #3340.

## Files in scope

The item's scope:

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/secant_descent.rs`
- `crates/uor-r4-float-train/src/lib.rs`
- `crates/uor-r4-float-train/Cargo.toml`
- `crates/uor-r4-float-train/src/flock_read*.rs`
- `Cargo.lock`
- `docs/harness/prep/in-space-trunk-to-text*`
- `docs/labs/in-space-trunk-to-text/**`
- `docs/loop/tasks/in-space-trunk-to-text/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/in-space-trunk-to-text.md`

And, for the second TRAIN split and the width, both on Casey's direction above (the item's scope is widened to match):

- `crates/uor-r4-float-train/src/hamilton_sheet.rs`: the loader of a second, unpinned split, and the model build at a given width;
- `crates/uor-r4-float-train/src/spiralcore_fbs_sheet.rs`: `build_fbs_model` at a given width;
- `crates/uor-r4-float-train/src/native_vector_writer.rs` and `crates/uor-r4-float-train/src/spiralcore_fbs*.rs`: only where a shape assumes width 256.

The pinned TRAIN, DEV and tokenizer, and every existing pin, stay as they are. A second split is checked by the sha256 its run records, not by a new pin.

## Done when

1. **`--objective cell`,** with a host or CUDA test that the loss is zero exactly when the target is a margin nearer than the nearest wrong token, and positive otherwise. Its negative gate (the max over rivals replaced by their mean, which is `nearest`) must make the test fail.
2. **The copied-row test** in `NearestReading`, with `copied_rows` and `copied_hits` for reply rows whose previous token and target stand side by side in the window's prompt, and `rest_rows` and `rest_hits` for reply rows that neither open nor copy. A host test pins the split on a hand-built window. Its negative gate (the prompt check widened to the whole window) must make the test fail.
3. **`--train-split DIR --train-sha256 HEX`:** TRAIN read from another `UORT` split, refused unless its tokens hash to `HEX`. Its sha256 and token count go in the run's config. The pinned TRAIN stays the default.
4. **`--width N`** (default 256): the trunk, the table and every read built at `N`. Checkpoints record their width, and a width-256 build is byte-for-byte what main builds. The suite stays green, including the CUDA tests on a pod.
5. **The pre-registered runs** (one sheet, a fresh FIT and pre-run review, CHARTER §4.5):
   - (a) the cell objective at width 256 on the pinned TRAIN;
   - (b) the same with the extra text;
   - (c) the same as (b) at width 512.

   Two seeds each, scored by the test in 2. Twins: claude-146's `df1d7892` Hamming-read runs, with the `nearest` objective on the pinned TRAIN at width 256, sealed in the store.
6. **The REPORT** leads with tokens/s (training, and decode against context). One notebook row, and the result on main.

## Stop and decide (never stop and wait)

- **Width 512 needs CUDA kernel changes beyond a shape:** if they fit in this unit, make them, re-pin any kernel sha256 they move, and run the full CUDA suite. Otherwise run (a) and (b) here, and give (c) its own unit.
- **A probe projects a run past about an hour per arm** (run-claude PART 4): fix the code first, or record why the run needs longer, in the sheet.
- **A run diverges or crashes:** stop it and classify it, instrument or object, then rerun that seed. A missing number is never read as a fail.
