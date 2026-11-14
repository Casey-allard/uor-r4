# ORDER: one-model-end-to-end, claude-368 (an expert speaks with the weight of what it knows, and a sure expert refuses a reply it never saw)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `426b2f424df5941dd726f32fe21c3a693e2835c2`, the head of claude-367's PR 3958, which is in the merge queue; `origin/main` is `7068363b4cbb789b27519cea4148fd95723d6eb9`. This branch takes `main` by a plain merge once that PR is merged.

## Goal

**Make the key word of a turn outvote its frame: weigh each pair expert by how surely it knows its answer, let a sure expert refuse a reply it never saw, and read what the model then says beside claude-367's setting on a fresh panel.**

**Why now.** claude-367's experts add 35 good everyday replies of 239, held out. Its report names what is left of the lookup gap, on p-s1 held out:

- **The frame outvotes the key word.** Every pair of the turn has one vote and a question has more pairs of frame words than of the key word. Four good rows are lost on every model ("Can birds fly?" turns to "No."). "How many legs does a spider have?" stays "It has four legs." though TRAIN holds the turn 221 times with eight.
- **A wrong reply stands** when frame pairs alone give it 1/32 of the top vote: 19 rows ("It's Madrid." for the capital of China).
- **25 rows are left to the model** because the top vote is under 1.5 whole votes, where one pair that knows the answer gives 1.

**The mechanism.** After the hold-out, an expert has a total `t`, a count `c` for each reply, and `m`, its largest count.

1. **Weighed vote.** The expert gives a reply its share `c/t` times its own sureness `m/t`: `(c * m << 20) / (t * t)`, in integers. An expert that saw one reply gives it one whole vote; one that saw its turns answered a hundred ways gives each about a ten-thousandth.
2. **Refusal.** An expert is sure when `t` is at least `n` and `8 * m >= 7 * t`. A reply that a sure expert of the turn gives no count does not stand, whatever its vote, and the experts' answer is their top reply among those every sure expert holds. When the sure experts share no reply with a vote, nothing is refused.
3. The rest is claude-367's: the reply stands at `keep` of the top vote; under the floor the experts do not answer; otherwise their top reply is given when it fits.

**The steps, in one unit.** Build 1 and 2 behind a setting whose default is claude-367's vote, bit for bit. Read the grid below on the seed-23 and seed-24 panels, held out. Pick by the rule below, commit the setting and one new fact conversation, then write the seed-25 panel and read it once per model: experts off, claude-367's setting, the picked setting; held out and as served. Read the conversations.

**Grid.** Weigh: even, weighed. Keep: 1/32, 1/8, 1/4, 1/2. Floor: 500, 750, 1000, 1500 thousandths. Refusal: off, `n` of 2, 4, 8. Cap off.

**The pick,** fixed here: among the settings that, on every model on both panels, replace at most 3 told-value rows and 2 never-told rows and lose at most 4 good everyday rows (claude-367's setting loses 1 and 4), the largest sum of good everyday replies over the six held-out readings; ties go to fewer good rows lost, then even, then refusal off or else the larger `n`, then the larger floor, then the smaller keep.

**Rules check** (CHARTER §3). Same table, same prime addresses, exact counts and integer shares (§3.1, ADR-0003). No learned weight, no squashing, no dense product, no float in serving. No pod, no training.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a pair-expert vote already weighed by the expert's own counts, or a reading of one.
- **Result (at base):** none.
  - `crates/uor-r4-integer/src/pair_experts.rs` (claude-367) gives every pair one vote, `(c << 20) / t`, whatever its counts say, and has no refusal: the three items above are its own report's.
  - `git grep -n -i -E 'purity|sureness|weigh(ed)? (vote|expert)|sure expert' 426b2f424df5941dd726f32fe21c3a693e2835c2 -- crates/uor-r4-integer/src crates/uor-r4-object-store/src crates/uor-r4-training/src` finds nothing.

## Claim

Written by hand, as claude-359 to claude-367 were: `uor-harness loop claim` refuses while this lab's worktree `.worktrees/claude-367` holds the item (its PR is in the merge queue). This unit changes claude-367's own files, so its branch starts at that PR's head.

The weighed vote, the refusal and the two tools' flags were built and tested on the laptop while claude-367's PR waited in the merge queue, before this order's commit. No grid was read before this order's commit: the bars, the grid and the pick rule above were fixed first. One reading was made before it: the replay of claude-367's seed-24 files at claude-367's own setting, which checks the build and names no new setting.

## Files in scope

- `crates/uor-r4-integer/src/pair_experts.rs`, `crates/uor-r4-integer/src/stack/chat.rs`, `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/bin/place-asks.rs`, `crates/uor-r4-integer/src/bin/uor-chat.rs`
- `docs/loop/tasks/one-model-end-to-end/claude-368/**`, `docs/research/notebook/float-training.md`

## Done when

1. The weighed vote and the refusal exist with tests that fail without the rule they pin: the weighed vote on hand counts; a sure expert refuses a reply it never saw; an expert under `n` or under 7/8 refuses nothing; with the default setting every vote and verdict is claude-367's (its seed-24 row files replay byte for byte).
2. The REPORT gives, for each saved model on the seed-25 panel, off, claude-367's setting and the picked one, held out and as served: good everyday replies by what the reply needs and by form, told values, never-told, rows newly good and lost; negatives first.
3. The REPORT gives the conversations word for word.

## Bars

Fixed here, before the grid is read. G is claude-366's count. D is G at the picked setting less G at claude-367's setting, held out, per model, on the seed-25 panel; M is the mean over p-s1, p-s2, p-s3.

- **Adds:** M at +8 or more, and on each model against experts off: told values not down by more than 3, never-told not down by more than 2, good replies on rows that need no word of the turn not down by more than 3.
- **Does not add:** M under +3, or the pick is claude-367's own setting (then no seed-25 panel is written).
- **Not called:** anything between, or M at +8 with a guard missed.

Out of scope, counted and reported: adding, counting and lists for operands TRAIN does not hold.

## Stop rules

- The setting and the bars are not moved after a seed-25 reply is read. The seed-25 panel is read once per model and setting.
- No pod and no training. Nothing is served as a product.
