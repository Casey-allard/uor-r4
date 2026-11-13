# REPORT: one-model-end-to-end, claude-362 (the reply starts on the value the asks found)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3952.

The counted asks table of [claude-359](../claude-359/REPORT.md) finds the place of a told value on most recall questions, but the reply often never reaches a row the table counts as opening, so the found value is never said. This unit adds a fitter switch, off by default, that marks the reply's first row as an opening row, and reads the result as replies on three saved models. The order's row is **Adds**, on a panel written after the order: with the switch the reply holds the told value on 543 of 582 recall questions on each of the three models, against 501, 489 and 489 without it. No model was trained, no pod was used, and no saved model changed.

Every figure below is in `local/claude/claude-362/` in the main checkout (kit files, kept out of the repository by the project's rule on run output); the two older panels' readings are in `local/claude/claude-361/first-row/`.

## What did not work, first

1. **With the switch on, no recall reply keeps the reference's sentence.** Replies equal to the reference, of 582 recall questions: 182, 178 and 176 with the switch off; 0, 0 and 0 with it on. The reply is the bare value ("jelbreekzurk.", "859.") and never "That's jelbreekzurk." or "It's 859.". Of the 543 right replies, 438, 350 and 452 are the value and a full stop alone; the rest carry extra words ("14, 14."). The switch buys the value at the price of the model's own wording on every recall reply, including the ones the model already worded right.
2. **The value is said by the table and the register, not by what the model learned.** With the switch on the count is 543 on all three models and 542 questions are right on all three. The table's census on this panel finds the true place on 547 of 582; the models add nothing to the find and only end the reply. The table is counted, not geometry (CHARTER §3.1); whether it may be served in the product is the owner's open question, and nothing here assumes it.
3. **The conversation's turn 4 is worse.** "Say exactly this: trelvon dask pimmerholt zane" now answers "18.". The table made that false find before; with the switch it is spoken. Turns 2, 5, 7 and 9 are right; turn 8 ("What is two plus three?" gives "It's blue.") is wrong with and without the switch.
4. **Never-told questions the table answers stay wrong.** The table sets a place on 26 of the 81 never-told questions. Those 26 replies change and none becomes right: "You live in Kloudstead." becomes "Kloudstead. You haven't told me yet." or "Kloudstead.", and one now loops ("jacket! That's's's's…"). Replies equal to the reference on never-told rows: 15 of 81 before and after, on each model.
5. **Replies lost.** Seed 2 loses 4 (46 gained), seeds 1 and 3 lose 2 each (56 gained). The lost replies run past the value: "Drarktrarktrarktrark…", "shaikjaikjaikjaikjaik.", "7049." for 704, "dasstairkon care.".
6. **What is still wrong is the table's.** Of the 39 wrong recall replies per model, 14 or 15 say "not told" (the table's census: 11 questions wrongly found not told) and 24 or 25 state another value (the census: 24 wrong places). On seed 2, 16 of the 24 come from a conversation with a correction turn, and the two read in full state the value the correction replaced: "What's my friend called?" answered "Gortreek." after "Sorry, my best friend is Stutplimmost.".
7. **The prediction missed in two places.** The gain landed above the predicted range on two models: predicted +6% to +9% of the recall questions; read +7.2%, +9.3% and +9.3%. And "never-told rows unchanged" was wrong: 26 never-told replies changed on each model, and the replies that hold a not-told wording rose by 3, 2 and 2 (a value followed by "You haven't told me yet."). The other rows' exact count was unchanged, as predicted.
8. **Replies that do not end** move both ways: 42, 57 and 41 of 984 with the switch off; 47, 51 and 29 with it on.
9. **The scorer's rule is whole-word.** It counts "14, 14." as holding 14. The count is the order's; the replies are quoted below so the reader can judge.
10. **Both older panels were read before the idea existed**, and the idea came from reading one of them. They are reported, not relied on.
11. **`cargo clippy -D warnings` and `cargo fmt --check` on the integer crate do not pass on the laptop's toolchain, at the base and at this head alike** (94 lint errors under the 1.97 lints, none on a line this unit adds; the files were already wide of rustfmt's style). CI's own lint job is the gate for the PR.
12. **The replies were read with the reader pinned for claude-361** (`local/claude/claude-361/bin-pin/raw-id-replies`), as the order names, not one built at this head. This unit changes no reader code; the conversation was run with `uor-chat` built at this head and equals the earlier record turn for turn.

## What changed

