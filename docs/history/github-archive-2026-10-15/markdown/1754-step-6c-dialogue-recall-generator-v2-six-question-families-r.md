# #1754 — Step 6c: dialogue-recall generator v2 (six question families) (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T19:58:35Z
- **Updated:** 2026-10-09T13:09:06Z
- **Closed:** 2026-10-05T21:24:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1754
- **Merged:** 2026-10-05T21:24:44Z
- **Merge commit:** 0a5ccb73cd82eb74d4a83c06d2415cf8e18e3e57
- **Base / Head:** main / UOR-Foundation:claude/step6c-dialogue-recall-v2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. Follows #1744 (merged).

## Why

Step 6/6b (#820 issuecomment-6001644913) found that the v1 recall curriculum gives about +10 on `conversational-v3` memory (6 runs: 15–21/30 against controls 6–10) but plateaus near 18/30. The misses that remain are question families v1 never produced. This PR adds those families to the generator. It does not train or evaluate anything.

## What changes

The only file changed is `crates/uor-r4-training/src/bin/dialogue-recall-corpus.rs`.

`generator=v2` is now the default. Each draw is a v1 dialogue with probability 0.46; otherwise it is one of six new families, each chosen with probability 0.09. Every family keeps v1's binding pressure, natural varied phrasing, open value pools, 0–6 distractor exchanges and 1–3 questions. Each also abstains about one time in ten, only for a key that was never stated.

| family | what is stated | questions |
|---|---|---|
| `rule` | Negation or contrast: "X is allowed but Y is not", "we can … but we can't …", "Y is banned, though X is okay", "… X yes, Y no". Sometimes two settings. | What is (not) allowed, also as a choice ("is X or Y allowed?"). Abstains for an unstated setting. |
| `implicit_update` | A plan changes without "actually": "the plan changed, we're … instead", "we voted again and chose …", "it moved to …", "now it's …", "new plan: …". Sometimes a second change; sometimes a second plan that does not change. | The latest value (wins); the first plan ("what was the first plan?"); the unchanged plan. Abstains for an unstated plan. |
| `self_fact` | "I'm making X, and my friend N is making Y" (9 relations). Sometimes a second first-person fact. | "What am I …?" is answered from the user's stated fact. "What is N …?" binds to N. Abstains only for a relation the user never stated (rare) or a third person, so "I/me" questions are mostly answered. |
| `attribute` | Two or three objects of one kind told apart by colour, material or size, with different locations or owners. | Ask by attribute; the distractor is the other object's location or owner. Reverse ("which mug is in the attic?"). Abstains for an unmentioned attribute. |
| `count` | Counts per container (the same noun told apart by colour, or different nouns) or per person, as numerals (13–60) and number words. | The count for one container or person. Abstains for an unmentioned container or person. |
| `order` | A sequence of 3–4 events: "starts with A, after that B, and then C", "first … later … last", "this morning … after lunch … in the evening", "at 14:15 … at 16:30 …". | First, second, third or last; after X; before X; at a time; in a day part. Abstains for an unmentioned time. |

**v1 behaviour is kept.** With the same panel directories, `generator=v1` reproduces #1744's stores byte for byte: `train/tokens.u16`, `response_mask.u8`, `samples.jsonl` and the dev split, all checked against the #1744 run. Under v2, `train_on=answers` (the default) still trains only the question answers.

**Acknowledgements are filtered against the panels.** They go through the same panel filter as the distractor bank. The reason: "Got it." is a check phrase in the new `conversational-v4` panel, which had been forcing about 1,000 redraws per 2M tokens.

**Reporting.** `generator.json` now has:
- `dialogues_per_family` and `questions_per_family`;
- trained reply and token counts per category;
- the v2 pool sizes and shares.

`leak.json` lists the v2 entries dropped by the panel filter.

## Two sample dialogues per new family (train, seed 1)

Each annotation shows the category, the expected value and the stated values the reply must not name. Only the annotated replies are trained (`train_on=answers`).

**rule — `train-0000010`**
```text
User: What is the closest star to Earth?
Asst: The sun is the closest star to Earth.
User: They told us meditating is permitted in the classroom, but skydiving is not.
Asst: Nice!
User: What can't we do in the classroom?
Asst: Skydiving is not allowed.    [rule:not_allowed; expect='skydiving'; forbid=['meditating', 'meditate']]
```
**rule — `train-0000069`**
```text
User: At the concert hall, we can rollerblade but we can't toboggan.
Asst: That's nice to hear.
User: I'm feeling inspired today.
Asst: I'm glad you're feeling inspired. Enjoy it.
User: What is the capital of Denmark?
Asst: The capital of Denmark is Copenhagen.
User: What can we do at the concert hall?
Asst: Rollerblading.    [rule:allowed; expect='rollerblading'; forbid=['tobogganing', 'toboggan']]
```
**implicit_update — `train-0000043`**
```text
User: The fundraiser menu is going to be quesadillas.
Asst: That sounds like fun.
User: What's another word for begin?
Asst: You could say start or commence.
User: Everyone wanted lamb stew, so we switched.
Asst: Sounds good.
User: Do you ever get bored?
Asst: I don't get bored the way people do, but I enjoy a good conversation.
User: What did we settle on for the fundraiser menu?
Asst: You settled on lamb stew.    [implicit_update:latest; expect='lamb stew'; forbid=['quesadillas']]
```
**implicit_update — `train-0000149`**
```text
User: We decided on crepes for the team dinner dish.
Asst: Okay, thanks.
User: For the farewell meal, we picked moussaka.
Asst: Nice!
User: The plan for the team dinner dish changed, we're making tamales instead.
Asst: Okay, thanks for the update.
User: What's another word for dirty?
Asst: You could say messy or grimy.
User: What did we settle on for the team dinner dish?
Asst: The team dinner dish is tamales now.    [implicit_update:latest; expect='tamales'; forbid=['crepes', 'moussaka']]
```
**self_fact — `train-0000013`**
```text
User: My friend Chiara wants mango lassi, and I'll have chai.
Asst: That's nice to hear.
User: I work as an architect.
Asst: I'll remember that.
User: What is the capital of Ireland?
Asst: The capital of Ireland is Dublin.
User: Remind me, what did I order?
Asst: You ordered chai.    [self_fact:self; expect='chai'; forbid=['mango lassi', 'architect']]
User: What did Ragnar order?
Asst: I'm not sure. You never told me about Ragnar.    [abstain:other_unstated_person; abstain]
```
**self_fact — `train-0000020`**
```text
User: I'm making dal, and my friend Zephyr is making pierogi.
Asst: That sounds like fun.
User: I'm feeling grateful today.
Asst: I'm glad you're feeling grateful. Enjoy it.
User: Remind me, what dish am I making?
Asst: You're making dal.    [self_fact:self; expect='dal'; forbid=['pierogi']]
User: What is my friend Zephyr making?
Asst: Pierogi.    [self_fact:other; expect='pierogi'; forbid=['dal']]
```
**attribute — `train-0000002`**
```text
User: We have a couple of rings: the chunky one is in the filing cabinet and the giant one is in the garage.
Asst: Okay, thanks.
User: Hey, where is the chunky ring?
Asst: In the filing cabinet.    [attribute:location; expect='in the filing cabinet'; forbid=['in the garage']]
User: What is the opposite of victory?
Asst: The opposite of victory is defeat.
User: Which ring is in the garage?
Asst: The giant one.    [attribute:reverse; expect='giant'; forbid=['in the filing cabinet']]
User: One more thing: where did I put the oversized ring?
Asst: You haven't told me about an oversized ring.    [abstain:attribute_unstated; abstain]
```
**attribute — `train-0000022`**
```text
User: The silk thimble went in the recycling bin, while the bamboo thimble is in the linen cupboard.
Asst: Sounds good.
User: What is the opposite of begin?
Asst: The opposite of begin is finish.
User: How can I save money on groceries?
Asst: Plan meals for the week, shop with a list, and compare prices per unit.
User: Hey, where can I find the silk thimble?
Asst: The silk thimble is in the recycling bin.    [attribute:location; expect='in the recycling bin'; forbid=['in the linen cupboard']]
```
**count — `train-0000018`**
```text
User: I counted fifty-one spoons in the crate, and 56 on the tray.
Asst: Okay, thanks.
User: I might try walking along the river later this month.
Asst: That sounds like a nice way to spend some time.
User: The birds outside are so loud this morning.
Asst: It sounds like they are having a busy morning.
User: How many spoons are on the tray?
Asst: 56.    [count:per_container; expect='56'; forbid=['fifty-one']]
```
**count — `train-0000023`**
```text
User: Vashti has 14 nails, and Gwendolyn has 37.
Asst: Sounds good.
User: What's another word for help?
Asst: You could say assist or support.
User: What's another word for walk?
Asst: You could say stroll or wander.
User: What's Gwendolyn's count?
Asst: 37.    [count:per_person; expect='37'; forbid=['14']]
```
**order — `train-0000003`**
```text
User: What is the deepest part of the ocean?
Asst: The Mariana Trench is the deepest known part of the ocean.
User: In the morning there's badminton, in the afternoon kickboxing, and at night a haircut.
Asst: Sounds good.
User: What's on in the morning?
Asst: In the morning, you have badminton.    [order:day_part; expect='badminton'; forbid=['kickboxing', 'a haircut']]
User: What's the second thing?
Asst: Kickboxing.    [order:second; expect='kickboxing'; forbid=['badminton', 'a haircut']]
```
**order — `train-0000015`**
```text
User: Acupuncture is at 16:15, calligraphy at 21:30, and a photo shoot at 22:35.
Asst: Good to know.
User: What comes first?
Asst: Acupuncture.    [order:first; expect='acupuncture'; forbid=['calligraphy', 'a photo shoot']]
User: How can I save money on groceries?
Asst: Plan meals for the week, shop with a list, and compare prices per unit.
User: Sorry, I forgot. What do I have after calligraphy?
Asst: Next is a photo shoot.    [order:after; expect='a photo shoot'; forbid=['acupuncture', 'calligraphy']]
```

## 2M-token laptop sample (no training)

The run used `generator=v2 seed=1 token_budget=2000000 train_on=answers`, protocol 2, context 384 and tokenizer `d36d3e87…`. The panels were `data/panels` plus the local ladder panel directory: 38 files and 5,531 strings, **including `conversational-v4`**, which landed there at 15:51 today. The run took 2.1 s and peaked at 20 MB RSS.

| | train | dev |
|---|---|---|
| dialogues / tokens | 9,446 / 1,999,945 | 300 / 64,976 |
| replies dialogue-train can sample (= question answers) | 13,670 (146,021 tokens) | 451 |

Dialogues and trained answer replies per family (train):

| family | dialogues | answers in family category | abstentions |
|---|---|---|---|
| v1 | 4,364 | 5,117 (binding 2,973, update 709, cross 512, reverse 542, assistant-stated 381) | 886 |
| rule | 864 | 1,163 (allowed 408, not allowed 449, choice 306) | 175 |
| implicit_update | 779 | 975 (latest 537, first 362, unchanged 76) | 181 |
| self_fact | 898 | 1,131 (self 685, other 446) | 261 (unstated person 192, unstated own relation 69) |
| attribute | 842 | 1,112 (location 560, owner 379, reverse 173) | 166 |
| count | 864 | 1,080 (per person 434, per container 646) | 190 |
| order | 835 | 1,206 (after 300, before 235, first 166, second 156, last 162, day part 95, at time 74, third 18) | 27 |

- Abstentions are 13.8% of all questions.
- Redraws: over_context 110, distractor naming a key or value 87, abstention key mentioned earlier 3, panel 6-gram 2, panel turn 2.

**Leak report** (decoded stores, all 38 panel files, including v4):

| split | turns checked | panel-turn matches | shared 6-grams | recall-row value words | value stems shared | strict-key dialogues |
|---|---|---|---|---|---|---|
| train | 112,820 | 0 | 0 | 0 | 0 | 60.5% |
| dev | 3,662 | 0 | 0 | 0 | 0 | 57.0% |

`conversational-v3` stays untouched as data. None of its sentences, 6-grams or values appear: its values and every panel content word are excluded from the value pools, and its recall-row values (and v4's) appear nowhere in the text.

**Determinism.** I ran the same command twice. The two runs gave identical SHA-256 for `train/tokens.u16`, `train/response_mask.u8`, `train/samples.jsonl`, `dev/tokens.u16`, `dev/response_mask.u8` and `leak.json`.

## conversational-v4 overlaps the v1 pools (please read)

Adding the v4 files to `panels=` removed about 230 entries from the v1 value pools that v4 also uses. These values were therefore in the v1 corpora trained before v4 existed: Step 6/6b and any `dialogue-recall` source built from #1744. Among them:

- **Names:** Teodor, Odette, Thea, Ezra, Saoirse, Dario, Ingrid, Kwame, Amara, Clementine, Yusuf and Mireille.
- **Pet names:** Nutmeg, Tofu, Mochi, Quill and Pudding.
- **Colours:** lavender, maroon and turquoise.
- **Places and drinks:** Halifax, Lisbon, Prague and lemonade.
- **Instruments:** tuba, violin, cello, flute, harp and others.
- **Locations:** "in the attic" and "in the cellar".
- **Streets:** every Avenue, Lane and Street on the tree and bird names.
- **Dates:** the April and September dates.

The keys the ferret, the gerbil, the canoe and the attic also became v4 recall values and are now dropped. The full list is in `leak.json` (`dropped_by_panel_filter`).

This means v4 is not value-disjoint from the v1 training data. If v4 is meant as a held-out panel for models trained on v1 data, its memory rows that use these values are not held out for those models. That is for the v4 authors and the coordinator to judge. Any corpus generated from this head with v4 in `panels=` excludes them.

## Tests (exact head, local)

```text
rustfmt --check --edition 2021 crates/uor-r4-training/src/bin/dialogue-recall-corpus.rs   # clean
cargo test -p uor-r4-training --bin dialogue-recall-corpus --release --offline            # 17 passed
```

One test per new family checks answer correctness against the stated text, not the generator's own bookkeeping:

- **`rules_answer_the_allowed_or_banned_activity`.** The clause that states the expected activity (gerund or base form) carries a negation marker exactly when the question asks what is *not* allowed.
- **`implicit_updates_keep_the_latest_value_and_can_recall_the_first`.**
  - For a "latest" question, the old value is stated before the expected one and is forbidden.
  - For a "first" question, the expected value is stated before the latest one.
  - No statement says "actually".
- **`self_facts_are_answered_and_bound_to_the_right_person`.** A self answer comes from a clause without the other person's name, and is never an abstention. An other-person answer comes from that person's clause. Self questions outnumber other-person questions, and abstentions stay under a quarter.
- **`attribute_objects_are_told_apart_by_their_attribute`.** The clause holding the expected location or owner contains the asked attribute and no other object's attribute. Reverse questions name the object's location.
- **`counts_belong_to_their_container_or_person`.** The clause holding the count contains what distinguishes its container or person and nothing that distinguishes the others.
- **`temporal_order_answers_follow_the_stated_sequence`.** Answers to first, second, third and last match the order of appearance. After and before name the neighbouring event. Answers at a time or in a day part match the anchor.
- **`v2_draws_are_deterministic_clean_and_mixed`.** It checks same-seed identity over 3,000 draws, the semantic contract, no panel turn, 6-gram or value word, panel-disjoint values, and every family present.

The 10 v1 tests pass unchanged, including the trainer-sampling test.

## Generating (on the pod, from the repo root at this head; ladder panels, including v4, copied to `/root/data/ladder-panel`)

```sh
cargo build --release -p uor-r4-training --bin dialogue-recall-corpus --bin mix-chat-corpus
target/release/dialogue-recall-corpus generate out=/root/data/ft/dialogue-recall-v2-50m \
  tokenizer=/root/data/tokenizer.json panels=data/panels,/root/data/ladder-panel \
  seed=1 token_budget=50000000 dev_seed=1000003 dev_dialogues=300 \
  protocol=2 context=384 train_on=answers generator=v2
```

**Expected output.** About 236k dialogues and about 342k trained answer replies, in about a minute. The stores still need the `dialogue-train` from #1744 or later.

## Limitations and open questions

- **No new evidence.** Whether these families lift `conversational-v3`/`v4` memory past the ~18/30 plateau is untested.
- **Narrow v2 pools.** Some pools are narrow after the two-directory filter: rule activities 26, order events 39, count values 47, size attributes 8. A few combinations are odd but grammatical ("at the concert hall, we can rollerblade").
- **Re-run when v4 lands in `data/panels`.** `conversational-v4` is not yet in `data/panels` on main. It is covered here through the ladder directory. When its PR lands, re-run the leak check with `panels=data/panels` alone.

Not merged; awaiting exact-head review.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6001958169 — 2026-10-05T19:59:20Z — Casey-allard

Exact-head check at 8740a09c (Claude lab): `rustfmt --check --edition 2021` clean; `cargo test --release -p uor-r4-training --bin dialogue-recall-corpus` 17 passed (incl. one answer-correctness test per new family). Single-file change; `generator=v1` reproduces #1744 byte-for-byte per the author. On the v4 value overlap: it only affects the Step 6/6b B models (trained on v1 pools), which are not part of the Step 6c decision; 6c arm C trains on v2 data generated with v4 in the leak filter, and controls A never saw dialogue-recall data. Approve at this head.


---
_Comment/review ids in this file: 6001958169_

