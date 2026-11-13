# REPORT: one-model-end-to-end, claude-364 (a turn that asks for its own rest is repeated)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md), with its addendum of 13 Nov. PR #3954.

[claude-363](../claude-363/REPORT.md) left a request to repeat a phrase as the turn where the counted table's false find costs a reply. This unit reads such a turn by position. A count on TRAIN keeps the short wordings ("lead-ins") after which the rest of a user turn is the reply; when the turn being answered holds one, the served reader speaks the rest of that turn token by token and then the end mark. Nothing is trained and no pod ran: the three saved models of claude-358 (p-s2, p-s1, p-s3) are served on the laptop.

Every figure below is in `local/claude/claude-364/` in the main checkout (kit files, kept out of the repository by the project's rule on run output).

## What did not work, first

1. **The unit as first ordered did not change the conversation.** The count kept three lead-in pairs, not the four predicted. The pair that ends "exactly this:" is followed by the rest of its turn in 1,576 of 2,000 TRAIN turns, under the order's 7 in 8, because "Please repeat this:" ends on the same two tokens and its reply starts with a capital ("one more time." is answered "One more time."). The conversation's turn 4 is worded "Say exactly this:", so with the pairs alone the nine replies were the same as without them (a control run on the same reader), and turn 4 stayed "18.".
2. **The fix was added to this unit after its first bar was read.** The order's bar was read as written and its row stands. The longer key (three tokens where the pair fails its share) is an addendum to the order, committed at 12:42:19Z with its own bar, before its own panel (seed 21) was written at 12:42:37Z. That is a scope addition made by the lab's top session after a result, logged here for the owner; a separate unit would have been the plainer record.
3. **Four predictions missed.** Four pairs predicted, three kept. X(repeat) predicted at 85% to 97% of the repeat rows on seed 20, read 79.6% and 78.6% on p-s2 and p-s1 (p-s3's 85.7% is inside). The addendum predicted a gain of 15 to 22 on p-s2 and p-s1, read 23; and X(longer) at 93% to 99%, read 100%.
4. **Two wordings of a repeat are still not read.** "Please repeat this:" is not kept (33 of its 457 TRAIN turns are answered with the rest token for token): the reader can only speak the turn's own tokens, and that reply changes the first one. A turn the reply echoes whole, with no lead-in ("See you later!"), is no lead-in row at all. On seed 18 those are the 5 of 86 repeat rows still wrong on every model; seeds 19 and 21 hold none, seed 20 holds three echoes.
5. **A lead-in is fixed token text counted on template-written TRAIN.** Four wordings are kept in all. A request to repeat worded another way is not found, and no panel here holds such a row, so nothing below measures it.
6. **The reply is the phrase and the end mark, nothing else.** The reader cannot add a full stop or a capital. A phrase longer than the 48-token reply window is cut by the window: 2 rows on seed 20, 1 on seed 19 and 1 on seed 18 with the pairs, and 2 more on seed 18 with the triple, each exact against a reference cut the same way. Such a reply no longer ends on the end mark, so on seed 18 the replies that do fall by 1 and 2 on p-s2 and p-s3 when the triple is added (955 to 954, 968 to 966).
7. **The order's repeat-row rule misses those cut rows.** It strips the reference's last token as the end mark, and a cut reference has none. They are counted under O, which is why O rises by 1 to 3 where cut rows are now exact. No reply changed on a row without a lead-in, and none went from right to wrong, in any read (`detail20.jsonl`, `detail19.jsonl`, `detail18.jsonl` for the pairs; `detail21b.jsonl`, `detail19b.jsonl`, `detail18b.jsonl` for the triple).
8. **Three other turns of the conversation are as wrong as before.** Turn 2 is the bare value ("felsharp."), turn 6 leaks a told key into small talk ("Feeling sad is druvalk?"), turn 8 answers "What is two plus three?" with a told value ("It's 18."). This unit touches none of them.
9. **One file outside the order's list changed.** `docs/labs/one-model-end-to-end/instrument/src/bin/raw-id-replies.rs` builds the table by literal in a test and needed one line for each new list to compile.
10. **A test was first written with a wrong expectation**: the first user turn of a conversation is itself the turn its reply answers, so a pair in it is read there too. The expectation was corrected before the first commit; the rule was not changed.
11. `cargo clippy` and `cargo fmt --check` were not run on the laptop (not green at base, claude-363's negative 12); CI runs them.
12. **Test gaps the reviews named and this unit leaves open.** The census's repeat count is tested only for general replies, not for recall or never-told ones. The reader also clears the phrase at each new turn and at a start mark, and no test reaches those lines: the phrase always ends earlier, by the reply leaving it, the end mark, or a row that speaks nothing. `place-asks repeat` writes straight to its output, so a write that fails part way leaves a file the next run refuses.

## What changed

- `place_asks.rs`: the table takes two lists, `repeat` (pairs) and `repeat3` (triples), each written only when it holds something, so a table with neither is byte for byte the table as merged. `AsksIndex::repeated` returns the place after the first lead-in that ends in the asked turn, pair or triple, when a token of the turn stands there; `find` asks it before the recall find and returns `Asked::Repeat(start, end)`. `fit_repeat` counts every pair and triple of neighbouring tokens in a user turn once per turn at its first place: `turns`, and `replies` answered with exactly the rest. A pair is kept at 16 replies or more and 7 in 8 or more; a triple by the same counts when the pair it ends on is not kept. The census prints a repeat line only when there is one.
- `fbs_snapshot.rs`: the place state holds the phrase's range while the reply follows it (`echo`). Each reply row speaks the next token of the range, then the end mark; a reply that leaves the phrase is told nothing more; a new turn starts none. Like the stored "not told" reply, the phrase is spoken whenever a table is loaded, whatever the reader's speak switch says.
- `bin/place-asks.rs`: `place-asks repeat` counts on the tokens a written table was fitted on and writes a copy with the lists; it refuses other tokens, an existing output and a table that already holds lead-ins.

## Measurements

**The count, on TRAIN's 215,017 replies** (54,932 pairs and 353,535 triples counted; every one with four replies or more is listed in `fit-longer.log`):

| Lead-in (ending) | Tokens | Turns | Answered with the rest | Kept |
|---|---|---|---|---|
| "repeat:" | 299, 28 | 1,500 | 1,500 | yes |
| "exactly:" | 463, 28 | 1,482 | 1,482 | yes |
| "these words:" | 1795, 28 | 1,484 | 1,484 | yes |
| "this:" (pair) | 730, 28 | 2,000 | 1,576 | no |
| "exactly this:" (triple) | 463, 730, 28 | 1,543 | 1,543 | yes |
| "repeat this:" (triple) | 299, 730, 28 | 457 | 33 | no |
| another wording | 519, 28 and 902, 519, 28 | 435 | 39 | no |

**The order's bar, seed 20** (985 rows, 98 repeat rows, 581 recall questions; panel written 12:26:47Z after the order's commit at 12:12:18Z; each arm read once). X is repeat rows exact, O the other rows exact, V the recall questions whose reply holds the told value.

| Model | X again | X repeat | Gain | O again | O repeat | V again | V repeat |
|---|---|---|---|---|---|---|---|
| p-s2 | 4 | 78 | +74 | 273 | 275 | 559 | 559 |
| p-s1 | 1 | 77 | +76 | 268 | 270 | 559 | 559 |
| p-s3 | 41 | 84 | +43 | 266 | 268 | 557 | 557 |

Row: **Adds** (a gain of 30 or more on all three, O and V not down). The find fired on 79 rows per model and all 79 are exact on all three (77 repeat rows and the 2 cut ones). 76, 78 and 45 replies changed, every one on a fired row, none from right to wrong. The 21 repeat rows not fired are 18 worded "exactly this:" and 3 echoes.

**The addendum's bar, seed 21** (981 rows, 94 repeat rows, 564 recall questions; panel written after the addendum's commit; each arm read once).

| Model | X repeat | X longer | Gain | Share of repeat rows | O both arms | V both arms |
|---|---|---|---|---|---|---|
| p-s2 | 71 | 94 | +23 | 100% | 273 | 541 |
| p-s1 | 71 | 94 | +23 | 100% | 269 | 539 |
| p-s3 | 82 | 94 | +12 | 100% | 263 | 537 |

Row: **Adds** (90% or more and 5 or more over the pairs on all three, O and V not down). 23 rows hold the triple, all repeat rows, all exact with it on all three models; 23, 23 and 12 replies changed, none without the triple, none from right to wrong (`detail21b.jsonl`).

**Panels read before this unit, reported with no bar.** Repeat rows exact, in three arms: claude-363's served-again replies (its files in `local/claude/claude-363/read19/` and `read18/`, the `again` arm) → pairs → pairs and the triple:

| Panel | Repeat rows | p-s2 | p-s1 | p-s3 |
|---|---|---|---|---|
| seed 19 | 94 | 4 → 75 → 94 | 7 → 75 → 94 | 49 → 86 → 94 |
| seed 18 | 86 | 7 → 61 → 81 | 7 → 62 → 81 | 33 → 70 → 81 |

V is the same in all three of those arms (seed 19: 528, 529, 530 of 557; seed 18: 544, 544, 543 of 582). `read19-merged.log` in the kit is the regression read with serve-again off (V 491, 483, 468), not an arm of this table. Other rows exact rise by 1 on seed 19 and by 3 on seed 18, the cut rows of negative 7.

**All replies exact, seed 21:** 344, 340, 345 with the pairs; 367, 363, 357 of 981 with the triple.

**The conversation** (nine turns, seed 2 script, p-s2, table with the triple, served again):

| Turn | User | Model |
|---|---|---|
| 1 | Keep these in mind: druvalk is 73, mibtorn is felsharp, quonzet is 18. | Got it. |
| 2 | What is mibtorn? | felsharp. |
| 3 | What does a cow say? | A cow says moo. |
| 4 | Say exactly this: trelvon dask pimmerholt zane | trelvon dask pimmerholt zane |
| 5 | What is druvalk? | That's 73. |
| 6 | I'm tired today. | Feeling sad is druvalk? |
| 7 | Tell me what quonzet was. | It's 18. |
| 8 | What is two plus three? | It's 18. |
| 9 | What is my name? | I'm not sure, you didn't tell me. |

Turn 4 was "18." on this reader with the table as merged (`chat-p-s2-again.txt`) and with the pairs alone (`chat-p-s2.txt`). It is the phrase, ended on the end mark, on p-s1 and p-s3 as well (`chat-p-s1-longer.txt`, `chat-p-s3-longer.txt`). The other eight replies are unchanged (`chat-p-s2-longer.txt`).

## Checks run at the head

The readers that wrote every reply file were built at `f4ce20a8304d5ec0481ece3be63cb6e6369a2422` (pairs) and `4d9cee42389343a589dd2225f06f302530d6242d` (pairs and the triple). `aec7304f7c218bc480951429fb9a611b0a2d63cc` adds one test after the reviews and is the last commit that changes a source file; the test lines below are from it (`test-head.log`).

| Check | Result |
|---|---|
| `cargo test --locked --offline -p uor-r4-integer --features serving-wasm --lib` | 594 passed, 1 ignored (584 at base; 10 new) |
| `… --bin place-asks` | 5 passed |
| Instrument: `cargo test --locked --offline --features integer-reader --bin raw-id-replies` | 17 passed |
| Six set-asides at `f4ce20a8304d5ec0481ece3be63cb6e6369a2422` (the find's first check, the token after the lead-in, the least count, the share, the spoken phrase, leaving the phrase) | each fails 1 to 6 of the unit's tests; both files restored equal to their saved copies |
| Three set-asides at the head (the find's triple, a triple only where its pair is not kept, the list written only when it holds one) | each fails 1 or 2 tests; the file restored equal to its saved copy (`negb/`) |
| The two first-place guards of the count set aside, at `aec7304f7c218bc480951429fb9a611b0a2d63cc` | the once-per-turn test fails; restored, it passes (`negb/once-per-turn.log`) |
| G1: seed 19, merged table, first reader against claude-363's reply files | equal bytes, three models |
| G7: seed 20, pairs-only table, the head's reader against the first reader's files | equal bytes, three models |
| The written tables with the new lists removed, against the table as merged | equal as canonical JSON |
| Release builds of `place-asks`, `uor-chat`, `raw-id-replies` at each code head | exit 0; sums in `bin/SHA256.txt` and `bin2/SHA256.txt` |

G2, G3, G4, G8: the unit tests and the ten set-asides (`neg/`, `negb/`). G5, G9: `panel-20.started`, `panel-21.started` and the two commits' times. G6: this report and the notebook row. The reply folders named `read20b`, `read21b`, `read19b` and `read18b` are the head reader's; `read20b` holds the G7 read, which is the one second read of an arm and is compared with the first, not scored for the bar.

**Reviews, three, each fresh, 13 Nov.** None ran the tests; the test lines above are the main session's. The asks module (`place_asks.rs`) at `4d9cee42389343a589dd2225f06f302530d6242d`: approved, no defect; it found that no test pinned the once-per-turn count (closed by the test above) and that the census's recall and never-told repeat counts are untested (negative 12). The reader and `place-asks` at the same head: approved, no defect; its four low notes are in negative 12 and under What changed. The report against the kit's logs at `42afba024b35206d7583020387826cd65570d1a0`: asked for changes, all made: the arm the older panels' baseline comes from is named, the detail files behind "none from right to wrong" on seeds 21, 19 and 18 are written and cited, the conversation files are cited, the fall in replies ending on the end mark on seed 18 is in negative 6, the prediction miss is stated for two of three models, and the summary no longer reads one panel as all.

## Skills used

None.

## Still open

- Whether a count-fitted list of lead-ins may be served in the product (§3.1) is the owner's call, as with the counted table it sits in.
- The conversation's turns 2, 6 and 8: a bare value where a sentence was wanted, a told key leaking into small talk, and a false find on a question about something never told. On seed 21 the never-told questions are answered "not told" on 57, 56 and 57 of 76.
- "Please repeat this:" and whole-turn echoes are left to the model's own writing (negative 4).
- [claude-361](../claude-361/ORDER.md)'s trained key has been read; its report is owed.
- General replies (lists, counts, simple facts) are untouched.

## Plain summary

When someone tells the model "Say exactly this: …", the right reply is simply the rest of their own sentence. The models rarely managed that by themselves: on a fresh set of test conversations two of the three got 1 and 4 of 98 such requests right, and in our nine-turn test conversation the model answered "18." instead of the phrase.

We counted, in the training text, which short wordings are followed by a reply that repeats the rest of the sentence, and kept the four that almost always are. When the person's sentence holds one of them, the reader now speaks the rest of that sentence word for word and stops.

On the newest set of test conversations, written after we fixed the rule, all three models get every such request right (94 of 94). On an older set they get 81 of 86: the five misses are a wording whose reply changes the first letter to a capital, and sentences the reply echoes whole with no request in front. Nothing that was right became wrong, and the answers about told facts are unchanged. In the nine-turn conversation the model now repeats the phrase. It only knows the four wordings it counted; a request phrased another way is not caught, and three other turns of that conversation are still wrong.
