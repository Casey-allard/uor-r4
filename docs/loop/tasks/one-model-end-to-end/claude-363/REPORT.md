# REPORT: one-model-end-to-end, claude-363 (a reply is served again when its found value was never offered)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3953.

[claude-362](../claude-362/REPORT.md) opened the first row of every reply, which said the found value on more recall questions but replaced every sentence the model had written with the bare value. This unit serves each reply with the counted table as merged, and serves it a second time, first row open, only when the asks set a place and no row of the first reply was offered it. Read as replies on three saved models, on a panel written after the order, the order's row is **Adds**. No training, no pod and no artifact change.

Every figure below is in `local/claude/claude-363/` in the main checkout (kit files, kept out of the repository by the project's rule on run output).

## What did not work, first

1. **The conversation loses one turn and gains one.** On the nine-turn conversation (seed 2), "Say exactly this: trelvon dask pimmerholt zane" was answered "trelvon dask pimmerholt zane" and then ran on into "Assistant." with the merged table; served again it answers "18.". The table makes a false find on that turn. The model's own reply never opened it, so the merged reply was untouched; serve-again speaks it. The order expected this. Turn 2 ("What is mibtorn?") goes from "Mibtorn?" to "felsharp.". Turns 6 and 8 are wrong in both arms ("Feeling sad is druvalk?"; "What is two plus three?" gives "It's 18.").
2. **A reply served again is the bare value, not a sentence.** " 99." where the reference is " That's 99."; " kodrairk." for " Groustkokree is kodrairk.". None of the 37, 49 and 62 replies served again equals its reference, so the count of exact recall replies does not rise: it stays where the merged table left it. On p-s2 and p-s1 some run on after the value: 35 of 37, 29 of 49 and 62 of 62 are the value and a full stop and nothing else; the others say more (" plirrelbra, rotferk."; " 15, 15, 15, 15, 15."; " 99 is bigger."; " 875, trekgeend is 790."). The scorer counts them as holding the value.
   Three replies on p-s1 already held the value in a broken sentence, with no row offered the place, and were served again all the same: " Kurbroul again? / A 253." became " 253."; " Brousjakshen again? / A 45." became " 45, 4, 4, 4, 4, 4."; " Klindnid again? / A 54." became " 54, 5.". No reply that equalled its reference was changed on any model.
3. **The table's own misses are untouched.** The same 27 recall questions are wrong on all three models, so they are the table's and not the models': 14 are answered with the stored "not told" reply although the fact was told, and 13 state another value (" Your dog is named Jeelzairk." for " Your dog is named Tinlan."; " You love yellow." for " Your favorite food is salad."; " It's 52749." for " Nasatreet is 5274."). Two more on p-s2, one on p-s1 and none on p-s3 are the model's own (" Your lucky number is 24Q me").
4. **Never-told questions are unchanged: 20 of 75 are answered with a value.** " You work as a green.", " You grew up in Dublin.", " You live in Pumburg." where the reference says the fact was not told. 55 of 75 say so, 15 of 75 equal the reference, in the merged and the serve-again arm alike. No never-told reply was served again.
5. **Replies that do not end on the end mark:** 34, 34 and 25 of 980 (merged: 36, 47, 35). The logs count the end mark only; they do not say how each of the others stopped.
6. **Two of three gains are outside the predicted range.** Predicted +7% to +10% of the recall questions; read +6.6%, +8.3%, +11.1%.
7. **A reply served again costs two serves, and the chat withholds its words until it knows which reply stands.** The prompt is read again from its start (a session of a place read cannot be saved and restored). With the switch on, `StackChat` emits nothing while the first reply is served and then emits the standing reply's ids in order, so a caller's stream starts late and an emit error arrives after the reply is in the history. The time was not measured.
8. **One file changed that the order did not name:** `crates/uor-r4-integer/src/stack/hamming.rs` (6 lines). The session reaches the place read only through `HammingStack::place_advance`, which now passes the switch on.
9. **The instrument's serve-again wiring is pinned by the reply files, not by a unit test** (the instrument review's one medium finding). Its decision rule and its flag's refusals are tested. Moving the "held at the first step" read to another step, or dropping a row's `again` and `first` fields, would break no test. What stands in for one: on both panels and all three models, every row served again carries as `first` exactly the merged arm's reply for that row, every other row is the merged arm's reply with no extra field, the receipt's count is the count of marked rows, and every reply served again holds the value. The chat's path has an end-to-end test. Not tried: `--serve-again 1` with `--mark-reply 0` (every reading here marks the reply).
10. **The first builder stopped after one deliverable of three** (its context budget). The instrument flag went to a second builder and the chat flag was written in the main session.
11. **The panel's left-out rule is stricter than claude-362's.** 7 of 987 prompts equal a prompt of one of five older panels (seeds 14, 16, 18 and two earlier ones) and were left out before any read; claude-362 compared with two panels. All 7 are general rows.
12. **`cargo clippy -D warnings` and `cargo fmt --check` are not green on this crate at base or head** on the laptop toolchain, as in claude-362; they were not used as gates.

