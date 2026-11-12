# REPORT: one-model-end-to-end, claude-354

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3944.

A laptop-only hunt, approved by the owner on 12 Nov ("Yes, hunt it"), for why the laptop copies still give other replies than their trained models on about a quarter of the rows. No pod, no training, no float pass. The five models are the four sealed stage 3 models of [claude-352](../claude-352/REPORT.md) and arm r's export; no artifact and no reader code is changed.

**No second reader fault was found.** The laptop copy and the trained model choose the same token on 98.4% to 99.0% of the panel's 40,719 scored rows, and over all rows they are right nearly as often (table 1). They part where the reader's first two choices are nearly tied (tables 2 and 5), which is what rounding in the integer reader would give. The rounding itself is not measured, and 17 to 82 differing rows a model are not explained that way. One differing token in a reply of about fifteen is enough to make the whole reply read differently.

**What the models say first.** The first seven replies of the panel, in panel order, right or wrong, where the place read's laptop copy (p-s2) leaves its trained model. A long repeat is cut with "…". "Rank" is the place of the trained model's token among the laptop reader's 4,096 choices at that token; "gap" is how much farther it is than the reader's own choice, in the reader's distance units (a typical row's first choice leads its second by 503,070).

| Asked (the stated value) | Trained model | Laptop copy | First differing token | Trained / laptop | Rank | Gap |
|---|---|---|---|---|---|---|
| (no value asked) | wushodeer buldreeststst… | wushodeer buldreeststast stoudhutlur ramdark / Assia s s s… | 11th | "st" / "ast" | 2 | 2,875 |
| "Where did I grow up?" (Jaistchukfield) | You grew up in Jaistant: You grew up in in in… | You grew up in Jaistantelly ba know. | 9th | ":" / "elly" | 11 | 3,643 |
| "What was fotpimbrud again?" (waspoumbroud) | It's b all of them. | It's boof. | 4th | " all" / "oof" | 2,583 | 24,529 |
| (no value asked) | That's great news about the camera! | That's great news about the like like like… | 8th | " camera" / " like" | 14 | 12,377 |
| (no value asked) | The capital of Norway is blue. | The capital of Norway is M's's's… | 9th | " blue" / " M" | 2 | 2,228 |
| (the stated value is folfain) | It's S all them. | It's white. | 3rd | " S" / " white" | 5 | 2,972 |
| (no value asked) | zourkferknat / Assh | zourkferknat thanksvaibim pourkferknat | 8th | " / " / " thank" | 2 | 797 |

Neither side is right in any of the seven. Most differing replies are like these: both replies are already poor, and the two readers fall to different poor continuations.

| Gate | Reading | Status |
|---|---|---|
| G1 | The table of step 1 for all five models is under Measurements. | **met** |
| G2 | No fault was found, so there is no change and no test of one. | **does not apply** |
| G3 | No fault was found, so no model was re-read with a changed reader. | **does not apply** |
| G4 | How far each model is from 0.10, and what is not explained, is under Measurements and Still open. | **met** |

## What did not work, first

