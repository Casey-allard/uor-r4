# REPORT: one-model-end-to-end, claude-360 (what a key must see: the find by frame offset)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3950.

claude-359 showed that an exact find lets the saved models state a told value on 491 to 513 of 578 fresh questions, against 314 to 325 by their own read, and that the exact find is a counted table the model does not own. This unit asks how much of the table's input the find needs, so that the next unit builds a trained key over that input from the project's geometry. The counted find was fitted again on TRAIN under 30 cuts of its input and its census was read on both panels. Laptop only: no model was read, nothing was trained, no pod was used.

Every figure below is in `local/claude/claude-360/cuts-1/` and `local/claude/claude-360/cuts-2/` in the main checkout, one log per cut (kit files, kept out of the repository by the project's rule on run output).

**How to make any row again.** `place-asks ablate --tokens local/claude/claude-349/train-off/train/tokens.u16 --kit local/claude/claude-351/train-places.json --held local/claude/claude-349/panel-off/train/tokens.u16 --held-kit local/claude/claude-352/panel-kit/panel-places.json --held local/claude/claude-357/fresh16/panel-off/train/tokens.u16 --held-kit local/claude/claude-357/fresh16/panel-places.json --slots 2,3` at this head. The fit reads no clock and no random source.

## What did not work, first

1. **Nothing in the model changed.** This unit measured. The trained key is not built; the unit names what to build.
2. **The half of the key the model trains today is aimed at an offset that, past the census's link gate, finds nothing when read alone.** The token before a place, read alone, finds 0 of 562 and 0 of 578 true places: 559 and 572 questions are refused and 3 and 6 get nothing. The trained read (claude-351, `place_find`) keys a place by two tokens: the token at offset 1 in each ask head's first half, which is the half the place pull moves the question rows toward (`place_pull_bits`), and the token at offset 2 in the rest, which is matched against the code of the token before the asking row and which the place pull does not move. An earlier draft of this report said the trained key reads offset 1 only. That was a misreading of `place_codes`, which gives one code a row; `place_find` joins two rows' codes. It is corrected in this report.
3. **A read that finds the place does not know when nothing was told.** The cut that finds the most true places (offsets 2 and 3) sets a place on 49 of 79 and 35 of 71 never-told questions. With the question cut to its last three tokens, 47 to 79 of 79 and 44 to 71 of 71 never-told questions get a place, by the offsets kept. Among the cuts that find more than 500 true places on the first panel, the fewest never-told questions with a place there are 13 and 15 of 79: the whole frame with the whole question, and with its last eight tokens. On the second panel those two cuts give 19 and 17 of 71, and three narrower frames with the whole question do about as well as the whole frame's 19: offsets 1 to 4 give 18, and offsets 1 to 3 give 19 with or without the turn's end. A cut that sets fewer places than these on never-told questions does it by finding little or nothing. Four cuts find no place at all and so set none: no offset, offset 1 alone, the turn's end alone, and the question's last token. Offset 1 with the turn's end sets 10 and 3 and finds 207 and 190 true places. The question's last two tokens set 10 on the second panel (19 on the first, which is not fewer) and find 273 and 281.
4. **The find is still counts, not geometry (CHARTER §3.1).** A cut fit is the same counting on less input. It says which input holds the information; it does not say a trained key will learn it.
5. A cut is a mask over offsets and a count of a question's last tokens. Whether the order of the offsets matters was not measured: a fit that reads the tokens at offsets 2 and 3 without telling the two offsets apart does not exist in this unit.
6. "True place" is a census of the find, not a reply. claude-359 showed that 58 to 83 found places per model and panel are never spoken.
7. No bar was registered. The second batch of 12 cuts was chosen after the first 18 were read, to locate where the refusal fails. The first panel chose table v4's features in claude-359; the second panel is the clean check. The two panels agree on which cuts find most true places and which find few; the order of close rows can flip (offset 2 against offset 3: 514 and 511 on the first panel, 530 and 532 on the second). On the refusal they differ in places: the question's last eight tokens against the whole question set 15 against 13 places on never-told questions on the first panel, and 17 against 19 on the second.
8. The `ablate` mode fits before it reads the held splits, so a held kit that does not belong to its tokens is refused only after the fit has run.
9. One new test indexed the table's two row shapes wrongly and failed on its own slip; it was fixed before the first code commit. The review's two cheap findings (a loose `--slots` parser, untested `ablate` refusals) are fixed at this head.

## What changed

- `crates/uor-r4-integer/src/place_asks.rs`: `Cut { slots, question_last }`, with `Cut::ALL` for no cut; `fit_cut`, which `fit` now calls with no cut; `AsksIndex::with_cut` and the reader's `asked`; the masked frame, applied after the link gate on the raw token before the place. A dropped offset is filled as an offset with no token is, so it is the same at every candidate. The fit's labels still read the whole question.
- `crates/uor-r4-integer/src/bin/place-asks.rs`: the `ablate` mode. It fits under the cut of `--slots` and `--question-last`, prints the census of each `--held` and `--held-kit` pair under the same cut, and writes no file.
- A table file and a session carry no cut: `PlaceAsks` has no new field, `AsksIndex::new` reads everything, and nothing outside the tests and `ablate` calls `with_cut` or `fit_cut`.

## Measurements

A frame is eight offsets around a candidate place: 1 to 5 are the tokens before it, nearest first; 6 to 8 are its turn's last tokens, the last first. Each row is one fit on TRAIN (table v4's input, SHA-256 of the tokens `e25775f3…`) and its census on the first panel (562 questions with a told value, 79 never told) and the second panel (578 and 71). The four counts are true place / wrong place / "not told" raised / nothing. "Never told with a place" is the count of never-told questions on which a place was set; lower is better.

