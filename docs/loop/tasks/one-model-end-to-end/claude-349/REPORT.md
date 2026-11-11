# REPORT: one-model-end-to-end, claude-349

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3926.

Stage 1 of the plan for one arm (turn admission): counts on the laptop that decide whether the arm is built. No pod, no training, no model was run, and no bar or status changed. Every claim below is [claude]'s.

| Row of the ORDER's table | Reading | Applies |
|---|---|---|
| 1. Labels on under 0.80 of recall replies | 490 of 562, 0.872 | no |
| 2. Labelled turn holds the value on under 0.90 | 488 of 490, 0.996 | no |
| 3. Equal bags on over 0.02 | 0 of 488 | no |
| 4. Copy inside the right turn under 0.60 for every reply form | replies that repeat the statement's wording at the value: 135 of 241, 0.560; replies that do not: 0 of 247 | **yes** |

**Row 4 decides: the arm is not built as written.** Picking the turn is not the only missing step: the copy fails even with the right turn admitted. The repeating form misses the line by 10 replies; the other half of the replies cannot be copied by this read at all. Cut finer, by the token before the value, three small groups of repeating replies pass (13, 8 and 1 replies; section 2).

## What did not work, first

- **The copy read cannot take the value out of the right turn.** With the labelled turn admitted, the latest exact match on the token before (copy key 1, arm r's rule) names every token of the value on 135 of 488 recall replies (0.277). With the true turn admitted (the latest user turn that states the value) it is 140 of 562 (0.249). The ORDER's line was 0.60.
- **Reason 1: the reply is worded unlike the statement, 247 of 488.** The token before the value in the reply is not the token before it in the turn, so there is nothing to match: "That's 836.", "You're …", "You love green." against "I like green best of all the colors." The copy read gets 0 of these 247.
- **Reason 2: of the 241 replies that do repeat the statement's wording, 106 still fail.** The ones read are lists: "Note these down: woutdai is 93, zundtetdroul is 32, feltam is 70, …" answered "Woutdai is 93.". The read asks for what follows " is", and the latest " is" in the turn belongs to the last item. Of the 562 values, 355 sit in a turn of three or more clauses. How many of the 106 are lists was not counted.
- **The first form of the census failed its review.** A fresh reviewer returned NOT FIT: the pairing accepted a system line that was not a memory line, the mask check covered only turn content, and the value search could stop inside a longer word. All three are fixed with tests. The rerun moved four cells of one table by one reply each (section 4) and nothing else.
- **The registered stop list was the wrong size.** Numbers are written one digit a token, and the digits are among TRAIN's 64 most frequent tokens, so a reply like "That's 19." has no content token left. 72 recall replies got no label; the 20 the census prints all have a number as the value. With the 16 most frequent tokens stopped, all 562 are labelled and 560 labelled turns hold the value.
- **No stop list gives both a full label and a clean one.** General replies that get a label, where none is wanted, of the 1,945 that have an earlier user turn: 587 at 16 (0.302), 227 at 64 (0.117), 28 at 256 (0.014), while the recall replies labelled fall 562, 490, 286. Of all 2,945 general replies the shares are 0.199, 0.077 and 0.010; the ORDER's line for this fault was 0.05.
- **The floor was not counted as registered.** The ORDER's floor uses sign codes of table points; row 4 applied before the row that needs it. A table-free form (the question's words) is reported in its place.
- **Not counted:** the slot key's ceiling; labels on TRAIN (the panel only); the reach in the registered 18-turn script, which the ORDER asked for (TRAIN's and the panel's replies have at most 8 earlier user turns); and where a read that starts at the value should stop.
- **Read as text, and not the worst 20.** Row 4 asks for the 20 worst replies read token by token. The census prints the first 20 copy misses as text (value, question, the turn, the reply); those were read.

## What changed

- `crates/uor-r4-float-train/src/turn_census.rs` (new): the census and its seven tests. It reads conversations into turns, pairs the panel with its lined twin, labels replies, and counts the copy read inside one turn.
- `train-in-space --data-root DIR --turn-census TRAIN PANEL LINED`: the option that runs it. No other mode changes.
- `ORDER.md`: the review's findings (before the counts) and amendment 1 (below).
- One notebook row.

