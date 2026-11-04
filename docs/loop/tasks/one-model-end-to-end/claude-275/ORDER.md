# ORDER: one-model-end-to-end, claude-275 (the long panel, the long TRAIN and P2b's sheet)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `2326bece3`. Draft PR #3654.

## Goal

P2b trains the one model at the 2,048-token window on a long TRAIN and scores recall across turns, against a no-memory control (L-010). claude-274 made scoring and decoding follow the window. Three things are still missing before the run's sheet can be written:
- **No long held-out panel exists.** `heldout-panel` draws only `conversation_in` (`docs/labs/one-model-end-to-end/instrument/src/main.rs:171`), and conversations stop at 256 tokens.
- **No scorer reads the queries.** `sample_windows` decodes each window's first reply opening, and claude-274's `last` takes the last one, cutting a reply that runs past the window (7 of 16 at 192). Nothing grades whether a reply states the queried note's value.
- **No long TRAIN exists,** and `--census` reads only the pinned TRAIN (`census`, `in_space_long.rs:4777`).

This unit builds those three, writes the long panel and the long TRAIN on the laptop (CPU data generation is host work), and writes P2b's sheet with its FIT and pre-run review. The run itself is the next unit.

- **Plan step:** Casey's 4 November plan, P2b, sheet unit 2. It is the unit's last, under the new one-unit bound noted on #2028 (6626180519).
- **What it changes in the model:** the data the model trains on (conversations up to 2,048 tokens, with notes asked for up to about 2,000 tokens later), and what it is scored on (each recall query, graded by whether the reply states the note's value).
- **No GPU** in this unit. The sheet is the pre-registration for the run (§4.5).

## Cancel probe (run first)

1. Does any panel writer already draw chained conversations in a held-out cell?
2. Does the corpus writer's chained draw take an exclusion list, and in which cell?
3. Does any scorer select a reply opening by query, or grade recall of a note's value?
4. Can the census count against a named TRAIN?
5. Does any open order hold these files? The claim's overlap check passed.

**Result, at `2326bece3`: not cancelled.** The finding is about the instrument, not the mechanism. The generator and the window exist; the panel, the selector and the census do not.
1. No. `heldout-panel`'s draw loop calls only `world.conversation_in(&mut rng, cell)` (`main.rs:171`).
2. Yes, in TRAIN only. `m-world-recall-corpus` takes `exclude=PANEL.json`, an array of rows with `user_turns` (`:226`–`:248`). Its chained draw calls `world.chained(.., Cell::same(Split::Train), budget, &excluded, ..)` (`:417`), and an excluded note or query drops its whole draw (`:473`).
3. No. `sample_windows` (`in_space_long.rs:4951`) takes each window's first opening and `last_openings` (`:4980`) its last. The rows files record picks, not query positions. `Tag.chain` (`milestone_world_v2.rs:1228`) records each query's note, episode and distance, and `Tag.answer` its value, but no scorer reads them.
4. No. `census` loads `load_train_split(data_root)`, the pinned `mw-balp/train`. Training's `--train-split` loader, `load_named_split` (`:9788`), exists but the census does not take it.
5. The claim passed with no overlap.

## Files in scope

- `docs/labs/one-model-end-to-end/instrument/**`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-training/src/milestone_world_v2.rs` (only if a query's reply position must be exposed)
- `docs/harness/prep/one-model-end-to-end-long-context-2026-11-04.md`
- `docs/loop/tasks/one-model-end-to-end/claude-275/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. **The chained panel writer.** `heldout-panel chain=N` draws chained conversations in its `cell=` through `MWorld2::chained`, under the spaced meter, as TRAIN's chained draw does. It writes `tokens.u16`, `response_mask.u8` and `manifest.json`, and never replaces an existing folder. It also writes:
   - `queries.json`: for every chained query, its conversation, the corpus position of its reply's first token, its reference reply's length, its answer, and its `Chain` record;
   - `user_turns.json`: rows of `user_turns`, the form `m-world-recall-corpus exclude=` reads.

   A laptop test writes a small chained panel. At every query, the mask opens a reply there, the reply's text holds the answer, and positions increase.
2. **The long panel**, written on the laptop at `cell=dev,dev`: phrasings and values both held out. Its sha256s go in the sheet, and it goes in the store with a read-back.
3. **The long TRAIN**, written on the laptop by `m-world-recall-corpus chain=2048 context=2048 protocol=2 exclude=<the panel's user_turns.json>`. Its sha256s and redraw counts go in the sheet, and it goes in the store with a read-back.
4. **The query selector and recall grade.** `train-in-space --score-rows --panel-queries NAME=FILE` decodes every query of panel NAME whose reference reply ends inside the window, from a prompt that ends at the query's opening, with the memory on and off.
   - It writes `samples-NAME-queries-{memory,no-memory}.json`.
   - It counts recall: the decoded reply states the answer. Counts are binned by distance, and queries past the window are counted, not decoded.
   - The selector and the grade are pure functions, tested on the laptop, each with a negative gate.
5. **The census over a named TRAIN.** `--census 1 --train-split DIR --train-sha256 HEX --train-mask-sha256 HEX` counts against that TRAIN.
   - For a queries panel, it also counts the share of answers that appear in any TRAIN reply.
   - Run on the laptop: the long panel against the long TRAIN.
6. **The row stride.** `--score-rows --context N` with 192 < N ≤ 1,536 is refused by name. Several windows share a forward there, and `row_picks`' stride call site has no test. A test pins the refusal, with a negative gate.
7. **P2b's sheet.** `docs/harness/prep/one-model-end-to-end-long-context-2026-11-04.md`, with the shape rule's headings (Bar, Base artifact, Data split, Steps, Minimum test power), and a fresh non-author FIT.
   - **Amended 4 Nov, before any work:** the pre-run review moves to the run's unit and happens right before launch, as §5.10 orders. The run's pin is this unit's merge commit, which does not exist until this unit merges. A job file reviewed here would have to change its pin line afterwards, which voids the review.

## Stop and ask if

Stop and decide; never stop and wait.
- **The chained draw fails under exclusion** (16 dropped draws in a row): shrink the panel or the exclusion, and record it.
- **The census finds panel answers in TRAIN replies:** class it instrument, and fix the cell or the exclusion before the sheet.
- **A done-when needs a GPU** to be shown: keep it host-side if a pure function can carry it. Otherwise move it to the run's unit and say so.