### The frame, with the whole question

| Offsets kept | First panel, of 562 | Never told with a place, of 79 | Second panel, of 578 | Never told with a place, of 71 |
|---|---|---|---|---|
| all eight (table v4) | 539 / 14 / 8 / 1 | 13 | 560 / 9 / 7 / 2 | 19 |
| none | 0 / 0 / 559 / 3 | 0 | 0 / 0 / 572 / 6 | 0 |
| 1 | 0 / 0 / 559 / 3 | 0 | 0 / 0 / 572 / 6 | 0 |
| 2 | 514 / 40 / 5 / 3 | 64 | 530 / 40 / 4 / 4 | 48 |
| 3 | 511 / 42 / 6 / 3 | 64 | 532 / 35 / 7 / 4 | 60 |
| **2, 3** | **546 / 11 / 4 / 1** | 49 | **565 / 5 / 4 / 4** | 35 |
| 1, 2 | 493 / 41 / 25 / 3 | 46 | 514 / 41 / 19 / 4 | 36 |
| 1 to 3 | 519 / 21 / 21 / 1 | 33 | 544 / 13 / 17 / 4 | 19 |
| 1 to 4 | 528 / 16 / 17 / 1 | 29 | 551 / 10 / 14 / 3 | 18 |
| 1 to 5 | 538 / 14 / 9 / 1 | 30 | 560 / 9 / 8 / 1 | 26 |
| 6 to 8 (the turn's end) | 0 / 0 / 559 / 3 | 0 | 0 / 0 / 572 / 6 | 0 |
| 1 and 6 to 8 | 207 / 195 / 157 / 3 | 10 | 190 / 215 / 167 / 6 | 3 |
| 1, 2 and 6 to 8 | 498 / 52 / 11 / 1 | 36 | 517 / 47 / 10 / 4 | 28 |
| 1 to 3 and 6 to 8 | 526 / 26 / 9 / 1 | 24 | 555 / 16 / 4 / 3 | 19 |

### The question, with all eight offsets

| Question tokens read | First panel, of 562 | Never told with a place, of 79 | Second panel, of 578 | Never told with a place, of 71 |
|---|---|---|---|---|
| the last 1 | 0 / 0 / 0 / 562 | 0 | 0 / 0 / 0 / 578 | 0 |
| the last 2 | 273 / 50 / 232 / 7 | 19 | 281 / 45 / 244 / 8 | 10 |
| the last 3 | 520 / 22 / 18 / 2 | 47 | 532 / 24 / 22 / 0 | 44 |
| the last 4 | 530 / 18 / 12 / 2 | 41 | 549 / 15 / 13 / 1 | 33 |
| the last 6 | 527 / 14 / 19 / 2 | 22 | 546 / 10 / 22 / 0 | 21 |
| the last 8 | 539 / 14 / 4 / 5 | 15 | 561 / 10 / 5 / 2 | 17 |
| all (table v4) | 539 / 14 / 8 / 1 | 13 | 560 / 9 / 7 / 2 | 19 |

### Both cut

| Offsets kept, question tokens read | First panel, of 562 | Never told with a place, of 79 | Second panel, of 578 | Never told with a place, of 71 |
|---|---|---|---|---|
| 1, the last 1 | 0 / 0 / 0 / 562 | 0 | 0 / 0 / 0 / 578 | 0 |
| 1 and 2, the last 2 | 242 / 53 / 262 / 5 | 43 | 251 / 39 / 280 / 8 | 30 |
| 1 and 2, the last 3 | 518 / 38 / 1 / 5 | 79 | 533 / 32 / 1 / 12 | 71 |
| 1 to 3, the last 3 | 529 / 29 / 1 / 3 | 79 | 551 / 24 / 1 / 2 | 71 |
| 1 to 3, the last 4 | 534 / 19 / 6 / 3 | 65 | 545 / 13 / 19 / 1 | 51 |
| 1 to 3, the last 6 | 509 / 22 / 25 / 6 | 52 | 534 / 15 / 25 / 4 | 38 |
| 1 to 5, the last 3 | 527 / 22 / 11 / 2 | 60 | 536 / 22 / 19 / 1 | 54 |
| 1 to 5, the last 6 | 530 / 16 / 13 / 3 | 38 | 543 / 11 / 24 / 0 | 24 |
| 1 to 3 and 6 to 8, the last 3 | 518 / 27 / 14 / 3 | 66 | 530 / 25 / 22 / 1 | 55 |
| 1 to 3 and 6 to 8, the last 6 | 506 / 28 / 24 / 4 | 40 | 535 / 14 / 26 / 3 | 30 |

### What the counts say

1. **Where the value is: the two tokens before the token before it.** Offsets 2 and 3 alone find 546 of 562 and 565 of 578 true places, more than all eight offsets (539 and 560). Offset 2 alone finds 514 and 530. Offset 1 alone finds none, and so does the turn's end alone. Offset 1 is not without use beside the others: added to offsets 2 and 3 it finds fewer true places (519 and 544) and sets fewer places on never-told questions (33 and 19 against 49 and 35). A reading, not a count: the token before a value is one of a few link words that stand before every value.
2. **How much of the question the find needs.** Its last two tokens find 273 and 281. Its last three find 520 and 532, with 22 and 24 wrong places. Its last eight find 539 and 561, with 14 and 10 wrong, which is what the whole question finds (539 and 560, with 14 and 9 wrong). So most of the find is in the last three tokens and all of it is in the last eight.
3. **Whether anything was told is another read.** Places set on never-told questions, of 79 and of 71: 13 and 19 with the whole frame and the whole question, 15 and 17 with the question's last eight tokens, 22 and 21 with its last six, 47 and 44 with its last three; 30 and 26 with offsets 1 to 5 only, 49 and 35 with offsets 2 and 3 only. With the question cut to its last three tokens and the frame to offsets 1 and 2, or 1 to 3, every never-told question gets a place (79 and 71). On the second panel a frame of offsets 1 to 3 or 1 to 4 with the whole question refuses as well as the whole frame (19 and 18 against 19); on the first panel it does not (33 and 29 against 13).
4. **The read the model trains today, read from the code and not measured here.** `place_find` asks at one row of the question with two tokens: the asking token's learned question code in each ask head's first half, and the code of the token before it in the rest. It keys a candidate place the same way: the token at offset 1 in the first half and the token at offset 2 in the rest. The place pull moves only the question rows, toward the code of the token at offset 1. The place read refuses the question carry (`--hamming-place` refuses `--hamming-carry-question`), so its question is those two tokens and no more. The census reads offset 1 after its own gate: every candidate it scores is already a row after a link token, so "offset 1 alone finds nothing" says what offset 1 adds past that gate. The trained read has no such gate: `place_pick` scores every position of the earlier user turns, and the half aimed at offset 1 is the only part of its key that can keep the find to rows after a link token. So the learned half can do the gate's job, and by the census it adds nothing past the gate. Past the gate, what can tell one place from another in the trained read is the other half, a match of one question token with the token at offset 2 by the token's own code. The nearest cut of the census, offsets 1 and 2 with the question's last two tokens, finds 242 of 562 and 251 of 578; with the question's last three tokens it finds 518 and 533. That cut is not the trained read: the asking row need not be among the question's last two tokens, and the trained read's second half matches a token with itself where the table counts any pairing.

### What the next unit builds

- **The key's two halves name offsets 2 and 3, in place of 1 and 2.** At a candidate place the first half of each ask head holds the code of the token two before it, and the rest the code of the token three before it. Offset 1 then leaves the key, and with it the only part of today's key that can keep the find to rows after a link token. How that gate is kept is the next unit's first design point: every cut of this census had the gate, so no row here says what a read without it finds.
- **The mechanism is the one in the code.** A code is the signs of a token's point turned by the key's rotation; a head's first half holds one token's code and the rest another's (`place_find`'s join); the compare is Hamming distance inside each ask head's margin. Nothing of a new kind enters. What changes is which tokens the two halves name, and what the place pull moves the question rows toward.
- **The question reads more than two tokens.** The place read refuses the question carry today. The next unit lets the learned half read earlier question tokens through it, up to the eight the census names.
- **The find and the refusal are trained and scored apart.** The find is read on told questions (true and wrong places). The refusal is the ask head's margin, read on never-told questions (places set). A unit that reports one count for both would hide finding 3.
- **Labels.** The place kit already gives each recall reply its place (`PlaceTeach`); the pull moves the question rows toward the new key as it moves them toward today's.
- It is a change to the float trainer, whose tests run on CUDA only: a prep sheet with a non-author FIT, then a pod inside the caps.