## What changed

- **Reader** (`fbs_snapshot.rs`, `stack/hamming.rs`, `stack/session.rs`): `IntegerStackSession::set_open_reply_start(bool)`, off by default. With it on, the row of the assistant marker's last token is an open row when the asks set a place there; no later row is opened by it, and it does nothing when the asks find nothing or answer with the stored reply. `reset()` keeps the setting.
- **Chat** (`stack/chat.rs`, `bin/uor-chat.rs`): `StackChat::set_serve_again(bool)` and `uor-chat --place-serve-again 0|1`. A reply is served; if the register held a place after the step that chose its first id and no step of it was offered a place, the conversation starts over at the same prompt with the session switch on, and the second reply stands. The switch is turned off again whether the second serve succeeds or fails. There is never a third serve. The flag is refused without `--place-asks` and with `--prompt-ids` or `--serve`. The `--say` record and each script turn carry `served_again` when the flag is 1.
- **Instrument** (`raw-id-replies.rs`): `--serve-again 0|1`, the same rule per row on a fresh session. A row served again is written as `{"i","ids","again":true,"first":[…]}`; the receipt gains `serve_again.replies_served_again`. Refused without `--place-asks` and with `--place-rows 1`, `--time-steps 1`, `--count-visits 1` or `--trace-control 1`.
- **Tests:** four reader tests, one decision-rule test and one end-to-end chat test in the integer crate; two in the instrument.

## Measurements

**The fresh panel.** The reading panel's writer at `seed=19`, started 2026-11-13T11:11:51Z, after the order's commit (11:10:30Z) and its draft PR (opened 11:10:35Z); first read 11:45:27Z, once per arm (`panel-19.log`, `read19-merged.log`, `read19-open.log`, `read19-again.log`). 987 prompts written, 980 read: 557 recall questions and 423 other rows, of which 75 are never-told questions and 348 are general. Models: claude-358's p-s2, p-s1, p-s3. Table: claude-359's fit (`claude-362/asks-off.json`, sha `6e5e985c…`); the *open* arm uses claude-362's table (`asks-on.json`). Reader sha `7dc63b80…`, built at `4b38c27b45bb071f94a2c064bea23d2c7897e1af`. Scorer: `claude-361/bin-pin/whole-word-recall`; all 557 references score right.

