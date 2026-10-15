# #1744 — Step 6: natural-dialogue recall generator (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T16:46:42Z
- **Updated:** 2026-10-09T13:09:06Z
- **Closed:** 2026-10-05T17:00:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1744
- **Merged:** 2026-10-05T17:00:51Z
- **Merge commit:** 06ba4f4933f0581351cfca1316a06209c10770fe
- **Base / Head:** main / UOR-Foundation:claude/step6-dialogue-recall
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820.

## Why

`chat-100m2-balp-s1` recalls in context in the M-world format (D19 grounded-session MQAR 105/109 without the sieve) but states the asked value on only 7/30 `multi_turn_memory` rows of the frozen `conversational-v3` panel. Those rows are plain conversational English ("The turtle is Shelby and the goldfish is Flash." … "Which name did we give the turtle?"). This PR adds the training data for that gap: natural dialogues that need the same retrieval. It does not train or evaluate anything.

## What it adds

**New binary.** `crates/uor-r4-training/src/bin/dialogue-recall-corpus.rs` is deterministic from a seed. It writes `uor-r4-chat-corpus/v1` stores (the format `m-world corpus`, `synthetic-memory-corpus` and `mix-chat-corpus` use), so its output can go into a mix as one more source.

**One change to how `dialogue-train` reads a store** (`crates/uor-r4-training/src/dialogue_episodes.rs`). The episode index now accepts an assistant turn whose content and EOS are unmasked (context only); see "Training only on the answers" below.

```text
dialogue-recall-corpus generate out=NEW_ROOT tokenizer=TOKENIZER.json [panels=DIR[,DIR...]] \
  [seed=1] (dialogues=N | token_budget=N) [dev_seed=1000003] [dev_dialogues=300] \
  [protocol=2] [context=384] [samples=200] [train_on=answers|all] [source_commit=SHA]
dialogue-recall-corpus leak out=NEW_ROOT store=STORE_DIR tokenizer=TOKENIZER.json panels=DIR[,DIR...]
```

**Dialogue contents.** Each dialogue has facts in user turns, usually two keys of the same type in one sentence, so the typical error is the other key's value. Sometimes the assistant chooses the values and the user accepts them. A dialogue also has 0–6 distractor exchanges, sometimes a fact turn from a second relation, and then 1–3 questions. There are 16 relations: pet names, purchases, favourite dishes, drink orders, gifts, ages, jobs, travel, addresses, instruments, role names, storage locations, paint colours, appointment times, event dates and garden beds. Each relation has several phrasings for stating a fact, asking about it, correcting it, asking in reverse and answering.

**Question categories.**

| Category | What the dialogue does |
|---|---|
| `binding` | Asks about one of several keys of the same type |
| `update` | A correction such as "Correction: Esme bought peaches, not hummus." Asking the corrected key must give the latest value. Asking the other key must give its unchanged value. |
| `reverse` | Asks who or which key holds a value |
| `assistant_stated` | The assistant chose the value |
| `cross_relation` | One fact in each of two relations |
| `abstain` | Asks about a key that was never given a value (same relation, another relation, or no facts at all). The reply is "You haven't told me that." or a similar sentence, and it names no stated value. About 15% of questions. |

Answers use the relation's own sentence, the bare value, "You said …", "You changed it to …" or "I suggested …".

**Open values.** Value pools are filtered against the panels. Half of all pet and person names are pronounceable synthetic names drawn per dialogue, and towns and streets are also synthetic.

**Distractor bank.** It now holds 455 exchanges after the panel filter (it had 64). They are small talk, feelings, activities, chores, opposites, synonyms, spellings, capitals, short facts and tips, all disjoint from every panel given.

**Checks inside the generator.** Every question carries its expected value, the stated distractor values it must not name and the other keys it must not name; the generator checks each dialogue against these. It also decodes the masked tokens of each answer from the encoded store and checks they hold the expected value. Each split is then loaded and indexed the way `dialogue-train` reads it.

