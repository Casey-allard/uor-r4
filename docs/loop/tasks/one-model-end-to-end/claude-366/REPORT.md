# REPORT: one-model-end-to-end, claude-366 (the address table's models, opened on the laptop)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3956.

The order asked for the models trained with the address table to be exported to integers, served on the laptop as the saved models are, and read on everyday turns beside the saved model of the same seed, on a panel written after the bars. The served form is built (artifact schema `uor-r4.lut-stack/11`), the three missing seeds are trained, and all six address models are opened and read. **The E8 arm reads "Not called": its three seeds give 6, 8 and 2 more good everyday replies of 235 than the saved models, a mean of +5.3 against a bar of +10. The token arm reads "Not called" too: 0, 3 and 9 more, a mean of +4.0.** Told values and "not told" stay inside the guards on every seed (told is 1 lower on two token models). Reading a prompt runs at about nine tenths of the saved model's tokens a second with the E8 rows (4,421 to 3,955 on seed 1); writing a reply is about the same (1,692 to 1,756; -2% to +4% over the three E8 pairs). In the nine-turn conversation turn 2 is right on every address model (the saved seed-1 model has it wrong); turns 6 and 8 still fail.

## What did not work, first

1. **The E8 arm does not clear the bar.** D is +6, +8 and +2, so M is +5.3; "Adds" needs +10 and "Does not add" is under +3. Seed 3 gains 6 exact replies and loses 8.
2. **Each arm makes one failure better and another worse.** With the E8 rows, replies TRAIN never gives the turn fall (64, 61, 69 to 52, 50, 54 of 235) and replies that never end rise (25, 29, 23 to 31, 32, 36). With the token rows it is the other way round: replies that never end fall (25, 29, 23 to 15, 10, 10) and replies TRAIN never gives the turn rise (64, 61, 69 to 74, 77, 73). The float pass showed the same signs before this unit (ORDER).
3. **Turns 6 and 8 of the conversation are wrong on every address model.** "What is two plus three?" is answered "18 is bigger.", "73 is bigger." or "18." "I'm tired today." gets a line of sympathy with the wrong feeling, a loop, "73 is bigger." or "Your hometown is druvalk is 18."
4. **The token arm does not clear the bar either.** D is 0, +3 and +9, so M is +4.0. Seed 1 gains 7 exact replies, loses 4, and its G does not move.
5. **The gain the order cited from the float pass was mostly not on everyday replies.** On the float scoring pass's own 1,108 prompts the served e8-s1 artifact is exact on 213 rows and the served p-s1 artifact on 185, a gain of 28; in float the same two models are exact on 215 and 187, also 28. So the export carries the float gain whole. But 21 of the 28 are "Say exactly this" rows of invented words (10 to 31 of 107) and 6 are value rows (59 to 65 of 509); on the 413 other rows the gain is 1 (116 to 117). The order's "exact everyday replies in float, of 520" counted the copy rows as everyday. The served model already copies through turn memory (88 of 89 repeat rows exact on the fresh panel, every model) and states values through the counted table, so the panel cannot show that gain. Read at seed 1 only: the other seeds' float samples are in the store and were not fetched.
6. **The row check does not pin where in the step the rows are read.** With the read moved before the layer's second half, e8-s2 differs from its trained model on 586 of 40,719 scored rows; moved after the Hamming read, on 598; in its committed place, on 562. The check ranks the committed place best by only 24 to 36 rows. No committed test pins the place: it rests on the review's line-by-line read against the trainer.
7. **One pod failed before it trained anything (instrument, not a reading).** Lane b's first pod (`try-2`) reached the job's 2,400-second limit in the pre-run scoring pass on a slow host (CPUs at 41% of their rated clock; the other lanes took about 1,211 s for the same pass). `token-s1` was never trained there. About $1.13 was spent for nothing; the seed was rerun on a new pod as `try-6`, and the sheet carries the run note.
8. **The first timing attempt failed.** The reader refuses `--serve-again` with `--time-steps`; the kit's timing script now serves a timed row once, so a timing's replies are not the panel's replies.
9. **The rows cost size and reading speed.** An artifact with the table is 42,938,944 bytes against 19,345,344 without. Reading a prompt falls by about a tenth with the E8 rows and by 4% to 11% with the token rows (table below).
10. **Low items from the exact-head review are left open.** `e8_cell` takes a slice and not an eight-wide array; `AddressImage::checked` has no cap on layers where the file reader has one; `set_decision_trace` accepts a stack that holds the table though the trace's replay does not apply it; the snapshot tool does not refuse a non-finite row (it becomes 0 or the bound) and leaves the rows file behind when the snapshot's own write fails; no committed test drives the exporter's `run()` or pins "an artifact of today reads byte for byte as today" as a unit test (the evidence is the real exports: `b-s2`'s snapshot exports to p-s2's artifact byte for byte); `the_240_cells_are_the_240_roots` checks the test's own helper only.
11. **Not run.** The float crate's GPU tests run only on a pod: 22 passed at the snapshot pin `b036715ac6982df1b8479274703d2272e8fb6966`; the head differs from that pin in tests and documents only. The pod-side negative gate for the float decode's tie rule was not run. `cargo clippy` and `cargo fmt --check` were not run on the laptop. Eight integration targets fail on this laptop with this branch and with `main`'s own code alike (seven load local files that are not here; `m5_full_path_cost_controls` flags the same functions on `main`).
12. **Schema `/11` has no stack contract.** The contracts under `conformance/stack/` are another lab's; none is added here.
13. **One of the sheet's checks was replaced.** The exporter refuses float readouts, so the check that the rows matter is the laptop contrast: the same artifact with an all-zero table (table below).
14. **Two owner readings are open.** Whether a learned table addressed by a fixed hash of the input token may be served in a product (CHARTER §3.1); the E8 arm's address is the state's own E8 cell. And whether reading and counting the served replies on the laptop, as development research that supports no milestone, sits inside §3.6. Neither blocks this unit: nothing here is served as a product.

