# ORDER: one-model-end-to-end, claude-367 (pair experts check the reply: the corpus answers when the model's reply was never seen with the turn)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `cd9b63736781a368fec543c8abf16cf8d3532a4a`.

## Goal

**Give the served model exact pair experts counted from TRAIN that check each everyday reply against the turn's own tokens and answer in its place when the reply was never seen with them, and read what the model then says beside the same model without them.**

**Why now.** claude-366 read every trained model on a fresh panel and found one gap under the everyday replies.

- **The model writes the right frame and the wrong value.** Of 235 everyday rows of the seed-23 panel, the three saved models are good on 113 of the 116 rows whose reply needs no word of the turn, and on 29 to 32 of the 119 rows whose reply does. Asked the capital of Greece, a saved model says "The capital of Greece is Saturday." The learned address tables moved the 119 rows by -1 to +8 a seed (claude-366: not called, both arms).
- **TRAIN holds the answers.** 226 of the 235 turns stand in TRAIN word for word; a 16,384-step run reads TRAIN 9.1 times.
- **A reply that needs a fact depends on two tokens of the turn at once**, the question word and the key word, and no served or trained part forms that pair exactly. The Hamming read's copy heads and the token rows key on one token; the E8 rows key on a cell of the learned state.
- **A similarity read makes the model's own error.** The set-overlap read on `main` (`stack/prompt_reply.rs`), keyed on the last user turn with the turn's own entry held out, answers "Do you know the capital of Greece?" with "Berlin is the capital of Germany.": 11 good of 57 rows of nine fact forms.
- **Pairs of the turn's tokens, voting, do not:** 57 of those 57 on the same hold-out. Alone they are wrong on small talk ("Thanks for the help." answered "Okay, I have all of them."), where the model is right.
- **Overruling the model token by token loses rows** (two simulated forms, net negative at every floor).
- **Checking the model's finished reply gains rows.** The reply stands when some pair of the turn has been seen with that reply; otherwise the pairs' top reply is given. On the first 168 everyday rows, p-s1, hold-out: 100 good to 124; the pairs answered 65 rows, 30 newly good and 6 lost; goodbyes 32 of 32 unchanged; capitals 0 to 5 of 5, the next day 0 to 4 of 4, opposites 3 to 7 of 7. The 6 lost rows are turns whose only specific phrasing was held out, answered by pairs of common tokens; pairs seen at most 2,000 times keep the fact forms (and gain "legs", 0 to 2 of 2).

All of the above is sizing on a panel already read (kit files under `local/claude/claude-366/forms/`); none of it is a bar. The last count was stopped at 168 rows for want of memory on the laptop.

**The mechanism.** Each token has its prime; an expert is an unordered pair of distinct tokens of the last user turn, addressed by the product of the two primes (a semiprime below 2^31), as the prime router of `docs/mechanisms/exact-memory.md` names it. No head chooses which two tokens are the key: every pair of the turn is an expert, and TRAIN's counts decide which ones know. An expert holds the replies it was seen with and how often; a pair seen more than a cap of times is no expert. The model writes its reply as today. Then:

1. A turn the counted table classes as asking for a told value, or as a repeat ask, is left alone.
2. The reply **stands** when the turn has no expert, when the experts' vote for that reply is within a fixed ratio of their vote for their top reply, or when their top vote is under a fixed floor: they do not know the turn (amendment 2).
3. Otherwise the experts' top reply is given, and the conversation holds that reply.

A vote is the sum over the turn's experts of the reply's count over the expert's count, in integer fixed point; ties go to the smaller reply index.

**The steps, in one unit.**

1. **The table** (`pair_experts.rs`, `place-asks experts`): built once from a TRAIN split, written as a file with its own magic and the SHA-256 of the tokens it was counted on.
2. **The check** after the served reply (`uor-chat`, the reply reader): rules 1 to 3; with no table named, every reply is id for id today's.
3. **Hold-out for an honest count:** a reader flag that takes the row's own TRAIN entries out of every expert before it votes, so the reading is on a phrasing the experts did not see.
4. **The numbers** (the cap, the ratio and, by amendment 2, the floor) are set by the Rust tool on the seed-23 panel, which is already read, and committed before the seed-24 panel is written.
5. **Read on the laptop:** the three saved models, off, held out and as served, on the seed-24 panel; the nine-turn conversation and a second conversation of fact questions, word for word; tokens a second; the table's size.

**Claim check.**
- (a) **Step.** The owner direction's "one model end to end" (`PLAN.md`): the gap left on everyday turns is closed from the corpus the model was trained on.
- (b) **What it changes in the model.** The served reply is checked by exact pair experts: a reply never seen with the turn's key is replaced by the one the key's experts hold.

**Rules check** (CHARTER §3). The address is a product of two primes and the store is exact counts: the project's prime router (§3.1; ADR-0003). The vote is a sum of ratios of integer counts and the pick is the largest: no squashing function, no normalised exponential (§3.2). The check is one lookup per pair of the turn and a sum: no dense product (§3.3). Laptop only: builds, host checks and serving the integer artifact (§3.5, §3.6); no pod, no training, no comparator. Whether a table counted from TRAIN may be served in a product is the owner's reading of §3.1, open since claude-365; it is named in the REPORT and nothing here is served as a product.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a served check or read keyed on pairs of the turn's tokens, or a reading of one.
- **Result (at base):** none.
  - `crates/uor-r4-integer/src/stack/prompt_reply.rs` (deepseek-204, deepseek-207) picks one stored reply by set overlap of the whole prompt: the 11 of 57 above.
  - `crates/uor-r4-object-store/src/address_read.rs` holds `PrimeAddress::semiprime` and a sieve that ranks by the count of shared primes: the same overlap measure, no vote, not linked into the serving crate.
  - `crates/uor-r4-training/src/stack_prime_route.rs` keys a relation store by the semiprime of "the last two distinct tokens tagged entity or relation": a learned tag head picks the two atoms, the part that failed (`exact-memory.md`: "1,480 (77.7%) were the query entity tagged wrongly").
  - `git grep -n -i -E 'pair.?expert' origin/main -- crates/uor-r4-integer/src` finds nothing.

