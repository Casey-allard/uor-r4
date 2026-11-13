# ORDER: one-model-end-to-end, claude-365 (a question never told finds no place: a floor for each question form)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `729ae2aeeecb375c8566e27e6a8b47cb0ef1a353`.

## Goal

**Cut the replies that answer a question never told with a told value, without losing the questions that were told.**

**Why now.** [claude-364](../claude-364/REPORT.md) left three wrong turns in the nine-turn conversation. One is "What is two plus three?" answered "It's 18.", a value told for another key. The same failure is counted on its newest panel (seed 21, read by claude-364, so it sizes this unit and carries no bar): the served table sets a place on 20 of the 76 questions about a fact never told, and the three saved models then say "not told" on 57, 56 and 57 of 76. On the other kinds the table is already close: 542 of 564 told questions find their place (16 a wrong place, 5 a false "not told"), and of 2,884 other replies none gets a place and 12 a false "not told".

**The cause, as read in the code.** The find keeps the best candidate when its score reaches one floor, fitted on TRAIN over every question that asks for a fact (`floor`, 656 in the merged table). Question forms differ in how high their true places score, so one floor is a compromise: a form whose true places score far above 656 still accepts a wrong candidate at 700.

**The steps, in one unit.**

1. **Size it by count on the laptop, before any build.** Group TRAIN's replies by the form of the question they answer. For each form seen often enough, fit that form's own floor with the routine the fit already uses for the one floor (keep right picks, drop wrong ones). Count, on TRAIN and on the panels already read (seeds 21, 20 and 19), what the find returns with the per-form floors against the merged table: told questions (right place, wrong place, false "not told", nothing), never-told questions (place set, "not told"), other replies (place set, "not told"). Two form rules are counted: the question's distinct tokens as read today; and the same with tokens that are rare among TRAIN's questions left out, so that questions that differ only in an invented key share a form.
2. **Build it, if step 1 clears the go rule.** The table holds the per-form floors (absent in a table of today, which is then written and read byte for byte as today); the find takes a form's floor when the table holds one and the one floor otherwise.
3. **Read it on a panel written after this order's bar,** with the three saved p models, served again as merged (claude-363), against the merged table on the same panel; and the nine-turn conversation.

**Go rule for step 2, from step 1's count.** With floors fitted on TRAIN only, on each of the three read panels: never-told questions with a place set fall by 6 or more, and told questions that find their right place fall by no more than 3. Under that on any panel, the unit reports the count and changes the design before building; it does not build.

**The bar for step 3, fixed here, before the panel is written.** The panel is the world's writer at `seed=22`, written after this order's commit, read by no count and no model before. N is its never-told rows; S is the rows among them whose reply says "not told"; V and O are claude-364's counts (told value stated; other rows exact). Each is read with the merged table and with the new one, per model.

- **Adds:** S rises by 6 or more on all three models, V falls by no more than 3 and O by no more than 2 on each.
- **Does not add at this configuration:** S rises by fewer than 3 on two or more models, or V falls by more than 3 on any.
- **Not called:** anything else. The per-model table goes in the report.

**Claim check.**
- (a) **Step.** The owner direction's "one model end to end" (`PLAN.md`): a grounded reply says it was not told when it was not.
- (b) **What it changes in the model.** Nothing in the trained artifact. It changes which candidate places the served counted table accepts.

**Rules check** (CHARTER §3). Counts from TRAIN and integer compares: no squashing function, no dense product, no float in the integer crate, no neural grader. The floors are counted, as the table they join is; whether a count-fitted table may be served in a product (§3.1) is the owner's open question and is not decided here.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a floor that already depends on the question, or a count of one.
- **Result (at base):** none. `PlaceAsks` holds `floor`, `floor_other`, `asking_floor` and `margin`, each one number (`crates/uor-r4-integer/src/place_asks.rs`). claude-360's cuts changed what the table reads of a question, not its floor.

## Claim

Written by hand, as claude-359 to claude-364 were: `uor-harness loop claim` refuses while this lab's worktree `.worktrees/claude-361` holds the item (its PR is in the merge path). The two units share no file.

## Files in scope

- `crates/uor-r4-integer/src/place_asks.rs`, `crates/uor-r4-integer/src/bin/place-asks.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs`, `crates/uor-r4-integer/src/stack/tests.rs`, `docs/labs/one-model-end-to-end/instrument/src/bin/raw-id-replies.rs` (only where a table literal needs the new field)
- `docs/loop/tasks/one-model-end-to-end/claude-365/**`, `docs/research/notebook/float-training.md`

## Done when

1. Step 1's counts are in the REPORT for TRAIN and the three read panels, both form rules, with tests that fail without the rule they pin.
2. Either the go rule failed and the REPORT says so with the count, or the build is in with tests (a table of today reads and writes byte for byte as today; a form's floor is taken only for that form) and the fresh panel's per-model table is in the REPORT against the bar.
3. The nine-turn conversation is in the REPORT word for word, with the merged table and the new one.

## Stop rules

- No bar and no go rule is moved after a count or a reply is read.
- The fresh panel is written only after this order is committed, and is read once per arm.
- No pod: every step is a count, a build or an integer read on the laptop.

## Gates

- G1 a table of today is written and read as today: the merged table's bytes are unchanged by a round trip, and the three p models' replies on the seed-21 panel with the merged table equal claude-364's reply files.
- G2 the per-form fit keeps a form's floor only at the count the code names, and a form under it takes the one floor: tests, each failing with its rule set aside.
- G3 the find takes a form's floor for that form and no other: a test that fails with the lookup set aside.
- G4 the fresh panel's time is after this order's commit time.
- G5 REPORT and notebook row.
