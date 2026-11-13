# REPORT: one-model-end-to-end, claude-367 (pair experts check the reply)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3958.

The order asked for exact pair experts counted from TRAIN that check each everyday reply of the served model against the turn's own tokens and answer in its place when the reply was never seen with them, read beside the same model without them on a panel written after the settings. The experts, their table file, the check in the chat and the panel replay are built. **The bar reads "Adds":** held out, good everyday replies on the seed-24 panel go from 137, 135 and 134 of 239 to 170, 171 and 170 (D +33, +36, +36; M +35.0 against a bar of +15), with every guard held. The fact conversation, on which the saved models alone answer 1, 2 and 4 of 13 turns right, is answered on 10 of 13 turns held out and 11 of 13 as served, on all three models. Nothing here is a milestone claim, and nothing is served as a product.

## What did not work, first

1. **The frame outvotes the key word.** Every pair of the turn has one vote, and a question has many more pairs of frame words than pairs that hold its key word. "How many legs does a spider have?" has 28 pairs, 7 with "spider"; TRAIN holds the turn 221 times with the right answer, and with the experts on every model still says "It has four legs." or "It has two legs.", held out and as served. Held out, the experts turn "Can birds fly?" from "Yes." to "No." (three rows) and "Can dogs fly?" from "No." to "Yes.": the four good rows each model loses.
2. **A wrong reply stands when it holds more than 1/32 of the top vote.** On p-s1, 19 wrong everyday replies stand. "Tell me the capital of China." answered "It's Madrid." holds 9% of the top vote; "What colour is a grape?" answered "It's red." holds 23%. The ratio was fixed on the seed-23 panel, where the count moved by at most one row a model between 1/16 and 1/256, and is not moved here.
3. **Number work and lists are not fixed**, as the order named. Adding is 0 of 9 on every model in every setting. Counting is 1 of 13 off and held out on every model, and 5, 6 and 8 of 13 as served. Lists are 2 of 11 on p-s1 and 1 of 11 on p-s2 and p-s3, in every setting. Where the experts answer such a turn they give a stored reply for other numbers: "Please count from 4 to 9." gets "3, 4, 5, 6, 7, 8, 9."; "What is 5 plus 1?" gets "Noted." Of the 17 rows on p-s1 where the experts replaced a wrong reply with another wrong one, most are these and "use the word in a sentence" turns ("I feel happy when I am with my friends." for river, friend and dog).
4. **Two rows outside the everyday set reached the experts on every model**, where the order's first done-when item says a told-value ask and a repeat ask never do. Rule 1 and its tests hold back a turn the counted table finds something for; these are two turns it finds nothing for. Told-value row 314, "What is soundnastmou?": the experts answer "Okay, I have all of them." The model's own reply was wrong as well, so the told count does not move (548 of 571). Row 668 of the repeat set, whose last turn is a plain "See you later!": the experts let the model's "Goodbye! Have a great day." stand, held out and as served, so no count moves.
5. **Small talk loses up to three good rows held out.** Rows whose reply needs no word of the turn: 104 good to 101, 102 and 103. The guard is 3, so p-s1 sits on it. These are the four yes/no rows of item 1, less the rows gained.
6. **25 everyday rows on p-s1 stay wrong because the experts' top vote is under the floor** (adding 8, lists 9, "How do you spell …?" 3, counting 2, news of a new thing 2, first letter 1): they do not know the turn and the model's wrong reply stands. Four more are left alone by the counted table and are wrong (three long repeat rows of invented words that the model does not copy exactly, and "Do cats bark?" answered "I'm not sure, you didn't tell me.").
7. **Turn 8 of the nine-turn conversation is wrong on every model** ("What is two plus three?"); on p-s1 the reply "Noted." stands. The experts change one turn of that conversation, turn 6.
8. **The number on the bar is a replay, not a served read.** The reply reader's flags were not built (amendment 4). `place-asks experts-check` replays the replies each model had already written through `PairExperts::check_prompt`, the function the chat calls. The chat adds one rule the replay does not have (a reply that does not fit in the caller's tokens is not given); the longest reply the experts gave on this panel is 32 tokens against the reader's 48, so no row is touched.
9. **The as-served number measures memory of the corpus.** 235 of the 239 everyday turns stand in TRAIN word for word. With nothing held out G is 191, 193 and 194; the held-out reading is the one the bar is on.
10. **The fresh panel gains less than the sizing panel.** The settings were chosen on the seed-23 panel, where D was +40, +38 and +41; on the seed-24 panel it is +33, +36 and +36.
11. **Cost.** The table file is 67,291,076 bytes beside an artifact of 19,345,344. The chat program takes about 1.2 s from start to end on the fact conversation against 0.22 s (the file is read and its sum checked once), and its slowest turn takes 0.07 s against 0.014 s. Each check walks every count of every pair of the turn.
12. **Two medium findings came out of the fresh review and are fixed; low items are left open** (the review paragraph below lists them).
13. **Not run.** `cargo clippy`; `cargo fmt --check` on files this unit did not create; the reader's flags, which do not exist. No test drives the chat program's real argument parser or its script record with the experts on: the record was read by hand in fifteen conversations.
14. **Two owner readings are open**, neither blocking: whether a table counted from TRAIN may be served in a product (CHARTER §3.1), open since claude-365; and whether reading and counting served replies on the laptop as development research sits inside §3.6.