## Checks run at the head

Run on the laptop at the head that carries this report; one cargo at a time, `--locked --offline`.

| Check | Result |
|---|---|
| `cargo test -p uor-r4-integer --features serving-wasm --lib --bin uor-chat --bin place-asks` | 573 passed, 1 ignored (library); 9 passed (`uor-chat`); 4 passed (`place-asks`) |
| Instrument: `cargo check --release --features integer-reader` | clean |
| `cargo check -p uor-r4-training -p uor-r4-cpu-train` | clean (run before the review's two parser fixes, which touch the tool only) |
| Negative gate: the reader's frame cut set aside | `a_cut_reader_reads_no_offset_it_drops_and_no_question_token_past_its_count` fails on the two candidates' scores; restored |
| Negative gate: the reader's question cut set aside | the same test fails on the question's tokens; restored |
| Negative gates: the fit's frame cut set aside at true places, then at every candidate | `a_cut_fit_counts_no_token_at_an_offset_it_drops_and_no_question_token_it_does_not_read` fails each time on a row past offset 2; restored |
| Negative gates: the fit's question cut set aside at true places, then at every candidate | the same test fails each time on a row that holds the question's first word; restored |
| Negative gates: `none` mixed with offsets accepted; an offset named twice accepted | `a_cut_is_read_from_its_flags_and_written_back_the_same` fails on `--slots none,6` and on `--slots 1,1`; restored |
| After the first six gates, the working diff against the patch saved before them | equal |
| The table fitted with no cut on TRAIN | SHA-256 `6e5e985cedb7d94305d8dd6d5abb7ef247a7f58787f6ea1997f19745a8de1417`, the bytes of v4 |
| `ablate` with no cut | the census of table v4 as claude-359 measured it: 539 / 14 / 8 / 1 and 13 of 79; 560 / 9 / 7 / 2 and 19 of 71 |
| `ablate --slots 2,3` with the tool rebuilt at this head | the row above: 546 / 11 / 4 / 1 and 49; 565 / 5 / 4 / 4 and 35 |
| `ablate` given the first panel's tokens with the second panel's kit | refused, naming both hashes |

Not run as negative gates: `held` and `same_marks`, whose test reads each function's own result.

## Skills used

None beyond the project loop. The reviewer was a subagent of this session.

## Still open

- The trained key over offsets 2 and 3, the question's reach, and the refusal read apart: the next unit, as written above.
- Whether the two offsets must be told apart (finding 5 of the first section).
- From claude-359, unchanged: a found place that is not spoken (58 to 83 questions per model and panel); a test for the session restore's reset of the spoken token; the three "not told" forms; the reply that runs on after the value.
- Whether a count-fitted table may be served in the product is the owner's call (CHARTER §3.1); this unit serves none.

## Plain summary

The exact find of claude-359 reads eight tokens around each candidate value and the whole question. Cut down piece by piece, it shows that the value is found from the two tokens before the token before it (546 of 562 and 565 of 578), that the token just before it finds nothing when read alone, and that most of the find is in the question's last three tokens (520 and 532) and all of it in the last eight. By its code, which this unit read and did not measure, the read the model trains today learns to match its question to that one token, which can keep it to rows after a link word; past that, it can tell places apart by matching one question token with the token two before the value. Knowing that a fact was never told is a separate read: the cut that finds the place best sets a place on 49 of 79 and 35 of 71 never-told questions, against 13 and 19 for the whole table. The next unit moves the trained key onto the two tokens the census names and lets the question read more than two tokens; the sign codes and the Hamming compare stay as they are.
