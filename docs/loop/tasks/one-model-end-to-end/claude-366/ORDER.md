# ORDER: one-model-end-to-end, claude-366 (the address table's models, opened on the laptop)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `590a5bd103995f62cf6de2e8fd49f6ae2fb0a102`.

## Goal

**Serve the models trained with the address table as integer artifacts on the laptop, and read what they say on everyday turns beside the saved models.**

**Why now.** The saved models' own replies are the gap that is left, and a trained part that narrows it already exists and cannot be opened.

- **What the saved models get wrong.** On the seed-22 panel, with the counted table as merged, a told value is stated on 556 of 582 questions and a never-told question is answered "not told" on 72 of 73 (claude-365). The everyday turns are the model's own. Of 236, p-s2 gives the reference or another reply TRAIN gives the same turn on 137; on 74 it gives a reply TRAIN never gives that turn, and on 23 it never ends. 229 of the 236 turns stand in TRAIN word for word. By reading: a wrong known fact ("Which number is larger: 18 or 15?" answered "15 is bigger."; "What colour is a tomato?" answered "It's green."), the wrong word in the right sentence ("Guess what, I bought a guitar today." answered "A new bike! That sounds wonderful."), and loops.
- **More steps do not help.** p-s2's held-out next-token hits are 12,575 of 14,287 at step 2,048 and 12,683 at step 16,384.
- **A learned table in the trunk does.** claude-346 trained the saved models' recipe with the address table at 16,384 steps. Exact everyday replies in float, of 520: 119 without the table; 141 with the table addressed by the input token (seed 2); 142 and 177 addressed by the state's E8 cell (seeds 2 and 3). Replies that never end, of 1,108: 99, then 56 (token) and 190, 200 (E8).
- **Those models cannot be opened.** The exporter refuses a checkpoint that holds the rows and the integer reader has no section for them (claude-346's open item 4). Everything served since then is the model without the table.

**The steps, in one unit.**

1. **Build the served form.** The integer reader's section for the rows, the cell of a state block and the token address in integers, the read after each layer's Hamilton block, and the exporter. An artifact of today reads byte for byte as today.
2. **Train the missing seeds** under a prep sheet with a non-author FIT: the token arm at seeds 1 and 3 and the E8 arm at seed 1, at claude-346's pin and recipe, so each arm has three seeds beside the three saved models.
3. **Snapshot the address checkpoints on a pod** at this unit's pin, with each model's float rows, and seal them.
4. **Export and read on the laptop:** the row check against the float rows first; then every model served as the saved models are (the counted table as merged), on a panel written after this order, by kind, with the nine-turn conversation word for word.

**Claim check.**
- (a) **Step.** The owner direction's "one model end to end" (`PLAN.md`): the model that is opened on the laptop is the best model that was trained.
- (b) **What it changes in the model.** The served model gains the trained rows: a learned table read by the state's own E8 cell, or by the input token, after each layer.

**Rules check** (CHARTER §3). The E8 cell of a state block is the project's geometry (§3.1); the read is a table row added to the residual, in integers on the served grid (§3.6). No squashing function, no dense product (§3.2, §3.3). The token arm is a learned table addressed by a fixed hash of the input token; it was the E8 arm's control, and whether it may be served in a product is the owner's reading of §3.1, named in the REPORT. Training and snapshots run on a pod inside the caps (§3.5). No comparator.

## Cancel probe (run first)

- **What would make this unit unnecessary:** an exporter or an integer reader that takes the address rows, or a served reading of a model that holds them.
- **Result (at base):** none. `git grep -i address_memory origin/main -- crates/uor-r4-integer/src` finds nothing; claude-346's job script keeps an address run's checkpoint and writes no snapshot because the export refuses it; the REPORT lists the served form as open. The task stands.

## Claim

Written by hand, as claude-359 to claude-365 were: `uor-harness loop claim` refuses while this lab's worktree `.worktrees/claude-365` holds the item (its PR is in the merge path). The two units share no source file.

## Files in scope

- `crates/uor-r4-integer/src/stack/**`, `crates/uor-r4-integer/src/fbs_snapshot.rs`, `crates/uor-r4-integer/src/bin/**` (the section, the read, the exporter, the readers' checks)
- `crates/uor-r4-float-train/src/e8_address.rs`, `crates/uor-r4-float-train/src/fbs_export.rs`, `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`, `crates/uor-r4-float-train/src/in_space_long.rs` (the rows in the snapshot, the float rows for the check)
- `docs/labs/one-model-end-to-end/instrument/**` (the reply reader, if a table literal changes)
- `docs/harness/prep/**` (the sheet), `docs/loop/tasks/one-model-end-to-end/claude-366/**`, `docs/research/notebook/float-training.md`

## Done when

1. An address model is exported and served on the laptop, with tests that fail without the rule they pin: the integer cell of a block equals the float trainer's on shared vectors, walls and ties included; the read adds the row the address names and no other; an artifact of today reads byte for byte as today.
2. The row check is in the REPORT for every address artifact: the share of scored rows on which it picks its trained model's token, beside the saved models' share.
3. The REPORT gives each model's replies on the fresh panel by kind beside the same-seed saved model, the nine-turn conversation word for word, and tokens a second with and without the rows, negatives first.

## Bars

Fixed here, before any served reply of an address model is read. Panel: the same writer at `seed=23`, written after this order's commit; every model served with the counted table as merged by claude-365, the whole conversation as the prompt.

- **G:** everyday replies (not a repeat row, not a recall or never-told question) that are the reference or a reply TRAIN gives the same user turn word for word, and end.
- **D** = G of an address model minus G of the saved p model of the same seed; **M** = the mean of D over an arm's seeds.
- **Adds (an arm):** M at or above +10, told values stated not down by more than 3 a seed, never-told answered "not told" not down by more than 2 a seed.
- **Does not add at this configuration:** M under +3.
- **Not called:** between.
- **Row 0:** an address artifact whose row check is more than 0.010 under the lowest saved model's (0.9860) is an export or reader fault, not a reading.
- If both arms add, the line's models become the arm with the higher M; within 3 of each other, the arm with fewer replies that never end.

## Stop rules

- No bar is moved after a served reply is read. The fresh panel is read once per model.
- A pod goes up only with the sheet's FIT, inside the caps, with its watcher armed first; `uor-pod status` is read first.
- Nothing is served as a product: the rows are read only from an artifact that holds them.