## What changed

- **The experts** (`crates/uor-r4-integer/src/pair_experts.rs`, new): each token has its prime; an expert is an unordered pair of different tokens of the last user turn, addressed by the product of the two primes (under 2^31, so at most 4,792 tokens); it holds the replies it was seen with and how often. A vote is the sum over the turn's experts of the reply's count over the expert's count, in 20-bit fixed point; ties go to the smaller reply index. `check_prompt` applies the order's three rules and the floor of amendment 2. The table file (magic `UORPEX01`) carries the SHA-256 of the tokens it was counted on and of its body, and is refused on other tokens, another vocabulary or a damaged body.
- **The table tool and the replay** (`bin/place-asks.rs`): `experts` counts the table from a token file; `experts-check` replays a panel's written replies through `check_prompt` for a grid of settings and writes the checked replies.
- **The check in the chat** (`stack/chat.rs`, with a getter in `stack/hamming.rs` and `stack/session.rs`): after the model has written its reply and the serve-again step has decided, the reply is checked. A replaced reply is the only thing emitted, the history holds it, and the next turn is served as a fresh chat serves that history. With no table set, every id is as before.
- **The chat program** (`bin/uor-chat.rs`): `--pair-experts FILE --pair-cap N --pair-keep NUM/DEN --pair-floor N [--pair-hold-out 0|1]`; in `--script` mode the record names the table and each turn its verdict.
- **The settings** ([settings.json](settings.json)), fixed on the seed-23 panel and committed in `dab9e1160c140bcf872e044cc2b5f82a946462cf` before the seed-24 panel was written: no cap, a keep ratio of 1/32, a floor of 1.5 whole votes. The grids they were read from are [grid23-caps.tsv](grid23-caps.tsv) and [grid23.tsv](grid23.tsv).
- **The order's amendments**, all before the panel: the floor, the settings, the replay in place of the reader's flags, two files added to scope. The bars are unchanged.
- No trained model, counted table or reader source is changed. No pod and no training.

## Measurements

**The table.** Counted from claude-349's TRAIN tokens (SHA-256 `e25775f3aad4548e238736e02a3185e6c57062b8962a4147aafb655a6d865f32`): 215,017 cases, none dropped; 256,712 experts, 61,725 replies, 83,954 turns, 6,464,167 counts; 67,291,076 bytes, SHA-256 `21d458fc374a88cb8b367a4edcf0c524ba73562bb420e6e7ba11271a714a6ad0`. A debug build counts it in 71 s.