| Seed 19 panel | p-s2 | p-s1 | p-s3 |
|---|---|---|---|
| V, merged (of 557) | 491 | 483 | 468 |
| V, serve again | 528 | 529 | 530 |
| V gain | +37 (6.6%) | +46 (8.3%) | +62 (11.1%) |
| V, claude-362's always-open arm | 528 | 529 | 530 |
| E, merged | 166 | 162 | 157 |
| E, serve again | 166 | 162 | 157 |
| E, always-open arm | 0 | 0 | 1 |
| Other rows exact, merged and serve again (of 423) | 92 | 92 | 139 |
| Replies served again | 37 | 49 | 62 |
| … whose first reply already held the value | 0 | 3 | 0 |
| … whose second reply holds the value | 37 | 49 | 62 |
| … that are never-told or other rows | 0 | 0 | 0 |
| Recall replies right with the merged table and wrong served again | 0 | 0 | 0 |
| Recall replies still wrong | 29 | 28 | 27 |
| Never-told (of 75): say so / exact, merged and serve again | 55 / 15 | 55 / 15 | 55 / 15 |
| Never-told: say so, always-open arm | 60 | 60 | 60 |
| Replies ending on the end mark (of 980): merged / serve again | 944 / 946 | 933 / 946 | 945 / 955 |

**The row, by the order's bar.** V gain at least 5% of the recall questions (28 of 557) on all three: 37, 46, 62. E(again) at least 90% of E(merged) on all three: 100%. Other rows' exact not down by more than 1%: unchanged. **Adds.**

The always-open arm reads 5 more never-told questions right than the other two arms; this unit does not explain that.

**Replies served again, p-s2, every sixth** (first reply, second reply, reference):

| Row | First | Served again | Reference |
|---|---|---|---|
| 18 | " Ditplund?" | " 99." | " That's 99." |
| 119 | " Choulfur?" | " plirrelbra, rotferk." | " That's plirrelbra." |
| 215 | " Groustkokree?" | " kodrairk." | " Groustkokree is kodrairk." |
| 401 | " Breetstructions." | " 6613." | " Breetstam is 6613." |
| 530 | " Klourbraist." | " finfustger." | " It's finfustger." |
| 769 | " I'll remember all of that." | " 933." | " That's 933." |
| 974 | " Draidbye!" | " treentrain." | " Draidnod is treentrain." |

**The seed 18 panel, already read by claude-362** (984 rows, 582 recall questions; reported, no bar). Merged: V 501, 489, 489, the bytes of claude-362's reply files (`cmp`). Serve again: V 544, 544, 543; E 182, 178, 176, as merged; other rows' exact 78, 80, 104, as merged; replies served again 44, 59, 56, all recall questions.

**The nine-turn conversation, p-s2** (`chat-merged.body.json`, `chat-again.body.json`; `uor-chat` at this head):

| Turn | User | Merged table | Serve again |
|---|---|---|---|
| 1 | Keep these in mind: druvalk is 73, mibtorn is felsharp, quonzet is 18. | Got it. | Got it. |
| 2 | What is mibtorn? | Mibtorn? | felsharp. (served again) |
| 3 | What does a cow say? | A cow says moo. | A cow says moo. |
| 4 | Say exactly this: trelvon dask pimmerholt zane | trelvon dask pimmerholt zane / Assistant. | 18. (served again) |
| 5 | What is druvalk? | That's 73. | That's 73. |
| 6 | I'm tired today. | Feeling sad is druvalk? | Feeling sad is druvalk? |
| 7 | Tell me what quonzet was. | It's 18. | It's 18. |
| 8 | What is two plus three? | It's 18. | It's 18. |
| 9 | What is my name? | I'm not sure, you didn't tell me. | I'm not sure, you didn't tell me. |

## Checks run at the head

Run at `4b38c27b45bb071f94a2c064bea23d2c7897e1af`, the last commit that changes code; later commits add this report and the notebook row.