**Amendment 1 to the ORDER, made while counting.**
- The lined panel is `recall=oracle`, not `recall=reply`. The reply-worded line holds the whole reply, so it cannot say what the value is; the oracle line is "Memory: VALUE.".
- A reply form is counted two ways: by the token before the value in the reply, and by whether that token also stands before the value in the turn (the ORDER's "repeats the statement's words").
- Added beside the registered counts: the stop lists 0, 16 and 256; the ceiling with the true turn admitted; the clauses of the turn that holds the value; what the question's own words pick with nothing learned; and general replies that have an earlier user turn.

## Measurements

Inputs, written on the laptop by `m-world-recall-corpus` and equal to the recorded files (token counts and checksums are the writer's receipts): TRAIN `conversations=60000 seed=13 recall=off` (7,370,634 tokens, `e25775f3…`), panel `conversations=1000 seed=14 recall=off` (121,894 tokens, `b8ad9fe4…`), panel `recall=oracle` (131,619 tokens, `12d7e9df…`). The panel has 3,586 replies: 2,945 general, 79 abstentions, 562 that state a value. Every share is of panel replies.

**1. Labels.** The label is the earlier user turn that holds the most of the reply's content tokens (ties to the later turn). 1,945 of the 2,945 general replies have an earlier user turn; the other 1,000 are first replies and cannot get a label.

| Most frequent tokens stopped | Recall replies labelled, of 562 | Labelled turn holds the value | General replies labelled, of 1,945 | Abstentions labelled, of 79 |
|---|---|---|---|---|
| 0 | 562 | 560 | 868 | 39 |
| 16 | 562 | 560 | 587 | 21 |
| 64 (registered) | 490 | 488 | 227 | 5 |
| 256 | 286 | 284 | 28 | 0 |

All 562 values are stated in an earlier user turn. At 64 the label is the latest such turn on 488. Had the model's own turns been candidates, one would have won on 130 of 562; the turn being answered on 59.

**2. Ceiling.** Recall replies whose every value token the copy read names, with one turn admitted. "No turn admitted" is key 1 over everything from the conversation's start to the token being read.

| Turn admitted | Replies | Key 1, whole value | Key 1, first token | Key 1, tokens after the first | Key 2, whole value | No turn admitted (today), whole value |
|---|---|---|---|---|---|---|
| The labelled turn (it holds the value) | 488 | 135 (0.277) | 161 | 379 | 185 (0.379) | 59 |
| The latest turn that states the value | 562 | 140 (0.249) | 169 | 411 | 185 (0.329) | 59 |

By reply form, labelled turn admitted, key 1. "Repeats" means the token before the value is the same in the reply and in the turn.

| Token before the value in the reply | Replies | Whole value | Share | Repeats | Whole value, of those | Share |
|---|---|---|---|---|---|---|
| " is" | 274 | 114 | 0.416 | 219 | 114 | 0.521 |
| "'s" (That's, It's) | 141 | 0 | 0 | 0 | 0 | |
| " in" | 29 | 13 | 0.448 | 13 | 13 | 1.000 |
| " named" | 16 | 7 | 0.438 | 8 | 7 | 0.875 |
| "'re" | 14 | 0 | 0 | 0 | 0 | |
| " love" | 11 | 0 | 0 | 0 | 0 | |
| " a" | 3 | 1 | 0.333 | 1 | 1 | 1.000 |
| All | 488 | 135 | 0.277 | 241 | 135 | 0.560 |

The ORDER names two forms, "It's VALUE." and a reply that repeats the statement's words. Read that way, the repeating form is at 0.560 and the other at 0 of 247, so row 4 applies. Cut finer, three small forms pass (" in" 13 of 13, " named" 7 of 8, " a" 1 of 1: 22 replies, 0.045 of the 488); the large repeating form, " is", is at 0.521.

**3. Where the value sits.** In the turn that states it, the value's tokens stand together, as whole words, on 562 of 562. That turn has one clause on 144, two on 63, three on 73, four or more on 282.

**4. What the question's words pick, nothing learned.** The pick is the latest earlier user turn, or clause, that shares the most content tokens with the question. The four marked cells were one lower before the review's word-edge fix.

| Most frequent tokens stopped | The right turn is the pick, of 562 | The value's clause shares a question token | The value's clause is the pick |
|---|---|---|---|
| 0 | 491 (0.874) | 333 | 311* (0.553) |
| 16 | 486 (0.865) | 314 | 304* (0.541) |
| 64 | 450 (0.801) | 272* | 269* (0.479) |
| 256 | 253 | 148 | 146 |

**5. Holes and reach.** Of 488: an earlier user turn that holds the labelled turn's subject words and not this value on 2 (later than the label on 1); a turn with the same tokens in another order on 0. Earlier user turns a reply has, TRAIN (215,017 replies): none 60,000; one 51,799; two 40,318; three 28,789; four 12,150; five 6,972; six 6,968; seven or more 8,021; most 8. From the labelled turn's first content token (its "User:" mark not counted) to the reply's end fits in 192 tokens on 395 of 490 (0.806) and in 256 on 490.

**What the numbers say, and what they do not.**
- Which turn. With nothing learned, the question's words pick the right turn on 0.80 to 0.87 (stop lists 0 to 64; 0.450 at 256). That is the figure that matters at serving time. The 0.996 is from the reply's own words, which a served model does not have before it answers; it says only that the training label is sound when there is one.
- Which clause. The question's words pick the value's clause on 0.48 to 0.55 (stop lists 0 to 64; 0.260 at 256). With no token stopped, the clause shares no token with the question on 229 of 562, and shares one without being the pick on 22. Shared words cannot find the 229 ("Where did I grow up?" against "I am from Jaistchukfield."); the counts do not say what can.
- The value itself is one unbroken run of tokens in its turn, every time. A read that starts at its first token and takes the tokens that follow passes through the whole value. Where such a read stops was not counted; in a list the next tokens are ", zundtetdroul is 32".
- **A proposal, not a result:** pick the value's place, not its turn, with the question, and read the value from the pick onward, first token included. It is aimed at both reasons above: arm r's read re-finds each token by the reply's previous token, which fails in a list and fails when the reply is worded its own way. Nothing counted here picks a place inside a turn on more than 0.55.

## Checks run at the head

- `cargo test --release --locked -p uor-r4-float-train --lib turn_census`: 7 passed. The tests pin the turn split and its mask check (content, role marker, separator, start mark), the pairing and its four refusals, a line that is not a memory line refused, the tie to the later turn, the copy rule on "it's maya" against "is maya", a value inside a longer word passed over, the clause cut, and one census end to end.
- `cargo clippy` on the crate's library and `train-in-space`: nothing in the new file. `rustfmt --check` on the new file: clean.
- The three corpora equal the recorded token files by SHA-256 (listed in [the sheet of 10 Nov](../../../../harness/prep/one-model-end-to-end-memory-line-2026-11-10.md)).
- The census agrees with the writer's own receipts: 3,586 replies on the panel and 215,017 in TRAIN (the writer's turn counts), 562 values and 79 "none" lines.
- The census refuses a panel pair that differs beyond its memory lines, a system line that is not a memory line or not directly before a reply, and a response mask that is not 1 on exactly the replies; none fired on these files.
- Inside the count: the value's tokens were found in the reply on 488 of 488 and 562 of 562, and in the stating turn on 562 of 562. The form table's rows add to 488, 135 and 241.
- The count output is `local/claude/claude-349/census.out` on the laptop, SHA-256 `a0856506ba182c15…`; every number above was checked against it.
- `uor-harness ratchet d23`, `d25`, `d32`: no additions.

## Skills used

None beyond the project loop. The fresh-context reviewers were subagents of this session: one read the plan before the counts; two read the census and this page at `3109b61080e7a6f31b9a2a9f1f6ed71d2d283e01` (one NOT FIT on the instrument's guards, one FIT with findings on this page; all taken here); one read the fixes at `ab60b78da1802ccb4bed15946ce9954ffc927b62` (FIT, with four wording points on this page and one test assertion, all taken in the last commit). The review record is on the PR.

## Still open

- **The changed design needs the owner's word.** The owner approved one arm, a learned selector for which fact answers. The counts say a selector of turns is not enough. The proposal above points at a place and reads from there; it is not designed, not counted and not built. Parts that exist: the ask heads' query from the question, the Hamming keys, the found-copy pull.
- The questions that share no word with the value's clause (229 of 562) are what learning would have to add. No laptop count sizes that.
- The floor with table sign codes, the slot key's ceiling and the 18-turn script's reach are not counted.
- A label for training needs a stop list near 16 and a way to keep general replies out; the leak at 16 is 0.302 of the general replies that could get one.
- With labels as they are, a 192-token window holds 0.806 of labelled turns; 256 holds all.

## Plain summary

The plan was to let the model pick the earlier turn that holds the answer and copy from it. With nothing learned, the question's own words find the right turn four times in five. The copy fails even when the right turn is handed to it: half the replies are worded differently from what the user said, and many of the rest have the answer in a list. The copy read gets the value on about a quarter of the replies. The value is always one run of tokens, so the proposed fix is to point at where it starts and read on. That is a changed design; it is not counted and not started.