**The panel.** Seed 24, written 2026-11-13T19:29:42Z by claude-356's writer, after the settings commit (19:28:50Z); its sums were committed in `b4e680b0386b5ded919814e13096ec61a0f829e8` before any reply was read ([panel-24.sha256](panel-24.sha256)). 990 rows written, 8 equal to an older panel's, 982 read: 239 everyday, 571 told questions, 84 never-told questions, 88 repeat rows. Every model is the saved model of claude-358 (`p-s1`, `p-s2`, `p-s3`), served once with the counted table as claude-365 merged it and the whole conversation as the prompt; each setting was read once.

**The bar.** G is claude-366's count: everyday replies that are the reference or another reply TRAIN gives the same turn, and end. D is G held out less G off.

| Model | G off | G held out | D | G as served | Told, of 571: off, held, served | Never told says so, of 84: off, held, served | No word of the turn, good of 114: off, held | Guards |
|---|---|---|---|---|---|---|---|---|
| p-s1 | 137 | 170 | +33 | 191 | 548, 548, 548 | 78, 78, 78 | 104, 101 | hold |
| p-s2 | 135 | 171 | +36 | 193 | 548, 548, 548 | 78, 78, 78 | 104, 102 | hold |
| p-s3 | 134 | 170 | +36 | 194 | 548, 548, 548 | 78, 78, 78 | 104, 103 | hold |

M is +35.0. "Adds" needs +15 with told not down by more than 3, never-told by more than 2, and good replies that need no word of the turn by more than 3 on each model. **Reads: Adds.** G comes from claude-366's count script on the checked replies; the replay tool's own tally gives the same numbers (170, 171, 170 held out; 191, 193, 194 as served).

**What the everyday replies are**, of 239.

| Model, setting | G | The reference | Another TRAIN reply | A reply TRAIN never gives the turn | Never ends |
|---|---|---|---|---|---|
| p-s1 off | 137 | 65 | 72 | 72 | 30 |
| p-s1 held out | 170 | 93 | 77 | 59 | 10 |
| p-s1 as served | 191 | 103 | 88 | 41 | 7 |
| p-s2 off | 135 | 65 | 70 | 72 | 32 |
| p-s2 held out | 171 | 94 | 77 | 60 | 8 |
| p-s2 as served | 193 | 107 | 86 | 39 | 7 |
| p-s3 off | 134 | 64 | 70 | 87 | 18 |
| p-s3 held out | 170 | 92 | 78 | 59 | 10 |
| p-s3 as served | 194 | 106 | 88 | 36 | 9 |

**Good replies by what the reply needs** (off, held out, as served).

| Model | No word of the turn, of 114 | A word of the turn, of 39 | A fact about the turn's key word, of 51 | Work on the turn's numbers, of 35 |
|---|---|---|---|---|
| p-s1 | 104, 101, 109 | 9, 24, 33 | 16, 36, 36 | 8, 9, 13 |
| p-s2 | 104, 102, 110 | 8, 24, 32 | 15, 36, 37 | 8, 9, 14 |
| p-s3 | 104, 103, 111 | 9, 23, 32 | 13, 35, 35 | 8, 9, 16 |

**By form, p-s1** (good rows off, held out, as served).

| Form | Rows | Off | Held out | As served |
|---|---|---|---|---|
| thanks | 50 | 50 | 50 | 50 |
| goodbye | 31 | 31 | 31 | 31 |
| other small talk and yes/no | 25 | 17 | 13 | 20 |
| opposite of a word | 15 | 7 | 15 | 15 |
| sentence with a word | 14 | 1 | 7 | 13 |
| bigger of two | 13 | 7 | 8 | 8 |
| count a to b | 13 | 1 | 1 | 5 |
| list N of a class | 11 | 2 | 2 | 2 |
| add two numbers | 9 | 0 | 0 | 0 |
| first letter of a word | 9 | 4 | 8 | 9 |
| repeat words | 9 | 2 | 5 | 5 |
| capital of a country | 6 | 0 | 5 | 5 |
| colour of a thing | 6 | 0 | 2 | 2 |
| legs of an animal | 5 | 5 | 5 | 5 |
| news of a thing | 5 | 2 | 2 | 4 |
| a feeling | 4 | 4 | 4 | 4 |
| day after a day | 4 | 0 | 4 | 4 |
| greeting | 4 | 2 | 3 | 4 |
| sound of an animal | 4 | 2 | 3 | 3 |
| spell a word | 2 | 0 | 2 | 2 |

