# ORDER: one-model-end-to-end, claude-374 (pair rows at full weight, rare pairs raised by 1/R)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `c615085ff22ea7009605fdeb3855a7f8b7837cc3`. Follows claude-373 (PR #3964) and claude-370 (PR #3961).

## Goal

**Weigh each pair row of claude-370's table by 1 + 1/R, where R is the number of different TRAIN replies its pair is seen with. Train it the same way, and read whether the fact replies come out right while the copying and the reply's end stay claude-370's.**

**Why now.** claude-373 weighed the rows by 1/R and did not add. Its fact rows fell from 85 to 75 and from 85 to 71 of 188, and its exact replies fell by 141 and 68 of 2,218. The `rare` model copied the turn's words and then did not stop ("peebrarfais choutmout klerkteel Assistant: peebrarfais …"). Its REPORT reads this as: "Frame pairs carry where the reply ends." 1/R scaled them to 1/4096, and under the rows' adaptive optimizer that freezes them. The REPORT names this unit's step under "Still open": "keeps them at full weight for the end decision, and weighs only the content rows by 1/R."

**The mechanism.**

- **The weight.** The R of claude-373, counted in the same pass and capped at 4,096. A row's weight is 1 + 1/R. A pair seen with one reply weighs 2. A pair seen with thousands weighs just over 1, so it is claude-370's row. The weight is a fixed number per row, counted once at start-up and saved with the rows. It is not learned.
- **The read.** The bag of a reply is the sum over its turn's pairs of (1 + 1/R) × row. Everything else is claude-370's: the address (the product of the two tokens' primes), where the bag is added, the zero start, the training by the output error and the rows' own optimizer. Equivalently, the plain bag of claude-370 plus claude-373's 1/R bag, from the same rows at the same seam.
- **The flag.** `--pair-weight boost` with `--pair-rows MIN`. `--pair-weight replies` stays claude-373's, and without the flag everything is claude-370's, bit for bit. The saved rows say which weight they carry, and scoring prints it.
- **One repair, from claude-373's review.** A boosted rows file carries a `boost` mark beside its weights, and a mark without weights is refused. Scoring prints the kind it loaded, so I-8 can compare it with training. A `replies` file stays claude-373's byte for byte, so a lost weights tensor there is still caught only by I-8.

**Rules check** (CHARTER §3). Geometry only: the address stays the prime route. The weight is a sum of 1 and an exact count's reciprocal, a fixed scale on each row. There is no learned gate and no squashing function, and division and addition are allowed (D81). The read is a gather, a scale and a sum: no dense matrix product. Training runs on a GPU. Nothing is served. The control is claude-370's arm `pair` at this unit's pin, not a neural comparator.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a pair-addressed learned table weighed with a floor at claude-370's weight, already built or already read.
- **Result (at base):** none. `crates/uor-r4-float-train/src/pair_rows.rs` holds unweighted rows (claude-370) and 1/R rows (claude-373, read: does not add). Nothing weighs above 1.
- **What earlier work warns of:** claude-373's own reading is an inference from the replies, not a measurement. If the end decision does not come from the frame pairs, this unit reads close to `pair` and does not add. That is a real answer too, and the bar below calls it.

## Claim

Written by hand, as claude-359 to claude-373 were. No code of this unit exists at this order's commit.

## Files in scope

- `crates/uor-r4-float-train/src/pair_rows.rs`, `crates/uor-r4-float-train/src/bin/train-in-space.rs`, `crates/uor-r4-float-train/src/in_space_long.rs`
- `docs/loop/tasks/one-model-end-to-end/claude-374/**`, `docs/harness/prep/**` (this unit's sheet), `docs/research/notebook/float-training.md`

## Done when

1. `--pair-weight boost` exists with tests that fail without the rule they pin:
   - a row's weight is 1 + 1/R. A pair seen with one reply weighs 2, and the bag is the weighted sum of its turn's rows and nothing else;
   - without the flag the bag, the saved files and the forward are claude-370's bit for bit, and `--pair-weight replies` is claude-373's bit for bit;
   - the weight and its kind are saved and loaded with the rows. A boosted mark without weights is refused, and scoring prints the kind it loaded (none, replies or boost);
   - a reply decoded with cached steps equals the whole-window reply.
2. The sheet is committed with a non-author FIT before any training run.
3. The REPORT gives, for every model: exact replies on both panels, the fact and number rows (exact, and ended with the answer word), replies by kind and by need, replies that did not end, the trainer's speed, and replies word for word. Negatives come first.

## Bars

Fixed here, before any model of this unit is trained. The sheet may sharpen them before its FIT, never after a result is read. They are claude-373's, so the two units read alike.

- **Runs.** claude-370's recipe: claude-358's `p` recipe, 16,384 steps, `--pair-rows 2`. Arm `pair`: that recipe. Arm `boost`: that recipe with `--pair-weight boost`. Seeds 2 and 3. Same TRAIN, tokenizer and panels, same GPU kind for every model.
- **Panels.** P14 (1,108 scored replies) and P23 (1,110), scored in float by the trainer's own pass.
- **The number.** The fact and number rows: 101 on P14 and 87 on P23, 188 per model, as claude-370's `subst` table counts them. A row counts when the reply ends with the answer word. For each seed, `boost` minus `pair`.
- **Adds:** each seed gains 10 or more of 188, and exact replies over the two panels fall by no more than 1.5 points (33 of 2,218) at either seed.
- **Does not add:** a mean gain under 4.
- **Not called:** anything between.
- Read without a bar: exact facts, repeat words, replies by kind and by need, replies that did not end, the weight census (R by quantile), steps per second.

## Stop rules

- A run that does not finish, or a scoring pass that is an instrument failure (claude-370's I-8), is repaired and run again from a new root. It is never read as a negative.
- No arm, seed or step count is added after a result is read.