## Training only on the answers (`train_on=answers`, the default)

`train_on=answers` sets the response mask to 1 only on the assistant turns that answer a recall or abstention question, content and EOS. Acknowledgements, suggestions and distractor replies keep their tokens in the context with mask 0. `train_on=all` masks every assistant turn, as the first version did.

**How `dialogue-train` uses the mask.** `geometric-stack dialogue-train` builds `stack_dialogue::DialogueSplit::index` → `dialogue_episodes::EpisodeIndex::new`.

- **Where replies come from.** An episode, which is a reply the trainer can sample, is created only for each contiguous run of mask 1 (`record_response`). `sample_ids` draws uniformly over those episodes.
- **Where the loss goes.** `materialize` copies the document prefix up to the sampled run and puts weight 1 only on that run's targets (`position + 1 >= prefix_positions`). Every earlier token, masked or not, has weight 0.

So a mask-0 assistant turn is never sampled and never trained, but it is present in later episodes' prefixes.

**The index change this needed.** Before this PR, the index refused any unmasked EOS that did not end its document ("unmasked dialogue EOS must terminate its document"). It now also admits an unmasked EOS when the segment since the previous EOS contains the exact assistant marker and no masked token, which is a context-only assistant turn. A stray unmasked EOS is still refused, and so is a partially masked segment.

- Every corpus the old index accepted is indexed identically: the new case was previously an error.
- A `train_on=answers` store **needs a `dialogue-train` built from this head**. An older binary refuses it with that error. `train_on=all` stores load in either.

## Ten sample dialogues (train, seed 1)

Under `train_on=answers` the annotated replies are the only trained (mask-1) turns; every other assistant turn is context. Each annotation gives the category, the value the reply must contain and the stated values it must not name.