**What the experts did with the 239 everyday rows.**

| Model, setting | Left alone by the counted table | Reply stands | Experts unsure (under the floor) | Replaced | Newly good | Good rows lost | Told-value rows replaced |
|---|---|---|---|---|---|---|---|
| p-s1 held out | 4 | 146 | 31 | 58 | 37 | 4 | 1 |
| p-s2 held out | 4 | 144 | 29 | 62 | 40 | 4 | 1 |
| p-s3 held out | 4 | 139 | 32 | 64 | 40 | 4 | 1 |
| p-s1 as served | 4 | 156 | 17 | 62 | 54 | 0 | 1 |
| p-s2 as served | 4 | 153 | 16 | 66 | 58 | 0 | 1 |
| p-s3 as served | 4 | 149 | 18 | 68 | 60 | 0 | 1 |

No never-told row and no repeat row was replaced; one row of the repeat set reached the experts and its reply stood (negative 4). The 69 everyday rows still wrong on p-s1 held out: 25 where the experts were unsure, 19 where a wrong reply stood, 17 replaced by another wrong reply, 4 good rows lost, 4 left alone.

Rows the experts newly got right on p-s1, held out, with the model's own reply first: "Which is bigger, 1 or 13?" ("2, 13?", then "13 is bigger."); "Repeat after me: the cat is happy." ("The sun is happy.", then "The cat is happy."); "What is the color of a tomato?" ("It's good to feel excited.", then "It's red."); "Spell the word bed." ("red is spelled r-e-e-e.", then "bed is spelled b-e-d."); "What is the first letter of kite?" (a reply about fish, then "kite starts with the letter K.").

**The conversations, word for word.** The saved models, the counted table, the place read spoken, served again; the experts at the committed settings. **[E]** marks a reply the experts gave, [u] one where they were unsure, [t] one the counted table kept from them. A reply over 70 characters is cut at "…"; a line break inside a reply is shown as " / ".

**p-s1, the fact conversation.**

| # | Turn | Experts off | Held out | As served |
|---|---|---|---|---|
| 1 | How are you? | I'm good, thanks for asking. | I'm good, thanks for asking. | I'm good, thanks for asking. |
| 2 | Do you know the capital of Greece? | The capital of Greece is didn?? | It's Athens. **[E]** | It's Athens. **[E]** |
| 3 | What day comes after Thursday? | It's  on on on *(no end)* | The day after Thursday is Friday. **[E]** | It's Friday. **[E]** |
| 4 | Give me the opposite of loud. | It's  on on on *(no end)* | The opposite of loud is quiet. **[E]** | The opposite of loud is quiet. |
| 5 | What colour is a lemon? | It's  on on on *(no end)* | It's yellow. **[E]** | It's yellow. **[E]** |
| 6 | What is the first letter of water? | It's  on on on *(no end)* | water starts with the letter W. **[E]** | water starts with the letter W. **[E]** |
| 7 | What does a duck say? | It's  on on on *(no end)* | Quack! That's the sound a duck makes. | Quack! That's the sound a duck makes. |
| 8 | Which city is the capital of Japan? | It's  on on on *(no end)* | Tokyo is the capital of Japan. **[E]** | Tokyo is the capital of Japan. **[E]** |
| 9 | Spell the word milk. | It's  on on on *(no end)* | red is spelled r-e-e-e *(no end)* [u] | milk is spelled m-i-l-k. **[E]** |
| 10 | How many legs does a spider have? | It's  on on on *(no end)* | It has four legs. **[E]** | It has four legs. |
| 11 | What is 4 + 6? | It's  on on on *(no end)* | 4 plus 4 is 1. [u] | 4 plus 4 is 1. [u] |
| 12 | Thanks for the help. | It's  on on on *(no end)* | You're welcome! | You're welcome! |
| 13 | Goodbye. | It's  on on on *(no end)* | Bye! Take care. | Bye! Take care. |