## What changed

- **The rows in integers** (`crates/uor-r4-integer/src/fbs_address.rs`, new): the rows file (magic `UORADR01`, a 28-byte header, 23,592,988 bytes for these models), `e8_cell` (the nearest of E8's 240 roots to an 8-wide block, in integers, walls and ties included), `token_cell` (the fixed hash of seed, layer, block and input token) and `AddressImage`, which refuses a table of the wrong size, a ragged width or a value past the bound.
- **The artifact** (`stack/format.rs`, `stack/address.rs`, `stack/mod.rs`): schema `uor-r4.lut-stack/11` adds one section for the table. An artifact without the table is written and read as schema `/10`, byte for byte as before.
- **The served step** (`stack/session.rs`): after each layer's Hamilton block and before the Hamming read, each position adds one row per 8-wide block of its residual, the row its address names. The same line stands in the timed step.
- **The exporter** (`crates/uor-r4-integer/src/bin/fbs-export.rs`): `--address-rows FILE` binds a rows file to the snapshot it was written beside (sum, arm, width, depth); a snapshot that names rows is refused without the file.
- **The float side** (`crates/uor-r4-float-train/src/e8_address.rs`, `fbs_export.rs`, `bin/fbs-snapshot.rs`): the snapshot tool writes the rounded rows file and its note beside the snapshot, and the float decode names the served cell (tested against the integer cell on the pod).
- **Tests** (`stack/tests.rs`, `fbs_address.rs`, `e8_address.rs`): 17 new tests in the integer crate and 3 in the float crate. No change to a trained model, the counted table or the instrument's reader source.
- **The sheet** ([one-model-end-to-end-address-seeds-2026-11-13.md](../../../../harness/prep/one-model-end-to-end-address-seeds-2026-11-13.md)): three training lanes and the snapshot passes, with the non-author fitness reads and two run notes.

## Measurements

**The panel.** Seed 23, written 2026-11-13T15:53:47Z by claude-356's writer, after the order's commit (14:12:07Z); its sums were committed in `10015dfb252de5cb679a6d226e17cd6188ad26d8` before any reply was read ([panel-23.sha256](panel-23.sha256)). 984 rows written, 8 equal to an older panel's, 976 read: 235 everyday, 568 told questions, 84 never-told questions, 89 repeat rows. 226 of the 235 everyday turns stand in TRAIN word for word. Every model is served with the counted table as claude-365 merged it, the whole conversation as the prompt, and read once.

**The bar.** G is the order's count: everyday replies that are the reference or another reply TRAIN gives the same turn, and end. D is G beside the saved model of the same seed.

| Model | G of 235 | Saved G | D | The reference | Another TRAIN reply | A reply TRAIN never gives | Never ends | Told, of 568 | Never told says so, of 84 |
|---|---|---|---|---|---|---|---|---|---|
| p-s1 (saved) | 145 | | | 65 | 80 | 64 | 25 | 545 | 82 |
| p-s2 (saved) | 144 | | | 67 | 77 | 61 | 29 | 544 | 82 |
| p-s3 (saved) | 142 | | | 65 | 77 | 69 | 23 | 545 | 82 |
| e8-s1 | 151 | 145 | +6 | 73 | 78 | 52 | 31 | 545 (0) | 82 (0) |
| e8-s2 | 152 | 144 | +8 | 71 | 81 | 50 | 32 | 545 (+1) | 82 (0) |
| e8-s3 | 144 | 142 | +2 | 63 | 81 | 54 | 36 | 545 (0) | 82 (0) |
| token-s1 | 145 | 145 | 0 | 68 | 77 | 74 | 15 | 544 (-1) | 82 (0) |
| token-s2 | 147 | 144 | +3 | 71 | 76 | 77 | 10 | 545 (+1) | 82 (0) |
| token-s3 | 151 | 142 | +9 | 73 | 78 | 73 | 10 | 544 (-1) | 82 (0) |

One reply of each model answers a turn TRAIN does not hold and is in none of the columns. **E8 arm: M = (6 + 8 + 2) / 3 = +5.3, guards hold: Not called. Token arm: M = (0 + 3 + 9) / 3 = +4.0, guards hold: Not called.**

**Everyday rows, each address model beside its saved model**, of 235.

| Model | Exact: saved to address | Newly exact | Lost | Never ends: saved to address | Same reply word for word |
|---|---|---|---|---|---|
| e8-s1 | 65 to 73 | 10 | 2 | 25 to 31 | 160 |
| e8-s2 | 67 to 71 | 8 | 4 | 29 to 32 | 165 |
| e8-s3 | 65 to 63 | 6 | 8 | 23 to 36 | 158 |
| token-s1 | 65 to 68 | 7 | 4 | 25 to 15 | 165 |
| token-s2 | 67 to 71 | 7 | 3 | 29 to 10 | 168 |
| token-s3 | 65 to 73 | 9 | 1 | 23 to 10 | 173 |

**What the replies say** (e8-s2 beside p-s2). Newly right: "Do you know the capital of Greece?" was "The capital of Greece is Saturday." and is "The capital of Greece is Athens."; "Tell me a word that means the opposite of fast." was "The opposite of fast is spelled t-r-e." and is "The opposite of fast is slow."; "What is the day after Saturday?" was "… is white." and is "… is Sunday."; "What is the opposite of up?" looped and is "The opposite of up is down."; "Make a sentence with the word rain." looped and is "The rain fell on the roof all night." Lost: "Can you say red and green?" was "Red and green." and is "Red and and and…"; "What day comes after Sunday?" was "It's Monday." and is "It's Sunday." Never ends: "Spell the word milk." gives "red is spelled c-a-a-a…"; "Give me four animals, please." gives "Here are four animals: Here are are are…"; "What sound does a lion make?" gives "A new hat! That's the sound a a a…". Long "Say exactly this" rows whose reference is cut at the reply cap count as everyday by the fixed method, for saved and address models alike.

**Where the everyday replies fail, by what the reply needs** (the 235 everyday rows grouped by the form of the user's turn, told from its words, so approximate; good replies as G counts them).

| What the reply needs | Rows | Saved p-s1, p-s2, p-s3 | e8-s1, e8-s2, e8-s3 | token-s1, token-s2, token-s3 |
|---|---|---|---|---|
| No word of the turn (thanks, goodbyes, greetings, feelings, the rest) | 116 | 113, 113, 113 | 114, 113, 114 | 114, 113, 114 |
| A word of the turn itself (its first letter, its spelling, a sentence with it, words to repeat, the thing in a piece of news) | 45 | 13, 13, 14 | 13, 14, 12 | 12, 13, 14 |
| A fact about the turn's word (a capital, a colour, an animal's sound or legs, the next day, an opposite, a list of a class) | 45 | 13, 12, 10 | 17, 19, 12 | 13, 15, 16 |
| Work on the turn's numbers (add, count from a to b, the bigger of two) | 29 | 6, 6, 5 | 7, 6, 6 | 6, 6, 7 |

Every model is right on 113 or 114 of the 116 rows whose reply needs no word of the turn, and on 29 to 39 of the 119 rows whose reply does. Most of either table's gain is in the third row (E8: +4, +7, +2 of its +6, +8, +2; token: 0, +3, +6 of its 0, +3, +9). Summed over the three saved models, of the rows of each form: capitals 0 good of 18, adding 0 of 21, lists 0 of 27, legs 0 of 6, spelling 0 of 9, a sentence with a word 1 of 24, the next day 1 of 18, counting 3 of 30.

**The nine-turn conversation** (`uor-chat` at this head, the counted table, turn memory as served). Turns 1, 3, 4, 5, 7 and 9 are the same on all nine models: "Got it.", "A cow says moo.", the phrase back exactly, "That's 73.", "It's 18.", "I'm not sure, you didn't tell me."

| Model | 2. "What is mibtorn?" | 6. "I'm tired today." | 8. "What is two plus three?" |
|---|---|---|---|
| p-s1 (saved) | "felsharp,, quon." | "I'm a helpful assistant that runs on your computer." | "Noted." |
| p-s2 (saved) | "felsharp." | "Feeling sad is druvalk?" | "It's 18." |
| p-s3 (saved) | "Mibtorn is felsharp." | "I'm tired today." | "The day after." |
| e8-s1 | "Mibtorn is felsharp." | "I'm sorry you feel lonely. I hope it gets better soon." | "18 is bigger." |
| e8-s2 | "Mibtorn is felsharp." | "I'm sorry you feel feel to to to" (stopped as a short cycle) | "18 is bigger." |
| e8-s3 | "Mibtorn is felsharp." | "73 is bigger." | "73 is bigger." |
| token-s1 | "felsharp." | "I'm sorry you feel bored. I hope it gets better soon." | "18 is bigger." |
| token-s2 | "Mibtorn is felsharp." | "I'm sorry you feel excited!" | "18 is bigger." |
| token-s3 | "Mibtorn is felsharp." | "Your hometown is druvalk is 18." | "18." |

**Tokens a second**, each address model beside its saved model at the same moment on the laptop, on the first 120 prompts of the seed-22 panel (already read; a timed row is served once).

| Pair | Reading a prompt: saved to address | Writing a reply: saved to address |
|---|---|---|
| p-s1, e8-s1 | 4,421 to 3,955 | 1,692 to 1,756 |
| p-s2, e8-s2 | 4,409 to 3,939 | 1,699 to 1,691 |
| p-s3, e8-s3 | 4,444 to 3,991 | 1,826 to 1,795 |
| p-s1, token-s1 | 2,040 to 1,819 | 832 to 735 |
| p-s2, token-s2 | 4,362 to 4,064 | 1,656 to 1,689 |
| p-s3, token-s3 | 2,137 to 2,045 | 777 to 806 |

The token-s1 and token-s3 pairs were timed later, with the laptop under heavy load from other programs, so both sides of those two rows run at about half the speed of the rows above; each pair still ran side by side.

**The row check** (done-when 2), before any reply of a model was read: the share of 40,719 scored rows on which the artifact picks its trained model's token. The saved models' shares are 0.9911 (p-s1, 364 rows differ), 0.9908 (p-s2, 375) and 0.9860 (p-s3, 572); the order's floor is 0.9760. The last two columns are the same artifact with an all-zero table.

| Model | Artifact SHA-256 | Rows that differ | Share | Differ, all-zero table | Share |
|---|---|---|---|---|---|
| e8-s1 | `d8b3f4513bb606a20dd274441c9a296d8ec31a37a905a30d54e5097821693bea` | 526 | 0.9871 | 4,438 | 0.8910 |
| e8-s2 | `3f8bf9be7025c4c22169035bea11704451f982eac82ef8b895abaac918b06cb7` | 562 | 0.9862 | 4,347 | 0.8932 |
| e8-s3 | `5e7670aacbaf5d21ea282dcc5ad79f3ebb4221f00c0aa6c8e3555599971cb608` | 568 | 0.9861 | 4,848 | 0.8809 |
| token-s1 | `8dcb9a2beafc4af62f4d8fc660842b838fc64fc45bf9c35f5285124816fdf3b4` | 410 | 0.9899 | 7,277 | 0.8213 |
| token-s2 | `821d4cce358073a2d06892c40e5fc2e9e2378daa2f24610eec1bbe3191754465` | 395 | 0.9903 | 7,165 | 0.8240 |
| token-s3 | `e9706b631a823e1fb26e469598263e3b8eb7268a169613e01b2b2825962813ea` | 384 | 0.9906 | 6,579 | 0.8384 |

Every address artifact is above the floor and within 0.005 of the saved models. `b-s2` (the same recipe trained at this unit's pin with no table) snapshots to p-s2's artifact byte for byte, `f4667934627e99d709533f30505058fdf07912ddf3cfc9ade6ded5f415e8c680`.

**Does serving lose the float gain?** A check outside the bars, on the float scoring pass's own 1,108 prompts for seed 1 (each prompt its own window, no counted table, 48 tokens). The served e8-s1 artifact gives the float model's reply on 910 rows and is exact on 213 (float: 215). The served p-s1 artifact is exact on 185 (float: 187). Served, the address model gains 28 exact replies over the saved model (48 newly exact, 20 lost); in float the gain is also 28. The address model ends 879 replies of 1,108 where the saved model ends 998. By the kind of the reference (told from its words, so approximate):

| Kind of reference | Rows | Served exact: saved p-s1 | Served exact: e8-s1 | Float exact: e8-s1 | Never ends: saved | Never ends: e8-s1 |
|---|---|---|---|---|---|---|
| "Say exactly this" (invented words) | 107 | 10 | 31 | 32 | 15 | 31 |
| A told value | 509 | 59 | 65 | 64 | 62 | 110 |
| "Not told" | 79 | 0 | 0 | 0 | 10 | 43 |
| Everything else | 413 | 116 | 117 | 119 | 23 | 45 |

On the fresh panel, served as the order says, replies equal to the reference word for word by kind: repeat rows 88 of 89 on every model; told questions 181, 192, 182 of 568 saved and 197, 195, 200 with the E8 rows; never-told 30 of 84 on every model (82 say "not told" in other words).

**The float scoring pass** (GPU, 1,108 rows, no counted table; replies right and ending on time, the pod's own count; item 5's 215 for e8-s1 is the laptop's exact count of the same samples, one row lower, and that difference was not traced): e8-s1 216, e8-s2 218, e8-s3 254, token-s1 195, token-s2 219, token-s3 221; the saved models 187 (p-s1), 187 (p-s2), 224 (p-s3).

**Compute.** Seven pods, one RTX 5090 each, all down, about $6.63 in all: lane a (`e8-s1`) $1.96; lane b's first pod (`try-2`, failed before training, item 7) about $1.13; lane b's rerun (`try-6`, `token-s1`) $1.59; lane c (`token-s3`) $1.71; three snapshot passes $0.09, $0.07 and $0.08. Every pod went down on its own four checks with its watcher armed first; the last (`vast-60527529`) at 17:48:12Z. Everything else ran on the laptop.

## Checks run at the head

| Check | Result |
|---|---|
| `cargo test --locked --offline -p uor-r4-integer --features serving-wasm --lib`, at the head | 618 passed, 0 failed, 1 ignored |
| The exporter's own tests (`--bin fbs-export`) | 5 passed |
| The float crate's GPU tests, on the snapshot pod at pin `b036715ac6982df1b8479274703d2272e8fb6966` | 22 passed, 0 failed; among them the float decode against the integer cell on the shared blocks and 16,384 drawn ones (a quarter within 3 steps of zero), and the served image against the rounded rows |
| Negative gates, each rule set aside in turn and restored: hold-32, note-alone, odd-flip, record, row-index, rows-sum, schema, step-read, tie-order, tie-side, timed-read, token-hash, every-layer | each test fails, as it must |
| Negative gates for the review's tests: the eighth-place tie, a file with no layer, a ragged file, a ragged table, another position's token | each test fails, as it must; with another position's token `the_served_step_adds_the_address_rows` still passes, which is why the token test was added |
| An artifact of today: `b-s2`'s snapshot exported at this head | p-s2's artifact byte for byte |
| The rebuilt reader on the seed-22 panel; the rebuilt `uor-chat` on the conversation | p-s2's 975 replies and nine turns id for id as claude-365's files |
| The kit's count on claude-365's seed-22 replies | G 137, 134, 136 of 236, the order's numbers |
| The row check and the all-zero contrast, for each address artifact, before its replies | table above |
| `uor-harness prep-check` on the sheet | FIT |

**Review.** Two rounds of fresh reviewers (`delegate:reviewer`, no part in the work), given the order's rules and the code diff and nothing else. Round one read `78705230ef176f325e890d901ce881893c1d6d9b`; the findings of its four reads were taken in `b036715ac6982df1b8479274703d2272e8fb6966`, the snapshot pin. Round two read the head `890cf908affa27967fdb1541042613809ddbf6af` in three slices: the rows module; the stack's section and the served step; the exporter and the float side. Each returned a pass with low items: no wrong served result, no rule break. Its test findings were taken in `ad316b248849117ba1a0a1ce5dbece4d701c0da5` (tests only: the eighth-place tie, the no-layer and ragged cases isolated, the token arm reading its own position's token); the rest are item 10 above. The reviewers ran no GPU test.

Head binaries (`local/claude/claude-366/bin/SHA256.txt`, built at `f5353b9b45ca83afbe3335d8e94b9398ee2cfc5a`; later commits change tests and documents only): exporter `f685044c…`, chat `68af59df…`, table tool `baa26ebc…`, reader `f7c92f56…`, row check `5d876aea…`. Counted table: SHA-256 `420e71c3b4a6fe0c1f958131f9c42f4b5064b90b49af19cb1bbc0860c33b53da`.

## Skills used

None.

## Still open

- **Replies that never end.** 31, 32 and 36 of 235 everyday replies with the E8 rows (25, 29, 23 saved); 15, 10 and 10 with the token rows.
- **Replies TRAIN never gives the turn.** 52, 50 and 54 of 235 with the E8 rows and 74, 77 and 73 with the token rows (64, 61, 69 saved): a wrong fact, a wrong day, a reply to another question.
- **Turns 6 and 8 of the conversation.** The model's own reply on a turn the counted table leaves alone, on every model.
- **The reply that needs the turn's own word or number: 119 of the 235 everyday rows, 29 to 39 good on every model.** This is the gap that is left on everyday turns, and the address table moves it by -1 to +8 rows a seed. The model writes the right frame and the wrong filling ("The capital of Greece is Saturday.", "7 plus 7 is 14.", "rain starts with the letter D."). 226 of the 235 turns stand in TRAIN word for word, the exact turn of a failed form 17 to 265 times, and a run reads TRAIN 9.1 times.
- **Sizing for the next unit, by count on this panel after it was read (kit files, not a bar).** A reply of this kind depends on two tokens of the turn at once, the question word and the key word. With the turn's own TRAIN entry held out: the set-overlap read `main` already has (`stack/prompt_reply.rs`), keyed on the last turn, is good on 84 of 235 and on 11 of 57 rows of nine fact forms, and makes the model's own error ("Do you know the capital of Greece?" answered "Berlin is the capital of Germany."); letting each unordered pair of the turn's tokens vote for the replies it was seen with is good on 57 of those 57. The same vote is wrong on small talk ("Thanks for the help." answered "Okay, I have all of them."), where the model alone is right on 113 of 116. The next unit (claude-367) puts the two together: pair experts counted from TRAIN, addressed by the two tokens themselves, beside the trunk.
- **Not the next step: one trunk with both address tables, or more seeds of either.** Both tables key on one token or on a cell of the learned state; neither forms the pair.
- Item 5's split by kind at seeds 2 and 3.
- **A test that pins the read's place in the step**, and the review's low items (item 10).
- The two owner readings (item 14).

## Plain summary

Six models trained with a learned address table were exported to integers and read on the laptop, on 976 fresh rows, beside the saved models. Neither kind of table clears the bar fixed before any reply was read: models with the table addressed by the state's E8 cell give 6, 8 and 2 more good everyday replies of 235 than the saved models (a mean of +5.3), those with the table addressed by the input token 0, 3 and 9 more (a mean of +4.0); "Adds" needed +10, so both arms read "Not called". Told values and "not told" stay within one reply of the saved models. The models still write the right sentence with the wrong word in it when a reply needs a fact about a word of the turn. Counts on this panel, made after it was read, say such a reply depends on two tokens of the turn taken together, which neither table forms; the next unit builds that.