**1. `train-0000000` — binding** (relations: gifts)
```text
User: I wrapped the presents today. Xavier is getting a kaleidoscope, and Joaquin is getting a terrarium.
Asst: Sounds good.
User: I'm feeling worried today.
Asst: That sounds hard. Be gentle with yourself today.
User: What is the opposite of borrow?
Asst: The opposite of borrow is lend.
User: What is the capital of Albania?
Asst: The capital of Albania is Tirana.
User: What's another word for scared?
Asst: You could say afraid or frightened.
User: What am I giving Joaquin?
Asst: Joaquin is getting a terrarium.    [binding; expect='terrarium'; forbid=['kaleidoscope']]
User: One more thing: what did I get Xavier?
Asst: You said a kaleidoscope.    [binding; expect='kaleidoscope'; forbid=['terrarium']]
```
**2. `train-0000001` — binding** (relations: garden_beds, gifts; panel-disjoint keys)
```text
User: We planted hostas in the shady corner and parsley in the wheelbarrow.
Asst: Good to know.
User: We planted tulips in the greenhouse.
Asst: Good to know.
User: I got my flatmate a phone stand.
Asst: Okay, I'll keep that in mind.
User: What's another word for hard?
Asst: You could say difficult or tough.
User: What's growing in the shady corner?
Asst: Hostas.    [binding; expect='hostas'; forbid=['parsley', 'tulips', 'phone stand']]
User: What's growing in the wheelbarrow?
Asst: You planted parsley in the wheelbarrow.    [binding; expect='parsley'; forbid=['hostas', 'tulips', 'phone stand']]
```
**3. `train-0000021` — update** (relations: event_dates; panel-disjoint keys)
```text
User: The regatta is on May 21st.
Asst: Noted!
User: The regatta was rescheduled to May 24th. It's no longer May 21st.
Asst: Got it, I've noted the change.
User: What date is the regatta?
Asst: The regatta is on May 24th.    [update:latest; expect='May 24th'; forbid=['May 21st']]
```
**4. `train-0000033` — update** (relations: paint_colors; panel-disjoint keys)
```text
User: We painted the van ivory.
Asst: Got it. The van is ivory.
User: We repainted the van. It's mint now, not ivory.
Asst: Okay, noted.
User: Which color did we pick for the van?
Asst: You painted the van mint.    [update:latest; expect='mint'; forbid=['ivory']]
```
**5. `train-0000009` — reverse** (relations: pet_names, ages)
```text
User: We have a couple of new pets: the chinchilla is called Lougak and the donkey is called Anouk.
Asst: That's nice to hear.
User: My best friend is 41 years old.
Asst: Thanks for telling me.
User: Which of our pets is Anouk?
Asst: Anouk is the donkey.    [reverse; expect='the donkey'; forbid=['Lougak', '41']]
User: Do you remember what the chinchilla is called?
Asst: You named the chinchilla Lougak.    [binding; expect='Lougak'; forbid=['Anouk', '41']]
User: Quick question: what age did my best friend just turn?
Asst: You said 41.    [binding; expect='41'; forbid=['Lougak', 'Anouk']]
```
**6. `train-0000010` — assistant_stated** (relations: gifts)
```text
User: Any gift ideas for Leopold and my best friend?
Asst: How about a flashlight for Leopold and an umbrella for your best friend?
User: Those work for me.
Asst: Good choice.
User: Hey, what am I giving my best friend?
Asst: An umbrella.    [assistant_stated; expect='umbrella'; forbid=['flashlight']]
```
**7. `train-0000016` — cross_relation** (relations: drink_orders, jobs; panel-disjoint keys)
```text
User: I just finished filing my paperwork.
Asst: Nice work! It feels good to get that done.
User: Sunniva ordered ayran. Kalinda is a translator.
Asst: Got it. Sunniva had ayran. Kalinda is a translator.
User: I had a strange dream last night.
Asst: Dreams can be so odd. What happened in it?
User: How do I pack light for a trip?
Asst: Choose clothes that mix and match, and roll them instead of folding.
User: How do you spell calendar?
Asst: It is spelled c-a-l-e-n-d-a-r.
User: I feel like the week is going by slowly.
Asst: Some weeks really drag. Hang in there.
User: What is Kalinda's job?
Asst: Kalinda works as a translator.    [cross_relation; expect='translator'; forbid=['ayran']]
```
**8. `train-0000004` — abstain** (relations: favorite_dishes, drink_orders, addresses)
```text
User: Selma loves gnocchi, and Vashti would eat pho every night if they could.
Asst: I'll remember that.
User: Gideon asked for hibiscus cooler, and Cedric went with iced latte.
Asst: That sounds like fun.
User: What is the capital of Mozambique?
Asst: The capital of Mozambique is Maputo.
User: I'm learning to sew.
Asst: That's a useful skill. Start with simple straight seams.
User: I'm thinking about having a picnic this weekend.
Asst: That sounds like a nice way to spend some time.
User: I can't decide what to watch tonight.
Asst: Maybe pick something light and funny if you want to relax.
User: Where does Yara live?
Asst: You haven't told me that yet.    [abstain:other_relation; abstain; forbid=['gnocchi', 'pho', 'hibiscus cooler', 'iced latte']]
```
**9. `train-0000005` — abstain** (relations: event_dates)
```text
User: What is the opposite of asleep?
Asst: The opposite of asleep is awake.
User: I might try learning to juggle later this month.
Asst: That sounds like a nice way to spend some time.
User: What is the capital of Angola?
Asst: The capital of Angola is Luanda.
User: What's another word for think?
Asst: You could say ponder or consider.
User: Do you ever get bored?
Asst: I don't get bored the way people do, but I enjoy a good conversation.
User: Which date did we set for the quiz night?
Asst: I don't know. You haven't told me that.    [abstain:no_facts; abstain; forbid=[]]
```
**10. `train-0000006` — binding** (relations: purchases, favorite_dishes; panel-disjoint keys)
```text
User: Soraya came home with scones, while my daughter bought peaches.
Asst: Nice!
User: My mother-in-law loves lasagna, and my flatmate would eat gnocchi every night if they could.
Asst: Got it. Your mother-in-law loves lasagna. Your flatmate loves gnocchi.
User: What's my mother-in-law's favorite dish?
Asst: Your mother-in-law loves lasagna.    [binding; expect='lasagna'; forbid=['scones', 'peaches', 'gnocchi']]
```