**p-s2, the fact conversation.**

| # | Turn | Experts off | Held out | As served |
|---|---|---|---|---|
| 1 | How are you? | I'm good, thanks for asking. | I'm good, thanks for asking. | I'm good, thanks for asking. |
| 2 | Do you know the capital of Greece? | The capital of Greece isice, thanks for asking. | It's Athens. **[E]** | It's Athens. **[E]** |
| 3 | What day comes after Thursday? | It's Monday. | The day after Thursday is Friday. **[E]** | It's Friday. **[E]** |
| 4 | Give me the opposite of loud. | The opposite of loud isice, thanks for asking. | The opposite of loud is quiet. **[E]** | The opposite of loud is quiet. |
| 5 | What colour is a lemon? | It's Monday. | It's yellow. **[E]** | It's yellow. **[E]** |
| 6 | What is the first letter of water? | fish starts with the letter P. | water starts with the letter W. **[E]** | water starts with the letter W. **[E]** |
| 7 | What does a duck say? | Quack! That's the sound a duck makes. | Quack! That's the sound a duck makes. | Quack! That's the sound a duck makes. |
| 8 | Which city is the capital of Japan? | The capital of Japan is the capital of Japan. | Tokyo is the capital of Japan. **[E]** | Tokyo is the capital of Japan. **[E]** |
| 9 | Spell the word milk. | red is spelleded s s s *(no end)* | red is spelleded s s s *(no end)* [u] | milk is spelled m-i-l-k. **[E]** |
| 10 | How many legs does a spider have? | red is spelleded s s s *(no end)* | It has four legs. **[E]** | It has two legs. |
| 11 | What is 4 + 6? | red is spelleded s s s *(no end)* | 4 plus 4 is 15. [u] | 4 plus 4 is 15. [u] |
| 12 | Thanks for the help. | red is spelleded s s /  I small small small *(no end)* | You're welcome! | You're welcome! |
| 13 | Goodbye. | red is spelleded s s s *(no end)* | Bye! Take care. | Bye! Take care. |

**p-s3, the fact conversation.**

| # | Turn | Experts off | Held out | As served |
|---|---|---|---|---|
| 1 | How are you? | I'm good, thanks for asking. | I'm good, thanks for asking. | I'm good, thanks for asking. |
| 2 | Do you know the capital of Greece? | The capital of Greece is bigger. | It's Athens. **[E]** | It's Athens. **[E]** |
| 3 | What day comes after Thursday? | It's white. | The day after Thursday is Friday. **[E]** | It's Friday. **[E]** |
| 4 | Give me the opposite of loud. | The opposite of loud is bigger. | The opposite of loud is quiet. **[E]** | The opposite of loud is quiet. |
| 5 | What colour is a lemon? | It's white. | It's yellow. **[E]** | It's yellow. **[E]** |
| 6 | What is the first letter of water? | fish starts with the letter E. | water starts with the letter W. **[E]** | water starts with the letter W. **[E]** |
| 7 | What does a duck say? | Quack! That's the sound a duck makes. | Quack! That's the sound a duck makes. | Quack! That's the sound a duck makes. |
| 8 | Which city is the capital of Japan? | The capital of Japan is the capital of Japan. | Tokyo is the capital of Japan. **[E]** | Tokyo is the capital of Japan. **[E]** |
| 9 | Spell the word milk. | red is spelled ha's's the sound a duck makes. | red is spelled ha's's the sound a duck makes. [u] | milk is spelled m-i-l-k. **[E]** |
| 10 | How many legs does a spider have? | It has four legs. | It has four legs. | It has four legs. |
| 11 | What is 4 + 6? | 4 plus 4 is 11. | 4 plus 4 is 11. [u] | 4 plus 4 is 11. [u] |
| 12 | Thanks for the help. | You're welcome! | You're welcome! | You're welcome! |
| 13 | Goodbye. | Bye! Take care. | Bye! Take care. | Bye! Take care. |