- **Nothing is repaired.** Every reply the laptop gave before this unit it gives now. The share of replies that differ from the trained model's is unchanged: 0.27, 0.25, 0.18 and 0.22 for the four stage 3 models and 0.21 for arm r, against a bar of 0.10.
- **The bar is not expected to be met by this export on replies of this length.** Table 1's rates, multiplied out over each reply's length (the second check under table 3), give 0.18 to 0.26 of the replies differing somewhere.
- **Rounding is the likely cause, not a measured one.** The near-tie pattern is what integer rounding gives. It is also what a small systematic error in the reader would give. Telling the two apart takes the trained model's own vectors at the differing rows, which no saved file holds.
- **Where the two differ, the trained model is right more often than the copy:** on 377 rows against 281 with the turn memory on (the place read's seed 3: 95 against 47), and on 1,543 against 1,087 with it off.
- **The control loses stated values on the laptop and the place read does not** (79 to 69, against 183 to 183, 210 to 209 and 173 to 170). The control's loss is 14 replies lost and 4 gained; that lean is not explained beyond table 4.
- **Near a tie, the kinds of row do not differ at one rate** (table 5): first tokens on 10% to 30% of such rows, later tokens on 36% to 54%. Not explained.
- **The first review found a gap in the program.** Rows of a reply that opened before their window, or at its first token, would have been read with no reply start on the laptop, while the trained model's pass reads its reply table there. The program now counts such rows and keeps them out of the comparison. This panel has none at the 192 window, so no number changed.
- claude-352's registered reading stays NOT_RUN. No bar or status is changed.

## What changed

- A new laptop program, [`row-picks`](../../../../labs/one-model-end-to-end/instrument/src/bin/row-picks.rs), in the item's instrument crate. It reads integer artifacts only.
  - **Rows:** it cuts a panel into the windows the float scoring pass cuts, steps the laptop reader through each with the true text before every row, and writes the reader's choice at every scored row beside the pod's saved choice (`rows-p-memory.json`, `rows-p-no-memory.json`). It refuses to write unless its row positions and targets equal the pod's file. A row whose reply opened before its window, or at its first token, is labelled `carried` and kept out of the counts of a run that marks reply starts: the laptop session cannot be told of such a start.
  - **Replies:** for each decoded reply that is not the trained model's, it feeds the prompt and the tokens both replies share, checks that the reader's next token is the saved laptop reply's and the table's nearest row, and writes the rank and the gap of the trained model's token.
- No change to the reader (`crates/uor-r4-integer`), to any artifact, or to the float side.

## Measurements

Laptop only. The reader is built from `1666dbc12`; `row-picks` SHA-256 `080bd465b2b3d82e5e1312e436ccf7a9e9ef9cf6df9d84a2b04c3ec7d3d171e6`. The panel is claude-352's no-line panel (tokens SHA-256 `b8ad9fe4…5fe4`, the SHA the pod's score names), read at the 192 window the pod scored at. The trained models' numbers are the pod's saved files; the float column of table 1 equals the pod's own receipts (34,625 for the control, 33,543 for arm r).

All five artifacts carry both turn-memory tables (13,195 opening keys and 166,841 reply keys; arm r 7,414 and 140,003). No scored row of this panel is carried at the 192 window (0 of 40,719 on all ten runs), so every row is compared.

**1. Row by row, with the true text before each row (40,719 rows a model).**

| Model | Turn memory | Rows chosen differently | Share | Right: trained model | Right: laptop copy |
|---|---|---|---|---|---|
| Control (n-s2) | on | 504 | 1.2% | 34,625 | 34,618 |
| Place read (p-s2) | on | 670 | 1.6% | 34,838 | 34,802 |
| Place read (p-s1) | on | 520 | 1.3% | 34,968 | 34,971 |
| Place read (p-s3) | on | 597 | 1.5% | 34,823 | 34,775 |
| Arm r (r-s2) | on | 424 | 1.0% | 33,543 | 33,535 |
| Control (n-s2) | off | 1,521 | 3.7% | 31,303 | 31,286 |
| Place read (p-s2) | off | 2,003 | 4.9% | 32,531 | 32,412 |
| Place read (p-s1) | off | 1,676 | 4.1% | 32,643 | 32,509 |
| Place read (p-s3) | off | 1,888 | 4.6% | 32,683 | 32,555 |
| Arm r (r-s2) | off | 983 | 2.4% | 30,281 | 30,223 |

With the memory off the first token of a reply is the weak row: 630 to 713 of the 3,586 first tokens differ on the stage 3 models (283 on arm r), against 31 to 34 (90 on arm r) with it on. The turn memory decides the first token; without it the model is near a tie there. So the turn memory hides differences at the first token. Whether it causes any is read under table 3: on the 46 to 204 replies a model that can be compared, it could have a part in 0 to 7.

**2. The differing rows are the rows where the reader's first two choices are nearly tied** (turn memory on). Each cell is rows chosen differently of rows in the band; the band is how far the reader's first choice leads its second.