## Disjointness from the panels (`panels=data/panels,<ladder panel dir>`)

`panels=` takes a comma-separated list. The run below used `data/panels` plus `~/uor-r4-local/ladder/panel`: 33 files and 4,731 strings, including everyday-32, heldout-200 and stretch-32.

- **Value pools.** Every value is dropped if it shares a content-word stem with any panel string or check field: 1,473 panel stems, 203 pool entries dropped.
- **Recall-row value words.** These are the `terms` and `forbid` columns of the recall rows: 108 stems, which include colours, weekdays and two–twelve. None of them appears anywhere in the generated text.
- **Whole turns and 6-grams.** No turn equals a panel string. No turn shares a word 6-gram with a panel string. A draw that would is redrawn.
- **Keys.** Keys may share words with the panels. About 60% of dialogues use only keys that share no content word with any panel. Shared key and frame words are listed with counts.

`leak.json` is computed from the written token stores, decoded back to text.

## Measured on the laptop (no training)

The run used `seed=1 token_budget=2000000 train_on=answers`, both panel directories, tokenizer `bundle-learned-1/tokenizer.json` (`d36d3e87…`), protocol 2 and context 384. It took 2.4 s and peaked at 20 MB RSS.

| | train | dev (`dev_seed=1000003`) |
|---|---|---|
| dialogues / tokens | 9,332 / 1,999,942 (214 tokens per dialogue) | 300 / 66,276 |
| replies dialogue-train can sample (eligible responses, context 384) | **12,877 = exactly the question answers**, none excluded | **421 = the answers** |
| answer share of trained replies / reply tokens | **1.0 / 1.0** | 1.0 |

Questions and trained answer replies (train), with reply tokens including EOS:

| category | replies | reply tokens |
|---|---|---|
| binding | 6,330 | 66,767 |
| abstain | 1,930 (15.0%) | 26,516 |
| update | 1,583 | 16,879 |
| cross_relation | 1,162 | 12,407 |
| reverse | 1,081 | 10,490 |
| assistant_stated | 791 | 8,497 |
| **total** | **12,877** | **141,556** |

Context-only assistant turns (mask 0):

| kind | replies | reply tokens |
|---|---|---|
| distractor | 27,451 | 428,996 |
| acknowledgement | 14,243 | 129,761 |
| suggestion | 791 | 17,163 |

**Comparison with the first version.** It trained every assistant turn: 53,839 trained replies per 2M tokens, of which 23.6% were answers.

**Redraws:** over_context 128, distractor naming a key/value 53, panel 6-gram 5, abstention key mentioned earlier 4, panel turn 1.

**Leak report** (decoded stores, both panel directories):

| split | turns checked | panel-turn matches | shared 6-grams | recall-row value words | value stems shared | strict-key dialogues |
|---|---|---|---|---|---|---|
| train | 110,724 | 0 | 0 | 0 | 0 | 60.2% |
| dev | 3,672 | 0 | 0 | 0 | 0 | 61.7% |

**Determinism.** I ran the same command twice. The two runs gave identical SHA-256 for `train/tokens.u16`, `train/response_mask.u8`, `dev/tokens.u16`, `dev/response_mask.u8` and `leak.json`.

## Tests (exact head, local)

```text
rustfmt --check --edition 2021 crates/uor-r4-training/src/bin/dialogue-recall-corpus.rs crates/uor-r4-training/src/dialogue_episodes.rs   # clean
cargo test -p uor-r4-training --bin dialogue-recall-corpus --release --offline        # 10 passed
cargo test -p uor-r4-training --lib --release --offline dialogue                      # 32 passed (dialogue_episodes, stack_dialogue, dialogue_development, ...)
```