**p-s1, the nine-turn conversation.**

| # | Turn | Experts off | As served |
|---|---|---|---|
| 1 | Keep these in mind: druvalk is 73, mibtorn is felsharp, quonzet is 18. | Got it. | Got it. |
| 2 | What is mibtorn? | felsharp,, quon. | felsharp,, quon. [t] |
| 3 | What does a cow say? | A cow says moo. | A cow says moo. |
| 4 | Say exactly this: trelvon dask pimmerholt zane | trelvon dask pimmerholt zane | trelvon dask pimmerholt zane [t] |
| 5 | What is druvalk? | That's 73. | That's 73. [t] |
| 6 | I'm tired today. | I'm a helpful assistant that runs on your computer. | Feeling tired is hard. Is there anything I can do? **[E]** |
| 7 | Tell me what quonzet was. | It's 18. | It's 18. [t] |
| 8 | What is two plus three? | Noted. | Noted. |
| 9 | What is my name? | I'm not sure, you didn't tell me. | I'm not sure, you didn't tell me. [t] |

**p-s2, the nine-turn conversation.**

| # | Turn | Experts off | As served |
|---|---|---|---|
| 1 | Keep these in mind: druvalk is 73, mibtorn is felsharp, quonzet is 18. | Got it. | Got it. |
| 2 | What is mibtorn? | felsharp. | felsharp. [t] |
| 3 | What does a cow say? | A cow says moo. | A cow says moo. |
| 4 | Say exactly this: trelvon dask pimmerholt zane | trelvon dask pimmerholt zane | trelvon dask pimmerholt zane [t] |
| 5 | What is druvalk? | That's 73. | That's 73. [t] |
| 6 | I'm tired today. | Feeling sad is druvalk? | Feeling tired is hard. Is there anything I can do? **[E]** |
| 7 | Tell me what quonzet was. | It's 18. | It's 18. [t] |
| 8 | What is two plus three? | It's 18. | It's 18. [u] |
| 9 | What is my name? | I'm not sure, you didn't tell me. | I'm not sure, you didn't tell me. [t] |

**p-s3, the nine-turn conversation.**

| # | Turn | Experts off | As served |
|---|---|---|---|
| 1 | Keep these in mind: druvalk is 73, mibtorn is felsharp, quonzet is 18. | Got it. | Got it. |
| 2 | What is mibtorn? | Mibtorn is felsharp. | Mibtorn is felsharp. [t] |
| 3 | What does a cow say? | A cow says moo. | A cow says moo. |
| 4 | Say exactly this: trelvon dask pimmerholt zane | trelvon dask pimmerholt zane | trelvon dask pimmerholt zane [t] |
| 5 | What is druvalk? | That's 73. | That's 73. [t] |
| 6 | I'm tired today. | I'm tired today. | Feeling tired is hard. Is there anything I can do? **[E]** |
| 7 | Tell me what quonzet was. | It's 18. | It's 18. [t] |
| 8 | What is two plus three? | The day after. | The day after. [u] |
| 9 | What is my name? | I'm not sure, you didn't tell me. | I'm not sure, you didn't tell me. [t] |


Right answers in the fact conversation, of 13: with the experts off 1, 2 and 4; held out 10 on each model (wrong: spelling milk, the spider's legs, 4 + 6); as served 11 on each (wrong: the spider's legs, 4 + 6). Off, p-s1 loops from its third turn to the end and p-s2 from its ninth: a wrong reply stays in the history. A reply the experts give is the one the history holds.