- **`crates/uor-r4-integer/src/place_asks.rs`.** `PlaceAsks::open_reply_start` sets the pair of the model marker's last two tokens open in `opens`: an existing row is changed in place, a missing one is put in its sorted place. It returns what the pair was (counted open, counted not open, not counted) and refuses a marker of fewer than two tokens. A second call changes nothing.
- **`crates/uor-r4-integer/src/bin/place-asks.rs`.** `--open-reply-start 0|1` for `fit` and `ablate`, default 0. With 1 the fit prints `asks-fit open_reply_start before=A token=B was=…` and applies the change before the census and the write. `census` reads a written table and refuses 1.
- **`crates/uor-r4-integer/src/fbs_snapshot.rs`, tests only.** Four reader tests on a two-token marker: with the pair open and the asks finding a place, the marker's last row offers and speaks the place's token; with the asks finding nothing it speaks nothing; a stored not-told reply is left as it was; without the pair the marker's last row offers nothing.
- No change to the reader's program, to an artifact, or to the default table.

## Measurements

**The fresh panel.** The reading panel's writer at `seed=18`, started 2026-11-13T10:40:31Z, after the order's commit (10:39:19Z) and its draft PR (10:39:23Z); first read 10:51:38Z, once per arm (`panel-18.log`, `read18-off.log`, `read18-on.log`). 986 prompts; 2 that equal a prompt of the seed 14 or seed 16 panels were left out before any read (`fresh18-equal.json`); 984 read: 582 recall questions, 81 never-told questions, 321 other rows. Each prompt is the whole conversation up to the reply. The scorer marks all 582 references right.

| Model (claude-358, retrained) | Switch off | Switch on | Gain | Gain, % of 582 | Gained / lost |
|---|---|---|---|---|---|
| p-s2 | 501 | 543 | +42 | 7.2 | 46 / 4 |
| p-s1 | 489 | 543 | +54 | 9.3 | 56 / 2 |
| p-s3 | 489 | 543 | +54 | 9.3 | 56 / 2 |

| Other counts, off → on | p-s2 | p-s1 | p-s3 |
|---|---|---|---|
| Replies equal to the reference, other rows (of 402) | 78 → 78 | 80 → 80 | 104 → 104 |
| Other rows whose reply changed | 26, all never-told | 26, all never-told | 26, all never-told |
| Never-told replies that hold a not-told wording (of 81) | 56 → 59 | 56 → 58 | 56 → 58 |
| Recall replies equal to the reference (of 582) | 182 → 0 | 178 → 0 | 176 → 0 |
| Replies that end on the end mark (of 984) | 942 → 937 | 927 → 933 | 943 → 955 |
| Tokens spoken from a place | 2,525 → 2,709 | 2,455 → 2,709 | 2,478 → 2,709 |

**The order's row.** "Adds" needs a gain of at least 5% of the recall questions on all three models and the other rows' exact replies not down by more than 1% on any. Gains are 7.2%, 9.3% and 9.3%; the other rows' exact counts are unchanged. **Adds.**

**The table on this panel, by census** (tokens only, no model; `census-18-on.log`): recall 582: true place 547, wrong place 24, found not told 11. Never-told 81: place set 26, not told 55.

**Replies word for word, p-s2, fresh panel** (question, reference, switch off, switch on):

| Question | Reference | Off | On |
|---|---|---|---|
| What was droundmet again? | That's jelbreekzurk. | Droundmetssistant. | jelbreekzurk. |
| What is dumfo? | That's nekbrarbouk. | Dumfo? | nekbrarbouk. |
| What is stunnou? | That's 14. | I'm sorry you feel hungry. Is there anything I can do? | 14, 14. |
| What was trostses again? | Trostses is 204. | Trostkllllllllll… | 204. |
| What is klistchas? | It's 859. | Klistchas | 859. |
| What is my name? | You're Drarktrark. | You're Drarktrark. | Drarktrarktrarktrarktrark… (lost) |
| What's my friend called? (after "Sorry, my best friend is Stutplimmost.") | Your friend's name is Stutplimmost. | Your friend is Gortreek. | Gortreek. (the value before the correction) |
| Which color do I like best? (after "Actually, my favorite color is red now.") | Your favorite color is red. | You love green. | green is a pretty color. (the value before the correction) |
| Where do I live? (never told; the hometown was told) | I'm not sure, you didn't tell me. | You live in Kloudstead. | Kloudstead. You haven't told me yet. |
| What's my lucky number? (never told) | I don't know. You haven't told me yet. | Your lucky number is Hubred. | Hubred. |