**New tests.**

- **`train_on_answers_leaves_exactly_the_answer_turns_sampleable`.** It writes a split with the real `write_split` under a byte-level tokenizer, loads it with `DialogueSplit::load`, indexes it with `episode_contract_for`, and decodes every episode `dialogue-train` can sample. Under `answers` these are exactly the answer turns, in order. Under `all` they are every assistant turn. `sample_ids` draws only from them.
- **`context_only_assistant_turns_are_prefix_never_responses`** (`dialogue_episodes`). The unmasked turn appears in the prefix with weight 0, and only the answer is supervised. A stray unmasked EOS and a partially masked segment are still refused.
- **`the_distractor_bank_is_large_and_clean`.** The bank holds at least 400 exchanges and none of them leaks.

**Earlier tests, still passing.** Determinism; the answer names the bound value and no distractor; binding keys are stated in one turn; update semantics; abstentions name no stated value and make up 11–20% of questions; no panel turn, 6-gram or value word; the leak detector catches injected cases.

## Generating the Step 6 set (on the pod, from the repo root at this head)

Copy the ladder panel directory to the pod first, for example to `/root/data/ladder-panel`.

```sh
cargo build --release -p uor-r4-training --bin dialogue-recall-corpus --bin mix-chat-corpus --example geometric-stack
target/release/dialogue-recall-corpus generate out=/root/data/ft/dialogue-recall-50m \
  tokenizer=/root/data/tokenizer.json panels=data/panels,/root/data/ladder-panel \
  seed=1 token_budget=50000000 dev_seed=1000003 dev_dialogues=300 \
  protocol=2 context=384 train_on=answers
# then, as one more source of a mix (train with the geometric-stack built from this head):
target/release/mix-chat-corpus out=/root/data/ft/<mix> tokenizer=/root/data/tokenizer.json \
  inputs=<existing store>,/root/data/ft/dialogue-recall-50m/train labels=<label>,dialogue-recall
```

**Expected output.** About 233k train dialogues and roughly 150 MB of store, in about a minute. The source would add about 322k sampleable answer replies. Because the response-uniform sampler gives a source a share proportional to its eligible responses, choose the budget against the base store's response count.

## Limitations and open questions

- **Needs this head's `dialogue-train`.** A `train_on=answers` store loads only in a `dialogue-train` built from this head. Older pod binaries refuse it.
- **Narrow pools.** Some value pools are narrow after the stricter two-directory filter: drinks 21, location phrases 26, instruments 28, colours 37, plants 38. Names, places and streets are open.
- **What the dev split is.** It shares pools and templates with train and is only for monitoring. Whether this data moves `multi_turn_memory` is untested.

Not merged; awaiting exact-head review.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5999163504 — 2026-10-05T16:59:56Z — Casey-allard

Exact-head review at a950bf27 (self-review per owner rule; GitHub refuses self-approval).

- `dialogue_episodes.rs`: the new admission is narrow — an unmasked mid-document EOS is accepted only when its whole segment since the previous EOS/BOS is unmasked and contains the exact assistant marker; masked answer turns never reach the check (selected=1), stray or partly masked turns are still refused (tested). Every store the old index accepted indexes identically (the admitted case was previously an error). Back-scan is bounded by the document length.
- Generator: `train_on=answers` default; leak check over data/panels + ladder panels: 0 turns / 0 6-grams / 0 values; deterministic.
- Checks at this head on the laptop: `rustfmt --check --edition 2021` clean on both files; `cargo test --release -p uor-r4-training --bin dialogue-recall-corpus` 10 passed; `--lib dialogue_episodes` 10 passed.

Approve at this head. Step 6 (pre-registered issuecomment-5999139757) uses it.


---
_Comment/review ids in this file: 5999163504_