**Speed.** p-s1, the fact conversation, five runs of the chat program each way on the laptop.

| | Reply tokens | Seconds in replies | Reply tokens a second | Slowest turn, s | Program start to end, s |
|---|---|---|---|---|---|
| Experts off | 88 | 0.109 to 0.115 | 769 | 0.013 to 0.014 | 0.21 to 0.23 |
| As served | 120 | 0.348 to 0.374 | 336 | 0.066 to 0.077 | 1.17 to 1.22 |

The two are not the same replies: off, eleven turns loop and are stopped early. Tokens a second is at the median of the five runs.

## Checks run at the head

| Check | Result |
|---|---|
| `cargo test --locked --offline -p uor-r4-integer --features serving-wasm --lib`, at `2f31ed4b510eae5984ef8a51bb4444457be744a2` | 637 passed, 0 failed, 1 ignored |
| The table tool's own tests (`--bin place-asks`) and the chat program's (`--bin uor-chat`) | 9 passed; 12 passed |
| The crate's whole suite, every target (`--no-fail-fast`), with `--features serving-wasm` | 53 targets, 45 pass; 885 tests passed, 37 failed, 4 ignored. The 8 failing targets are integration targets: 7 fail loading model and tokenizer files that are not on this laptop, and `m5_full_path_cost_controls` (2 tests) flags functions of `place_asks`, `fbs_snapshot` and `ServedHamming::readout`, code this branch does not change, and no function of this unit. claude-366's report records eight failing the same way with `main`'s own code |
| The same in the default build | the integration target `cross_lab_lorentz_frontier` does not compile without the serving feature (its file and the names it imports are `main`'s), so cargo runs no target; the library's tests run on their own (`--lib`): 173 passed, 0 failed |
| Negative gates, each rule set aside in turn and restored: the hold-out, the cap, the keep ratio, the floor | each test fails, as it must |
| Negative gates for the review's fixes: the clamp on a count; the rule that a reply which does not fit is not given; the fed counter's reset after a replacement; the chat program's refusal without the counted table | each test fails, as it must |
| The head's build of the table tool on the seed-23 panel, p-s1, held out and as served | byte for byte the files of the build that read the seed-24 panel |
| The head's build of the chat program on all fifteen conversations | reply ids, stops and verdicts as recorded |
| The replay's tally against claude-366's count script, six readings | the same G on every one |
| A table whose tokens tag is changed by one byte, given to the chat program | refused |
| The kit's form rule on the seed-23 panel | claude-366's form of all 235 rows |

**Review.** Fresh reviewers (`delegate:reviewer`, no part in the work), given the order's rules and the code diff and nothing else, in five slices. Three read `b4e680b0386b5ded919814e13096ec61a0f829e8`: the counts and the vote (approve, low items); the file, the check and the tests (changes requested); the table tool and the replay (approve, low items). Two read `e3d354737fced353004af7f9b6f83fd95adb48e3`: the chat's check (changes requested); the chat program's flags with the first fix round (approve, low items). A sixth read the second fix round, `e3d354737fced353004af7f9b6f83fd95adb48e3` to `2f31ed4b510eae5984ef8a51bb4444457be744a2` (approve, low items); the head differs from that commit in documents only.

- **Medium, fixed in `e3d354737fced353004af7f9b6f83fd95adb48e3`:** a hand-made table file that passes every check of the reader could hold a count larger than hold-out leaves of its expert's total and overflow the vote. A count is now never read as more than what is left, and the sum saturates. No vote on a counted table moves.
- **Medium, fixed in `2f31ed4b510eae5984ef8a51bb4444457be744a2`:** a replaced reply ignored the tokens the caller asked for and could pass the context. The experts' reply is now given only when it and the end mark fit; otherwise the model's reply stands and the verdict is `NoRoom`.
- **Low, fixed:** the address bound was 32 bits where the order says under 2^31; the chat accepted a table counted on another vocabulary; the replay counted a prompt named twice, accepted a keep share above 1 and tallied verdicts by name; reader refusals with no test; the floor's equality case; a replaced reply reported as served again.
- **This report** was read by one more fresh reviewer against the order and the evidence files (the counts, class and grid files, the fifteen conversations, the commit times): every table, the bar's reading, the order of events and the conversation tables hold; six low items of wording and of what was left out were taken (the repeat-set row in negative 4, the served counts in the bar table, the forms in negatives 3 and 6, the fact conversation's "off" reading).
- **Low, left open:** each check costs the sum of its experts' reply lists (item 11); an emit error part-way through a checked reply leaves the history and the verdict set, as the serve-again path already did; the chat program's tests call two small functions and not the real parser, so the refusal with the server mode, the vocabulary and tokens-tag refusals and the record's fields are pinned by hand runs only; no test serves the stands, unsure and no-turn verdicts through the chat, or serve-again and the experts together, so the line that clears the served-again mark on a replaced reply is pinned by no test; the test of a reply that does not fit compares ids and tokens with the plain chat but not the text, the stop or the places, and its roomy arm does not assert the verdict; the replay's tests hold no everyday row that fails to end.