## Claim

Written by hand, as claude-359 to claude-366 were: `uor-harness loop claim` refuses while this lab's worktree `.worktrees/claude-366` holds the item (its PR is in the merge path). The two units share no source line.

## Files in scope

- `crates/uor-r4-integer/src/pair_experts.rs` (new: the counts, the file, the vote, the check), `crates/uor-r4-integer/src/native_geometry.rs` and `crates/uor-r4-integer/src/serving_root.rs` (its registration in the crate's two builds)
- `crates/uor-r4-integer/src/stack/chat.rs`, `crates/uor-r4-integer/src/stack/tests.rs` (the check after a served reply and its tests)
- `crates/uor-r4-integer/src/bin/place-asks.rs` (the `experts` mode), `crates/uor-r4-integer/src/bin/uor-chat.rs` (the flag)
- `docs/labs/one-model-end-to-end/instrument/**` (the reply reader's flags)
- `docs/loop/tasks/one-model-end-to-end/claude-367/**`, `docs/research/notebook/float-training.md`

## Done when

1. The table and the check exist, with tests that fail without the rule they pin: a vote is the integer sum the definition gives, on hand counts; the hold-out takes out exactly the row's own counts; a pair over the cap never votes; a turn with no expert keeps the model's reply; a told-value ask and a repeat ask never reach the experts; with no table the replies are id for id today's; the file refuses other tokens, another vocabulary or a damaged body; ties break the same way every run.
2. The REPORT gives, for each saved model off, held out and as served on the seed-24 panel: G by what the reply needs, told values, never-told, replies that never end, and the rows the experts answered (newly good, lost); negatives first.
3. The REPORT gives both conversations word for word, tokens a second with and without the table, and the table file's size.

## Bars

Fixed here, before any reply checked by the experts is read on the seed-24 panel. Panel: claude-356's writer at `seed=24`, written after the cap and the ratio are committed; every model served with the counted table as merged by claude-365, the whole conversation as the prompt, each setting read once.

G is claude-366's count: everyday replies that are the reference or another reply TRAIN gives the same turn, and end. D is G held out minus G off, per saved model; M is the mean of D over p-s1, p-s2 and p-s3.

- **Adds:** M at +15 or more, and on each model: told values not down by more than 3, never-told not down by more than 2, and good replies on the rows whose reply needs no word of the turn not down by more than 3.
- **Does not add:** M under +5.
- **Not called:** anything between, or M at +15 with a guard missed.

The as-served reading (nothing held out) has no bar: 96% of everyday turns stand in TRAIN word for word, so it measures memory of the corpus, and is reported as that.

Named out of scope, counted and reported, no bar: work on the turn's numbers (adding, counting, the bigger of two) and lists of a class, which no stored reply can answer for operands TRAIN does not hold.

## Stop rules

- The cap, the ratio, the floor and the bars are not moved after a seed-24 reply is read. The seed-24 panel is read once per model and setting.
- No pod and no training in this unit.
- Nothing is served as a product: the experts are read only from a table file named on the command line.

## Amendments, all made before the seed-24 panel is written

The bars above are unchanged by every amendment.

1. **Registration files** (commit `98432ac9e`): the crate registers a module in `native_geometry.rs` and `serving_root.rs`.
2. **A floor on the experts' top vote.** Rule 2 gains a third way for a reply to stand: the experts' top vote is under a floor, so they do not know the turn. A vote of 1 is one expert that saw only that reply.
   - **Why.** On the seed-23 panel, held out, the best cap and ratio alone took good everyday replies from 145 to 178 (p-s1), and took the rows whose reply needs no word of the turn from 113 good to 105: the guard this ORDER sets at 3. The lost rows are small talk whose own phrasing is held out ("I appreciate it." six times), answered by pairs of common tokens.
   - **What separates them.** Every one of the 10 lost rows had a top vote under 1.9; 40 of the 43 newly good rows had one of 2.0 or more.
3. **The settings**, in `settings.json` beside this file, with the grids they were read from (`grid23-caps.tsv`, `grid23.tsv`):
   - **Cap: off** (the type's largest value). A cap of 5,000 or more changes at most one row a model; a cap of 2,000 or 300 loses rows (p-s1: 163 and 129 good, against 177 with none).
   - **Ratio: 1/32.** From 1/16 to 1/256 the count moves by at most one row a model.
   - **Floor: 1.5.** From 1.25 to 2.0 the count moves by at most one row a model; at 2.5 it falls by five.
   - **Seed-23 reading at these settings, held out** (sizing, no bar): good everyday replies 145 to 185, 144 to 182, 142 to 183 of 235; one good row lost a model; no told-value, never-told or repeat row replaced.
4. **One check for the chat and the panel.** The reply reader is not changed. `place-asks experts-check` replays a panel's finished replies through `PairExperts::check_prompt`, the function the chat calls after a served reply, so the panel count and the chat apply one rule. The reader's own flags (step 2, "the reply reader") are not built.
5. **Files in scope, added:** `crates/uor-r4-integer/src/stack/session.rs` and `crates/uor-r4-integer/src/stack/hamming.rs` (a getter that hands the chat the model's own counted table and turn marks; no behaviour change), and `docs/loop/tasks/one-model-end-to-end/claude-367/chat-facts.json` (the second conversation, fixed here before any chat is served with the experts).
