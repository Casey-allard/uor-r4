# ORDER: one-model-end-to-end, claude-364

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `1dd3cced9de121168894cb4e1b39c00defac1265`.

## Goal

**Asked to repeat a phrase, the model repeats that phrase.** In [claude-363](../claude-363/REPORT.md)'s nine-turn conversation "Say exactly this: trelvon dask pimmerholt zane" is answered "18.": the counted table finds a place in an earlier turn, and the reply speaks it. On the two panels already read, a request to repeat is about one row in ten (86 of 984 at seed 18, 94 of 980 at seed 19), and the three saved models answer 7, 7 and 33 of them exactly at seed 18 and 4, 7 and 49 at seed 19. The phrase is in the turn being answered. This unit points the place register at it.

## What is built

1. Fit: `place-asks repeat` counts, on TRAIN, every pair of neighbouring tokens in a user turn: how many replies are exactly the rest of that turn after the pair's first place in it, against how many turns hold the pair. A pair is kept when at least 16 replies are the rest of the turn and at least 7 of every 8 turns that hold it are answered so. The kept pairs are written into a copy of an existing table as one new list; a table with no such list reads as before and is written byte for byte as before.
2. Find: before the recall find, the first place in the asked turn that follows a kept pair, with at least one token after it, is `Asked::Repeat(start, end)`: the rest of that turn.
3. Reader: the place register speaks `tokens[start..end]` one by one and then the end mark, the way it speaks the stored "not told" reply, for as long as the reply follows them.
4. Tests for each, with the negative gates below. No training, no pod, no artifact change.

## The bar, written before the new panel exists

A fresh panel: the reading panel's writer at `seed=20`, written after this order is pushed, read once per arm, on claude-358's p-s2, p-s1 and p-s3. Arms: *again* (the table as merged, served again: claude-363's reader) and *repeat* (the same with the kept pairs in the table). A repeat row is a row whose reference without its end mark has at least three tokens and is the end of the last user turn's content. X is the repeat rows whose reply equals the reference. O is the exact replies on every other row. V is the recall questions whose reply holds the told value, by the pinned scorer.

- **Adds:** on all three models X(repeat) − X(again) is at least 30, O is not down by more than 2, and V is not down by more than 3.
- **Costs:** O is down by more than 2, or V by more than 3, on any model.
- **Does not add at this configuration:** neither of the above, and the X gain is under 10 on any model.
- **Otherwise not called.**

Prediction: the fit keeps four pairs (the last two tokens of " these words:", "exactly this:", "exactly:" and " repeat:"); the find fires on 88% to 100% of the repeat rows and is right on 95% or more of the rows it fires on; X(repeat) is 85% to 97% of the repeat rows on every model, so the gain is +70 to +90 on p-s2 and p-s1 and +30 to +50 on p-s3; O changes by 0 to 3 rows; V is unchanged.

Reported with no bar: the pairs the fit counts, with their counts; rows the find fires on that are not repeat rows, word for word; repeat rows it does not fire on; the seed 18 and 19 panels, already read; the nine-turn conversation word for word (turn 4 is expected to be the phrase).

## What a pass is

| # | Gate | Check |
|---|---|---|
| G1 | A table with no pair list: replies byte for byte today's, and the table file's bytes unchanged by a read and write. | tests; the seed 19 replies re-read with the merged table equal claude-363's files |
| G2 | The find answers `Repeat` only after a kept pair in the asked turn, with a token after it, and before any recall find. | tests in the integer crate, each failing with its line set aside |
| G3 | The register speaks the rest of the turn and then the end mark, and stops when the reply leaves it. | tests, each failing with its line set aside |
| G4 | The fit keeps a pair by the two counts above and no other. | tests |
| G5 | The seed 20 panel is written after this order and read once per arm; the row is called by the bar. | the kit's logs with times |
| G6 | The REPORT gives replies word for word and what did not work first; one notebook row. | the REPORT, the notebook |

## Not established by this unit

- Whether a count-fitted list of lead-in pairs may be served in the product (§3.1, the owner's call), as with the counted table.
- The table's false finds and misses on recall questions; anything trained ([claude-361](../claude-361/ORDER.md) is being read); general replies; a request to repeat worded in a way TRAIN does not hold.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a find that already reads the turn being answered, or models that already repeat a phrase.
- **Result (at claim):** none. `AsksIndex::find` takes its candidates from the finished user turns only, and the saved models answer 4 to 49 of 94 repeat rows exactly.

## Claim

Written by hand: claude-361 (PR #3951, a draft whose runs are being read) holds the item for `uor-harness loop claim`.

## Files in scope

- `crates/uor-r4-integer/src/place_asks.rs`, `crates/uor-r4-integer/src/fbs_snapshot.rs`, `crates/uor-r4-integer/src/bin/place-asks.rs`, the crate's tests
- `docs/loop/tasks/one-model-end-to-end/claude-364/**`, `docs/research/notebook/**` (one row)

## Inputs

- The three saved models: `local/claude/claude-358/`. The table: claude-359's fit (`local/claude/claude-362/asks-off.json`). TRAIN: the tokens that table was fitted on.
- The reader binaries are rebuilt at this unit's head; the scorer is `local/claude/claude-361/bin-pin/whole-word-recall`.

## Addendum, 13 Nov: a longer key where the pair fails its share

Written after the two reads of the seed-20 panel and before the seed-21 panel exists. The bar above was read as written and its row stands; nothing in it is changed here.

**What the first read showed.** The count kept three pairs, not the four predicted. The pair that ends "exactly this:" is followed by the rest of its turn in 1,576 of 2,000 TRAIN turns, under 7 in 8: "Please repeat this:" shares the pair, and its reply starts with a capital. The 18 repeat rows of the seed-20 panel with that lead-in are not found, and the conversation's turn 4, worded that way, is unchanged.

**What is added.** The count also takes every three neighbouring tokens of a user turn, counted as the pairs are: once in each turn at the first place with a token after it, `turns` such turns, and `replies` of them answered with the rest of the turn. A triple is kept by the same two counts (16 replies or more, 7 in 8 or more) when the pair it ends on is not kept. The table takes the kept triples as a second list, `repeat3`, absent from a table that holds none. The find takes the first place in the turn where a kept pair or a kept triple ends.

**Bar.** Arms *repeat* (the table with the three pairs, as read above) and *longer* (the same with the kept triples), on p-s2, p-s1 and p-s3, on a seed-21 panel written after this addendum, each read once. X, O and V as above.

- **Adds:** on all three models X(longer) is 90% of the repeat rows or more and at least 5 over X(repeat), O is not down by more than 2, and V is not down by more than 3.
- **Costs:** O down by more than 2 or V down by more than 3 on any model.
- **Does not add at this configuration:** neither, and X(longer) − X(repeat) under 3 on any model.
- Otherwise not called.

**Prediction.** The count keeps one triple, the one that ends "exactly this:", with every one of its turns answered by the rest; the triple that ends "repeat this:" is not kept. On the panel it fires on 15 to 25 more rows, each exact; X(longer) is 93% to 99% of the repeat rows; the gain is 15 to 22 on p-s2 and p-s1 and 6 to 14 on p-s3; O moves by 0 to 2; V is unchanged. The conversation's turn 4 is the phrase and the end mark.

**Gates.** G7: a table with no triple list reads as before: the seed-20 *repeat* arm read again on the new reader equals the files above byte for byte, and a table's bytes are unchanged by read and write. G8: the find and the count, with a set-aside for each new condition. G9: the seed-21 panel is written after this addendum's commit and read once per arm.