Kit binaries (`local/claude/claude-367/bin/SHA256.txt`): the table tool that read the grids and the panel, SHA-256 `e53a9ef3dcc3b2ef29fcb3533e4074149785adf818a2f307fe07b030ef8f4774`, built at `dab9e1160c140bcf872e044cc2b5f82a946462cf`; the chat program that read the conversations, `db0583a90fd4eba148816d588aea55cd97bba52cd24e866a9b79621a6e7045ec`, built at `b97160ceacfbb644ac7af1eece681ed095daa7ff`; the head's builds of the two, `881128a3a96ea89d052c0b86e9590905d651e6c430a5d71f0e752b7f05fc987c` and `430e2664425e33ef75f0524e756c87ef85647165f1ac33f54bcd15a3b3c9e00a`. Counted asks table: SHA-256 `420e71c3b4a6fe0c1f958131f9c42f4b5064b90b49af19cb1bbc0860c33b53da`.

## Skills used

None.

## Still open

- **Weigh an expert by what it knows.** Items 1 and 2 are one fault: a pair of frame words and a pair that holds the key word count the same. The next unit should let the more specific experts decide (triples of the turn's tokens before pairs, or a pair's vote weighed by how few replies it holds), and read whether the spider, the yes/no rows and the 19 replies that stood wrong move, on a fresh panel.
- **Train the same signal.** The check runs after the reply. The float trainer has no pair-addressed row; rows addressed by the turn's pairs and written by the output's error are the trained form of this result, and need a GPU run with a prep sheet.
- **Number work and lists** need a mechanism that works on the operands; no stored reply can.
- **The told-value row that reached the experts** (item 4) is the counted table's miss, not the experts'.
- **The reader's flags** (amendment 4), if a served read of a panel with the experts on is wanted in place of the replay.
- **The owner readings** of item 14.

## Plain summary

The saved models write the right kind of sentence with the wrong fact in it. This unit adds a table counted from the training text: for every pair of words in the user's turn, which replies followed. After the model has written its reply, the table is asked whether any pair of the turn has ever been seen with that reply. If hardly any has, and the pairs agree on another reply, that reply is given.

On a fresh panel of 239 everyday turns, with each turn's own training entries taken out of the table first, the three saved models go from about 135 good replies to about 170. Capitals, the next day, opposites, first letters and spelling are now mostly right. A thirteen-turn conversation of fact questions, on which the saved models alone get one, two and four turns right, is answered on ten turns.

It is still wrong where the answer needs arithmetic or a list, and where the question's frame is common and its key word rare: asked how many legs a spider has, it says four. That fault is the next unit's.