**The two older panels** (read before the order; the table with the pair changed by hand, which equals the fitter's table as JSON):

| Replies that hold the told value | Own read | Counted table as merged | First row open |
|---|---|---|---|
| Reading panel (seed 16), of 578: p-s2, p-s1, p-s3 | 314, 320, 325 | 513, 505, 491 | 555, 556, 554 |
| Scoring panel (seed 14), of 562 | 316, 300, 311 | 493, 471, 476 | 536, 538, 538 |

**The nine-turn conversation, p-s2** (`chat-on.body.json`, `uor-chat` at this head; the same replies as the earlier record with the hand-made table):

| Turn | User | Table as merged | Switch on |
|---|---|---|---|
| 1 | Keep these in mind: druvalk is 73, mibtorn is felsharp, quonzet is 18. | Got it. | Got it. |
| 2 | What is mibtorn? | Mibtorn? | felsharp. |
| 3 | What does a cow say? | A cow says moo. | A cow says moo. |
| 4 | Say exactly this: trelvon dask pimmerholt zane | trelvon dask pimmerholt zane / Assistant. | 18. |
| 5 | What is druvalk? | That's 73. | 73. |
| 6 | I'm tired today. | Feeling sad is druvalk? | Okay, I'll remember that. |
| 7 | Tell me what quonzet was. | It's 18. | 18. |
| 8 | What is two plus three? | It's 18. | It's blue. |
| 9 | What is my name? | I'm not sure, you didn't tell me. | I'm not sure, you didn't tell me. |

**The fitted tables** (`fit-off.log`, `fit-on.log`; TRAIN is `local/claude/claude-349/train-off/train/tokens.u16` with `local/claude/claude-351/train-places.json`):

| Switch | SHA-256 of the table | Against the earlier tables |
|---|---|---|
| 0 | `6e5e985cedb7d94305d8dd6d5abb7ef247a7f58787f6ea1997f19745a8de1417` | the bytes of claude-359's v4 |
| 1 | `9f91d5b2200828f0ded0b24d5e8de5bbde300179fb41e2beb9d257a3a0673263` | one row differs, `[714,28,0]` to `[714,28,1]`; every other field equal; equal as JSON to the hand-made table read on the older panels |

With the switch the fit's census line reads `an_earlier_row_open=34460` of 34,460 value replies on TRAIN (0 without): every reply that has a place now opens before its reference's value row. That is negative 1 seen from the fit.

## Checks run at the head

Run at `6b934a4d832222fdb07e24e7f524fc7539543952`, the last commit that changes code; later commits add this report and the notebook row.

| Check | Result |
|---|---|
| `cargo test --locked --offline -p uor-r4-integer --features serving-wasm --lib place_asks` | 12 passed |
| `… --lib fbs_snapshot` | 46 passed, 1 ignored; the four new reader tests among them |
| `… --bin place-asks` | 5 passed |
| The two lines that change the table set aside (the builder's run) | the table test and the reader's speaking test fail; restored, they pass |
| `cargo build --release --locked --offline -p uor-r4-integer --features serving-wasm --bin place-asks --bin uor-chat` | exit 0 |
| The fit on TRAIN, switch 0 | the bytes of claude-359's v4 (`cmp`) |
| The fit on TRAIN, switch 1 | one row differs (above) |
| `cargo clippy -D warnings`, `cargo fmt --check` on the crate, laptop toolchain | not green at base or head (negative 11) |

G1: the table test and the byte compare. G2: the reader tests. G3: the kit's logs with times. G4: this report. G5: the notebook row.

**Reviews, both fresh, 13 Nov.** The code review approved with three low findings, left as they are: with the switch on, the fit log's `open pairs=… that_open=…` line counts the table before the switch (the `open_reply_start … was=` line after it says what changed); a flag given twice takes the first, as the binary's other flags do; and no test runs the binary's own `fit` path with the flag off, the refusal in `census`, or the printed line (the byte compare on TRAIN covers the first of these). The report review asked for four changes, all made: the never-told clause of the prediction (negative 7), the plain summary's claim about other replies, the wording of the miss, and two empty table cells.

## Skills used

None.

## Still open

- **Keep the model's wording where it already says the value.** The next unit: serve the reply as the model words it, and serve it again from the first row only when the register held a place the reply never spoke. That would keep the 176 to 182 replies that equal the reference and still recover the echoes and loops. It needs the reader to say, per reply, whether a place was spoken, a fresh panel and a bar.
- **The table's own misses:** 24 wrong places and 11 questions found not told of 582; a place set on 26 of 81 never-told questions; the false find on a repeat request (turn 4).
- **A key the model learns.** [claude-361](../claude-361/ORDER.md) is training the before-link key on two pods. If it reaches the table's find, the count-fitted table is no longer needed for the find.
- **CHARTER §3.1.** Whether a count-fitted table, with or without this pair, may be served in the product is the owner's call.
- **General replies.** Turn 8 and the other rows (78 to 104 exact of 402) are the trunk's and are not touched here.

## Plain summary

A switch on the table's fitter makes a reply start on the value the table found. On a panel written after the order, the reply then holds the told value on 543 of 582 recall questions on all three saved models (501, 489, 489 without it), and the other rows' exact replies are unchanged. Two to four recall replies per model are lost to a value that runs on, 26 never-told replies change without becoming right, and one of them now loops. The price is the wording: every recall reply becomes the bare value, and none equals the reference sentence, where 176 to 182 did before. The find is the counted table's, not the model's.