| Check | Result |
|---|---|
| `cargo test --locked --offline -p uor-r4-integer --features serving-wasm --lib` | 584 passed, 1 ignored |
| `… --lib place_asked` (the four new reader tests among them) | 15 passed |
| `… --bin uor-chat` | 9 passed |
| The reader's `opened ||` set aside (the builder's run) | the opening test fails; restored, it passes |
| The chat's switch-on line set aside | the end-to-end test fails: the reply starts on 40, not 31 |
| The chat's switch-off line set aside | the end-to-end test fails: the next plain reply starts on 31 |
| Instrument: `cargo test --locked --offline --features integer-reader --bin raw-id-replies` (the builder's run) | 17 passed |
| Release builds of `uor-chat` and `raw-id-replies` | exit 0 |
| Seed 18, merged table, this reader against claude-362's reply files | equal bytes, three models |
| Seed 18, claude-362's table, this reader against claude-362's reply files | equal bytes, three models |
| `cargo clippy -D warnings`, `cargo fmt --check`, laptop toolchain | not green at base or head (negative 12) |

G1: the reader tests and the two byte compares. G2: the reader tests. G3: the end-to-end chat test and its two set-aside runs. G4: the kit's logs with times. G5: this report and the notebook row.

**Reviews, both fresh, 13 Nov, at `4b38c27b45bb071f94a2c064bea23d2c7897e1af`.** Neither reviewer ran the tests (each reached its context budget reading); the test lines above are the main session's runs at that head. The integer-crate review approved with four low findings: a failed second serve leaves a part-made second reply in the history, and an emission error with the switch on arrives after the whole reply is in the history (both now stated in `set_serve_again`'s doc, a comment-only commit after the head above); `--place-serve-again 0` without `--place-asks` is refused although it asks for the default (left: the flag names a table it needs); the end-to-end test's last block would also pass under a rule that looked at the first step only (the decision-rule test covers that case). The instrument review approved with one medium finding (negative 9) and three low: the receipt counts second serves and not changed replies (on these panels one reply of 307 served again came back unchanged, seed 18, p-s2); `--mark-reply 0` untried (negative 9); several lines run past 100 columns.

**The report review, fresh, 13 Nov,** read this report against the kit's logs at `f04d8d0e976dcd3a6e1a99cfd074bcccb0b6b0c1` and asked for five text changes, all made: the three p-s1 replies that held the value and were served again (negative 2 and the summary); "two, one and none" in negative 3; the draft PR's opening time (11:10:35Z) and the order's time named as its commit time; 423 other rows stated as 348 general and 75 never-told; "do not end on the end mark" in negative 5. It found the bar's arithmetic, the measurement table, the seed 18 paragraph, the times' order and the conversation in agreement with the logs. It did not check that the 27 missed questions are the same on all three models (the main session's `comm` of the three lists) or the seven sample replies.

## Skills used

None.

## Still open

- Whether a count-fitted table may be served in the product (§3.1) is the owner's call; this unit serves the same table as claude-359 and adds no fitted part.
- The table's misses: 14 told facts answered "not told", 13 wrong places and 20 of 75 never-told questions answered with a value, per model. [claude-361](../claude-361/ORDER.md)'s trained key is running on two pods.
- A request to repeat a phrase is where the table's false find now costs a turn. Pointing the place register at the phrase in the turn being answered is the next unit.
- A reply served again is a bare value. Whether to keep the model's lead-in ("That's", "It's") before the spoken value was not tried.
- General replies (lists, counts, simple facts) are untouched.

## Plain summary

When the model is asked for something it was told, a lookup table finds where the answer sits in the conversation. Often the model's own sentence already says it. Sometimes the model says something else and never reaches the answer. Before, we had two choices: leave those replies wrong, or force every reply to start with the bare answer, which threw away every good sentence. Now the reader lets the model answer first, and only if the found answer was never used does it start that one reply over from the answer. On a fresh set of 557 recall questions, right answers went from 491, 483 and 468 to 528, 529 and 530 on three models, and every reply that already matched its reference stayed exactly as it was. (Three replies on one model that held the answer inside a broken sentence were started over too, and came back as the bare answer.) What is still wrong is the table: about 27 questions where it finds nothing or the wrong spot, 20 of 75 questions about things never told that it answers anyway, and one turn of a test conversation ("say exactly this…") where it now blurts a number.