| Model | under 1,000 | 1,000 to 4,000 | 4,000 to 16,000 | 16,000 to 64,000 | 64,000 and up |
|---|---|---|---|---|---|
| Control (n-s2) | 210 of 364 | 173 of 521 | 78 of 797 | 38 of 1,542 | 5 of 37,495 |
| Place read (p-s2) | 191 of 322 | 235 of 601 | 177 of 1,041 | 65 of 5,844 | 2 of 32,911 |
| Place read (p-s1) | 164 of 340 | 184 of 555 | 118 of 703 | 50 of 3,805 | 4 of 35,316 |
| Place read (p-s3) | 211 of 389 | 209 of 740 | 147 of 999 | 29 of 4,858 | 1 of 33,733 |
| Arm r (r-s2) | 184 of 330 | 133 of 519 | 69 of 697 | 35 of 1,385 | 3 of 37,788 |

About half differ in the nearest band, about a third in the next, and 1 to 5 rows of more than 32,000 in the last. A typical row's lead is 476,967 to 584,048 (the median); a differing row's is 1,411 to 2,331. On the differing rows the trained model's token is the reader's second choice on 263, 285, 261, 272 and 236 rows, its third to fifth on 132, 181, 158, 155 and 111, its sixth to twentieth on 73, 122, 84, 123 and 59, and beyond on 36, 82, 17, 47 and 18. Rows where the target is copied from the prompt differ least: 13 to 88 of 10,556.

**3. The G1 table: where a decoded reply first leaves the trained model's** (1,108 replies a model, turn memory on, as the models are served). Every one of the 1,249 replies was fed again; each time the reader gave the saved laptop reply's token, and that token was the table's nearest row.

| Model | Replies that differ | First leaves at the 1st token | 2nd to 4th | Later | Trained model's token is the reader's 2nd choice | 3rd to 5th | 6th to 20th | Beyond | Gap (median) |
|---|---|---|---|---|---|---|---|---|---|
| Control (n-s2) | 300 (0.27) | 16 | 109 | 175 | 146 | 78 | 52 | 24 | 3,831 |
| Place read (p-s2) | 275 (0.25) | 21 | 73 | 181 | 146 | 67 | 32 | 30 | 4,716 |
| Place read (p-s1) | 197 (0.18) | 17 | 57 | 123 | 110 | 59 | 21 | 7 | 5,982 |
| Place read (p-s3) | 245 (0.22) | 20 | 52 | 173 | 131 | 58 | 42 | 14 | 5,161 |
| Arm r (r-s2) | 232 (0.21) | 82 | 100 | 50 | 122 | 59 | 39 | 12 | 2,055 |

What the trained model says at that token with its turn memory off:

| Model | The same token as with the memory on | Not comparable: its memory-off reply had already left | The laptop's token | A third token |
|---|---|---|---|---|
| Control (n-s2) | 67 | 233 | 0 | 0 |
| Place read (p-s2) | 96 | 179 | 0 | 0 |
| Place read (p-s1) | 39 | 151 | 6 | 1 |
| Place read (p-s3) | 80 | 163 | 1 | 1 |
| Arm r (r-s2) | 202 | 28 | 0 | 2 |

Only the replies whose memory-off reply had not already left can be compared: 67, 96, 46, 82 and 204. Among them the turn memory could have a part in the first differing token in 0, 0, 7, 2 and 2. The other 233, 179, 151, 163 and 28 say nothing either way.

With the turn memory off on both sides, 479, 519, 462 and 523 of the stage 3 models' 1,108 replies differ, 219 to 287 of them at the first token. Arm r has no memory-off laptop replies.

Two checks tie table 3 to table 1 (laptop kit, not committed; see Checks):

- Where the trained model's reply follows the panel's text up to the differing token, that token is one of table 1's rows. There are 59, 68, 51, 58 and 104 such replies, and every one is among table 1's differing rows.
- Table 1's rates, taken on the rows of the decoded replies and multiplied out over each laptop reply's length, give 202, 292, 221, 274 and 226 differing replies, against 300, 275, 197, 245 and 232 seen. Four agree within 13%. The control has 98 more than the rates give. A guess, not a measurement: its replies leave the panel's text most often, and the rates are taken on the panel's text only.

**4. Who is right where they differ** (turn memory on).

| Model | Rows chosen differently | Trained model right | Laptop copy right | Neither |
|---|---|---|---|---|
| Control (n-s2) | 504 | 69 | 62 | 373 |
| Place read (p-s2) | 670 | 106 | 70 | 494 |
| Place read (p-s1) | 520 | 58 | 61 | 401 |
| Place read (p-s3) | 597 | 95 | 47 | 455 |
| Arm r (r-s2) | 424 | 49 | 41 | 334 |

The trained model is right on 377 of these rows and the laptop copy on 281: a net 96 rows of 203,595 (0.05%) to the trained model. The lean is clearest on the place read's seed 3 (95 against 47). With the memory off it is 1,543 against 1,087. Stated values of 562 at 192, trained then laptop, with the replies lost and gained: control 79 and 69 (14 lost, 4 gained); place read 183 and 183 (8, 8), 210 and 209 (6, 5), 173 and 170 (6, 3).

**5. By kind of row, under a lead of 4,000 and at or over it** (turn memory on; a count by the laptop kit on the program's row files). Each cell is rows chosen differently of rows.

| Model | Lead under 4,000: first token | 2nd to 4th | Later | Copied | Lead 4,000 and up: first token | 2nd to 4th | Later | Copied |
|---|---|---|---|---|---|---|---|---|
| Control (n-s2) | 31 of 142 | 180 of 357 | 172 of 386 | 44 of 199 | 0 of 3,444 | 32 of 10,389 | 89 of 26,001 | 19 of 10,357 |
| Place read (p-s2) | 30 of 178 | 195 of 371 | 201 of 374 | 41 of 100 | 4 of 3,408 | 63 of 10,375 | 177 of 26,013 | 47 of 10,456 |
| Place read (p-s1) | 32 of 252 | 157 of 310 | 159 of 333 | 20 of 76 | 2 of 3,334 | 72 of 10,436 | 98 of 26,054 | 23 of 10,480 |
| Place read (p-s3) | 30 of 301 | 163 of 351 | 227 of 477 | 33 of 132 | 2 of 3,285 | 49 of 10,395 | 126 of 25,910 | 33 of 10,424 |
| Arm r (r-s2) | 88 of 290 | 144 of 323 | 85 of 236 | 10 of 29 | 2 of 3,296 | 32 of 10,423 | 73 of 26,151 | 3 of 10,527 |

The copied rows are also counted in the three columns by token place. With a lead of 4,000 or more, under 1% of the rows of every kind differ. Arm r's 90 differing first tokens are near ties too (88 of them). Under 4,000 the rate is not the same for every kind: first tokens differ on 10% to 30% of such rows, second to fourth tokens on 45% to 53%, later tokens on 36% to 54%, copied rows on 22% to 41%. Why is not known.

**What this rules out.**

- A wrong window, a cut row or a shifted row: the rows equal the pod's, 40,719 of 40,719 on ten runs.
- A reply start the laptop was not told of: no scored row of this panel is carried.
- The turn memory, as far as it can be compared: table 3's second table, on the 46 to 204 comparable replies a model.
- A fault touching 1% or more of the rows that are not near a tie, in any kind of row: table 5's right half. The rows that still differ there (0 to 177 a kind a model) and the far cases stay open.
- A fault of the place read alone: the control and arm r, which have none, differ nearly as much (504 and 424 rows against 520 to 670).

**What it does not rule out.** A small systematic error in the reader that only tips near ties: it would look like tables 2 and 5. The uneven rates of table 5's left half are a reason to keep that open.

**What a float trace would have to record** to go further: at table 1's differing rows, the trained model's final vector before the nearest-token choice, and each read's kept position. The first gives the size of the rounding a coordinate, which is what would tell rounding from a small systematic error; the second says how many of the far cases (rank beyond the twentieth, 17 to 82 rows a model) are a read that kept another position. The float scoring pass has a query cache option; this unit did not check what it holds. Either way that is a pod run and is not asked for here.

## Checks run at the head

| Check | Result |
|---|---|
| `cargo test --locked --release --bin row-picks --features integer-reader` (instrument crate) | 6 passed, 0 failed |
| the same tests with three lines broken on purpose (the window's back clamp, the end of the copied pair, the carried range), then restored | the three tests that name them fail; restored, 6 pass |
| `cargo build --locked --release --bin row-picks --features integer-reader`, source as at `1666dbc12` | built; the SHA-256 above |
| rows carried, all ten row runs | 0 of 40,719 |
| the ten row runs: positions and targets against the pod's files | equal on every run (the program refuses otherwise) |
| the nine reply runs: the reader's token against the saved laptop reply's | equal on every differing reply (the program refuses otherwise) |
| the float column of table 1 against the pod's score receipts | equal where a receipt is on the laptop (control 34,625; arm r 33,543) |
| the program's breakdown against the kit's first count of the same files | equal |
| a second full run (19 runs, 57 files) against the first, made before a rename of one identifier | equal byte for byte |
| a third full run by the build above against the second, made before the review's changes | the nine reply files equal byte for byte; the ten row files equal once the new fields are set aside |
| `uor-harness ratchet d32`, `d23`, `d25` | no additions |

Not run: anything on a pod, any float pass, the integer crate's tests (its code is not changed). The two tying checks under table 3, the memory-off column of table 3, table 5 and the stated-value counts are counts by the laptop kit (`jq` files under `local/claude/claude-354/` and `local/claude/export-repair/`), not by committed code. `uor-harness loop pr-check` runs after this commit and its result is in the review record on the PR.

## Skills used

None beyond the project loop. Two fresh reviewers, subagents of this session, read the program and this report apart. Both asked for changes in the first round: the program's carried rows and three test gaps, and wording here that said more than was measured. No number changed. Two more fresh reviewers read the changes and approved with notes; the wording notes are applied here and the program's four low notes are under Still open.

## Still open

- **The whole-reply bar.** 0.10 of replies differing measures the length of a reply as much as the copy. A row-by-row check with the true text (this program, about a minute a model on the laptop) with the right counts beside it measures the copy. Whether a later sheet registers that is the owner's call; nothing is changed here.
- **The control's lean** (14 stated values lost, 4 gained) and its 98 replies beyond what the rates give.
- **The far cases:** 17 to 82 differing rows a model where the trained model's token is beyond the reader's twentieth choice. A read keeping another position would look like this; it needs the float trace above.
- **Rounding or a small systematic error:** not told apart here, and the uneven near-tie rates of table 5 are not explained. The same trace would say.
- **Windows whose first token is a reply token:** this panel has none at 192. A panel or window that has them needs a session that can be told of a reply start before its first token; until then the program sets those rows aside.
- **Four low notes of the second code review, not applied,** so that the program is as reviewed and as run (`1666dbc12`). None changes a count.
  - No test drives the run loop's own marking calls; the tests pin `turn` and `carried`, which the loop uses.
  - The program's header says a carried reply opened before its window; the code also counts one that opens at the window's first token, as it should.
  - The header names no receipt field (`compared` and `turn_memory` are new).
  - A choices array shorter than its rows in the trained model's file stops the run with a message about ids, not about lengths. Nothing is written either way.
- **A float reading at the 256 window** does not exist (carried from claude-353).
- claude-352's open items on the place read itself are untouched.

## Plain summary

No fault was found in the laptop reader. Token by token the laptop copies choose what their trained models choose on about 99 rows in 100, and over all rows they are right nearly as often. Where the two differ, the reader was nearly undecided between two tokens. That is what the laptop's integer rounding would do, but the rounding was not measured, a small systematic error in the reader would look the same, a few dozen rows a model are not explained either way, and on the differing rows the trained model is right somewhat more often (377 rows against 281). Because a reply is many tokens long, about one reply in four ends up worded differently, usually where both replies were already poor. Nothing in the laptop reader was changed. The laptop numbers of claude-353 stand as readings of the models: at the full window the place read says the stated value on 238, 269 and 223 of 562 and the control on 76. On this evidence, what stands between this and a model worth talking to is mainly the trained model's own replies, not the copy.
